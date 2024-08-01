use crate::watch::decorate_log;
use clap::Args;
use shi::{cmd, command::parent::ParentCommand, command::Command, leaf, shell::Shell};
use std::io::{BufRead, BufReader, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::sync::mpsc::channel;
use std::time;

enum ReplCommand {
    Fn { id: u8, describe: String },
    Set { id: u8, describe: String },
    Get { id: u8, describe: String },
}

#[derive(Debug, Args)]
pub struct ReplSubcommand {
    /// ticar2 ip to connect
    ip: IpAddr,
    /// ticar2 port to connect
    #[arg(short, long)]
    port: u16,
}

impl ReplSubcommand {
    pub async fn run(self) {
        let addr = SocketAddr::new(self.ip, self.port);
        let stream = TcpStream::connect_timeout(&addr, time::Duration::from_secs(1)).unwrap();

        let (tx, rx) = channel();

        let stream_clone = stream.try_clone().unwrap();
        std::thread::spawn(move || {
            let stream = BufReader::new(stream_clone);
            let mut commands = Vec::<ReplCommand>::new();

            let mut lines = stream.lines().map_while(Result::ok);
            while let Some(line) = lines.next() {
                decorate_log(&line);

                // expect I 00:00:00 RPC_REGISTER id=%d {FN|SET|GET} describe
                let mut split = line.split_ascii_whitespace();
                // I
                if split.next() != Some("I") {
                    continue;
                }
                // 00:00:00
                split.next();
                // RPC_REGISTER
                if split.next() != Some("RPC_REGISTER") {
                    if !commands.is_empty() {
                        break;
                    }
                    continue;
                }
                // id=%d
                let id: u8 = match split.next() {
                    Some(id) => match id.split_once('=') {
                        Some(("id", id)) => match id.parse() {
                            Ok(id) => id,
                            _ => continue,
                        },
                        _ => continue,
                    },
                    _ => continue,
                };
                // {FN|SET|GET}
                let cmd = match split.next() {
                    Some("FN") => ReplCommand::Fn {
                        id,
                        describe: split.collect::<Vec<&str>>().join(" "),
                    },
                    Some("SET") => ReplCommand::Set {
                        id,
                        describe: split.collect::<Vec<&str>>().join(" "),
                    },
                    Some("GET") => ReplCommand::Get {
                        id,
                        describe: split.collect::<Vec<&str>>().join(" "),
                    },
                    _ => continue,
                };
                commands.push(cmd);
            }
            tx.send(commands).unwrap();
            lines.for_each(|s| decorate_log(&s));
        });

        let commands = rx.recv().unwrap();

        let mut fun: Vec<(u8, String)> = Vec::new();
        let mut set: Vec<(u8, String)> = Vec::new();
        let mut get: Vec<(u8, String)> = Vec::new();
        for cmd in commands {
            match cmd {
                ReplCommand::Fn { id, describe } => fun.push((id, describe)),
                ReplCommand::Set { id, describe } => set.push((id, describe)),
                ReplCommand::Get { id, describe } => get.push((id, describe)),
            }
        }

        let stream_clone = stream.try_clone().unwrap();
        let mut shell = Shell::new_with_state("> ", stream_clone);
        shell
            .register(Command::Parent(ParentCommand::new_with_help(
                "fn",
                "call rpc function",
                fun.iter()
                    .map(|(id, describe)| {
                        let id = *id;
                        cmd!(describe, move |tx: &mut TcpStream, args| {
                            let arg = match args.first() {
                                Some(s) => s,
                                None => {
                                    let code = [0x7f, id, 0, 0];
                                    tx.write_all(&code).unwrap();
                                    return Ok("".to_string());
                                }
                            };

                            let code: [u8; 4] = match arg {
                                float if float.chars().filter(|&c| c == '.').count() == 1 => {
                                    let val = match float.parse::<f64>() {
                                        Ok(val) => val,
                                        _ => {
                                            return Ok(
                                                "Please enter a valid floating point number."
                                                    .to_string(),
                                            )
                                        }
                                    };
                                    let val: u16 = ((val * 100.0) as i16) as u16;
                                    [0x7f, id, (val >> 8) as u8, (val & 0xff) as u8]
                                }
                                int if int
                                    .chars()
                                    .filter(|&c| c != '-')
                                    .all(|c| c.is_ascii_digit()) =>
                                {
                                    let val = match int.parse::<i16>() {
                                        Ok(val) => val,
                                        _ => return Ok("Please enter a valid integer.".to_string()),
                                    };
                                    [0x7f, id, (val >> 8) as u8, (val & 0xff) as u8]
                                }
                                _ => return Ok("Need one num parameter.".to_string()),
                            };

                            tx.write_all(&code).unwrap();
                            Ok(String::new())
                        })
                    })
                    .collect(),
            )))
            .unwrap();
        shell
            .register(Command::Parent(ParentCommand::new_with_help(
                "get",
                "get remote variable",
                get.iter()
                    .map(|(id, describe)| {
                        let id = *id;
                        cmd!(describe, move |tx: &mut TcpStream, _| {
                            let code: [u8; 4] = [0x7f, id, 0, 0];
                            tx.write_all(&code).unwrap();
                            std::thread::sleep(time::Duration::from_millis(400));
                            Ok(String::new())
                        })
                    })
                    .collect(),
            )))
            .unwrap();
        shell
            .register(Command::Parent(ParentCommand::new_with_help(
                "set",
                "set remote variable",
                set.iter()
                    .map(|(id, describe)| {
                        let id = *id;
                        cmd!(describe, move |tx: &mut TcpStream, args| {
                            let arg = match args.first() {
                                Some(s) => s,
                                None => return Ok("Need one parameter.".to_string()),
                            };

                            let code: [u8; 4] = match arg {
                                float if float.chars().filter(|&c| c == '.').count() == 1 => {
                                    let val = match float.parse::<f64>() {
                                        Ok(val) => val,
                                        _ => {
                                            return Ok(
                                                "Please enter a valid floating point number."
                                                    .to_string(),
                                            )
                                        }
                                    };
                                    let val: u16 = ((val * 100.0) as i16) as u16;
                                    [0x7f, id, (val >> 8) as u8, (val & 0xff) as u8]
                                }
                                int if int
                                    .chars()
                                    .filter(|&c| c != '-')
                                    .all(|c| c.is_ascii_digit()) =>
                                {
                                    let val = match int.parse::<i16>() {
                                        Ok(val) => val,
                                        _ => return Ok("Please enter a valid integer.".to_string()),
                                    };
                                    [0x7f, id, (val >> 8) as u8, (val & 0xff) as u8]
                                }
                                _ => return Ok("Need one num parameter.".to_string()),
                            };

                            tx.write_all(&code).unwrap();
                            Ok(String::new())
                        })
                    })
                    .collect(),
            )))
            .unwrap();
        shell
            .register(leaf!(shi::command::EchoCommand::new()))
            .unwrap();

        shell.run().unwrap();
    }
}
