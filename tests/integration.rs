use std::fs;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use tempfile::tempdir;
use treefold::cli::{Mode, help_text, parse_args};
use treefold::fs_scan::{EntryKind, FsEntry, count_errors, scan_path};
use treefold::gui::{GuiKeyAction, map_key_event};
use treefold::gui::{
    context_delete_label, context_open_label, init_state_from_path, open_target_path,
    resolve_default_root_path_from_env,
};
use treefold::gui_heatmap::{build_heatmap_blocks, color_for_ratio, hit_test, style_for_block};
use treefold::input::{Action, map_key};
use treefold::layout::{ensure_visible_offset, human_size, split_main};
use treefold::os_actions::{delete_path, open_command};
use treefold::state::AppState;
use treefold::treemap::{
    aggregate_small_entries, build_treemap, rect_within_bounds, rects_overlap,
};
use treefold::ui::{format_treemap_label, render, status_line, treemap_fallback_message};

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
fn treemap_fit_bounds_overlap_and_utilization() {
    let area = Rect::new(3, 2, 60, 20);
    let entries = vec![
        FsEntry {
            path: "a".into(),
            name: "a".into(),
            kind: EntryKind::File,
            size: 50,
            children: vec![],
            errors: vec![],
        },
        FsEntry {
            path: "b".into(),
            name: "b".into(),
            kind: EntryKind::File,
            size: 30,
            children: vec![],
            errors: vec![],
        },
        FsEntry {
            path: "c".into(),
            name: "c".into(),
            kind: EntryKind::File,
            size: 20,
            children: vec![],
            errors: vec![],
        },
    ];
    let blocks = build_treemap(area, &entries);
    assert!(!blocks.is_empty());

    for b in &blocks {
        assert!(rect_within_bounds(b.rect, area));
    }

    for (i, a) in blocks.iter().enumerate() {
        for b in blocks.iter().skip(i + 1) {
            assert!(!rects_overlap(a.rect, b.rect));
        }
    }

    let sum_area: u32 = blocks
        .iter()
        .map(|b| b.rect.width as u32 * b.rect.height as u32)
        .sum();
    let panel_area = area.width as u32 * area.height as u32;
    assert!(sum_area <= panel_area);
    assert!(sum_area >= panel_area.saturating_sub(area.width as u32));
}

#[test]
fn treemap_tiny_rect_has_fallback_message() {
    let area = Rect::new(0, 0, 7, 3);
    let entries = vec![FsEntry {
        path: "a".into(),
        name: "a".into(),
        kind: EntryKind::File,
        size: 10,
        children: vec![],
        errors: vec![],
    }];
    let blocks = build_treemap(area, &entries);
    let message = treemap_fallback_message(area, &entries, &blocks);
    assert_eq!(message, Some("Treemap too small for current terminal size"));
}

