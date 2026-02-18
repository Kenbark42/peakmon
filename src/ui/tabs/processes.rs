use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Cell, Row};
use ratatui::Frame;

use crate::metrics::process::{ProcessInfo, ProcessSortField};
use crate::metrics::MetricsCollector;
use crate::ui::theme;
use crate::ui::widgets::sortable_table::{self, SortableColumn};
use crate::util::format_bytes;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    metrics: &MetricsCollector,
    scroll_offset: usize,
    selected: usize,
) {
    let sort = &metrics.processes;

    let columns = vec![
        SortableColumn {
            title: "PID".to_string(),
            width: Constraint::Length(8),
            is_sorted: sort.sort_field == ProcessSortField::Pid,
            ascending: sort.sort_ascending,
        },
        SortableColumn {
            title: "Name".to_string(),
            width: Constraint::Min(20),
            is_sorted: sort.sort_field == ProcessSortField::Name,
            ascending: sort.sort_ascending,
        },
        SortableColumn {
            title: "State".to_string(),
            width: Constraint::Length(7),
            is_sorted: false,
            ascending: false,
        },
        SortableColumn {
            title: "CPU%".to_string(),
            width: Constraint::Length(8),
            is_sorted: sort.sort_field == ProcessSortField::Cpu,
            ascending: sort.sort_ascending,
        },
        SortableColumn {
            title: "Memory".to_string(),
            width: Constraint::Length(12),
            is_sorted: sort.sort_field == ProcessSortField::Memory,
            ascending: sort.sort_ascending,
        },
    ];

    let (items, total_count): (Vec<&ProcessInfo>, usize) = if sort.tree_mode {
        // Tree mode returns owned ProcessInfo, so we collect refs differently
        // We'll handle tree mode separately below
        let filtered = sort.filtered_processes();
        let total = sort.processes.len();
        (filtered, total)
    } else {
        let filtered = sort.filtered_processes();
        let total = sort.processes.len();
        (filtered, total)
    };

    // For tree mode, get the tree data
    let tree_data;
    let display_items: Vec<&ProcessInfo> = if sort.tree_mode {
        tree_data = sort.tree_view();
        tree_data.iter().collect()
    } else {
        items
    };

    let visible_rows = area.height.saturating_sub(4) as usize;
    let clamped_offset = scroll_offset.min(display_items.len().saturating_sub(visible_rows));
    let rows: Vec<Row> = display_items
        .iter()
        .skip(clamped_offset)
        .take(visible_rows)
        .map(|p| {
            let state_color = theme::process_state_color(p.status);
            let name_display = if sort.tree_mode && p.depth > 0 {
                let indent = "  ".repeat(p.depth.min(8));
                format!("{indent}{}", p.name)
            } else {
                p.name.clone()
            };
            Row::new(vec![
                Cell::from(format!("{}", p.pid)),
                Cell::from(name_display),
                Cell::from(Span::styled(
                    p.status.label(),
                    Style::default().fg(state_color),
                )),
                Cell::from(format!("{:.1}%", p.cpu_usage)),
                Cell::from(format_bytes(p.memory)),
            ])
        })
        .collect();

    let mode_indicator = if sort.tree_mode { " [tree]" } else { "" };
    let title = if sort.filter.is_empty() {
        format!("Processes ({}){}", display_items.len(), mode_indicator)
    } else {
        format!(
            "Processes ({}/{}){} [filter: {}]",
            display_items.len(),
            total_count,
            mode_indicator,
            sort.filter
        )
    };

    let highlight = if display_items.is_empty() {
        None
    } else {
        Some(selected.saturating_sub(clamped_offset))
    };
    sortable_table::render(frame, area, &title, &columns, rows, highlight);
}
