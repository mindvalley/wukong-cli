use chrono::{DateTime, Local, NaiveDateTime, Utc};
use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block, Borders, Cell, Padding, Paragraph, Row, Table},
    Frame,
};
use time_humanize::HumanTime;

use crate::commands::tui::app::{App, Deployment};

pub struct DeploymentWidget;

impl DeploymentWidget {
    pub fn draw<B: Backend>(app: &App, frame: &mut Frame<B>, rect: Rect) {
        let deployments_block = Block::default()
            .title(" Deployments ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightBlue));

        if app.state.is_fetching_deployments {
            let loading_widget = Paragraph::new(Text::styled(
                "Loading...",
                Style::default().fg(Color::White),
            ))
            .block(deployments_block.padding(Padding::new(1, 1, 0, 0)));
            frame.render_widget(loading_widget, rect);
            return;
        }

        let widget = Paragraph::new(Text::raw("")).block(deployments_block);
        frame.render_widget(widget, rect);

        let green_versions: Vec<&Deployment> = app
            .state
            .deployments
            .iter()
            .filter(|pipeline| {
                pipeline.environment == app.state.current_namespace.to_lowercase()
                    && pipeline.version == "green"
            })
            .collect();

        let blue_versions: Vec<&Deployment> = app
            .state
            .deployments
            .iter()
            .filter(|pipeline| {
                pipeline.environment == app.state.current_namespace.to_lowercase()
                    && pipeline.version == "blue"
            })
            .collect();

        let has_green_version = !green_versions.is_empty();
        let has_blue_version = !blue_versions.is_empty();

        let mut green_rows = vec![];
        if has_green_version {
            let green = green_versions[0];
            green_rows = setup_rows(green);
        }
        let mut blue_rows = vec![];
        if has_blue_version {
            let blue = blue_versions[0];
            blue_rows = setup_rows(blue);
        }

        let green_block = Block::default()
            .title(" Green ")
            .borders(Borders::ALL)
            .padding(Padding::new(1, 1, 0, 0));
        let green_widget = Table::new(green_rows)
            .block(green_block)
            .widths(&[Constraint::Min(20), Constraint::Min(40)])
            .column_spacing(1);
        let blue_block = Block::default()
            .title(" Blue ")
            .borders(Borders::ALL)
            .padding(Padding::new(1, 1, 0, 0));
        let blue_widget = Table::new(blue_rows)
            .block(blue_block)
            .widths(&[Constraint::Min(20), Constraint::Min(40)])
            .column_spacing(1);

        // if both versions are exist, split the screen
        if has_green_version && has_blue_version {
            let [top, bottom] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(rect.inner(&Margin { vertical: 1, horizontal: 1 }))
        else {
            return;
        };
            frame.render_widget(green_widget, top);
            frame.render_widget(blue_widget, bottom);
        } else if has_green_version {
            frame.render_widget(
                green_widget,
                rect.inner(&Margin {
                    vertical: 1,
                    horizontal: 1,
                }),
            );
        } else if has_blue_version {
            frame.render_widget(
                blue_widget,
                rect.inner(&Margin {
                    vertical: 1,
                    horizontal: 1,
                }),
            );
        }
    }
}

fn humanize_timestamp(timestamp: &Option<i64>) -> String {
    if let Some(timestamp) = timestamp {
        let naive = NaiveDateTime::from_timestamp_opt(
            timestamp / 1000,
            (timestamp % 1000) as u32 * 1_000_000,
        )
        .unwrap();

        let dt = DateTime::<Utc>::from_utc(naive, Utc).with_timezone(&Local);
        // convert to std::time::SystemTime as the HumanTime expecting this
        format!(
            "{}",
            HumanTime::from(Into::<std::time::SystemTime>::into(dt))
        )
    } else {
        "N/A".to_string()
    }
}

fn setup_rows(deployment: &Deployment) -> Vec<Row> {
    vec![
        Row::new(vec![
            Cell::from(Span::styled("Enabled", Style::default().fg(Color::White))),
            Cell::from(Span::styled(
                match deployment.enabled {
                    true => "✅",
                    false => "❌",
                },
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                "Deployed Ref",
                Style::default().fg(Color::White),
            )),
            Cell::from(Span::styled(
                deployment.deployed_ref.clone().unwrap_or("N/A".to_string()),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                "Build Artifact",
                Style::default().fg(Color::White),
            )),
            Cell::from(Span::styled(
                deployment
                    .build_artifact
                    .clone()
                    .unwrap_or("N/A".to_string()),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                "Triggered By",
                Style::default().fg(Color::White),
            )),
            Cell::from(Span::styled(
                deployment.deployed_by.clone().unwrap_or("N/A".to_string()),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                "Last Deployment",
                Style::default().fg(Color::White),
            )),
            Cell::from(Span::styled(
                humanize_timestamp(&deployment.last_deployed_at),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
        ]),
        Row::new(vec![
            Cell::from(Span::styled("Status", Style::default().fg(Color::White))),
            Cell::from(Span::styled(
                deployment.status.clone().unwrap_or("N/A".to_string()),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
        ]),
    ]
}
