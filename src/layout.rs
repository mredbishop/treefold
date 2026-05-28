use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn split_main(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(67)])
        .split(area);
    (chunks[0], chunks[1])
}

pub fn ensure_visible_offset(selected: usize, offset: usize, viewport_items: usize) -> usize {
    if viewport_items == 0 {
        return 0;
    }
    if selected < offset {
        selected
    } else if selected >= offset + viewport_items {
        selected + 1 - viewport_items
    } else {
        offset
    }
}

pub fn human_size(size: u64) -> String {
    if size < 1024 {
        return format!("{size} B");
    }
    let kib = size as f64 / 1024.0;
    if kib < 1024.0 {
        return format!("{kib:.1} KiB");
    }
    let mib = kib / 1024.0;
    if mib < 1024.0 {
        return format!("{mib:.1} MiB");
    }
    let gib = mib / 1024.0;
    format!("{gib:.1} GiB")
}
