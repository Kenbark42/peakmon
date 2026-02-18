use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use super::theme;

pub fn render(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(60, 30, area);
    frame.render_widget(Clear, popup);

    let lines = vec![
        header_line("Navigation"),
        key_line("1-9, 0", "Switch to tab by number"),
        key_line("Tab / Shift+Tab", "Cycle through tabs"),
        key_line("F1-F10", "Switch to tab by function key"),
        Line::raw(""),
        header_line("Scrolling"),
        key_line("j / Down", "Scroll down / select next"),
        key_line("k / Up", "Scroll up / select previous"),
        key_line("g / G", "Jump to top / bottom"),
        key_line("PgDn / PgUp", "Page down / page up"),
        Line::raw(""),
        header_line("General"),
        key_line("+/-", "Increase / decrease refresh rate"),
        key_line("/", "Filter (Processes & Logs)"),
        key_line("?", "Toggle this help"),
        key_line("q / Ctrl+C", "Quit"),
        Line::raw(""),
        header_line("Processes Tab"),
        key_line("c / m / p / n", "Sort by CPU / Mem / PID / Name"),
        key_line("t", "Toggle tree view"),
        key_line("K", "Kill selected process (SIGTERM)"),
        Line::raw(""),
        header_line("AI Tab"),
        key_line("j / k", "Select model"),
        key_line("P", "Pull (download) a model"),
        key_line("D", "Delete selected model"),
        key_line("Enter", "Load selected model"),
        key_line("U", "Unload selected model"),
        Line::raw(""),
        header_line("Logs Tab"),
        key_line("l", "Cycle log level filter"),
        key_line("a", "Toggle auto-scroll"),
    ];

    let block = Block::default()
        .title(Line::styled(" Help ", theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .style(ratatui::style::Style::default().bg(theme::BASE));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, popup);
}

fn header_line(text: &str) -> Line<'_> {
    Line::from(Span::styled(format!("  {text}"), theme::title_style()))
}

fn key_line<'a>(key: &'a str, desc: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("    {key:<20}"), theme::key_hint_style()),
        Span::styled(desc, theme::label_style()),
    ])
}

fn centered_rect(width_pct: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .flex(Flex::Center)
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_pct) / 2),
            Constraint::Percentage(width_pct),
            Constraint::Percentage((100 - width_pct) / 2),
        ])
        .split(vertical[1]);
    horizontal[1]
}
