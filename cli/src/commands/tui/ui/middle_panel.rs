use crate::commands::tui::App;
use ratatui::{
    prelude::{Backend, Rect},
    Frame,
};

use crate::commands::tui::app::Block;
use crate::commands::tui::ui::logs::LogsWidget; // Add missing import
use crate::commands::tui::ui::database::DatabaseWidget; // Add missing import


pub struct MiddlePanelWidget;

impl MiddlePanelWidget {
    pub fn draw<B: Backend>(app: &mut App, frame: &mut Frame<B>, rect: Rect) {
        let current_active_middle_block= app.get_active_middle_block();
        match current_active_middle_block{
            Block::Database => {
                DatabaseWidget::draw(app, frame, rect);
            }
            Block::Log => {
                LogsWidget::draw(app, frame, rect);
            }
            Block::Build => {
                // Handle Build variant
            }
            Block::Dialog(_) => {
                // Handle Dialog variant
            }
            Block::Empty => {
                // Handle Empty variant
            }
            Block::Deployment => todo!(),
        }
    }
}
