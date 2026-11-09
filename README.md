![gotodo](https://raw.githubusercontent.com/nurkert/gotodo/main/assets/gotodo-banner.png)

# gotodo

A calm, keyboard-first to-do manager for Linux, written in Rust. Your library
stays on this device and is encrypted at rest. It opens with a password or an
unlocked KeePassXC database. There is no cloud and no account.

## Install

Download the `.deb` for your architecture from the
[releases page](https://github.com/nurkert/gotodo/releases), then:

```sh
sudo apt install ./gotodo_0.0.1-1_amd64.deb
```

## Quick Entry and Sway

`gotodo quick` opens a small popup for capturing a to-do from anywhere. Bind it
to a key:

```
bindsym $mod+Shift+t exec gotodo quick
```

No `for_window` rule is needed: the popup has a fixed size, so Sway floats and
centers it instead of tiling it. Its `app_id` is `gotodo-quick` if you want to
style it. The main window's `app_id` is `gotodo`.

**GNOME.** Add a custom shortcut under Settings > Keyboard > Keyboard Shortcuts >
Custom Shortcuts with the command `gotodo quick`. GNOME does not let an app place
itself or stay on top, so the popup appears where the compositor puts it. The
shortcut also hands focus over only if GNOME grants it to the running app; if
typing does not start right away, click the popup once. The popup uses a
see-through window so it can fade in as one piece. If your renderer shows black
corners, start gotodo with `GOTODO_OPAQUE=1`.

Type the title and add details inline. Tab completes the suggestion, Enter
saves, Shift+Enter saves and stays open for the next one, Esc closes.

| Token | Meaning | Examples |
| --- | --- | --- |
| `#name` | project or area | `#haus` |
| `@name` | tag, created if new | `@errand` |
| `!when` | start date, with optional reminder time | `!tomorrow`, `!fri`, `!morgen-9:00`, `!someday`, `!3d` |
| `^when` | deadline | `^24.12.`, `^next-week` |

Use `-` for spaces inside a token.

## Shortcuts

Press F1 in the app for the full list. Super and Alt are never used, so they
stay free for your window manager.

| Keys | Action |
| --- | --- |
| Ctrl+N or Space | new to-do |
| Ctrl+Shift+M | move to a project or area |
| Ctrl+S | when |
| Ctrl+T / Ctrl+E | today / this evening |
| Ctrl+R / Ctrl+O | anytime / someday |
| Ctrl+Shift+D | deadline |
| Ctrl+Shift+T | tags |
| Ctrl+Shift+R | repeat |
| Ctrl+K | complete |
| Ctrl+F or just start typing | quick find |
| Ctrl+P | all commands |
| Ctrl+1 to Ctrl+7 | go to a list |
| Ctrl+Z / Ctrl+Shift+Z | undo / redo |
| Ctrl+L | lock |

## Security

The library is one file, `~/.local/share/gotodo/vault.gotodo`, with mode `0600`.

- A random 256-bit data key encrypts the whole library with XChaCha20-Poly1305.
  The header is authenticated too, and the body is padded to 4 KiB blocks.
- The password unlocks that key through Argon2id (128 MiB, 3 passes).
- Changing the password generates a new data key.
- Writes are atomic, so a crash leaves the previous or the new file intact.
- Only one process can hold the vault open.
- Core dumps and same-user `ptrace` are disabled, and the key is `mlock`ed when
  the system allows it.
- The app locks itself after 15 minutes without input, or with Ctrl+L.
- Reminders say only "A to-do is due" unless you enable titles in Settings.

**KeePassXC, in short.** gotodo keeps a random key inside your KeePassXC
database. While that database is unlocked, gotodo opens by itself, with no
password to type. Lock KeePassXC and gotodo locks too. A short guide in the app
(shown after you create a vault, and under Settings > Unlock with KeePassXC)
walks through the three steps and ticks them off as they happen. The details:

**KeePassXC setup.** Enable Tools > Settings > Secret Service Integration, expose a
group under Database Settings > Secret Service Integration, then choose Connect
KeePassXC in gotodo's settings. gotodo stores a random secret in that group, so
it can only read it while the database is unlocked. If KeePassXC locks or quits,
a session opened through it locks too. gotodo never triggers an unlock prompt,
and it refuses other Secret Service providers such as GNOME Keyring.

**Limits.** While unlocked, your to-dos are in memory in plain form. The toolkit
keeps its own copies of text, so memory cannot be fully wiped on lock. Use
encrypted swap if that matters to you. A lost password cannot be recovered. An
attacker who can replace the file could restore an older valid copy.

## Build

```sh
cargo test
cargo run --release
```

On Linux you need `pkg-config`, `libxkbcommon-dev`, `libwayland-dev` and
`libvulkan-dev`.

## Release

Pushing a tag that matches the version in `Cargo.toml` builds, tests and
publishes `amd64` and `arm64` packages:

```sh
git tag v0.0.1 && git push origin v0.0.1
```

Inter is licensed under the SIL Open Font License, see `assets/fonts/OFL.txt`.
