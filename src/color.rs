// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::env;
use std::fmt::{Formatter, Result};
use termion::color::{AnsiValue, Color as TColor, Rgb};

#[derive(Debug)]
pub struct Color(Box<dyn TColor>);

impl Color {
    fn new<C: TColor + 'static>(color: C) -> Self {
        Color(Box::new(color))
    }
}

impl TColor for Color {
    fn write_fg(&self, f: &mut Formatter) -> Result {
        self.0.write_fg(f)
    }

    fn write_bg(&self, f: &mut Formatter) -> Result {
        self.0.write_bg(f)
    }
}

impl TColor for &Color {
    fn write_fg(&self, f: &mut Formatter) -> Result {
        self.0.write_fg(f)
    }

    fn write_bg(&self, f: &mut Formatter) -> Result {
        self.0.write_bg(f)
    }
}

pub struct Palette {
    pub green: Color,
    pub red: Color,
    pub orange: Color,
}

impl Palette {
    pub fn new() -> Palette {
        if let Some(value) = env::var_os("COLORTERM") {
            if value == "truecolor" {
                return Palette {
                    green: Color::new(Rgb(129, 199, 132)),
                    red: Color::new(Rgb(229, 115, 115)),
                    orange: Color::new(Rgb(255, 183, 77)),
                };
            }
        }
        Palette {
            green: Color::new(AnsiValue(2)),
            red: Color::new(AnsiValue(1)),
            orange: Color::new(AnsiValue(208)),
        }
    }
}
