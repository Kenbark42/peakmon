use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::metrics::MetricsCollector;
use crate::ui::theme;
use crate::ui::widgets::sparkline_panel;
use crate::util::format_rate;

pub fn render(frame: &mut Frame, area: Rect, metrics: &MetricsCollector) {
    // Connection summary + Total RX/TX sparklines + per-interface sparklines
    let iface_count = metrics.network.interfaces.len();

    let mut constraints: Vec<Constraint> = vec![
        Constraint::Length(3), // Connections summary
        Constraint::Length(6), // Total RX
        Constraint::Length(6), // Total TX
    ];

    for _ in 0..iface_count {
        constraints.push(Constraint::Length(5)); // iface RX
        constraints.push(Constraint::Length(5)); // iface TX
    }

    // If not enough space, just show connections + totals
    let total_sections = 3 + iface_count * 2;
    if area.height < total_sections as u16 * 4 {
        constraints = vec![
            Constraint::Length(3),
            Constraint::Length(6),
            Constraint::Min(6),
        ];
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(&constraints)
        .split(area);

    let width = area.width as usize;

    // Connection summary
    let conn = &metrics.network.connections;
    let conn_block = Block::default()
        .title(Line::styled(" TCP Connections ", theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .style(Style::default().bg(theme::BASE));

    let conn_line = Line::from(vec![
        Span::styled(" ESTABLISHED ", theme::label_style()),
        Span::styled(format!("{}", conn.established), theme::value_style()),
        Span::styled("  LISTEN ", theme::label_style()),
        Span::styled(format!("{}", conn.listen), theme::value_style()),
        Span::styled("  TIME_WAIT ", theme::label_style()),
        Span::styled(format!("{}", conn.time_wait), theme::value_style()),
        Span::styled("  CLOSE_WAIT ", theme::label_style()),
        Span::styled(format!("{}", conn.close_wait), theme::value_style()),
        Span::styled("  Total ", theme::label_style()),
        Span::styled(format!("{}", conn.total()), theme::value_style()),
    ]);

    frame.render_widget(Paragraph::new(conn_line).block(conn_block), chunks[0]);

    // Total RX
    let rx_max = metrics.network.total_rx_history.max() as u64;
    let rx_data = metrics.network.total_rx_history.as_u64_vec(width);
    if chunks.len() > 1 {
        sparkline_panel::render(
            frame,
            chunks[1],
            "Total RX",
            &rx_data,
            Some(rx_max.max(1)),
            theme::GREEN,
            &format_rate(metrics.network.total_rx_rate),
        );
    }

    // Total TX
    let tx_max = metrics.network.total_tx_history.max() as u64;
    let tx_data = metrics.network.total_tx_history.as_u64_vec(width);
    if chunks.len() > 2 {
        sparkline_panel::render(
            frame,
            chunks[2],
            "Total TX",
            &tx_data,
            Some(tx_max.max(1)),
            theme::BLUE,
            &format_rate(metrics.network.total_tx_rate),
        );
    }

    // Per-interface if space permits
    let mut chunk_idx = 3;
    for iface in &metrics.network.interfaces {
        if chunk_idx + 1 >= chunks.len() {
            break;
        }

        let irx_max = iface.rx_history.max() as u64;
        let irx_data = iface.rx_history.as_u64_vec(width);
        sparkline_panel::render(
            frame,
            chunks[chunk_idx],
            &format!("{} RX", iface.name),
            &irx_data,
            Some(irx_max.max(1)),
            theme::TEAL,
            &format_rate(iface.rx_rate),
        );

        let itx_max = iface.tx_history.max() as u64;
        let itx_data = iface.tx_history.as_u64_vec(width);
        sparkline_panel::render(
            frame,
            chunks[chunk_idx + 1],
            &format!("{} TX", iface.name),
            &itx_data,
            Some(itx_max.max(1)),
            theme::SKY,
            &format_rate(iface.tx_rate),
        );

        chunk_idx += 2;
    }
}
