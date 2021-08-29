// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::env::Args;

#[derive(Debug)]
pub struct Parser<'a> {
    args: std::env::Args,
    flags: Vec<Flag<'a>>,
    binary: String,
}

#[derive(Debug)]
pub struct Flag<'a>(pub &'a str, char, &'a str, bool);

#[derive(Debug)]
pub enum Token<'a> {
    Argument(String),
    Option(&'a Flag<'a>, Option<String>),
    UnknownOpt(String),
}

fn parse_token<'a>(options: &'a [Flag], arg: &str) -> Vec<Token<'a>> {
    let current_arg = &arg[1..];
    let mut tokens = vec![];
    for (i, c) in current_arg.char_indices() {
        if let Some(option) = options.iter().find(|option| c == option.1) {
            if option.3 {
                if i + 1 < current_arg.len() {
                    tokens.push(Token::Option(
                        option,
                        Some(String::from(&current_arg[i + 1..])),
                    ));
                    break;
                } else {
                    tokens.push(Token::Option(option, None));
                }
            } else {
                tokens.push(Token::Option(option, None));
            }
        } else {
            tokens.push(Token::UnknownOpt(c.to_string()));
        }
    }
    tokens
}

fn parse_long_token<'a>(options: &'a [Flag], arg: &str) -> Token<'a> {
    let current_arg = &arg[2..];
    match current_arg.find('=') {
        None => {
            return if let Some(option) = options.iter().find(|option| current_arg == option.2) {
                Token::Option(option, None)
            } else {
                Token::UnknownOpt(current_arg.to_string())
            }
        }
        Some(i) => {
            let first = &current_arg[..i];
            let last = &current_arg[i + 1..];
            if let Some(option) = options.iter().find(|option| first == option.2) {
                return if option.3 && !last.is_empty() {
                    Token::Option(option, Some(String::from(last)))
                } else {
                    Token::Option(option, None)
                };
            } else {
                Token::UnknownOpt(current_arg.to_string())
            }
        }
    }
}

fn tokenize<'a>(args: &mut Args, flags: &'a [Flag]) -> Vec<Token<'a>> {
    let mut tokens = vec![];
    let mut accept_opt = true;
    for arg in args {
        if !tokens.is_empty() {
            let tokens_len = tokens.len();
            let prev_token = &tokens[tokens_len - 1];
            if let Token::Option(opt, None) = prev_token {
                tokens[tokens_len - 1] = Token::Option(opt, Some(String::from(&arg)));
                continue;
            }
        }
        if arg == "-" {
            tokens.push(Token::Argument(String::from("-")));
        } else if arg == "--" {
            accept_opt = false;
        } else if arg.len() > 2 && arg.starts_with("--") && accept_opt {
            tokens.push(parse_long_token(flags, &arg));
        } else if arg.len() > 1 && arg.starts_with('-') && accept_opt {
            tokens.append(&mut parse_token(flags, &arg));
        } else {
            tokens.push(Token::Argument(arg));
        }
    }
    tokens
}

pub fn normalize(tokens: &mut Vec<Token>) {
    let mut to_merge = vec![];
    let mut inc = 0;
    let mut token_iter = tokens.iter().enumerate().peekable();
    while let Some((i, token)) = token_iter.next() {
        if let Token::Option(flag, arg) = token {
            if flag.3 && *arg == None {
                if let Some((_j, Token::Argument(value))) = token_iter.peek() {
                    to_merge.push((i - inc, *flag, value.to_string()));
                    inc += 1;
                }
            }
        }
    }
    for (i, flag, arg) in to_merge {
        tokens.remove(i);
        tokens.remove(i);
        tokens.insert(i, Token::Option(flag, Some(arg)));
    }
}

impl<'a> Parser<'a> {
    pub fn new(mut args: Args) -> Parser<'a> {
        let mut binary = env!("CARGO_PKG_NAME").to_string();
        if let Some(name) = args.next() {
            let path: Vec<&str> = name.split('/').collect();
            if let Some(value) = path.last() {
                binary = value.to_string();
            }
        }
        Parser {
            args,
            flags: vec![],
            binary,
        }
    }

    pub fn binary_name(&self) -> String {
        self.binary.to_string()
    }

    pub fn flag(
        &mut self,
        name: &'static str,
        short: char,
        long: &'static str,
        takes_value: bool,
    ) -> &mut Parser<'a> {
        self.flags.push(Flag(name, short, long, takes_value));
        self
    }

    pub fn help(&mut self) -> &mut Parser<'a> {
        self.flags.push(Flag("help", 'h', "help", false));
        self
    }

    // pub fn verbose(&mut self) -> &mut Parser {
    // self.flags.push(Flag("verbose", 'V', "verbose", false));
    // self
    // }

    pub fn version(&mut self) -> &mut Parser<'a> {
        self.flags.push(Flag("version", 'v', "version", false));
        self
    }

    pub fn license(&mut self) -> &mut Parser<'a> {
        self.flags.push(Flag("license", 'L', "license", false));
        self
    }
    // pub fn debug(&mut self) -> &mut Parser {
    // self.flags.push(Flag("debug", 'd', "debug", false));
    // self
    // }

    pub fn parse(&'a mut self) -> Vec<Token<'a>> {
        let mut tokens = tokenize(&mut self.args, &self.flags);
        normalize(&mut tokens);
        tokens
    }
}
