use std::{io::stdout, sync::Arc, time::Duration};

use crate::{
    config::{ApiChannel, Config},
    error::WKCliError,
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
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
    VersionSelection,
    LogSearchBar,
    LogFilterIncludeBar,
    LogFilterExcludeBar,
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

pub async fn handle_tui(channel: ApiChannel) -> Result<bool, WKCliError> {
    let config = Config::load_from_default_path()?;

    let (sender, mut receiver) = tokio::sync::mpsc::channel::<NetworkEvent>(100);

    let app = Arc::new(Mutex::new(App::new(&config, sender)));
    let app_ui = Arc::clone(&app);

    let arc_channel = Arc::new(channel);

    tokio::spawn(async move {
        while let Some(network_event) = receiver.recv().await {
            let app = Arc::clone(&app);
            let arc_channel = Arc::clone(&arc_channel);

            tokio::spawn(async move {
                let _ = handle_network_event(app, network_event, &arc_channel).await;
            });
        }
    });

    start_ui(&app_ui).await?;

    Ok(true)
}

pub async fn start_ui(app: &Arc<Mutex<App>>) -> std::io::Result<bool> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .expect("unable to enter alternate screen");

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;

    // The lower the tick_rate, the higher the FPS, but also the higher the CPU usage.
    let tick_rate = Duration::from_millis(1000);
    let event_manager = EventManager::new();
    event_manager.spawn_event_listen_thread(tick_rate);

    let mut is_first_render = true;
    let mut is_first_fetch_builds = true;

    loop {
        let mut app_ref = app.lock().await;

        terminal.draw(|frame| ui::draw(frame, &mut app_ref))?;

        // move cursor to the top left corner to avoid screen scrolling:
        // terminal.set_cursor(1, 1)?;

        let result = match event_manager.next().unwrap() {
            events::Event::Input(key) => app_ref.handle_input(key).await,
            events::Event::Tick => app_ref.update().await,
        };

        if result == AppReturn::Exit {
            break;
        }

        if is_first_render {
            // fetch data on the first frame
            app_ref.dispatch(NetworkEvent::GetDeployments).await;

            is_first_render = false;
        }

        if is_first_fetch_builds
            && app_ref.state.current_namespace.is_some()
            && app_ref.state.current_version.is_some()
        {
            app_ref.dispatch(NetworkEvent::GetBuilds).await;
            is_first_fetch_builds = false;
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
    terminal.show_cursor()?;

    terminal.clear()?;

    Ok(true)
}
