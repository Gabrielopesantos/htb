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
pub struct Download {
    #[arg(short = 'l', long = "link")]
    pub link: String,
}

#[derive(Args)]
struct Record;

#[derive(Args)]
struct List;
