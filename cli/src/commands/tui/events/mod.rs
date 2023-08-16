use std::{sync::mpsc, thread, time::Duration};

use crossterm::event::{self, KeyEvent};

pub mod key;

pub enum Event {
    Input(KeyEvent),
    Tick,
}

pub struct EventManager {
    rx: mpsc::Receiver<Event>,
    // Need to be kept around to prevent disposing the sender side.
    _tx: mpsc::Sender<Event>,
}

impl EventManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        Self { rx, _tx: tx }
    }

    pub fn spawn_event_listen_thread(&self, tick_rate: Duration) {
        let event_tx = self._tx.clone();
        thread::spawn(move || {
            loop {
                // poll for tick rate duration, if no event, sent tick event.
                if event::poll(tick_rate).unwrap() {
                    if let event::Event::Key(key) = event::read().unwrap() {
                        event_tx.send(Event::Input(key)).unwrap();
                    }
                }

                event_tx.send(Event::Tick).unwrap();
            }
        });
    }

    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.rx.recv()
    }
}
