use std::{sync::mpsc, time::Duration};

use crossterm::event::{self, KeyEventKind};

use self::key::Key;

pub mod key;
pub mod network;

pub enum Event {
    Input(Key),
    Tick,
}

pub struct EventManager {
    receiver: mpsc::Receiver<Event>,
    // Need to be kept around to prevent disposing the sender side.
    sender: mpsc::Sender<Event>,
}

impl EventManager {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();

        Self { receiver, sender }
    }

    pub fn spawn_event_listen_thread(&self, tick_rate: Duration) {
        let event_sender = self.sender.clone();

        std::thread::spawn(move || {
            loop {
                // poll for tick rate duration, if no event, sent tick event.
                if let Ok(poll) = event::poll(tick_rate) {
                    if poll {
                        if let Ok(event) = event::read() {
                            match event {
                                event::Event::Key(key) => {
                                    // https://ratatui.rs/tutorial/counter-async-app/async-event-stream.html#admonition-attention
                                    if key.kind == KeyEventKind::Press {
                                        if event_sender.send(Event::Input(key.into())).is_err() {
                                            break;
                                        };
                                    }
                                }
                                event::Event::FocusGained => {}
                                event::Event::FocusLost => {}
                                event::Event::Mouse(_) => {}
                                event::Event::Paste(_) => {}
                                event::Event::Resize(_, _) => {}
                            }
                        }
                    }
                }

                event_sender.send(Event::Tick).unwrap();
            }
        });
    }

    /// Attempts to read an event.
    /// This funtion will block the current thread.
    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.receiver.recv()
    }
}
