use clap::crate_version;
use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout, Rect},
    style::{self, Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, ListItem, Padding, Paragraph, Wrap},
    Frame,
};

use super::{app::App, hotkey::HotKey, CurrentScreen};

pub fn draw<B>(frame: &mut Frame<B>, app: &App)
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

    let application_block = Block::default()
        .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
        .padding(Padding::new(1, 0, 0, 0))
        .style(Style::default());

    let application_area = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Application: "),
            Span::styled(
                &app.state.current_application,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(style::Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("Namespace: "),
            Span::styled(
                &app.state.current_namespace,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(style::Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("CLI Version: "),
            Span::styled(
                crate_version!(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(style::Modifier::BOLD),
            ),
        ]),
    ])
    .wrap(Wrap { trim: true })
    .block(application_block);

    let hotkeys = vec![HotKey::SelectNamespace, HotKey::Quit]
        .into_iter()
        .map(|hotkey| {
            Line::from(vec![
                Span::styled(
                    format!("<{}>", hotkey.keycode()),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(style::Modifier::BOLD),
                ),
                Span::raw(format!(" {}", hotkey.desc())),
            ])
        })
        .collect::<Vec<_>>();

    let view_controls_block = Block::default()
        .borders(Borders::RIGHT | Borders::TOP | Borders::BOTTOM)
        .style(Style::default());

    let view_controls_area = Paragraph::new(hotkeys)
        .block(view_controls_block)
        .wrap(Wrap { trim: true });

    frame.render_widget(application_area, top_left);
    frame.render_widget(view_controls_area, top_right);

    let logs_block = Block::default()
        .title(" Logs ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::LightGreen));
    let logs_area =
        Paragraph::new(Text::styled("", Style::default().fg(Color::LightGreen))).block(logs_block);

    frame.render_widget(logs_area, mid);

    let builds_block = Block::default()
        .title(" Build Artifacts ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::LightYellow));

    let builds_area = Paragraph::new(Text::styled("", Style::default().fg(Color::LightYellow)))
        .block(builds_block);

    let deployment_block = Block::default()
        .title(" Deployment ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::LightBlue));

    let deployment_area = Paragraph::new(Text::styled("", Style::default().fg(Color::LightBlue)))
        .block(deployment_block);

    frame.render_widget(builds_area, bottom_left);
    frame.render_widget(deployment_area, bottom_right);

    // if state.show_namespace_selection {
    //     let items: Vec<ListItem> = self
    //         .namespace_selections
    //         .items
    //         .iter()
    //         .map(|i| {
    //             let lines = vec![Line::from(i.clone())];
    //             ListItem::new(lines).style(Style::default().fg(Color::White))
    //         })
    //         .collect();
    //
    //     let popup_block = Block::default()
    //         .title(" Namespace Selection ")
    //         .borders(Borders::ALL)
    //         .title_alignment(Alignment::Center)
    //         .style(Style::default().bg(Color::Black));
    //
    //     // Create a List from all list items and highlight the currently selected one
    //     let items = List::new(items)
    //         .block(popup_block)
    //         .highlight_style(
    //             Style::default()
    //                 .fg(Color::Black)
    //                 .bg(Color::LightGreen)
    //                 .add_modifier(Modifier::BOLD),
    //         )
    //         .highlight_symbol(">> ");
    //
    //     let area = centered_rect(60, 25, frame.size());
    //     frame.render_widget(Clear, area);
    //     // We can now render the item list
    //     frame.render_stateful_widget(items, area, &mut self.namespace_selections.state);
    // }

    if let CurrentScreen::Exiting = app.current_screen {
        frame.render_widget(Clear, frame.size()); //this clears the entire screen and anything already drawn
        let popup_block = Block::default()
            .title("Y/N")
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let exit_text = Text::styled(
            "Would you like to exit? (y/n)",
            Style::default().fg(Color::Red),
        );
        // the `trim: false` will stop the text from being cut off when over the edge of the block
        let exit_paragraph = Paragraph::new(exit_text)
            .block(popup_block)
            .wrap(Wrap { trim: false });

        let area = centered_rect(60, 25, frame.size());
        frame.render_widget(exit_paragraph, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
