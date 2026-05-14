use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    symbols::Marker,
    text::Line,
    widgets::{
        Axis, Block, BorderType, Chart, Clear, Dataset, GraphType, Paragraph, Scrollbar,
        ScrollbarOrientation,
    },
};

use super::{App, FocusedPanel, LINUX};
use crate::app::layout::AppLayout;
use crate::swap_info::SizeUnits;
#[cfg(target_os = "linux")]
use crate::swap_info::find_mount_device;
use crate::theme::Theme;

impl App {
    fn panel_border(&self, panel: FocusedPanel, theme: &Theme) -> ratatui::style::Color {
        if self.focused_panel == panel {
            theme.primary
        } else {
            theme.border
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
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
                Line::from(self.theme_title_str())
                    .bold()
                    .fg(theme.primary)
                    .right_aligned(),
            )
            .title(
                Line::from(self.timeout_title_str())
                    .bold()
                    .fg(theme.primary)
                    .centered(),
            )
            .style(Style::default().bg(theme.background).fg(theme.text));

        let outer = frame.area();
        let main_area = main_block.inner(outer);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(main_area);

        let mut layout = AppLayout {
            outer,
            main_area,
            top_half: chunks[0],
            bottom_half: chunks[1],
            chart_area: Rect::default(),
            swap_devices_area: None,
            divider_x: None,
            process_area: Rect::default(),
            info_area: None,
            info_divider_x: None,
        };

        if self.display_devices && LINUX {
            let dev_pct = (self.devices_split_ratio * 100.0) as u16;
            let chart_pct = 100 - dev_pct;

            let upper_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(dev_pct),
                    Constraint::Length(1),
                    Constraint::Percentage(chart_pct),
                ])
                .split(chunks[0]);

            layout.swap_devices_area = Some(upper_chunks[0]);
            layout.divider_x = Some(upper_chunks[1].x);
            layout.chart_area = upper_chunks[2];