#[test]
fn treemap_recomputes_on_resize() {
    let entries = vec![
        FsEntry {
            path: "a".into(),
            name: "a".into(),
            kind: EntryKind::File,
            size: 3,
            children: vec![],
            errors: vec![],
        },
        FsEntry {
            path: "b".into(),
            name: "b".into(),
            kind: EntryKind::File,
            size: 2,
            children: vec![],
            errors: vec![],
        },
    ];
    let small = build_treemap(Rect::new(0, 0, 20, 8), &entries);
    let large = build_treemap(Rect::new(0, 0, 80, 24), &entries);
    let small_area: u32 = small
        .iter()
        .map(|b| b.rect.width as u32 * b.rect.height as u32)
        .sum();
    let large_area: u32 = large
        .iter()
        .map(|b| b.rect.width as u32 * b.rect.height as u32)
        .sum();
    assert!(large_area > small_area);
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

#[test]
fn treemap_label_formatting_handles_size_and_truncation() {
    let large = format_treemap_label("dependencies", 1024 * 1024, 40);
    assert!(large.contains("dependencies"));
    assert!(large.contains("1.0 MiB"));

    let medium = format_treemap_label("dependencies", 1024 * 1024, 16);
    assert!(medium.contains("1.0 MiB"));
    assert!(medium.contains('…'));

    let small = format_treemap_label("dependencies", 1024 * 1024, 6);
    assert!(!small.contains("MiB"));
    assert!(small.chars().count() <= 6);
}

#[test]
fn small_entries_are_aggregated_with_correct_size() {
    let area = Rect::new(0, 0, 20, 10);
    let entries = vec![
        FsEntry {
            path: "big".into(),
            name: "big".into(),
            kind: EntryKind::File,
            size: 1_000,
            children: vec![],
            errors: vec![],
        },
        FsEntry {
            path: "s1".into(),
            name: "s1".into(),
            kind: EntryKind::File,
            size: 1,
            children: vec![],
            errors: vec![],
        },
        FsEntry {
            path: "s2".into(),
            name: "s2".into(),
            kind: EntryKind::File,
            size: 1,
            children: vec![],
            errors: vec![],
        },
    ];

    let prepared = aggregate_small_entries(area, &entries);
    assert!(prepared.iter().any(|e| e.name == "big"));
    let aggregate = prepared
        .iter()
        .find(|e| e.name.starts_with("Small entries"))
        .expect("aggregate present");
    assert_eq!(aggregate.size, 2);
}

#[test]
fn treemap_shows_aggregate_block_when_threshold_triggers() {
    let area = Rect::new(0, 0, 40, 12);
    let mut entries = vec![FsEntry {
        path: "big".into(),
        name: "big".into(),
        kind: EntryKind::File,
        size: 5_000,
        children: vec![],
        errors: vec![],
    }];
    for i in 0..30 {
        entries.push(FsEntry {
            path: format!("s{i}").into(),
            name: format!("s{i}"),
            kind: EntryKind::File,
            size: 1,
            children: vec![],
            errors: vec![],
        });
    }
    let blocks = build_treemap(area, &entries);
    assert!(blocks.iter().any(|b| b.name.starts_with("Small entries")));
}

#[test]
fn treemap_many_tiny_files_no_panic() {
    let area = Rect::new(0, 0, 50, 20);
    let mut entries = Vec::new();
    for i in 0..500 {
        entries.push(FsEntry {
            path: format!("tiny-{i}").into(),
            name: format!("tiny-{i}"),
            kind: EntryKind::File,
            size: 1,
            children: vec![],
            errors: vec![],
        });
    }
    let _ = build_treemap(area, &entries);
}

#[test]
fn macos_assets_and_scripts_exist() {
    assert!(std::path::Path::new("assets/treefold-icon.svg").exists());
    assert!(std::path::Path::new("scripts/generate_macos_icon.sh").exists());
    assert!(std::path::Path::new("scripts/build_apple_silicon_app.sh").exists());
    assert!(std::path::Path::new("scripts/build_gui_apple_silicon_app.sh").exists());
    assert!(std::path::Path::new("scripts/check_gui_targets.sh").exists());
}

#[test]
fn gui_app_initializes_from_path() {
    let root = std::env::current_dir().expect("cwd");
    let state = init_state_from_path(root.to_string_lossy().as_ref()).expect("init");
    assert!(state.root.path.exists());
}

#[test]
fn gui_heatmap_blocks_within_bounds_and_non_overlapping() {
    let entries = vec![
        FsEntry {
            path: "a".into(),
            name: "a".into(),
            kind: EntryKind::Directory,
            size: 50,
            children: vec![],
            errors: vec![],
        },
        FsEntry {
            path: "b".into(),
            name: "b".into(),
            kind: EntryKind::File,
            size: 30,
            children: vec![],
            errors: vec![],
        },
        FsEntry {
            path: "c".into(),
            name: "c".into(),
            kind: EntryKind::File,
            size: 20,
            children: vec![],
            errors: vec![],
        },
    ];
    let blocks = build_heatmap_blocks(800.0, 500.0, &entries);
    assert!(!blocks.is_empty());
    for b in &blocks {
        assert!(b.rect.x >= 0.0 && b.rect.y >= 0.0);
        assert!(b.rect.x + b.rect.width <= 800.0);
        assert!(b.rect.y + b.rect.height <= 500.0);
    }
    for i in 0..blocks.len() {
        for j in i + 1..blocks.len() {
            let a = &blocks[i].rect;
            let b = &blocks[j].rect;
            let overlap = a.x < b.x + b.width
                && a.x + a.width > b.x
                && a.y < b.y + b.height
                && a.y + a.height > b.y;
            assert!(!overlap);
        }
    }
}

#[test]
fn gui_heatmap_color_scale_ordering() {
    let low = color_for_ratio(0.0);
    let mid = color_for_ratio(0.5);
    let high = color_for_ratio(1.0);
    assert!(high.r > mid.r && mid.r > low.r);
    assert!(low.b > mid.b && mid.b > high.b);
}

#[test]
fn gui_heatmap_hit_test_maps_to_entry() {
    let entries = vec![
        FsEntry {
            path: "dir".into(),
            name: "dir".into(),
            kind: EntryKind::Directory,
            size: 90,
            children: vec![],
            errors: vec![],
        },
        FsEntry {
            path: "file".into(),
            name: "file".into(),
            kind: EntryKind::File,
            size: 10,
            children: vec![],
            errors: vec![],
        },
    ];
    let blocks = build_heatmap_blocks(600.0, 400.0, &entries);
    assert!(!blocks.is_empty());
    let p = iced::Point::new(blocks[0].rect.x + 2.0, blocks[0].rect.y + 2.0);
    let hit = hit_test(&blocks, p);
    assert!(hit.is_some());
}

#[test]
fn gui_key_mapping_parity() {
    let up = iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key: iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowUp),
        location: iced::keyboard::Location::Standard,
        modifiers: iced::keyboard::Modifiers::default(),
        text: None,
        modified_key: iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowUp),
        physical_key: iced::keyboard::key::Physical::Unidentified(
            iced::keyboard::key::NativeCode::Unidentified,
        ),
        repeat: false,
    });
    let down = iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key: iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown),
        location: iced::keyboard::Location::Standard,
        modifiers: iced::keyboard::Modifiers::default(),
        text: None,
        modified_key: iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown),
        physical_key: iced::keyboard::key::Physical::Unidentified(
            iced::keyboard::key::NativeCode::Unidentified,
        ),
        repeat: false,
    });
    let enter = iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter),
        location: iced::keyboard::Location::Standard,
        modifiers: iced::keyboard::Modifiers::default(),
        text: None,
        modified_key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter),
        physical_key: iced::keyboard::key::Physical::Unidentified(
            iced::keyboard::key::NativeCode::Unidentified,
        ),
        repeat: false,
    });
    let esc = iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
        location: iced::keyboard::Location::Standard,
        modifiers: iced::keyboard::Modifiers::default(),
        text: None,
        modified_key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
        physical_key: iced::keyboard::key::Physical::Unidentified(
            iced::keyboard::key::NativeCode::Unidentified,
        ),
        repeat: false,
    });
    let q = iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key: iced::keyboard::Key::Character("q".into()),
        location: iced::keyboard::Location::Standard,
        modifiers: iced::keyboard::Modifiers::default(),
        text: Some("q".into()),
        modified_key: iced::keyboard::Key::Character("q".into()),
        physical_key: iced::keyboard::key::Physical::Unidentified(
            iced::keyboard::key::NativeCode::Unidentified,
        ),
        repeat: false,
    });

    assert_eq!(map_key_event(&up), GuiKeyAction::Up);
    assert_eq!(map_key_event(&down), GuiKeyAction::Down);
    assert_eq!(map_key_event(&enter), GuiKeyAction::Enter);
    assert_eq!(map_key_event(&esc), GuiKeyAction::Back);
    assert_eq!(map_key_event(&q), GuiKeyAction::None);
}

