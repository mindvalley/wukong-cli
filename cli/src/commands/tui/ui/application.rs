use clap::crate_version;
use ratatui::{
    style::{self, Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
};

use crate::commands::tui::app::App;

pub struct ApplicationWidget {
    pub widget: Paragraph<'static>,
}

impl ApplicationWidget {
    pub fn new(app: &App) -> Self {
        let current_application = app.state.current_application.clone();
        let current_namespace = app.state.current_namespace.clone();

        let application_block = Block::default()
            .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
            .padding(Padding::new(1, 0, 0, 0))
            .style(Style::default());

        let application_widget = Paragraph::new(vec![
            Line::from(vec![
                Span::raw("Application: "),
                Span::styled(
                    current_application,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(style::Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("Namespace: "),
                Span::styled(
                    current_namespace,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(style::Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("CLI Version: "),
                Span::styled(
                    crate_version!(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(style::Modifier::BOLD),
                ),
            ]),
        ])
        .wrap(Wrap { trim: true })
        .block(application_block);

        Self {
            widget: application_widget,
        }
    }
}
