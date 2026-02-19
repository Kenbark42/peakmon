use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Row, Table};
use ratatui::Frame;

use crate::metrics::MetricsCollector;
use crate::ui::theme;
use crate::ui::widgets::{metric_gauge, sparkline_panel};
use crate::util::{format_bytes, format_percent, format_rate};

pub fn render(frame: &mut Frame, area: Rect, metrics: &MetricsCollector) {
    let has_battery = metrics.battery.available;

    let mut constraints = vec![
        Constraint::Length(5), // CPU sparkline
        Constraint::Length(3), // Memory gauge
        Constraint::Length(3), // Swap gauge
    ];
    if has_battery {
        constraints.push(Constraint::Length(3)); // Battery gauge
    }
    constraints.push(Constraint::Min(6)); // Top processes table
    constraints.push(Constraint::Length(3)); // Network summary

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut idx = 0;

    // CPU sparkline
    let cpu_data = metrics
        .cpu
        .aggregate_history
        .as_u64_vec(area.width as usize);
    let cpu_label = format_percent(metrics.cpu.aggregate_usage);
    sparkline_panel::render(
        frame,
        main_chunks[idx],
        "CPU",
        &cpu_data,
        Some(100),
        theme::BLUE,
        &cpu_label,
    );
    idx += 1;

    // Memory gauge
    let mem_label = format!(
        "{} / {} ({})",
        format_bytes(metrics.memory.used_ram),
        format_bytes(metrics.memory.total_ram),
        format_percent(metrics.memory.ram_percent),
    );
    metric_gauge::render(
        frame,
        main_chunks[idx],
        "Memory",
        metrics.memory.ram_percent,
        &mem_label,
    );
    idx += 1;

    // Swap gauge
    let swap_label = format!(
        "{} / {} ({})",
        format_bytes(metrics.memory.used_swap),
        format_bytes(metrics.memory.total_swap),
        format_percent(metrics.memory.swap_percent),
    );
    metric_gauge::render(
        frame,
        main_chunks[idx],
        "Swap",
        metrics.memory.swap_percent,
        &swap_label,
    );
    idx += 1;

    // Battery gauge (conditional)
    if has_battery {
        let bat = &metrics.battery;
        let state = if bat.fully_charged {
            "Charged"
        } else if bat.is_charging {
            "Charging"
        } else {
            "Discharging"
        };
        let bat_label = format!(
            "{:.0}% \u{25cf} {}  {} cycles  Health: {:.1}%  {:.1}\u{00b0}C  {:.1}W",
            bat.percent, state, bat.cycle_count, bat.health_percent, bat.temperature, bat.watts,
        );
        metric_gauge::render(frame, main_chunks[idx], "Battery", bat.percent, &bat_label);
        idx += 1;
    }

    // Top processes
    let procs: Vec<Row> = metrics
        .processes
        .processes
        .iter()
        .take(10)
        .map(|p| {
            Row::new(vec![
                format!("{}", p.pid),
                p.name.clone(),
                format!("{:.1}%", p.cpu_usage),
                format_bytes(p.memory),
            ])
        })
        .collect();

    let header = Row::new(vec!["PID", "NAME", "CPU%", "MEM"])
        .style(theme::label_style())
        .height(1);

    let widths = [
        Constraint::Length(8),
        Constraint::Min(20),
        Constraint::Length(8),
        Constraint::Length(12),
    ];

    let proc_block = Block::default()
        .title(Line::styled(" Top Processes ", theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .style(Style::default().bg(theme::BASE));

    let table = Table::new(procs, &widths).header(header).block(proc_block);
    frame.render_widget(table, main_chunks[idx]);
    idx += 1;

    // Network summary
    let net_info = format!(
        " RX: {}  TX: {} ",
        format_rate(metrics.network.total_rx_rate),
        format_rate(metrics.network.total_tx_rate),
    );
    let net_block = Block::default()
        .title(Line::styled(" Network ", theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .style(Style::default().bg(theme::BASE));

    let net_para = ratatui::widgets::Paragraph::new(Line::styled(net_info, theme::value_style()))
        .block(net_block);
    frame.render_widget(net_para, main_chunks[idx]);
}
