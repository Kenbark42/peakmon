use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::tabs::Tab;
use super::theme;
use crate::util::format_uptime;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    current_tab: Tab,
    hostname: &str,
    uptime_secs: u64,
    load_avg: [f64; 3],
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    // Top line: app name + hostname + uptime
    let uptime_str = format_uptime(uptime_secs);
    let info_line = Line::from(vec![
        Span::styled(" peakmon ", theme::title_style()),
        Span::styled(format!("  {hostname}"), theme::value_style()),
        Span::styled(format!("  up {uptime_str}"), theme::label_style()),
        Span::styled(
            format!(
                "  load {:.2} {:.2} {:.2}",
                load_avg[0], load_avg[1], load_avg[2]
            ),
            theme::label_style(),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(info_line).style(theme::header_style()),
        chunks[0],
    );

    // Tab bar
    let mut tab_spans = vec![Span::raw(" ")];
    for tab in &Tab::ALL {
        let num = tab.index() + 1;
        let display_num = if num == 10 {
            "0".to_string()
        } else {
            num.to_string()
        };
        let label = format!(" {display_num}:{} ", tab.label());
        if *tab == current_tab {
            tab_spans.push(Span::styled(label, theme::active_tab_style()));
        } else {
            tab_spans.push(Span::styled(label, theme::inactive_tab_style()));
        }
        tab_spans.push(Span::raw(" "));
    }
    let tab_line = Line::from(tab_spans);
    frame.render_widget(
        Paragraph::new(tab_line).style(theme::header_style()),
        chunks[1],
    );
}
