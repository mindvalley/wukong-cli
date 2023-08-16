use ratatui::{
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
};

use crate::commands::tui::app::App;

pub struct LogsWidget {
    pub widget: Paragraph<'static>,
}

impl LogsWidget {
    pub fn new(app: &App) -> Self {
        let logs_block = Block::default()
            .title(" Logs ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightGreen));

        Self {
            widget: Paragraph::new(Text::styled("", Style::default().fg(Color::LightGreen)))
                .block(logs_block),
        }
    }
}
