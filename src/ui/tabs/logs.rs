use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::logs::stream::LogStream;
use crate::logs::LogLevel;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, log_stream: &LogStream, scroll_offset: usize) {
    let filtered = log_stream.filtered_entries();

    let filter_info = if let Some(ref level) = log_stream.level_filter {
        format!(" [level: {}]", level.as_str())
    } else {
        String::new()
    };

    let text_filter_info = if log_stream.text_filter.is_empty() {
        String::new()
    } else {
        format!(" [filter: {}]", log_stream.text_filter)
    };

    let auto_info = if log_stream.auto_scroll {
        " [auto-scroll]"
    } else {
        ""
    };

    let title = format!(
        "Logs ({}){}{}{}",
        filtered.len(),
        filter_info,
        text_filter_info,
        auto_info,
    );

    let block = Block::default()
        .title(Line::styled(format!(" {title} "), theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    let visible_height = area.height.saturating_sub(2) as usize;

    let effective_offset = if log_stream.auto_scroll {
        filtered.len().saturating_sub(visible_height)
    } else {
        scroll_offset.min(filtered.len().saturating_sub(visible_height))
    };

    let lines: Vec<Line> = filtered
        .iter()
        .skip(effective_offset)
        .take(visible_height)
        .map(|entry| {
            let level_color = match entry.level {
                LogLevel::Error => theme::RED,
                LogLevel::Fault => theme::RED,
                LogLevel::Info => theme::GREEN,
                LogLevel::Debug => theme::MAUVE,
                LogLevel::Default => theme::SUBTEXT,
            };

            Line::from(vec![
                Span::styled(&entry.timestamp, theme::label_style()),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", entry.level.as_str()),
                    Style::default().fg(level_color),
                ),
                Span::raw(" "),
                Span::styled(&entry.process, Style::default().fg(theme::BLUE)),
                Span::raw(": "),
                Span::styled(&entry.message, theme::value_style()),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
