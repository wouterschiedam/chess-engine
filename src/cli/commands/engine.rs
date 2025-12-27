use crate::board::Board;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

pub struct UciEngineProcess {
    path: String,
    child: Option<Child>,
    stdin: Option<std::process::ChildStdin>,
    reader: Option<BufReader<std::process::ChildStdout>>,
}

impl UciEngineProcess {
    pub fn new(path: String) -> Self {
        Self {
            path,
            child: None,
            stdin: None,
            reader: None,
        }
    }

    pub fn ensure_started(&mut self) -> Option<()> {
        if self.child.is_none() {
            let mut child = Command::new(&self.path)
                .arg("uci")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .ok()?;

            let mut stdin = child.stdin.take()?;
            let stdout = child.stdout.take()?;
            let mut reader = BufReader::new(stdout);

            // 1. Send 'uci' and wait for 'uciok'
            writeln!(stdin, "uci").ok();
            stdin.flush().ok();

            let mut line = String::new();
            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                if line.contains("uciok") {
                    break;
                }
                line.clear();
            }

            // 2. Send 'isready' and wait for 'readyok'
            writeln!(stdin, "isready").ok();
            stdin.flush().ok();

            line.clear();
            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                if line.contains("readyok") {
                    break;
                }
                line.clear();
            }

            // 3. Set starting position
            writeln!(stdin, "position startpos").ok();
            stdin.flush().ok();

            self.stdin = Some(stdin);
            self.reader = Some(reader);
            self.child = Some(child);
        }
        Some(())
    }

    pub fn get_move(&mut self, board: &Board, depth: u8) -> Option<String> {
        self.ensure_started()?;

        let stdin = self.stdin.as_mut()?;
        
        // Update position
        writeln!(stdin, "position fen {}", board.to_fen()).ok();
        
        // Start search
        writeln!(stdin, "go depth {}", depth).ok();
        stdin.flush().ok();

        let reader = self.reader.as_mut()?;
        let mut line = String::new();
        while reader.read_line(&mut line).unwrap_or(0) > 0 {
            let trimmed = line.trim();
            if trimmed.starts_with("bestmove") {
                let mv = trimmed.split_whitespace().nth(1).map(|s| s.to_string());
                return mv;
            }
            line.clear();
        }

        self.stop();
        None
    }

    pub fn stop(&mut self) {
        self.stdin.take();
        self.reader.take();
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
        }
    }
}
