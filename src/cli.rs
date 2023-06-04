use clap::Parser;

/// Print your mirrorlist status and Arch Linux latest news
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Print mirrorlist status (default)
    #[arg(short, long)]
    pub mirrorlist: Option<Option<bool>>,
    /// Print the latest news
    #[arg(short, long)]
    pub news: Option<u8>,
}
