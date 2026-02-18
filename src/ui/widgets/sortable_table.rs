use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Row, Table};
use ratatui::Frame;

use crate::ui::theme;

pub struct SortableColumn {
    pub title: String,
    pub width: Constraint,
    pub is_sorted: bool,
    pub ascending: bool,
}

impl SortableColumn {
    pub fn header_text(&self) -> String {
        if self.is_sorted {
            let arrow = if self.ascending { " ^" } else { " v" };
            format!("{}{}", self.title, arrow)
        } else {
            self.title.clone()
        }
    }
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    columns: &[SortableColumn],
    rows: Vec<Row>,
    highlight_index: Option<usize>,
) {
    let header_cells: Vec<String> = columns.iter().map(|c| c.header_text()).collect();
    let header = Row::new(header_cells).style(theme::label_style()).height(1);

    let widths: Vec<Constraint> = columns.iter().map(|c| c.width).collect();

    let block = Block::default()
        .title(Line::styled(format!(" {title} "), theme::title_style()))
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    let table = Table::new(rows, &widths)
        .header(header)
        .block(block)
        .row_highlight_style(theme::highlight_style());

    if let Some(_idx) = highlight_index {
        let mut state = ratatui::widgets::TableState::default().with_selected(Some(_idx));
        frame.render_stateful_widget(table, area, &mut state);
    } else {
        frame.render_widget(table, area);
    }
}
