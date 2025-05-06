mod swap_info;

use swap_info::{get_processes_using_swap, chart_info};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    text::{Line},
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,BorderType,Borders},
	layout::{Alignment, Constraint, Direction, Layout, Rect},

};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    running: bool,
    pub vertical_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: false,
            vertical_scroll_state: ScrollbarState::default(),
            vertical_scroll: 0,
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    fn render(&mut self, frame: &mut Frame) {
        // Create the main block with title "SwapTop"
        let main_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("SwapTop")
            .title_alignment(Alignment::Left);

        // Split the main area into two parts: top and bottom
        let main_area = main_block.inner(frame.area());

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
            .split(main_area);

        // Top frame with title "Chart"
        let top_block = Block::bordered()
            .borders(Borders::NONE)
            .title("Chart")
            .title_alignment(Alignment::Right);
        let top_text = "Chart content goes here"; // Replace with your actual chart content
        frame.render_widget(
            Paragraph::new(top_text).block(top_block),
            chunks[0]
        );

        // Process list with scrolling functionality
        let process_lines = self.create_process_lines();
        self.vertical_scroll_state = self.vertical_scroll_state.content_length(process_lines.len());

        // Bottom frame with title "Process"
        let bottom_block = Block::bordered()
            .title("Process (j/k or ▲/▼ to scroll)")
            .title_alignment(Alignment::Right);

        // Render the scrollable process list
        let process_paragraph = Paragraph::new(process_lines)
            .block(bottom_block)
            .scroll((self.vertical_scroll as u16, 0));

        frame.render_widget(process_paragraph, chunks[1]);

        // Render the scrollbar
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            chunks[1],
            &mut self.vertical_scroll_state,
        );

        // Render the main block (this must be done last)
        frame.render_widget(main_block, frame.area());
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
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

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
    
    fn create_process_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Add header line
        lines.push(Line::from(vec![
            "PID".bold(),
            " | ".into(),
            "PROCESS".bold(),
            " | ".into(),
            "SWAP (KB)".bold()
        ]));

        if let Ok(mut processes) = get_processes_using_swap() {
            // Sort processes by swap usage (highest first)
            processes.sort_by(|a, b| b.swap_kb.cmp(&a.swap_kb));

            for process in processes {
                lines.push(Line::from(vec![
                    format!("{:6}", process.pid).into(),
                    " | ".into(),
                    format!("{:15}", process.name).into(),
                    " | ".into(),
                    format!("{:10}", process.swap_kb).into()
                ]));
            }
        }

        lines
    }
}

