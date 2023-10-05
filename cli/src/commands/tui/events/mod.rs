pub mod key;
pub mod network;

use self::key::Key;
use crossterm::event::{self, MouseEvent, MouseEventKind};
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
                if event::poll(tick_rate).unwrap() {
                    if let event::Event::Key(key) = event::read().unwrap() {
                        if event_sender.send(Event::Input(key.into())).is_err() {
                            break;
                        };
                        // Send only mouse click:
                    } else if let event::Event::Mouse(mouse_event) = event::read().unwrap() {
                        // Send only mouse click: to increase performance
                        if let MouseEventKind::Down(_) = mouse_event.kind {
                            if event_sender.send(Event::MouseInput(mouse_event)).is_err() {
                                break;
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
