use crate::watch::decorate_log;
use clap::Args;
use console::{Key, Term};
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::time;

const SPEED_THETA: i16 = 123;
const SPEED_FORWARD: i16 = 200;
const CODE_MOVE_ZERO_THETA: [u8; 4] = [0x7f, 0x01, 0, 0];
const CODE_MOVE_ZERO_FORWARD: [u8; 4] = [0x7f, 0x00, 0, 0];
const CODE_MOVE_LEFT: [u8; 4] = [0x7f, 0x01, (-SPEED_THETA >> 8) as u8, -SPEED_THETA as u8];
const CODE_MOVE_RIGHT: [u8; 4] = [0x7f, 0x01, (SPEED_THETA >> 8) as u8, SPEED_THETA as u8];
const CODE_MOVE_FORWARD: [u8; 4] = [0x7f, 0x00, (SPEED_FORWARD >> 8) as u8, SPEED_FORWARD as u8];
const CODE_MOVE_BACK: [u8; 4] = [
    0x7f,
    0x00,
    (-SPEED_FORWARD >> 8) as u8,
    -SPEED_FORWARD as u8,
];

#[derive(Debug, Args)]
pub struct ControlSubcommand {
    /// ticar2 ip to connect
    ip: IpAddr,
    /// ticar2 port to connect
    #[arg(short, long)]
    port: u16,
}

impl ControlSubcommand {
    pub async fn run(self) {
        let addr = SocketAddr::new(self.ip, self.port);
        let mut stream = TcpStream::connect_timeout(&addr, time::Duration::from_secs(1)).unwrap();

        let stream_clone = stream.try_clone().unwrap();
        std::thread::spawn(|| {
            let stream = BufReader::new(stream_clone);
            stream
                .lines()
                .map_while(Result::ok)
                .for_each(|s| decorate_log(&s));
        });

        let term = Term::stdout();
        loop {
            let key = term.read_key().unwrap();
            let mut buf = Vec::with_capacity(8);

            match key {
                Key::ArrowLeft | Key::Char('a') => {
                    buf.write_all(&CODE_MOVE_LEFT).unwrap();
                    buf.write_all(&CODE_MOVE_ZERO_FORWARD).unwrap();
                }
                Key::ArrowRight | Key::Char('d') => {
                    buf.write_all(&CODE_MOVE_RIGHT).unwrap();
                    buf.write_all(&CODE_MOVE_ZERO_FORWARD).unwrap();
                }
                Key::ArrowUp | Key::Char('w') | Key::Char('W') | Key::Shift => {
                    buf.write_all(&CODE_MOVE_ZERO_THETA).unwrap();
                    buf.write_all(&CODE_MOVE_FORWARD).unwrap()
                }
                Key::ArrowDown | Key::Char('s') | Key::Char('S') => {
                    buf.write_all(&CODE_MOVE_ZERO_THETA).unwrap();
                    buf.write_all(&CODE_MOVE_BACK).unwrap()
                }
                Key::Char('A') => {
                    buf.write_all(&CODE_MOVE_LEFT).unwrap();
                    buf.write_all(&CODE_MOVE_FORWARD).unwrap();
                }
                Key::Char('D') => {
                    buf.write_all(&CODE_MOVE_RIGHT).unwrap();
                    buf.write_all(&CODE_MOVE_FORWARD).unwrap();
                }
                Key::Char(' ') => {
                    buf.write_all(&CODE_MOVE_ZERO_FORWARD).unwrap();
                    buf.write_all(&CODE_MOVE_ZERO_THETA).unwrap();
                }
                _ => continue,
            }

            stream.write_all(&buf).unwrap();
        }
    }
}
