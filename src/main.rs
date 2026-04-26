// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::Result;
use clap::Parser;
use milcheck::Milcheck;
use milcheck::cli::Cli;

fn main() -> Result<()> {
    env_logger::builder().format_timestamp(None).init();

    let cli = Cli::parse();

    let mut milcheck = Milcheck::from(cli);
    milcheck.run().inspect_err(|e| {
        log::error!("{}", e);
    })?;
    Ok(())
}
