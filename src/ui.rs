use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};

use crate::fs_scan::EntryKind;
use crate::layout::{human_size, split_main};
use crate::state::AppState;
use crate::treemap::{TreemapBlock, build_treemap};

pub fn render(frame: &mut Frame<'_>, state: &AppState, status: &str) {
    let outer = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(frame.area());
    let (left, right) = split_main(outer[0]);
    render_list(frame, state, left);
    render_treemap(frame, state, right);
    frame.render_widget(Paragraph::new(status), outer[1]);
}

fn render_list(frame: &mut Frame<'_>, state: &AppState, area: Rect) {
    let block = Block::default().title("Entries").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }

    let items: Vec<ListItem> = state
        .current_children()
        .iter()
        .map(|e| {
            let kind = match e.kind {
                EntryKind::Directory => "d",
                EntryKind::File => "f",
            };
            let err = if e.errors.is_empty() { "" } else { " !" };
            ListItem::new(format!("[{kind}] {}  {}{err}", e.name, human_size(e.size)))
        })
        .collect();

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(state.selected_index));
    let list = List::new(items)
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, inner, &mut list_state);
}

fn render_treemap(frame: &mut Frame<'_>, state: &AppState, area: Rect) {
    let block = Block::default().title("Treemap").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }
    let children = state.current_children();
    if children.is_empty() {
        frame.render_widget(Paragraph::new("Empty directory"), inner);
        return;
    }

    let blocks = build_treemap(inner, children);
    if let Some(msg) = treemap_fallback_message(inner, children, &blocks) {
        frame.render_widget(Paragraph::new(msg).wrap(Wrap { trim: true }), inner);
        return;
    }
    let selected = state.selected_child().map(|e| e.path.clone());
    for b in blocks {
        let is_selected = selected.as_ref().is_some_and(|p| p == &b.path);
        let style = if is_selected {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default()
        };
        let title = if b.rect.width > 4 && b.rect.height > 2 {
            truncate_label(&b.name, b.rect.width.saturating_sub(2) as usize)
        } else {
            String::new()
        };
        let w = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(style);
        frame.render_widget(Clear, b.rect);
        frame.render_widget(w, b.rect);
    }
}

pub fn treemap_fallback_message(
    area: Rect,
    children: &[crate::fs_scan::FsEntry],
    blocks: &[TreemapBlock],
) -> Option<&'static str> {
    if area.width < 8 || area.height < 4 {
        return Some("Treemap too small for current terminal size");
    }
    if children.iter().all(|e| e.size == 0) || blocks.is_empty() {
        return Some("No non-zero entries");
    }
    None
}

fn truncate_label(input: &str, width: usize) -> String {
    if input.chars().count() <= width {
        return input.to_string();
    }
    if width <= 1 {
        return String::new();
    }
    let mut out = String::new();
    for c in input.chars().take(width - 1) {
        out.push(c);
    }
    out.push('…');
    out
}

pub fn status_line(state: &AppState, error_count: usize) -> String {
    format!(
        "{} | {} | errors: {} | q quit | arrows/hjkl move | enter open | esc back | r refresh",
        state.current_dir.display(),
        human_size(state.current_entry().map_or(0, |e| e.size)),
        error_count
    )
}
