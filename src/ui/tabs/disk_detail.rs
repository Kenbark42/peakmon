use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Row, Table};
use ratatui::Frame;

use crate::metrics::MetricsCollector;
use crate::ui::theme;
use crate::ui::widgets::sparkline_panel;
use crate::util::{format_bytes, format_percent, format_rate};

pub fn render(frame: &mut Frame, area: Rect, metrics: &MetricsCollector) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),   // Volume table
            Constraint::Length(6), // Read I/O sparkline
            Constraint::Length(6), // Write I/O sparkline
        ])
        .split(area);

    // Volume table
    let rows: Vec<Row> = metrics
        .disk
        .disks
        .iter()
        .map(|d| {
            Row::new(vec![
                d.name.clone(),
                d.mount_point.clone(),
                format_bytes(d.total_space),
                format_bytes(d.available_space),
                format_percent(d.used_percent),
            ])
        })
        .collect();

    let header = Row::new(vec!["Name", "Mount", "Total", "Available", "Used"])
        .style(theme::label_style())
        .height(1);

    let widths = [
        Constraint::Length(16),
        Constraint::Length(16),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(8),
    ];

    let block = Block::default()
        .title(Line::styled(" Volumes ", theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    let table = Table::new(rows, &widths).header(header).block(block);
    frame.render_widget(table, chunks[0]);

    // Read I/O sparkline
    let read_max = metrics.disk.read_history.max() as u64;
    let read_data = metrics.disk.read_history.as_u64_vec(area.width as usize);
    sparkline_panel::render(
        frame,
        chunks[1],
        "Disk Read",
        &read_data,
        Some(read_max.max(1)),
        theme::TEAL,
        &format_rate(metrics.disk.read_rate),
    );

    // Write I/O sparkline
    let write_max = metrics.disk.write_history.max() as u64;
    let write_data = metrics.disk.write_history.as_u64_vec(area.width as usize);
    sparkline_panel::render(
        frame,
        chunks[2],
        "Disk Write",
        &write_data,
        Some(write_max.max(1)),
        theme::PEACH,
        &format_rate(metrics.disk.write_rate),
    );
}
