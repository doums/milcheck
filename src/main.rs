// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use milcheck::cli::{Parser, Token};
use milcheck::Milcheck;
use std::env;
use std::process;

fn handle_args(args: &[Token], binary_name: String) -> Result<(), String> {
    for arg in args {
        match arg {
            Token::Option(flag, _) => {
                if flag.0 == "help" {
                    println!(
                        r"{} {}
Pierre D.
{}

USAGE:
    {} [FLAGS]

FLAGS:
    -h, --help Prints this message
    -n, --news Prints the latest news in the output
    -v, --version Prints version information
    -L, --license Prints license information",
                        env!("CARGO_PKG_NAME"),
                        env!("CARGO_PKG_VERSION"),
                        env!("CARGO_PKG_DESCRIPTION"),
                        binary_name
                    );
                    process::exit(0);
                } else if flag.0 == "version" {
                    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"),);
                    process::exit(0);
                } else if flag.0 == "license" {
                    println!("Mozilla Public License, v2.0");
                    process::exit(0);
                }
            }
            Token::UnknownOpt(option) => {
                return Err(format!(
                    "unknown option \"{}\", run {} --help",
                    option, binary_name
                ));
            }
            Token::Argument(arg) => {
                return Err(format!(
                    "unexpected argument \"{}\", run {} --help",
                    arg, binary_name
                ));
            }
        }
    }
    Ok(())
}

fn main() {
    let mut parser = Parser::new(env::args());
    let binary_name = parser.binary_name();
    let parsed = parser
        .help()
        .version()
        .license()
        .flag("news", 'n', "news", false)
        .parse();
    handle_args(&parsed, binary_name).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });
    let mut milcheck = Milcheck::from(&parsed);
    milcheck.run().unwrap_or_else(|err| {
        eprintln!("error: {}", err);
        process::exit(1);
    });
}
