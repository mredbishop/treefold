use std::io;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::fs_scan::{count_errors, scan_path};
use crate::input::{Action, map_key};
use crate::layout::ensure_visible_offset;
use crate::state::AppState;
use crate::ui::{render, status_line};

pub fn run() -> Result<()> {
    let path = parse_root_arg()?;
    let root = scan_path(&path)
        .map_err(|e| anyhow!(e))
        .context("initial scan failed")?;
    let mut state = AppState::new(root);
    let mut term = TerminalGuard::new()?;

    loop {
        let viewport = term.terminal.get_frame().area().height.saturating_sub(4) as usize;
        state.scroll_offset =
            ensure_visible_offset(state.selected_index, state.scroll_offset, viewport);
        let status = status_line(&state, count_errors(&state.root));
        term.terminal.draw(|f| render(f, &state, &status))?;

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }
        if let Event::Key(key) = event::read()? {
            match map_key(key) {
                Action::Up => state.move_up(),
                Action::Down => state.move_down(),
                Action::PageUp => state.move_page_up(10),
                Action::PageDown => state.move_page_down(10),
                Action::Top => state.move_top(),
                Action::Bottom => state.move_bottom(),
                Action::Enter => state.enter_selected(),
                Action::Back => state.go_parent(),
                Action::Refresh => {
                    state.root = scan_path(&path).map_err(|e| anyhow!(e))?;
                    state.clamp_selection();
                }
                Action::Quit => break,
                Action::None => {}
            }
            state.clamp_selection();
        }
    }

    Ok(())
}

fn parse_root_arg() -> Result<PathBuf> {
    let mut args = std::env::args().skip(1);
    if let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            println!("treefold <optional-path>\n\nControls: arrows/hjkl, enter/l, esc/h, r, q");
            std::process::exit(0);
        }
        let path = PathBuf::from(arg);
        if !path.exists() {
            return Err(anyhow!("invalid path: {}", path.display()));
        }
        Ok(path)
    } else {
        Ok(std::env::current_dir()?)
    }
}

struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalGuard {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}
