use std::{sync::Arc, time::Duration};

use crate::{config::Config, error::WKCliError};
use crossterm::{
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::CrosstermBackend, widgets::ListState, Terminal};
use tokio::sync::Mutex;

use self::{
    app::{App, AppReturn},
    events::{
        network::{handle_network_event, NetworkEvent},
        EventManager,
    },
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

    let (sender, mut receiver) = tokio::sync::mpsc::channel::<NetworkEvent>(100);

    let app = Arc::new(Mutex::new(App::new(&config, sender)));
    let app_ui = Arc::clone(&app);

    tokio::spawn(async move {
        while let Some(network_event) = receiver.recv().await {
            let app = Arc::clone(&app);
            tokio::spawn(async move {
                let _ = handle_network_event(app, network_event).await;
            });
        }
    });

    start_ui(&app_ui).await?;

    Ok(true)
}

pub async fn start_ui(app: &Arc<Mutex<App>>) -> std::io::Result<bool> {
    let mut stdout = std::io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen).expect("unable to enter alternate screen");

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    // The lower the tick_rate, the higher the FPS, but also the higher the CPU usage.
    let tick_rate = Duration::from_millis(200);
    let event_manager = EventManager::new();
    event_manager.spawn_event_listen_thread(tick_rate);

    let mut the_first_frame = true;

    loop {
        let mut app_ref = app.lock().await;

        if the_first_frame {
            // fetch data on the first frame
            app_ref.dispatch(NetworkEvent::FetchDeployments).await;
            app_ref.dispatch(NetworkEvent::FetchBuilds).await;

            the_first_frame = false;
        }

        terminal.draw(|frame| ui::draw(frame, &mut app_ref))?;

        let result = match event_manager.next().unwrap() {
            events::Event::Input(key) => app_ref.handle_input(key).await,
            events::Event::Tick => app_ref.update().await,
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
