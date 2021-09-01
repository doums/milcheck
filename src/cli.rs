// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::env::Args;

#[derive(Debug)]
pub struct Parser<'a> {
    args: Vec<String>,
    flags: Vec<Flag<'a>>,
    binary: String,
}

#[derive(Debug)]
pub struct Flag<'a>(pub &'a str, char, &'a str, bool);

#[derive(Debug, Clone, Copy)]
pub enum Token<'a> {
    Argument(&'a str),
    Option(&'a Flag<'a>, Option<&'a str>),
    UnknownFlag(&'a str),
    UnknownShortFlag(char),
}

fn parse_token<'a>(options: &'a [Flag], arg: &'a str) -> Vec<Token<'a>> {
    let current_arg = &arg[1..];
    let mut tokens = vec![];
    for (i, c) in current_arg.char_indices() {
        if let Some(option) = options.iter().find(|option| c == option.1) {
            if option.3 {
                if i + 1 < current_arg.len() {
                    tokens.push(Token::Option(option, Some(&current_arg[i + 1..])));
                    break;
                } else {
                    tokens.push(Token::Option(option, None));
                }
            } else {
                tokens.push(Token::Option(option, None));
            }
        } else {
            tokens.push(Token::UnknownShortFlag(c));
        }
    }
    tokens
}

fn parse_long_token<'a>(options: &'a [Flag], arg: &'a str) -> Token<'a> {
    let current_arg = &arg[2..];
    match current_arg.find('=') {
        None => {
            return if let Some(option) = options.iter().find(|option| current_arg == option.2) {
                Token::Option(option, None)
            } else {
                Token::UnknownFlag(current_arg)
            }
        }
        Some(i) => {
            let first = &current_arg[..i];
            let last = &current_arg[i + 1..];
            if let Some(option) = options.iter().find(|option| first == option.2) {
                return if option.3 && !last.is_empty() {
                    Token::Option(option, Some(last))
                } else {
                    Token::Option(option, None)
                };
            } else {
                Token::UnknownFlag(current_arg)
            }
        }
    }
}

fn tokenize<'a>(args: &'a [String], flags: &'a [Flag]) -> Vec<Token<'a>> {
    let mut tokens = vec![];
    let mut accept_opt = true;
    for arg in args {
        if arg == "-" {
            tokens.push(Token::Argument(arg));
        } else if arg == "--" {
            accept_opt = false;
        } else if arg.len() > 2 && arg.starts_with("--") && accept_opt {
            tokens.push(parse_long_token(flags, arg));
        } else if arg.len() > 1 && arg.starts_with('-') && accept_opt {
            tokens.append(&mut parse_token(flags, arg));
        } else {
            tokens.push(Token::Argument(arg));
        }
    }
    tokens
}

pub fn normalize<'a>(tokens: &[Token<'a>]) -> Vec<Token<'a>> {
    let mut result = vec![];
    let mut token_iter = tokens.iter().peekable();
    while let Some(token) = token_iter.next() {
        if let Token::Option(flag, None) = token {
            if flag.3 {
                if let Some(Token::Argument(value)) = token_iter.peek() {
                    result.push(Token::Option(flag, Some(value)));
                    token_iter.next();
                    continue;
                }
            }
        }
        result.push(*token);
    }
    result
}

impl<'a> From<Args> for Parser<'a> {
    fn from(mut args: Args) -> Self {
        let mut binary = env!("CARGO_PKG_NAME").to_string();
        if let Some(name) = args.next() {
            let path: Vec<&str> = name.split('/').collect();
            if let Some(value) = path.last() {
                binary = value.to_string();
            }
        }
        Parser {
            args: args.collect(),
            flags: vec![],
            binary,
        }
    }
}

impl<'a, const N: usize> From<&[&str; N]> for Parser<'a> {
    fn from(args: &[&str; N]) -> Self {
        let mut binary = env!("CARGO_PKG_NAME").to_string();
        if let Some(name) = args.first() {
            let path: Vec<&str> = name.split('/').collect();
            if let Some(value) = path.last() {
                binary = value.to_string();
            }
        }
        Parser {
            args: args.iter().skip(1).map(|str| str.to_string()).collect(),
            flags: vec![],
            binary,
        }
    }
}

