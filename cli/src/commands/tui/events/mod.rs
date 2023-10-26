pub mod key;
pub mod network;

use self::key::Key;
use crossterm::event::{self, KeyEventKind, MouseEvent, MouseEventKind};
use std::{sync::mpsc, time::Duration};

pub enum Event {
    Input(Key),
    MouseInput(MouseEvent),
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
                                    if key.kind == KeyEventKind::Press
                                        && event_sender.send(Event::Input(key.into())).is_err()
                                    {
                                        break;
                                    }
                                }
                                event::Event::Mouse(mouse_event) => {
                                    if let MouseEventKind::Down(_) = mouse_event.kind {
                                        if event_sender
                                            .send(Event::MouseInput(mouse_event))
                                            .is_err()
                                        {
                                            break;
                                        }
                                    }
                                }
                                event::Event::FocusGained => {}
                                event::Event::FocusLost => {}
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
