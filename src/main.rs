// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::Parser;
use milcheck::Milcheck;
use std::convert::TryFrom;
use std::process;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Print the latest news after mirrors status
    #[arg(short, long)]
    news: Option<u8>,
}

fn main() {
    let cli = Cli::parse();
    let mut milcheck = Milcheck::try_from(cli.news).unwrap_or_else(|err| {
        eprintln!("{}, run milcheck --help", err);
        process::exit(1);
    });
    milcheck.run().unwrap_or_else(|err| {
        eprintln!("error: {}", err);
        process::exit(1);
    });
}
