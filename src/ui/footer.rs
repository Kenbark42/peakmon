use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use std::time::{Duration, Instant};

use super::tabs::Tab;
use super::theme;
use crate::metrics::ai::{AiMetrics, ChatStatus};

#[allow(clippy::too_many_arguments)]
pub fn render(
    frame: &mut Frame,
    area: Rect,
    current_tab: Tab,
    filter_mode: bool,
    filter_buffer: &str,
    refresh_rate: Duration,
    ai: &AiMetrics,
    copy_feedback: Option<Instant>,
) {
    let hints = if filter_mode {
        let display = if filter_buffer.is_empty() {
            "type to filter".to_string()
        } else {
            filter_buffer.to_string()
        };
        vec![
            Span::styled(" /", theme::key_hint_style()),
            Span::styled(format!("{display}_ "), theme::value_style()),
            Span::styled(" Enter", theme::key_hint_style()),
            Span::styled(" confirm  ", theme::label_style()),
            Span::styled("Esc", theme::key_hint_style()),
            Span::styled(" cancel", theme::label_style()),
        ]
    } else {
        let mut h = vec![
            Span::styled(" q", theme::key_hint_style()),
            Span::styled(" quit  ", theme::label_style()),
            Span::styled("?", theme::key_hint_style()),
            Span::styled(" help  ", theme::label_style()),
            Span::styled("Tab", theme::key_hint_style()),
            Span::styled(" switch  ", theme::label_style()),
            Span::styled("+/-", theme::key_hint_style()),
            Span::styled(" rate  ", theme::label_style()),
        ];

        match current_tab {
            Tab::Processes => {
                h.extend([
                    Span::styled("/", theme::key_hint_style()),
                    Span::styled(" filter  ", theme::label_style()),
                    Span::styled("c", theme::key_hint_style()),
                    Span::styled(" cpu  ", theme::label_style()),
                    Span::styled("m", theme::key_hint_style()),
                    Span::styled(" mem  ", theme::label_style()),
                    Span::styled("p", theme::key_hint_style()),
                    Span::styled(" pid  ", theme::label_style()),
                    Span::styled("n", theme::key_hint_style()),
                    Span::styled(" name  ", theme::label_style()),
                    Span::styled("t", theme::key_hint_style()),
                    Span::styled(" tree  ", theme::label_style()),
                    Span::styled("K", theme::key_hint_style()),
                    Span::styled(" kill", theme::label_style()),
                ]);
            }
            Tab::Logs => {
                h.extend([
                    Span::styled("/", theme::key_hint_style()),
                    Span::styled(" filter  ", theme::label_style()),
                    Span::styled("l", theme::key_hint_style()),
                    Span::styled(" level  ", theme::label_style()),
                    Span::styled("a", theme::key_hint_style()),
                    Span::styled(" autoscroll", theme::label_style()),
                ]);
            }
            Tab::Ai => {
                if ai.chat_status == ChatStatus::Generating {
                    h.extend([
                        Span::styled("Esc", theme::key_hint_style()),
                        Span::styled(" cancel generation", theme::label_style()),
                    ]);
                } else {
                    h.extend([
                        Span::styled("j/k", theme::key_hint_style()),
                        Span::styled(" select  ", theme::label_style()),
                        Span::styled("i", theme::key_hint_style()),
                        Span::styled(" chat  ", theme::label_style()),
                        Span::styled("S", theme::key_hint_style()),
                        Span::styled(" search  ", theme::label_style()),
                        Span::styled("D", theme::key_hint_style()),
                        Span::styled(" delete  ", theme::label_style()),
                        Span::styled("U", theme::key_hint_style()),
                        Span::styled(" unload  ", theme::label_style()),
                        Span::styled("C", theme::key_hint_style()),
                        Span::styled(" clear  ", theme::label_style()),
                        Span::styled("y", theme::key_hint_style()),
                        Span::styled(" copy  ", theme::label_style()),
                        Span::styled("Y", theme::key_hint_style()),
                        Span::styled(" copy all", theme::label_style()),
                    ]);
                }
            }
            Tab::Temperatures => {
                h.extend([
                    Span::styled("j/k", theme::key_hint_style()),
                    Span::styled(" select sensor", theme::label_style()),
                ]);
            }
            _ => {
                h.extend([
                    Span::styled("j/k", theme::key_hint_style()),
                    Span::styled(" scroll", theme::label_style()),
                ]);
            }
        }
        // Show "Copied!" feedback for 2 seconds after a copy
        if let Some(t) = copy_feedback {
            if t.elapsed() < Duration::from_secs(2) {
                h.push(Span::styled("  ✓ Copied!", theme::value_style()));
            }
        }

        h
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(16)])
        .split(area);

    let line = Line::from(hints);
    frame.render_widget(Paragraph::new(line).style(theme::footer_style()), chunks[0]);

    let rate_ms = refresh_rate.as_millis();
    let rate_text = if rate_ms >= 1000 {
        format!("{:.1}s ", rate_ms as f64 / 1000.0)
    } else {
        format!("{rate_ms}ms ")
    };
    let rate_line = Line::from(vec![
        Span::styled("refresh ", theme::label_style()),
        Span::styled(rate_text, theme::value_style()),
    ]);
    frame.render_widget(
        Paragraph::new(rate_line)
            .alignment(Alignment::Right)
            .style(theme::footer_style()),
        chunks[1],
    );
}
