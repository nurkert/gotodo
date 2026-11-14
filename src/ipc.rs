//! Single-instance control socket, so `gotodo quick` reaches a running app.
//! Only fixed commands travel over it; to-do content never does. The one
//! reply is the status line for bars: counts, never titles.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use tokio::io::AsyncWriteExt;

use iced::futures::{SinkExt, Stream};
use tokio::io::AsyncReadExt;
use tokio::net::UnixListener;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Show,
    QuickEntry,
}

impl Command {
    fn as_byte(self) -> u8 {
        match self {
            Self::Show => b's',
            Self::QuickEntry => b'q',
        }
    }

    fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            b's' => Some(Self::Show),
            b'q' => Some(Self::QuickEntry),
            _ => None,
        }
    }
}

fn socket_path() -> PathBuf {
    let dir = std::env::var_os("XDG_RUNTIME_DIR").map_or_else(std::env::temp_dir, PathBuf::from);
    dir.join("gotodo.sock")
}

const STATUS_BYTE: u8 = b'c';

/// The latest status line, set by the app and read by the listener.
static STATUS: Mutex<String> = Mutex::new(String::new());

pub fn running() -> bool {
    UnixStream::connect(socket_path()).is_ok()
}

/// Hands the command to a running instance. `false` means none is listening.
pub fn forward(command: Command) -> bool {
    match UnixStream::connect(socket_path()) {
        Ok(mut stream) => stream.write_all(&[command.as_byte()]).is_ok(),
        Err(_) => false,
    }
}

/// Updates what `gotodo status` prints.
pub fn publish(line: String) {
    if let Ok(mut status) = STATUS.lock() {
        *status = line;
    }
}

/// Asks the running instance for its status line, in Waybar's JSON format.
pub fn status() -> String {
    let ask = || -> std::io::Result<String> {
        let mut stream = UnixStream::connect(socket_path())?;
        stream.set_read_timeout(Some(Duration::from_secs(1)))?;
        stream.write_all(&[STATUS_BYTE])?;
        let mut line = String::new();
        stream.read_to_string(&mut line)?;
        Ok(line)
    };
    match ask() {
        Ok(line) if !line.is_empty() => line,
        _ => status_line("", "gotodo is not running", "off"),
    }
}

pub fn status_line(text: &str, tooltip: &str, class: &str) -> String {
    serde_json::json!({ "text": text, "tooltip": tooltip, "class": class }).to_string()
}

pub fn listen() -> impl Stream<Item = Command> {
    iced::stream::channel(8, async |mut output: iced::futures::channel::mpsc::Sender<Command>| {
        let path = socket_path();
        let _ = std::fs::remove_file(&path);
        let Ok(listener) = UnixListener::bind(&path) else {
            return;
        };
        while let Ok((mut stream, _)) = listener.accept().await {
            let mut byte = [0u8; 1];
            if stream.read_exact(&mut byte).await.is_ok() {
                if byte[0] == STATUS_BYTE {
                    let line = STATUS.lock().map(|s| s.clone()).unwrap_or_default();
                    let _ = stream.write_all(line.as_bytes()).await;
                    let _ = stream.shutdown().await;
                    continue;
                }
                if let Some(command) = Command::from_byte(byte[0]) {
                    let _ = output.send(command).await;
                }
            }
        }
    })
}
