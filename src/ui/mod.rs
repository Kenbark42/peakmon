pub mod footer;
pub mod header;
pub mod help;
pub mod layout;
pub mod tabs;
pub mod theme;
pub mod widgets;

use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table};
use ratatui::Frame;

use crate::app::{AiInputMode, App};
use tabs::Tab;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Paint the entire screen with BASE background before anything else renders
    frame.render_widget(
        Block::default().style(Style::default().bg(theme::BASE)),
        area,
    );

    let app_layout = layout::compute_layout(area);

    // Header
    let hostname = app.hostname.as_str();
    let uptime = app.metrics.uptime();
    let load_avg = app.metrics.cpu.load_avg;
    header::render(
        frame,
        app_layout.header,
        app.current_tab,
        hostname,
        uptime,
        load_avg,
    );

    // Body - dispatch to current tab
    match app.current_tab {
        Tab::Dashboard => tabs::dashboard::render(frame, app_layout.body, &app.metrics),
        Tab::Cpu => tabs::cpu_detail::render(frame, app_layout.body, &app.metrics),
        Tab::Memory => tabs::memory_detail::render(frame, app_layout.body, &app.metrics),
        Tab::Disk => tabs::disk_detail::render(frame, app_layout.body, &app.metrics),
        Tab::Network => tabs::network_detail::render(frame, app_layout.body, &app.metrics),
        Tab::Processes => tabs::processes::render(
            frame,
            app_layout.body,
            &app.metrics,
            app.scroll_offset,
            app.process_selected,
        ),
        Tab::Logs => tabs::logs::render(frame, app_layout.body, &app.log_stream, app.scroll_offset),
        Tab::Gpu => tabs::gpu_detail::render(frame, app_layout.body, &app.metrics),
        Tab::Ai => {
            tabs::ai_detail::render(frame, app_layout.body, &app.metrics, app.ai_chat_scroll)
        }
        Tab::Temperatures => tabs::temperatures::render(frame, app_layout.body, &app.metrics),
    }

    // Footer
    footer::render(
        frame,
        app_layout.footer,
        app.current_tab,
        app.filter_mode,
        &app.filter_buffer,
        app.refresh_rate,
        &app.metrics.ai,
    );

    // Kill confirmation overlay
    if let Some((pid, ref name)) = app.confirm_kill {
        let popup = centered_rect(50, 5, area);
        frame.render_widget(Clear, popup);
        let text = Line::from(vec![
            Span::styled("Kill ", Style::default().fg(theme::RED)),
            Span::styled(format!("{name} (PID {pid})"), theme::value_style()),
            Span::styled("? ", Style::default().fg(theme::RED)),
            Span::styled("[y]es / [any] cancel", theme::label_style()),
        ]);
        let block = Block::default()
            .title(Line::styled(
                " Confirm Kill ",
                Style::default().fg(theme::RED),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::RED))
            .style(Style::default().bg(theme::BASE));
        let p = Paragraph::new(text).block(block);
        frame.render_widget(p, popup);
    }

    // AI delete confirmation overlay
    if let Some(ref model_name) = app.ai_confirm_delete {
        let popup = centered_rect(50, 5, area);
        frame.render_widget(Clear, popup);
        let text = Line::from(vec![
            Span::styled("Delete model ", Style::default().fg(theme::RED)),
            Span::styled(model_name.as_str(), theme::value_style()),
            Span::styled("? ", Style::default().fg(theme::RED)),
            Span::styled("[y]es / [any] cancel", theme::label_style()),
        ]);
        let block = Block::default()
            .title(Line::styled(
                " Confirm Delete ",
                Style::default().fg(theme::RED),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::RED))
            .style(Style::default().bg(theme::BASE));
        let p = Paragraph::new(text).block(block);
        frame.render_widget(p, popup);
    }

    // AI pull prompt overlay
    if app.ai_input_mode == AiInputMode::PullPrompt {
        let popup = centered_rect(50, 5, area);
        frame.render_widget(Clear, popup);
        let display = if app.ai_input_buffer.is_empty() {
            "e.g. llama3.2:3b".to_string()
        } else {
            format!("{}_", app.ai_input_buffer)
        };
        let text = Line::from(vec![
            Span::styled(" Model: ", theme::label_style()),
            Span::styled(display, theme::value_style()),
        ]);
        let block = Block::default()
            .title(Line::styled(" Pull Model ", theme::title_style()))
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .style(Style::default().bg(theme::BASE));
        let p = Paragraph::new(text).block(block);
        frame.render_widget(p, popup);
    }

    // AI chat input overlay
    if app.ai_input_mode == AiInputMode::ChatInput {
        let popup = bottom_bar(area, 3);
        frame.render_widget(Clear, popup);
        let display = if app.ai_input_buffer.is_empty() {
            "Type your message...".to_string()
        } else {
            format!("{}_", app.ai_input_buffer)
        };
        let model_name = app
            .metrics
            .ai
            .first_loaded_model_name()
            .unwrap_or_else(|| "model".to_string());
        let text = Line::from(vec![
            Span::styled(" > ", Style::default().fg(theme::GREEN)),
            Span::styled(display, theme::value_style()),
        ]);
        let block = Block::default()
            .title(Line::styled(
                format!(" Chat with {model_name} "),
                theme::title_style(),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::GREEN))
            .style(Style::default().bg(theme::BASE));
        let p = Paragraph::new(text).block(block);
        frame.render_widget(p, popup);
    }

    // AI search input overlay
    if app.ai_input_mode == AiInputMode::SearchInput {
        let popup = centered_rect(50, 5, area);
        frame.render_widget(Clear, popup);
        let display = if app.ai_input_buffer.is_empty() {
            "e.g. llama, qwen, phi".to_string()
        } else {
            format!("{}_", app.ai_input_buffer)
        };
        let text = Line::from(vec![
            Span::styled(" Search: ", theme::label_style()),
            Span::styled(display, theme::value_style()),
        ]);
        let block = Block::default()
            .title(Line::styled(
                " Search Ollama Library ",
                theme::title_style(),
            ))
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .style(Style::default().bg(theme::BASE));
        let p = Paragraph::new(text).block(block);
        frame.render_widget(p, popup);
    }

    // AI search results overlay
    if app.metrics.ai.show_search {
        render_search_overlay(frame, area, app);
    }

    // Help overlay
    if app.show_help {
        help::render(frame, area);
    }
}

