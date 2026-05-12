//! `Canvas` program that paints a squarified treemap of the latest scan.
//!
//! The widget borrows the cached `SizeTree` (recomputed once per scan), the
//! current focus path, and the zoom-controlled `max_tiles` from `App`. It
//! lays tiles out each frame, draws labels (name + size) on the ones that
//! are big enough, and publishes a `TreemapTileClicked` message when the
//! user clicks a tile that points at a directory.

use std::path::PathBuf;

use iced::widget::canvas as canvas_widget;
use iced::widget::canvas::{Action, Event, Frame, Geometry, Path, Stroke, Text};
use iced::{Color, Element, Length, Pixels, Point, Rectangle, Renderer, Size, Theme, mouse};

use crate::ui::app::{App, find_dir};
use crate::ui::message::Message;
use crate::visualization::treemap::{self, Tile, TreemapOptions};

pub struct TreemapCanvas<'a> {
    app: &'a App,
}

impl<'a> TreemapCanvas<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }

    pub fn into_element(self) -> Element<'a, Message> {
        canvas_widget::Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn tiles(&self, bounds: Rectangle) -> Vec<Tile> {
        let Some(tree) = self.app.last_size_tree.as_ref() else {
            return Vec::new();
        };
        let node = self
            .app
            .treemap_focus
            .as_deref()
            .and_then(|p| find_dir(&tree.root, p))
            .unwrap_or(&tree.root);

        let opts = TreemapOptions {
            width: bounds.width.max(1.0) as u32,
            height: bounds.height.max(1.0) as u32,
            max_tiles: self.app.treemap_max_tiles.max(1),
            color_by_extension: true,
        };
        treemap::layout_node(node, &opts)
    }
}

impl<'a> canvas_widget::Program<Message> for TreemapCanvas<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let tiles = self.tiles(bounds);

        for tile in &tiles {
            let fill = color_for(&tile.label, tile.is_dir);
            let rect = Path::rectangle(
                Point::new(tile.x, tile.y),
                Size::new(tile.w.max(1.0), tile.h.max(1.0)),
            );
            frame.fill(&rect, fill);
            frame.stroke(
                &rect,
                Stroke::default().with_color(Color::from_rgba(0.0, 0.0, 0.0, 0.5)),
            );

            // Skip text when the tile is too small to be readable.
            if tile.w < 60.0 || tile.h < 22.0 {
                continue;
            }

            let label_color = readable_text_color(fill);
            let mut name = tile.label.clone();
            let max_chars = ((tile.w - 8.0) / 7.0).floor() as usize;
            if max_chars > 0 && name.chars().count() > max_chars {
                name = format!("{}…", name.chars().take(max_chars.saturating_sub(1)).collect::<String>());
            }

            frame.fill_text(Text {
                content: name,
                position: Point::new(tile.x + 4.0, tile.y + 3.0),
                color: label_color,
                size: Pixels(13.0),
                ..Text::default()
            });

            if tile.h >= 36.0 {
                let size_str = humansize::format_size(tile.size, humansize::DECIMAL);
                frame.fill_text(Text {
                    content: size_str,
                    position: Point::new(tile.x + 4.0, tile.y + 18.0),
                    color: label_color,
                    size: Pixels(11.0),
                    ..Text::default()
                });
            }
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event else {
            return None;
        };
        let position = cursor.position_in(bounds)?;
        let tiles = self.tiles(bounds);
        let hit = tiles
            .iter()
            .find(|t| {
                position.x >= t.x
                    && position.x <= t.x + t.w
                    && position.y >= t.y
                    && position.y <= t.y + t.h
            })?;
        if !hit.is_dir {
            return None;
        }
        let path: PathBuf = hit.path.clone()?;
        Some(Action::publish(Message::TreemapTileClicked(path)).and_capture())
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        let Some(position) = cursor.position_in(bounds) else {
            return mouse::Interaction::default();
        };
        let tiles = self.tiles(bounds);
        let over_dir = tiles.iter().any(|t| {
            t.is_dir
                && position.x >= t.x
                && position.x <= t.x + t.w
                && position.y >= t.y
                && position.y <= t.y + t.h
        });
        if over_dir {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

fn color_for(label: &str, is_dir: bool) -> Color {
    let key = if is_dir {
        label
    } else {
        match label.rsplit_once('.') {
            Some((_, ext)) if !ext.is_empty() => ext,
            _ => label,
        }
    };
    let h: u32 = key
        .bytes()
        .fold(2166136261u32, |a, b| (a ^ b as u32).wrapping_mul(16777619));
    // Use HSV-ish mapping so we get distinguishable colors at consistent
    // brightness — purely-hashed RGB sometimes produces near-black tiles
    // that make labels unreadable.
    let hue = (h & 0xffff) as f32 / 65535.0;
    let saturation = if is_dir { 0.45 } else { 0.65 };
    let value = if is_dir { 0.55 } else { 0.78 };
    hsv_to_rgb(hue, saturation, value)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i.rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    Color::from_rgb(r, g, b)
}

fn readable_text_color(bg: Color) -> Color {
    let luma = 0.299 * bg.r + 0.587 * bg.g + 0.114 * bg.b;
    if luma > 0.6 {
        Color::from_rgb(0.05, 0.05, 0.05)
    } else {
        Color::from_rgb(0.97, 0.97, 0.97)
    }
}
