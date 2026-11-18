//! Ratatui rendering. Layout: header / library sidebar + track list / player.

use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Gauge, Paragraph, Row, Table},
    Frame,
};
use std::time::Duration;

use crate::app::{App, Focus, SidebarEntry, View};
use crate::library::fmt_duration;

mod theme {
    use ratatui::style::Color;

    pub const ACCENT: Color = Color::Rgb(30, 215, 96);
    pub const TEXT: Color = Color::Rgb(235, 235, 235);
    pub const MUTED: Color = Color::Rgb(130, 130, 130);
    pub const SUBTLE: Color = Color::Rgb(45, 45, 45);
    pub const HIGHLIGHT: Color = Color::Rgb(38, 48, 41);
}

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(5),
    ])
    .split(frame.area());

    render_header(frame, chunks[0], app);
    render_body(frame, chunks[1], app);
    render_player(frame, chunks[2], app);
}

fn render_header(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let block = Block::new()
        .borders(Borders::BOTTOM)
        .border_style(Style::new().fg(theme::SUBTLE));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cols = Layout::horizontal([Constraint::Length(24), Constraint::Min(0)]).split(inner);

    let brand = Paragraph::new(Line::from(vec![
        Span::styled("▰ ", Style::new().fg(theme::ACCENT)),
        Span::styled(
            "HYPERSOUND",
            Style::new().fg(theme::TEXT).add_modifier(Modifier::BOLD),
        ),
    ]));
    frame.render_widget(brand, cols[0]);

    let hint = if app.searching {
        Line::from(vec![
            Span::styled(
                "SEARCH ",
                Style::new().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(&app.search, Style::new().fg(theme::TEXT)),
            Span::styled("▌", Style::new().fg(theme::ACCENT)),
            Span::styled("   enter / esc  done", Style::new().fg(theme::MUTED)),
        ])
    } else {
        Line::from(vec![
            Span::styled("↑↓", Style::new().fg(theme::TEXT)),
            Span::styled(" navigate   ", Style::new().fg(theme::MUTED)),
            Span::styled("enter", Style::new().fg(theme::TEXT)),
            Span::styled(" play   ", Style::new().fg(theme::MUTED)),
            Span::styled("space", Style::new().fg(theme::TEXT)),
            Span::styled(" pause   ", Style::new().fg(theme::MUTED)),
            Span::styled("/ ", Style::new().fg(theme::TEXT)),
            Span::styled("search   ", Style::new().fg(theme::MUTED)),
            Span::styled("q ", Style::new().fg(theme::TEXT)),
            Span::styled("quit", Style::new().fg(theme::MUTED)),
        ])
    };
    frame.render_widget(Paragraph::new(hint).alignment(Alignment::Right), cols[1]);
}

fn render_body(frame: &mut Frame, area: ratatui::layout::Rect, app: &mut App) {
    let show_art = area.width >= 96 && area.height >= 16;
    let cols = if show_art {
        Layout::horizontal([
            Constraint::Length(28),
            Constraint::Min(0),
            Constraint::Length(26),
        ])
        .split(area)
    } else {
        Layout::horizontal([Constraint::Length(28), Constraint::Min(0)]).split(area)
    };
    render_sidebar(frame, cols[0], app);
    render_library(frame, cols[1], app);
    if show_art {
        render_art(frame, cols[2], app);
    }
}

fn render_sidebar(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme::SUBTLE))
        .title(" YOUR LIBRARY ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(inner);

    let mut lines = Vec::new();
    for (i, entry) in app.sidebar.iter().enumerate() {
        let is_active = i == app.sidebar_index;
        let is_focused = is_active && app.focus == Focus::Sidebar;
        let count = app.entry_count(entry);
        lines.push(
            Line::from(vec![
                Span::styled(
                    if is_active { "▌" } else { " " },
                    Style::new().fg(theme::ACCENT),
                ),
                Span::styled(
                    format!(" {}  ", entry.icon()),
                    Style::new().fg(if is_active {
                        theme::ACCENT
                    } else {
                        theme::MUTED
                    }),
                ),
                Span::styled(
                    entry.label().to_string(),
                    Style::new()
                        .fg(if is_active { theme::TEXT } else { theme::MUTED })
                        .add_modifier(if is_active {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
                Span::styled(format!("  {}", count), Style::new().fg(theme::MUTED)),
            ])
            .style(if is_focused {
                Style::new().bg(theme::HIGHLIGHT)
            } else {
                Style::new()
            }),
        );
    }
    frame.render_widget(Paragraph::new(lines), rows[0]);

    let folder = app.music_dir.to_string_lossy();
    let path_lines = vec![
        Line::from(Span::styled("MUSIC FOLDER", Style::new().fg(theme::MUTED))),
        Line::from(Span::styled(
            truncate(&folder, (rows[1].width as usize).saturating_sub(2)),
            Style::new().fg(theme::SUBTLE),
        )),
    ];
    frame.render_widget(Paragraph::new(path_lines), rows[1]);
}

fn render_art(frame: &mut Frame, area: ratatui::layout::Rect, app: &mut App) {
    let vertical = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(14),
        Constraint::Min(0),
    ])
    .split(area);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme::SUBTLE))
        .title(" COVER ");
    let inner = block.inner(vertical[1]);
    frame.render_widget(block, vertical[1]);

    let w = inner.width as u32;
    let h = inner.height as u32;
    let Some(image) = app.album_art(w, h * 2) else {
        frame.render_widget(
            Paragraph::new("No cover")
                .alignment(Alignment::Center)
                .style(Style::new().fg(theme::MUTED)),
            inner,
        );
        return;
    };

    let mut lines = Vec::with_capacity(h as usize);
    for y in 0..h {
        let mut spans = Vec::with_capacity(w as usize);
        for x in 0..w {
            let top = image.get_pixel(x, y * 2);
            let bottom = image.get_pixel(x, y * 2 + 1);
            spans.push(Span::styled(
                "▀",
                Style::new()
                    .fg(Color::Rgb(top[0], top[1], top[2]))
                    .bg(Color::Rgb(bottom[0], bottom[1], bottom[2])),
            ));
        }
        lines.push(Line::from(spans));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_library(frame: &mut Frame, area: ratatui::layout::Rect, app: &mut App) {
    let title = if app.searching || !app.search.is_empty() {
        format!(" SEARCH “{}” ", app.search)
    } else {
        format!(
            " {}  ·  {} tracks ",
            app.selected_entry().label(),
            app.visible.len()
        )
    };

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme::SUBTLE))
        .title(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.visible.is_empty() {
        let message = if !app.search.is_empty() {
            vec![
                Line::from("No matches"),
                Line::from(Span::styled(
                    format!("Nothing found for “{}”", app.search),
                    Style::new().fg(theme::MUTED),
                )),
            ]
        } else if matches!(app.selected_entry(), SidebarEntry::View(View::Favorites)) {
            vec![
                Line::from("No favorites yet"),
                Line::from(Span::styled(
                    "Press f on a track to add it here",
                    Style::new().fg(theme::MUTED),
                )),
            ]
        } else {
            vec![
                Line::from("Your library is empty"),
                Line::from(Span::styled(
                    format!("Add audio files to {}", app.music_dir.display()),
                    Style::new().fg(theme::MUTED),
                )),
                Line::from(Span::styled(
                    "mp3 · flac · ogg · m4a · wav · opus …",
                    Style::new().fg(theme::SUBTLE),
                )),
            ]
        };
        let v = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(message.len() as u16),
            Constraint::Min(0),
        ])
        .split(inner);
        frame.render_widget(Paragraph::new(message).alignment(Alignment::Center), v[1]);
        return;
    }

    let header = Row::new(["#", "TITLE", "ARTIST", "ALBUM", "TIME"])
        .style(Style::new().fg(theme::MUTED).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let mut rows = Vec::with_capacity(app.visible.len());
    for (row, &index) in app.visible.iter().enumerate() {
        let track = &app.library[index];
        let is_current = app.current == Some(index);
        let is_selected = row == app.selected;

        let marker = if is_current {
            if app.playing { "▶" } else { "❚❚" }.to_string()
        } else {
            (row + 1).to_string()
        };

        let favorite = if app.favorites.contains(&track.path) {
            " ♥"
        } else {
            ""
        };
        let title_cell = Cell::from(Line::from(vec![
            Span::raw(track.title.clone()),
            Span::styled(favorite, Style::new().fg(theme::ACCENT)),
        ]));

        let mut style = if is_current {
            Style::new().fg(theme::ACCENT)
        } else {
            Style::new().fg(theme::TEXT)
        };
        if is_selected {
            style = style.bg(theme::HIGHLIGHT).add_modifier(Modifier::BOLD);
        }

        rows.push(
            Row::new([
                Cell::from(marker),
                title_cell,
                Cell::from(track.artist.clone()),
                Cell::from(track.album.clone()),
                Cell::from(fmt_duration(track.duration)),
            ])
            .style(style),
        );
    }

    let widths = [
        Constraint::Length(5),
        Constraint::Percentage(38),
        Constraint::Percentage(27),
        Constraint::Min(0),
        Constraint::Length(9),
    ];
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::new().padding(ratatui::widgets::Padding::horizontal(1)))
        .style(Style::new().fg(theme::TEXT));

    // Keep the selected row in view (the header takes one line + one margin).
    let viewable = inner.height.saturating_sub(2) as usize;
    let mut offset = app.table_state.offset();
    if app.selected >= offset + viewable {
        offset = app.selected + 1 - viewable;
    } else if app.selected < offset {
        offset = app.selected;
    }
    *app.table_state.offset_mut() = offset;

    frame.render_stateful_widget(table, inner, &mut app.table_state);
}

