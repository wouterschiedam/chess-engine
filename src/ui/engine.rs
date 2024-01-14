use std::process::Stdio;
use std::time::Duration;

use iced::futures::channel::mpsc;
use iced::keyboard::KeyCode;
use iced::{subscription, Subscription};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdout, Command};
use tokio::sync::mpsc::Receiver;
use tokio::time::{self, timeout};

use super::ui::Message;

#[derive(Debug, PartialEq)]
pub enum EngineStatus {
    TurnedOff,
    TurnedOn,
}

pub enum EngineState {
    Start(UIengine),
    Thinking(Child, String, Receiver<String>),
    TurnedOff,
}

#[derive(Debug, Clone)]
pub struct UIengine {
    pub engine_path: String,
    pub search_up_to: String,
    pub position: String,
}

impl UIengine {
    pub fn new() -> Self {
        Self {
            engine_path: String::from("./target/debug/chess-engine"),
            search_up_to: "3".to_string(),
            position: String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        }
    }

    pub fn run_engine(engine: UIengine) -> Subscription<Message> {
        subscription::channel(
            std::any::TypeId::of::<UIengine>(),
            100,
            move |mut output| {
                let engine = engine.clone();
                async move {
                    let mut state = EngineState::Start(engine);

                    loop {
                        match &mut state {
                            EngineState::Start(engine) => {
                                // Create mspc channel with sender and receiver
                                let (sender, receiver): (
                                    tokio::sync::mpsc::Sender<String>,
                                    tokio::sync::mpsc::Receiver<String>,
                                ) = tokio::sync::mpsc::channel(100);

                                // Create new command to start the engine
                                let mut cmd = Command::new(&engine.engine_path);

                                cmd.kill_on_drop(true)
                                    .stdin(Stdio::piped())
                                    .stdout(Stdio::piped());

                                // Start chess engine
                                let mut process = cmd.spawn().expect("Error starting engine");

                                // Some local variables used to start communication with the engine.
                                let pos = String::from("position fen ")
                                    + &engine.position
                                    + &String::from("\n");
                                let limit = &engine.search_up_to;

                                // Write to stdin asynchronously
                                if let Some(mut stdin) = process.stdin.take() {
                                    // You can use write_all to write a slice of bytes asynchronously
                                    stdin
                                        .write_all(b"uci\n")
                                        .await
                                        .expect("Error writing to stdin");

                                    // Optionally, flush the buffer
                                    stdin.flush().await.expect("Error flushing stdin");
                                }

                                let mut reader = BufReader::new(
                                    process.stdout.take().expect("Failed to get stdout"),
                                );

                                let mut buffer_str = String::new();

                                // read input from engine
                                let uciok =
                                    read_input_from_process(&mut reader, &mut buffer_str).await;

                                if uciok {
                                    if let Some(mut stdin) = process.stdin.take() {
                                        // You can use write_all to write a slice of bytes asynchronously
                                        stdin
                                            .write_all(b"isready\n")
                                            .await
                                            .expect("Error writing to stdin");
                                        // Optionally, flush the buffer
                                        stdin.flush().await.expect("Error flushing stdin");
                                    }

                                    let mut buffer_str = String::new();

                                    let readyok =
                                        read_input_from_process(&mut reader, &mut buffer_str).await;

                                    println!("{readyok}");

                                    if readyok {
                                        output.try_send(Message::EngineReady(sender)).expect(
                                            "Error on the mpsc channel in the engine subscription",
                                        );

                                        state = EngineState::Thinking(
                                            process,
                                            engine.search_up_to.to_string(),
                                            receiver,
                                        );

                                        println!("Engine started");

                                        continue;
                                    }
                                }

                                // Send quit command to engine
                                if let Some(mut stdin) = process.stdin.take() {
                                    stdin
                                        .write_all(b"quit\n")
                                        .await
                                        .expect("Error stopping the engine");
                                }

                                eprintln!("Engine took too long to start, aborting...");
                                let terminate_timeout =
                                    timeout(Duration::from_millis(1000), process.wait()).await;
                                if let Err(_) = terminate_timeout {
                                    eprintln!("Engine didn't quit, killing the process now...");
                                    let kill_result =
                                        timeout(Duration::from_millis(500), process.kill()).await;
                                    if let Err(e) = kill_result {
                                        eprintln!("Error killing the engine process: {e}");
                                    }
                                    eprintln!("Engine stopped");
                                }

                                output.try_send(Message::EngineStopped(false)).expect(
                                    "Error in the mspc channel in the subscription channel",
                                );
                                state = EngineState::TurnedOff;
                            }
                            EngineState::Thinking(process, search_up_to, receiver) => {
                                let message = receiver.try_recv();

                                if let Ok(message) = message {
                                    if &message == "stop" || &message == "quit" {
                                        // Send quit command to engine
                                        if let Some(mut stdin) = process.stdin.take() {
                                            stdin
                                                .write_all(b"quit\n")
                                                .await
                                                .expect("Error stopping the engine");

                                            stdin.flush().await.expect("Error flushing stdin");
                                        }

                                        // kill process
                                        let terminate_timeout =
                                            timeout(Duration::from_millis(1000), process.wait())
                                                .await;
                                        if let Err(_) = terminate_timeout {
                                            eprintln!(
                                                "Engine didn't quit, killing the process now..."
                                            );
                                            let kill_result =
                                                timeout(Duration::from_millis(500), process.kill())
                                                    .await;
                                            if let Err(e) = kill_result {
                                                eprintln!("Error killing the engine process: {e}");
                                            }
                                        }

                                        output.try_send(Message::EngineStopped(true)).expect(
                                            "Error on the mspc channel in the engine subscription",
                                        );
                                        state = EngineState::TurnedOff;
                                        continue;
                                    } else {
                                        let pos = String::from("position fen ")
                                            + &message
                                            + &String::from("\n");

                                        let limit = String::from("go depth ")
                                            + &search_up_to
                                            + &String::from("\n");

                                        if let Some(mut stdin) = process.stdin.take() {
                                            stdin
                                                .write_all(pos.as_bytes())
                                                .await
                                                .expect("Error communicating with the engine");
                                            stdin
                                                .write_all(limit.as_bytes())
                                                .await
                                                .expect("Error communicating with the engine");

                                            stdin.flush().await.expect("Error flushing stdin");
                                        }
                                    }
                                }

                                let mut buffer_str = String::new();
                                let mut eval: Option<KeyCode> = None;
                                let mut bestmove: Option<KeyCode> = None;

                                if let Some(out) = process.stdout.as_mut() {
                                    let mut reader = BufReader::new(
                                        process.stdout.take().expect("Failed to get stdout"),
                                    );
                                    loop {
                                        let read_line_result = async {
                                            reader
                                                .read_line(&mut buffer_str)
                                                .await
                                                .map(|_| buffer_str.clone())
                                        };

                                        match time::timeout(
                                            Duration::from_millis(7000),
                                            read_line_result,
                                        )
                                        .await
                                        {
                                            Ok(Ok(line)) => {
                                                let vector: Vec<&str> =
                                                    line.split_whitespace().collect();

                                                println!("{:?}", vector);
                                            }
                                            Ok(Err(e)) => {
                                                eprintln!("Error reading line: {:?}", e);
                                                break;
                                            }
                                            Err(_) => {
                                                eprintln!("Time out occurred");
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            EngineState::TurnedOff => {
                                tokio::time::sleep(std::time::Duration::from_millis(10)).await
                            }
                        }
                    }
                }
            },
        )
    }
}

pub async fn read_input_from_process(
    reader: &mut BufReader<ChildStdout>,
    mut buffer_str: &mut String,
) -> bool {
    loop {
        let read_line_result = async {
            reader
                .read_line(&mut buffer_str)
                .await
                .map(|_| buffer_str.clone())
        };

        match time::timeout(Duration::from_millis(3000), read_line_result).await {
            Ok(Ok(line)) => {
                if line.contains("uciok") || line.contains("readyok") {
                    return true;
                }
            }
            Ok(Err(e)) => {
                eprintln!("Error reading line: {:?}", e);
                break false;
            }
            Err(_) => {
                eprintln!("Timeout occurred");
                break false;
            }
        }
    }
}
