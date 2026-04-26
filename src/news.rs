// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::{Context, Result};
use chrono::DateTime;
use html2text::{
    from_read_with_decorator,
    render::{RichAnnotation, TaggedLine, TextDecorator},
};
use rss::Channel;
use std::fmt::{Display, Error as fmtError, Formatter};
use termion::style::{Bold, Reset as StyleReset, Underline};
use termion::{color::*, style::Italic};

use crate::ARCHLINUX_ORG_URL;

const RSS_FEED: &str = "/feeds/news/";
// https://tachyons.io/docs/typography/measure/
const LINE_LENGTH: usize = 66;

pub struct NewsItem<'rss_item> {
    title: &'rss_item str,
    link: &'rss_item str,
    content: String,
    date: String,
}

impl<'rss> NewsItem<'rss> {
    fn from_rss(item: &'rss rss::Item, term_width: usize) -> Self {
        let description = item.description().and_then(|desc| {
            from_read_with_decorator(desc.as_bytes(), term_width, ContentDecorator(vec![]))
                .context("fail to parse html item")
                .ok()
        });
        let date = item.pub_date().and_then(|dt| {
            DateTime::parse_from_rfc2822(dt)
                .map(|d| d.format("%Y-%m-%d").to_string())
                .context("fail to parse date")
                .ok()
        });
        NewsItem {
            title: item.title().unwrap_or_default(),
            link: item.link().unwrap_or_default(),
            content: description.unwrap_or("-".to_string()),
            date: date.unwrap_or_default(),
        }
    }
}

impl<'rss> Display for NewsItem<'rss> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmtError> {
        write!(
            f,
            "{}{}{}{} {}{}{}{}\n{}{}{}{}\n\n{}{}{}",
            // date
            Italic,
            Fg(Magenta),
            self.date,
            StyleReset,
            // title
            Bold,
            Fg(Green),
            self.title,
            StyleReset,
            // link
            Underline,
            Fg(Blue),
            self.link,
            StyleReset,
            //content
            StyleReset,
            Fg(Reset),
            self.content
        )
    }
}

fn rss_feed(url: &str) -> Result<Channel> {
    let content = reqwest::blocking::get(url)
        .context("rss get failed")?
        .bytes()
        .context("rss get: failed to decode response")?;
    Channel::read_from(&content[..]).context("failed to read RSS channel")
}

pub fn get(last: Option<u8>) -> Result<String> {
    let mut term_width = termion::terminal_size()?.0 as usize;
    if term_width > LINE_LENGTH {
        term_width = LINE_LENGTH;
    }
    let channel = rss_feed(&format!("{}{}", ARCHLINUX_ORG_URL, RSS_FEED))?;
    let mut news: Vec<NewsItem> = channel
        .items()
        .iter()
        .map(|item| NewsItem::from_rss(item, term_width))
        .collect();
    if let Some(l) = last {
        news.truncate(l as usize);
    };
    let output = format!(
        "{}{}Latest News{}\n{}{}{}/news{}{}",
        Bold,
        Fg(Yellow),
        StyleReset,
        Underline,
        Fg(Blue),
        ARCHLINUX_ORG_URL,
        StyleReset,
        Fg(Reset)
    );
    let formatted_news = news
        .iter()
        .fold(String::new(), |acc, item| format!("{}\n{}", acc, item));
    Ok(format!("{}\n{}", output, formatted_news))
}

#[derive(Debug)]
struct ContentDecorator(Vec<String>);

impl TextDecorator for ContentDecorator {
    type Annotation = RichAnnotation;

    fn decorate_link_start(&mut self, url: &str) -> (String, Self::Annotation) {
        self.0.push(url.to_string());
        (
            format!(">[{}] ", self.0.len()),
            RichAnnotation::Link(url.to_string()),
        )
    }

    fn decorate_link_end(&mut self) -> String {
        "<".to_string()
    }

    fn decorate_em_start(&self) -> (String, Self::Annotation) {
        ("_".to_string(), RichAnnotation::Emphasis)
    }

    fn decorate_em_end(&self) -> String {
        "_".to_string()
    }

    fn decorate_strong_start(&self) -> (String, Self::Annotation) {
        ("*".to_string(), RichAnnotation::Strong)
    }

    fn decorate_strong_end(&self) -> String {
        "*".to_string()
    }

    fn decorate_strikeout_start(&self) -> (String, Self::Annotation) {
        ("~".to_string(), RichAnnotation::Strikeout)
    }

    fn decorate_strikeout_end(&self) -> String {
        "~".to_string()
    }

    fn decorate_code_start(&self) -> (String, Self::Annotation) {
        ("`".to_string(), RichAnnotation::Code)
    }

    fn decorate_code_end(&self) -> String {
        "`".to_string()
    }

    fn decorate_preformat_first(&self) -> Self::Annotation {
        RichAnnotation::Preformat(false)
    }

    fn decorate_preformat_cont(&self) -> Self::Annotation {
        RichAnnotation::Preformat(true)
    }

    fn decorate_image(&mut self, src: &str, title: &str) -> (String, Self::Annotation) {
        self.0.push(title.to_string());
        (
            format!("[I][{}] ", self.0.len()),
            RichAnnotation::Image(src.into()),
        )
    }

    fn make_subblock_decorator(&self) -> Self {
        ContentDecorator(vec![])
    }

    fn header_prefix(&self, level: usize) -> String {
        let mut s = String::with_capacity(level + 1);
        for _ in 0..level {
            s.push('#')
        }
        s.push(' ');
        s
    }

    fn quote_prefix(&self) -> String {
        "> ".to_string()
    }

    fn unordered_item_prefix(&self) -> String {
        "* ".to_string()
    }

    fn ordered_item_prefix(&self, i: i64) -> String {
        format!("{}. ", i)
    }

    fn finalise(&mut self, _links: Vec<String>) -> Vec<TaggedLine<Self::Annotation>> {
        let mut lines = vec![];
        self.0.iter().enumerate().for_each(|(i, val)| {
            lines.push(TaggedLine::from_string(
                format!("[{}] {}", i + 1, val),
                &RichAnnotation::Default,
            ))
        });
        lines
    }
}
