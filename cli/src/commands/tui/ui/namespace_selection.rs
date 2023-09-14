use ratatui::{
    prelude::{Alignment, Backend},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};

use crate::commands::tui::{
    app::App,
    events::{key::Key, network::NetworkEvent},
    CurrentScreen,
};

use super::{centered_rect, create_loading_widget};

pub struct NamespaceSelectionWidget;

impl NamespaceSelectionWidget {
    pub fn draw<B: Backend>(app: &mut App, frame: &mut Frame<B>) {
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

        let area = centered_rect(60, 25, frame.size());
        frame.render_widget(Clear, area);

        if app.state.is_checking_namespaces {
            let loading_widget = create_loading_widget(popup_block);
            frame.render_widget(loading_widget, area);
        } else {
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

            frame.render_stateful_widget(items, area, &mut app.namespace_selections.state);
        }
    }

    pub async fn handle_input(key: Key, app: &mut App) {
        match key {
            Key::Up => app.namespace_selections.previous(),
            Key::Down => app.namespace_selections.next(),
            Key::Esc | Key::Char('q') => app.current_screen = CurrentScreen::Main,
            Key::Enter => {
                let selected = app
                    .namespace_selections
                    .items
                    .get(app.namespace_selections.state.selected().unwrap())
                    .unwrap()
                    .clone();

                if app.state.current_namespace != selected {
                    app.state.current_namespace = selected;

                    app.dispatch(NetworkEvent::FetchBuilds).await;
                    // to re-trigger the polling
                    app.state.is_fetching_log_entries = true;
                    app.state.start_polling_log_entries = false;
                    app.state.has_log_errors = false;
                }

                app.current_screen = CurrentScreen::Main;
            }
            _ => {}
        }
    }
}
