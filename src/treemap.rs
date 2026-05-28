use ratatui::layout::Rect;

use crate::fs_scan::FsEntry;

#[derive(Debug, Clone)]
pub struct TreemapBlock {
    pub path: std::path::PathBuf,
    pub name: String,
    pub rect: Rect,
}

pub fn build_treemap(area: Rect, children: &[FsEntry]) -> Vec<TreemapBlock> {
    if area.width == 0 || area.height == 0 {
        return Vec::new();
    }
    let mut entries: Vec<&FsEntry> = children.iter().filter(|e| e.size > 0).collect();
    if entries.is_empty() {
        return Vec::new();
    }
    entries.sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.name.cmp(&b.name)));
    split_recursive(area, &entries, 0)
}

fn split_recursive(area: Rect, entries: &[&FsEntry], depth: usize) -> Vec<TreemapBlock> {
    if entries.is_empty() || area.width == 0 || area.height == 0 {
        return Vec::new();
    }
    if entries.len() == 1 {
        return vec![TreemapBlock {
            path: entries[0].path.clone(),
            name: entries[0].name.clone(),
            rect: area,
        }];
    }

    let total: u64 = entries.iter().map(|e| e.size).sum();
    if total == 0 {
        return Vec::new();
    }

    let split_horizontal = depth.is_multiple_of(2);
    let span = if split_horizontal {
        area.width
    } else {
        area.height
    };
    if span <= 1 {
        return entries
            .iter()
            .take(1)
            .map(|entry| TreemapBlock {
                path: entry.path.clone(),
                name: entry.name.clone(),
                rect: area,
            })
            .collect();
    }

    let first = entries[0];
    let mut first_span = ((first.size as u128 * span as u128) / total as u128) as u16;
    first_span = first_span.clamp(1, span - 1);

    let (a, b) = if split_horizontal {
        (
            Rect {
                x: area.x,
                y: area.y,
                width: first_span,
                height: area.height,
            },
            Rect {
                x: area.x + first_span,
                y: area.y,
                width: area.width - first_span,
                height: area.height,
            },
        )
    } else {
        (
            Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: first_span,
            },
            Rect {
                x: area.x,
                y: area.y + first_span,
                width: area.width,
                height: area.height - first_span,
            },
        )
    };

    let mut out = split_recursive(a, &entries[0..1], depth + 1);
    out.extend(split_recursive(b, &entries[1..], depth + 1));
    out
}
