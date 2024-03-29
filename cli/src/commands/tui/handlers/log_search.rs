use super::common_key_events;
use crate::commands::tui::{
    app::{App, AppReturn, Block, DialogContext, Input},
    events::key::Key,
};

pub async fn handler(key: Key, app: &mut App) -> AppReturn {
    match key {
        key if common_key_events::back_event(key) => {
            app.state.show_search_bar = false;
            app.set_current_route_state(
                Some(Block::Middle(app.state.selected_tab)),
                Some(Block::Dialog(DialogContext::LogIncludeFilter)),
            );
        }
        key if common_key_events::delete_event(key) => delete_char(&mut app.state.search_bar_input),
        Key::Right => move_cursor_right(&mut app.state.search_bar_input),
        Key::Left => move_cursor_left(&mut app.state.search_bar_input),
        Key::Char(new_char) => {
            enter_char(&mut app.state.search_bar_input, new_char);
        }
        _ => {}
    };

    AppReturn::Continue
}

fn move_cursor_left(search_bar_input: &mut Input) {
    let cursor_moved_left = search_bar_input.cursor_position.saturating_sub(1);
    search_bar_input.cursor_position = clamp_cursor(search_bar_input, cursor_moved_left);
}

fn clamp_cursor(search_bar_input: &mut Input, new_cursor_pos: usize) -> usize {
    new_cursor_pos.clamp(0, search_bar_input.input.len())
}

fn move_cursor_right(search_bar_input: &mut Input) {
    let cursor_moved_right = search_bar_input.cursor_position.saturating_add(1);
    search_bar_input.cursor_position = clamp_cursor(search_bar_input, cursor_moved_right);
}

pub fn enter_char(search_bar_input: &mut Input, new_char: char) {
    search_bar_input.input.push(new_char);
    move_cursor_right(search_bar_input);
}

fn delete_char(search_bar_input: &mut Input) {
    let is_not_cursor_leftmost = search_bar_input.cursor_position != 0;
    if is_not_cursor_leftmost {
        // Method "remove" is not used on the saved text for deleting the selected char.
        // Reason: Using remove on String works on bytes instead of the chars.
        // Using remove would require special care because of char boundaries.

        let current_index = search_bar_input.cursor_position;
        let from_left_to_current_index = current_index - 1;

        // Getting all characters before the selected character.
        let before_char_to_delete = search_bar_input
            .input
            .chars()
            .take(from_left_to_current_index);
        // Getting all characters after selected character.
        let after_char_to_delete = search_bar_input.input.chars().skip(current_index);

        // Put all characters together except the selected one.
        // By leaving the selected one out, it is forgotten and therefore deleted.
        search_bar_input.input = before_char_to_delete.chain(after_char_to_delete).collect();
        move_cursor_left(search_bar_input);
    }
}

pub fn reset_cursor(search_bar_input: &mut Input) {
    search_bar_input.input = "".to_string();
    search_bar_input.cursor_position = 0;
}
