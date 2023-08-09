use crate::error::WKCliError;
use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::{Backend, Constraint, CrosstermBackend, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
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

    fn draw<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> std::io::Result<bool> {
        loop {
            terminal.draw(|f| self.ui(f))?;
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

    fn ui<B: Backend>(&self, f: &mut Frame<B>) {
        // Create the layout sections.
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(10),
                    Constraint::Percentage(45),
                    Constraint::Percentage(45),
                ]
                .as_ref(),
            )
            .split(f.size());

        let header_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());

        let header_area =
            Paragraph::new(Text::styled("", Style::default().fg(Color::White))).block(header_block);

        f.render_widget(header_area, chunks[0]);

        let logs_block = Block::default()
            .title(" Logs ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightGreen));
        let logs_area = Paragraph::new(Text::styled("", Style::default().fg(Color::LightGreen)))
            .block(logs_block);

        f.render_widget(logs_area, chunks[1]);

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

        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunks[2]);

        f.render_widget(builds_area, bottom_chunks[0]);
        f.render_widget(deployment_area, bottom_chunks[1]);

        if let CurrentScreen::Exiting = self.current_screen {
            f.render_widget(Clear, f.size()); //this clears the entire screen and anything already drawn
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

            let area = centered_rect(60, 25, f.size());
            f.render_widget(exit_paragraph, area);
        }
    }
}

pub async fn handle_tui() -> Result<bool, WKCliError> {
    let mut stdout = std::io::stdout();
    enable_raw_mode().expect("failed to enable raw mode");
    execute!(stdout, EnterAlternateScreen).expect("unable to enter alternate screen");

    let mut terminal =
        Terminal::new(CrosstermBackend::new(stdout)).expect("creating terminal failed");

    let mut painter = Painter::new();
    painter.draw(&mut terminal)?;

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
