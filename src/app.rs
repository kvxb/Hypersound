//! Application state and all keyboard-driven behaviour.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::TableState;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use crate::library::Track;
use crate::player::MusicPlayer;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Sidebar,
    Library,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum View {
    AllSongs,
    Recent,
    Favorites,
}

impl View {
    pub const ALL: [View; 3] = [View::AllSongs, View::Recent, View::Favorites];

    pub fn label(self) -> &'static str {
        match self {
            View::AllSongs => "All Songs",
            View::Recent => "Recently Added",
            View::Favorites => "Favorites",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            View::AllSongs => "♫",
            View::Recent => "◷",
            View::Favorites => "♥",
        }
    }
}

pub struct App {
    pub library: Vec<Track>,
    pub music_dir: PathBuf,
    pub view: View,
    pub focus: Focus,
    pub sidebar_index: usize,
    pub selected: usize,
    pub current: Option<usize>,
    pub playing: bool,
    pub favorites: HashSet<PathBuf>,
    pub search: String,
    pub searching: bool,
    /// Indices into `library`, in the order shown for the current view/search.
    pub visible: Vec<usize>,
    pub table_state: TableState,
    pub status: String,
    pub should_quit: bool,
    player: MusicPlayer,
}

impl App {
    pub fn new(library: Vec<Track>, music_dir: PathBuf) -> Self {
        // Audio devices can be missing (headless machine / misconfigured ALSA).
        // The UI still works as a library browser in that case.
        let (player, status) = match MusicPlayer::new() {
            Ok(p) => (p, String::new()),
            Err(e) => {
                // A placeholder would need a device too, so we surface the
                // error in the status bar and let playback attempts report it.
                (MusicPlayer::unavailable(), e)
            }
        };

        let mut app = Self {
            library,
            music_dir,
            view: View::AllSongs,
            focus: Focus::Library,
            sidebar_index: 0,
            selected: 0,
            current: None,
            playing: false,
            favorites: HashSet::new(),
            search: String::new(),
            searching: false,
            visible: Vec::new(),
            table_state: TableState::default(),
            status,
            should_quit: false,
            player,
        };
        app.rebuild_visible();
        app
    }

    // ------------------------------------------------------------------
    // List / selection helpers
    // ------------------------------------------------------------------

    fn rebuild_visible(&mut self) {
        let query = self.search.trim().to_lowercase();
        let favorites = &self.favorites;

        self.visible = (0..self.library.len())
            .filter(|&i| {
                let track = &self.library[i];
                match self.view {
                    View::Favorites => favorites.contains(&track.path),
                    _ => true,
                }
            })
            .filter(|&i| {
                if query.is_empty() {
                    return true;
                }
                let track = &self.library[i];
                track.title.to_lowercase().contains(&query)
                    || track.artist.to_lowercase().contains(&query)
                    || track.album.to_lowercase().contains(&query)
            })
            .collect();

        if self.view == View::Recent {
            self.visible
                .sort_by(|&a, &b| self.library[b].added.cmp(&self.library[a].added));
        }

        if self.selected >= self.visible.len() {
            self.selected = self.visible.len().saturating_sub(1);
        }
        self.table_state.select(if self.visible.is_empty() {
            None
        } else {
            Some(self.selected)
        });
    }

    pub fn selected_track(&self) -> Option<&Track> {
        self.visible.get(self.selected).map(|&i| &self.library[i])
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.current.map(|i| &self.library[i])
    }

    pub fn progress(&self) -> Duration {
        self.player.position()
    }

    pub fn volume(&self) -> f32 {
        self.player.volume()
    }

    fn move_selection(&mut self, delta: isize) {
        if self.visible.is_empty() {
            return;
        }
        let len = self.visible.len() as isize;
        self.selected = (self.selected as isize + delta).rem_euclid(len) as usize;
        self.table_state.select(Some(self.selected));
    }

    fn move_sidebar(&mut self, delta: isize) {
        let len = View::ALL.len() as isize;
        self.sidebar_index = (self.sidebar_index as isize + delta).rem_euclid(len) as usize;
        self.view = View::ALL[self.sidebar_index];
        self.selected = 0;
        self.rebuild_visible();
    }

    // ------------------------------------------------------------------
    // Playback
    // ------------------------------------------------------------------

    fn play_index(&mut self, index: usize) {
        match self.player.play_file(&self.library[index].path) {
            Ok(()) => {
                self.current = Some(index);
                self.playing = true;
                self.status.clear();
            }
            Err(e) => {
                self.playing = false;
                self.status = e;
            }
        }
    }

