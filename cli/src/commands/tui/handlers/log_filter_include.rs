use super::common_key_events;
use crate::commands::tui::{
    app::{ActiveBlock, App, AppReturn, DialogContext, Input},
    events::key::Key,
};

pub async fn handler(key: Key, app: &mut App) -> AppReturn {
    match key {
        key if common_key_events::back_event(key) => {
            app.state.show_filter_bar = false;
            app.set_current_route_state(
                Some(ActiveBlock::Log),
                Some(ActiveBlock::Dialog(DialogContext::LogIncludeFilter)),
            );
        }
        key if common_key_events::delete_event(key) => {
            delete_char(&mut app.state.filter_bar_include_input)
        }
        Key::Right => move_cursor_right(app),
        Key::Left => move_cursor_left(&mut app.state.filter_bar_include_input),
        Key::Tab => move_to_next_input(app),
        Key::Char(new_char) => {
            enter_char(app, new_char);
        }
        _ => {}
    };

    AppReturn::Continue
}

fn move_to_next_input(app: &mut App) {
    app.set_current_route_state(
        Some(ActiveBlock::Dialog(DialogContext::LogExcludeFilter)),
        Some(ActiveBlock::Dialog(DialogContext::LogExcludeFilter)),
    );
}

fn move_cursor_left(filter_bar_include_input: &mut Input) {
    let cursor_moved_left = filter_bar_include_input.cursor_position.saturating_sub(1);
    filter_bar_include_input.cursor_position =
        clamp_cursor(filter_bar_include_input, cursor_moved_left);
}

fn clamp_cursor(filter_bar_include_input: &mut Input, new_cursor_pos: usize) -> usize {
    new_cursor_pos.clamp(0, filter_bar_include_input.input.len())
}

fn move_cursor_right(app: &mut App) {
    let cursor_moved_right = app
        .state
        .filter_bar_include_input
        .cursor_position
        .saturating_add(1);

    // if corsor is at the end move to exclude input:
    if cursor_moved_right > app.state.filter_bar_include_input.input.len() {
        app.set_current_route_state(
            Some(ActiveBlock::Dialog(DialogContext::LogExcludeFilter)),
            Some(ActiveBlock::Dialog(DialogContext::LogExcludeFilter)),
        );
    } else {
        app.state.filter_bar_include_input.cursor_position =
            clamp_cursor(&mut app.state.filter_bar_include_input, cursor_moved_right);
    }
}

pub fn enter_char(app: &mut App, new_char: char) {
    app.state.filter_bar_include_input.input.push(new_char);
    move_cursor_right(app);
}

fn delete_char(filter_bar_include_input: &mut Input) {
    let is_not_cursor_leftmost = filter_bar_include_input.cursor_position != 0;
    if is_not_cursor_leftmost {
        // Method "remove" is not used on the saved text for deleting the selected char.
        // Reason: Using remove on String works on bytes instead of the chars.
        // Using remove would require special care because of char boundaries.

        let current_index = filter_bar_include_input.cursor_position;
        let from_left_to_current_index = current_index - 1;

        // Getting all characters before the selected character.
        let before_char_to_delete = filter_bar_include_input
            .input
            .chars()
            .take(from_left_to_current_index);
        // Getting all characters after selected character.
        let after_char_to_delete = filter_bar_include_input.input.chars().skip(current_index);

        // Put all characters together except the selected one.
        // By leaving the selected one out, it is forgotten and therefore deleted.
        filter_bar_include_input.input =
            before_char_to_delete.chain(after_char_to_delete).collect();
        move_cursor_left(filter_bar_include_input);
    }
}

pub fn reset_cursor(filter_bar_include_input: &mut Input) {
    filter_bar_include_input.input = "".to_string();
    filter_bar_include_input.cursor_position = 0;
}
