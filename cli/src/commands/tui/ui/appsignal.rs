use ratatui::{backend::Backend, layout::Rect, Frame};

use crate::commands::tui::app::{App, Block};

pub struct AppsignalWidget;

impl AppsignalWidget {
    pub fn draw<B: Backend>(app: &mut App, frame: &mut Frame<B>, rect: Rect) {
        // app.state.logs_widget_width = rect.width;
        // app.state.logs_widget_height = rect.height;

        // app.update_draw_lock(Block::Log, rect);
    }
}
