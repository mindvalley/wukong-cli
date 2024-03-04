use std::{
    io::stdout,
    panic::{self, PanicInfo},
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    config::{ApiChannel, Config},
    error::WKCliError,
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    style::Print,
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
mod handlers;
mod ui;

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

    let channel = Arc::new(channel);

    let config = Arc::new(Config::load_from_default_path()?);

    // Set panic hook
    panic::set_hook(Box::new(panic_hook));

    tokio::spawn(async move {
        while let Some(network_event) = receiver.recv().await {
            let app_clone = Arc::clone(&app);
            let channel_clone = Arc::clone(&channel);
            let config_clone = Arc::clone(&config);

            tokio::spawn(async move {
                let _ = handle_network_event(app_clone, network_event, channel_clone, config_clone)
                    .await;
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
    let tick_rate = Duration::from_millis(250);
    let event_manager = EventManager::new();
    event_manager.spawn_event_listen_thread(tick_rate);

    let mut show_welcome_screen = true;
    let mut is_welcome_screen_first_frame = true;
    let mut is_first_fetch_builds = true;
    let mut is_first_fetch_appsignals = true;
    let mut is_first_fetch_database_metrics = true;

    let mut app_ref = app.lock().await;

    terminal.draw(|frame| ui::draw_welcome_screen(frame, &mut app_ref))?;

    drop(app_ref);

    loop {
        let mut app_ref = app.lock().await;

        let result = match event_manager.next().unwrap() {
            events::Event::Input(key) => handlers::input_handler(key, &mut app_ref).await,
            events::Event::MouseInput(mouse_event) => {
                handlers::handle_mouse_event(mouse_event, &mut app_ref)
            }
            events::Event::Tick => app_ref.update().await,
        };

        if result == AppReturn::Exit {
            break;
        }

        if let (Some(gcloud_authenticated), Some(okta_authenticated)) = (
            app_ref.state.is_gcloud_authenticated,
            app_ref.state.is_okta_authenticated,
        ) {
            if gcloud_authenticated && okta_authenticated && show_welcome_screen {
                if app_ref.state.welcome_screen_timer.is_none() {
                    app_ref.state.welcome_screen_timer =
                        Some(Instant::now() + Duration::from_secs(4));

                    // fetch data on the first frame
                    // fetch deployments first, as we need the namespace and version to fetch builds and appsignal data
                    app_ref.dispatch(NetworkEvent::GetDeployments).await;
                }

                if let Some(timer) = app_ref.state.welcome_screen_timer {
                    let time_remaining = timer.saturating_duration_since(Instant::now());

                    if time_remaining == Duration::from_secs(0) {
                        // Reset the timer
                        app_ref.state.welcome_screen_timer.take();
                        show_welcome_screen = false;
                    }
                }
            } else {
                app_ref.state.welcome_screen_timer = None;
            }
        } else {
            app_ref.state.welcome_screen_timer = None;
        }

        if show_welcome_screen {
            if is_welcome_screen_first_frame {
                app_ref.dispatch(NetworkEvent::VerifyOktaRefreshToken).await;
                app_ref.dispatch(NetworkEvent::VerifyGCloudToken).await;
            }
            terminal.draw(|frame| ui::draw_welcome_screen(frame, &mut app_ref))?;
            is_welcome_screen_first_frame = false;
        } else {
            terminal.draw(|frame| ui::draw_main_screen(frame, &mut app_ref))?;
        }

        // builds and appsignal data require a namespace and version to be selected
        // and we only know the namespace and version after deployments are fetched
        if app_ref.state.current_namespace.is_some() && app_ref.state.current_version.is_some() {
            if is_first_fetch_builds {
                app_ref.dispatch(NetworkEvent::GetBuilds).await;
                is_first_fetch_builds = false;
            }
            if is_first_fetch_appsignals {
                app_ref.dispatch(NetworkEvent::GetAppsignalData).await;
                is_first_fetch_appsignals = false;
            }

            if is_first_fetch_database_metrics {
                app_ref.dispatch(NetworkEvent::GetDatabaseMetrics).await;
                is_first_fetch_database_metrics = false;
            }
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

pub fn panic_hook(panic_info: &PanicInfo<'_>) {
    let mut stdout = stdout();

    let msg = match panic_info.payload().downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => match panic_info.payload().downcast_ref::<String>() {
            Some(s) => &s[..],
            None => "Box<Any>",
        },
    };

    let _ = disable_raw_mode();
    let _ = execute!(stdout, DisableMouseCapture, LeaveAlternateScreen);

    // Print stack trace. Must be done after!
    if let Some(panic_info) = panic_info.location() {
        let _ = execute!(
            stdout,
            Print(format!(
                "thread '<unnamed>' panicked at '{msg}', {panic_info}",
            )),
        );
    }
}
