mod swap_info;
mod theme;

use crate::swap_info::{SwapUpdate, aggregate_processes, convert_swap};
use crate::theme::{Theme, ThemeType};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    symbols::Marker,
    text::Line,
    widgets::{
        Axis, Block, BorderType, Chart, Dataset, GraphType, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
};
use std::time::{Duration, Instant};
use swap_info::{SizeUnits, get_chart_info, get_processes_using_swap};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

#[derive(Debug, Default)]
pub struct App {
    running: bool,
    pub vertical_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
    pub swap_size_unit: crate::SizeUnits,
    pub swap_processes_lines: Vec<Line<'static>>,
    pub last_update: Option<Instant>,
    pub chart_info: SwapUpdate,
    pub aggregated: bool,
    current_theme: ThemeType,
    time_window: [f64; 2],
    chart_data: Vec<(f64, f64)>,
    timeout: u64,
    visible_height: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: false,
            vertical_scroll_state: ScrollbarState::default(),
            vertical_scroll: 0,
            swap_size_unit: SizeUnits::KB,
            swap_processes_lines: Vec::new(),
            last_update: None,
            chart_info: SwapUpdate::default(),
            aggregated: false,
            current_theme: ThemeType::Dracula,
            time_window: [0.0, 60.0],
            chart_data: Vec::new(),
            timeout: 1000,
            visible_height: 0,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        self.swap_processes_lines = self.create_process_lines(self.aggregated);
        self.chart_info = get_chart_info(self.swap_size_unit.to_owned())?;
        self.last_update = Some(Instant::now());

        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            if event::poll(Duration::from_millis(100))? {
                self.handle_crossterm_events()?;
            }

