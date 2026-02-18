use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{BarChart, Block, Borders};
use ratatui::Frame;

use crate::metrics::MetricsCollector;
use crate::ui::theme;
use crate::ui::widgets::sparkline_panel;

pub fn render(frame: &mut Frame, area: Rect, metrics: &MetricsCollector) {
    if metrics.temperature.sensors.is_empty() {
        let block = Block::default()
            .title(Line::styled(" Temperatures ", theme::title_style()))
            .borders(Borders::ALL)
            .border_style(theme::border_style());

        let msg = ratatui::widgets::Paragraph::new(Line::styled(
            " No temperature sensors available. May require running with elevated privileges.",
            theme::label_style(),
        ))
        .block(block);
        frame.render_widget(msg, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),    // Sensor bar chart
            Constraint::Length(7), // Selected sensor sparkline
        ])
        .split(area);

    // Sensor bar chart
    let bar_data: Vec<(String, u64)> = metrics
        .temperature
        .sensors
        .iter()
        .map(|s| {
            let label = if s.label.len() > 10 {
                s.label[..10].to_string()
            } else {
                s.label.clone()
            };
            (label, s.temperature as u64)
        })
        .collect();

    let bar_refs: Vec<(&str, u64)> = bar_data.iter().map(|(l, v)| (l.as_str(), *v)).collect();

    let max_temp = metrics
        .temperature
        .sensors
        .iter()
        .map(|s| s.max_temperature)
        .fold(100.0_f64, f64::max) as u64;

    let selected = metrics.temperature.selected_sensor;
    let selected_label = metrics
        .temperature
        .sensors
        .get(selected)
        .map(|s| s.label.as_str())
        .unwrap_or("--");

    let bar_block = Block::default()
        .title(Line::styled(
            format!(" Sensors (selected: {selected_label}) "),
            theme::title_style(),
        ))
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    let barchart = BarChart::default()
        .block(bar_block)
        .data(&bar_refs)
        .bar_width(8)
        .bar_gap(1)
        .bar_style(Style::default().fg(theme::PEACH))
        .value_style(Style::default().fg(theme::TEXT))
        .max(max_temp.max(1));

    frame.render_widget(barchart, chunks[0]);

    // Selected sensor sparkline
    if let Some(sensor) = metrics.temperature.sensors.get(selected) {
        let data = sensor.history.as_u64_vec(area.width as usize);
        sparkline_panel::render(
            frame,
            chunks[1],
            &format!("{} History", sensor.label),
            &data,
            Some(max_temp.max(1)),
            theme::RED,
            &format!("{:.1} C", sensor.temperature),
        );
    }
}
