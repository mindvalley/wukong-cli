use ratatui::{
    prelude::{Alignment, Backend, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};
use regex::Regex;
use wukong_sdk::services::gcloud::google::logging::r#type::LogSeverity;

use crate::commands::tui::app::{App, State, MAX_LOG_ENTRIES_LENGTH};

pub struct LogsWidget;

impl LogsWidget {
    pub fn draw<B: Backend>(app: &mut App, frame: &mut Frame<B>, rect: Rect) {
        app.state.logs_widget_width = rect.width;
        app.state.logs_widget_height = rect.height;

        let main_block = create_main_block();
        frame.render_widget(main_block, rect);

        let search_bar_constraint = if app.state.show_search_bar { 3 } else { 0 };
        // let search_bar_constraint = 3;

        let [info, search_bar_area, logs_area] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(1),
                    Constraint::Min(search_bar_constraint),
                    Constraint::Percentage(99),
                ]
                .as_ref(),
            )
            .split(rect.inner(&Margin {
                vertical: 1,
                horizontal: 1,
            }))
        else {
            return;
        };

        let title = create_title(&app.state);
        frame.render_widget(title, info);

        let search_bar = Paragraph::new(app.state.search_bar_input.input.clone())
            .style(Style::default().fg(Color::LightGreen))
            .block(Block::default().borders(Borders::ALL).title(" Search "));
        frame.render_widget(search_bar, search_bar_area);

        if app.state.show_search_bar {
            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            frame.set_cursor(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                search_bar_area.x + app.state.search_bar_input.cursor_position as u16 + 1,
                // Move one line down, from the border to the input line
                search_bar_area.y + 1,
            );
        }

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
            let loading_message = create_loading_block();
            frame.render_widget(loading_message, logs_area);
            return;
        } else if app.state.has_log_errors {
            let error_message = create_error_block();
            frame.render_widget(error_message, logs_area);
            return;
        }

        render_log_entries(frame, logs_area, &mut app.state);
        render_scrollbar(frame, logs_area, &mut app.state.logs_vertical_scroll_state);
    }
}

fn create_main_block() -> Block<'static> {
    Block::default()
        .title(" Logs ")
        .borders(Borders::ALL)
        .padding(Padding::new(1, 1, 0, 0))
        .style(Style::default().fg(Color::LightGreen))
}

fn create_title(state: &State) -> Block {
    Block::default()
        .title(format!(
          "Use arrow keys or h j k l to scroll ◄ ▲ ▼ ►. Total {} logs. \t [Severity{}], [Tailing: {}]",
          if state.log_entries_length == MAX_LOG_ENTRIES_LENGTH {
              format!("{}+", state.log_entries_length)
          } else {
              state.log_entries_length.to_string()
          },
          if state.logs_severity == Some(LogSeverity::Error) {
              " >= Error".to_string()
          } else {
              ": Default".to_string()
          },
          if state.logs_tailing {
            "On"
          } else {
            "Off"
          }
        ))
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray))
}

fn create_loading_block() -> Paragraph<'static> {
    Paragraph::new(Text::styled(
        "Loading...",
        Style::default().fg(Color::White),
    ))
    .block(Block::default().padding(Padding::new(1, 1, 0, 0)))
}

fn create_error_block() -> Paragraph<'static> {
    Paragraph::new(Text::styled(
        "Something went wrong while fetching logs.",
        Style::default().fg(Color::White),
    ))
    .block(Block::default().padding(Padding::new(1, 1, 0, 0)))
}

fn render_log_entries<B: Backend>(frame: &mut Frame<'_, B>, logs_area: Rect, state: &mut State) {
    let regex = Regex::new(&format!(r"(?i){}", state.search_bar_input.input.trim())).unwrap();

    let filtered_log_entries = state
        .log_entries
        .iter()
        .filter(|each| {
            if regex.is_match(&each.to_string()) {
                return true;
            }

            false
        })
        .collect::<Vec<_>>();

    let mut log_entries = Vec::new();
    for each in filtered_log_entries {
        let output_string = each.to_string();

        let mut matches: Vec<(usize, usize)> = Vec::new();
        for found in regex.find_iter(&output_string.clone()) {
            let start = found.start();
            let end = found.end();

            // merge the match if it overlaps with any existing match
            // to avoid highlighting issue
            let mut is_matched = false;
            for m in &mut matches {
                if m.0 <= start && m.1 >= end {
                    is_matched = true;
                    break;
                }

                if m.0 < start && start < m.1 && end > m.1 {
                    m.1 = end;
                    is_matched = true;
                    break;
                }
                if m.1 > end && end > m.0 && start < m.0 {
                    m.0 = start;
                    is_matched = true;
                    break;
                }
            }

            if !is_matched {
                matches.push((start, end));
            }
        }

        // sort the matches so the output will be correct
        // since we are adding offset manually
        matches.sort_by(|a, b| a.0.cmp(&b.0));

        let mut line = Vec::new();
        let mut last_pos = 0;
        for (index, m) in matches.iter().enumerate() {
            if index == 0 {
                line.push(Span::styled(
                    output_string[..m.0].to_string(),
                    Style::default().fg(Color::White),
                ));
            }

            if last_pos != 0 {
                line.push(Span::styled(
                    output_string[last_pos..m.0].to_string(),
                    Style::default().fg(Color::White),
                ));
            }

            line.push(Span::styled(
                output_string[m.0..m.1].to_string(),
                Style::default().fg(Color::Cyan),
            ));

            last_pos = m.1;

            if index == matches.len() - 1 {
                line.push(Span::styled(
                    output_string[m.1..].to_string(),
                    Style::default().fg(Color::White),
                ));
            }
        }

        log_entries.push(Line::from(line));
    }

    state.logs_vertical_scroll_state = state
        .logs_vertical_scroll_state
        .content_length(state.log_entries_length as u16);

    let paragraph = Paragraph::new(log_entries)
        .block(Block::default().padding(Padding::new(1, 1, 0, 0)))
        // we can't use wrap if we want to scroll to bottom
        // because we don't know the state of the render
        // waiting this https://github.com/ratatui-org/ratatui/issues/136
        // .wrap(Wrap { trim: true })
        .scroll((
            state.logs_vertical_scroll as u16,
            state.logs_horizontal_scroll as u16,
        ));

    frame.render_widget(paragraph, logs_area);
}

fn render_scrollbar<B: Backend>(
    frame: &mut Frame<'_, B>,
    logs_area: Rect,
    logs_vertical_scroll_state: &mut ScrollbarState,
) {
    frame.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None),
        logs_area.inner(&Margin {
            vertical: 1,
            horizontal: 0,
        }),
        logs_vertical_scroll_state,
    );
}