fn render_player(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme::SUBTLE))
        .title(" NOW PLAYING ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(inner);

    // --- track info + volume -------------------------------------------------
    let cols = Layout::horizontal([Constraint::Min(0), Constraint::Length(22)]).split(rows[0]);
    let info = match app.current_track() {
        Some(track) => Line::from(vec![
            Span::styled(
                if app.playing { "▶  " } else { "❚❚  " },
                Style::new().fg(theme::ACCENT),
            ),
            Span::styled(
                track.title.clone(),
                Style::new().fg(theme::TEXT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  —  ", Style::new().fg(theme::MUTED)),
            Span::styled(track.artist.clone(), Style::new().fg(theme::MUTED)),
        ]),
        None => Line::from(Span::styled(
            "Nothing playing  ·  pick a song and press enter",
            Style::new().fg(theme::MUTED),
        )),
    };
    frame.render_widget(Paragraph::new(info), cols[0]);

    let vol = app.volume();
    let filled = (vol * 6.0).round() as usize;
    let bars = format!(
        "{}{}",
        "▮".repeat(filled),
        "▯".repeat(6usize.saturating_sub(filled))
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(bars, Style::new().fg(theme::ACCENT)),
            Span::styled(
                format!(" {:>3}%", (vol * 100.0).round() as u8),
                Style::new().fg(theme::MUTED),
            ),
        ]))
        .alignment(Alignment::Right),
        cols[1],
    );

    // --- progress ------------------------------------------------------------
    let cols = Layout::horizontal([
        Constraint::Length(7),
        Constraint::Min(0),
        Constraint::Length(7),
    ])
    .split(rows[1]);
    let duration = app
        .current_track()
        .map(|t| t.duration)
        .unwrap_or(Duration::ZERO);
    let progress = app.progress();
    let ratio = if duration.is_zero() {
        0.0
    } else {
        (progress.as_secs_f64() / duration.as_secs_f64()).clamp(0.0, 1.0)
    };
    frame.render_widget(
        Paragraph::new(fmt_duration(progress)).style(Style::new().fg(theme::MUTED)),
        cols[0],
    );
    frame.render_widget(
        Gauge::default()
            .ratio(ratio)
            .gauge_style(Style::new().fg(theme::ACCENT).bg(theme::SUBTLE)),
        cols[1],
    );
    frame.render_widget(
        Paragraph::new(fmt_duration(duration))
            .alignment(Alignment::Right)
            .style(Style::new().fg(theme::MUTED)),
        cols[2],
    );

    // --- controls ------------------------------------------------------------
    let cols = Layout::horizontal([Constraint::Length(36), Constraint::Min(0)]).split(rows[2]);
    let play_icon = if app.playing { "⏸" } else { "▶" };
    let controls = Line::from(vec![
        Span::raw("   "),
        Span::styled(" ⏮ ", Style::new().fg(theme::MUTED)),
        Span::raw("   "),
        Span::styled(
            format!(" {play_icon} "),
            Style::new()
                .fg(Color::Black)
                .bg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(" ⏭ ", Style::new().fg(theme::MUTED)),
    ]);
    frame.render_widget(
        Paragraph::new(controls).alignment(Alignment::Center),
        cols[0],
    );

    let hints = if app.status.is_empty() {
        Line::from(vec![
            Span::styled("n/p", Style::new().fg(theme::TEXT)),
            Span::styled(" track   ", Style::new().fg(theme::MUTED)),
            Span::styled("←/→", Style::new().fg(theme::TEXT)),
            Span::styled(" seek   ", Style::new().fg(theme::MUTED)),
            Span::styled("+/-", Style::new().fg(theme::TEXT)),
            Span::styled(" volume   ", Style::new().fg(theme::MUTED)),
            Span::styled("f", Style::new().fg(theme::TEXT)),
            Span::styled(" favorite", Style::new().fg(theme::MUTED)),
        ])
    } else {
        Line::from(Span::styled(
            app.status.clone(),
            Style::new().fg(Color::Rgb(255, 120, 120)),
        ))
    };
    frame.render_widget(Paragraph::new(hints).alignment(Alignment::Right), cols[1]);
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let start = text.chars().count().saturating_sub(max.saturating_sub(1));
    let suffix: String = text.chars().skip(start).collect();
    format!("…{suffix}")
}
