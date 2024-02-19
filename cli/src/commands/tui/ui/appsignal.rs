use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block as WidgetBlock, Borders, Cell, Padding, Paragraph, Row, Table},
    Frame,
};

use crate::commands::tui::app::App;

pub struct AppsignalWidget;

impl AppsignalWidget {
    pub fn draw<B: Backend>(app: &mut App, frame: &mut Frame<B>, rect: Rect) {
        if let Some(ref error) = app.state.appsignal_error {
            let error_widget =
                Paragraph::new(Text::styled(error, Style::default().fg(Color::White)));
            frame.render_widget(error_widget, rect);
            return;
        }

        if app.state.is_fetching_appsignal_data {
            let loading_widget = Paragraph::new(Text::styled(
                "Loading...",
                Style::default().fg(Color::White),
            ));
            frame.render_widget(loading_widget, rect);
            return;
        }

        match app.state.is_appsignal_enabled {
            Some(true) => {
                let [top, bottom] = *Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(1), Constraint::Length(8)].as_ref())
                    .split(rect)
                else {
                    return;
                };

                let namespace = Paragraph::new(format!(
                    "namespace: {}",
                    app.state.appsignal_namespace.as_ref().unwrap()
                ));
                frame.render_widget(namespace, top);

                let rows = vec![
                    Row::new(vec![
                        Cell::from(Span::styled("", Style::default().fg(Color::White))),
                        Cell::from(Span::styled(
                            "1h AVG",
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Cell::from(Span::styled(
                            "8h AVG",
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Cell::from(Span::styled(
                            "24h AVG",
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )),
                    ]),
                    Row::new(vec![
                        Cell::from(Span::styled(
                            "Error rate",
                            Style::default().fg(Color::White),
                        )),
                        Cell::from(Span::styled(
                            format!(
                                "{:.4}%",
                                app.state.appsignal.average_error_rates.in_1_hour.clone()
                            ),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Cell::from(Span::styled(
                            format!(
                                "{:.4}%",
                                app.state.appsignal.average_error_rates.in_8_hours.clone()
                            ),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Cell::from(Span::styled(
                            format!(
                                "{:.4}%",
                                app.state.appsignal.average_error_rates.in_24_hours.clone()
                            ),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )),
                    ]),
                    Row::new(vec![
                        Cell::from(Span::styled(
                            "Throughput",
                            Style::default().fg(Color::White),
                        )),
                        Cell::from(Span::styled(
                            format!(
                                "{:.2}k/min",
                                app.state.appsignal.average_throughputs.in_1_hour.clone() / 1000.0
                            ),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Cell::from(Span::styled(
                            format!(
                                "{:.2}k/min",
                                app.state.appsignal.average_throughputs.in_8_hours.clone() / 1000.0
                            ),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Cell::from(Span::styled(
                            format!(
                                "{:.2}k/min",
                                app.state.appsignal.average_throughputs.in_24_hours.clone() // / 1000.0
                            ),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )),
                    ]),
                    Row::new(vec![
                        Cell::from(Span::styled("", Style::default().fg(Color::White))),
                        Cell::from(Span::styled(
                            "",
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Cell::from(Span::styled(
                            "",
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Cell::from(Span::styled(
                            "",
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )),
                    ]),
                    Row::new(vec![
                        Cell::from(Span::styled("", Style::default().fg(Color::White))),
                        Cell::from(Span::styled(
                            "Mean",
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Cell::from(Span::styled(
                            "P90",
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Cell::from(Span::styled(
                            "P95",
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )),
                    ]),
                    Row::new(vec![
                        Cell::from(Span::styled(
                            "Latency (ms)",
                            Style::default().fg(Color::White),
                        )),
                        Cell::from(Span::styled(
                            format!("{:.2}", app.state.appsignal.average_latencies.mean.clone()),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Cell::from(Span::styled(
                            format!("{:.2}", app.state.appsignal.average_latencies.p90.clone()),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Cell::from(Span::styled(
                            format!("{:.2}", app.state.appsignal.average_latencies.p95.clone()),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )),
                    ]),
                ];

                let table = Table::new(rows)
                    .block(
                        WidgetBlock::default()
                            .borders(Borders::ALL)
                            .padding(Padding::new(1, 1, 0, 0)),
                    )
                    .widths(&[
                        Constraint::Length(15),
                        Constraint::Length(15),
                        Constraint::Length(15),
                        Constraint::Length(15),
                    ])
                    .column_spacing(1);
                frame.render_widget(table, bottom);
            }
            Some(false) => {
                let widget = Paragraph::new(Text::styled(
                    "Appsignal is not enabled.",
                    Style::default().fg(Color::White),
                ));
                frame.render_widget(widget, rect);
            }
            None => {
                let widget = Paragraph::new(Text::styled(
                    "Appsignal configs are not loaded",
                    Style::default().fg(Color::White),
                ));
                frame.render_widget(widget, rect);
            }
        }
    }
}
