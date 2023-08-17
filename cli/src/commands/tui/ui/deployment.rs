use ratatui::{
    prelude::{Backend, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::commands::tui::app::App;

pub struct DeploymentWidget;

impl DeploymentWidget {
    pub fn draw<B: Backend>(_app: &App, frame: &mut Frame<B>, rect: Rect) {
        let deployment_block = Block::default()
            .title(" Deployment ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightBlue));

        let widget = Paragraph::new(Text::styled("", Style::default().fg(Color::LightBlue)))
            .block(deployment_block);
        frame.render_widget(widget, rect);
    }
}