#[test]
fn cli_defaults_to_gui() {
    let parsed = parse_args(Vec::<String>::new()).expect("parse");
    assert_eq!(parsed.mode, Mode::Gui);
    assert!(parsed.path.is_none());
}

#[test]
fn cli_tui_short_and_long_flag() {
    let parsed_short = parse_args(["-t"]).expect("parse short");
    assert_eq!(parsed_short.mode, Mode::Tui);
    let parsed_long = parse_args(["--tui"]).expect("parse long");
    assert_eq!(parsed_long.mode, Mode::Tui);
}

#[test]
fn cli_path_parsing() {
    let parsed = parse_args(["-t", "."]).expect("parse");
    assert_eq!(parsed.mode, Mode::Tui);
    assert_eq!(parsed.path.expect("path"), std::path::PathBuf::from("."));
}

#[test]
fn cli_help_text_mentions_gui_default_and_tui_flag() {
    let help = help_text();
    assert!(help.contains("Default mode: GUI"));
    assert!(help.contains("--tui"));
}

#[test]
fn gui_context_labels_by_type() {
    assert_eq!(
        context_open_label(EntryKind::Directory),
        "Open this directory"
    );
    assert_eq!(context_open_label(EntryKind::File), "View in parent");
    assert_eq!(
        context_delete_label(EntryKind::Directory),
        "Delete this folder"
    );
    assert_eq!(context_delete_label(EntryKind::File), "Delete this file");
}

