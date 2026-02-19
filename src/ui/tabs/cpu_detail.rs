use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{BarChart, Block, Borders};
use ratatui::Frame;

use crate::metrics::MetricsCollector;
use crate::ui::theme;
use crate::ui::widgets::sparkline_panel;
use crate::util::format_percent;

pub fn render(frame: &mut Frame, area: Rect, metrics: &MetricsCollector) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Aggregate sparkline
            Constraint::Min(8),    // Per-core bar chart
        ])
        .split(area);

    // Aggregate CPU sparkline
    let cpu_data = metrics
        .cpu
        .aggregate_history
        .as_u64_vec(area.width as usize);
    let cpu_label = format_percent(metrics.cpu.aggregate_usage);
    sparkline_panel::render(
        frame,
        chunks[0],
        "CPU (aggregate)",
        &cpu_data,
        Some(100),
        theme::BLUE,
        &cpu_label,
    );

    // Per-core bar chart
    let core_labels: Vec<String> = (0..metrics.cpu.core_count)
        .map(|i| format!("C{i}"))
        .collect();
    let bar_data: Vec<(&str, u64)> = core_labels
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let usage = metrics.cpu.per_core_usage.get(i).copied().unwrap_or(0.0);
            (label.as_str(), usage as u64)
        })
        .collect();

    let bar_block = Block::default()
        .title(Line::styled(
            format!(" Per-Core Usage ({} cores) ", metrics.cpu.core_count),
            theme::title_style(),
        ))
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .style(Style::default().bg(theme::BASE));

    let bar_width = if metrics.cpu.core_count > 0 {
        let available = area.width.saturating_sub(2) as usize;
        let per_bar = available / metrics.cpu.core_count.max(1);
        per_bar.clamp(1, 6) as u16
    } else {
        3
    };

    let barchart = BarChart::default()
        .block(bar_block)
        .data(&bar_data)
        .bar_width(bar_width)
        .bar_gap(1)
        .bar_style(Style::default().fg(theme::BLUE))
        .value_style(Style::default().fg(theme::TEXT))
        .max(100);

    frame.render_widget(barchart, chunks[1]);
}
