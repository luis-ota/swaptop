mod swap_info;

use std::time::{Duration, Instant};
use swap_info::{get_processes_using_swap, get_chart_info, SizeUnits};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    style::{Color, Modifier, Style, Stylize},
    text::{Line},
    symbols::Marker,
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,BorderType,Borders,Axis, Chart, Dataset, GraphType, LegendPosition},
	layout::{Alignment, Constraint, Direction, Layout, Rect},

};
use crate::swap_info::SwapUpdate;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

const DATASET_STYLE: Style = Style::new()
    .fg(Color::Yellow)
    .add_modifier(Modifier::BOLD);

#[derive(Debug, Default)]
pub struct App {
    running: bool,
    pub vertical_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
    pub swap_size_unit: crate::SizeUnits,
    pub swap_processes_lines: Vec<Line<'static>>,
    pub last_update: Option<Instant>,
    pub chart_info: SwapUpdate,
    history_data: Vec<(f64, f64)>,
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
            history_data: Vec::new()
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        self.swap_processes_lines = self.create_process_lines();
        self.chart_info = get_chart_info()?;
        self.last_update = Some(Instant::now());

        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            if event::poll(Duration::from_millis(100))? {
                self.handle_crossterm_events()?;
            }

            if let Some(last_update) = self.last_update {
                if last_update.elapsed() >= Duration::from_secs(2) {
                    self.swap_processes_lines = self.create_process_lines();
                    self.chart_info = get_chart_info()?;

                    self.last_update = Some(Instant::now());
                }
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let main_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("SwapTop")
            .title_alignment(Alignment::Left);

        let main_area = main_block.inner(frame.area());

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
            .split(main_area);

        self.render_animated_chart(frame, chunks[0]);
        self.render_processes_list(frame, chunks[1]);

        
        frame.render_widget(main_block, frame.area());
        
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

        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),

            (_, KeyCode::Char('j') | KeyCode::Down) => {
                self.vertical_scroll = self.vertical_scroll.saturating_add(1);
                self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
            }
            (_, KeyCode::Char('k') | KeyCode::Up) => {
                self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
                self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
            }
            _ => {}
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }
    
    fn create_process_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        lines.push(Line::from(vec![
            format!("{:12}", "    PID").bold(),
            " | ".into(),
            format!("{:30}", "PROCESS").bold(),
            " | ".into(),
            format!("{:10}", "USED").bold()        ]));

        if let Ok(mut processes) = get_processes_using_swap(self.swap_size_unit.clone()) {
            processes.sort_by(|a, b| b.swap_size.cmp(&a.swap_size));

            for process in processes {
                lines.push(Line::from(vec![
                    format!("{:12}", process.pid).into(),
                    " | ".into(),
                    format!("{:30}", process.name).into(),
                    " | ".into(),
                    format!("{:10}", process.swap_size).into()
                ]));
            }
        }

        lines
    }

    fn render_animated_chart(&mut self, frame: &mut Frame, area: Rect) {
        const MAX_DATA_POINTS: usize = 200;
        
    
        
        if self.history_data.len() >= MAX_DATA_POINTS {
            self.history_data.remove(0);
        }

        let current_time = self.history_data.len() as f64;
        self.history_data.push((-current_time, self.chart_info.used_swap as f64));

        let max_x = current_time.max(MAX_DATA_POINTS as f64);
        let max_y = self.history_data.iter()
            .map(|&(_, y)| y)
            .fold(0.0, f64::max)
            .max(1.0);

        let datasets = vec![Dataset::default()
            .marker(Marker::Braille)
            .style(DATASET_STYLE)
            .graph_type(GraphType::Line)
            .data(&self.history_data)];

        let chart = Chart::new(datasets)
            .block(Block::bordered())
            .x_axis(
                Axis::default()
                    .bounds([-max_x, 0.0])
                    .style(Style::default().fg(Color::Gray))
            )
            .y_axis(
                Axis::default()
                    .bounds([0.0, max_y * 1.1])
                    .style(Style::default().fg(Color::Gray))
            );

        frame.render_widget(chart, area);
    }
    fn render_processes_list(&mut self, frame: &mut Frame, area: Rect) {
        self.vertical_scroll_state = self.vertical_scroll_state.content_length(self.swap_processes_lines.len());

        let unit_buttons = match self.swap_size_unit {
            SizeUnits::KB => "▶KB◀─MB─GB",
            SizeUnits::MB => "KB─▶MB◀─GB",
            SizeUnits::GB => "KB─MB─▶GB◀",
        };
        let bottom_block = Block::bordered()
            .title("Process (j/k or ▲/▼ to scroll)")
            .title_alignment(Alignment::Right);

        let process_paragraph = Paragraph::new(self.swap_processes_lines.clone())
            .block(bottom_block)
            .scroll((self.vertical_scroll as u16, 0));

        frame.render_widget(process_paragraph, area);

        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            area,
            &mut self.vertical_scroll_state,
        );
    }
}

