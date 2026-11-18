//! Thin wrapper around rodio's playback primitives.

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
use std::fs::File;
use std::path::Path;
use std::time::Duration;

struct Inner {
    // The device handle must stay alive for as long as we want sound.
    _device: MixerDeviceSink,
    player: Player,
}

/// Audio playback. If no output device is available the UI still works as a
/// library browser, and playback calls simply report an error.
pub struct MusicPlayer {
    inner: Option<Inner>,
    volume: f32,
}

impl MusicPlayer {
    pub fn new() -> Result<Self, String> {
        let mut device = DeviceSinkBuilder::open_default_sink()
            .map_err(|e| format!("could not open audio device: {e}"))?;
        device.log_on_drop(false);
        let player = Player::connect_new(device.mixer());
        Ok(Self {
            inner: Some(Inner {
                _device: device,
                player,
            }),
            volume: 0.7,
        })
    }

    pub fn unavailable() -> Self {
        Self {
            inner: None,
            volume: 0.7,
        }
    }

    /// Replace whatever is playing with the given file and start playback.
    pub fn play_file(&self, path: &Path) -> Result<(), String> {
        let Some(inner) = &self.inner else {
            return Err("audio device unavailable".to_string());
        };
        let file =
            File::open(path).map_err(|e| format!("cannot open '{}': {e}", path.display()))?;
        let decoder = Decoder::try_from(file)
            .map_err(|e| format!("cannot decode '{}': {e}", path.display()))?;
        inner.player.clear();
        inner.player.append(decoder);
        inner.player.play();
        Ok(())
    }

    pub fn pause(&self) {
        if let Some(inner) = &self.inner {
            inner.player.pause();
        }
    }

    pub fn resume(&self) {
        if let Some(inner) = &self.inner {
            inner.player.play();
        }
    }

    /// True when the playback queue has run dry (the track ended).
    pub fn finished(&self) -> bool {
        self.inner
            .as_ref()
            .map(|inner| inner.player.empty())
            .unwrap_or(false)
    }

    pub fn position(&self) -> Duration {
        self.inner
            .as_ref()
            .map(|inner| inner.player.get_pos())
            .unwrap_or(Duration::ZERO)
    }

    pub fn seek(&self, position: Duration) -> Result<(), String> {
        let Some(inner) = &self.inner else {
            return Err("audio device unavailable".to_string());
        };
        inner
            .player
            .try_seek(position)
            .map_err(|e| format!("seek failed: {e}"))
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        if let Some(inner) = &self.inner {
            inner.player.set_volume(self.volume);
        }
    }
}
