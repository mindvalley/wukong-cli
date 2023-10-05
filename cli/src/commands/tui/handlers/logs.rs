use wukong_sdk::services::gcloud::google::logging::r#type::LogSeverity;

use crate::commands::tui::{
    action::Action,
    app::{App, AppReturn},
    events::{key::Key, network::NetworkEvent},
};

use super::common_key_events;

pub async fn handler(key: Key, app: &mut App) -> AppReturn {
    match key {
        key if common_key_events::up_event(key) => {
            let new_scroll_position = app.state.logs_vertical_scroll.saturating_sub(5);
            handle_vertical_scroll(app, new_scroll_position)
        }
        key if common_key_events::down_event(key) => {
            let new_scroll_position = app.state.logs_vertical_scroll.saturating_add(5);
            handle_vertical_scroll(app, new_scroll_position)
        }
        key if common_key_events::left_event(key) => {
            let new_scroll_position = app.state.logs_horizontal_scroll.saturating_sub(5);
            handle_horizontal_scroll(app, new_scroll_position)
        }
        key if common_key_events::right_event(key) => {
            let new_scroll_position = app.state.logs_horizontal_scroll.saturating_add(5);
            handle_horizontal_scroll(app, new_scroll_position)
        }
        key if Action::from_key(key) == Some(Action::ToggleLogsTailing) => {
            app.state.logs_tailing = !app.state.logs_tailing;
        }
        key if Action::from_key(key) == Some(Action::ShowErrorAndAbove) => {
            handle_show_error_and_above(app).await;
        }
        key if Action::from_key(key) == Some(Action::SearchLogs) => {
            app.state.show_search_bar = !app.state.show_search_bar;
            if app.state.show_search_bar {
                app.state.show_filter_bar = false;
            }
        }
        key if Action::from_key(key) == Some(Action::FilterLogs) => {
            app.state.show_filter_bar = !app.state.show_filter_bar;
            if app.state.show_filter_bar {
                app.state.show_search_bar = false;
            }
        }
        _ => {}
    };

    AppReturn::Continue
}

async fn handle_show_error_and_above(app: &mut App) {
    app.dispatch(NetworkEvent::GetGCloudLogs).await;

    app.state.is_fetching_log_entries = true;
    app.state.start_polling_log_entries = false;

    app.state.log_entries = vec![];
    app.state.log_entries_length = 0;
    // Need to reset scroll, or else it will be out of bound

    // Add if not already in the list
    // or else remove it
    app.state.logs_severity = match app.state.logs_severity {
        Some(LogSeverity::Error) => None,
        _ => Some(LogSeverity::Error),
    };
}

fn handle_vertical_scroll(app: &mut App, new_scroll_position: usize) {
    app.state.logs_vertical_scroll = new_scroll_position;
    app.state.logs_vertical_scroll_state = app
        .state
        .logs_vertical_scroll_state
        .position(app.state.logs_vertical_scroll as u16);

    app.state.logs_enable_auto_scroll_to_bottom = false;
}

fn handle_horizontal_scroll(app: &mut App, new_scroll_position: usize) {
    app.state.logs_horizontal_scroll = new_scroll_position;
    app.state.logs_horizontal_scroll_state = app
        .state
        .logs_horizontal_scroll_state
        .position(app.state.logs_horizontal_scroll as u16);

    app.state.logs_enable_auto_scroll_to_bottom = false;
}

#[derive(Default)]
pub struct Input {
    /// Current value of the input box
    pub input: String,
    /// Position of cursor in the editor area.
    pub cursor_position: usize,
}

impl Input {
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        // self.input.insert(self.cursor_position, new_char);
        self.input.push(new_char);

        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }

    fn handle_input(&mut self, key: Key) -> bool {
        match key {
            Key::Char(to_insert) => {
                self.enter_char(to_insert);
                true
            }
            Key::Backspace => {
                self.delete_char();
                true
            }
            Key::Left => {
                self.move_cursor_left();
                true
            }
            Key::Right => {
                self.move_cursor_right();
                true
            }
            Key::Esc => {
                self.input = "".to_string();
                self.reset_cursor();
                false
            }
            _ => true,
        }
    }
}
