use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Args, Debug)]
#[group(required = true)]
pub struct StreamKind {
    /// Use laserstream
    #[arg(long)]
    pub laser: bool,

    /// Use yellowstone geyser
    #[arg(long)]
    pub grpc: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Record time blocks were received
    Record {
        /// Stream to fetch from (laser or grpc)
        #[command(flatten)]
        kind: StreamKind,

        /// Path to append to
        #[arg(short, long)]
        path: PathBuf,
    },
}

#[derive(Debug, Parser)]
#[clap(name = "app", version)]
pub struct App {
    #[clap(subcommand)]
    pub command: Command,
}
