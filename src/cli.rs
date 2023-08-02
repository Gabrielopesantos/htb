use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Download and keep track of audio content")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Download and record audio content")]
    Download(DownloadArgs),
    #[command(about = "Record audio content")]
    Record(DownloadArgs),
    #[command(about = "List all media in a catalog")]
    List {
        #[arg(short = 'd', long = "directory", required = false)]
        directory: Option<String>,

        #[arg(
            short = 't',
            long = "tags",
            required = false,
            help = "Comma separated key values"
        )]
        tags: Option<String>,
    },
    #[command(about = "Download recorded media that is not persisted locally")]
    Diff,
}

#[derive(Args)]
pub struct DownloadArgs {
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
