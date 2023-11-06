mod build;
mod common_key_events;
mod deployment;
mod empty;
mod log_filter_exclude;
mod log_filter_include;
mod log_search;
mod logs;
mod namespace_selection;
mod time_filter_selection;
mod version_selection;

use super::{
    app::{App, AppReturn, Block, DialogContext},
    events::key::Key,
};
use crossterm::event::{MouseEvent, MouseEventKind};

pub async fn input_handler(key: Key, app: &mut App) -> AppReturn {
    match key {
        key if common_key_events::exit_event(key) => AppReturn::Exit,
        _ => handle_block_events(key, app).await,
    }
}

async fn handle_block_events(key: Key, app: &mut App) -> AppReturn {
    let current_route = app.get_current_route();

    match current_route.active_block {
        Block::Empty => empty::handler(key, app).await, // Main Input
        Block::Log => logs::handler(key, app).await,
        Block::Dialog(DialogContext::NamespaceSelection) => {
            namespace_selection::handler(key, app).await
        }
        Block::Dialog(DialogContext::VersionSelection) => {
            version_selection::handler(key, app).await
        }
        Block::Dialog(DialogContext::LogSearch) => log_search::handler(key, app).await,
        Block::Dialog(DialogContext::LogIncludeFilter) => {
            log_filter_include::handler(key, app).await
        }
        Block::Dialog(DialogContext::LogExcludeFilter) => {
            log_filter_exclude::handler(key, app).await
        }
        Block::Dialog(DialogContext::LogTimeFilter) => {
            time_filter_selection::handler(key, app).await
        }
        Block::Deployment => deployment::handler(key, app).await,
        Block::Build => build::handler(key, app).await,
    }
}

pub fn handle_mouse_event(event: MouseEvent, app: &mut App) -> AppReturn {
    if let MouseEventKind::Down(button) = event.kind {
        let (x, y) = (event.column, event.row);
        match button {
            crossterm::event::MouseButton::Left => {
                // Trigger left click widget activity
                on_left_mouse_up(x, y, app);
            }
            crossterm::event::MouseButton::Right => {}
            _ => {}
        }
    };

    AppReturn::Continue
}

/// Moves the mouse to the widget that was clicked on, then propagates the click down to be
/// handled by the widget specifically.
fn on_left_mouse_up(x: u16, y: u16, app: &mut App) {
    // Iterate through the widget map and go to the widget where the click
    // is within.
    for (new_block_id, widget) in &app.block_map {
        if let (Some((tlc_x, tlc_y)), Some((brc_x, brc_y))) =
            (widget.top_left_corner, widget.bottom_right_corner)
        {
            if (x >= tlc_x && y >= tlc_y) && (x < brc_x && y < brc_y) {
                app.push_navigation_stack(*new_block_id);
                break;
            }
        }
    }
}
