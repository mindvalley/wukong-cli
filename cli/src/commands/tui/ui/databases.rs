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
        if app.state.is_fetching_database_metrics {
            let loading_message = create_loading_block();
            frame.render_widget(loading_message, rect);
            return;
        }
        let name_style = Style::default().fg(Color::White);

        let rows = app
            .state
            .databases
            .database_metrics
            .iter()
            .map(|database_instance| {
                Row::new(vec![
                    Cell::from(Span::styled(
                        format!("{:}", database_instance.name),
                        name_style,
                    )),
                    Cell::from(Span::styled(
                        format!("{:>15.2}", database_instance.cpu_utilization),
                        name_style,
                    )),
                    Cell::from(Span::styled(
                        format!("{:>12.2}", database_instance.memory_usage),
                        name_style,
                    )),
                    Cell::from(Span::styled(
                        format!("{:>11.2}", database_instance.memory_free),
                        name_style,
                    )),
                    Cell::from(Span::styled(
                        format!("{:>13.2}", database_instance.memory_cache),
                        name_style,
                    )),
                    Cell::from(Span::styled(
                        format!("{:>17}", database_instance.connections_count),
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
            .widths(&[
                Constraint::Min(70),
                Constraint::Length(18),
                Constraint::Length(15),
                Constraint::Length(15),
                Constraint::Length(15),
                Constraint::Length(18),
            ])
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

fn create_error_block(error: &str) -> Paragraph<'_> {
    Paragraph::new(Text::styled(error, Style::default().fg(Color::White)))
        .block(WidgetBlock::default().padding(Padding::new(1, 1, 0, 0)))
}
