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

        let mut rows = app
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

        // for now, we create 3 columns for the hotkey list
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

        // If the hotkeys list only need less spaces than the available spaces,
        // add empty `vec![]` to the list so the layout will be draw perfectly.
        // Otherwise, remove the excess rows so we don't break the layout.
        if rows.len() < areas.len() {
            // if `areas.len()` is 3,
            //    `rows.len()` is 2,
            // this will add 1 `vec![]` to the `rows`.
            (0..(areas.len() - rows.len())).for_each(|_| rows.push(vec![]));
        } else {
            // if `areas.len()` is 3,
            //    `rows.len()` is 4,
            // this will remove 1 `Vec<Row>` frow the `rows`.
            (0..(rows.len() - areas.len())).for_each(|_| {
                rows.pop();
            });
        }

        for (i, row) in rows.into_iter().enumerate() {
            let block = if i < areas.len() - 1 {
                Block::default().borders(middle_border)
            } else {
                Block::default().borders(last_border)
            };

            let widget = Table::new(row)
                .block(block)
                .widths(&[Constraint::Length(8), Constraint::Min(21)])
                .column_spacing(1);

            frame.render_widget(widget, areas[i]);
        }
    }
}
