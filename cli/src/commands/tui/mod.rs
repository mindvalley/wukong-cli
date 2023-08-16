use std::time::Duration;

use crate::{config::Config, error::WKCliError};
use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::{Backend, CrosstermBackend},
    widgets::ListState,
    Terminal,
};

use self::{
    app::{App, AppReturn},
    events::EventManager,
};

mod action;
mod app;
mod events;
mod ui;

pub enum CurrentScreen {
    Main,
    Exiting,
}

pub struct StatefulList<T> {
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

pub async fn handle_tui() -> Result<bool, WKCliError> {
    let config = Config::load_from_default_path()?;
    let mut app = App::new(&config);

    start_ui(&mut app)?;

    Ok(true)
}

pub fn start_ui(app: &mut App) -> std::io::Result<bool> {
    let mut stdout = std::io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen).expect("unable to enter alternate screen");

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let tick_rate = Duration::from_millis(200);
    let event_manager = EventManager::new();
    event_manager.spawn_event_listen_thread(tick_rate);

    // let network_manager = NetworkManager::new();
    // network_manager.spawn_network_thread();

    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        let result = match event_manager.next().unwrap() {
            events::Event::Input(key) => {
                todo!()
            }
            events::Event::Tick => app.update(),
        };

        if result == AppReturn::Exit {
            break;
        }

        // if let Event::Key(key) = event::read()? {
        //     if app.state.show_namespace_selection {
        //         if let CurrentScreen::Main = app.current_screen {
        //             match key.code {
        //                 KeyCode::Up => {
        //                     app.namespace_selections.previous();
        //                 }
        //                 KeyCode::Down => {
        //                     app.namespace_selections.next();
        //                 }
        //                 KeyCode::Enter => {
        //                     app.state.current_namespace = app
        //                         .namespace_selections
        //                         .items
        //                         .get(app.namespace_selections.state.selected().unwrap())
        //                         .unwrap()
        //                         .clone();
        //
        //                     app.state.show_namespace_selection = false;
        //                 }
        //                 KeyCode::Char('q') => {
        //                     app.state.show_namespace_selection = false;
        //                 }
        //                 _ => {}
        //             }
        //         }
        //     } else {
        //         match app.current_screen {
        //             CurrentScreen::Main => match key.code {
        //                 KeyCode::Char('q') => {
        //                     app.current_screen = CurrentScreen::Exiting;
        //                 }
        //                 KeyCode::Char('n') => {
        //                     app.state.show_namespace_selection = true;
        //                 }
        //                 _ => {}
        //             },
        //             CurrentScreen::Exiting => match key.code {
        //                 KeyCode::Char('y') => {
        //                     break;
        //                 }
        //                 KeyCode::Char('n') => {
        //                     app.current_screen = CurrentScreen::Main;
        //                 }
        //                 _ => {}
        //             },
        //         }
        //     }
        // }
    }

    // post-run
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .expect("unable to leave alternate screen");

    terminal.clear()?;
    terminal.show_cursor()?;

    Ok(true)
}
