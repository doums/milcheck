// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod color;
mod error;
mod event;
mod http;
mod render;
use color::Palette;
use error::Error;
use http::Http;
use render::Render;
use serde::{Deserialize, Serialize};
use std::cmp;
use std::error::Error as StdError;
use std::fs;
use std::io;
use std::str;
use std::sync::mpsc;
use std::thread;
// use std::time::Duration;
use std::env;
use termion::color::{AnsiValue, Color, Fg, Reset as ColorReset, Rgb};
use termion::style::{Bold, Reset};

const PACMAN_MIRRORLIST: &'static str = "/etc/pacman.d/mirrorlist";
const MIRROR_STATUS_URI: &'static str = "https://www.archlinux.org/mirrors/status/";
const MIRROR_STATUS_JSON_URI: &'static str = "https://www.archlinux.org/mirrors/status/json/";
const OUTOFSYNC_HTML_TAG: &'static str = "<table id=\"outofsync_mirrors\"";
const INSYNC_HTML_TAG: &'static str = "<table id=\"successful_mirrors\"";
const OK: &'static str = "Ok";
const NOT_FOUND: &'static str = "Not found!";
const OUT_OF_SYNC: &'static str = "Out of sync!";
const HEADERS: [&'static str; 9] = [
    "State",
    "Url",
    "Protocol",
    "Country",
    "Completion %",
    "Delay h:m",
    "Avg dur s",
    "Dev dur s",
    "Score",
];

#[derive(Debug, Clone)]
pub enum MirrorState {
    NotFound(String),
    Synced(Mirror),
    OutOfSync(Mirror),
}

#[derive(Debug, Clone)]
pub struct Mirror {
    url: String,
    protocol: String,
    country: String,
    completion: Option<f64>,
    delay: Option<(u32, u32)>,
    duration_avg: Option<f64>,
    duration_stddev: Option<f64>,
    score: Option<f64>,
}

impl Mirror {
    fn completion_to_str(&self) -> String {
        if let Some(value) = self.completion {
            if value != 100f64 {
                format!("{:.1}", value)
            } else {
                format!("{:.0}", value)
            }
        } else {
            "".to_string()
        }
    }

    fn delay_to_str(&self) -> String {
        if let Some(value) = self.delay {
            let (hour, minute) = value;
            format!("{:}:{:>02}", hour, minute)
        } else {
            "".to_string()
        }
    }

    fn duration_avg_to_str(&self) -> String {
        if let Some(value) = self.duration_avg {
            format!("{:.2}", value)
        } else {
            "".to_string()
        }
    }

    fn duration_stddev_to_str(&self) -> String {
        if let Some(value) = self.duration_stddev {
            format!("{:.2}", value)
        } else {
            "".to_string()
        }
    }

    fn score_to_str(&self) -> String {
        if let Some(value) = self.score {
            format!("{:.1}", value)
        } else {
            "".to_string()
        }
    }

    fn get_len(&self, field: &'static str) -> Result<usize, Error> {
        match field {
            "url" => Ok(self.url.len()),
            "protocol" => Ok(self.protocol.len()),
            "country" => Ok(self.country.len()),
            "completion" => Ok(self.completion_to_str().len()),
            "delay" => Ok(self.delay_to_str().len()),
            "duration_avg" => Ok(self.duration_avg_to_str().len()),
            "duration_stddev" => Ok(self.duration_stddev_to_str().len()),
            "score" => Ok(self.score_to_str().len()),
            _ => Err(Error::new(format!(
                "Mirror does not have a field \"{}\"",
                field
            ))),
        }
    }
}

