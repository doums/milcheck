use crate::error::Error;
use html2text::{
    from_read_rich, from_read_with_decorator,
    render::text_renderer::{RichAnnotation, TaggedLine, TaggedLineElement, TextDecorator},
};
use scraper::{Html, Selector};
use std::fmt::{Display, Error as fmtError, Formatter};
use std::vec;
use termion::style::{Bold, Reset as StyleReset, Underline};
use termion::terminal_size;
use termion::{color::*, style::Italic};

// https://tachyons.io/docs/typography/measure/
const LINE_LENGTH: usize = 66;

pub struct Article<'a> {
    title: &'a str,
    link: &'a str,
    content: &'a str,
    date: &'a str,
}

impl<'a> Display for Article<'a> {
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

pub struct News<'a> {
    raw_html: String,
    arch_url: &'a str,
}

#[derive(Debug)]
struct ContentDecorator(Vec<String>);

impl<'a> News<'a> {
    pub fn new(raw_html: String, arch_url: &'a str) -> Self {
        News { raw_html, arch_url }
    }

    pub fn parse(&mut self) -> Result<String, Error> {
        let mut term_width = terminal_size()?.0 as usize;
        if term_width > LINE_LENGTH {
            term_width = LINE_LENGTH;
        }
        let document = Html::parse_document(&self.raw_html);
        let date_selector = Selector::parse("#news > .timestamp").unwrap();
        let content_selector = Selector::parse("#news > .article-content").unwrap();
        let dates = document
            .select(&date_selector)
            .map(|element| element.text().collect())
            .collect::<Vec<String>>();
        let contents: Vec<String> = document
            .select(&content_selector)
            .map(|element| element.html())
            .map(|element| {
                from_read_with_decorator(element.as_bytes(), term_width, ContentDecorator(vec![]))
            })
            .collect();
        let titles = parse_titles(&document, self.arch_url);
        if ![titles.len(), dates.len(), contents.len()]
            .iter()
            .all(|&count| count == titles.len())
        {
            return Err(Error::new("failed to parse news data"));
        }
        let articles = titles.iter().enumerate().map(|(i, val)| Article {
            title: &val.0,
            link: &val.1,
            content: &contents[i],
            date: &dates[i],
        });
        let output = format!(
            "{}{}Latest News{}\n{}{}{}/news{}{}",
            Bold,
            Fg(Yellow),
            StyleReset,
            Underline,
            Fg(Blue),
            self.arch_url,
            StyleReset,
            Fg(Reset)
        );
        let articles = articles.fold(String::new(), |acc, article| {
            format!("{}\n{}", acc, article)
        });
        Ok(format!("{}\n{}", output, articles))
    }
}

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

    fn decorate_em_start(&mut self) -> (String, Self::Annotation) {
        ("_".to_string(), RichAnnotation::Emphasis)
    }

    fn decorate_em_end(&mut self) -> String {
        "_".to_string()
    }

    fn decorate_strong_start(&mut self) -> (String, Self::Annotation) {
        ("*".to_string(), RichAnnotation::Strong)
    }

    fn decorate_strong_end(&mut self) -> String {
        "*".to_string()
    }

    fn decorate_strikeout_start(&mut self) -> (String, Self::Annotation) {
        ("~".to_string(), RichAnnotation::Strikeout)
    }

    fn decorate_strikeout_end(&mut self) -> String {
        "~".to_string()
    }

    fn decorate_code_start(&mut self) -> (String, Self::Annotation) {
        ("`".to_string(), RichAnnotation::Code)
    }

    fn decorate_code_end(&mut self) -> String {
        "`".to_string()
    }

    fn decorate_preformat_first(&mut self) -> Self::Annotation {
        RichAnnotation::Preformat(false)
    }

    fn decorate_preformat_cont(&mut self) -> Self::Annotation {
        RichAnnotation::Preformat(true)
    }

    fn decorate_image(&mut self, title: &str) -> (String, Self::Annotation) {
        self.0.push(title.to_string());
        (format!("[I][{}] ", self.0.len()), RichAnnotation::Image)
    }

    fn make_subblock_decorator(&self) -> Self {
        ContentDecorator(vec![])
    }

    fn finalise(self) -> Vec<TaggedLine<Self::Annotation>> {
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

fn parse_titles(document: &Html, arch_url: &str) -> Vec<(String, String)> {
    let title_selector = Selector::parse("#news > h4 > a").unwrap();
    return document
        .select(&title_selector)
        .filter_map(|element| {
            let tagged_lines = from_read_rich(element.html().as_bytes(), 1024);
            if let Some(element) = tagged_lines.first() {
                if let Some(TaggedLineElement::Str(tagged_string)) = element.iter().next() {
                    if let Some(RichAnnotation::Link(link)) = tagged_string.tag.first() {
                        return Some((tagged_string.s.to_owned(), format!("{}{}", arch_url, link)));
                    }
                }
            }
            None
        })
        .collect();
}
