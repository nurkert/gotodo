mod backup;
mod device;
mod i18n;
mod ipc;
mod keepass;
mod model;
mod notify;
mod parse;
mod paths;
mod prefs;
mod recurrence;
mod session;
mod ui;
mod vault;

use iced::{Color, theme};

use ui::{App, Start};

const USAGE: &str = "\
gotodo: a calm, encrypted to-do manager

USAGE:
    gotodo               open the app
    gotodo quick         open the Quick Entry popup; run it again to dismiss it
                         (bind this to a key)
    gotodo --background  start without a window and keep running, so Quick
                         Entry opens at once (start this with your session)
    gotodo status        print counts for a status bar such as Waybar
    gotodo --help        show this text
    gotodo --version

Sway example:
    exec gotodo --background
    bindsym $mod+Shift+t exec gotodo quick
";

/// Keeps decrypted memory out of core dumps and away from same-user ptrace.
fn harden_process() {
    let no_core = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
    unsafe {
        libc::setrlimit(libc::RLIMIT_CORE, &no_core);
        #[cfg(target_os = "linux")]
        libc::prctl(libc::PR_SET_DUMPABLE, 0);
    }
}

fn main() -> iced::Result {
    harden_process();
    let start = match std::env::args().nth(1).as_deref() {
        None => Start::Main,
        Some("quick") => Start::Quick,
        Some("--background") => Start::Background,
        Some("status") => {
            println!("{}", ipc::status());
            return Ok(());
        }
        Some("--version" | "-V") => {
            println!("gotodo {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some("--help" | "-h") => {
            print!("{USAGE}");
            return Ok(());
        }
        Some(other) => {
            eprintln!("unknown argument: {other}\n\n{USAGE}");
            std::process::exit(2);
        }
    };

    let command = match start {
        Start::Main => Some(ipc::Command::Show),
        Start::Quick => Some(ipc::Command::QuickEntry),
        // A second background start has nothing to do.
        Start::Background => None,
    };
    if ipc::running() {
        if let Some(command) = command {
            ipc::forward(command);
        }
        return Ok(());
    }

    iced::daemon(move || App::boot(start), App::update, App::view)
        .title(App::window_title)
        .theme(App::theme)
        .scale_factor(App::scale)
        .style(|_, theme: &iced::Theme| theme::Style {
            background_color: Color::TRANSPARENT,
            text_color: theme.palette().text,
        })
        .subscription(App::subscription)
        .font(include_bytes!("../assets/fonts/Inter-Regular.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/Inter-Medium.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/Inter-SemiBold.ttf").as_slice())
        .default_font(ui::default_font())
        .antialiasing(true)
        .run()
}
