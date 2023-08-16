use ratatui::{
    prelude::{Alignment, Backend, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use self::{
    application::ApplicationWidget, builds::BuildsWidget, deployment::DeploymentWidget,
    help::HelpWidget, logs::LogsWidget, namespace_selection::NamespaceSelectionWidget,
};

use super::{app::App, CurrentScreen};

mod application;
mod builds;
mod deployment;
mod help;
mod logs;
pub mod namespace_selection;

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
    let application_widget = ApplicationWidget::new(app);
    frame.render_widget(application_widget.widget, top_left);

    let help_widget = HelpWidget::new(app);
    frame.render_widget(help_widget.widget, top_right);

    // MIDDLE
    let logs_widget = LogsWidget::new(app);
    frame.render_widget(logs_widget.widget, mid);

    // BOTTOM
    let builds_widget = BuildsWidget::new(app);
    frame.render_widget(builds_widget.widget, bottom_left);

    let deployment_widget = DeploymentWidget::new(app);
    frame.render_widget(deployment_widget.widget, bottom_right);

    if let CurrentScreen::NamespaceSelection = app.current_screen {
        let items: Vec<ListItem> = app
            .namespace_selections
            .items
            .iter()
            .map(|i| {
                let lines = vec![Line::from(i.clone())];
                ListItem::new(lines).style(Style::default().fg(Color::White))
            })
            .collect();

        let popup_block = Block::default()
            .title(" Namespace Selection ")
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center)
            .style(Style::default().bg(Color::Black));

        // Create a List from all list items and highlight the currently selected one
        let items = List::new(items)
            .block(popup_block)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        let area = centered_rect(60, 25, frame.size());

        frame.render_widget(Clear, area);
        frame.render_stateful_widget(items, area, &mut app.namespace_selections.state);
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
