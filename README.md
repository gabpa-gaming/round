# Round

A modern, minimalist music player built with Rust and Dioxus, featuring a clean interface and powerful playlist management capabilities.

## Overview

Round is a desktop music player that prioritizes simplicity and functionality. It automatically organizes your music library by extracting metadata from audio files, manages playlists, and provides an intuitive browsing experience through your music collection.

## Features

### Core Features
- **Audio Playback**: Play MP3, FLAC, WAV, AAC, and OGG audio files
- **Music Library Management**: Automatic metadata extraction (artist, album, title, track numbers)
- **Album Art Support**: Displays embedded album art or searches for cover images in song directories
- **Smart File Browser**: Grid and list view modes that adapt based on folder content
- **Queue Management**: Customizable playback queue with play next/add to queue functionality

### Playlist Features
- **Create Custom Playlists**: Organize your favorite songs into playlists
- **Playlist Management**: Rename, delete, and manage playlist contents
- **Add Songs to Playlists**: Right-click context menu for quick playlist operations
- **Playlist Playback**: Play entire playlists with queue integration

### Playback Features
- **Playback Modes**:
  - Normal: Sequential playback
  - Shuffle: Intelligent randomization avoiding recently played tracks
  - Loop: Repeat current track
- **Playback Controls**: Play, pause, skip forward/backward, seek
- **Volume Control**: Adjustable volume slider
- **Progress Tracking**: Visual progress bar with time display
- **Queue System**: Play next queue with fallback to folder/playlist playback

### User Interface
- **Dark Theme**: Easy-on-the-eyes dark color scheme
- **Responsive Layout**: Three-column layout (file browser, now playing, sidebar)
- **Context Menus**: Right-click menus for files, folders, and playlists
- **Up Next Display**: See what's coming up in the queue
- **Animated Text**: Smooth scrolling for long song titles and metadata

## Prerequisites

Before compiling Round, ensure you have the following installed:

### Required
- **Rust** (1.70 or later) - Install from [rustup.rs](https://rustup.rs/)
- **Dioxus CLI** - Install with: `curl -sSL http://dioxus.dev/install.sh | sh`

### System Dependencies (Linux)

Round requires certain system libraries for desktop functionality. On **Fedora/RHEL-based systems**:

```bash
sudo dnf install gtk3-devel gdk-pixbuf2-devel cairo-devel pango-devel atk-devel
```

On **Debian/Ubuntu-based systems**:

```bash
sudo apt install libgtk-3-dev libgdk-pixbuf2.0-dev libcairo2-dev libpango1.0-dev libatk1.0-dev
```

On **Arch Linux**:

```bash
sudo pacman -S gtk3 cairo pango atk gdk-pixbuf2
```

### Audio Libraries
Round also requires ALSA development libraries (usually pre-installed on Linux):

```bash
# Fedora/RHEL
sudo dnf install alsa-lib-devel

# Debian/Ubuntu
sudo apt install libasound2-dev

# Arch
sudo pacman -S alsa-lib
```

## Building from Source

1. **Clone the repository** (or navigate to the project directory):
   ```bash
   cd round
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

   The compiled binary will be in `target/release/round`

## Running Round

### Development Mode

For development with hot-reloading:

```bash
dx serve
```

This will:
- Compile the application
- Launch the app
- Watch for file changes
- Auto-reload on code changes

### Production Binary

After building with `cargo build --release`:

```bash
./target/release/round
```

Or install it system-wide:

```bash
cargo install --path .
round
```

## Usage

### Getting Started

1. **Launch Round**: The app will start with your system's default music directory
2. **Browse Files**: Navigate through folders using the file browser
3. **Play Music**: Click on any audio file to start playback
4. **Create Playlists**: Right-click in the playlist section or on songs to create new playlists

### Playback Modes

Click the mode button (bottom right of now-playing sidebar):
- **→** Normal mode - Play sequentially
- **⚂** Shuffle mode - Random playback
- **↻** Loop mode - Repeat current track

## Data Storage

Round stores data in the following locations:

- **Database**: `~/.local/share/round/music_library.db`
- **Album Art Cache**: `~/.local/share/round/cover_*.jpg/png`

## Supported Audio Formats

- MP3 (.mp3)
- FLAC (.flac)
- WAV (.wav)
- AAC (.aac)
- OGG (.ogg)

### Adding Features

The codebase is modular and follows these patterns:
- Each UI component is a separate `#[component]` function
- State is managed through Dioxus Signals
- Database operations are centralized in `db.rs`
- Audio control is handled via async message passing

### Debugging

Enable verbose logging:
```bash
RUST_LOG=debug dx serve
```

### Contributing

When contributing:
1. Ensure code compiles without warnings
2. Test on Linux (primary platform)
3. Follow existing code style
4. Update documentation as needed

## License

This project is licensed under the terms specified by the author.

## Credits

Built with:
- [Dioxus](https://dioxuslabs.com/) - Rust UI framework
- [Rodio](https://github.com/RustAudio/rodio) - Audio playback
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite bindings
- [audiotags](https://github.com/TianyiShi2001/audiotags) - Audio metadata

