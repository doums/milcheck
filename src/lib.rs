// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod cli;
mod error;
mod event;
mod http;
mod news;
mod render;
use cli::Token;
use error::Error;
use http::Http;
use news::News;
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
const ARCHLINUX_ORG_URL: &str = "https://archlinux.org";
const OUTOFSYNC_HTML_TAG: &str = "<table id=\"outofsync_mirrors\"";
const INSYNC_HTML_TAG: &str = "<table id=\"successful_mirrors\"";
const OK: &str = "Ok";
const NOT_FOUND: &str = "Not found!";
const OUT_OF_SYNC: &str = "Out of sync!";
const HEADERS: [&str; 9] = [
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

pub struct Milcheck {
    print_news: bool,
}

impl<'a> From<&Vec<Token<'a>>> for Milcheck {
    fn from(tokens: &Vec<Token<'a>>) -> Self {
        let print_news = tokens.iter().any(|token| {
            if let Token::Option(flag, _) = token {
                return flag.0 == "news";
            }
            false
        });
        Milcheck { print_news }
    }
}

impl Milcheck {
    pub fn run(&mut self) -> Result<(), Error> {
        let (tx, rx) = mpsc::channel();
        let mut render = Render::new();
        let tx_cloned = Sender::clone(&tx);
        match logic(tx_cloned, rx, &mut render, self.print_news) {
            Ok((mirrors, news)) => {
                drop(tx);
                render.finish()?;
                print_mirrors(mirrors)?;
                if let Some(text) = news {
                    println!("{}", text);
                }
            }
            Err(err) => {
                drop(tx);
                render.finish()?;
                return Err(err);
            }
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
        let completion = json.completion_pct.map(|completion| completion * 100f64);
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
    fn new(mirrors: &[MirrorState]) -> Result<Self, Error> {
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

fn find_max_len(mirrors: &[MirrorState], key: &'static str) -> Result<usize, Error> {
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

fn print_mirrors(mirrors: Vec<MirrorState>) -> Result<(), Error> {
    let max_lengths = MaxLength::new(&mirrors)?;
    print_headers(&max_lengths);
    for mirror_state in &mirrors {
        match mirror_state {
            MirrorState::NotFound(server) => {
                println!(
                    "{}{}{}{} {}",
                    Bold,
                    Fg(Yellow),
                    format!("{:>width$}", NOT_FOUND, width = max_lengths.state),
                    Reset,
                    server
                );
            }
            MirrorState::OutOfSync(mirror) => {
                print_mirror(&max_lengths, &mirror, OUT_OF_SYNC, Red);
            }
            MirrorState::Synced(mirror) => {
                print_mirror(&max_lengths, &mirror, OK, Green);
            }
        }
    }
    println!();
    Ok(())
}

fn parse_mirrorlist() -> Result<Vec<String>, String> {
    let mut mirrors = vec![];
    let mirrorlist = fs::read_to_string(PACMAN_MIRRORLIST).map_err(|err| {
        format!(
            "an error occured while reading the file {}: {}",
            PACMAN_MIRRORLIST, err
        )
    })?;
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
        Err(format!("no server found in {}", PACMAN_MIRRORLIST))
    } else {
        Ok(mirrors)
    }
}

pub fn logic(
    tx: Sender<&'static str>,
    rx: Receiver<&'static str>,
    render: &mut Render,
    print_news: bool,
) -> Result<(Vec<MirrorState>, Option<String>), Error> {
    let mut mirrors = vec![];
    render.run(rx);
    tx.send("parsing local mirrorlist")?;
    let mirrorlist = parse_mirrorlist()?;
    tx.send("fetching mirror status list")?;
    let request = Http::get(MIRROR_STATUS_URL);
    let json_request = Http::get(MIRROR_STATUS_JSON_URL);
    let mut arch_org_request = None;
    if print_news {
        arch_org_request = Some(Http::get(ARCHLINUX_ORG_URL));
    }
    let response = request.wait()?;
    let json_response = json_request.wait()?;
    let mut org_response = None;
    if let Some(req) = arch_org_request {
        org_response = Some(req.wait()?);
    }
    tx.send("deserialize json data")?;
    let json: JsonResponse = serde_json::from_str(&json_response)
        .map_err(|err| format!("json response parsing failed: {}", err))?;
    tx.send("web scraping")?;
    let v: Vec<&str> = response.split("</table>").collect();
    if v.len() != 4 {
        return Err(Error::new("web scraping failed"));
    }
    if v[0].find(OUTOFSYNC_HTML_TAG).is_none() {
        return Err(Error::new("web scraping failed"));
    }
    if v[1].find(INSYNC_HTML_TAG).is_none() {
        return Err(Error::new("web scraping failed"));
    }
    tx.send("building data")?;
    for server in mirrorlist {
        if let Some(mirror) = json.urls.iter().find(|&mirror| mirror.url == server) {
            if let Some(_i) = v[0].find(&server) {
                mirrors.push(MirrorState::OutOfSync(Mirror::from(mirror)));
            } else {
                mirrors.push(MirrorState::Synced(Mirror::from(mirror)));
            }
        } else {
            mirrors.push(MirrorState::NotFound(server));
        }
    }
    let news_text = if print_news {
        tx.send("parsing news data")?;
        if org_response.is_none() {
            return Err(Error::new("fail to fetch archlinux.org data"));
        }
        let mut news_parser = News::new(org_response.unwrap(), ARCHLINUX_ORG_URL);
        Some(news_parser.parse()?)
    } else {
        None
    };
    tx.send("done")?;
    Ok((mirrors, news_text))
}
