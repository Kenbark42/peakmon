use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap};
use ratatui::Frame;

use crate::metrics::ai::{AiMetrics, ChatStatus, PullStatus};
use crate::metrics::MetricsCollector;
use crate::ui::theme;
use crate::ui::widgets::sparkline_panel;
use crate::util::{format_bytes, format_percent};

pub fn render(frame: &mut Frame, area: Rect, metrics: &MetricsCollector, chat_scroll: usize) {
    let ai = &metrics.ai;

    let has_chat = !ai.chat_messages.is_empty();
    let has_perf = ai.chat_metrics.is_some();

    // Dynamic layout: allocate space based on what content exists
    let perf_height = if has_perf { 3 } else { 0 };
    let chat_min = if has_chat { 6 } else { 3 };

    let mut constraints = vec![
        Constraint::Length(3),     // AI Services
        Constraint::Length(8),     // Models table (compact)
        Constraint::Min(chat_min), // Chat area (flexible)
    ];
    if has_perf {
        constraints.push(Constraint::Length(perf_height)); // Performance bar
    }
    constraints.push(Constraint::Length(3)); // AI Resource Usage sparkline

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut idx = 0;
    render_services(frame, chunks[idx], ai);
    idx += 1;
    render_models(frame, chunks[idx], ai);
    idx += 1;
    render_chat(frame, chunks[idx], ai, chat_scroll);
    idx += 1;
    if has_perf {
        render_performance(frame, chunks[idx], ai);
        idx += 1;
    }
    render_resource_usage(frame, chunks[idx], ai, area.width);
}

fn render_services(frame: &mut Frame, area: Rect, ai: &AiMetrics) {
    let block = Block::default()
        .title(Line::styled(" AI Services ", theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .style(Style::default().bg(theme::BASE));

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
        .border_style(theme::border_style())
        .style(Style::default().bg(theme::BASE));

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
        Cell::from(Span::styled("tok/s", theme::title_style())),
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

            let tps = ai
                .last_tps
                .get(&model.name)
                .map(|t| format!("{t:.1}"))
                .unwrap_or_else(|| "-".to_string());

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
                Cell::from(Span::styled(tps, Style::default().fg(theme::TEAL))),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(12),
            Constraint::Percentage(12),
            Constraint::Percentage(12),
            Constraint::Percentage(16),
            Constraint::Percentage(18),
        ],
    )
    .header(header)
    .block(block);

    frame.render_widget(table, area);
}

