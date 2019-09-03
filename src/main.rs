// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use milcheck::run;
use std::io;
use std::process;

fn main() -> Result<(), io::Error> {
    run().unwrap_or_else(|err| {
        eprintln!("error: {}", err);
        process::exit(1);
    });
    Ok(())
}
