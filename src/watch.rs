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

pub fn decorate_log(log: &str) {
    let log = log.trim();
    let mut iter = log.split_ascii_whitespace();

    let level = match iter.next() {
        Some("E") => style("E").red().bold(),
        Some("W") => style("W").yellow().bold(),
        Some("I") => style("I").green().bold(),
        Some("D") => style("D").bold(),
        Some("T") => style("T").bold(),
        _ => {
            println!("{}", log);
            return;
        }
    };

    // time 00:01:10
    let time = match iter.next() {
        Some(time)
            if time
                .chars()
                .filter(|&c| c != ':')
                .all(|c| c.is_ascii_digit())
                && time.chars().filter(|&c| c == ':').count() == 2 =>
        {
            style(time).dim()
        }
        Some(info) => {
            println!(
                "{} {} {}",
                level,
                info,
                iter.collect::<Vec<&str>>().join(" ")
            );
            return;
        }
        None => {
            println!("{}", log);
            return;
        }
    };

    let span = match iter.next() {
        Some(span) if span.chars().filter(|&c| c == ':').count() == 1 => style(span).dim(),
        Some(types)
            if types
                .chars()
                .filter(|&c| c != '_')
                .all(|c| c.is_ascii_uppercase()) =>
        {
            println!(
                "{} {} {} {}",
                level,
                time,
                // use level style for types
                style(types).italic().bold(),
                iter.collect::<Vec<&str>>().join(" "),
            );
            return;
        }
        Some(info) => {
            println!(
                "{} {} {} {}",
                level,
                time,
                info,
                iter.collect::<Vec<&str>>().join(" "),
            );
            return;
        }
        _ => {
            println!("{}", log);
            return;
        }
    };

    if level.to_string() == "T" {
        let var = match iter.next() {
            Some(var) => var,
            None => {
                println!("{}", log);
                return;
            }
        };
        let (varname, varvalue) = match var.split_once("=") {
            Some((varname, varvalue)) => (varname, varvalue),
            None => {
                println!("{}", log);
                return;
            }
        };
        println!(
            "{} {} {} \t {}:{}",
            level,
            time,
            span,
            style(varname).italic(),
            style(varvalue).bold(),
        );
        return;
    }

    let types = match iter.next() {
        Some(types)
            if types
                .chars()
                .filter(|&c| c != '_')
                .all(|c| c.is_ascii_uppercase()) =>
        {
            style(types).italic().bold()
        }
        Some(info) => {
            println!(
                "{} {} {} {} {}",
                level,
                time,
                span,
                info,
                iter.collect::<Vec<&str>>().join(" ")
            );
            return;
        }
        _ => {
            println!("{}", log);
            return;
        }
    };

    println!(
        "{} {} {} {} {}",
        level,
        time,
        span,
        types,
        iter.collect::<Vec<&str>>().join(" ")
    );
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
