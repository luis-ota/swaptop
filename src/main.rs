    mod swap_info;
    mod theme;

    use std::time::{Duration, Instant};
    use swap_info::{get_processes_using_swap, get_chart_info, SizeUnits};
    use color_eyre::Result;
    use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use ratatui::{
        DefaultTerminal, Frame,
        style::{Style, Stylize},
        text::{Line},
        symbols::Marker,
        widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, BorderType, Axis, Chart, Dataset, GraphType},
        layout::{Alignment, Constraint, Direction, Layout, Rect},

    };
    use crate::swap_info::{aggregate_processes, convert_swap, SwapUpdate};
    use crate::theme::{Theme, ThemeType};

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
        history_data: Vec<(f64, f64)>,
        pub aggregated: bool,
        current_theme: ThemeType,
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
                history_data: Vec::new(),
                aggregated: false,
                current_theme: ThemeType::Dracula,
            }
        }

        pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
            self.running = true;
            self.swap_processes_lines = self.create_process_lines(self.aggregated);
            self.chart_info = get_chart_info()?;
            self.last_update = Some(Instant::now());

            while self.running {
                terminal.draw(|frame| self.render(frame))?;

                if event::poll(Duration::from_millis(100))? {
                    self.handle_crossterm_events()?;
                }

                if let Some(last_update) = self.last_update {
                    if last_update.elapsed() >= Duration::from_secs(1) {
                        self.swap_processes_lines = self.create_process_lines(self.aggregated);
                        self.chart_info = get_chart_info()?;

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
                .title(Line::from(" swaptop ").bold().fg(theme.primary).left_aligned())
                .title(Line::from(format!("theme (t to change): {:?}", self.current_theme)).bold().fg(theme.primary).right_aligned())
                .style(Style::default().bg(theme.background).fg(theme.text));

            let main_area = main_block.inner(frame.area());

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
                .split(main_area);

            self.render_animated_chart(frame, chunks[0], &theme);
            self.render_processes_list(frame, chunks[1], &theme);

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

            match key.code {  // Changed from (key.modifiers, key.code)
                // quit
                KeyCode::Esc | KeyCode::Char('q') => self.quit(),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => self.quit(),

                // up and down list
                KeyCode::Char('d') | KeyCode::Down => {
                    self.vertical_scroll = self.vertical_scroll.saturating_add(1);
                    self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
                }
                KeyCode::Char('u') | KeyCode::Up => {
                    self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
                    self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
                }

                // change unit
                KeyCode::Char('k') => self.swap_size_unit = SizeUnits::KB,
                KeyCode::Char('m') => self.swap_size_unit = SizeUnits::MB,
                KeyCode::Char('g') => self.swap_size_unit = SizeUnits::GB,

                // aggregate
                KeyCode::Char('a') => self.aggregated = !self.aggregated,

                // change theme
                KeyCode::Char('t') => self.cycle_theme(),

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
        fn quit(&mut self) {
            self.running = false;
        }

        fn create_process_lines(&self, aggregated: bool) -> Vec<Line<'static>> {
            let mut lines = Vec::new();

            lines.push(Line::from(vec![
                format!("{:12}", if self.aggregated {"COUNT"}else{"PID"}).bold(),
                " | ".into(),
                format!("{:30}", "PROCESS").bold(),
                " | ".into(),
                format!("{:10}", "USED").bold()        ]));

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
                    if let SizeUnits::KB = self.swap_size_unit {process_size = format!("{}", process.swap_size)}

                        lines.push(Line::from(vec![
                        format!("{:12}", process.pid).into(),
                        " | ".into(),
                        format!("{:30}", process.name).into(),
                        " | ".into(),
                        format!("{:10}", process_size).into()
                    ]));
                }
            }

            lines
        }

        fn render_animated_chart(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {

            let total = convert_swap(self.chart_info.total_swap, self.swap_size_unit.clone());
            let used = convert_swap(self.chart_info.used_swap, self.swap_size_unit.clone());

            let total_used_title: String = match self.swap_size_unit {
                SizeUnits::KB => format!("total avaliable: {} | used: {}", total, used),
                SizeUnits::MB => format!("total avaliable: {} | used: {:.2}", total.round(), used),
                SizeUnits::GB => format!("total avaliable: {:.2} | used: {:.2}", total, used),
            };

            let swap_usage_percent = self.chart_info.used_swap as f64 / self.chart_info.total_swap as f64 * 100.0;
            // Create the dataset for the chart
            let datasets = vec![Dataset::default()
                .marker(Marker::Dot)
                .style(Style::default().fg(theme.primary))
                .graph_type(GraphType::Line)
                .data(&self.history_data)];

            let chart = Chart::new(datasets)
                .block(Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme.border))
                           .title(Line::from(format!("swap usage {}%", swap_usage_percent.round() as u64))
                               .fg(theme.primary).bold().right_aligned())
                           .title(Line::from(total_used_title).fg(theme.text).left_aligned())
                    
                           
                )
                .x_axis(
                    Axis::default()
                        .style(Style::default().fg(theme.text)),
                )
                .y_axis(
                    Axis::default()
                        .style(Style::default().fg(theme.text)),
                );

            frame.render_widget(chart, area);
        }

        fn render_processes_list(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
            let unit_buttons = match self.swap_size_unit {
                SizeUnits::KB => "▶KB◀─MB─GB",
                SizeUnits::MB => "KB─▶MB◀─GB",
                SizeUnits::GB => "KB─MB─▶GB◀",
            };
            
            let visible_height = area.height as usize;
            let content_height = self.swap_processes_lines.len() + 2;
            self.vertical_scroll = self.vertical_scroll.min(content_height.saturating_sub(visible_height));
            self.vertical_scroll_state = self.vertical_scroll_state
                .content_length(content_height)
                .position(self.vertical_scroll);

            let bottom_block = Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.background))
                .title(Line::from("(a to aggregate) (u/d or ▲/▼ to scroll)").fg(theme.text).right_aligned())
                .title(Line::from(format!("unit (k/m/g to change): {}", unit_buttons))
                    .fg(theme.secondary).bold().left_aligned());

            let process_paragraph = Paragraph::new(self.swap_processes_lines.clone())
                .alignment(Alignment::Center)
                .block(bottom_block)
                .scroll((self.vertical_scroll as u16, 0));

            frame.render_widget(process_paragraph, area);

            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓"))
                    .style(Style::default().fg(theme.border))
                    .thumb_style(Style::default().fg(theme.primary)),
                area,
                &mut self.vertical_scroll_state,
            );
        }
    }