fn render_search_overlay(frame: &mut Frame, area: Rect, app: &App) {
    let ai = &app.metrics.ai;
    let popup = centered_rect(65, 20, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(Line::styled(
            " Search Results — Enter to pull, S new search, Esc to close ",
            theme::title_style(),
        ))
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .style(Style::default().bg(theme::BASE));

    if let Some(ref status) = ai.search_status {
        let msg =
            Paragraph::new(Line::styled(format!(" {status}"), theme::label_style())).block(block);
        frame.render_widget(msg, popup);
        return;
    }

    if ai.search_results.is_empty() {
        let msg = Paragraph::new(Line::styled(" No results", theme::label_style())).block(block);
        frame.render_widget(msg, popup);
        return;
    }

    let header = Row::new(vec![
        Cell::from(Span::styled("Name", theme::title_style())),
        Cell::from(Span::styled("Description", theme::title_style())),
    ])
    .height(1);

    let rows: Vec<Row> = ai
        .search_results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let style = if i == ai.search_selected {
                theme::highlight_style()
            } else {
                Style::default()
            };

            // Truncate description to fit
            let desc = if result.description.len() > 50 {
                format!("{}...", &result.description[..47])
            } else {
                result.description.clone()
            };

            Row::new(vec![
                Cell::from(Span::styled(&*result.name, theme::value_style())),
                Cell::from(Span::styled(desc, theme::label_style())),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Percentage(35), Constraint::Percentage(65)],
    )
    .header(header)
    .block(block);

    frame.render_widget(table, popup);
}

fn centered_rect(width_pct: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .flex(Flex::Center)
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_pct) / 2),
            Constraint::Percentage(width_pct),
            Constraint::Percentage((100 - width_pct) / 2),
        ])
        .split(vertical[1]);
    horizontal[1]
}

fn bottom_bar(area: Rect, height: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(height)])
        .split(area);
    vertical[1]
}
