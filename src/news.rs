use std::vec;

use crate::error::Error;
use html2text::{
    from_read_rich, from_read_with_decorator,
    render::text_renderer::{RichAnnotation, TaggedLine, TaggedLineElement, TextDecorator},
};
use scraper::{Html, Selector};
use termion::style::{Bold, CrossedOut, Reset as StyleReset, Underline};
use termion::terminal_size;
use termion::{color::*, style::Italic};

pub struct Article<'a> {
    title: &'a str,
    link: &'a str,
    content: &'a str,
    date: &'a str,
}

pub struct News<'a> {
    articles: Vec<Article<'a>>,
    raw_html: String,
    arch_url: &'a str,
}

#[derive(Debug)]
struct ContentDecorator(Vec<String>);

impl<'a> News<'a> {
    pub fn new(raw_html: String, arch_url: &'a str) -> Self {
        News {
            articles: vec![],
            raw_html,
            arch_url,
        }
    }

    pub fn parse(&mut self) -> Result<String, Error> {
        let term_width = terminal_size()?.0 as usize;
        let document = Html::parse_document(&self.raw_html);
        let date_selector = Selector::parse("#news > .timestamp").unwrap();
        let content_selector = Selector::parse("#news > .article-content").unwrap();
        let dates = document
            .select(&date_selector)
            .map(|element| element.text().collect())
            .collect::<Vec<String>>();
        println!("{:#?}", dates);
        let contents: Vec<String> = document
            .select(&content_selector)
            .map(|element| element.html())
            .map(|element| {
                from_read_with_decorator(element.as_bytes(), term_width, ContentDecorator(vec![]))
            })
            .collect();
        println!("{:#?}", contents[0]);
        let titles = parse_titles(&document, self.arch_url);
        Ok("eheh".to_string())
    }
}

impl TextDecorator for ContentDecorator {
    type Annotation = RichAnnotation;

    fn decorate_link_start(&mut self, url: &str) -> (String, Self::Annotation) {
        self.0.push(url.to_string());
        (
            format!("{}{}* {}", Fg(Black), self.0.len() + 1, Fg(Blue)),
            // format!(
            // "{}{}{}* {}{}",
            // Italic,
            // Fg(Black),
            // self.0.len() + 1,
            // StyleReset,
            // Fg(Blue)
            // ),
            RichAnnotation::Link(url.to_string()),
        )
    }

    fn decorate_link_end(&mut self) -> String {
        format!("{}", Fg(Reset))
    }

    fn decorate_em_start(&mut self) -> (String, Self::Annotation) {
        (format!("{}", Fg(Magenta)), RichAnnotation::Emphasis)
        // (
        // format!("{}{}", Italic, Fg(Magenta)),
        // RichAnnotation::Emphasis,
        // )
    }

    fn decorate_em_end(&mut self) -> String {
        // format!("{}{}", Fg(Reset), StyleReset)
        format!("{}", Fg(Reset))
    }

    fn decorate_strong_start(&mut self) -> (String, Self::Annotation) {
        // (format!("{}{}", Bold, Fg(Blue)), RichAnnotation::Strong)
        println!("{:#?}", format!("{}", Fg(Blue)));
        (format!("{}", Fg(Blue)), RichAnnotation::Strong)
    }

    fn decorate_strong_end(&mut self) -> String {
        // format!("{}{}", Fg(Reset), StyleReset)
        format!("{}", Fg(Reset))
    }

    fn decorate_strikeout_start(&mut self) -> (String, Self::Annotation) {
        (
            format!("{}{}", CrossedOut, Fg(Red)),
            RichAnnotation::Strikeout,
        )
    }

    fn decorate_strikeout_end(&mut self) -> String {
        format!("{}{}", Fg(Reset), StyleReset)
    }

    fn decorate_code_start(&mut self) -> (String, Self::Annotation) {
        (format!("{}", Fg(Yellow)), RichAnnotation::Code)
    }

    fn decorate_code_end(&mut self) -> String {
        format!("{}", Fg(Reset))
    }

    fn decorate_preformat_first(&mut self) -> Self::Annotation {
        RichAnnotation::Preformat(false)
    }

    fn decorate_preformat_cont(&mut self) -> Self::Annotation {
        RichAnnotation::Preformat(true)
    }

    fn decorate_image(&mut self, title: &str) -> (String, Self::Annotation) {
        self.0.push(title.to_string());
        (
            format!("{}{}* {}", Fg(Black), self.0.len() + 1, Fg(Blue)),
            // format!(
            // "{}{}{}* {}{}",
            // Italic,
            // Fg(Black),
            // self.0.len() + 1,
            // StyleReset,
            // Fg(Blue)
            // ),
            RichAnnotation::Image,
        )
    }

    fn make_subblock_decorator(&self) -> Self {
        ContentDecorator(vec![])
    }

    fn finalise(self) -> Vec<TaggedLine<Self::Annotation>> {
        vec![]
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
