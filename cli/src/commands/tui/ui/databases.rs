use crate::commands::tui::app::{App};
use ratatui::{
    prelude::{Backend, Constraint, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Cell, Row, Table},
    Frame,
};
pub struct DatabasesWidget;

impl DatabasesWidget {
    pub fn draw<B: Backend>(_app: &mut App, frame: &mut Frame<B>, rect: Rect) {
        let name_style = Style::default().fg(Color::White);

        // Get database instances, cpu usage, free memory, and connections count from the cloudsql client
        let database_instances = "instance1"; //cloudsql_client.get_database_instances();
        let cpu_usage = "90%"; //cloudsql_client.get_cpu_usage();
        let free_memory = "10M"; //cloudsql_client.get_free_memory();
        let connections_count = "1000/5000"; //cloudsql_client.get_connections_count();

        let rows = vec![
            Row::new(vec![
                Cell::from(Span::styled("Database Instances", name_style)),
                Cell::from(Span::styled(database_instances.to_string(), name_style)),
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