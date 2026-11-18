# Hypersound

A fast, minimal, offline music player for the terminal, built in Rust. Spotify-inspired
layout — library sidebar, sortable track list, and a now-playing bar — with a modern dark
theme and a green accent. No daemon, no web server, no cloud: it reads audio files straight
from a folder on your machine.

## Quick start

```bash
cargo build --release
./target/release/hypersound                 # uses ~/Music
./target/release/hypersound /path/to/music  # explicit folder
HYPERSOUND_MUSIC_DIR=/path/to/music ./target/release/hypersound
```

The folder is created if it doesn't exist. The library is scanned recursively at startup;
supported formats are `mp3`, `flac`, `ogg`, `m4a`, `mp4`, `aac`, `wav`, `opus`, `wma`,
`aiff`/`aif`. Titles, artists, albums and durations come from embedded tags (via `lofty`),
with the file name used as a fallback.

## Controls

| Key | Action |
| --- | --- |
| `↑` / `↓` or `k` / `j` | Move selection (sidebar or track list) |
| `Tab` | Switch focus between sidebar and track list |
| `Enter` | Play selected track / switch view in sidebar |
| `Space` | Play / pause |
| `n` / `p` | Next / previous track (`p` restarts the track if >3s in) |
| `←` / `→` or `h` / `l` | Seek back / forward 5 seconds |
| `+` / `-` | Volume up / down |
| `f` | Toggle favorite on the selected track |
| `/` | Search by title, artist or album |
| `q` / `Esc` / `Ctrl+C` | Quit |

Tracks auto-advance at the end of playback.

## Project layout

```
src/
  main.rs      terminal setup, startup scan, event/render loop
  app.rs       application state, key handling, playback orchestration
  library.rs   recursive folder scan + metadata reading (lofty)
  player.rs    audio playback wrapper (rodio)
  ui.rs        ratatui rendering: header / sidebar / track list / player bar
```

## Dependencies

- **ratatui** + **crossterm** — terminal UI and input
- **rodio** — audio playback (ALSA on Linux)
- **lofty** — audio metadata
- **walkdir** — recursive folder walking

## Scope / limitations

This is an intentionally small skeleton meant to be extended. Favorites are kept in memory
for the current session, there is no playlist file or settings file yet, and the sidebar
hosts simple views (All Songs, Recently Added, Favorites) rather than user-defined
playlists. Those are natural next steps.
