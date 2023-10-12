use super::common_key_events;
use crate::commands::tui::{
    app::{ActiveBlock, App, AppReturn, DialogContext, Input},
    events::key::Key,
};

pub async fn handler(key: Key, app: &mut App) -> AppReturn {
    match key {
        key if common_key_events::back_event(key) => {
            app.set_current_route_state(
                Some(ActiveBlock::Log),
                Some(ActiveBlock::Dialog(DialogContext::LogIncludeFilter)),
            );
        }
        key if common_key_events::delete_event(key) => delete_char(app),
        Key::Right => move_cursor_right(&mut app.state.filter_bar_exclude_input),
        Key::Left => move_cursor_left(app),
        Key::Tab => move_to_next_input(app),
        Key::Char(new_char) => {
            enter_char(&mut app.state.filter_bar_exclude_input, new_char);
        }
        _ => {}
    };

    AppReturn::Continue
}

fn move_to_next_input(app: &mut App) {
    app.set_current_route_state(
        Some(ActiveBlock::Dialog(DialogContext::LogIncludeFilter)),
        Some(ActiveBlock::Dialog(DialogContext::LogIncludeFilter)),
    );
}

fn move_cursor_left(app: &mut App) {
    let cursor_moved_left = app
        .state
        .filter_bar_exclude_input
        .cursor_position
        .saturating_sub(1);

    // If the cursor is at the beginning move to the include input:
    if cursor_moved_left != 0 {
        app.state.filter_bar_exclude_input.cursor_position =
            clamp_cursor(&mut app.state.filter_bar_exclude_input, cursor_moved_left);
    }
}

fn clamp_cursor(filter_bar_exclude_input: &mut Input, new_cursor_pos: usize) -> usize {
    new_cursor_pos.clamp(0, filter_bar_exclude_input.input.len())
}

fn move_cursor_right(filter_bar_exclude_input: &mut Input) {
    let cursor_moved_right = filter_bar_exclude_input.cursor_position.saturating_add(1);
    filter_bar_exclude_input.cursor_position =
        clamp_cursor(filter_bar_exclude_input, cursor_moved_right);
}

pub fn enter_char(filter_bar_exclude_input: &mut Input, new_char: char) {
    filter_bar_exclude_input.input.push(new_char);
    move_cursor_right(filter_bar_exclude_input);
}

fn delete_char(app: &mut App) {
    let is_not_cursor_leftmost = app.state.filter_bar_exclude_input.cursor_position != 0;
    if is_not_cursor_leftmost {
        // Method "remove" is not used on the saved text for deleting the selected char.
        // Reason: Using remove on String works on bytes instead of the chars.
        // Using remove would require special care because of char boundaries.

        let current_index = app.state.filter_bar_exclude_input.cursor_position;
        let from_left_to_current_index = current_index - 1;

        // Getting all characters before the selected character.
        let before_char_to_delete = app
            .state
            .filter_bar_exclude_input
            .input
            .chars()
            .take(from_left_to_current_index);
        // Getting all characters after selected character.

        let after_char_to_delete = app
            .state
            .filter_bar_exclude_input
            .input
            .chars()
            .skip(current_index);
        // Put all characters together except the selected one.
        // By leaving the selected one out, it is forgotten and therefore deleted.
        app.state.filter_bar_exclude_input.input =
            before_char_to_delete.chain(after_char_to_delete).collect();
        move_cursor_left(app);
    }
}

pub fn reset_cursor(filter_bar_exclude_input: &mut Input) {
    filter_bar_exclude_input.input = "".to_string();
    filter_bar_exclude_input.cursor_position = 0;
}
