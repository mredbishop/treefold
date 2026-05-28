use std::path::{Path, PathBuf};

use crate::fs_scan::{EntryKind, FsEntry};

#[derive(Debug, Clone)]
pub struct AppState {
    pub root: FsEntry,
    pub current_dir: PathBuf,
    pub selected_index: usize,
    pub scroll_offset: usize,
}

impl AppState {
    pub fn new(root: FsEntry) -> Self {
        let current_dir = root.path.clone();
        Self {
            root,
            current_dir,
            selected_index: 0,
            scroll_offset: 0,
        }
    }

    pub fn current_entry(&self) -> Option<&FsEntry> {
        find_entry(&self.root, &self.current_dir)
    }

    pub fn current_children(&self) -> &[FsEntry] {
        self.current_entry()
            .map(|e| e.children.as_slice())
            .unwrap_or(&[])
    }

    pub fn selected_child(&self) -> Option<&FsEntry> {
        self.current_children().get(self.selected_index)
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        let max = self.current_children().len();
        if max > 0 && self.selected_index + 1 < max {
            self.selected_index += 1;
        }
    }

    pub fn move_page_up(&mut self, amount: usize) {
        self.selected_index = self.selected_index.saturating_sub(amount);
    }

    pub fn move_page_down(&mut self, amount: usize) {
        let len = self.current_children().len();
        if len == 0 {
            return;
        }
        self.selected_index = (self.selected_index + amount).min(len - 1);
    }

    pub fn move_top(&mut self) {
        self.selected_index = 0;
    }

    pub fn move_bottom(&mut self) {
        let len = self.current_children().len();
        if len > 0 {
            self.selected_index = len - 1;
        }
    }

    pub fn enter_selected(&mut self) {
        let next_path = self
            .selected_child()
            .filter(|e| e.kind == EntryKind::Directory)
            .map(|e| e.path.clone());
        if let Some(path) = next_path {
            self.current_dir = path;
            self.selected_index = 0;
            self.scroll_offset = 0;
            self.clamp_selection();
        }
    }

    pub fn go_parent(&mut self) {
        if self.current_dir == self.root.path {
            return;
        }
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.selected_index = 0;
            self.scroll_offset = 0;
            self.clamp_selection();
        }
    }

    pub fn clamp_selection(&mut self) {
        let len = self.current_children().len();
        if len == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= len {
            self.selected_index = len - 1;
        }
    }

    pub fn ensure_visible(&mut self, viewport_items: usize) {
        if viewport_items == 0 {
            return;
        }
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else {
            let end = self.scroll_offset + viewport_items;
            if self.selected_index >= end {
                self.scroll_offset = self.selected_index + 1 - viewport_items;
            }
        }
    }
}

fn find_entry<'a>(root: &'a FsEntry, path: &Path) -> Option<&'a FsEntry> {
    if root.path == path {
        return Some(root);
    }
    for child in &root.children {
        if let Some(found) = find_entry(child, path) {
            return Some(found);
        }
    }
    None
}
