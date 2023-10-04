use super::util::get_color;
use crate::commands::tui::app::{ActiveBlock, App, State, MAX_LOG_ENTRIES_LENGTH};
use ratatui::{
    prelude::{Alignment, Backend, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{
        Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};
use wukong_sdk::services::gcloud::google::logging::r#type::LogSeverity;

pub struct LogsWidget;

impl LogsWidget {
    pub fn draw<B: Backend>(app: &mut App, frame: &mut Frame<B>, rect: Rect) {
        app.state.logs_widget_width = rect.width;
        app.state.logs_widget_height = rect.height;

        app.update_draw_lock(ActiveBlock::Log, rect);

        let main_block = create_main_block(app);
        frame.render_widget(main_block, rect);

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

        let title = create_title(&app.state);
        frame.render_widget(title, info);

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

fn create_main_block(app: &mut App) -> Block<'static> {
    let current_route = app.get_current_route();

    let highlight_state = (
        current_route.active_block == ActiveBlock::Log,
        current_route.hovered_block == ActiveBlock::Log,
    );

    Block::default()
        .title(" Logs ")
        .borders(Borders::ALL)
        .padding(Padding::new(1, 1, 0, 0))
        .border_style(get_color(
            highlight_state,
            (Color::LightCyan, Color::LightGreen, Color::White),
        ))
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
    let mut first_color = false;

    let log_entries = state
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

    state.logs_vertical_scroll_state = state
        .logs_vertical_scroll_state
        .content_length(state.log_entries_length);

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
