use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use termion::{event::Key, input::TermRead};

pub const TICK_RATE: u64 = 250;

pub enum Event {
    Input(Key),
    Tick(u64),
}

pub struct Events {
    rx: mpsc::Receiver<Event>,
    #[allow(dead_code)]
    input_handle: thread::JoinHandle<()>,
    #[allow(dead_code)]
    tick_handle: thread::JoinHandle<()>,
}

impl Events {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        let input_handle = {
            let tx = tx.clone();

            thread::spawn(move || {
                let stdin = io::stdin();
                for event in stdin.keys() {
                    if let Ok(key) = event {
                        if tx.send(Event::Input(key)).is_err() {
                            return;
                        }
                    }
                }
            })
        };

        let tick_handle = {
            thread::spawn(move || loop {
                if tx.send(Event::Tick(TICK_RATE)).is_err() {
                    return;
                }
                thread::sleep(Duration::from_millis(TICK_RATE));
            })
        };

        Events {
            rx,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.rx.recv()
    }
}
