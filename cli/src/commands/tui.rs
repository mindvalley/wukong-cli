use crate::{config::Config, error::WKCliError};
use clap::crate_version;
use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::{Backend, Constraint, CrosstermBackend, Direction, Layout, Rect},
    style::{self, Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap},
    Frame, Terminal,
};

enum CurrentScreen {
    Main,
    Exiting,
}

struct Painter {
    current_screen: CurrentScreen,
}

impl Painter {
    fn new() -> Self {
        Self {
            current_screen: CurrentScreen::Main,
        }
    }

    fn draw<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        config: Config,
    ) -> std::io::Result<bool> {
        loop {
            terminal.draw(|f| self.ui(f, &config))?;
            if let Event::Key(key) = event::read()? {
                match self.current_screen {
                    CurrentScreen::Main => {
                        if let KeyCode::Char('q') = key.code {
                            self.current_screen = CurrentScreen::Exiting;
                        }
                    }
                    CurrentScreen::Exiting => match key.code {
                        KeyCode::Char('y') => {
                            return Ok(true);
                        }
                        KeyCode::Char('n') => {
                            self.current_screen = CurrentScreen::Main;
                        }
                        _ => {}
                    },
                }
            }
        }
    }

    fn ui<B: Backend>(&self, frame: &mut Frame<B>, config: &Config) {
        // Create the layout sections.
        let [top, mid, bottom] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Ratio(1, 7),
                    Constraint::Ratio(3, 7),
                    Constraint::Ratio(3, 7),
                ]
                .as_ref(),
            )
            .split(frame.size())
        else {
            return;
        };

        let [top_left, top_right] = *Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(top)
        else {
            return;
        };

        let [bottom_left, bottom_right] = *Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(bottom)
        else {
            return;
        };

        let application_block = Block::default()
            .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
            .padding(Padding::new(1, 0, 0, 0))
            .style(Style::default());

        // default namespace is "Prod"
        let selected_namespace = "prod";

        let application_area = Paragraph::new(vec![
            Line::from(vec![
                Span::raw("Application: "),
                Span::styled(
                    &config.core.application,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(style::Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("Namespace: "),
                Span::styled(
                    selected_namespace,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(style::Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("CLI Version: "),
                Span::styled(
                    crate_version!(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(style::Modifier::BOLD),
                ),
            ]),
        ])
        .wrap(Wrap { trim: true })
        .block(application_block);

        let view_controls_block = Block::default()
            .borders(Borders::RIGHT | Borders::TOP | Borders::BOTTOM)
            .style(Style::default());

        let view_controls_area = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    "<n> ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(style::Modifier::BOLD),
                ),
                Span::raw("Select namespace"),
            ]),
            Line::from(vec![
                Span::styled(
                    "<q> ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(style::Modifier::BOLD),
                ),
                Span::raw("Quit"),
            ]),
        ])
        .block(view_controls_block)
        .wrap(Wrap { trim: true });

        frame.render_widget(application_area, top_left);
        frame.render_widget(view_controls_area, top_right);

        let logs_block = Block::default()
            .title(" Logs ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightGreen));
        let logs_area = Paragraph::new(Text::styled("", Style::default().fg(Color::LightGreen)))
            .block(logs_block);

        frame.render_widget(logs_area, mid);

        let builds_block = Block::default()
            .title(" Build Artifacts ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightYellow));

        let builds_area = Paragraph::new(Text::styled("", Style::default().fg(Color::LightYellow)))
            .block(builds_block);

        let deployment_block = Block::default()
            .title(" Deployment ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightBlue));

        let deployment_area =
            Paragraph::new(Text::styled("", Style::default().fg(Color::LightBlue)))
                .block(deployment_block);

        frame.render_widget(builds_area, bottom_left);
        frame.render_widget(deployment_area, bottom_right);

        if let CurrentScreen::Exiting = self.current_screen {
            frame.render_widget(Clear, frame.size()); //this clears the entire screen and anything already drawn
            let popup_block = Block::default()
                .title("Y/N")
                .borders(Borders::NONE)
                .style(Style::default().bg(Color::DarkGray));

            let exit_text = Text::styled(
                "Would you like to exit? (y/n)",
                Style::default().fg(Color::Red),
            );
            // the `trim: false` will stop the text from being cut off when over the edge of the block
            let exit_paragraph = Paragraph::new(exit_text)
                .block(popup_block)
                .wrap(Wrap { trim: false });

            let area = centered_rect(60, 25, frame.size());
            frame.render_widget(exit_paragraph, area);
        }
    }
}

pub async fn handle_tui() -> Result<bool, WKCliError> {
    let config = Config::load_from_default_path()?;

    let mut stdout = std::io::stdout();
    enable_raw_mode().expect("failed to enable raw mode");
    execute!(stdout, EnterAlternateScreen).expect("unable to enter alternate screen");

    let mut terminal =
        Terminal::new(CrosstermBackend::new(stdout)).expect("creating terminal failed");

    let mut painter = Painter::new();
    painter.draw(&mut terminal, config)?;

    // post-run
    disable_raw_mode().expect("failed to disable raw mode");
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .expect("unable to leave alternate screen");
    terminal.show_cursor().expect("failed to show cursor");

    Ok(true)
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1] // Return the middle chunk
}
