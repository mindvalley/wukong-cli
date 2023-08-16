use crate::{config::Config, error::WKCliError};
use clap::crate_version;
use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::{Alignment, Backend, Constraint, CrosstermBackend, Direction, Layout, Rect},
    style::{self, Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Padding, Paragraph, Wrap},
    Frame, Terminal,
};

use self::hotkey::HotKey;

mod hotkey;

enum CurrentScreen {
    Main,
    Exiting,
}

struct State {
    current_application: String,
    current_namespace: String,
    show_namespace_selection: bool,
}

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn select(&mut self, index: usize) {
        if index >= self.items.len() {
            return;
        }

        self.state.select(Some(index));
    }

    #[allow(dead_code)]
    fn unselect(&mut self) {
        self.state.select(None);
    }
}

struct Painter {
    state: State,
    namespace_selections: StatefulList<String>,
    current_screen: CurrentScreen,
}

impl Painter {
    fn new(config: &Config) -> Self {
        let mut namespace_selections =
            StatefulList::with_items(vec![String::from("prod"), String::from("staging")]);
        namespace_selections.select(0);

        Self {
            state: State {
                current_application: config.core.application.clone(),
                current_namespace: String::from("prod"),
                show_namespace_selection: false,
            },
            namespace_selections,
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
                if self.state.show_namespace_selection {
                    if let CurrentScreen::Main = self.current_screen {
                        match key.code {
                            KeyCode::Up => {
                                self.namespace_selections.previous();
                            }
                            KeyCode::Down => {
                                self.namespace_selections.next();
                            }
                            KeyCode::Enter => {
                                self.state.current_namespace = self
                                    .namespace_selections
                                    .items
                                    .get(self.namespace_selections.state.selected().unwrap())
                                    .unwrap()
                                    .clone();

                                self.state.show_namespace_selection = false;
                            }
                            KeyCode::Char('q') => {
                                self.state.show_namespace_selection = false;
                            }
                            _ => {}
                        }
                    }
                } else {
                    match self.current_screen {
                        CurrentScreen::Main => match key.code {
                            KeyCode::Char('q') => {
                                self.current_screen = CurrentScreen::Exiting;
                            }
                            KeyCode::Char('n') => {
                                self.state.show_namespace_selection = true;
                            }
                            _ => {}
                        },
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
    }

    fn ui<B: Backend>(&mut self, frame: &mut Frame<B>, _config: &Config) {
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

        let application_area = Paragraph::new(vec![
            Line::from(vec![
                Span::raw("Application: "),
                Span::styled(
                    &self.state.current_application,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(style::Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("Namespace: "),
                Span::styled(
                    &self.state.current_namespace,
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

        let hotkeys = vec![HotKey::SelectNamespace, HotKey::Quit]
            .into_iter()
            .map(|hotkey| {
                Line::from(vec![
                    Span::styled(
                        format!("<{}>", hotkey.keycode()),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(style::Modifier::BOLD),
                    ),
                    Span::raw(format!(" {}", hotkey.desc())),
                ])
            })
            .collect::<Vec<_>>();

        let view_controls_block = Block::default()
            .borders(Borders::RIGHT | Borders::TOP | Borders::BOTTOM)
            .style(Style::default());

        let view_controls_area = Paragraph::new(hotkeys)
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

        if self.state.show_namespace_selection {
            let items: Vec<ListItem> = self
                .namespace_selections
                .items
                .iter()
                .map(|i| {
                    let lines = vec![Line::from(i.clone())];
                    ListItem::new(lines).style(Style::default().fg(Color::White))
                })
                .collect();

            let popup_block = Block::default()
                .title(" Namespace Selection ")
                .borders(Borders::ALL)
                .title_alignment(Alignment::Center)
                .style(Style::default().bg(Color::Black));

            // Create a List from all list items and highlight the currently selected one
            let items = List::new(items)
                .block(popup_block)
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            let area = centered_rect(60, 25, frame.size());
            frame.render_widget(Clear, area);
            // We can now render the item list
            frame.render_stateful_widget(items, area, &mut self.namespace_selections.state);
        }

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

    let mut painter = Painter::new(&config);
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
