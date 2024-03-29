use std::collections::HashMap;

use super::util::get_color;
use crate::commands::tui::app::{App, Block, DialogContext, State, MAX_LOG_ENTRIES_LENGTH};
use ratatui::{
    prelude::{Alignment, Backend, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block as WidgetBlock, Borders, Padding, Paragraph},
    Frame,
};
use regex::Regex;
use wukong_sdk::services::gcloud::google::logging::{r#type::LogSeverity, v2::LogEntry};

pub struct LogsWidget;

impl LogsWidget {
    pub fn draw<B: Backend>(app: &mut App, frame: &mut Frame<B>, rect: Rect) {
        let main_block = create_main_block(app);
        frame.render_widget(main_block, rect);

        let search_bar_constraint = if app.state.show_search_bar || app.state.show_filter_bar {
            3
        } else {
            0
        };

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
            .split(rect)
        else {
            return;
        };

        let title = create_title(&app.state);
        frame.render_widget(title, info);

        if app.state.show_search_bar {
            render_search_bar(frame, search_bar_area, app);
        }

        if app.state.show_filter_bar {
            render_filter_bar(frame, search_bar_area, app);
        }

        if let Some(ref error) = app.state.log_entries_error {
            let error_block = create_error_block(error);
            frame.render_widget(error_block, logs_area);
            return;
        }

        // it will show loader only on the first call
        if app.state.is_fetching_log_entries {
            let loading_message = create_loading_block();
            frame.render_widget(loading_message, logs_area);
            return;
        }

        render_log_entries(frame, logs_area, &mut app.state);
    }
}

fn create_main_block(_app: &mut App) -> WidgetBlock<'static> {
    WidgetBlock::default()
}

fn create_title(state: &State) -> WidgetBlock {
    WidgetBlock::default()
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
    .block(WidgetBlock::default())
}

fn create_error_block(error: &str) -> Paragraph<'_> {
    Paragraph::new(Text::styled(error, Style::default().fg(Color::White)))
        .block(WidgetBlock::default().padding(Padding::new(1, 1, 0, 0)))
}

fn render_log_entries<B: Backend>(frame: &mut Frame<'_, B>, logs_area: Rect, state: &mut State) {
    let block = WidgetBlock::default().padding(Padding::new(0, 0, 0, 0));

    let (inner_width, inner_height) = {
        let inner_rect = block.inner(logs_area);
        (inner_rect.width, inner_rect.height)
    };

    // store the current size so we can use it for scrolling calculation
    state.logs_size = (inner_width, inner_height);

    let num_rows: usize = inner_height as usize;
    let start = std::cmp::min(
        state.log_entries.1.len(),
        state.logs_table_current_start_index,
    );

    let end = std::cmp::min(state.log_entries.1.len(), start + num_rows);

    let log_entries = if state.show_search_bar {
        if state.search_bar_input.input.is_empty() {
            generate_log_entries_line_without_text_wrap(state, start, end)
        } else {
            let regex =
                Regex::new(&format!(r"(?i){}", state.search_bar_input.input.trim())).unwrap();

            filter_log_entries(regex, state.log_entries.1.iter().collect())
        }
    } else if state.show_filter_bar {
        let include = state.filter_bar_include_input.input.clone();
        let exclude = state.filter_bar_exclude_input.input.clone();

        let mut log_entries: Vec<&LogEntry> = state.log_entries.1.iter().collect();
        if !exclude.is_empty() {
            let regex = Regex::new(&format!(r"(?i){}", exclude.trim())).unwrap();

            log_entries = log_entries
                .into_iter()
                .filter(|each| {
                    if regex.is_match(&each.to_string()) {
                        return false;
                    }

                    true
                })
                .collect::<Vec<_>>();
        }

        if include.is_empty() {
            generate_log_entries_line_without_text_wrap(state, start, end)
        } else {
            let regex = Regex::new(&format!(
                r"(?i){}",
                state.filter_bar_include_input.input.trim()
            ))
            .unwrap();

            filter_log_entries(regex, log_entries)
        }
    } else if state.logs_textwrap {
        let mut wrapped_vec = Vec::new();
        let mut wrapped_hm = HashMap::new();
        let mut count = 0;
        let mut first_color = true;
        let mut is_fully_filled = false;

        for (i, log) in state.log_entries.1[start..end].iter().enumerate() {
            let color = if first_color {
                Color::Cyan
            } else {
                Color::White
            };

            let str = log.to_string();
            let mut wrapped = textwrap::wrap(&str, inner_width as usize)
                .iter()
                .map(|each| Line::styled(each.to_string(), Style::default().fg(color)))
                .collect::<Vec<_>>();

            // remove newline
            wrapped.pop();

            wrapped_hm.insert(start + i, (wrapped.clone(), wrapped.len()));
            wrapped_vec.extend(wrapped);

            first_color = !first_color;

            count += 1;

            if wrapped_vec.len() >= inner_height as usize {
                // the last rendered index is the first rendered index + the number of log
                // being rendered - 1
                state.logs_table_current_last_index =
                    state.logs_table_current_start_index + count - 1;

                state.logs_table_current_last_fully_rendered =
                    wrapped_vec.len() == inner_height as usize;

                is_fully_filled = true;
                break;
            }
        }

        if !is_fully_filled {
            state.logs_table_current_last_index = state.logs_table_current_start_index;
        }

        wrapped_vec
            .into_iter()
            .take(inner_height as usize)
            .collect()
    } else {
        generate_log_entries_line_without_text_wrap(state, start, end)
    };

    let paragraph = Paragraph::new(log_entries)
        .block(WidgetBlock::default().padding(Padding::new(0, 0, 0, 0)))
        .scroll((0, state.logs_horizontal_scroll as u16));

    frame.render_widget(paragraph, logs_area);
}

