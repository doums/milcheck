// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::{Context, Result};
use log::debug;
use reqwest::blocking;
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};

pub struct Http(JoinHandle<Result<()>>, Receiver<String>);

impl Http {
    pub fn get(url: &str) -> Http {
        let (tx, rx) = mpsc::channel();
        let url_cloned = String::from(url);
        let handle = thread::spawn(move || -> Result<()> {
            debug!("fetching [{}]", url_cloned);
            let content = blocking::get(&url_cloned)
                .context("http get failed")?
                .text()
                .context("http get: failed to decode response")?;
            Ok(tx.send(content)?)
        });
        Http(handle, rx)
    }

    pub fn wait(self) -> Result<String> {
        let res = self.1.recv().context("http get: failed to receive")?;
        self.0.join().unwrap().context("http get: failed to join")?;
        Ok(res)
    }
}
