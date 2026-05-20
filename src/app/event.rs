use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::Rect;

use crate::app::layout::{
    border_row_bottom, border_row_top, center_title_x, left_title_x, right_title_x,
};
use crate::app::{App, FocusedPanel, LINUX};
use crate::swap_info::SizeUnits;

const UNIT_LABEL_PREFIX: &str = "unit (k/m/g to change): ";

impl App {
    pub fn handle_crossterm_events(&mut self) -> color_eyre::Result<()> {
        match crossterm::event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(mouse) => self.on_mouse_event(mouse),
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        if self.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                    self.show_help = false;
                    self.help_scroll = 0;
                }
                KeyCode::Char('d') | KeyCode::Down => {
                    self.help_scroll = self.help_scroll.saturating_add(1);
                }
                KeyCode::Char('u') | KeyCode::Up => {
                    self.help_scroll = self.help_scroll.saturating_sub(1);
                }
                KeyCode::PageDown => {
                    self.help_scroll = self.help_scroll.saturating_add(10);
                }
                KeyCode::PageUp => {
                    self.help_scroll = self.help_scroll.saturating_sub(10);
                }
                KeyCode::Home => {
                    self.help_scroll = 0;
                }
                KeyCode::End => {
                    self.help_scroll = usize::MAX;
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Esc => {
                if self.show_info_panel {
                    self.show_info_panel = false;
                } else {
                    self.quit();
                }
            }
            KeyCode::Char('q') => self.quit(),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => self.quit(),

            KeyCode::Tab => {
                let start = self.focused_panel;
                loop {
                    self.focused_panel = self.focused_panel.next();
                    if self.panel_visible(self.focused_panel) || self.focused_panel == start {
                        break;
                    }
                }
            }
            KeyCode::BackTab => {
                let start = self.focused_panel;
                loop {
                    self.focused_panel = self.focused_panel.prev();
                    if self.panel_visible(self.focused_panel) || self.focused_panel == start {
                        break;
                    }
                }
            }

            KeyCode::Char('?') => {
                self.show_help = !self.show_help;
            }

            KeyCode::Char('d') | KeyCode::Down => self.scroll_focused(1),
            KeyCode::Char('u') | KeyCode::Up => self.scroll_focused(-1),
            KeyCode::End => self.scroll_focused_end(),
            KeyCode::Home => self.scroll_focused_home(),
            KeyCode::PageDown => {
                let page = self.visible_height.saturating_sub(4) as i32;
                self.scroll_focused(page.max(1));
            }
            KeyCode::PageUp => {
                let page = self.visible_height.saturating_sub(4) as i32;
                self.scroll_focused(-page.max(1));
            }

            KeyCode::Enter => {
                if self.selected_index > 0 {
                    self.show_info_panel = !self.show_info_panel;
                    self.info_scroll = 0;
                }
            }

            KeyCode::Char('k') => self.set_size_unit(SizeUnits::KB),
            KeyCode::Char('m') => self.set_size_unit(SizeUnits::MB),
            KeyCode::Char('g') => self.set_size_unit(SizeUnits::GB),

            KeyCode::Char('a') => self.toggle_aggregated(),
            KeyCode::Char('t') => self.cycle_theme(),
            KeyCode::Char('h') => self.toggle_display_devices(),

            KeyCode::Left | KeyCode::Right => self.change_timeout(key.code),

            KeyCode::Char('l') | KeyCode::Char('r') => {
                let step = if key.code == KeyCode::Char('l') {
                    -0.05
                } else {
                    0.05
                };
                let swap_vis = LINUX && self.display_devices;
                let info_vis = self.show_info_panel;
                match (swap_vis, info_vis, self.focused_panel) {
                    (true, false, _) => {
                        self.devices_split_ratio =
                            (self.devices_split_ratio + step).clamp(0.15, 0.85);
                        self.save_app_config();
                    }
                    (false, true, _) => {
                        self.info_split_ratio = (self.info_split_ratio + step).clamp(0.15, 0.85);
                        self.save_app_config();
                    }
                    (true, true, crate::app::FocusedPanel::SwapDevices) => {
                        self.devices_split_ratio =
                            (self.devices_split_ratio + step).clamp(0.15, 0.85);
                        self.save_app_config();
                    }
                    (true, true, _) => {
                        self.info_split_ratio = (self.info_split_ratio + step).clamp(0.15, 0.85);
                        self.save_app_config();
                    }
                    _ => {}
                }
            }

            _ => {}
        }
    }

    fn on_mouse_event(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.handle_click(mouse.column, mouse.row);
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if self.is_dragging {
                    let dx = mouse.column as i16 - self.drag_start_x as i16;
                    let total_width = self.layout.top_half.width as f64;
                    if total_width > 0.0 {
                        let ratio_delta = dx as f64 / total_width;
                        self.devices_split_ratio =
                            (self.drag_start_ratio + ratio_delta).clamp(0.15, 0.85);
                    }
                } else if self.is_info_dragging {
                    let dx = mouse.column as i16 - self.drag_start_x as i16;
                    let total_width = self.layout.bottom_half.width as f64;
                    if total_width > 0.0 {
                        let ratio_delta = dx as f64 / total_width;
                        self.info_split_ratio =
                            (self.drag_start_ratio - ratio_delta).clamp(0.15, 0.85);
                    }
                } else if self.is_scroll_dragging {
                    self.scroll_to_y(mouse.row);
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                if self.is_dragging {
                    self.is_dragging = false;
                    self.save_app_config();
                }
                if self.is_info_dragging {
                    self.is_info_dragging = false;
                    self.save_app_config();
                }
                if self.is_scroll_dragging {
                    self.is_scroll_dragging = false;
                }
            }
            MouseEventKind::ScrollDown => {
                let in_info = self.show_info_panel
                    && self
                        .layout
                        .info_area
                        .is_some_and(|a| self.layout.contains(mouse.column, mouse.row, a));

                if LINUX
                    && self.display_devices
                    && let Some(dev_area) = self.layout.swap_devices_area
                    && self.layout.contains(mouse.column, mouse.row, dev_area)
                {
                    let content_h = self.chart_info.swap_devices.len() + 2;
                    let inner_h = dev_area.height.saturating_sub(2) as usize;
                    self.swap_devices_scroll = self
                        .swap_devices_scroll
                        .saturating_add(1)
                        .min(content_h.saturating_sub(inner_h));
                } else if in_info {
                    self.info_scroll = self.info_scroll.saturating_add(1);
                } else if self
                    .layout
                    .contains(mouse.column, mouse.row, self.process_list_area())
                {
                    self.vertical_scroll = self.vertical_scroll.saturating_add(1);
                    self.vertical_scroll = self
                        .vertical_scroll
                        .min(self.swap_processes_lines.len().saturating_sub(1));
                    self.vertical_scroll_state =
                        self.vertical_scroll_state.position(self.vertical_scroll);
                }
            }
            MouseEventKind::ScrollUp => {
                let in_info = self.show_info_panel
                    && self
                        .layout
                        .info_area
                        .is_some_and(|a| self.layout.contains(mouse.column, mouse.row, a));

                if LINUX
                    && self.display_devices
                    && let Some(dev_area) = self.layout.swap_devices_area
                    && self.layout.contains(mouse.column, mouse.row, dev_area)
                {
                    self.swap_devices_scroll = self.swap_devices_scroll.saturating_sub(1);
                } else if in_info {
                    self.info_scroll = self.info_scroll.saturating_sub(1);
                } else if self
                    .layout
                    .contains(mouse.column, mouse.row, self.process_list_area())
                {
                    self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
                    self.vertical_scroll_state =
                        self.vertical_scroll_state.position(self.vertical_scroll);
                }
            }
            _ => {}
        }
    }

    fn handle_click(&mut self, col: u16, row: u16) {
        if self.show_help {
            let outer = self.layout.outer;
            let w = 62.min(outer.width.saturating_sub(4));
            let h = 28u16.min(outer.height.saturating_sub(4));
            let px = outer.x + (outer.width - w) / 2;
            let py = outer.y + (outer.height - h) / 2;
            if col < px || col >= px + w || row < py || row >= py + h {
                self.show_help = false;
            }
            return;
        }
        if self.try_click_divider(col, row) {
            return;
        }
        if self.try_click_info_divider(col, row) {
            return;
        }
        if self.is_on_scrollbar(col, row) {
            self.is_scroll_dragging = true;
            self.scroll_to_y(row);
            return;
        }
        if self.try_click_unit_buttons(col, row) {
            return;
        }
        if self.try_click_aggregate(col, row) {
            return;
        }
        if self.try_click_theme(col, row) {
            return;
        }
        if self.try_click_timeout(col, row) {
            return;
        }
        if self.try_click_process_row(col, row) {
            return;
        }
        self.try_click_devices_toggle(col, row);
    }

    fn try_click_divider(&mut self, col: u16, row: u16) -> bool {
        if let Some(divider_x) = self.layout.divider_x {
            let in_top_half = row >= self.layout.top_half.y
                && row < self.layout.top_half.y + self.layout.top_half.height;
            if (col as i16 - divider_x as i16).abs() <= 1 && in_top_half {
                self.is_dragging = true;
                self.drag_start_x = col;
                self.drag_start_ratio = self.devices_split_ratio;
                return true;
            }
        }
        false
    }

    fn try_click_info_divider(&mut self, col: u16, row: u16) -> bool {
        if let Some(dx) = self.layout.info_divider_x {
            let in_bottom = row >= self.layout.bottom_half.y
                && row < self.layout.bottom_half.y + self.layout.bottom_half.height;
            if (col as i16 - dx as i16).abs() <= 1 && in_bottom {
                self.is_info_dragging = true;
                self.drag_start_x = col;
                self.drag_start_ratio = self.info_split_ratio;
                return true;
            }
        }
        false
    }

    fn try_click_unit_buttons(&mut self, col: u16, row: u16) -> bool {
        let area = self.process_list_area();
        if row != border_row_top(area) {
            return false;
        }

        let prefix = UNIT_LABEL_PREFIX;
        let buttons_str = self.unit_buttons_str();
        let full_text = format!("{}{}", prefix, buttons_str);
        let title_x = left_title_x(area);

        let full_width = full_text.chars().count();
        if col < title_x || col >= title_x + full_width as u16 {
            return false;
        }

        let rel_col = (col - title_x) as usize;
        if rel_col < prefix.len() {
            return false;
        }

        let btn_offset = rel_col - prefix.len();

        let char_offset = |pat: &str| -> usize {
            buttons_str
                .find(pat)
                .map(|b| buttons_str[..b].chars().count())
                .unwrap_or(usize::MAX)
        };

        let kb_start = 0usize;
        let mb_start = char_offset("MB");
        let gb_start = char_offset("GB");

        if btn_offset >= kb_start && btn_offset < kb_start + 2 {
            self.set_size_unit(SizeUnits::KB);
            return true;
        }
        if btn_offset >= mb_start && btn_offset < mb_start + 2 {
            self.set_size_unit(SizeUnits::MB);
            return true;
        }
        if btn_offset >= gb_start && btn_offset < gb_start + 2 {
            self.set_size_unit(SizeUnits::GB);
            return true;
        }

        false
    }

    fn try_click_aggregate(&mut self, col: u16, row: u16) -> bool {
        let area = self.process_list_area();
        if row != border_row_top(area) {
            return false;
        }

        let full_title = if self.aggregated {
            "(a to segregate) (↑/↓ PgUp/PgDn to scroll)"
        } else {
            "(a to aggregate) (↑/↓ PgUp/PgDn to scroll)"
        };
        let full_start_x = right_title_x(area, full_title.chars().count());
        let label = self.aggregate_title_str();
        if col >= full_start_x && col < full_start_x + label.len() as u16 {
            self.toggle_aggregated();
            return true;
        }
        false
    }

    fn try_click_theme(&mut self, col: u16, row: u16) -> bool {
        let area = self.layout.outer;
        if row != border_row_top(area) {
            return false;
        }

        let title = self.theme_title_str();
        let title_x = right_title_x(area, title.len());
        if col >= title_x && col < title_x + title.len() as u16 {
            self.cycle_theme();
            return true;
        }
        false
    }

    fn try_click_timeout(&mut self, col: u16, row: u16) -> bool {
        let area = self.layout.outer;
        if row != border_row_top(area) {
            return false;
        }

        let title = self.timeout_title_str();
        let title_x = center_title_x(area, title.len());
        if col >= title_x && col < title_x + title.len() as u16 {
            let mid = title_x + title.len() as u16 / 2;
            if col < mid {
                self.change_timeout(KeyCode::Left);
            } else {
                self.change_timeout(KeyCode::Right);
            }
            return true;
        }
        false
    }

    fn try_click_devices_toggle(&mut self, col: u16, row: u16) -> bool {
        if !LINUX {
            return false;
        }

        if self.display_devices {
            if let Some(dev_area) = self.layout.swap_devices_area {
                let bottom_row = border_row_bottom(dev_area);
                if row == bottom_row {
                    let label = "(h to hide swap devices)";
                    let label_x = left_title_x(dev_area);
                    if col >= label_x && col < label_x + label.len() as u16 {
                        self.toggle_display_devices();
                        return true;
                    }
                }
                let top_row = border_row_top(dev_area);
                if row == top_row {
                    let label = "swap devices";
                    let label_x = left_title_x(dev_area);
                    if col >= label_x && col < label_x + label.len() as u16 {
                        self.toggle_display_devices();
                        return true;
                    }
                }
            }
        } else {
            let chart_area = self.layout.chart_area;
            let bottom_row = border_row_bottom(chart_area);
            if row == bottom_row {
                let label = "(h to show swap devices)";
                let label_x = left_title_x(chart_area);
                if col >= label_x && col < label_x + label.len() as u16 {
                    self.toggle_display_devices();
                    return true;
                }
            }
        }
        false
    }

    fn is_on_scrollbar(&self, col: u16, row: u16) -> bool {
        let area = self.process_list_area();
        col == area.x + area.width.saturating_sub(1) && row >= area.y && row < area.y + area.height
    }

    fn scroll_to_y(&mut self, y: u16) {
        let area = self.process_list_area();
        let track_top = area.y + 1;
        let track_height = area.height.saturating_sub(2);
        if track_height == 0 {
            return;
        }
        let content_height = self.swap_processes_lines.len() + 2;
        let max_scroll = content_height.saturating_sub(self.visible_height);
        let y_rel = (y.saturating_sub(track_top)).min(track_height - 1) as usize;
        self.vertical_scroll = (y_rel * content_height / track_height as usize).min(max_scroll);
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
    }

    fn try_click_process_row(&mut self, col: u16, row: u16) -> bool {
        let area = self.process_list_area();
        let inner_y = area.y + 1;
        let inner_bottom = area.y + area.height.saturating_sub(1);
        if row <= area.y || row >= inner_bottom {
            return false;
        }
        if col >= area.x + area.width {
            return false;
        }
        let scrollbar_col = area.x + area.width.saturating_sub(1);
        if col == scrollbar_col {
            return false;
        }
        let line_idx = self
            .vertical_scroll
            .saturating_add((row - inner_y) as usize);
        if line_idx > 0 && line_idx < self.swap_processes_lines.len() {
            self.selected_index = line_idx;
            self.show_info_panel = true;
            self.info_scroll = 0;
            return true;
        }
        false
    }

    fn scroll_focused(&mut self, delta: i32) {
        match self.focused_panel {
            FocusedPanel::ProcessList => {
                let max = self.swap_processes_lines.len().saturating_sub(1) as i32;
                let new = (self.selected_index as i32 + delta).max(1).min(max);
                self.selected_index = new as usize;
                let inner = self.visible_height.saturating_sub(2);
                if inner == 0 {
                    return;
                }
                if self.selected_index < self.vertical_scroll {
                    self.vertical_scroll = self.selected_index;
                } else if self.selected_index >= self.vertical_scroll + inner {
                    self.vertical_scroll = self.selected_index.saturating_sub(inner) + 1;
                }
                let max_scroll = self.swap_processes_lines.len().saturating_sub(inner);
                self.vertical_scroll = self.vertical_scroll.min(max_scroll);
                self.vertical_scroll_state =
                    self.vertical_scroll_state.position(self.vertical_scroll);
            }
            FocusedPanel::SwapDevices => {
                if LINUX && self.display_devices {
                    self.swap_devices_scroll =
                        (self.swap_devices_scroll as i32 + delta).max(0) as usize;
                }
            }
            FocusedPanel::InfoPanel => {
                if self.show_info_panel {
                    self.info_scroll = (self.info_scroll as i32 + delta).max(0) as usize;
                }
            }
        }
    }

    fn scroll_focused_end(&mut self) {
        match self.focused_panel {
            FocusedPanel::ProcessList => {
                self.selected_index = self.swap_processes_lines.len().saturating_sub(1);
                let inner = self.visible_height.saturating_sub(2);
                if inner > 0 {
                    self.vertical_scroll = self.swap_processes_lines.len().saturating_sub(inner);
                    self.vertical_scroll_state =
                        self.vertical_scroll_state.position(self.vertical_scroll);
                }
            }
            FocusedPanel::SwapDevices => {
                if LINUX
                    && self.display_devices
                    && let Some(dev_area) = self.layout.swap_devices_area
                {
                    let content = self.chart_info.swap_devices.len() + 2;
                    let inner = dev_area.height.saturating_sub(2) as usize;
                    self.swap_devices_scroll = content.saturating_sub(inner);
                }
            }
            FocusedPanel::InfoPanel => {
                if self.show_info_panel
                    && let Some(_info_area) = self.layout.info_area
                {
                    self.info_scroll = 0;
                }
            }
        }
    }

    fn scroll_focused_home(&mut self) {
        match self.focused_panel {
            FocusedPanel::ProcessList => {
                self.selected_index = 1;
                self.vertical_scroll = 0;
                self.vertical_scroll_state = self.vertical_scroll_state.position(0);
            }
            FocusedPanel::SwapDevices => {
                self.swap_devices_scroll = 0;
            }
            FocusedPanel::InfoPanel => {
                self.info_scroll = 0;
            }
        }
    }

    fn process_list_area(&self) -> Rect {
        if self.layout.process_area.width > 0 {
            self.layout.process_area
        } else {
            self.layout.bottom_half
        }
    }

    fn panel_visible(&self, panel: crate::app::FocusedPanel) -> bool {
        match panel {
            crate::app::FocusedPanel::ProcessList => true,
            crate::app::FocusedPanel::SwapDevices => LINUX && self.display_devices,
            crate::app::FocusedPanel::InfoPanel => self.show_info_panel,
        }
    }
}
