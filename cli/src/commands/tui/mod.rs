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
    NamespaceSelection,
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
        terminal.draw(|frame| ui::draw(frame, app))?;

        let result = match event_manager.next().unwrap() {
            events::Event::Input(key) => app.handle_input(key),
            events::Event::Tick => app.update(),
        };

        if result == AppReturn::Exit {
            break;
        }
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
