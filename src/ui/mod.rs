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
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::{AiInputMode, App};
use tabs::Tab;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
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
        Tab::Ai => tabs::ai_detail::render(frame, app_layout.body, &app.metrics),
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

    // Help overlay
    if app.show_help {
        help::render(frame, area);
    }
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
