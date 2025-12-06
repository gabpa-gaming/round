# Round

A modern, minimalist music player built with Rust and Dioxus, featuring a clean interface and powerful playlist management capabilities.

## Overview

Round is a desktop music player that prioritizes simplicity and functionality. It automatically organizes your music library by extracting metadata from audio files, manages playlists, and provides an intuitive browsing experience through your music collection.

## Features

### Core Features
- **Audio Playback**: Play MP3, FLAC, WAV, AAC, and OGG audio files
- **Music Library Management**: Automatic metadata extraction (artist, album, title, track numbers)
- **Album Art Support**: Displays album art or searches for cover images in song directories
- **File Browser**: Grid and list view modes that adapt based on folder content
- **Queue Management**: Customizable playback queue with play next/add to queue functionality

### Playlist Features
- **Create Custom Playlists**: Organize your favorite songs into playlists
- **Playlist Management**: Rename, delete, and manage playlist contents
- **Add Songs to Playlists**: Right-click context menu for playlist operations
- **Playlist Playback**: Play entire playlists with queue

### Playback Features
- **Playback Modes**:
  - Normal: Sequential playback
  - Shuffle: Intelligent randomization avoiding recently played tracks
  - Loop: Repeat current track
- **Playback Controls**: Play, pause, skip forward/backward, seek
- **Volume Control**: Adjustable volume slider
- **Progress Tracking**: Visual progress bar with time display
- **Queue System**: Play next queue with fallback to folder/playlist playback
- 
## Prerequisites
### Required
- **Rust** (1.70 or later) - Install from [rustup.rs](https://rustup.rs/)
- **Dioxus CLI** - Install with: `curl -sSL http://dioxus.dev/install.sh | sh`

## Building from Source

1. **Clone the repository** (or navigate to the project directory):
   ```bash
   cd round
   ```

2. **Build the project**:
   ```bash
   dx bundle 
   ```
   (Refer to https://dioxuslabs.com/learn/0.7/tutorial/bundle/)

## Running Round

### Development Mode

For development with hot-reloading:

```bash
dx serve
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

## License

This project's code is licensed under the GNU Public License v3

## Credits

Built with:
- [Dioxus](https://dioxuslabs.com/) - Rust UI framework
- [Rodio](https://github.com/RustAudio/rodio) - Audio playback
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite bindings
- [audiotags](https://github.com/TianyiShi2001/audiotags) - Audio metadata
- [Icon](https://www.iconarchive.com/show/christmas-icons-by-samborek/speaker-icon.html) - Samborek
