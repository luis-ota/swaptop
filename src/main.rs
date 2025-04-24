mod swap_info;

use swap_info::{get_processes_using_swap, chart_info};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
	layout::{Alignment, Constraint, Direction, Layout},

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
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
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
        .title("Chart")
        .title_alignment(Alignment::Center);
    let top_text = "Chart content goes here"; // Replace with your actual chart content
    frame.render_widget(
        Paragraph::new(top_text).block(top_block),
        chunks[0]
    );
    
    // Bottom frame with title "Process"
    let bottom_block = Block::bordered()
        .title("Process")
        .title_alignment(Alignment::Center);
    let bottom_text = "Process list goes here"; // Replace with your actual process content
    frame.render_widget(
        Paragraph::new(bottom_text).block(bottom_block),
        chunks[1]
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
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

