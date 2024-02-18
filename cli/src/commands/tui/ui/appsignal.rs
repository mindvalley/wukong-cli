use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Padding, Paragraph, Row, Table},
    Frame,
};

use crate::commands::tui::app::{App, Block};

pub struct AppsignalWidget;

impl AppsignalWidget {
    pub fn draw<B: Backend>(app: &mut App, frame: &mut Frame<B>, rect: Rect) {
        // app.state.logs_widget_width = rect.width;
        // app.state.logs_widget_height = rect.height;

        // app.update_draw_lock(Block::Log, rect);

        let [top, bottom] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(10), Constraint::Length(100)].as_ref())
            .split(rect)
        else {
            return;
        };

        if app.state.is_fetching_appsignal_data {
            let loading_widget = Paragraph::new(Text::styled(
                "Loading...",
                Style::default().fg(Color::White),
            ));
            frame.render_widget(loading_widget, rect);
            return;
        }

        let namespace = Paragraph::new(format!("namespace: web"));
        frame.render_widget(namespace, top);

        let table = Table::new(vec![Row::new(vec!["ID", "Name", "Age"])]).column_spacing(1);
        frame.render_widget(table, bottom);
    }
}
