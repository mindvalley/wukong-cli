use std::{sync::mpsc, time::Duration};

use crossterm::event::{self};

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
                if event::poll(tick_rate).unwrap() {
                    if let event::Event::Key(key) = event::read().unwrap() {
                        if event_sender.send(Event::Input(key.into())).is_err() {
                            break;
                        };
                    }
                }

                event_sender.send(Event::Tick).unwrap();
            }
        });
    }

    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.receiver.recv()
    }
}