fn render_chat(frame: &mut Frame, area: Rect, ai: &AiMetrics, chat_scroll: usize) {
    let status_indicator = match &ai.chat_status {
        ChatStatus::Generating => " [generating...] ",
        ChatStatus::Error(e) => {
            // We'll show error in title - truncate if needed
            let _ = e; // used below
            " [error] "
        }
        _ => "",
    };

    let title_extra = if let Some(ref model) = ai.chat_model {
        format!(" Chat — {model}{status_indicator}")
    } else {
        format!(" Chat{status_indicator}")
    };

    let block = Block::default()
        .title(Line::styled(title_extra, theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .style(Style::default().bg(theme::BASE));

    if ai.chat_messages.is_empty() {
        let hint = if ai.has_loaded_model() {
            " Press i to chat with selected model"
        } else {
            " Load a model (Enter) then press i to chat"
        };
        let msg = Paragraph::new(Line::styled(hint, theme::label_style())).block(block);
        frame.render_widget(msg, area);
        return;
    }

    // Build chat lines
    let inner_width = area.width.saturating_sub(2) as usize; // account for borders
    let mut lines: Vec<Line> = Vec::new();

    for msg in &ai.chat_messages {
        let (prefix, prefix_style, content_style) = if msg.role == "user" {
            (
                "You: ",
                Style::default().fg(theme::BLUE),
                theme::value_style(),
            )
        } else {
            (
                "AI: ",
                Style::default().fg(theme::GREEN),
                theme::label_style(),
            )
        };

        if msg.content.is_empty() && msg.role == "assistant" {
            // Generating placeholder
            if ai.chat_status == ChatStatus::Generating {
                lines.push(Line::from(vec![
                    Span::styled(prefix, prefix_style),
                    Span::styled("...", Style::default().fg(theme::SURFACE1)),
                ]));
            }
            continue;
        }

        // Word-wrap the content manually for proper display
        let prefix_len = prefix.len();
        let wrap_width = if inner_width > prefix_len {
            inner_width - prefix_len
        } else {
            inner_width
        };

        let content_lines = wrap_text(&msg.content, wrap_width);

        for (i, cline) in content_lines.into_iter().enumerate() {
            if i == 0 {
                lines.push(Line::from(vec![
                    Span::styled(prefix, prefix_style),
                    Span::styled(cline, content_style),
                ]));
            } else {
                // Indent continuation lines to align with content
                let indent = " ".repeat(prefix_len);
                lines.push(Line::from(vec![
                    Span::raw(indent),
                    Span::styled(cline, content_style),
                ]));
            }
        }

        // Add blank line between messages
        lines.push(Line::raw(""));
    }

    // Show error message if any
    if let ChatStatus::Error(ref e) = ai.chat_status {
        lines.push(Line::from(Span::styled(
            format!("Error: {e}"),
            Style::default().fg(theme::RED),
        )));
    }

    // Scroll with support for user-controlled offset from bottom
    let visible_height = area.height.saturating_sub(2) as usize; // borders
    let total_lines = lines.len();
    let max_scroll = total_lines.saturating_sub(visible_height);
    // chat_scroll is offset from bottom: 0 = follow bottom, higher = further up
    let clamped_chat_scroll = chat_scroll.min(max_scroll);
    let scroll = max_scroll.saturating_sub(clamped_chat_scroll);

    // Show scroll position hint in title when not at bottom
    let block = if clamped_chat_scroll > 0 {
        let pct = if max_scroll > 0 {
            ((max_scroll - clamped_chat_scroll) * 100) / max_scroll
        } else {
            100
        };
        block.title_bottom(Line::styled(
            format!(" {pct}% ↓j ↑k "),
            theme::label_style(),
        ))
    } else {
        block
    };

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));

    frame.render_widget(paragraph, area);
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }

    let mut result = Vec::new();
    for line in text.split('\n') {
        if line.is_empty() {
            result.push(String::new());
            continue;
        }

        let mut current = String::new();
        for word in line.split_whitespace() {
            if current.is_empty() {
                current = word.to_string();
            } else if current.len() + 1 + word.len() <= width {
                current.push(' ');
                current.push_str(word);
            } else {
                result.push(current);
                current = word.to_string();
            }
        }
        if !current.is_empty() {
            result.push(current);
        }
    }

    if result.is_empty() {
        result.push(String::new());
    }
    result
}

fn render_performance(frame: &mut Frame, area: Rect, ai: &AiMetrics) {
    let block = Block::default()
        .title(Line::styled(" Performance ", theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .style(Style::default().bg(theme::BASE));

    if let Some(ref m) = ai.chat_metrics {
        let total_secs = m.total_duration_ms / 1000.0;
        let load_secs = m.load_duration_ms / 1000.0;
        let mut spans = vec![
            Span::styled(" ", Style::default()),
            Span::styled(
                format!("{:.1}", m.tokens_per_sec),
                Style::default().fg(theme::TEAL),
            ),
            Span::styled(" tok/s", theme::label_style()),
            Span::styled("  |  ", Style::default().fg(theme::SURFACE1)),
            Span::styled("TTFT ", theme::label_style()),
            Span::styled(
                format!("{:.0}ms", m.ttft_ms),
                Style::default().fg(theme::PEACH),
            ),
            Span::styled("  |  ", Style::default().fg(theme::SURFACE1)),
            Span::styled("Prompt ", theme::label_style()),
            Span::styled(format!("{}", m.prompt_tokens), theme::value_style()),
            Span::styled("  |  ", Style::default().fg(theme::SURFACE1)),
            Span::styled("Gen ", theme::label_style()),
            Span::styled(format!("{}", m.gen_tokens), theme::value_style()),
            Span::styled("  |  ", Style::default().fg(theme::SURFACE1)),
            Span::styled("Total ", theme::label_style()),
            Span::styled(format!("{total_secs:.1}s"), theme::value_style()),
        ];
        if load_secs > 0.1 {
            spans.extend([
                Span::styled("  |  ", Style::default().fg(theme::SURFACE1)),
                Span::styled("Load ", theme::label_style()),
                Span::styled(format!("{load_secs:.1}s"), theme::value_style()),
            ]);
        }

        let line = Line::from(spans);
        let p = Paragraph::new(line).block(block);
        frame.render_widget(p, area);
    } else {
        let p = Paragraph::new(Line::styled(" No metrics yet", theme::label_style())).block(block);
        frame.render_widget(p, area);
    }
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
        "AI Resources",
        &data,
        Some(100),
        theme::MAUVE,
        &annotation,
    );
}
