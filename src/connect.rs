use clap::Args;
use std::net::{IpAddr, SocketAddr};
use tokio::io;
use tokio::net::TcpStream;
use tokio::time;

#[derive(Debug, Args)]
pub struct ConnectSubcommand {
    /// ticar2 ip to connect
    ip: IpAddr,
    /// ticar2 port to connect
    #[arg(short, long)]
    port: u16,
}

impl ConnectSubcommand {
    pub async fn run(self) {
        let addr = SocketAddr::new(self.ip, self.port);
        let mut stream = time::timeout(time::Duration::from_secs(1), TcpStream::connect(addr))
            .await
            .expect("Connect timeout")
            .expect("Failed to tcp connect");

        let (mut reader, mut writer) = stream.split();

        let mut stdout = io::stdout();
        let mut stdin = io::stdin();

        let reader2stdout = io::copy(&mut reader, &mut stdout);
        let stdin2writer = io::copy(&mut stdin, &mut writer);

        tokio::try_join!(reader2stdout, stdin2writer).expect("Failed io and tcp retransmission.");
    }
}
