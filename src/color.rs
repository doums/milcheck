// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use termion::color::{AnsiValue, Color, Rgb};

pub struct Palette<C: Color> {
    pub green: C,
    pub red: C,
    pub orange: C,
}

impl Palette<Rgb> {
    pub fn new() -> Palette<Rgb> {
        Palette {
            green: Rgb(129, 199, 132),
            red: Rgb(229, 115, 115),
            orange: Rgb(255, 183, 77),
        }
    }
}

impl Palette<AnsiValue> {
    pub fn new() -> Palette<AnsiValue> {
        Palette {
            green: AnsiValue(2),
            red: AnsiValue(1),
            orange: AnsiValue(208),
        }
    }
}
