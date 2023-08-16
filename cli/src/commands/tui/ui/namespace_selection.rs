use ratatui::{
    prelude::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::commands::tui::{app::App, events::key::Key, CurrentScreen};

use super::centered_rect;

pub struct NamespaceSelectionWidget {
    pub widget: Rect,
}

impl NamespaceSelectionWidget {
    pub fn new(app: &App, items: Vec<ListItem>, frame_rect: Rect) -> Self {
        let items: Vec<ListItem> = app
            .namespace_selections
            .items
            .iter()
            .map(|i| {
                let lines = vec![Line::from(i.clone())];
                ListItem::new(lines).style(Style::default().fg(Color::White))
            })
            .collect();

        let popup_block = Block::default()
            .title(" Namespace Selection ")
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center)
            .style(Style::default().bg(Color::Black));

        // Create a List from all list items and highlight the currently selected one
        let items = List::new(items)
            .block(popup_block)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        let area = centered_rect(60, 25, frame_rect);

        Self { widget: area }
    }

    pub fn handle_input(key: Key, app: &mut App) {
        match key {
            Key::Up => app.namespace_selections.previous(),
            Key::Down => app.namespace_selections.next(),
            Key::Esc | Key::Char('q') => app.current_screen = CurrentScreen::Main,
            Key::Enter => {
                app.state.current_namespace = app
                    .namespace_selections
                    .items
                    .get(app.namespace_selections.state.selected().unwrap())
                    .unwrap()
                    .clone();

                app.current_screen = CurrentScreen::Main;
            }
            _ => {}
        }
    }
}
