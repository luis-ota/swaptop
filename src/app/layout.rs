use ratatui::layout::Rect;

#[derive(Debug, Default, Clone)]
pub struct AppLayout {
    pub outer: Rect,
    #[allow(dead_code)]
    pub main_area: Rect,
    pub top_half: Rect,
    pub bottom_half: Rect,
    pub chart_area: Rect,
    pub swap_devices_area: Option<Rect>,
    pub divider_x: Option<u16>,
    pub process_area: Rect,
    pub info_area: Option<Rect>,
    pub info_divider_x: Option<u16>,
}

impl AppLayout {
    pub fn contains(&self, col: u16, row: u16, rect: Rect) -> bool {
        col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
    }
}

pub fn left_title_x(area: Rect) -> u16 {
    area.x + 1
}

pub fn right_title_x(area: Rect, text_len: usize) -> u16 {
    let inner_right = area.x + area.width.saturating_sub(1);
    inner_right.saturating_sub(text_len as u16)
}

pub fn center_title_x(area: Rect, text_len: usize) -> u16 {
    let inner_width = area.width.saturating_sub(2);
    area.x + 1 + inner_width.saturating_sub(text_len as u16) / 2
}

pub fn border_row_top(area: Rect) -> u16 {
    area.y
}

pub fn border_row_bottom(area: Rect) -> u16 {
    area.y + area.height.saturating_sub(1)
}
