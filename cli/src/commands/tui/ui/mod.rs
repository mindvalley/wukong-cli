use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout, Rect},
    Frame,
};

use self::{
    application::ApplicationWidget, builds::BuildsWidget, deployment::DeploymentWidget,
    help::HelpWidget, logs::LogsWidget, namespace_selection::NamespaceSelectionWidget,
    version_selection::VersionSelectionWidget,
};

use super::{app::App, CurrentScreen};

mod application;
mod builds;
mod deployment;
mod help;
mod logs;
pub mod namespace_selection;
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

    if let CurrentScreen::NamespaceSelection = app.current_screen {
        NamespaceSelectionWidget::draw(app, frame);
    } else if let CurrentScreen::VersionSelection = app.current_screen {
        VersionSelectionWidget::draw(app, frame);
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
