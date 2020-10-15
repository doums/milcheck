// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error::Error;
use reqwest::blocking;
use std::str;
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};

pub struct Http(JoinHandle<Result<(), Error>>, Receiver<String>);

impl Http {
    pub fn get(url: &str) -> Http {
        let (tx, rx) = mpsc::channel();
        let url_cloned = String::from(url);
        let handle = thread::spawn(move || -> Result<(), Error> {
            let content = blocking::get(&url_cloned)?.text()?;
            Ok(tx.send(content)?)
        });
        Http(handle, rx)
    }

    pub fn wait(self) -> Result<String, Error> {
        let mut response: String = "".to_string();
        if let Ok(msg) = self.1.recv() {
            response = msg;
        }
        self.0.join().unwrap()?;
        Ok(response)
    }
}
