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

        create_custom_text_block("fucking up here".to_string());

        // todo!();
        let name_style = Style::default().fg(Color::White);

        // Get database instances, cpu usage, free memory, and connections count from the cloudsql client
        let database_instances = "instance1"; //cloudsql_client.get_database_instances();
        let cpu_usage = "90%"; //cloudsql_client.get_cpu_usage();
        let free_memory = "10M"; //cloudsql_client.get_free_memory();
        let connections_count = "1000/5000"; //cloudsql_client.get_connections_count();

        let rows = vec![
            Row::new(vec![
                Cell::from(Span::styled("Database Instances", name_style)),
                Cell::from(Span::styled(
                    app.state.databases.database_instances[0].name.to_string(),
                    name_style,
                )),
            ]),
            Row::new(vec![
                Cell::from(Span::styled("CPU Usage", name_style)),
                Cell::from(Span::styled(cpu_usage.to_string(), name_style)),
            ]),
            Row::new(vec![
                Cell::from(Span::styled("Free Memory", name_style)),
                Cell::from(Span::styled(free_memory.to_string(), name_style)),
            ]),
            Row::new(vec![
                Cell::from(Span::styled("Connections Count", name_style)),
                Cell::from(Span::styled(connections_count.to_string(), name_style)),
            ]),
        ];

        let widget = Table::new(rows)
            .header(Row::new(vec![
                Cell::from(Span::styled("Metric", name_style)),
                Cell::from(Span::styled("Value", name_style)),
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
