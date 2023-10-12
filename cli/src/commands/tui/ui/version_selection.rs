use super::{centered_rect, create_loading_widget};
use crate::commands::tui::app::App;
use ratatui::{
    prelude::{Alignment, Backend},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};

pub struct VersionSelectionWidget;

impl VersionSelectionWidget {
    pub fn draw<B: Backend>(app: &mut App, frame: &mut Frame<B>) {
        let items: Vec<ListItem> = app
            .version_selections
            .items
            .iter()
            .map(|i| {
                let lines = vec![Line::from(i.clone())];
                ListItem::new(lines).style(Style::default().fg(Color::White))
            })
            .collect();

        let popup_block = Block::default()
            .title(" Version Selection ")
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center)
            .style(Style::default().bg(Color::Black));

        let area = centered_rect(60, 25, frame.size());
        frame.render_widget(Clear, area);

        if app.state.is_checking_version {
            let loading_widget = create_loading_widget(popup_block);
            frame.render_widget(loading_widget, area);
        } else {
            let items = List::new(items)
                .block(popup_block)
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            frame.render_stateful_widget(items, area, &mut app.version_selections.state);
        }
    }
}
