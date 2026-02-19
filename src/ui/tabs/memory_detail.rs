use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::metrics::MetricsCollector;
use crate::ui::theme;
use crate::ui::widgets::{metric_gauge, sparkline_panel};
use crate::util::{format_bytes, format_percent};

pub fn render(frame: &mut Frame, area: Rect, metrics: &MetricsCollector) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // RAM gauge
            Constraint::Length(3), // RAM breakdown info
            Constraint::Length(7), // RAM sparkline
            Constraint::Length(3), // Swap gauge
            Constraint::Min(7),    // Swap sparkline
        ])
        .split(area);

    // RAM gauge
    let ram_label = format!(
        "{} / {} ({})",
        format_bytes(metrics.memory.used_ram),
        format_bytes(metrics.memory.total_ram),
        format_percent(metrics.memory.ram_percent),
    );
    metric_gauge::render(
        frame,
        chunks[0],
        "RAM",
        metrics.memory.ram_percent,
        &ram_label,
    );

    // RAM breakdown: App / Wired / Compressed
    let breakdown = format!(
        " App: {}  Wired: {}  Compressed: {}",
        format_bytes(metrics.memory.app_memory),
        format_bytes(metrics.memory.wired),
        format_bytes(metrics.memory.compressed),
    );
    let info_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .style(Style::default().bg(theme::BASE));
    let info_para = Paragraph::new(Line::styled(breakdown, theme::value_style())).block(info_block);
    frame.render_widget(info_para, chunks[1]);

    // RAM history sparkline
    let ram_data = metrics.memory.ram_history.as_u64_vec(area.width as usize);
    sparkline_panel::render(
        frame,
        chunks[2],
        "RAM History",
        &ram_data,
        Some(100),
        theme::GREEN,
        &format_percent(metrics.memory.ram_percent),
    );

    // Swap gauge
    let swap_label = format!(
        "{} / {} ({})",
        format_bytes(metrics.memory.used_swap),
        format_bytes(metrics.memory.total_swap),
        format_percent(metrics.memory.swap_percent),
    );
    metric_gauge::render(
        frame,
        chunks[3],
        "Swap",
        metrics.memory.swap_percent,
        &swap_label,
    );

    // Swap history sparkline
    let swap_data = metrics.memory.swap_history.as_u64_vec(area.width as usize);
    sparkline_panel::render(
        frame,
        chunks[4],
        "Swap History",
        &swap_data,
        Some(100),
        theme::MAUVE,
        &format_percent(metrics.memory.swap_percent),
    );
}
