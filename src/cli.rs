use clap::{Args, Parser, Subcommand};
use std::net::IpAddr;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// connect to ticar2, watch log and send message.
    Connect(Connect),
}

impl Commands {
    pub fn run(self) {
        match self {
            Commands::Connect(c) => c.run(),
        }
    }
}

#[derive(Debug, Args)]
pub struct Connect {
    /// ticar2 ip to connect
    ip: IpAddr,
    /// ticar2 port to connect
    #[arg(short, long)]
    port: u16,
}

impl Connect {
    pub fn run(self) {
        println!("connect to {}:{}", self.ip, self.port);
    }
}