impl<'a> Parser<'a> {
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

    pub fn verbose(&mut self) -> &mut Parser<'a> {
        self.flags.push(Flag("verbose", 'V', "verbose", false));
        self
    }

    pub fn version(&mut self) -> &mut Parser<'a> {
        self.flags.push(Flag("version", 'v', "version", false));
        self
    }

    pub fn license(&mut self) -> &mut Parser<'a> {
        self.flags.push(Flag("license", 'L', "license", false));
        self
    }

    pub fn debug(&mut self) -> &mut Parser<'a> {
        self.flags.push(Flag("debug", 'd', "debug", false));
        self
    }

    pub fn parse(&'a mut self) -> Vec<Token<'a>> {
        let tokens = tokenize(&self.args, &self.flags);
        normalize(&tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_args() {
        let mut parser = Parser::from(&[]);
        let parsed = parser.parse();
        assert!(parsed.is_empty());
    }

    #[test]
    fn binary_name_empty_args() {
        let parser = Parser::from(&[]);
        let binary_name = parser.binary_name();
        assert_eq!(binary_name, env!("CARGO_PKG_NAME"));
    }

    #[test]
    fn binary_name_one_arg() {
        let parser = Parser::from(&["bin"]);
        let binary_name = parser.binary_name();
        assert_eq!(binary_name, "bin");
    }

    #[test]
    fn binary_name_args() {
        let parser = Parser::from(&["bin", "un"]);
        let binary_name = parser.binary_name();
        assert_eq!(binary_name, "bin");
    }

    #[test]
    fn parse_with_one_arg() {
        let mut parser = Parser::from(&["un"]);
        let parsed = parser.parse();
        assert!(parsed.is_empty());
    }

    #[test]
    fn parse_with_args_no_config() {
        let mut parser = Parser::from(&["un", "deux", "trois"]);
        let parsed = parser.parse();
        assert_eq!(parsed.len(), 2);
        match parsed.get(0) {
            Some(Token::Argument("deux")) => {}
            _ => panic!(),
        }
        match parsed.get(1) {
            Some(Token::Argument("trois")) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn flag_help() {
        let mut parser = Parser::from(&["bin", "-h"]);
        let parsed = parser.help().parse();
        assert_eq!(parsed.len(), 1);
        match parsed.get(0) {
            Some(Token::Option(Flag("help", 'h', "help", false), None)) => {}
            _ => panic!(),
        }
        let mut parser = Parser::from(&["bin", "--help"]);
        let parsed = parser.help().parse();
        assert_eq!(parsed.len(), 1);
        match parsed.get(0) {
            Some(Token::Option(Flag("help", 'h', "help", false), None)) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn flag_version() {
        let mut parser = Parser::from(&["bin", "-v"]);
        let parsed = parser.version().parse();
        assert_eq!(parsed.len(), 1);
        match parsed.get(0) {
            Some(Token::Option(Flag("version", 'v', "version", false), None)) => {}
            _ => panic!(),
        }
        let mut parser = Parser::from(&["bin", "--version"]);
        let parsed = parser.version().parse();
        assert_eq!(parsed.len(), 1);
        match parsed.get(0) {
            Some(Token::Option(Flag("version", 'v', "version", false), None)) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn flag_license() {
        let mut parser = Parser::from(&["bin", "-L"]);
        let parsed = parser.license().parse();
        assert_eq!(parsed.len(), 1);
        match parsed.get(0) {
            Some(Token::Option(Flag("license", 'L', "license", false), None)) => {}
            _ => panic!(),
        }
        let mut parser = Parser::from(&["bin", "--license"]);
        let parsed = parser.license().parse();
        assert_eq!(parsed.len(), 1);
        match parsed.get(0) {
            Some(Token::Option(Flag("license", 'L', "license", false), None)) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn flag_verbose() {
        let mut parser = Parser::from(&["bin", "-V"]);
        let parsed = parser.verbose().parse();
        assert_eq!(parsed.len(), 1);
        match parsed.get(0) {
            Some(Token::Option(Flag("verbose", 'V', "verbose", false), None)) => {}
            _ => panic!(),
        }
        let mut parser = Parser::from(&["bin", "--verbose"]);
        let parsed = parser.verbose().parse();
        assert_eq!(parsed.len(), 1);
        match parsed.get(0) {
            Some(Token::Option(Flag("verbose", 'V', "verbose", false), None)) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn flag_debug() {
        let mut parser = Parser::from(&["bin", "-d"]);
        let parsed = parser.debug().parse();
        assert_eq!(parsed.len(), 1);
        match parsed.get(0) {
            Some(Token::Option(Flag("debug", 'd', "debug", false), None)) => {}
            _ => panic!(),
        }
        let mut parser = Parser::from(&["bin", "--debug"]);
        let parsed = parser.debug().parse();
        assert_eq!(parsed.len(), 1);
        match parsed.get(0) {
            Some(Token::Option(Flag("debug", 'd', "debug", false), None)) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn flag_custom() {
        let mut parser = Parser::from(&["bin", "-f"]);
        let parsed = parser.flag("flag", 'f', "flag", false).parse();
        assert_eq!(parsed.len(), 1);
        match parsed.get(0) {
            Some(Token::Option(Flag("flag", 'f', "flag", false), None)) => {}
            _ => panic!(),
        }
        let mut parser = Parser::from(&["bin", "--flag"]);
        let parsed = parser.flag("flag", 'f', "flag", false).parse();
        assert_eq!(parsed.len(), 1);
        match parsed.get(0) {
            Some(Token::Option(Flag("flag", 'f', "flag", false), None)) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn flag_takes_no_arg_long() {
        let mut parser = Parser::from(&["bin", "--flag=ignored", "arg"]);
        let parsed = parser.flag("flag", 'f', "flag", false).parse();
        assert_eq!(parsed.len(), 2);
        match parsed.get(0) {
            Some(Token::Option(Flag("flag", ..), None)) => {}
            _ => panic!(),
        }
        match parsed.get(1) {
            Some(Token::Argument("arg")) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn flag_takes_no_arg_short() {
        let mut parser = Parser::from(&["bin", "-f", "arg"]);
        let parsed = parser.flag("flag", 'f', "flag", false).parse();
        assert_eq!(parsed.len(), 2);
        match parsed.get(0) {
            Some(Token::Option(Flag("flag", ..), None)) => {}
            _ => panic!(),
        }
        match parsed.get(1) {
            Some(Token::Argument("arg")) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn flag_takes_arg_long() {
        let mut parser = Parser::from(&["bin", "--flag=arg", "arg"]);
        let parsed = parser.flag("flag", 'f', "flag", true).parse();
        assert_eq!(parsed.len(), 2);
        match parsed.get(0) {
            Some(Token::Option(Flag("flag", ..), Some("arg"))) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn flag_takes_arg_long_split_1() {
        let mut parser = Parser::from(&["bin", "--flag", "arg"]);
        let parsed = parser.flag("flag", 'f', "flag", true).parse();
        assert_eq!(parsed.len(), 1);
        match parsed.get(0) {
            Some(Token::Option(flag, Some(arg))) => {
                assert_eq!(flag.0, "flag");
                assert_eq!(*arg, "arg");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn flag_takes_arg_long_split_2() {
        let mut parser = Parser::from(&["bin", "--flag", "-h"]);
        let parsed = parser.help().flag("flag", 'f', "flag", true).parse();
        assert_eq!(parsed.len(), 2);
        match parsed.get(0) {
            Some(Token::Option(Flag("flag", ..), None)) => {}
            _ => panic!(),
        }
        match parsed.get(1) {
            Some(Token::Option(Flag("help", ..), None)) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn flag_takes_arg_short_1() {
        let mut parser = Parser::from(&["bin", "-f", "arg"]);
        let parsed = parser.flag("flag", 'f', "flag", true).parse();
        assert_eq!(parsed.len(), 1);
        match parsed.get(0) {
            Some(Token::Option(Flag("flag", ..), Some("arg"))) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn flag_takes_arg_short_2() {
        let mut parser = Parser::from(&["bin", "-f", "-h"]);
        let parsed = parser.help().flag("flag", 'f', "flag", true).parse();
        assert_eq!(parsed.len(), 2);
        match parsed.get(0) {
            Some(Token::Option(Flag("flag", ..), None)) => {}
            _ => panic!(),
        }
        match parsed.get(1) {
            Some(Token::Option(Flag("help", ..), None)) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn flag_takes_arg_short_glued_1() {
        let mut parser = Parser::from(&["bin", "-farg", "un"]);
        let parsed = parser.flag("flag", 'f', "flag", true).parse();
        assert_eq!(parsed.len(), 2);
        match parsed.get(0) {
            Some(Token::Option(Flag("flag", ..), Some("arg"))) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn flag_takes_arg_short_glued_2() {
        let mut parser = Parser::from(&["bin", "-f-h"]);
        let parsed = parser.help().flag("flag", 'f', "flag", true).parse();
        assert_eq!(parsed.len(), 1);
        match parsed.get(0) {
            Some(Token::Option(Flag("flag", ..), Some("-h"))) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn unknown_short_flag() {
        let mut parser = Parser::from(&["bin", "-n"]);
        let parsed = parser.help().parse();
        assert_eq!(parsed.len(), 1);
        match parsed.get(0) {
            Some(Token::UnknownShortFlag('n')) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn unknown_long_flag() {
        let mut parser = Parser::from(&["bin", "--nobody"]);
        let parsed = parser.help().parse();
        assert_eq!(parsed.len(), 1);
        match parsed.get(0) {
            Some(Token::UnknownFlag("nobody")) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn parse_short_series_1() {
        let mut parser = Parser::from(&["bin", "-hnvd"]);
        let parsed = parser.version().verbose().debug().help().parse();
        assert_eq!(parsed.len(), 4);
        match parsed.get(0) {
            Some(Token::Option(Flag("help", ..), None)) => {}
            _ => panic!(),
        }
        match parsed.get(1) {
            Some(Token::UnknownShortFlag('n')) => {}
            _ => panic!(),
        }
        match parsed.get(2) {
            Some(Token::Option(Flag("version", ..), None)) => {}
            _ => panic!(),
        }
        match parsed.get(3) {
            Some(Token::Option(Flag("debug", ..), None)) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn parse_short_series_2() {
        let mut parser = Parser::from(&["bin", "-hfvd"]);
        let parsed = parser
            .version()
            .verbose()
            .debug()
            .help()
            .flag("flag", 'f', "flag", true)
            .parse();
        assert_eq!(parsed.len(), 2);
        match parsed.get(0) {
            Some(Token::Option(Flag("help", ..), None)) => {}
            _ => panic!(),
        }
        match parsed.get(1) {
            Some(Token::Option(Flag("flag", ..), Some("vd"))) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn just_a_dash() {
        let mut parser = Parser::from(&["bin", "-", "-h"]);
        let parsed = parser.help().parse();
        assert_eq!(parsed.len(), 2);
        match parsed.get(0) {
            Some(Token::Argument("-")) => {}
            _ => panic!(),
        }
        match parsed.get(1) {
            Some(Token::Option(Flag("help", ..), None)) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn double_dash() {
        let mut parser = Parser::from(&["bin", "-v", "--", "-h"]);
        let parsed = parser.help().version().parse();
        assert_eq!(parsed.len(), 2);
        match parsed.get(0) {
            Some(Token::Option(Flag("version", ..), None)) => {}
            _ => panic!(),
        }
        match parsed.get(1) {
            Some(Token::Argument("-h")) => {}
            _ => panic!(),
        }
    }
}
