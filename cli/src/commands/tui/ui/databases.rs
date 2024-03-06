use crate::commands::tui::app::App;
use ratatui::{
    prelude::{Backend, Constraint, Rect},
    style::{Color, Style},
    text::{Span, Text},
    widgets::{Block as WidgetBlock, Cell, Padding, Paragraph, Row, Table},
    Frame,
};
pub struct DatabasesWidget;

impl DatabasesWidget {
    pub fn draw<B: Backend>(app: &mut App, frame: &mut Frame<B>, rect: Rect) {
        if let Some(ref error) = app.state.databases.error {
            let error_message = create_error_block(error);
            frame.render_widget(error_message, rect);
            return;
        }
        // it will show loader only on the first call
        if app.state.databases.is_fetching {
            let loading_message = create_loading_block();
            frame.render_widget(loading_message, rect);
            return;
        }
        let name_style = Style::default().fg(Color::White);

        let rows = app
            .state
            .databases
            .database_instances
            .iter()
            .map(|database_instance| {
                Row::new(vec![
                    Cell::from(Span::styled(database_instance.name.to_string(), name_style)),
                    Cell::from(Span::styled(
                        database_instance.cpu_utilization.to_string(),
                        name_style,
                    )),
                    Cell::from(Span::styled(
                        database_instance.memory_usage.to_string(),
                        name_style,
                    )),
                    Cell::from(Span::styled(
                        database_instance.memory_free.to_string(),
                        name_style,
                    )),
                    Cell::from(Span::styled(
                        database_instance.memory_cache.to_string(),
                        name_style,
                    )),
                    Cell::from(Span::styled(
                        database_instance.connections_count.to_string(),
                        name_style,
                    )),
                ])
            })
            .collect::<Vec<Row>>();

        let widget = Table::new(rows)
            .header(Row::new(vec![
                Cell::from(Span::styled("Database ID", name_style)),
                Cell::from(Span::styled("CPU Utilization", name_style)),
                Cell::from(Span::styled("Memory Usage", name_style)),
                Cell::from(Span::styled("Memory Free", name_style)),
                Cell::from(Span::styled("Memory Cached", name_style)),
                Cell::from(Span::styled("Connections Count", name_style)),
            ]))
            .widths(&[Constraint::Min(20), Constraint::Length(1000)])
            .column_spacing(1);

        frame.render_widget(widget, rect);
    }
}

fn create_loading_block() -> Paragraph<'static> {
    Paragraph::new(Text::styled(
        "Loading...",
        Style::default().fg(Color::White),
    ))
    .block(WidgetBlock::default())
}

fn create_custom_text_block(text: String) -> Paragraph<'static> {
    Paragraph::new(Text::styled(text, Style::default().fg(Color::White)))
        .block(WidgetBlock::default())
}

fn create_error_block(error: &str) -> Paragraph<'_> {
    Paragraph::new(Text::styled(error, Style::default().fg(Color::White)))
        .block(WidgetBlock::default().padding(Padding::new(1, 1, 0, 0)))
}
