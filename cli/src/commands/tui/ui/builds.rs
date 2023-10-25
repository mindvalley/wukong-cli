use super::util::get_color;
use crate::commands::tui::app::{App, Block};
use ratatui::{
    prelude::{Backend, Constraint, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block as WidgetBlock, Borders, Cell, Padding, Paragraph, Row, Table},
    Frame,
};

pub struct BuildsWidget;

impl BuildsWidget {
    pub fn draw<B: Backend>(app: &mut App, frame: &mut Frame<B>, rect: Rect) {
        let name_style = Style::default().fg(Color::White);
        let current_route = app.get_current_route();

        let highlight_state = (
            current_route.active_block == Block::Build,
            current_route.hovered_block == Block::Build,
        );

        app.update_draw_lock(Block::Build, rect);

        let builds_block = WidgetBlock::default()
            .title(" Build Artifacts ")
            .borders(Borders::ALL)
            .padding(Padding::new(1, 1, 0, 0))
            .border_style(get_color(
                highlight_state,
                (Color::LightCyan, Color::LightYellow, Color::White),
            ));

        if let Some(ref error) = app.state.builds_error {
            let error_widget =
                Paragraph::new(Text::styled(error, Style::default().fg(Color::White)))
                    .block(builds_block);
            frame.render_widget(error_widget, rect);
            return;
        }

        if app.state.is_fetching_builds {
            let loading_widget = Paragraph::new(Text::styled(
                "Loading...",
                Style::default().fg(Color::White),
            ))
            .block(builds_block);
            frame.render_widget(loading_widget, rect);
            return;
        }

        let rows = app
            .state
            .builds
            .iter()
            .map(|build| {
                let commits = build
                    .commits
                    .iter()
                    .map(|commit| {
                        Line::from(vec![
                            Span::styled(
                                format!("{} ", &commit.id[0..7]),
                                Style::default().fg(Color::DarkGray),
                            ),
                            Span::raw(commit.message_headline.clone()),
                        ])
                    })
                    .collect::<Vec<_>>();
                Row::new(vec![
                    Cell::from(Span::styled(build.name.clone(), name_style)),
                    Cell::from(commits),
                ])
                .height(build.commits.len() as u16)
            })
            .collect::<Vec<_>>();

        let widget = Table::new(rows)
            .block(builds_block)
            .header(Row::new(vec![
                Cell::from(Span::styled("Name", name_style)),
                Cell::from(Span::styled("Commit(s)", name_style)),
            ]))
            .widths(&[Constraint::Min(20), Constraint::Length(1000)])
            .column_spacing(1);

        frame.render_widget(widget, rect);
    }
}
