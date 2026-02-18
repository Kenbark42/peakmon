use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::tabs::Tab;
use super::theme;

pub fn render(frame: &mut Frame, area: Rect, current_tab: Tab, filter_mode: bool) {
    let hints = if filter_mode {
        vec![
            Span::styled(" Filter: ", theme::key_hint_style()),
            Span::styled("type to filter  ", theme::label_style()),
            Span::styled("Enter", theme::key_hint_style()),
            Span::styled(" confirm  ", theme::label_style()),
            Span::styled("Esc", theme::key_hint_style()),
            Span::styled(" cancel", theme::label_style()),
        ]
    } else {
        let mut h = vec![
            Span::styled(" q", theme::key_hint_style()),
            Span::styled(" quit  ", theme::label_style()),
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
                    Span::styled(" name", theme::label_style()),
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
        h
    };

    let line = Line::from(hints);
    frame.render_widget(
        Paragraph::new(line).style(theme::footer_style()),
        area,
    );
}
