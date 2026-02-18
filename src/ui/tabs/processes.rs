use ratatui::layout::{Constraint, Rect};
use ratatui::widgets::Row;
use ratatui::Frame;

use crate::metrics::process::ProcessSortField;
use crate::metrics::MetricsCollector;
use crate::ui::widgets::sortable_table::{self, SortableColumn};
use crate::util::format_bytes;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    metrics: &MetricsCollector,
    scroll_offset: usize,
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

    let filtered = sort.filtered_processes();
    let visible_rows = area.height.saturating_sub(4) as usize; // borders + header
    let rows: Vec<Row> = filtered
        .iter()
        .skip(scroll_offset)
        .take(visible_rows)
        .map(|p| {
            Row::new(vec![
                format!("{}", p.pid),
                p.name.clone(),
                format!("{:.1}%", p.cpu_usage),
                format_bytes(p.memory),
            ])
        })
        .collect();

    let title = if sort.filter.is_empty() {
        format!("Processes ({})", filtered.len())
    } else {
        format!(
            "Processes ({}/{}) [filter: {}]",
            filtered.len(),
            sort.processes.len(),
            sort.filter
        )
    };

    sortable_table::render(frame, area, &title, &columns, rows, None);
}
