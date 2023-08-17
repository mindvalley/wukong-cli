use ratatui::{
    prelude::{Backend, Constraint, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::commands::tui::app::App;

pub struct BuildsWidget;

impl BuildsWidget {
    pub fn draw<B: Backend>(app: &App, frame: &mut Frame<B>, rect: Rect) {
        let name_style = Style::default().fg(Color::White);
        let builds_block = Block::default()
            .title(" Build Artifacts ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightYellow));

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
