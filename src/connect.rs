use clap::Args;
use std::net::IpAddr;

#[derive(Debug, Args)]
pub struct ConnectSubcommand {
    /// ticar2 ip to connect
    ip: IpAddr,
    /// ticar2 port to connect
    #[arg(short, long)]
    port: u16,
}

impl ConnectSubcommand {
    pub fn run(self) {
        println!("connect to {:?}", self);
    }
}
