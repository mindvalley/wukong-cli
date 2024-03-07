use crate::commands::tui::app::{App, Block, SelectedTab};
use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block as WidgetBlock, Borders, Tabs},
    Frame,
};
use strum::IntoEnumIterator;

use super::{
    appsignal::AppsignalWidget, centered_rect_by_padding, databases::DatabasesWidget,
    logs::LogsWidget, util::get_color,
};

pub struct MiddleWidget;

impl MiddleWidget {
    pub fn draw<B: Backend>(app: &mut App, frame: &mut Frame<B>, rect: Rect) {
        app.state.logs_widget_width = rect.width;
        app.state.logs_widget_height = rect.height;

        app.update_draw_lock(Block::Middle(app.state.selected_tab), rect);

        let current_route = app.get_current_route();

        let highlight_state = (
            matches!(current_route.active_block, Block::Middle(_)),
            matches!(current_route.hovered_block, Block::Middle(_)),
        );

        let titles = SelectedTab::iter()
            .map(|tab| format!(" {tab} "))
            .map(Line::from)
            .collect::<Vec<_>>();

        let tab = Tabs::new(titles)
            .block(
                WidgetBlock::default()
                    .title(" Logs/Metrics ")
                    .borders(Borders::ALL)
                    .border_style(get_color(
                        highlight_state,
                        (Color::LightCyan, Color::LightGreen, Color::White),
                    )),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Black).bg(Color::Green))
            .select(app.state.selected_tab as usize);
        frame.render_widget(tab, rect);

        let inner_rect = centered_rect_by_padding(2, 2, 2, 1, rect);
        match app.state.selected_tab {
            SelectedTab::GCloud => LogsWidget::draw(app, frame, inner_rect),
            SelectedTab::AppSignal => AppsignalWidget::draw(app, frame, inner_rect),
            SelectedTab::Databases => DatabasesWidget::draw(app, frame, inner_rect),
        }
    }
}
