use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Paragraph},
    Frame,
};

use self::{
    application::ApplicationWidget, builds::BuildsWidget, deployment::DeploymentWidget,
    help::HelpWidget, logs::LogsWidget, namespace_selection::NamespaceSelectionWidget,
    version_selection::VersionSelectionWidget,
};

use super::app::{ActiveBlock, App, DialogContext};

mod application;
mod builds;
mod deployment;
pub mod empty;
mod help;
pub mod logs;
pub mod namespace_selection;
pub mod util;
pub mod version_selection;

pub fn draw<B>(frame: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    // Create the layout sections.
    let [top, mid, bottom] = *Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Ratio(1, 7),
                Constraint::Ratio(3, 7),
                Constraint::Ratio(3, 7),
            ]
            .as_ref(),
        )
        .split(frame.size())
    else {
        return;
    };

    let [top_left, top_right] = *Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(top)
    else {
        return;
    };

    let [bottom_left, bottom_right] = *Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(bottom)
    else {
        return;
    };

    // TOP
    ApplicationWidget::draw(app, frame, top_left);
    HelpWidget::draw(app, frame, top_right);

    // MIDDLE
    LogsWidget::draw(app, frame, mid);

    // BOTTOM
    BuildsWidget::draw(app, frame, bottom_left);
    DeploymentWidget::draw(app, frame, bottom_right);

    let current_route = app.get_current_route();

    // Draw the dialog if the current route is a dialog.
    match current_route.active_block {
        ActiveBlock::Dialog(DialogContext::NamespaceSelection) => {
            NamespaceSelectionWidget::draw(app, frame);
        }
        ActiveBlock::Dialog(DialogContext::VersionSelection) => {
            VersionSelectionWidget::draw(app, frame);
        }
        _ => {}
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1] // Return the middle chunk
}

pub fn create_loading_widget(parent_block: Block) -> Paragraph {
    let loading_widget = Paragraph::new(Text::styled(
        "Loading...",
        Style::default().fg(Color::White),
    ))
    .block(parent_block);

    loading_widget
}
