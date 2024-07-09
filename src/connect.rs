use clap::Args;
use std::net::IpAddr;

#[derive(Debug, Args)]
pub struct Connect {
    /// ticar2 named in carlog.toml
    device: Option<String>,
    /// ticar2 ip to connect
    #[arg(short, long)]
    ip: Option<IpAddr>,
    /// ticar2 port to connect
    #[arg(short, long)]
    port: Option<u16>,
}

impl Connect {
    pub fn run(self) {
        println!("connect to {:?}", self);
    }
}
