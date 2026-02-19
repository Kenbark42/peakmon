use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::metrics::MetricsCollector;
use crate::ui::theme;
use crate::ui::widgets::{metric_gauge, sparkline_panel};
use crate::util::format_bytes;

pub fn render(frame: &mut Frame, area: Rect, metrics: &MetricsCollector) {
    let gpu = &metrics.gpu;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // GPU info
            Constraint::Length(5), // Utilization sparkline
            Constraint::Length(3), // Renderer gauge
            Constraint::Length(3), // Tiler gauge
            Constraint::Length(3), // Memory info
            Constraint::Min(0),    // spacer
        ])
        .split(area);

    // GPU info
    let info_block = Block::default()
        .title(Line::styled(" GPU ", theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .style(Style::default().bg(theme::BASE));

    let info_text = Line::from(vec![
        Span::styled(" Model: ", theme::label_style()),
        Span::styled(&gpu.model, theme::value_style()),
        Span::styled("  Cores: ", theme::label_style()),
        Span::styled(format!("{}", gpu.core_count), theme::value_style()),
    ]);

    frame.render_widget(Paragraph::new(info_text).block(info_block), chunks[0]);

    // Device utilization sparkline
    let util_data = gpu.utilization_history.as_u64_vec(area.width as usize);
    sparkline_panel::render(
        frame,
        chunks[1],
        "Device Utilization",
        &util_data,
        Some(100),
        theme::GREEN,
        &format!("{:.0}%", gpu.device_utilization),
    );

    // Renderer gauge
    metric_gauge::render(
        frame,
        chunks[2],
        "Renderer",
        gpu.renderer_utilization,
        &format!("{:.0}%", gpu.renderer_utilization),
    );

    // Tiler gauge
    metric_gauge::render(
        frame,
        chunks[3],
        "Tiler",
        gpu.tiler_utilization,
        &format!("{:.0}%", gpu.tiler_utilization),
    );

    // Memory info
    let mem_block = Block::default()
        .title(Line::styled(" GPU Memory ", theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .style(Style::default().bg(theme::BASE));

    let mem_text = Line::from(vec![
        Span::styled(" In Use: ", theme::label_style()),
        Span::styled(format_bytes(gpu.in_use_memory), theme::value_style()),
        Span::styled("  Allocated: ", theme::label_style()),
        Span::styled(format_bytes(gpu.alloc_memory), theme::value_style()),
    ]);

    frame.render_widget(Paragraph::new(mem_text).block(mem_block), chunks[4]);
}