            #[cfg(target_os = "linux")]
            self.render_swap_devices(frame, upper_chunks[0], &theme);
            self.render_divider(frame, upper_chunks[1], &theme);
            self.render_animated_chart(frame, upper_chunks[2], &theme);
        } else {
            layout.chart_area = chunks[0];
            self.render_animated_chart(frame, chunks[0], &theme);
        }

        self.render_processes_list(frame, chunks[1], &theme);
        layout.process_area = self.layout.process_area;
        layout.info_area = self.layout.info_area;
        layout.info_divider_x = self.layout.info_divider_x;
        frame.render_widget(main_block, outer);

        self.layout = layout;

        if self.show_help {
            self.render_help_popup(frame, &theme);
        }
    }

    fn render_divider(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let h = area.height as usize;
        let mut lines: Vec<&str> = Vec::with_capacity(h);
        let (l_row, mid, r_row) = (h / 4, h / 2, 3 * h / 4);
        for i in 0..h {
            let ch = if i == mid {
                "⬌"
            } else if i == l_row && h >= 6 {
                "l"
            } else if i == r_row && h >= 6 {
                "r"
            } else {
                "┃"
            };
            lines.push(ch);
        }
        let divider = Paragraph::new(lines.join("\n")).style(Style::default().fg(theme.primary));
        frame.render_widget(divider, area);
    }

    #[cfg(target_os = "linux")]
    fn render_swap_devices(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let total_used_title = self.generete_total_used_title();
        let total_n_used_line = Line::from(total_used_title).fg(theme.text).right_aligned();

        let src_w = self
            .chart_info
            .swap_devices
            .iter()
            .map(|d| {
                let src = find_mount_device(std::path::Path::new(&d.name))
                    .unwrap_or_else(|| "unknown".into());
                src.len()
            })
            .max()
            .unwrap_or(7)
            .clamp(7, 25);

        let name_w = self
            .chart_info
            .swap_devices
            .iter()
            .map(|d| d.name.len())
            .max()
            .unwrap_or(10)
            .clamp(10, 20);

        let sep = " | ";
        let (col_type, col_prio, col_total, col_used) = (10usize, 8usize, 10usize, 10usize);

        let mut lines = Vec::new();

        let tier = if area.width >= 80 {
            3
        } else if area.width >= 55 {
            2
        } else if area.width >= 38 {
            1
        } else {
            0
        };

        let header = match tier {
            3 => format!(
                "{0:<sw$}{4}{1:<nw$}{4}{2:<ct$}{4}{3:>cp$}{4}{5:>ctot$}{4}{6:>cu$}",
                "disk",
                "path",
                "type",
                "priority",
                sep,
                "total",
                "used",
                sw = src_w,
                nw = name_w,
                ct = col_type,
                cp = col_prio,
                ctot = col_total,
                cu = col_used
            ),
            2 => format!(
                "{0:<sw$}{2}{1:<nw$}{2}{3:>ctot$}{2}{4:>cu$}",
                "disk",
                "path",
                sep,
                "total",
                "used",
                sw = src_w,
                nw = name_w,
                ctot = col_total,
                cu = col_used
            ),
            1 => format!(
                "{0:<sw$}{2}{1:>ctot$}{2}{3:>cu$}",
                "disk",
                "total",
                sep,
                "used",
                sw = src_w,
                ctot = col_total,
                cu = col_used
            ),
            _ => format!(
                "{0:<sw$}{2}{1:>cu$}",
                "disk",
                "used",
                sep,
                sw = src_w,
                cu = col_used
            ),
        };
        lines.push(Line::from(header));

        for device in &self.chart_info.swap_devices {
            let used = super::format_swap_value(device.used);
            let source = find_mount_device(std::path::Path::new(&device.name))
                .unwrap_or_else(|| "unknown".into());
            let total = super::format_swap_value(device.size);

            let row = match tier {
                3 => format!(
                    "{0:<sw$}{4}{1:<nw$}{4}{2:<ct$}{4}{3:>cp$}{4}{5:>ctot$}{4}{6:>cu$}",
                    source,
                    device.name,
                    device.kind,
                    device.priority,
                    sep,
                    total,
                    used,
                    sw = src_w,
                    nw = name_w,
                    ct = col_type,
                    cp = col_prio,
                    ctot = col_total,
                    cu = col_used
                ),
                2 => format!(
                    "{0:<sw$}{2}{1:<nw$}{2}{3:>ctot$}{2}{4:>cu$}",
                    source,
                    device.name,
                    sep,
                    total,
                    used,
                    sw = src_w,
                    nw = name_w,
                    ctot = col_total,
                    cu = col_used
                ),
                1 => format!(
                    "{0:<sw$}{2}{1:>ctot$}{2}{3:>cu$}",
                    source,
                    total,
                    sep,
                    used,
                    sw = src_w,
                    ctot = col_total,
                    cu = col_used
                ),
                _ => format!(
                    "{0:<sw$}{2}{1:>cu$}",
                    source,
                    used,
                    sep,
                    sw = src_w,
                    cu = col_used
                ),
            };
            lines.push(Line::from(row));
        }

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.panel_border(FocusedPanel::SwapDevices, theme)))
            .style(Style::default().bg(theme.background))
            .title(total_n_used_line)
            .title(
                Line::from("swap devices")
                .fg(theme.text)
                .left_aligned(),
            )
            .title_bottom(Line::from("(h to hide swap devices)").left_aligned());

        let dev_content_height = lines.len() + 2;
        let dev_inner_height = area.height.saturating_sub(2) as usize;
        let dev_overflow = dev_content_height > dev_inner_height;
        self.swap_devices_scroll = self.swap_devices_scroll.min(if dev_overflow {
            dev_content_height.saturating_sub(dev_inner_height)
        } else {
            0
        });

        let para = Paragraph::new(lines)
            .block(block)
            .centered()
            .scroll((self.swap_devices_scroll as u16, 0));
        frame.render_widget(para, area);

        if dev_overflow {
            let dev_scroll_state = ratatui::widgets::ScrollbarState::new(dev_content_height)
                .position(self.swap_devices_scroll);
            frame.render_stateful_widget(
                ratatui::widgets::Scrollbar::new(
                    ratatui::widgets::ScrollbarOrientation::VerticalRight,
                )
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .style(Style::default().fg(theme.scrollbar))
                .thumb_style(Style::default().fg(theme.primary)),
                area,
                &mut dev_scroll_state.clone(),
            );
        }
    }

    fn render_animated_chart(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let total_used_title = self.generete_total_used_title();

        let total_n_used_line = if self.display_devices {
            Line::from("").fg(theme.text).left_aligned()
        } else {
            Line::from(total_used_title).fg(theme.text).left_aligned()
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

        let bottom_title = if LINUX && !self.display_devices {
            "(h to show swap devices)"
        } else {
            ""
        };
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
                    .title(total_n_used_line)
                    .title_bottom(Line::from(bottom_title).left_aligned())
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
        let (proc_area, info_area) = if self.show_info_panel && area.width >= 50 {
            let info_pct = (self.info_split_ratio * 100.0) as u16;
            let proc_pct = 100u16.saturating_sub(info_pct + 1);
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(proc_pct),
                    Constraint::Length(1),
                    Constraint::Percentage(info_pct),
                ])
                .split(area);
            self.layout.info_divider_x = Some(chunks[1].x);
            (chunks[0], Some(chunks[2]))
        } else {
            self.layout.info_divider_x = None;
            (area, None)
        };
        self.layout.process_area = proc_area;
        self.layout.info_area = info_area;

        if let Some(info_area) = info_area {
            self.render_info_sidebar(frame, info_area, theme);
            if let Some(dx) = self.layout.info_divider_x {
                let div = Rect::new(dx, area.y, 1, area.height);
                self.render_divider(frame, div, theme);
            }
        }

        let unit_buttons = match self.swap_size_unit {
            SizeUnits::KB => "▶KB◀─MB─GB",
            SizeUnits::MB => "KB─▶MB◀─GB",
            SizeUnits::GB => "KB─MB─▶GB◀",
        };

        self.visible_height = proc_area.height as usize;
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
            .border_style(Style::default().fg(self.panel_border(FocusedPanel::ProcessList, theme)))
            .style(Style::default().bg(theme.background))
            .title(
                Line::from(if self.aggregated {
                    "(a to segregate) (↑/↓ PgUp/PgDn to scroll)"
                } else {
                    "(a to aggregate) (↑/↓ PgUp/PgDn to scroll)"
                })
                .fg(theme.text)
                .right_aligned(),
            )
            .title(
                Line::from(format!(
                    "unit (k/m/g to change): {}",
                    unit_buttons,
                ))
                .fg(theme.secondary)
                .bold()
                .left_aligned(),
            )
            .title_bottom(Line::from("↑/↓ select  Enter info  ? help").fg(theme.text));

        let mut display_lines = self.swap_processes_lines.clone();
        if self.selected_index > 0
            && let Some(line) = display_lines.get_mut(self.selected_index)
        {
            let hl_off = Style::default()
                .bg(theme.highlight)
                .add_modifier(ratatui::style::Modifier::BOLD);
            let is_left = self.show_info_panel;
            let pad = if is_left {
                proc_area.width.saturating_sub(line.width() as u16) as usize
            } else {
                0
            };
            *line = std::mem::take(line).style(hl_off);
            if pad > 0 {
                let hl_pad = Style::default().bg(theme.primary);
                line.spans
                    .push(ratatui::text::Span::raw(" ".repeat(pad)).style(hl_pad));
            }
        }

        let process_paragraph = Paragraph::new(display_lines)
            .alignment(if self.show_info_panel {
                Alignment::Left
            } else {
                Alignment::Center
            })
            .block(bottom_block)
            .scroll((self.vertical_scroll as u16, 0));

        frame.render_widget(process_paragraph, proc_area);

        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .style(Style::default().fg(theme.scrollbar))
                .thumb_style(Style::default().fg(theme.primary)),
            proc_area,
            &mut self.vertical_scroll_state,
        );
    }

    fn render_info_sidebar(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let data_idx = self.selected_index.saturating_sub(1);
        let Some(proc) = self.processes_data.get(data_idx) else {
            return;
        };

        let unit_label = match self.swap_size_unit {
            SizeUnits::KB => "KB",
            SizeUnits::MB => "MB",
            SizeUnits::GB => "GB",
        };

        let fmt_kb = |kb: u64| -> String {
            let val = crate::swap_info::convert_swap(kb, self.swap_size_unit.clone());
            format!("{} {}", super::format_swap_value(val), unit_label)
        };

        let pid_label = if self.aggregated { "COUNT" } else { "PID" };
        let swap_str = format!(
            "{} {}",
            super::format_swap_value(proc.swap_size),
            unit_label
        );

        let mut info = vec![
            format!("{}:    {}", pid_label, proc.pid),
            format!("Name:  {}", proc.name),
            format!("Swap:  {}", swap_str),
        ];

        #[cfg(target_os = "linux")]
        if !self.aggregated
            && let Ok(detail) = crate::swap_info::get_process_detail(proc.pid)
        {
            if let Some(ref path) = detail.exe_path {
                info.push(format!("Exe:   {}", path));
            }
            if let Some(state) = detail.state {
                info.push(format!("State: {}", state));
            }
            if let Some(v) = detail.vm_peak {
                info.push(format!("VmPeak: {}", fmt_kb(v)));
            }
            if let Some(v) = detail.vm_size {
                info.push(format!("VmSize: {}", fmt_kb(v)));
            }
            if let Some(v) = detail.vm_rss {
                info.push(format!("VmRSS:  {}", fmt_kb(v)));
            }
            if let Some(v) = detail.vm_data {
                info.push(format!("VmData: {}", fmt_kb(v)));
            }
            if let Some(v) = detail.vm_stk {
                info.push(format!("VmStk:  {}", fmt_kb(v)));
            }
            if let Some(n) = detail.threads {
                info.push(format!("Threads: {}", n));
            }
            if let Some(u) = detail.uid {
                info.push(format!("Uid:    {}", u));
            }
        }

        let lines: Vec<Line> = info.into_iter().map(Line::from).collect();
        let info_height = lines.len() + 2;
        let info_vis = area.height.saturating_sub(2) as usize;
        let info_overflow = info_height > info_vis;
        self.info_scroll = self.info_scroll.min(if info_overflow {
            info_height.saturating_sub(info_vis)
        } else {
            0
        });

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.panel_border(FocusedPanel::InfoPanel, theme)))
            .title(" Info ")
            .style(Style::default().bg(theme.background).fg(theme.text));

        let para = Paragraph::new(lines)
            .block(block)
            .scroll((self.info_scroll as u16, 0));
        frame.render_widget(para, area);

        if info_overflow {
            let state =
                ratatui::widgets::ScrollbarState::new(info_height).position(self.info_scroll);
            frame.render_stateful_widget(
                ratatui::widgets::Scrollbar::new(
                    ratatui::widgets::ScrollbarOrientation::VerticalRight,
                )
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .style(Style::default().fg(theme.scrollbar))
                .thumb_style(Style::default().fg(theme.primary)),
                area,
                &mut state.clone(),
            );
        }
    }

    fn render_help_popup(&self, frame: &mut Frame, theme: &Theme) {
        let outer = frame.area();
        let w = 62.min(outer.width.saturating_sub(4));
        let h = 28u16.min(outer.height.saturating_sub(4));
        let x = outer.x + (outer.width - w) / 2;
        let y = outer.y + (outer.height - h) / 2;

        let shortcuts = vec![
            ("General", ""),
            ("q / Ctrl+C / Esc", "Quit"),
            ("?", "Toggle this help"),
            ("", ""),
            ("Navigation", ""),
            ("↑/↓ or u/d", "Move selection"),
            ("PgUp/PgDn", "Page up/down"),
            ("Home/End", "First/last process"),
            ("Enter / click", "Open/close info panel"),
            ("", ""),
            ("Actions", ""),
            ("a", "Toggle aggregate"),
            ("t", "Cycle theme"),
            ("h", "Show/hide swap devices"),
            ("k / m / g", "Unit: KB / MB / GB"),
            ("← / →", "Adjust refresh timeout"),
            ("", ""),
            ("Panels", ""),
            ("Tab / Shift+Tab", "Cycle focused panel"),
            ("l / r", "Resize focused panel divider"),
            ("", ""),
            ("Mouse", ""),
            ("Click on ⬌", "Drag to resize divider"),
            ("Scroll", "Scroll active panel"),
            ("Click process", "Select + open info"),
        ];

        let lines: Vec<Line> = shortcuts
            .iter()
            .map(|(key, desc)| {
                if key.is_empty() {
                    Line::from("")
                } else if desc.is_empty() {
                    Line::from(format!(" {}", key)).bold()
                } else {
                    Line::from(vec![
                        format!(" {:<20}", key).into(),
                        " ".into(),
                        desc.to_string().into(),
                    ])
                }
            })
            .collect();

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary))
            .title(" Keyboard Shortcuts ")
            .style(Style::default().bg(theme.background).fg(theme.text));

        let para = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Left);
        let area = Rect::new(x, y, w, h);
        frame.render_widget(Clear, area);
        frame.render_widget(para, area);
    }
}
