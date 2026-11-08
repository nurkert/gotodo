mod i18n;
mod ipc;
mod keepass;
mod model;
mod parse;
mod recurrence;
mod session;
mod ui;
mod vault;

use iced::{Color, theme};

use ui::App;

const USAGE: &str = "\
gotodo: a calm, encrypted to-do manager

USAGE:
    gotodo          open the app
    gotodo quick    open the Quick Entry popup (bind this to a key)
    gotodo --help   show this text
    gotodo --version

Sway example:
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
    let quick = match std::env::args().nth(1).as_deref() {
        None => false,
        Some("quick") => true,
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

    let command = if quick { ipc::Command::QuickEntry } else { ipc::Command::Show };
    if ipc::forward(command) {
        return Ok(());
    }

    iced::daemon(move || App::boot(quick), App::update, App::view)
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
