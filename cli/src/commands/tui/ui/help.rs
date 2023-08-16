use ratatui::{
    prelude::Constraint,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::commands::tui::app::App;

pub struct HelpWidget {
    pub widget: Table<'static>,
}

impl HelpWidget {
    pub fn new(app: &App) -> Self {
        let key_style = Style::default()
            .fg(Color::LightCyan)
            .add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(Color::White);

        let rows = app
            .actions
            .iter()
            .map(|action| {
                Row::new(vec![
                    Cell::from(Span::styled(action.keys()[0].to_string(), key_style)),
                    Cell::from(Span::styled(action.to_string(), desc_style)),
                ])
            })
            .collect::<Vec<_>>();

        Self {
            widget: Table::new(rows)
                .block(Block::default().borders(Borders::TOP | Borders::BOTTOM | Borders::RIGHT))
                .widths(&[Constraint::Length(4), Constraint::Min(20)])
                .column_spacing(1),
        }
    }
}
