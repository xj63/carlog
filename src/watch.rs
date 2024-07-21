use clap::Args;
use console::style;
use std::net::{IpAddr, SocketAddr};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::time;

#[derive(Debug, Args)]
pub struct WatchSubcommand {
    /// ticar2 ip to connect
    ip: IpAddr,
    /// ticar2 port to connect
    #[arg(short, long)]
    port: u16,
}

fn decorate_log(log: &str) {
    let log = log.trim();
    match log.split_once(' ') {
        Some(("E", text)) => println!("{}", style(log).red()),
        Some(("W", text)) => println!("{}", style(log).yellow()),
        Some(("I", text)) => println!("{}", style(log).green()),
        Some(("D", text)) => println!("{}", log),
        Some(("T", text)) => println!("{}", log),
        _ => println!("{}", log),
    };
}

impl WatchSubcommand {
    pub async fn run(self) {
        let addr = SocketAddr::new(self.ip, self.port);
        let stream = time::timeout(time::Duration::from_secs(1), TcpStream::connect(addr))
            .await
            .expect("Connect timeout")
            .expect("Failed to tcp connect");
        let mut stream = BufReader::new(stream);
        let mut buf = String::new();

        loop {
            if stream.read_line(&mut buf).await.is_ok() {
                decorate_log(&buf);
            }
            buf.clear();
        }
    }
}
