// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod cli;
mod event;
mod http;
mod news;
mod render;

use anyhow::{Result, anyhow, bail};
use cli::Cli;
use http::Http;
use render::Render;
use serde::{Deserialize, Serialize};
use std::cmp;
use std::fs;
use std::str;
use std::sync::mpsc::{self, Receiver, Sender};
use termion::color::{Color, Fg, Green, Red, Reset as ColorReset, Yellow};
use termion::style::{Bold, Reset};

const PACMAN_MIRRORLIST: &str = "/etc/pacman.d/mirrorlist";
const MIRROR_STATUS_URL: &str = "https://www.archlinux.org/mirrors/status/";
const MIRROR_STATUS_JSON_URL: &str = "https://www.archlinux.org/mirrors/status/json/";
pub const ARCHLINUX_ORG_URL: &str = "https://archlinux.org";
const OUTOFSYNC_HTML_TAG: &str = "<table id=\"outofsync_mirrors\"";
const INSYNC_HTML_TAG: &str = "<table id=\"successful_mirrors\"";
const OK: &str = "Ok";
const NOT_FOUND: &str = "Not found!";
const OUT_OF_SYNC: &str = "Out of sync!";
const HEADERS: [&str; 9] = [
    "State", "Url", "Proto", // Protocol
    "Country", "Comp%", // Completion
    "Delay", // Delay (hh:mm)
    "Avg",   // Average time (s)
    "Dev",   // The standard deviation time (s)
    "Score",
];

#[derive(Debug, Clone)]
pub struct Milcheck {
    print_mirrorlist: bool,
    print_news: bool,
    last: Option<u8>,
}

impl From<Cli> for Milcheck {
    fn from(cli: Cli) -> Self {
        let mut print_mirrorlist = false;
        let mut print_news = false;
        let mut last: Option<u8> = None;

        // if `-m` flag is passed without value, it's considered as true
        if let Some(None) = cli.mirrorlist {
            print_mirrorlist = true;
        }
        // if its value has been set by the user, use it
        if let Some(Some(v)) = cli.mirrorlist {
            print_mirrorlist = v;
        }

        // by default, without any flags, print mirrorlist status
        if cli.mirrorlist.is_none() && cli.news.is_none() {
            return Milcheck {
                print_mirrorlist: true,
                print_news: false,
                last: None,
            };
        }

        if let Some(n) = cli.news {
            print_news = true;
            last = Some(n);
        }

        Milcheck {
            print_mirrorlist,
            print_news,
            last,
        }
    }
}

impl Milcheck {
    pub fn run(&mut self) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        let mut render = Render::new();
        let tx_cloned = Sender::clone(&tx);
        let data = logic(
            tx_cloned,
            rx,
            &mut render,
            self.print_mirrorlist,
            self.print_news,
            self.last,
        );

        drop(tx);
        render.finish();

        let (mirrors, news) = data?;
        if let Some(m) = mirrors {
            print_mirrors(m)?;
        }
        if let Some(n) = news {
            println!("{}", n);
        }
        Ok(())
    }
}

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
            if (value - 100f64).abs() > f64::EPSILON {
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

    fn get_len(&self, field: &'static str) -> Result<usize> {
        match field {
            "url" => Ok(self.url.len()),
            "protocol" => Ok(self.protocol.len()),
            "country" => Ok(self.country.len()),
            "completion" => Ok(self.completion_to_str().len()),
            "delay" => Ok(self.delay_to_str().len()),
            "duration_avg" => Ok(self.duration_avg_to_str().len()),
            "duration_stddev" => Ok(self.duration_stddev_to_str().len()),
            "score" => Ok(self.score_to_str().len()),
            _ => bail!("mirror missing field \"{}\"", field),
        }
    }
}

