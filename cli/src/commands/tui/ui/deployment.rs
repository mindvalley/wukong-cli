use ratatui::{
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
};

use crate::commands::tui::app::App;

pub struct DeploymentWidget {
    pub widget: Paragraph<'static>,
}

impl DeploymentWidget {
    pub fn new(app: &App) -> Self {
        let deployment_block = Block::default()
            .title(" Deployment ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightBlue));

        Self {
            widget: Paragraph::new(Text::styled("", Style::default().fg(Color::LightBlue)))
                .block(deployment_block),
        }
    }
}
