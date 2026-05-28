use iced::widget::canvas::{self, Canvas, Frame, Path, Program, Stroke, Text};
use iced::{Color, Element, Length, Point, Rectangle, Theme, mouse};

use crate::fs_scan::{EntryKind, FsEntry};
use crate::layout::human_size;
use crate::treemap::build_treemap;

#[derive(Debug, Clone)]
pub struct HeatmapBlock {
    pub index: usize,
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub rect: Rectangle,
}

pub fn color_for_ratio(ratio: f32) -> Color {
    let clamped = ratio.clamp(0.0, 1.0);
    // cool -> warm scale
    let r = 0.10 + 0.85 * clamped;
    let g = 0.65 - 0.45 * clamped;
    let b = 0.85 - 0.75 * clamped;
    Color::from_rgb(r, g.max(0.1), b.max(0.1))
}

pub fn build_heatmap_blocks(width: f32, height: f32, entries: &[FsEntry]) -> Vec<HeatmapBlock> {
    if width <= 1.0 || height <= 1.0 {
        return Vec::new();
    }

    let area = ratatui::layout::Rect::new(0, 0, width as u16, height as u16);
    let tblocks = build_treemap(area, entries);
    let total = tblocks.iter().map(|b| b.size).sum::<u64>().max(1);

    tblocks
        .into_iter()
        .filter_map(|b| {
            let idx = entries
                .iter()
                .position(|e| Some(&e.path) == b.path.as_ref() && e.name == b.name);
            idx.map(|index| HeatmapBlock {
                index,
                name: b.name.clone(),
                size: b.size,
                is_dir: matches!(entries[index].kind, EntryKind::Directory),
                rect: Rectangle {
                    x: b.rect.x as f32,
                    y: b.rect.y as f32,
                    width: b.rect.width as f32,
                    height: b.rect.height as f32,
                },
            })
        })
        .map(|mut block| {
            let ratio = block.size as f32 / total as f32;
            // tiny nudge keeps 1px gaps from anti-aliasing seams
            block.rect.width = (block.rect.width - 0.5).max(0.0);
            block.rect.height = (block.rect.height - 0.5).max(0.0);
            let _ = ratio;
            block
        })
        .collect()
}

pub fn hit_test(blocks: &[HeatmapBlock], point: Point) -> Option<usize> {
    blocks.iter().find_map(|b| {
        let inside = point.x >= b.rect.x
            && point.x <= b.rect.x + b.rect.width
            && point.y >= b.rect.y
            && point.y <= b.rect.y + b.rect.height;
        inside.then_some(b.index)
    })
}

pub fn heatmap_canvas<'a, Message: Clone + 'a>(
    entries: Vec<FsEntry>,
    on_select: fn(usize) -> Message,
) -> Element<'a, Message> {
    Canvas::new(HeatmapCanvas { entries, on_select })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

struct HeatmapCanvas<Message> {
    entries: Vec<FsEntry>,
    on_select: fn(usize) -> Message,
}

impl<Message: Clone> Program<Message> for HeatmapCanvas<Message> {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        if let canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event
            && let Some(point) = cursor.position_in(bounds)
        {
            let blocks = build_heatmap_blocks(bounds.width, bounds.height, &self.entries);
            if let Some(idx) = hit_test(&blocks, point) {
                return Some(canvas::Action::publish((self.on_select)(idx)).and_capture());
            }
        }
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let blocks = build_heatmap_blocks(bounds.width, bounds.height, &self.entries);
        let total = blocks.iter().map(|b| b.size).sum::<u64>().max(1);

        for block in blocks {
            let ratio = block.size as f32 / total as f32;
            let fill = color_for_ratio(ratio);
            let rect = Path::rectangle(
                Point::new(block.rect.x, block.rect.y),
                iced::Size::new(block.rect.width, block.rect.height),
            );
            frame.fill(&rect, fill);
            frame.stroke(
                &rect,
                Stroke::default().with_color(Color::from_rgba(0.0, 0.0, 0.0, 0.45)),
            );

            if block.rect.width >= 80.0 && block.rect.height >= 22.0 {
                frame.fill_text(Text {
                    content: format!("{}  {}", block.name, human_size(block.size)),
                    position: Point::new(block.rect.x + 4.0, block.rect.y + 14.0),
                    color: Color::WHITE,
                    size: iced::Pixels(13.0),
                    ..Default::default()
                });
            }
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if let Some(point) = cursor.position_in(bounds) {
            let blocks = build_heatmap_blocks(bounds.width, bounds.height, &self.entries);
            if hit_test(&blocks, point).is_some() {
                return mouse::Interaction::Pointer;
            }
        }
        mouse::Interaction::default()
    }
}
