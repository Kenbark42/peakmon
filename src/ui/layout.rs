use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub header: Rect,
    pub body: Rect,
    pub footer: Rect,
}

pub fn compute_layout(area: Rect) -> AppLayout {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // header + tab bar
            Constraint::Min(10),   // body
            Constraint::Length(1), // footer
        ])
        .split(area);

    AppLayout {
        header: chunks[0],
        body: chunks[1],
        footer: chunks[2],
    }
}
