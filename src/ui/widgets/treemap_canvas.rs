//! `Canvas` program that paints a squarified treemap of the latest scan.
//!
//! The widget is *read-only* on `App`: it borrows the cached `SizeTree`
//! (recomputed once per scan) and lays out tiles each frame using
//! [`crate::visualization::treemap::layout`].

use iced::mouse;
use iced::widget::canvas as canvas_widget;
use iced::widget::canvas::{Frame, Geometry, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};

use crate::ui::app::App;
use crate::ui::message::Message;
use crate::visualization::treemap::{self, TreemapOptions};

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

        let Some(tree) = self.app.last_size_tree.as_ref() else {
            return vec![frame.into_geometry()];
        };

        let opts = TreemapOptions {
            width: bounds.width.max(1.0) as u32,
            height: bounds.height.max(1.0) as u32,
            max_tiles: 500,
            color_by_extension: true,
        };
        let tiles = treemap::layout(tree, &opts);

        for tile in tiles {
            let rect = Path::rectangle(
                Point::new(tile.x, tile.y),
                Size::new(tile.w.max(1.0), tile.h.max(1.0)),
            );
            frame.fill(&rect, color_for(&tile.label));
            frame.stroke(
                &rect,
                Stroke::default().with_color(Color::from_rgba(0.0, 0.0, 0.0, 0.4)),
            );
        }

        vec![frame.into_geometry()]
    }
}

fn color_for(label: &str) -> Color {
    let key = match label.rsplit_once('.') {
        Some((_, ext)) if !ext.is_empty() => ext,
        _ => label,
    };
    let h: u32 = key
        .bytes()
        .fold(2166136261u32, |a, b| (a ^ b as u32).wrapping_mul(16777619));
    let r = ((h >> 16) & 0xff) as f32 / 255.0;
    let g = ((h >> 8) & 0xff) as f32 / 255.0;
    let b = (h & 0xff) as f32 / 255.0;
    Color::from_rgb(r, g, b)
}
