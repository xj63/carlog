use clap::{Parser, Subcommand};

use carlog::connect::ConnectSubcommand;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// connect to ticar2, watch log and send message.
    Connect(ConnectSubcommand),
}

impl Commands {
    pub async fn run(self) {
        match self {
            Commands::Connect(c) => c.run().await,
        }
    }
}
