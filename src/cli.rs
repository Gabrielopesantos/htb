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
    Record(Record),
    List(List),
}

#[derive(Args)]
#[command(
    about = "About for Download Subcommand",
    long_about = "Long about for Download Subcommand"
)]
pub struct Download {
    #[arg(short = 'f', long = "filename")]
    pub filename: String,

    #[arg(short = 'u', long = "url")]
    pub url: String,
}

#[derive(Args)]
struct Record;

#[derive(Args)]
struct List;
