pub mod event;
pub mod layout;
pub mod render;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyCode};
use crossterm::execute;
use ratatui::{DefaultTerminal, style::Stylize, text::Line};
use std::time::{Duration, Instant};

use crate::app::layout::AppLayout;
use crate::config;
use crate::swap_info::{
    ProcessSwapInfo, SizeUnits, SwapUpdate, aggregate_processes, convert_swap, get_chart_info,
    get_processes_using_swap,
};
use crate::theme::ThemeType;

pub const LINUX: bool = cfg!(target_os = "linux");

pub fn format_swap_value(value: f64) -> String {
    let rounded = value.round();
    if (value - rounded).abs() < 0.01 {
        format!("{}", rounded as u64)
    } else {
        format!("{:.2}", value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    ProcessList,
    SwapDevices,
    InfoPanel,
}

impl FocusedPanel {
    pub fn next(&self) -> Self {
        match self {
            Self::ProcessList => Self::SwapDevices,
            Self::SwapDevices => Self::InfoPanel,
            Self::InfoPanel => Self::ProcessList,
        }
    }
    pub fn prev(&self) -> Self {
        match self {
            Self::ProcessList => Self::InfoPanel,
            Self::SwapDevices => Self::ProcessList,
            Self::InfoPanel => Self::SwapDevices,
        }
    }
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub display_devices: bool,
    pub vertical_scroll_state: ratatui::widgets::ScrollbarState,
    pub vertical_scroll: usize,
    pub swap_devices_scroll: usize,
    pub swap_size_unit: SizeUnits,
    pub swap_processes_lines: Vec<Line<'static>>,
    pub processes_data: Vec<ProcessSwapInfo>,
    pub selected_index: usize,
    pub show_info_panel: bool,
    pub info_scroll: usize,
    pub last_update: Option<Instant>,
    pub chart_info: SwapUpdate,
    pub aggregated: bool,
    pub current_theme: ThemeType,
    pub time_window: [f64; 2],
    pub chart_data: Vec<(f64, f64)>,
    pub timeout: u64,
    pub visible_height: usize,
    pub devices_split_ratio: f64,
    pub is_dragging: bool,
    pub drag_start_x: u16,
    pub drag_start_ratio: f64,
    pub is_scroll_dragging: bool,
    pub show_help: bool,
    pub focused_panel: FocusedPanel,
    pub info_split_ratio: f64,
    pub is_info_dragging: bool,
    pub layout: AppLayout,
    pub help_scroll: usize,
}

impl App {
    pub fn new() -> Self {
        let cfg = config::load_config();
        Self {
            running: false,
            display_devices: cfg.display_devices,
            vertical_scroll_state: ratatui::widgets::ScrollbarState::default(),
            vertical_scroll: 0,
            swap_devices_scroll: 0,
            swap_size_unit: cfg.size_unit,
            swap_processes_lines: Vec::new(),
            processes_data: Vec::new(),
            selected_index: 0,
            show_info_panel: cfg.show_info_panel,
            info_scroll: 0,
            last_update: None,
            chart_info: SwapUpdate::default(),
            aggregated: cfg.aggregated,
            current_theme: cfg.theme,
            time_window: [0.0, 60.0],
            chart_data: Vec::new(),
            timeout: cfg.timeout,
            visible_height: 0,
            devices_split_ratio: cfg.devices_split_ratio.clamp(0.15, 0.85),
            is_dragging: false,
            drag_start_x: 0,
            drag_start_ratio: 0.70,
            is_scroll_dragging: false,
            show_help: false,
            focused_panel: FocusedPanel::ProcessList,
            info_split_ratio: cfg.info_split_ratio.clamp(0.15, 0.85),
            is_info_dragging: false,
            layout: AppLayout::default(),
            help_scroll: 0,
        }
    }

    #[cfg(target_os = "linux")]
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        execute!(terminal.backend_mut(), EnableMouseCapture)?;
        self.running = true;
        self.swap_processes_lines = self.create_process_lines(self.aggregated);
        self.chart_info = get_chart_info(self.swap_size_unit.to_owned())?;
        self.last_update = Some(Instant::now());

        while self.running {
            if crossterm::event::poll(Duration::from_millis(100))? {
                self.handle_crossterm_events()?;
            }

            if let Some(last_update) = self.last_update
                && last_update.elapsed() >= Duration::from_millis(self.timeout)
            {
                self.chart_info = get_chart_info(self.swap_size_unit.to_owned())?;
                self.update_chart_data();
                self.last_update = Some(Instant::now());
                self.swap_processes_lines = self.create_process_lines(self.aggregated);
            }

            terminal.draw(|frame| self.render(frame))?;
        }
        execute!(terminal.backend_mut(), DisableMouseCapture)?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        execute!(terminal.backend_mut(), EnableMouseCapture)?;
        self.running = true;
        self.swap_processes_lines = self.create_process_lines(self.aggregated);
        self.chart_info = get_chart_info()?;
        self.last_update = Some(Instant::now());

        while self.running {
            if crossterm::event::poll(Duration::from_millis(100))? {
                self.handle_crossterm_events()?;
            }

            if let Some(last_update) = self.last_update
                && last_update.elapsed() >= Duration::from_millis(self.timeout)
            {
                self.chart_info = get_chart_info()?;
                self.update_chart_data();
                self.last_update = Some(Instant::now());
                self.swap_processes_lines = self.create_process_lines(self.aggregated);
            }

            terminal.draw(|frame| self.render(frame))?;
        }
        execute!(terminal.backend_mut(), DisableMouseCapture)?;
        Ok(())
    }

    pub fn update_chart_data(&mut self) {
        let timestamp = self.time_window[1];
        let swap_usage = self.chart_info.used_swap as f64;
        self.chart_data.push((timestamp, swap_usage));
        if self.chart_data.len() > 60 {
            self.chart_data.drain(0..1);
        }
        self.time_window[0] += 1.0;
        self.time_window[1] += 1.0;
    }

    pub fn generete_total_used_title(&self) -> String {
        let total = convert_swap(self.chart_info.total_swap, self.swap_size_unit.clone());
        let used = convert_swap(self.chart_info.used_swap, self.swap_size_unit.clone());
        format!(
            "total: {} | used: {}",
            format_swap_value(total),
            format_swap_value(used)
        )
    }

    pub fn create_process_lines(&mut self, aggregated: bool) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        lines.push(Line::from(vec![
            format!("{:>12}", if self.aggregated { "count" } else { "pid" }).bold(),
            " | ".into(),
            format!("{:30}", "process").bold(),
            " | ".into(),
            format!("{:10}", "used").bold(),
        ]));

        self.processes_data.clear();

        if let Ok(mut processes) = get_processes_using_swap(self.swap_size_unit.clone()) {
            processes.sort_by(|a, b| {
                b.swap_size
                    .partial_cmp(&a.swap_size)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            if aggregated {
                processes = aggregate_processes(processes);
            }

            for process in &processes {
                let process_size = format_swap_value(process.swap_size);

                lines.push(Line::from(vec![
                    format!("{:12}", process.pid).into(),
                    " | ".into(),
                    format!("{:30}", process.name.clone()).into(),
                    " | ".into(),
                    format!("{:10}", process_size).into(),
                ]));
            }
            self.processes_data = processes;
            self.selected_index = self.selected_index.min(lines.len().saturating_sub(1));
        }

        lines
    }

    pub fn cycle_theme(&mut self) {
        self.current_theme = match self.current_theme {
            ThemeType::Default => ThemeType::Solarized,
            ThemeType::Solarized => ThemeType::Monokai,
            ThemeType::Monokai => ThemeType::Dracula,
            ThemeType::Dracula => ThemeType::Nord,
            ThemeType::Nord => ThemeType::Default,
        };
        self.swap_processes_lines = self.create_process_lines(self.aggregated);
        self.save_app_config();
    }

    pub fn toggle_aggregated(&mut self) {
        self.aggregated = !self.aggregated;
        self.save_app_config();
    }

    pub fn toggle_display_devices(&mut self) {
        if LINUX {
            self.display_devices = !self.display_devices;
            self.save_app_config();
        }
    }

    pub fn set_size_unit(&mut self, unit: SizeUnits) {
        self.swap_size_unit = unit.clone();
        #[cfg(target_os = "linux")]
        if let Ok(info) = get_chart_info(unit.clone()) {
            self.chart_info = info;
        }
        #[cfg(target_os = "windows")]
        if let Ok(info) = get_chart_info() {
            self.chart_info = info;
        }
        self.swap_processes_lines = self.create_process_lines(self.aggregated);
        self.save_app_config();
    }

    pub fn change_timeout(&mut self, action: KeyCode) {
        match action {
            KeyCode::Left => {
                self.timeout = self.timeout.saturating_sub(100).max(1);
            }
            KeyCode::Right => {
                self.timeout = self.timeout.saturating_add(100).min(10000);
            }
            _ => {}
        }
        self.save_app_config();
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn save_app_config(&self) {
        config::save_config(&config::AppConfig {
            aggregated: self.aggregated,
            size_unit: self.swap_size_unit.clone(),
            display_devices: self.display_devices,
            theme: self.current_theme,
            timeout: self.timeout,
            devices_split_ratio: self.devices_split_ratio,
            show_info_panel: self.show_info_panel,
            info_split_ratio: self.info_split_ratio,
        });
    }

    pub fn unit_buttons_str(&self) -> &'static str {
        match self.swap_size_unit {
            SizeUnits::KB => "▶KB◀─MB─GB",
            SizeUnits::MB => "KB─▶MB◀─GB",
            SizeUnits::GB => "KB─MB─▶GB◀",
        }
    }

    pub fn aggregate_title_str(&self) -> &'static str {
        if self.aggregated {
            "(a to segregate)"
        } else {
            "(a to aggregate)"
        }
    }

    pub fn theme_title_str(&self) -> String {
        format!("theme (t to change): {:?}", self.current_theme)
    }

    pub fn timeout_title_str(&self) -> String {
        format!(" < {:?}ms > ", self.timeout)
    }
}