            if let Some(last_update) = self.last_update {
                if last_update.elapsed() >= Duration::from_millis(self.timeout) {
                    self.update_chart_data();
                    self.swap_processes_lines = self.create_process_lines(self.aggregated);
                    self.chart_info = get_chart_info(self.swap_size_unit.to_owned())?;
                    self.last_update = Some(Instant::now());
                }
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let theme = Theme::from(self.current_theme);

        let main_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border))
            .title(
                Line::from(" swaptop ")
                    .bold()
                    .fg(theme.primary)
                    .left_aligned(),
            )
            .title(
                Line::from(format!("theme (t to change): {:?}", self.current_theme))
                    .bold()
                    .fg(theme.primary)
                    .right_aligned(),
            )
            .title(
                Line::from(format!(" < {:?}ms > ", self.timeout))
                    .bold()
                    .fg(theme.primary)
                    .centered(),
            )
            .style(Style::default().bg(theme.background).fg(theme.text));

        let main_area = main_block.inner(frame.area());

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(main_area);

        if cfg!(target_os = "linux") {
            let upper_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
                .split(chunks[0]);

            self.render_swap_devices(frame, upper_chunks[0], &theme);
            self.render_animated_chart(frame, upper_chunks[1], &theme);
            self.render_processes_list(frame, chunks[1], &theme);
        } else {
            self.render_animated_chart(frame, chunks[0], &theme);
            self.render_processes_list(frame, chunks[1], &theme);
        }

        frame.render_widget(main_block, frame.area());
    }

    fn update_chart_data(&mut self) {
        let timestamp = self.time_window[1];
        let swap_usage = self.chart_info.used_swap as f64;
        self.chart_data.push((timestamp, swap_usage));
        if self.chart_data.len() > 60 {
            self.chart_data.drain(0..1);
        }
        self.time_window[0] += 1.0;
        self.time_window[1] += 1.0;
    }

    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            // quit
            KeyCode::Esc | KeyCode::Char('q') => self.quit(),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => self.quit(),

            // up and down list
            KeyCode::Char('d') | KeyCode::Down => {
                self.vertical_scroll = self.vertical_scroll.saturating_add(1);
                self.vertical_scroll_state =
                    self.vertical_scroll_state.position(self.vertical_scroll);
            }
            KeyCode::Char('u') | KeyCode::Up => {
                self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
                self.vertical_scroll_state =
                    self.vertical_scroll_state.position(self.vertical_scroll);
            }
            KeyCode::End => {
                self.vertical_scroll = self.swap_processes_lines.len();
                self.vertical_scroll_state =
                    self.vertical_scroll_state.position(self.vertical_scroll);
            }
            KeyCode::Home => {
                self.vertical_scroll = 0;
                self.vertical_scroll_state =
                    self.vertical_scroll_state.position(self.vertical_scroll);
            }

            KeyCode::PageDown => {
                let page_size = self.visible_height.saturating_sub(4);
                self.vertical_scroll = self
                    .vertical_scroll
                    .saturating_add(page_size)
                    .min(self.swap_processes_lines.len().saturating_sub(1));
                self.vertical_scroll_state =
                    self.vertical_scroll_state.position(self.vertical_scroll);
            }
            KeyCode::PageUp => {
                let page_size = self.visible_height.saturating_sub(4);
                self.vertical_scroll = self.vertical_scroll.saturating_sub(page_size);
                self.vertical_scroll_state =
                    self.vertical_scroll_state.position(self.vertical_scroll);
            }

            // change unit
            KeyCode::Char('k') => self.swap_size_unit = SizeUnits::KB,
            KeyCode::Char('m') => self.swap_size_unit = SizeUnits::MB,
            KeyCode::Char('g') => self.swap_size_unit = SizeUnits::GB,

            // aggregate
            KeyCode::Char('a') => self.aggregated = !self.aggregated,

            // change theme
            KeyCode::Char('t') => self.cycle_theme(),

            // change timeout
            KeyCode::Left | KeyCode::Right => self.change_timout(key.code),

            _ => {}
        }
    }
    fn cycle_theme(&mut self) {
        self.current_theme = match self.current_theme {
            ThemeType::Default => ThemeType::Solarized,
            ThemeType::Solarized => ThemeType::Monokai,
            ThemeType::Monokai => ThemeType::Dracula,
            ThemeType::Dracula => ThemeType::Nord,
            ThemeType::Nord => ThemeType::Default,
        };
        self.swap_processes_lines = self.create_process_lines(self.aggregated);
    }

    fn change_timout(&mut self, action: KeyCode) {
        match action {
            KeyCode::Left => {
                self.timeout = self.timeout.saturating_sub(100).max(1);
            }
            KeyCode::Right => {
                self.timeout = self.timeout.saturating_add(100).min(10000);
            }
            _ => {}
        }
    }
    fn quit(&mut self) {
        self.running = false;
    }

    fn create_process_lines(&self, aggregated: bool) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        lines.push(Line::from(vec![
            format!("{:12}", if self.aggregated { "COUNT" } else { "PID" }).bold(),
            " | ".into(),
            format!("{:30}", "PROCESS").bold(),
            " | ".into(),
            format!("{:10}", "USED").bold(),
        ]));

        if let Ok(mut processes) = get_processes_using_swap(self.swap_size_unit.clone()) {
            processes.sort_by(|a, b| {
                b.swap_size
                    .partial_cmp(&a.swap_size)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            if aggregated {
                processes = aggregate_processes(processes);
            }

            for process in processes {
                let mut process_size: String = format!("{:.2}", process.swap_size);
                if let SizeUnits::KB = self.swap_size_unit {
                    process_size = format!("{}", process.swap_size)
                }

                lines.push(Line::from(vec![
                    format!("{:12}", process.pid).into(),
                    " | ".into(),
                    format!("{:30}", process.name).into(),
                    " | ".into(),
                    format!("{:10}", process_size).into(),
                ]));
            }
        }

        lines
    }

    fn render_swap_devices(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // self.visible_height = area.height as usize;
        // let content_height = self.chart_info.swap_devices.len() + 2;

        // self.vertical_scroll = self
        //     .vertical_scroll
        //     .min(content_height.saturating_sub(self.visible_height));
        // self.vertical_scroll_state = self
        //     .vertical_scroll_state
        //     .content_length(content_height)
        //     .position(self.vertical_scroll);

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border))
            .style(Style::default().bg(theme.background))
            .title(Line::from("swap devices").fg(theme.text).left_aligned());

        let mut devices = Vec::new();

        for device in self.chart_info.clone().swap_devices {
            let mut used = format!("{:.2}", device.used_kb);
            if let SizeUnits::KB = self.swap_size_unit {
                used = format!("{}", device.used_kb)
            }
            devices.push(Line::from(format!(
                "{} | {} | {} | {}",
                device.name, device.kind, used, device.priority
            )));
        }

        let process_paragraph = Paragraph::new(devices)
            .alignment(Alignment::Center)
            .block(block);

        frame.render_widget(process_paragraph, area);
    }
    fn render_animated_chart(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let total = convert_swap(self.chart_info.total_swap, self.swap_size_unit.clone());
        let used = convert_swap(self.chart_info.used_swap, self.swap_size_unit.clone());

        let total_used_title: String = match self.swap_size_unit {
            SizeUnits::KB => format!("total avaliable: {} | used: {}", total, used),
            SizeUnits::MB => format!("total avaliable: {} | used: {:.2}", total.round(), used),
            SizeUnits::GB => format!("total avaliable: {:.2} | used: {:.2}", total, used),
        };

        let swap_usage_percent =
            self.chart_info.used_swap as f64 / self.chart_info.total_swap as f64 * 100.0;
        let datasets = vec![
            Dataset::default()
                .marker(Marker::Braille)
                .style(Style::default().fg(theme.primary))
                .graph_type(GraphType::Line)
                .data(&self.chart_data),
        ];

        let chart = Chart::new(datasets)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme.border))
                    .title(
                        Line::from(format!("swap usage {}%", swap_usage_percent.round() as u64))
                            .fg(theme.primary)
                            .bold()
                            .right_aligned(),
                    )
                    .title(Line::from(total_used_title).fg(theme.text).left_aligned())
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.background)),
            )
            .x_axis(
                Axis::default()
                    .style(Style::default().fg(theme.text))
                    .bounds(self.time_window),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().fg(theme.text))
                    .bounds([0.0, self.chart_info.total_swap as f64]),
            );

        frame.render_widget(chart, area);
    }

    fn render_processes_list(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let unit_buttons = match self.swap_size_unit {
            SizeUnits::KB => "▶KB◀─MB─GB",
            SizeUnits::MB => "KB─▶MB◀─GB",
            SizeUnits::GB => "KB─MB─▶GB◀",
        };

        self.visible_height = area.height as usize;
        let content_height = self.swap_processes_lines.len() + 2;
        self.vertical_scroll = self
            .vertical_scroll
            .min(content_height.saturating_sub(self.visible_height));
        self.vertical_scroll_state = self
            .vertical_scroll_state
            .content_length(content_height)
            .position(self.vertical_scroll);

        let bottom_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border))
            .style(Style::default().bg(theme.background))
            .title(
                Line::from("(a to aggregate) (u/d|▲/▼|home/end|pgup/pgdown to scroll)")
                    .fg(theme.text)
                    .right_aligned(),
            )
            .title(
                Line::from(format!("unit (k/m/g to change): {}", unit_buttons))
                    .fg(theme.secondary)
                    .bold()
                    .left_aligned(),
            );

        let process_paragraph = Paragraph::new(self.swap_processes_lines.clone())
            .alignment(Alignment::Center)
            .block(bottom_block)
            .scroll((self.vertical_scroll as u16, 0));

        frame.render_widget(process_paragraph, area);

        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .style(Style::default().fg(theme.scrollbar))
                .thumb_style(Style::default().fg(theme.primary)),
            area,
            &mut self.vertical_scroll_state,
        );
    }
}
