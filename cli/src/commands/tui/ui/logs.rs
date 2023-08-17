use ratatui::{
    prelude::{Backend, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::commands::tui::app::App;

pub struct LogsWidget;

impl LogsWidget {
    pub fn draw<B: Backend>(_app: &App, frame: &mut Frame<B>, rect: Rect) {
        let logs_block = Block::default()
            .title(" Logs ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightGreen));

        let widget = Paragraph::new(Text::styled("", Style::default().fg(Color::LightGreen)))
            .block(logs_block);

        frame.render_widget(widget, rect);
    }
}
