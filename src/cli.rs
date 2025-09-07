use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Download and keep track of audio content")]
#[command(arg_required_else_help = true)]
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
    List(ListArgs),
    #[command(about = "Download recorded media that is not persisted locally")]
    Diff,
}

#[derive(Args)]
pub struct DownloadArgs {
    #[arg(short = 'u', long = "url")]
    pub url: String,

    #[arg(
        short = 'd',
        long = "directory",
        help = "Directory to save the media, if not provided it will be saved in the root catalog (default)"
    )]
    pub directory: Option<String>,

    #[arg(
        short = 'f',
        long = "filename",
        help = "Filename to save the media, if not provided it will be from the video title (sanitized)"
    )]
    pub filename: Option<String>,

    #[arg(
        short = 't',
        long = "tags",
        help = "Comma separated key values. If `--no-record` is provided, tags will not be recorded."
    )]
    pub tags: Option<String>,

    #[arg(
        long = "no-record",
        help = "If provided, the media will not be recorded in the catalog"
    )]
    pub no_record: bool,
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(short = 'd', long = "directory")]
    pub directory: Option<String>,

    #[arg(short = 't', long = "tags", help = "Comma separated key values")]
    pub tags: Option<String>,
}
