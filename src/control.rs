use clap::Args;
use ratatui::{
    crossterm::{
        self,
        event::{self, KeyEvent},
    },
    prelude::{
        symbols, Alignment, Backend, Color, Constraint, CrosstermBackend, Frame, Layout, Line,
        Modifier, Rect, Span, Style, Terminal, Widget,
    },
    widgets::{block, Block, Gauge, LineGauge, List, ListItem, Paragraph},
    TerminalOptions, Viewport,
};
use std::net::{IpAddr, SocketAddr};
use std::thread;
use tokio::io::AsyncBufRead;
use tokio::io::{self, AsyncBufReadExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::time;
use tokio::time::Duration;

#[derive(Debug, Args)]
pub struct ControlSubcommand {
    /// ticar2 ip to connect
    ip: IpAddr,
    /// ticar2 port to connect
    #[arg(short, long)]
    port: u16,
}

#[derive(Clone, Copy, Debug)]
enum Control {
    Right(i16),
    Left(i16),
}

impl Control {
    fn encode(&self) -> [u8; 4] {
        match *self {
            Control::Left(speed) => [
                0xff,
                0x00,
                ((speed as u16 & 0xff00) >> 8) as u8,
                (speed & 0xff) as u8,
            ],
            Control::Right(speed) => [
                0xff,
                0x01,
                ((speed as u16 & 0xff00) >> 8) as u8,
                (speed & 0xff) as u8,
            ],
        }
    }
}

impl ControlSubcommand {
    pub async fn run(self) {
        let addr = SocketAddr::new(self.ip, self.port);
        let mut stream = time::timeout(time::Duration::from_secs(1), TcpStream::connect(addr))
            .await
            .expect("Connect timeout")
            .expect("Failed to tcp connect");

        let (mut rxtcp, mut txtcp) = stream.split();

        let (mut txstdout, mut rxstdout) = mpsc::channel::<String>(10);
        let (mut txremote, mut rxremote) = mpsc::channel::<Control>(10);

        let tcp2stdout = async move {
            let mut rxtcp = io::BufReader::new(rxtcp);
            let mut txstdout = txstdout.clone();

            loop {
                let mut buf = String::new();
                rxtcp.read_line(&mut buf).await;
                txstdout.send(buf).await;
            }
        };

        let rxremote2tcp = async move {
            loop {
                if let Some(control) = rxremote.recv().await {
                    let code = control.encode();
                    println!("{:?}", code);
                    txtcp.write_all(&code).await;
                }
            }
        };

        let terminal = console_init();

        console_log(rxstdout);

        phone_handling(txremote.clone());
        input_handling(txremote);
        tokio::join!(tcp2stdout, rxremote2tcp);

        console_restore(terminal);
    }
}

fn phone_handling(mut tx: mpsc::Sender<Control>) {
    use futures_util::{SinkExt, StreamExt};
    use serde::Deserialize;
    use tokio_tungstenite::connect_async;

    #[derive(Deserialize)]
    struct Sensor {
        values: [f64; 3],
    }

    impl Sensor {
        async fn control(self, tx: &mut mpsc::Sender<Control>) {
            let mut left: i16 = 0;
            let mut right: i16 = 0;

            let rot = self.values[1];

            left -= (rot * 10.0) as i16;
            right += (rot * 10.0) as i16;


            let forward = self.values[2].clamp(-50.0, 50.0);

            if forward.abs() <= 5.0 {
                tx.send(Control::Left(left)).await.unwrap();
                tx.send(Control::Right(right)).await.unwrap();
                return;
            }
            if forward.abs() >= 3.0 {
                left -= (forward * 30.0) as i16;
                right -= (forward * 30.0) as i16;
            }

            tx.send(Control::Left(left)).await.unwrap();
            tx.send(Control::Right(right)).await.unwrap();
        }
    }

    tokio::spawn(async move {
        let (mut ws, _) =
            connect_async("ws://192.168.4.3:8080/sensor/connect?type=android.sensor.orientation")
                .await
                .unwrap();

        while let Some(msg) = ws.next().await {
            let msg = msg.unwrap();
            if let Ok(msg) = serde_json::from_str::<Sensor>(&msg.to_string()) {
                msg.control(&mut tx).await;
            }
        }
    });
}

fn input_handling(tx: mpsc::Sender<Control>) {
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            loop {
                // poll for tick rate duration, if no events, sent tick event.
                match event::read().unwrap() {
                    event::Event::Key(key)
                        if key.code == event::KeyCode::Char('l')
                            && key.kind == event::KeyEventKind::Press =>
                    {
                        tx.send(Control::Left(1000)).await.unwrap();
                    }
                    event::Event::Key(key)
                        if key.code == event::KeyCode::Char('l')
                            && key.kind == event::KeyEventKind::Release =>
                    {
                        tx.send(Control::Left(0)).await.unwrap();
                    }
                    event::Event::Key(key)
                        if key.code == event::KeyCode::Char('h')
                            && key.kind == event::KeyEventKind::Press =>
                    {
                        tx.send(Control::Right(1000)).await.unwrap();
                    }
                    event::Event::Key(key)
                        if key.code == event::KeyCode::Char('h')
                            && key.kind == event::KeyEventKind::Release =>
                    {
                        tx.send(Control::Right(0)).await.unwrap();
                    }
                    event::Event::Key(key)
                        if key.code == event::KeyCode::Char('j')
                            && key.kind == event::KeyEventKind::Press =>
                    {
                        tx.send(Control::Right(-1000)).await.unwrap();
                        tx.send(Control::Left(-1000)).await.unwrap();
                    }
                    event::Event::Key(key)
                        if key.code == event::KeyCode::Char('j')
                            && key.kind == event::KeyEventKind::Release =>
                    {
                        tx.send(Control::Right(0)).await.unwrap();
                        tx.send(Control::Left(0)).await.unwrap();
                    }
                    event::Event::Key(key)
                        if key.code == event::KeyCode::Char('k')
                            && key.kind == event::KeyEventKind::Press =>
                    {
                        tx.send(Control::Right(1000)).await.unwrap();
                        tx.send(Control::Left(1000)).await.unwrap();
                    }
                    event::Event::Key(key)
                        if key.code == event::KeyCode::Char('k')
                            && key.kind == event::KeyEventKind::Release =>
                    {
                        tx.send(Control::Right(0)).await.unwrap();
                        tx.send(Control::Left(0)).await.unwrap();
                    }
                    _ => {
                        tx.send(Control::Left(0)).await.unwrap();
                        tx.send(Control::Right(0)).await.unwrap();
                    }
                };
            }
        })
    });
}

fn console_init() -> Terminal<CrosstermBackend<std::io::Stdout>> {
    crossterm::terminal::enable_raw_mode().unwrap();
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    Terminal::with_options(
        backend,
        TerminalOptions {
            // viewport: Viewport::Inline(8),
            viewport: Viewport::Inline(1),
        },
    )
    .unwrap()
}

fn console_log(mut log: Receiver<String>) {
    tokio::spawn(async move {
        while let Some(line) = log.recv().await {
            println!("{}", line.trim());
        }
    });
}

fn console_restore(mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>) {
    crossterm::terminal::disable_raw_mode().unwrap();
    terminal.clear().unwrap();
}