impl From<&JsonMirror> for Mirror {
    fn from(json: &JsonMirror) -> Self {
        let completion = json.completion_pct.map(|completion| completion * 100f64);
        let mut delay: Option<(u32, u32)> = None;
        if let Some(d) = json.delay {
            if d < 0 {
                delay = None;
            } else {
                let hours = d as f64 / 3600_f64;
                let normalized_hours = hours.trunc() as u32;
                let minutes = (hours.fract() * 60_f64).trunc() as u32;
                delay = Some((normalized_hours, minutes));
            }
        }
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
    delay: Option<i32>,
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
    fn new(mirrors: &[MirrorState]) -> Result<Self> {
        let state = cmp::max(find_max_state_len(mirrors), HEADERS[0].len());
        let url = cmp::max(find_max_len(mirrors, "url")?, HEADERS[1].len());
        let protocol = cmp::max(find_max_len(mirrors, "protocol")?, HEADERS[2].len());
        let country = cmp::max(find_max_len(mirrors, "country")?, HEADERS[3].len());
        let completion = cmp::max(find_max_len(mirrors, "completion")?, HEADERS[4].len());
        let delay = cmp::max(find_max_len(mirrors, "delay")?, HEADERS[5].len());
        let duration_avg = cmp::max(find_max_len(mirrors, "duration_avg")?, HEADERS[6].len()) + 1;
        let duration_stddev =
            cmp::max(find_max_len(mirrors, "duration_stddev")?, HEADERS[7].len()) + 1;
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

fn find_max_state_len(mirrors: &[MirrorState]) -> usize {
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

fn find_max_len(mirrors: &[MirrorState], key: &'static str) -> Result<usize> {
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

fn print_headers(max_len: &MaxLength) {
    println!(
        "{}{:>state$} {:<url$} {:<protocol$} {:<country$} {:>completion$} {:>delay$} {:>duration_avg$} {:>duration_stddev$} {:>score$}{}",
        Bold,
        HEADERS[0],
        HEADERS[1],
        HEADERS[2],
        HEADERS[3],
        HEADERS[4],
        HEADERS[5],
        HEADERS[6],
        HEADERS[7],
        HEADERS[8],
        Reset,
        state = max_len.state,
        url = max_len.url,
        protocol = max_len.protocol,
        country = max_len.country,
        completion = max_len.completion,
        delay = max_len.delay,
        duration_avg = max_len.duration_avg,
        duration_stddev = max_len.duration_stddev,
        score = max_len.score,
    );
}

fn print_mirror<C: Color + Copy>(
    max_len: &MaxLength,
    mirror: &Mirror,
    state: &'static str,
    color: C,
) {
    let completion_color = if let Some(value) = mirror.completion {
        if (95f64..100f64).contains(&value) {
            format!("{}", Fg(Yellow))
        } else if value < 95f64 {
            format!("{}", Fg(Red))
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };
    let delay_color = if let Some(value) = mirror.delay {
        let (hours, minutes) = value;
        if hours > 0 {
            format!("{}", Fg(Red))
        } else if minutes > 30 {
            format!("{}", Fg(Yellow))
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };
    let score_color = if let Some(value) = mirror.score {
        if value > 2f64 {
            format!("{}", Fg(Red))
        } else if value > 1f64 {
            format!("{}", Fg(Yellow))
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };
    println!(
        "{} {} {} {} {} {} {} {} {}",
        format_args!(
            "{}{}{:>width$}{}",
            Bold,
            Fg(color),
            state,
            Reset,
            width = max_len.state
        ),
        format_args!("{:<width$}", mirror.url, width = max_len.url),
        format_args!("{:<width$}", mirror.protocol, width = max_len.protocol),
        format_args!("{:<width$}", mirror.country, width = max_len.country),
        format_args!(
            "{}{:>width$}{}",
            completion_color,
            mirror.completion_to_str(),
            Fg(ColorReset),
            width = max_len.completion,
        ),
        format_args!(
            "{}{:>width$}{}",
            delay_color,
            mirror.delay_to_str(),
            Fg(ColorReset),
            width = max_len.delay
        ),
        format_args!(
            "{:>width$}",
            mirror.duration_avg_to_str(),
            width = max_len.duration_avg
        ),
        format_args!(
            "{:>width$}",
            mirror.duration_stddev_to_str(),
            width = max_len.duration_stddev
        ),
        format_args!(
            "{}{:>width$}{}",
            score_color,
            mirror.score_to_str(),
            Fg(ColorReset),
            width = max_len.score
        ),
    );
}

fn print_mirrors(mirrors: Vec<MirrorState>) -> Result<()> {
    let max_lengths = MaxLength::new(&mirrors)?;
    print_headers(&max_lengths);
    for mirror_state in &mirrors {
        match mirror_state {
            MirrorState::NotFound(server) => {
                println!(
                    "{}{}{:>state$}{} {}",
                    Bold,
                    Fg(Yellow),
                    NOT_FOUND,
                    Reset,
                    server,
                    state = max_lengths.state,
                );
            }
            MirrorState::OutOfSync(mirror) => {
                print_mirror(&max_lengths, mirror, OUT_OF_SYNC, Red);
            }
            MirrorState::Synced(mirror) => {
                print_mirror(&max_lengths, mirror, OK, Green);
            }
        }
    }
    println!();
    Ok(())
}

fn parse_mirrorlist() -> Result<Vec<String>> {
    let mut mirrors = vec![];
    let mirrorlist = fs::read_to_string(PACMAN_MIRRORLIST)
        .map_err(|err| anyhow!("failed to read {}: {}", PACMAN_MIRRORLIST, err))?;
    for line in mirrorlist.lines() {
        if let Some(url) = line.strip_prefix("Server = ") {
            if line.ends_with("/$repo/os/$arch") {
                let end = url.len() - 14;
                mirrors.push(String::from(&url[..end]));
            } else if line.ends_with("/$repo/os/$arch/") {
                let end = url.len() - 15;
                mirrors.push(String::from(&url[..end]));
            } else {
                mirrors.push(String::from(url));
            }
        }
    }
    if mirrors.is_empty() {
        bail!("no server found in {}", PACMAN_MIRRORLIST)
    } else {
        Ok(mirrors)
    }
}

pub fn logic(
    tx: Sender<&'static str>,
    rx: Receiver<&'static str>,
    render: &mut Render,
    print_mirrorlist: bool,
    print_news: bool,
    last: Option<u8>,
) -> Result<(Option<Vec<MirrorState>>, Option<String>)> {
    let mut mirrors = None;
    render.run(rx);
    if print_mirrorlist {
        let mut parsed = vec![];
        tx.send("parsing local mirrorlist")?;
        let mirrorlist = parse_mirrorlist()?;
        tx.send("fetching mirror status list")?;
        let request = Http::get(MIRROR_STATUS_URL);
        let json_request = Http::get(MIRROR_STATUS_JSON_URL);
        let response = request.wait()?;
        let json_response = json_request.wait()?;
        tx.send("deserialize json data")?;
        let json: JsonResponse = serde_json::from_str(&json_response)
            .map_err(|err| anyhow!("json response parsing failed: {}", err))?;
        tx.send("web scraping")?;
        let v: Vec<&str> = response.split("</table>").collect();
        if v.len() != 4 || !v[0].contains(OUTOFSYNC_HTML_TAG) || !v[1].contains(INSYNC_HTML_TAG) {
            bail!("mirror status scraping failed");
        }
        tx.send("building data")?;
        for server in mirrorlist {
            if let Some(mirror) = json.urls.iter().find(|&mirror| mirror.url == server) {
                if let Some(_i) = v[0].find(&server) {
                    parsed.push(MirrorState::OutOfSync(Mirror::from(mirror)));
                } else {
                    parsed.push(MirrorState::Synced(Mirror::from(mirror)));
                }
            } else {
                parsed.push(MirrorState::NotFound(server));
            }
        }
        mirrors = Some(parsed);
    }
    let news = if print_news {
        tx.send("fetching latest news")?;
        Some(news::get(last)?)
    } else {
        None
    };
    tx.send("done")?;
    Ok((mirrors, news))
}
