use std::{rc::Rc, time::Instant};

use crate::commands::tui::app::App;
use ratatui::{
    prelude::{Alignment, Backend, Constraint, Direction, Layout, Margin, Rect},
    style::{self, Color, Style},
    text::{Line, Span},
    widgets::{Block as WidgetBlock, Borders, Padding, Paragraph, Wrap},
    Frame,
};

pub struct WelcomeWidget;

impl WelcomeWidget {
    pub fn draw<B: Backend>(app: &mut App, frame: &mut Frame<B>) {
        // Get terminal size
        let frame_size = frame.size();

        if let [_top, center, _bottom] = *Self::split_main_frame(frame_size) {
            Self::draw_main_block(frame, center);

            if let [center_top, center_center, center_bottom] = *Self::split_layout_center(center) {
                Self::draw_version_info(frame, center_top);
                Self::draw_verification_info(app, frame, center_center);
                Self::draw_redirect_block(app, frame, center_bottom);
                Self::draw_auth_instructions(app, frame, center_bottom);
            }
        }
    }

    fn split_main_frame(frame_size: Rect) -> Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(30),
                    Constraint::Percentage(35),
                    Constraint::Percentage(35),
                ]
                .as_ref(),
            )
            .horizontal_margin(frame_size.width / 5)
            .split(frame_size)
    }

    fn split_layout_center(center: Rect) -> Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ]
                .as_ref(),
            )
            .split(center.inner(&Margin {
                vertical: 1,
                horizontal: 1,
            }))
    }

    fn draw_main_block<B: Backend>(frame: &mut Frame<B>, center: Rect) {
        let main_block = WidgetBlock::default()
            .title(" Wukong CLI ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .padding(Padding::uniform(1));

        frame.render_widget(main_block, center);
    }

    fn draw_version_info<B: Backend>(frame: &mut Frame<B>, top: Rect) {
        let version_info_block = WidgetBlock::default()
            .title(format!("Version: {}", env!("CARGO_PKG_VERSION")))
            .title_alignment(Alignment::Center)
            .padding(Padding::new(1, 1, 1, 0))
            .style(Style::default().fg(Color::DarkGray));

        frame.render_widget(version_info_block, top);
    }

    fn draw_verification_info<B: Backend>(app: &App, frame: &mut Frame<B>, layout: Rect) {
        let okta_status =
            Self::get_verification_status(app.state.is_okta_authenticated, "Okta token");
        let gcloud_status =
            Self::get_verification_status(app.state.is_gcloud_authenticated, "GCloud token");

        let application_widget = Paragraph::new(vec![
            Line::from(Self::format_status_line(okta_status)),
            Line::from(Self::format_status_line(gcloud_status)),
        ])
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

        frame.render_widget(application_widget, layout);
    }

    fn draw_redirect_block<B: Backend>(app: &App, frame: &mut Frame<B>, layout: Rect) {
        if let Some(timer) = app.state.welcome_screen_timer {
            let time_remaining = timer.saturating_duration_since(Instant::now());

            let redirect_block = Paragraph::new(vec![Line::styled(
                format!("Redirecting in {} seconds...", time_remaining.as_secs()),
                Style::default().fg(Color::DarkGray),
            )])
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

            frame.render_widget(redirect_block, layout);
        }
    }

    fn draw_auth_instructions<B: Backend>(app: &App, frame: &mut Frame<B>, layout: Rect) {
        if let (Some(gcloud_authenticated), Some(okta_authenticated)) = (
            app.state.is_gcloud_authenticated,
            app.state.is_okta_authenticated,
        ) {
            let gcloud_auth_block = Paragraph::new(vec![
                Line::styled(
                    Self::get_auth_instruction(
                        okta_authenticated,
                        "Please run `wukong init` to authenticate again. Press `q` to exit.",
                    ),
                    Style::default().fg(Color::Red),
                ),
                Line::styled(
                    Self::get_auth_instruction(
                        gcloud_authenticated,
                        "Please authenticate gcloud and try again. Press `q` to exit.",
                    ),
                    Style::default().fg(Color::Red),
                ),
            ])
            .alignment(Alignment::Center);

            frame.render_widget(gcloud_auth_block, layout);
        }
    }

    fn get_verification_status(status: Option<bool>, label: &str) -> (String, String, Color) {
        let (symbol, message, color) = match status {
            Some(true) => ("✓", format!(" {} token verified.", label), Color::Green),
            Some(false) => (
                "x",
                format!(" Failed to verify {} token.", label),
                Color::Red,
            ),
            None => ("", format!("Verifying {} token...", label), Color::Red),
        };

        (symbol.to_string(), format!("{}", message), color)
    }

    fn format_status_line((symbol, message, color): (String, String, Color)) -> Vec<Span<'static>> {
        vec![
            Span::styled(
                symbol,
                Style::default()
                    .fg(color)
                    .add_modifier(style::Modifier::BOLD),
            ),
            Span::raw(message),
        ]
    }

    fn get_auth_instruction(authenticated: bool, message: &str) -> &str {
        if !authenticated {
            message
        } else {
            ""
        }
    }
}
