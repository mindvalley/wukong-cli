use ratatui::{
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
};

use crate::commands::tui::app::App;

pub struct BuildsWidget {
    pub widget: Paragraph<'static>,
}

impl BuildsWidget {
    pub fn new(app: &App) -> Self {
        let builds_block = Block::default()
            .title(" Build Artifacts ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightYellow));

        Self {
            widget: Paragraph::new(Text::styled("", Style::default().fg(Color::LightYellow)))
                .block(builds_block),
        }
    }
}
