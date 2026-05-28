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
        .into_iter()
        .map(|mut block| {
            block.rect = clip_rect(block.rect, area);
            block
        })
        .filter(|b| b.rect.width > 0 && b.rect.height > 0)
        .collect()
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

fn clip_rect(rect: Rect, bounds: Rect) -> Rect {
    let x1 = rect.x.max(bounds.x);
    let y1 = rect.y.max(bounds.y);
    let x2 = rect
        .x
        .saturating_add(rect.width)
        .min(bounds.x.saturating_add(bounds.width));
    let y2 = rect
        .y
        .saturating_add(rect.height)
        .min(bounds.y.saturating_add(bounds.height));

    Rect {
        x: x1,
        y: y1,
        width: x2.saturating_sub(x1),
        height: y2.saturating_sub(y1),
    }
}

pub fn rect_within_bounds(rect: Rect, bounds: Rect) -> bool {
    rect.x >= bounds.x
        && rect.y >= bounds.y
        && rect.x.saturating_add(rect.width) <= bounds.x.saturating_add(bounds.width)
        && rect.y.saturating_add(rect.height) <= bounds.y.saturating_add(bounds.height)
}

pub fn rects_overlap(a: Rect, b: Rect) -> bool {
    let ax2 = a.x.saturating_add(a.width);
    let ay2 = a.y.saturating_add(a.height);
    let bx2 = b.x.saturating_add(b.width);
    let by2 = b.y.saturating_add(b.height);
    a.x < bx2 && ax2 > b.x && a.y < by2 && ay2 > b.y
}
