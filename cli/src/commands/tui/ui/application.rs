use clap::crate_version;
use ratatui::{
    prelude::{Backend, Rect},
    style::{self, Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
    Frame,
};

use crate::commands::tui::app::App;

pub struct ApplicationWidget;

impl ApplicationWidget {
    pub fn draw<B: Backend>(app: &App, frame: &mut Frame<B>, rect: Rect) {
        let current_application = app.state.current_application.clone();
        let current_namespace = app.state.current_namespace.clone();
        let current_version = app.state.current_version.clone();

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
                    current_namespace.or_else(|| Some("-".to_string())).unwrap(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(style::Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("Version: "),
                Span::styled(
                    current_version.or_else(|| Some("-".to_string())).unwrap(),
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

        frame.render_widget(application_widget, rect);
    }
}
