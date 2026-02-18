use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Sparkline};
use ratatui::Frame;

use crate::ui::theme;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    data: &[u64],
    max_val: Option<u64>,
    color: Color,
    annotation: &str,
) {
    let block = Block::default()
        .title(Line::styled(format!(" {title} "), theme::title_style()))
        .title_bottom(Line::styled(
            format!(" {annotation} "),
            Style::default().fg(color),
        ))
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    let sparkline = Sparkline::default()
        .block(block)
        .data(data)
        .max(max_val.unwrap_or(100))
        .style(Style::default().fg(color));

    frame.render_widget(sparkline, area);
}
