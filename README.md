# Harvux

A native Linux desktop client for [Harvest](https://www.getharvest.com/) time tracking, built with GTK4 and libadwaita.

![GPL-3.0](https://img.shields.io/badge/license-GPL--3.0--or--later-blue)

<img width="530" height="770" alt="image" src="https://github.com/user-attachments/assets/cb57bfd5-a501-4308-b0d9-8862c2415372" />


## Features

- **One-click timers** — start, stop, and restart time entries from your desktop
- **Project & task selection** — browse your Harvest workspace projects and tasks
- **Today's entries** — see all tracked time for the day with a running total
- **Quick restart** — click any previous entry to load it back into the timer
- **Inline notes** — add or edit notes on time entries as you work
- **Secure credentials** — API tokens stored in GNOME Keyring via the Secret Service API
- **Native GNOME look** — follows system theme, accent colors, and dark mode via libadwaita

## Screenshots

*Coming soon*

## Installation

### Flatpak (recommended)

```sh
# Build from source
flatpak-builder --force-clean --install --user build-dir com.github.samsonasu.Harvux.yml
```

> You'll need `cargo-sources.json` generated from the lockfile first. See [Building from source](#building-from-source) below.

### Building from source

**Dependencies:**

- Rust 1.80+
- GTK4 development libraries
- libadwaita development libraries

On Arch/Manjaro:

```sh
sudo pacman -S gtk4 libadwaita base-devel
```

On Fedora:

```sh
sudo dnf install gtk4-devel libadwaita-devel
```

On Ubuntu/Debian:

```sh
sudo apt install libgtk-4-dev libadwaita-1-dev
```

**Build and run:**

```sh
cargo build --release
./target/release/harvux
```

**For Flatpak builds**, generate the cargo sources file first:

```sh
pip install flatpak-cargo-generator
./build-aux/flatpak-cargo-generator.sh
flatpak-builder --force-clean build-dir com.github.samsonasu.Harvux.yml
```

## Setup

1. Open Harvux
2. Go to **Preferences** from the header bar menu
3. Get a personal access token from [id.getharvest.com/developers](https://id.getharvest.com/developers)
4. Enter your **Access Token** and **Account ID**
5. Click **Test Connection** to verify, then **Save**

Once saved, the app switches to the timer view and you're ready to track time.

## Usage

1. Select a **project** from the dropdown
2. Select a **task** for that project
3. Optionally add **notes**
4. Click **Start Timer** — the clock starts counting
5. Click **Stop Timer** when done

Your entry appears in the **Today** list at the bottom. Click any stopped entry to load it back into the timer for a quick restart.

## Tech Stack

- **Language:** Rust
- **UI:** GTK4 + libadwaita
- **HTTP:** reqwest with rustls-tls
- **Async:** Tokio runtime bridged to GLib main loop
- **Secrets:** oo7 (Secret Service / GNOME Keyring)
- **Packaging:** Flatpak (GNOME 47 runtime)

## License

GPL-3.0-or-later
