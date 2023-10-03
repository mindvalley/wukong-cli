use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::commands::tui::app::App;

pub struct HelpWidget;

impl HelpWidget {
    pub fn draw<B: Backend>(app: &App, frame: &mut Frame<B>, rect: Rect) {
        let height = rect.height;

        let key_style = Style::default()
            .fg(Color::LightCyan)
            .add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(Color::White);

        let rows = app
            .actions
            .chunks((height - 2) as usize)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|action| {
                        Row::new(vec![
                            Cell::from(Span::styled(action.keys()[0].to_string(), key_style)),
                            Cell::from(Span::styled(action.to_string(), desc_style)),
                        ])
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ]
                .as_ref(),
            )
            .split(rect);

        let middle_border = Borders::TOP | Borders::BOTTOM;
        let last_border = Borders::TOP | Borders::BOTTOM | Borders::RIGHT;

        let rows_len = rows.len();
        for (i, row) in rows.into_iter().enumerate() {
            let block = if i < areas.len() - 1 && i < rows_len - 1 {
                Block::default().borders(middle_border)
            } else {
                Block::default().borders(last_border)
            };

            let widget = Table::new(row)
                .block(block)
                .widths(&[Constraint::Length(8), Constraint::Min(21)])
                .column_spacing(1);

            if i < areas.len() {
                frame.render_widget(widget, areas[i]);
            }
        }
    }
}