impl From<&JsonMirror> for Mirror {
    fn from(json: &JsonMirror) -> Self {
        let completion = if let Some(completion) = json.completion_pct {
            Some(completion * 100f64)
        } else {
            None
        };
        let delay = if let Some(delay) = json.delay {
            let hours = delay as f64 / 3600_f64;
            let normalized_hours = hours.trunc() as u32;
            let minutes = (hours.fract() * 60_f64).trunc() as u32;
            Some((normalized_hours, minutes))
        } else {
            None
        };
        Mirror {
            url: String::from(&json.url),
            protocol: String::from(&json.protocol),
            country: String::from(&json.country),
            completion,
            delay,
            duration_avg: json.duration_avg,
            duration_stddev: json.duration_stddev,
            score: json.score,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonResponse {
    cutoff: u32,
    last_check: String,
    num_checks: u32,
    check_frequency: u32,
    urls: Vec<JsonMirror>,
    version: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonMirror {
    url: String,
    protocol: String,
    country: String,
    country_code: String,
    completion_pct: Option<f64>,
    delay: Option<u32>,
    duration_avg: Option<f64>,
    duration_stddev: Option<f64>,
    score: Option<f64>,
    last_sync: Option<String>,
    active: bool,
    isos: bool,
    ipv4: bool,
    ipv6: bool,
    details: String,
}

struct MaxLength {
    state: usize,
    url: usize,
    protocol: usize,
    country: usize,
    completion: usize,
    delay: usize,
    duration_avg: usize,
    duration_stddev: usize,
    score: usize,
}

impl MaxLength {
    fn new(mirrors: &Vec<MirrorState>) -> Result<Self, Error> {
        let state = cmp::max(find_max_state_len(mirrors), HEADERS[0].len());
        let url = cmp::max(find_max_len(mirrors, "url")?, HEADERS[1].len());
        let protocol = cmp::max(find_max_len(mirrors, "protocol")?, HEADERS[2].len());
        let country = cmp::max(find_max_len(mirrors, "country")?, HEADERS[3].len());
        let completion = cmp::max(find_max_len(mirrors, "completion")?, HEADERS[4].len());
        let delay = cmp::max(find_max_len(mirrors, "delay")?, HEADERS[5].len());
        let duration_avg = cmp::max(find_max_len(mirrors, "duration_avg")?, HEADERS[6].len());
        let duration_stddev = cmp::max(find_max_len(mirrors, "duration_stddev")?, HEADERS[7].len());
        let score = cmp::max(find_max_len(mirrors, "score")?, HEADERS[8].len());
        Ok(MaxLength {
            state,
            url,
            protocol,
            country,
            completion,
            delay,
            duration_avg,
            duration_stddev,
            score,
        })
    }
}

fn find_max_state_len(mirrors: &Vec<MirrorState>) -> usize {
    let mut max_len = 0;
    for mirror_state in mirrors {
        match mirror_state {
            MirrorState::NotFound(_) => {
                if NOT_FOUND.len() > max_len {
                    max_len = NOT_FOUND.len();
                }
            }
            MirrorState::OutOfSync(_) => {
                if OUT_OF_SYNC.len() > max_len {
                    max_len = OUT_OF_SYNC.len();
                }
            }
            MirrorState::Synced(_) => {
                if OK.len() > max_len {
                    max_len = OK.len();
                }
            }
        }
    }
    max_len
}

fn find_max_len(mirrors: &Vec<MirrorState>, key: &'static str) -> Result<usize, Error> {
    let mut max_len = 0;
    for mirror_state in mirrors {
        match mirror_state {
            MirrorState::NotFound(server) => {
                if key == "url" && server.len() > max_len {
                    max_len = server.len();
                }
            }
            MirrorState::OutOfSync(mirror) => {
                let value = mirror.get_len(key)?;
                if value > max_len {
                    max_len = value;
                }
            }
            MirrorState::Synced(mirror) => {
                let value = mirror.get_len(key)?;
                if value > max_len {
                    max_len = value;
                }
            }
        }
    }
    Ok(max_len)
}

fn print_headers(max_len: &MaxLength) -> Result<(), io::Error> {
    println!(
        "{}{} {} {} {} {} {} {} {} {}{}",
        Bold,
        format!("{:>width$}", HEADERS[0], width = max_len.state),
        format!("{:<width$}", HEADERS[1], width = max_len.url),
        format!("{:<width$}", HEADERS[2], width = max_len.protocol),
        format!("{:<width$}", HEADERS[3], width = max_len.country),
        format!("{:>width$}", HEADERS[4], width = max_len.completion),
        format!("{:>width$}", HEADERS[5], width = max_len.delay),
        format!("{:>width$}", HEADERS[6], width = max_len.duration_avg),
        format!("{:>width$}", HEADERS[7], width = max_len.duration_stddev),
        format!("{:>width$}", HEADERS[8], width = max_len.score),
        Reset
    );
    Ok(())
}

fn print_mirror<C: Color + Copy>(
    max_len: &MaxLength,
    mirror: &Mirror,
    state: &'static str,
    color: C,
    colors: &Palette<C>,
) {
    let completion_color = if let Some(value) = mirror.completion {
        if value < 100f64 && value >= 95f64 {
            format!("{}", Fg(colors.orange))
        } else if value < 95f64 {
            format!("{}", Fg(colors.red))
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };
    let delay_color = if let Some(value) = mirror.delay {
        let (hours, minutes) = value;
        if hours > 1 {
            format!("{}", Fg(colors.red))
        } else if minutes > 30 {
            format!("{}", Fg(colors.orange))
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };
    let score_color = if let Some(value) = mirror.score {
        if value > 2f64 {
            format!("{}", Fg(colors.red))
        } else if value > 1f64 {
            format!("{}", Fg(colors.orange))
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };
    println!(
        "{} {} {} {} {} {} {} {} {}",
        format!(
            "{}{}{:>width$}{}",
            Bold,
            Fg(color),
            state,
            Reset,
            width = max_len.state
        ),
        format!("{:<width$}", mirror.url, width = max_len.url),
        format!("{:<width$}", mirror.protocol, width = max_len.protocol),
        format!("{:<width$}", mirror.country, width = max_len.country),
        format!(
            "{}{:>width$}{}",
            completion_color,
            mirror.completion_to_str(),
            Fg(ColorReset),
            width = max_len.completion,
        ),
        format!(
            "{}{:>width$}{}",
            delay_color,
            mirror.delay_to_str(),
            Fg(ColorReset),
            width = max_len.delay
        ),
        format!(
            "{:>width$}",
            mirror.duration_avg_to_str(),
            width = max_len.duration_avg
        ),
        format!(
            "{:>width$}",
            mirror.duration_stddev_to_str(),
            width = max_len.duration_stddev
        ),
        format!(
            "{}{:>width$}{}",
            score_color,
            mirror.score_to_str(),
            Fg(ColorReset),
            width = max_len.score
        ),
    );
}

fn print_mirrors<C: Color + Copy>(
    mirrors: Vec<MirrorState>,
    colors: Palette<C>,
) -> Result<(), Error> {
    let max_lengths = MaxLength::new(&mirrors)?;
    print_headers(&max_lengths)?;
    for mirror_state in &mirrors {
        match mirror_state {
            MirrorState::NotFound(server) => {
                println!(
                    "{}{}{}{} {}",
                    Bold,
                    Fg(colors.orange),
                    format!("{:>width$}", NOT_FOUND, width = max_lengths.state),
                    Reset,
                    server
                );
            }
            MirrorState::OutOfSync(mirror) => {
                print_mirror(&max_lengths, &mirror, OUT_OF_SYNC, colors.red, &colors);
            }
            MirrorState::Synced(mirror) => {
                print_mirror(&max_lengths, &mirror, OK, colors.green, &colors);
            }
        }
    }
    println!();
    Ok(())
}

fn parse_mirrorlist() -> Result<Vec<String>, Box<dyn StdError>> {
    let mut mirrors = vec![];
    let mirrorlist = fs::read_to_string(PACMAN_MIRRORLIST).map_err(|err| {
        format!(
            "an error occured while reading the file {}: {}",
            PACMAN_MIRRORLIST, err
        )
    })?;
    for line in mirrorlist.lines() {
        if line.starts_with("Server = ") {
            if line.ends_with("/$repo/os/$arch") {
                let end = line.len() - 14;
                mirrors.push(String::from(&line[9..end]));
            } else if line.ends_with("/$repo/os/$arch/") {
                let end = line.len() - 15;
                mirrors.push(String::from(&line[9..end]));
            } else {
                mirrors.push(String::from(&line[9..]));
            }
        }
    }
    if mirrors.len() == 0 {
        Err(Box::new(Error::new(format!(
            "no server found in {}",
            PACMAN_MIRRORLIST
        ))))
    } else {
        Ok(mirrors)
    }
}

fn supports_truecolor() -> bool {
    if let Some(value) = env::var_os("COLORTERM") {
        if value == "truecolor" {
            return true;
        }
    }
    false
}

pub fn run() -> Result<(), Box<dyn StdError>> {
    let mut mirrors = vec![];
    let (tx, rx) = mpsc::channel();
    let (tx_m, rx_m) = mpsc::channel();
    let mut handles = vec![];
    let mut render = Render::new();
    render.run(rx)?;
    tx.send("parsing local mirrorlist")?;
    let mirrorlist = parse_mirrorlist()?;
    tx.send("fetching mirror status list")?;
    // thread::sleep(Duration::from_secs(2));
    let request = Http::fetch(MIRROR_STATUS_URI);
    let json_request = Http::fetch(MIRROR_STATUS_JSON_URI);
    let response = request.wait()?;
    let json_response = json_request.wait()?;
    tx.send("deserialize json data")?;
    // thread::sleep(Duration::from_secs(5));
    let json: JsonResponse = serde_json::from_str(&json_response)
        .map_err(|err| format!("json response parsing failed: {}", err))?;
    tx.send("web scraping")?;
    // thread::sleep(Duration::from_secs(5));
    let v: Vec<&str> = response.split("</table>").collect();
    if v.len() != 4 {
        return Err(Box::new(Error::new("web scraping failed")));
    }
    if let None = v[0].find(OUTOFSYNC_HTML_TAG) {
        return Err(Box::new(Error::new("web scraping failed")));
    }
    if let None = v[1].find(INSYNC_HTML_TAG) {
        return Err(Box::new(Error::new("web scraping failed")));
    }
    tx.send("build data for rendering")?;
    for server in mirrorlist {
        let out_of_sync = String::from(v[0]);
        let mirrors_json = json.urls.clone();
        let tx_cloned = mpsc::Sender::clone(&tx_m);
        let handle = thread::spawn(move || -> Result<(), Error> {
            if let Some(mirror) = mirrors_json.iter().find(|&mirror| mirror.url == server) {
                if let Some(_i) = out_of_sync.find(&server) {
                    tx_cloned.send(MirrorState::OutOfSync(Mirror::from(mirror)))?;
                } else {
                    tx_cloned.send(MirrorState::Synced(Mirror::from(mirror)))?;
                }
            } else {
                tx_cloned.send(MirrorState::NotFound(server))?;
            }
            Ok(())
        });
        handles.push(handle);
    }
    tx.send("done")?;
    drop(tx_m);
    for received in rx_m {
        mirrors.push(received);
    }
    for handle in handles {
        handle.join().unwrap()?;
    }
    drop(tx);
    render.finish()?;
    if supports_truecolor() {
        print_mirrors(mirrors, Palette::<Rgb>::new())?;
    } else {
        print_mirrors(mirrors, Palette::<AnsiValue>::new())?;
    }
    Ok(())
}