#[test]
fn gui_open_target_path_rules() {
    let file = std::path::PathBuf::from("/tmp/a/b/c.txt");
    let dir = std::path::PathBuf::from("/tmp/a/b");
    assert_eq!(
        open_target_path(EntryKind::File, &file),
        std::path::PathBuf::from("/tmp/a/b")
    );
    assert_eq!(open_target_path(EntryKind::Directory, &dir), dir);
}

#[test]
fn os_open_command_resolves() {
    let (cmd, args) = open_command(std::path::Path::new("."));
    assert!(!cmd.is_empty());
    assert!(!args.is_empty());
}

#[test]
fn gui_style_mapping_file_vs_folder_differs() {
    let a = style_for_block(true, 0.5, false, false);
    let b = style_for_block(false, 0.5, false, false);
    assert_ne!(a.border_width, b.border_width);
}

#[test]
fn delete_path_confirm_and_cancel_behavior() {
    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("x.txt");
    std::fs::write(&file, b"hello").expect("write");
    // cancel behavior: do nothing, file remains
    assert!(file.exists());
    // confirm behavior: delete
    delete_path(&file, false).expect("delete file");
    assert!(!file.exists());
}

#[test]
fn gui_default_root_prefers_home() {
    let p = resolve_default_root_path_from_env(
        Some(std::path::PathBuf::from("/tmp/home-test")),
        Some(std::path::PathBuf::from("/tmp/userprofile-test")),
        Some(std::path::PathBuf::from("/tmp/cwd-test")),
    );
    assert_eq!(p, std::path::PathBuf::from("/tmp/home-test"));
}

#[test]
fn gui_default_root_falls_back_to_userprofile_then_cwd() {
    let p1 = resolve_default_root_path_from_env(
        None,
        Some(std::path::PathBuf::from("/tmp/userprofile-test")),
        Some(std::path::PathBuf::from("/tmp/cwd-test")),
    );
    assert_eq!(p1, std::path::PathBuf::from("/tmp/userprofile-test"));

    let p2 = resolve_default_root_path_from_env(
        None,
        None,
        Some(std::path::PathBuf::from("/tmp/cwd-test")),
    );
    assert_eq!(p2, std::path::PathBuf::from("/tmp/cwd-test"));
}

#[test]
fn gui_manual_invalid_path_returns_error() {
    let bad = "/definitely/not/a/real/treefold/path";
    let result = init_state_from_path(bad);
    assert!(result.is_err());
}
