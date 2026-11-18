//! Scans the music folder and reads track metadata with `lofty`.

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::Accessor;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use walkdir::WalkDir;

/// A single playable file in the library.
#[derive(Clone, Debug)]
pub struct Track {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: Duration,
    /// File modification time, used by the "Recently Added" view.
    pub added: SystemTime,
    pub playlist: Option<String>,
}

/// File extensions we look for (case-insensitive).
const SUPPORTED: &[&str] = &[
    "mp3", "flac", "ogg", "oga", "m4a", "mp4", "aac", "wav", "opus", "wma", "aiff", "aif",
];

/// Recursively find and read all supported audio files under `dir`.
pub fn scan(dir: &Path) -> Vec<Track> {
    let mut tracks = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !SUPPORTED.contains(&ext.as_str()) {
            continue;
        }

        let added = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let (mut title, mut artist, mut album, mut duration) =
            (String::new(), String::new(), String::new(), Duration::ZERO);

        if let Ok(tagged) = lofty::read_from_path(path) {
            duration = tagged.properties().duration();
            if let Some(tag) = tagged.primary_tag().or_else(|| tagged.tags().first()) {
                title = tag.title().map(|c| c.into_owned()).unwrap_or_default();
                artist = tag.artist().map(|c| c.into_owned()).unwrap_or_default();
                album = tag.album().map(|c| c.into_owned()).unwrap_or_default();
            }
        }

        // Fall back to the file name when a file has no usable tags.
        if title.trim().is_empty() {
            title = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
        }
        if artist.trim().is_empty() {
            artist = "Unknown Artist".to_string();
        }
        if album.trim().is_empty() {
            album = "Unknown Album".to_string();
        }

        let playlist = path.strip_prefix(dir).ok().and_then(|rel| {
            let mut parts = rel.components();
            let first = parts.next()?;
            parts
                .next()
                .map(|_| first.as_os_str().to_string_lossy().into_owned())
        });

        tracks.push(Track {
            path: path.to_path_buf(),
            title,
            artist,
            album,
            duration,
            added,
            playlist,
        });
    }

    tracks.sort_by(|a, b| {
        a.title
            .to_lowercase()
            .cmp(&b.title.to_lowercase())
            .then_with(|| a.artist.to_lowercase().cmp(&b.artist.to_lowercase()))
    });
    tracks
}

pub fn discover_playlists(tracks: &[Track]) -> Vec<String> {
    let mut names: Vec<String> = tracks.iter().filter_map(|t| t.playlist.clone()).collect();
    names.sort();
    names.dedup();
    names
}

/// `m:ss` / `h:mm:ss` formatting for durations shown in the UI.
pub fn fmt_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs >= 3600 {
        format!("{}:{:02}:{:02}", secs / 3600, (secs / 60) % 60, secs % 60)
    } else {
        format!("{}:{:02}", secs / 60, secs % 60)
    }
}
