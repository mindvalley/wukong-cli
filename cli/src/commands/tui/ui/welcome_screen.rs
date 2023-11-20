use std::time::Instant;

use crate::commands::tui::app::App;
use ratatui::{
    prelude::{Alignment, Backend, Constraint, Direction, Layout, Margin},
    style::{self, Color, Style},
    text::{Line, Span},
    widgets::{Block as WidgetBlock, Borders, Padding, Paragraph, Wrap},
    Frame,
};

pub struct WelcomeWidget;

impl WelcomeWidget {
    pub fn draw<B: Backend>(app: &mut App, frame: &mut Frame<B>) {
        let is_okta_authenticated = app.state.is_okta_authenticated;
        let is_gcloud_authenticated = app.state.is_gcloud_authenticated;

        // // Get terminal size
        let frame_size = frame.size();

        let [_top, center, _bottom] = *Layout::default()
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
        else {
            return;
        };

        let main_block = WidgetBlock::default()
            .title(" Wukong CLI ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .padding(Padding::uniform(1));

        frame.render_widget(main_block, center);

        let title_block = WidgetBlock::default()
            .title(format!("Version: {}", env!("CARGO_PKG_VERSION")))
            .title_alignment(Alignment::Center)
            .padding(Padding::new(1, 1, 1, 0))
            .style(Style::default().fg(Color::DarkGray));

        let [version_info, okta_verification, gcloud_verification] = *Layout::default()
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
        else {
            return;
        };

        frame.render_widget(title_block, version_info);

        let application_widget = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    if let Some(is_okta_authenticated) = is_okta_authenticated {
                        if is_okta_authenticated {
                            "✓"
                        } else {
                            "x"
                        }
                    } else {
                        ""
                    },
                    Style::default()
                        .fg(if let Some(is_okta_authenticated) = is_okta_authenticated {
                            if is_okta_authenticated {
                                Color::Green
                            } else {
                                Color::Red
                            }
                        } else {
                            Color::Red
                        })
                        .add_modifier(style::Modifier::BOLD),
                ),
                Span::raw(if let Some(is_okta_authenticated) = is_okta_authenticated {
                    if is_okta_authenticated {
                        " Okta token verified."
                    } else {
                        " Failed to verify Okta token."
                    }
                } else {
                    " Verifying Okta token..."
                }),
            ]),
            Line::from(vec![
                Span::styled(
                    if let Some(is_gcloud_authenticated) = is_gcloud_authenticated {
                        if is_gcloud_authenticated {
                            "✓"
                        } else {
                            "x"
                        }
                    } else {
                        ""
                    },
                    Style::default()
                        .fg(
                            if let Some(is_gcloud_authenticated) = is_gcloud_authenticated {
                                if is_gcloud_authenticated {
                                    Color::Green
                                } else {
                                    Color::Red
                                }
                            } else {
                                Color::Red
                            },
                        )
                        .add_modifier(style::Modifier::BOLD),
                ),
                Span::raw(
                    if let Some(is_gcloud_authenticated) = is_gcloud_authenticated {
                        if is_gcloud_authenticated {
                            " GCloud token verified."
                        } else {
                            " Failed to verify GCloud token."
                        }
                    } else {
                        " Verifying GCloud token..."
                    },
                ),
            ]),
        ])
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

        frame.render_widget(application_widget, okta_verification);

        if let Some(timer) = app.state.welcome_screen_timer {
            let time_remaining = timer.saturating_duration_since(Instant::now());

            let redirect_block = Paragraph::new(vec![Line::styled(
                format!("Redirecting in {} seconds...", time_remaining.as_secs()),
                Style::default().fg(Color::DarkGray),
            )])
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

            frame.render_widget(redirect_block, gcloud_verification);
        }

        if let (Some(gcloud_authenticated), Some(okta_authenticated)) = (
            app.state.is_gcloud_authenticated,
            app.state.is_okta_authenticated,
        ) {
            let gcloud_auth_block = Paragraph::new(vec![
                Line::styled(
                    if !okta_authenticated {
                        "Please run `wukong init` to authenticate again. Press `q` to exit."
                    } else {
                        ""
                    },
                    Style::default().fg(Color::Red),
                ),
                Line::styled(
                    if !gcloud_authenticated {
                        "Please authenticate gcloud and run again. Press `q` to exit."
                    } else {
                        ""
                    },
                    Style::default().fg(Color::Red),
                ),
            ])
            .alignment(Alignment::Center);

            frame.render_widget(gcloud_auth_block, gcloud_verification);
        }
    }
}
