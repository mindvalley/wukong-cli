use ratatui::{
    prelude::{Backend, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::commands::tui::app::App;

pub struct HelpWidget;

impl HelpWidget {
    pub fn draw<B: Backend>(app: &App, frame: &mut Frame<B>, rect: Rect) {
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

        let widget = Table::new(rows)
            .block(Block::default().borders(Borders::TOP | Borders::BOTTOM | Borders::RIGHT))
            .widths(&[Constraint::Length(4), Constraint::Min(20)])
            .column_spacing(1);

        frame.render_widget(widget, rect);
    }
}