fn render_search_bar<B: Backend>(frame: &mut Frame<'_, B>, input_area: Rect, app: &mut App) {
    let current_route = app.get_current_route();

    let highlight_state = (
        current_route.active_block == Block::Dialog(DialogContext::LogSearch),
        current_route.hovered_block == Block::Dialog(DialogContext::LogSearch),
    );

    let search_bar = Paragraph::new(app.state.search_bar_input.input.clone())
        .style(Style::default().fg(Color::LightGreen))
        .block(
            WidgetBlock::default()
                .borders(Borders::ALL)
                .border_style(get_color(
                    highlight_state,
                    (Color::LightCyan, Color::White, Color::White),
                ))
                .title(" Search "),
        );

    frame.render_widget(search_bar, input_area);
    // Make the cursor visible and ask ratatui to put it at the specified coordinates after
    // rendering
    frame.set_cursor(
        // Draw the cursor at the current position in the input field.
        // This position is can be controlled via the left and right arrow key
        input_area.x + app.state.search_bar_input.cursor_position as u16 + 1,
        // Move one line down, from the border to the input line
        input_area.y + 1,
    );
}

fn render_filter_bar<B: Backend>(frame: &mut Frame<'_, B>, input_area: Rect, app: &mut App) {
    let [include_bar_area, exclude_bar_area] = *Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(input_area.inner(&Margin {
            vertical: 0,
            horizontal: 0,
        }))
    else {
        return;
    };
    let current_route = app.get_current_route();

    let include_highlight_state = (
        current_route.active_block == Block::Dialog(DialogContext::LogIncludeFilter),
        current_route.hovered_block == Block::Dialog(DialogContext::LogIncludeFilter),
    );
    let exclude_highlight_state = (
        current_route.active_block == Block::Dialog(DialogContext::LogExcludeFilter),
        current_route.hovered_block == Block::Dialog(DialogContext::LogExcludeFilter),
    );

    let filter_include_bar = Paragraph::new(app.state.filter_bar_include_input.input.clone())
        .style(Style::default().fg(Color::LightGreen))
        .block(
            WidgetBlock::default()
                .title(" Include ")
                .borders(Borders::ALL)
                .border_style(get_color(
                    include_highlight_state,
                    (Color::LightCyan, Color::White, Color::White),
                )),
        );
    frame.render_widget(filter_include_bar, include_bar_area);

    let filter_exclude_bar = Paragraph::new(app.state.filter_bar_exclude_input.input.clone())
        .style(Style::default().fg(Color::LightGreen))
        .block(
            WidgetBlock::default()
                .borders(Borders::ALL)
                .border_style(get_color(
                    exclude_highlight_state,
                    (Color::LightCyan, Color::White, Color::White),
                ))
                .title(" Exclude "),
        );

    frame.render_widget(filter_exclude_bar, exclude_bar_area);

    match current_route.active_block {
        Block::Dialog(DialogContext::LogIncludeFilter) => {
            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            frame.set_cursor(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                include_bar_area.x + app.state.filter_bar_include_input.cursor_position as u16 + 1,
                // Move one line down, from the border to the input line
                include_bar_area.y + 1,
            );
        }
        Block::Dialog(DialogContext::LogExcludeFilter) => {
            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            frame.set_cursor(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                exclude_bar_area.x + app.state.filter_bar_exclude_input.cursor_position as u16 + 1,
                // Move one line down, from the border to the input line
                exclude_bar_area.y + 1,
            );
        }
        _ => {}
    }
}

fn generate_log_entries_line_without_text_wrap(
    state: &State,
    start: usize,
    end: usize,
) -> Vec<Line> {
    // ) -> Vec<Row> {
    let mut first_color = true;

    state.log_entries.1[start..end]
        .iter()
        .map(|log_entry| {
            let color = if first_color {
                Color::Cyan
            } else {
                Color::White
            };
            first_color = !first_color;
            Line::styled(format!("{}", log_entry), Style::default().fg(color))
        })
        .collect()
}

fn filter_log_entries(regex: Regex, log_entries: Vec<&LogEntry>) -> Vec<Line> {
    let filtered_log_entries = log_entries
        .iter()
        .filter(|each| regex.is_match(&each.to_string()))
        .collect::<Vec<_>>();

    let mut log_entries = Vec::new();
    let mut first_color = true;

    for each in filtered_log_entries.iter() {
        let style = if first_color {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        let output_string = each.to_string();

        let mut matches: Vec<(usize, usize)> = Vec::new();
        for found in regex.find_iter(&output_string.clone()) {
            let start = found.start();
            let end = found.end();
            matches.push((start, end));
        }

        let mut line = Vec::new();
        let mut last_pos = 0;
        for (index, m) in matches.iter().enumerate() {
            if index == 0 {
                line.push(Span::styled(output_string[0..m.0].to_string(), style));
            }

            if last_pos != 0 {
                line.push(Span::styled(
                    output_string[last_pos..m.0].to_string(),
                    style,
                ));
            }

            line.push(Span::styled(
                output_string[m.0..m.1].to_string(),
                Style::default().bg(Color::Yellow).fg(Color::Black),
            ));

            if index == matches.len() - 1 {
                line.push(Span::styled(output_string[m.1..].to_string(), style));
            }

            last_pos = m.1;
        }

        log_entries.push(Line::from(line));

        first_color = !first_color;
    }

    log_entries
}
