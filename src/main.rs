// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::Parser;
use milcheck::cli::Cli;
use milcheck::Milcheck;
use std::process;

fn main() {
    let cli = Cli::parse();
    let mut milcheck = Milcheck::from(cli);
    milcheck.run().unwrap_or_else(|err| {
        eprintln!("error: {}", err);
        process::exit(1);
    });
}
