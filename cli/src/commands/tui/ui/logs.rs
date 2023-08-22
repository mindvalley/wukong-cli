use ratatui::{
    prelude::{Alignment, Backend, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Padding, Paragraph},
    Frame,
};

use crate::commands::tui::app::App;

pub struct LogsWidget;

impl LogsWidget {
    pub fn draw<B: Backend>(app: &App, frame: &mut Frame<B>, rect: Rect) {
        let [info, logs_area] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Percentage(99)].as_ref())
            .split(rect.inner(&Margin { vertical: 1, horizontal: 1 }))
        else {
            return;
        };

        let logs_block = Block::default()
            .title(" Logs ")
            .borders(Borders::ALL)
            .padding(Padding::new(1, 1, 0, 0))
            .style(Style::default().fg(Color::LightGreen));
        let widget = Paragraph::new(Text::raw("")).block(logs_block);
        frame.render_widget(widget, rect);

        let title = Block::default()
            .title("Use arrow keys or h j k l to scroll ◄ ▲ ▼ ►")
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(title, info);

        if app.state.is_fetching_logs {
            let loading_widget = Paragraph::new(Text::styled(
                "Loading...",
                Style::default().fg(Color::White),
            ))
            .block(Block::default().padding(Padding::new(1, 1, 0, 0)));
            frame.render_widget(loading_widget, logs_area);
        } else {
            let log_entries: Vec<Line> = app
                .state
                .log_entries
                .iter()
                .map(|log| Line::from(format!("{}", log.clone())))
                .collect();

            let widget = Paragraph::new(log_entries)
                .block(Block::default().padding(Padding::new(1, 1, 0, 0)))
                .style(Style::default().fg(Color::White));
            frame.render_widget(widget, logs_area);
        }
    }
}
