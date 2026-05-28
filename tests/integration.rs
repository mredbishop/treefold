use std::fs;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use tempfile::tempdir;
use treefold::fs_scan::{EntryKind, FsEntry, count_errors, scan_path};
use treefold::input::{Action, map_key};
use treefold::layout::{ensure_visible_offset, human_size, split_main};
use treefold::state::AppState;
use treefold::treemap::build_treemap;
use treefold::ui::{render, status_line};

fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

#[test]
fn scanner_sizes_and_sort() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    fs::write(root.join("a.txt"), vec![0u8; 10]).expect("write a");
    fs::write(root.join("b.txt"), vec![0u8; 20]).expect("write b");
    fs::create_dir(root.join("sub")).expect("mkdir sub");
    fs::write(root.join("sub").join("c.txt"), vec![0u8; 30]).expect("write c");

    let entry = scan_path(root).expect("scan");
    assert_eq!(entry.size, 60);
    let sub = entry
        .children
        .iter()
        .find(|c| c.name == "sub")
        .expect("sub");
    assert_eq!(sub.size, 30);
    assert_eq!(entry.children[0].name, "sub");
    assert_eq!(entry.children[1].name, "b.txt");
    assert_eq!(entry.children[2].name, "a.txt");
}

#[test]
fn navigation_enter_and_parent() {
    let root = FsEntry {
        path: "root".into(),
        name: "root".into(),
        kind: EntryKind::Directory,
        size: 10,
        children: vec![FsEntry {
            path: "root/sub".into(),
            name: "sub".into(),
            kind: EntryKind::Directory,
            size: 10,
            children: vec![],
            errors: vec![],
        }],
        errors: vec![],
    };
    let mut st = AppState::new(root);
    st.enter_selected();
    assert!(st.current_dir.ends_with("sub"));
    st.go_parent();
    assert_eq!(st.current_dir, std::path::PathBuf::from("root"));
    st.go_parent();
    assert_eq!(st.current_dir, std::path::PathBuf::from("root"));
}

#[test]
fn selection_clamp_and_scroll() {
    let mut st = AppState::new(FsEntry {
        path: "r".into(),
        name: "r".into(),
        kind: EntryKind::Directory,
        size: 1,
        children: vec![FsEntry {
            path: "r/f".into(),
            name: "f".into(),
            kind: EntryKind::File,
            size: 1,
            children: vec![],
            errors: vec![],
        }],
        errors: vec![],
    });
    st.selected_index = 9;
    st.clamp_selection();
    assert_eq!(st.selected_index, 0);
    assert_eq!(ensure_visible_offset(9, 0, 5), 5);
}

#[test]
fn keybindings_map() {
    assert_eq!(map_key(key(KeyCode::Up, KeyModifiers::NONE)), Action::Up);
    assert_eq!(
        map_key(key(KeyCode::Char('k'), KeyModifiers::NONE)),
        Action::Up
    );
    assert_eq!(
        map_key(key(KeyCode::Down, KeyModifiers::NONE)),
        Action::Down
    );
    assert_eq!(
        map_key(key(KeyCode::Char('j'), KeyModifiers::NONE)),
        Action::Down
    );
    assert_eq!(
        map_key(key(KeyCode::PageUp, KeyModifiers::NONE)),
        Action::PageUp
    );
    assert_eq!(
        map_key(key(KeyCode::Char('u'), KeyModifiers::CONTROL)),
        Action::PageUp
    );
    assert_eq!(
        map_key(key(KeyCode::PageDown, KeyModifiers::NONE)),
        Action::PageDown
    );
    assert_eq!(
        map_key(key(KeyCode::Char('d'), KeyModifiers::CONTROL)),
        Action::PageDown
    );
    assert_eq!(
        map_key(key(KeyCode::Char('g'), KeyModifiers::NONE)),
        Action::Top
    );
    assert_eq!(
        map_key(key(KeyCode::Char('G'), KeyModifiers::SHIFT)),
        Action::Bottom
    );
    assert_eq!(
        map_key(key(KeyCode::Right, KeyModifiers::NONE)),
        Action::Enter
    );
    assert_eq!(
        map_key(key(KeyCode::Enter, KeyModifiers::NONE)),
        Action::Enter
    );
    assert_eq!(
        map_key(key(KeyCode::Char('l'), KeyModifiers::NONE)),
        Action::Enter
    );
    assert_eq!(
        map_key(key(KeyCode::Left, KeyModifiers::NONE)),
        Action::Back
    );
    assert_eq!(map_key(key(KeyCode::Esc, KeyModifiers::NONE)), Action::Back);
    assert_eq!(
        map_key(key(KeyCode::Char('h'), KeyModifiers::NONE)),
        Action::Back
    );
    assert_eq!(
        map_key(key(KeyCode::Char('r'), KeyModifiers::NONE)),
        Action::Refresh
    );
    assert_eq!(
        map_key(key(KeyCode::Char('q'), KeyModifiers::NONE)),
        Action::Quit
    );
    assert_eq!(
        map_key(key(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        Action::Quit
    );
    assert_eq!(
        map_key(key(KeyCode::Char('x'), KeyModifiers::NONE)),
        Action::None
    );
}

#[test]
fn layout_size_and_split() {
    let (left, right) = split_main(Rect::new(0, 0, 120, 20));
    assert!(left.width >= 39 && left.width <= 41);
    assert!(right.width >= 79 && right.width <= 81);
    assert_eq!(human_size(0), "0 B");
    assert_eq!(human_size(999), "999 B");
    assert_eq!(human_size(1024), "1.0 KiB");
    assert_eq!(human_size(1024 * 1024), "1.0 MiB");
}

#[test]
fn treemap_algorithm_properties() {
    let a = FsEntry {
        path: "a".into(),
        name: "a".into(),
        kind: EntryKind::File,
        size: 10,
        children: vec![],
        errors: vec![],
    };
    let b = FsEntry {
        path: "b".into(),
        name: "b".into(),
        kind: EntryKind::File,
        size: 10,
        children: vec![],
        errors: vec![],
    };
    let rects = build_treemap(Rect::new(0, 0, 40, 10), &[a.clone(), b.clone()]);
    assert_eq!(rects.len(), 2);
    let area0 = rects[0].rect.width as u32 * rects[0].rect.height as u32;
    let area1 = rects[1].rect.width as u32 * rects[1].rect.height as u32;
    assert!(area0.abs_diff(area1) <= 20);

    let c = FsEntry {
        size: 30,
        ..a.clone()
    };
    let d = FsEntry {
        size: 10,
        ..b.clone()
    };
    let rects2 = build_treemap(Rect::new(0, 0, 40, 10), &[c, d]);
    let area_big = rects2[0].rect.width as u32 * rects2[0].rect.height as u32;
    let area_small = rects2[1].rect.width as u32 * rects2[1].rect.height as u32;
    assert!(area_big > area_small);
}

#[test]
fn render_smoke_empty_and_selected() {
    let root = FsEntry {
        path: "root".into(),
        name: "root".into(),
        kind: EntryKind::Directory,
        size: 0,
        children: vec![],
        errors: vec![],
    };
    let state = AppState::new(root);
    let backend = TestBackend::new(80, 24);
    let mut term = Terminal::new(backend).expect("terminal");
    term.draw(|f| render(f, &state, "status")).expect("draw");

    let status = status_line(&state, count_errors(&state.root));
    assert!(status.contains("errors: 0"));
}
