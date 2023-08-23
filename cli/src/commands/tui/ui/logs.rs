use ratatui::{
    prelude::{Alignment, Backend, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
    Frame,
};

use crate::commands::tui::app::App;

pub struct LogsWidget;

impl LogsWidget {
    pub fn draw<B: Backend>(app: &mut App, frame: &mut Frame<B>, rect: Rect) {
        let logs_block = Block::default()
            .title(" Logs ")
            .borders(Borders::ALL)
            .padding(Padding::new(1, 1, 0, 0))
            .style(Style::default().fg(Color::LightGreen));
        frame.render_widget(logs_block, rect);

        let [info, logs_area] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Percentage(99)].as_ref())
            .split(rect.inner(&Margin { vertical: 1, horizontal: 1 }))
        else {
            return;
        };

        let title = Block::default()
            .title("Use arrow keys or h j k l to scroll ◄ ▲ ▼ ►")
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(title, info);

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

        let mut log_entries = vec![];
        let mut first_color = true;
        for log in &app.state.log_entries {
            if first_color {
                log_entries.push(Line::styled(
                    format!("{}", log),
                    Style::default().fg(Color::White),
                ));
            } else {
                log_entries.push(Line::styled(
                    format!("{}", log),
                    Style::default().fg(Color::LightCyan),
                ));
            }

            first_color = !first_color;
        }

        app.state.logs_vertical_scroll_state = app
            .state
            .logs_vertical_scroll_state
            .content_length(log_entries.len() as u16);

        let paragraph = Paragraph::new(log_entries)
            .block(Block::default().padding(Padding::new(1, 1, 0, 0)))
            .wrap(Wrap { trim: true })
            .scroll((app.state.logs_vertical_scroll as u16, 0));

        frame.render_widget(paragraph, logs_area);
    }
}
