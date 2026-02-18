use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Gauge};
use ratatui::Frame;

use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, title: &str, percent: f64, label: &str) {
    let clamped = percent.clamp(0.0, 100.0);

    let block = Block::default()
        .title(Line::styled(format!(" {title} "), theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    let gauge = Gauge::default()
        .block(block)
        .gauge_style(theme::gauge_style(clamped))
        .percent(clamped as u16)
        .label(label);

    frame.render_widget(gauge, area);
}
