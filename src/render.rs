// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error::Error;
use crate::event::{Event, Events};
use std::io::{self, Write};
use std::iter::Cycle;
use std::process;
use std::slice::Iter;
use std::sync::mpsc::{Receiver, TryRecvError::Disconnected};
use std::thread::{self, JoinHandle};
use std::time::Instant;
use termion::clear::AfterCursor;
use termion::color::{Fg, Reset as ColorReset, Yellow};
use termion::cursor::{DetectCursorPos, Goto, Hide, Restore, Save, Show};
use termion::raw::IntoRawMode;
use termion::style::{Italic, Reset};

const SPINNER_RATE: u128 = 40;

pub struct Render(Option<JoinHandle<Result<(), Error>>>);

impl Render {
    pub fn new() -> Render {
        Render(None)
    }

    pub fn run(&mut self, rx: Receiver<&'static str>) {
        let handle = thread::spawn(move || -> Result<(), Error> {
            draw(rx)?;
            Ok(())
        });
        self.0 = Some(handle);
    }

    pub fn finish(self) -> Result<(), Error> {
        if let Some(handle) = self.0 {
            handle.join().unwrap()
        } else {
            Ok(())
        }
    }
}

impl Default for Render {
    fn default() -> Self {
        Render::new()
    }
}

struct Spinner<'a> {
    time: Instant,
    step: &'a str,
    steps: &'a [String; 14],
    steps_it: Cycle<Iter<'a, String>>,
}

impl<'a> Spinner<'a> {
    fn new(steps: &'a [String; 14]) -> Self {
        Spinner {
            time: Instant::now(),
            step: &steps[0],
            steps,
            steps_it: steps.iter().cycle(),
        }
    }

    fn reset(&mut self) {
        self.steps_it = self.steps.iter().cycle();
        self.time = Instant::now();
        self.step = &self.steps[0];
    }
}

impl<'a> Iterator for Spinner<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.time.elapsed().as_millis() > SPINNER_RATE {
            self.time = Instant::now();
            if let Some(value) = self.steps_it.next() {
                self.step = value;
                Some(value)
            } else {
                None
            }
        } else {
            Some(self.step)
        }
    }
}

fn draw(rx: Receiver<&'static str>) -> Result<(), Error> {
    let mut stdout = io::stdout().into_raw_mode()?;
    let (init_a, init_b) = stdout.cursor_pos()?;
    let events = Events::new();
    write!(stdout, "{}{}", Save, Hide)?;
    let mut tmp_state = "";
    let steps = [
        " ----".to_string(),
        format!("{}c{}----", Fg(Yellow), Fg(ColorReset)),
        format!("{}C{}----", Fg(Yellow), Fg(ColorReset)),
        format!(" {}C{}---", Fg(Yellow), Fg(ColorReset)),
        format!(" {}c{}---", Fg(Yellow), Fg(ColorReset)),
        format!(" {}C{}---", Fg(Yellow), Fg(ColorReset)),
        format!("  {}C{}--", Fg(Yellow), Fg(ColorReset)),
        format!("  {}c{}--", Fg(Yellow), Fg(ColorReset)),
        format!("  {}C{}--", Fg(Yellow), Fg(ColorReset)),
        format!("   {}C{}-", Fg(Yellow), Fg(ColorReset)),
        format!("   {}c{}-", Fg(Yellow), Fg(ColorReset)),
        format!("   {}C{}-", Fg(Yellow), Fg(ColorReset)),
        format!("    {}C{}", Fg(Yellow), Fg(ColorReset)),
        format!("    {}c{}", Fg(Yellow), Fg(ColorReset)),
    ];
    let mut spinner = Spinner::new(&steps);
    loop {
        match rx.try_recv() {
            Ok(state) => {
                tmp_state = state;
                spinner.reset();
            }
            Err(err) => {
                if let Disconnected = err {
                    break;
                }
            }
        }
        write!(stdout, "{}{}", Goto(init_a, init_b), AfterCursor)?;
        if let Some(value) = spinner.next() {
            write!(stdout, "{}  ", value)?;
        }
        write!(stdout, "{}{}{}", Italic, tmp_state, Reset)?;
        stdout.flush()?;
        if let Event::Input(key) = events.next()? {
            if Events::is_exit_key(key) {
                write!(stdout, "{}{}{}", Show, Restore, AfterCursor)?;
                stdout.flush()?;
                drop(stdout);
                process::exit(1);
            }
        }
    }
    // events.finish()?;
    write!(stdout, "{}{}{}", Restore, AfterCursor, Show)?;
    stdout.flush()?;
    Ok(())
}
