use super::common_key_events;
use crate::commands::tui::{
    app::{App, AppReturn, Block, SelectedTab},
    events::key::Key,
};

pub async fn handler(key: Key, app: &mut App) -> AppReturn {
    match key {
        key if common_key_events::back_event(key) => {
            if let Some(Block::Middle(_)) = app.state.expanded_block {
                app.state.expanded_block = None;
            } else {
                app.set_current_route_state(
                    Some(Block::Empty),
                    Some(Block::Middle(app.state.selected_tab)),
                );
            }
        }
        Key::Char(character) => {
            // use number key to switch tab
            if let Some(digit) = character.to_digit(10) {
                if let Some(newly_selected_tab) = SelectedTab::get_tab(digit) {
                    app.state.selected_tab = newly_selected_tab;
                    app.set_current_route_state(
                        Some(Block::Middle(app.state.selected_tab)),
                        Some(Block::Middle(app.state.selected_tab)),
                    );
                }
            }
        }
        // This is not working for now as the Key Event has bug
        // https://github.com/crossterm-rs/crossterm/issues/727
        // Key::Ctrl(character) => {
        //     if let Some(digit) = character.to_digit(10) {
        //         if let Some(newly_selected_tab) = SelectedTab::get_tab(digit) {
        //             app.state.selected_tab = newly_selected_tab;
        //             app.set_current_route_state(
        //                 Some(Block::Middle(app.state.selected_tab)),
        //                 Some(Block::Middle(app.state.selected_tab)),
        //             );
        //         }
        //     }
        // }
        _ => {}
    };

    AppReturn::Continue
}
