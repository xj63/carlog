use clap::{Parser, Subcommand};

use crate::connect::ConnectSubcommand;
use crate::control::ControlSubcommand;
use crate::generate::GenerateSubcommand;
use crate::watch::WatchSubcommand;

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
    /// generate manual or shell complete
    Generate(GenerateSubcommand),
    /// remote control ticar2
    Control(ControlSubcommand),
    /// watch ticar2 log
    Watch(WatchSubcommand),
}

impl Commands {
    pub async fn run(self) {
        match self {
            Commands::Connect(cmd) => cmd.run().await,
            Commands::Generate(cmd) => cmd.run().await,
            Commands::Control(cmd) => cmd.run().await,
            Commands::Watch(cmd) => cmd.run().await,
        }
    }
}
