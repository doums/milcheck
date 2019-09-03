// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error::Error;
use std::io;
use std::sync::mpsc::{self, Receiver, RecvError, TryRecvError::Disconnected};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use termion::event::Key;
use termion::input::TermRead;

const TICK_RATE: Duration = Duration::from_millis(16);
const EXIT_KEYS: [Key; 3] = [Key::Char('q'), Key::Esc, Key::Ctrl('c')];

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: Receiver<Event<Key>>,
    input_handle: JoinHandle<Result<(), Error>>,
    tick_handle: JoinHandle<Result<(), Error>>,
}

impl Events {
    pub fn is_exit_key(key: Key) -> bool {
        EXIT_KEYS.iter().any(|&k| k == key)
    }

    pub fn new() -> Events {
        let (tx, rx) = mpsc::channel();
        let (tick_tx, tick_rx) = mpsc::channel();
        let input_handle = {
            let tx = tx.clone();
            let tick_tx = tick_tx.clone();
            thread::spawn(move || -> Result<(), Error> {
                let stdin = io::stdin();
                for input in stdin.keys() {
                    let key = input?;
                    tx.send(Event::Input(key))?;
                    if EXIT_KEYS.iter().any(|&k| k == key) {
                        tick_tx.send(())?;
                        return Ok(());
                    }
                }
                Ok(())
            })
        };
        let tick_handle = {
            let tx = tx.clone();
            thread::spawn(move || -> Result<(), Error> {
                loop {
                    tx.send(Event::Tick)?;
                    match tick_rx.try_recv() {
                        Ok(()) => return Ok(()),
                        Err(err) => {
                            if let Disconnected = err {
                                return Ok(());
                            }
                        }
                    }
                    thread::sleep(TICK_RATE);
                }
            })
        };
        Events {
            rx: rx,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, RecvError> {
        self.rx.recv()
    }

    pub fn finish(self) -> Result<(), Error> {
        self.input_handle.join().unwrap()?;
        self.tick_handle.join().unwrap()?;
        Ok(())
    }
}
