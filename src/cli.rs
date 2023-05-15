use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version)]
#[command(about = "About", long_about = "Long about")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    Download(Download),
    Record(Download), // Uses same arguments as `Download`
    List(List),
    Diff(Diff),
}

#[derive(Args)]
#[command(
    about = "About for Download Subcommand",
    long_about = "Long about for Download Subcommand"
)]
pub struct Download {
    #[arg(short = 'd', long = "directory", required = false, help = "")]
    pub directory: Option<String>,

    #[arg(short = 'f', long = "filename", required = false)]
    pub filename: Option<String>,

    #[arg(short = 'u', long = "url")]
    pub url: String,

    #[arg(
        short = 't',
        long = "tags",
        required = false,
        help = "Comma separated key values"
    )]
    pub tags: Option<String>,
}

// NOTE: Not needed for now (or ever)
#[derive(Args)]
struct Record;

// NOTE: List, Get or Filter. Filters can be provided so it makes sense to be filter?
#[derive(Args)]
pub struct List {
    #[arg(short = 'd', long = "directory", required = false)]
    pub directory: Option<String>,

    #[arg(
        short = 't',
        long = "tags",
        required = false,
        help = "Comma separated key values"
    )]
    pub tags: Option<String>,
}

#[derive(Args)]
pub struct Diff;
