use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

use crate::metrics::MetricsCollector;
use crate::ui::theme;
use crate::ui::widgets::sparkline_panel;
use crate::util::format_rate;

pub fn render(frame: &mut Frame, area: Rect, metrics: &MetricsCollector) {
    // Total RX/TX sparklines + per-interface sparklines
    let iface_count = metrics.network.interfaces.len();
    let total_sections = 2 + iface_count * 2; // total rx/tx + per-iface rx/tx

    let mut constraints: Vec<Constraint> = vec![
        Constraint::Length(6), // Total RX
        Constraint::Length(6), // Total TX
    ];

    for _ in 0..iface_count {
        constraints.push(Constraint::Length(5)); // iface RX
        constraints.push(Constraint::Length(5)); // iface TX
    }

    // If not enough space, just show totals
    if area.height < total_sections as u16 * 5 {
        constraints = vec![Constraint::Length(6), Constraint::Min(6)];
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(&constraints)
        .split(area);

    let width = area.width as usize;

    // Total RX
    let rx_max = metrics.network.total_rx_history.max() as u64;
    let rx_data = metrics.network.total_rx_history.as_u64_vec(width);
    if !chunks.is_empty() {
        sparkline_panel::render(
            frame,
            chunks[0],
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
    if chunks.len() > 1 {
        sparkline_panel::render(
            frame,
            chunks[1],
            "Total TX",
            &tx_data,
            Some(tx_max.max(1)),
            theme::BLUE,
            &format_rate(metrics.network.total_tx_rate),
        );
    }

    // Per-interface if space permits
    let mut chunk_idx = 2;
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
