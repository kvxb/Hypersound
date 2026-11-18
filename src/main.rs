mod app;
mod library;
mod player;
mod ui;

use app::App;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::path::PathBuf;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let music_dir = resolve_music_dir();
    if !music_dir.exists() {
        std::fs::create_dir_all(&music_dir)?;
    }

    eprintln!("Scanning {} …", music_dir.display());
    let library = library::scan(&music_dir);
    eprintln!("Found {} track(s).", library.len());

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new(library, music_dir);
    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result?;
    eprintln!("Bye ♪");
    Ok(())
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    let tick_rate = Duration::from_millis(100);
    while !app.should_quit {
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key);
            }
        }
        app.tick();
        terminal.draw(|frame| ui::render(frame, app))?;
    }
    Ok(())
}

/// Music folder resolution: CLI argument → env var → ~/Music.
fn resolve_music_dir() -> PathBuf {
    if let Some(arg) = std::env::args().nth(1) {
        return PathBuf::from(arg);
    }
    if let Ok(dir) = std::env::var("HYPERSOUND_MUSIC_DIR") {
        return PathBuf::from(dir);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join("Music")
}
