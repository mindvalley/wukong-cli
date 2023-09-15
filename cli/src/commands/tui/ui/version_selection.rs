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

pub struct VersionSelectionWidget;

impl VersionSelectionWidget {
    pub fn draw<B: Backend>(app: &mut App, frame: &mut Frame<B>) {
        let items: Vec<ListItem> = app
            .version_selections
            .items
            .iter()
            .map(|i| {
                let lines = vec![Line::from(i.clone())];
                ListItem::new(lines).style(Style::default().fg(Color::White))
            })
            .collect();

        let popup_block = Block::default()
            .title(" Version Selection ")
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center)
            .style(Style::default().bg(Color::Black));

        let area = centered_rect(60, 25, frame.size());
        frame.render_widget(Clear, area);

        if app.state.is_checking_version {
            let loading_widget = create_loading_widget(popup_block);
            frame.render_widget(loading_widget, area);
        } else {
            let items = List::new(items)
                .block(popup_block)
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            frame.render_stateful_widget(items, area, &mut app.version_selections.state);
        }
    }

    pub async fn handle_input(key: Key, app: &mut App) {
        match key {
            Key::Up => app.version_selections.previous(),
            Key::Down => app.version_selections.next(),
            Key::Esc | Key::Char('q') => set_current_screen_to_main(app),
            Key::Enter => handle_enter_key(app).await,
            _ => {}
        }
    }
}

async fn handle_enter_key(app: &mut App) {
    let selected_version_index = match app.version_selections.state.selected() {
        Some(index) => index,
        None => return, // No selected version, nothing to do
    };

    let selected_version = app.version_selections.items[selected_version_index].clone();

    if selected_version == app.state.current_version {
        set_current_screen_to_main(app);
        return;
    }

    fetch_and_reset_polling(app, selected_version).await;
    set_current_screen_to_main(app);
}

async fn fetch_and_reset_polling(app: &mut App, selected_version: String) {
    app.state.current_version = selected_version;

    app.state.is_fetching_log_entries = true;
    app.state.start_polling_log_entries = false;
    app.state.has_log_errors = false;

    app.dispatch(NetworkEvent::FetchBuilds).await;
}

fn set_current_screen_to_main(app: &mut App) {
    app.current_screen = CurrentScreen::Main;
}
