use ratatui::{
    prelude::{Alignment, Backend, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation},
    Frame,
};
use wukong_sdk::services::gcloud::google::logging::r#type::LogSeverity;

use crate::commands::tui::app::{App, MAX_LOG_ENTRIES_LENGTH};

pub struct LogsWidget;

impl LogsWidget {
    pub fn draw<B: Backend>(app: &mut App, frame: &mut Frame<B>, rect: Rect) {
        app.state.logs_widget_width = rect.width;
        app.state.logs_widget_height = rect.height;

        let logs_block = Block::default()
            .title(" Logs ")
            .borders(Borders::ALL)
            .padding(Padding::new(1, 1, 0, 0))
            .style(Style::default().fg(Color::LightGreen));
        frame.render_widget(logs_block, rect);

        let [info, logs_area] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Percentage(99)].as_ref())
            .split(rect.inner(&Margin {
                vertical: 1,
                horizontal: 1,
            }))
        else {
            return;
        };

        let title = Block::default()
            .title(format!(
                "Use arrow keys or h j k l to scroll ◄ ▲ ▼ ►. Total {} logs. \t [Severity: {}]",
                if app.state.log_entries_length == MAX_LOG_ENTRIES_LENGTH {
                    format!("{}+", app.state.log_entries_length)
                } else {
                    app.state.log_entries_length.to_string()
                },
                if app.state.logs_serverity == Some(LogSeverity::Error) {
                    "Error".to_string()
                } else {
                    "Default".to_string()
                }
            ))
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(title, info);

        if app.state.has_log_errors {
            let loading_widget = Paragraph::new(Text::styled(
                "Something went wrong while fetching logs.",
                Style::default().fg(Color::White),
            ))
            .block(Block::default().padding(Padding::new(1, 1, 0, 0)));
            frame.render_widget(loading_widget, logs_area);
            return;
        }

        // it will show loader only on the first call
        if app.state.is_fetching_log_entries {
            let loading_widget = Paragraph::new(Text::styled(
                "Loading...",
                Style::default().fg(Color::White),
            ))
            .block(Block::default().padding(Padding::new(1, 1, 0, 0)));
            frame.render_widget(loading_widget, logs_area);
            return;
        }

        let mut first_color = false;

        let log_entries = app
            .state
            .log_entries
            .iter()
            .map(|log_entry| {
                first_color = !first_color;

                if first_color {
                    Line::styled(format!("{}", log_entry), Style::default().fg(Color::White))
                } else {
                    Line::styled(
                        format!("{}", log_entry),
                        Style::default().fg(Color::LightCyan),
                    )
                }
            })
            .collect::<Vec<Line>>();

        app.state.logs_vertical_scroll_state = app
            .state
            .logs_vertical_scroll_state
            .content_length(app.state.log_entries_length);

        let paragraph = Paragraph::new(log_entries)
            .block(Block::default().padding(Padding::new(1, 1, 0, 0)))
            // we can't use wrap if we want to scroll to bottom
            // because we don't know the state of the render
            // waiting this https://github.com/ratatui-org/ratatui/issues/136
            // .wrap(Wrap { trim: true })
            .scroll((
                app.state.logs_vertical_scroll as u16,
                app.state.logs_horizontal_scroll as u16,
            ));

        frame.render_widget(paragraph, logs_area);
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            logs_area.inner(&Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut app.state.logs_vertical_scroll_state,
        );
    }
}
