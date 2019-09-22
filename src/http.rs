// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error::Error;
use std::process::Command;
use std::str;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

pub struct Http(JoinHandle<Result<(), Error>>, Receiver<String>);

impl Http {
    pub fn fetch(url: &str) -> Http {
        let (tx, rx) = mpsc::channel();
        let url_cloned = String::from(url);
        let handle = thread::spawn(move || -> Result<(), Error> {
            curl(url_cloned, tx)?;
            Ok(())
        });
        Http(handle, rx)
    }

    pub fn wait(self) -> Result<String, Error> {
        let mut response: String = "".to_string();
        if let Ok(msg) = self.1.recv() {
            response = String::from(msg);
        }
        self.0.join().unwrap()?;
        Ok(response)
    }
}

fn curl(url: String, tx: Sender<String>) -> Result<(), Error> {
    let output = Command::new("curl")
        .arg(&url)
        .output()
        .map_err(|_err| "this binary needs curl to work, make sure that curl is installed")?;
    if !output.status.success() {
        let curl_stderr = str::from_utf8(&output.stderr)?;
        Err(Error::new(format!(
            "curl failed to get response from {}: {}",
            url, curl_stderr
        )))
    } else {
        let response = String::from_utf8(output.stdout)?;
        Ok(tx.send(response)?)
    }
}
