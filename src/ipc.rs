//! Single-instance control socket, so `gotodo quick` reaches a running app.
//! Only fixed commands travel over it; to-do content never does.

use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

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

/// Hands the command to a running instance. `false` means none is listening.
pub fn forward(command: Command) -> bool {
    match UnixStream::connect(socket_path()) {
        Ok(mut stream) => stream.write_all(&[command.as_byte()]).is_ok(),
        Err(_) => false,
    }
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
                if let Some(command) = Command::from_byte(byte[0]) {
                    let _ = output.send(command).await;
                }
            }
        }
    })
}