    fn play_selected(&mut self) {
        if let Some(&index) = self.visible.get(self.selected) {
            self.play_index(index);
        }
    }

    pub fn toggle_pause(&mut self) {
        if self.current.is_none() {
            self.play_selected();
            return;
        }
        if self.playing {
            self.player.pause();
            self.playing = false;
        } else {
            self.player.resume();
            self.playing = true;
        }
    }

    pub fn next(&mut self) {
        if self.visible.is_empty() {
            return;
        }
        let pos = self
            .current
            .and_then(|c| self.visible.iter().position(|&i| i == c))
            .unwrap_or(self.selected);
        let next = (pos + 1) % self.visible.len();
        self.selected = next;
        self.play_index(self.visible[next]);
    }

    fn previous(&mut self) {
        if self.visible.is_empty() {
            return;
        }
        let pos = self
            .current
            .and_then(|c| self.visible.iter().position(|&i| i == c))
            .unwrap_or(self.selected);
        let prev = if pos == 0 {
            self.visible.len() - 1
        } else {
            pos - 1
        };
        self.selected = prev;
        self.play_index(self.visible[prev]);
    }

    fn seek_relative(&mut self, seconds: i64) {
        let pos = self.player.position();
        let amount = Duration::from_secs(seconds.unsigned_abs());
        let target = if seconds < 0 {
            pos.saturating_sub(amount)
        } else {
            pos.saturating_add(amount)
        };
        // Stay a second shy of the end so we don't immediately auto-advance.
        let end = self
            .current_track()
            .map(|t| t.duration)
            .unwrap_or(Duration::MAX)
            .saturating_sub(Duration::from_secs(1));
        match self.player.seek(target.min(end)) {
            Ok(()) => self.status.clear(),
            Err(e) => self.status = e,
        }
    }

    fn restart_or_previous(&mut self) {
        if self.progress() > Duration::from_secs(3) {
            let _ = self.player.seek(Duration::ZERO);
        } else {
            self.previous();
        }
    }

    fn toggle_favorite(&mut self) {
        let Some(track) = self.selected_track() else {
            return;
        };
        let path = track.path.clone();
        if !self.favorites.remove(&path) {
            self.favorites.insert(path);
        }
        if self.view == View::Favorites {
            self.rebuild_visible();
        }
    }

    fn change_volume(&mut self, delta: f32) {
        let vol = (self.player.volume() + delta).clamp(0.0, 1.0);
        self.player.set_volume(vol);
    }

    // ------------------------------------------------------------------
    // Main loop
    // ------------------------------------------------------------------

    /// Called every frame (~100 ms) to drive auto-advance.
    pub fn tick(&mut self) {
        if self.playing
            && self.current.is_some()
            && self.progress() > Duration::from_millis(250)
            && self.player.finished()
        {
            self.next();
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != crossterm::event::KeyEventKind::Press {
            return;
        }

        if self.searching {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.searching = false;
                }
                KeyCode::Backspace => {
                    self.search.pop();
                    self.selected = 0;
                    self.rebuild_visible();
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.search.push(c);
                    self.selected = 0;
                    self.rebuild_visible();
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true
            }

            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::Sidebar => Focus::Library,
                    Focus::Library => Focus::Sidebar,
                };
            }

            KeyCode::Up | KeyCode::Char('k') => match self.focus {
                Focus::Sidebar => self.move_sidebar(-1),
                Focus::Library => self.move_selection(-1),
            },
            KeyCode::Down | KeyCode::Char('j') => match self.focus {
                Focus::Sidebar => self.move_sidebar(1),
                Focus::Library => self.move_selection(1),
            },

            KeyCode::Enter => match self.focus {
                Focus::Sidebar => self.focus = Focus::Library,
                Focus::Library => self.play_selected(),
            },

            KeyCode::Char(' ') => self.toggle_pause(),
            KeyCode::Char('n') => self.next(),
            KeyCode::Char('p') => self.restart_or_previous(),
            KeyCode::Right | KeyCode::Char('l') => self.seek_relative(5),
            KeyCode::Left | KeyCode::Char('h') => self.seek_relative(-5),

            KeyCode::Char('+') | KeyCode::Char('=') => self.change_volume(0.05),
            KeyCode::Char('-') | KeyCode::Char('_') => self.change_volume(-0.05),

            KeyCode::Char('f') => self.toggle_favorite(),
            KeyCode::Char('/') => {
                self.searching = true;
                self.search.clear();
                self.selected = 0;
                self.rebuild_visible();
            }

            _ => {}
        }
    }
}
