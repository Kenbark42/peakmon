use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

use crate::metrics::ai::{AiMetrics, PullStatus};
use crate::metrics::MetricsCollector;
use crate::ui::theme;
use crate::ui::widgets::sparkline_panel;
use crate::util::{format_bytes, format_percent};

pub fn render(frame: &mut Frame, area: Rect, metrics: &MetricsCollector) {
    let ai = &metrics.ai;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // AI Services
            Constraint::Min(10),    // Models table
            Constraint::Length(10), // AI Processes
            Constraint::Length(5),  // AI Resource Usage sparkline
        ])
        .split(area);

    render_services(frame, chunks[0], ai);
    render_models(frame, chunks[1], ai);
    render_ai_processes(frame, chunks[2], ai);
    render_resource_usage(frame, chunks[3], ai, area.width);
}

fn render_services(frame: &mut Frame, area: Rect, ai: &AiMetrics) {
    let block = Block::default()
        .title(Line::styled(" AI Services ", theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    let mut spans: Vec<Span> = vec![Span::raw(" ")];
    for service in &ai.services {
        let dot = if service.detected { "● " } else { "○ " };
        let dot_color = if service.detected {
            theme::GREEN
        } else {
            theme::SURFACE1
        };
        spans.push(Span::styled(dot, Style::default().fg(dot_color)));
        spans.push(Span::styled(service.name, theme::value_style()));
        if let Some(ref ver) = service.version {
            spans.push(Span::styled(format!(" v{ver}"), theme::label_style()));
        }
        if let Some(pid) = service.pid {
            spans.push(Span::styled(format!(" [{pid}]"), theme::label_style()));
        }
        spans.push(Span::raw("  "));
    }

    let line = Line::from(spans);
    let p = Paragraph::new(line).block(block);
    frame.render_widget(p, area);
}

fn render_models(frame: &mut Frame, area: Rect, ai: &AiMetrics) {
    let pull_info = match &ai.pull_status {
        Some(PullStatus::Progress { status, percent }) => {
            let pct = percent.map(|p| format!(" {p:.0}%")).unwrap_or_default();
            format!(" — Pulling: {status}{pct}")
        }
        Some(PullStatus::Done) => " — Pull complete!".to_string(),
        Some(PullStatus::Error(e)) => format!(" — {e}"),
        None => String::new(),
    };

    let title = format!(" Ollama Models{pull_info} ");
    let block = Block::default()
        .title(Line::styled(title, theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    if !ai.ollama_available {
        let msg = Paragraph::new(Line::styled(
            " Ollama not detected — install from ollama.com",
            theme::label_style(),
        ))
        .block(block);
        frame.render_widget(msg, area);
        return;
    }

    if ai.ollama_models.is_empty() {
        let msg = Paragraph::new(Line::styled(
            " No models downloaded — press P to pull a model",
            theme::label_style(),
        ))
        .block(block);
        frame.render_widget(msg, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from(Span::styled("Name", theme::title_style())),
        Cell::from(Span::styled("Size", theme::title_style())),
        Cell::from(Span::styled("Quant", theme::title_style())),
        Cell::from(Span::styled("VRAM", theme::title_style())),
        Cell::from(Span::styled("Status", theme::title_style())),
    ])
    .height(1);

    let rows: Vec<Row> = ai
        .ollama_models
        .iter()
        .enumerate()
        .map(|(i, model)| {
            let quant = model
                .details
                .as_ref()
                .and_then(|d| d.quantization_level.as_deref())
                .unwrap_or("-");
            let vram = ai
                .model_vram(&model.name)
                .unwrap_or_else(|| "-".to_string());
            let status = ai.model_status(&model.name);
            let status_color = if status == "Loaded" {
                theme::GREEN
            } else {
                theme::SUBTEXT
            };

            let style = if i == ai.model_selected {
                theme::highlight_style()
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(Span::styled(&*model.name, theme::value_style())),
                Cell::from(Span::styled(format_bytes(model.size), theme::label_style())),
                Cell::from(Span::styled(quant.to_string(), theme::label_style())),
                Cell::from(Span::styled(vram, theme::label_style())),
                Cell::from(Span::styled(status, Style::default().fg(status_color))),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(35),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(block);

    frame.render_widget(table, area);
}

fn render_ai_processes(frame: &mut Frame, area: Rect, ai: &AiMetrics) {
    let block = Block::default()
        .title(Line::styled(" AI Processes ", theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    if ai.ai_processes.is_empty() {
        let msg = Paragraph::new(Line::styled(
            " No AI processes running",
            theme::label_style(),
        ))
        .block(block);
        frame.render_widget(msg, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from(Span::styled("PID", theme::title_style())),
        Cell::from(Span::styled("Name", theme::title_style())),
        Cell::from(Span::styled("CPU%", theme::title_style())),
        Cell::from(Span::styled("Memory", theme::title_style())),
        Cell::from(Span::styled("State", theme::title_style())),
    ])
    .height(1);

    let rows: Vec<Row> = ai
        .ai_processes
        .iter()
        .map(|p| {
            let state_color = theme::process_state_color(p.status);
            Row::new(vec![
                Cell::from(Span::styled(format!("{}", p.pid), theme::label_style())),
                Cell::from(Span::styled(&*p.name, theme::value_style())),
                Cell::from(Span::styled(
                    format!("{:.1}", p.cpu_usage),
                    theme::value_style(),
                )),
                Cell::from(Span::styled(format_bytes(p.memory), theme::label_style())),
                Cell::from(Span::styled(
                    p.status.label(),
                    Style::default().fg(state_color),
                )),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Percentage(35),
            Constraint::Length(8),
            Constraint::Percentage(20),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(block);

    frame.render_widget(table, area);
}

fn render_resource_usage(frame: &mut Frame, area: Rect, ai: &AiMetrics, width: u16) {
    let annotation = format!(
        "CPU: {}  Mem: {}",
        format_percent(ai.aggregate_cpu),
        format_bytes(ai.aggregate_memory)
    );

    let data = ai.cpu_history.as_u64_vec(width as usize);
    sparkline_panel::render(
        frame,
        area,
        "AI Resource Usage",
        &data,
        Some(100),
        theme::MAUVE,
        &annotation,
    );
}
