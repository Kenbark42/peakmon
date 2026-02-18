use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;

use crate::metrics::MetricsCollector;
use crate::ui::theme;
use crate::ui::widgets::sparkline_panel;
use crate::util::{format_bytes, format_percent, format_rate};

pub fn render(frame: &mut Frame, area: Rect, metrics: &MetricsCollector) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),    // Volume table
            Constraint::Length(6), // Read I/O sparkline
            Constraint::Length(6), // Write I/O sparkline
        ])
        .split(area);

    // Volume table with per-disk I/O
    let rows: Vec<Row> = metrics
        .disk
        .disks
        .iter()
        .map(|d| {
            Row::new(vec![
                Cell::from(Span::raw(d.name.clone())),
                Cell::from(Span::raw(d.mount_point.clone())),
                Cell::from(Span::raw(format_bytes(d.total_space))),
                Cell::from(Span::raw(format_bytes(d.available_space))),
                Cell::from(Span::styled(
                    format_percent(d.used_percent),
                    theme::gauge_style(d.used_percent),
                )),
                Cell::from(Span::styled(format_rate(d.read_rate), theme::value_style())),
                Cell::from(Span::styled(
                    format_rate(d.write_rate),
                    theme::value_style(),
                )),
            ])
        })
        .collect();

    let header = Row::new(vec![
        "Name", "Mount", "Total", "Avail", "Used", "Read/s", "Write/s",
    ])
    .style(theme::label_style())
    .height(1);

    let widths = [
        Constraint::Length(16),
        Constraint::Min(12),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(7),
        Constraint::Length(10),
        Constraint::Length(10),
    ];

    let block = Block::default()
        .title(Line::styled(" Volumes ", theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    let table = Table::new(rows, &widths).header(header).block(block);
    frame.render_widget(table, chunks[0]);

    // Read I/O sparkline (aggregate)
    let read_max = metrics.disk.read_history.max() as u64;
    let read_data = metrics.disk.read_history.as_u64_vec(area.width as usize);
    sparkline_panel::render(
        frame,
        chunks[1],
        "Disk Read (total)",
        &read_data,
        Some(read_max.max(1)),
        theme::TEAL,
        &format_rate(metrics.disk.read_rate),
    );

    // Write I/O sparkline (aggregate)
    let write_max = metrics.disk.write_history.max() as u64;
    let write_data = metrics.disk.write_history.as_u64_vec(area.width as usize);
    sparkline_panel::render(
        frame,
        chunks[2],
        "Disk Write (total)",
        &write_data,
        Some(write_max.max(1)),
        theme::PEACH,
        &format_rate(metrics.disk.write_rate),
    );
}
