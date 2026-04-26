use clap::Parser;

/// Print your mirrorlist status and Arch Linux latest news
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Print mirrorlist status
    #[arg(short, long)]
    pub mirrorlist: bool,
    /// Print the latest news
    #[arg(short, long)]
    pub news: Option<Option<u8>>,
}
