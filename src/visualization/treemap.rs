use std::path::Path;

use crate::Result;
use crate::analysis::SizeTree;
use crate::error::Error;

#[derive(Debug, Clone)]
pub struct TreemapOptions {
    pub width: u32,
    pub height: u32,
    pub max_tiles: usize,
    /// Color tiles by file extension if true; otherwise by depth.
    pub color_by_extension: bool,
}

impl Default for TreemapOptions {
    fn default() -> Self {
        Self {
            width: 1600,
            height: 1000,
            max_tiles: 500,
            color_by_extension: true,
        }
    }
}

/// A single rectangle in a laid-out treemap.
#[derive(Debug, Clone, PartialEq)]
pub struct Tile {
    pub label: String,
    pub size: u64,
    /// Filesystem path of the item this tile represents. `None` for the
    /// synthetic "(other)" aggregate tile produced when `max_tiles` truncates
    /// the long tail.
    pub path: Option<std::path::PathBuf>,
    /// `true` when the tile represents a directory (drill-target), `false`
    /// for a file or the aggregate tile.
    pub is_dir: bool,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Render a treemap of `tree` to `out`. Output format is inferred from the
/// extension; only SVG is supported today (PDF/PNG reports embed the same
/// `layout` output via their own renderer).
pub fn render(tree: &SizeTree, opts: &TreemapOptions, out: &Path) -> Result<()> {
    let ext = out
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext != "svg" {
        return Err(Error::Report(format!(
            "treemap render: unsupported extension {:?} (only .svg)",
            ext
        )));
    }
    let svg = render_svg(tree, opts);
    std::fs::write(out, svg).map_err(|e| Error::Io {
        path: Some(out.to_path_buf()),
        source: e,
    })?;
    Ok(())
}

/// Compute the squarified treemap rectangles for the immediate children
/// (subdirectories + files) of `tree.root`. The layout fills the rectangle
/// `(0, 0, opts.width, opts.height)`.
///
/// Implements the squarified treemap algorithm from
/// Bruls, Huijsen, van Wijk (2000). Tiles in the returned vector are in the
/// order they were laid out (row-by-row), not sorted by size.
pub fn layout(tree: &SizeTree, opts: &TreemapOptions) -> Vec<Tile> {
    layout_node(&tree.root, opts)
}

/// Like [`layout`] but lays out the children of an arbitrary `DirNode` so
/// callers can drill into a subdirectory without rebuilding the whole tree.
pub fn layout_node(node: &crate::analysis::tree::DirNode, opts: &TreemapOptions) -> Vec<Tile> {
    if opts.width == 0 || opts.height == 0 {
        return Vec::new();
    }
    let mut items = dir_items(node);
    if items.is_empty() {
        return Vec::new();
    }
    items.sort_by_key(|i| std::cmp::Reverse(i.size));
    if opts.max_tiles > 0 && items.len() > opts.max_tiles {
        let tail_sum: u64 = items[opts.max_tiles - 1..].iter().map(|i| i.size).sum();
        items.truncate(opts.max_tiles - 1);
        items.push(Item {
            label: "(other)".into(),
            size: tail_sum,
            path: None,
            is_dir: false,
        });
        items.sort_by_key(|i| std::cmp::Reverse(i.size));
    }
    let total: u128 = items.iter().map(|i| i.size as u128).sum();
    if total == 0 {
        return Vec::new();
    }

    let total_area = opts.width as f64 * opts.height as f64;
    let scale = total_area / total as f64;
    let scaled: Vec<ScaledItem> = items
        .into_iter()
        .map(|i| ScaledItem {
            label: i.label,
            size: i.size,
            path: i.path,
            is_dir: i.is_dir,
            area: (i.size as f64) * scale,
        })
        .collect();

    let mut tiles = Vec::with_capacity(scaled.len());
    squarify(
        &scaled,
        Rect {
            x: 0.0,
            y: 0.0,
            w: opts.width as f64,
            h: opts.height as f64,
        },
        &mut tiles,
    );
    tiles
}

#[derive(Debug, Clone)]
struct Item {
    label: String,
    size: u64,
    path: Option<std::path::PathBuf>,
    is_dir: bool,
}

#[derive(Debug, Clone)]
struct ScaledItem {
    label: String,
    size: u64,
    path: Option<std::path::PathBuf>,
    is_dir: bool,
    area: f64,
}

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn dir_items(node: &crate::analysis::tree::DirNode) -> Vec<Item> {
    let mut items: Vec<Item> = Vec::with_capacity(node.children.len() + node.files.len());
    for c in &node.children {
        items.push(Item {
            label: file_name_label(&c.path),
            size: c.size,
            path: Some(c.path.clone()),
            is_dir: true,
        });
    }
    for f in &node.files {
        items.push(Item {
            label: file_name_label(&f.path),
            size: f.size,
            path: Some(f.path.clone()),
            is_dir: false,
        });
    }
    items.retain(|i| i.size > 0);
    items
}

fn file_name_label(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

fn squarify(items: &[ScaledItem], initial: Rect, out: &mut Vec<Tile>) {
    let mut rect = initial;
    let mut row: Vec<&ScaledItem> = Vec::new();
    let mut i = 0;
    while i < items.len() {
        let candidate = &items[i];
        if candidate.area <= 0.0 {
            i += 1;
            continue;
        }
        let side = rect.w.min(rect.h);
        if side <= 0.0 {
            break;
        }

        if row.is_empty() {
            row.push(candidate);
            i += 1;
            continue;
        }

        let current_worst = worst_aspect(&row, side);
        row.push(candidate);
        let new_worst = worst_aspect(&row, side);
        if new_worst <= current_worst {
            i += 1;
        } else {
            row.pop();
            rect = layout_row(&row, rect, out);
            row.clear();
        }
    }
    if !row.is_empty() {
        layout_row(&row, rect, out);
    }
}

fn worst_aspect(row: &[&ScaledItem], side: f64) -> f64 {
    debug_assert!(!row.is_empty());
    let mut s = 0.0;
    let mut rmin = f64::INFINITY;
    let mut rmax: f64 = 0.0;
    for it in row {
        s += it.area;
        if it.area < rmin {
            rmin = it.area;
        }
        if it.area > rmax {
            rmax = it.area;
        }
    }
    if s <= 0.0 || rmin <= 0.0 {
        return f64::INFINITY;
    }
    let side2 = side * side;
    let s2 = s * s;
    (side2 * rmax / s2).max(s2 / (side2 * rmin))
}

fn layout_row(row: &[&ScaledItem], rect: Rect, out: &mut Vec<Tile>) -> Rect {
    let s: f64 = row.iter().map(|it| it.area).sum();
    if s <= 0.0 {
        return rect;
    }
    // Always lay the row along the shorter side; the strip's other dimension
    // is `s / shorter_side`, and successive tiles split the shorter side
    // proportionally to their area.
    if rect.w >= rect.h {
        let strip_w = (s / rect.h).min(rect.w);
        let mut y = rect.y;
        let last = row.len() - 1;
        for (idx, it) in row.iter().enumerate() {
            let th = if idx == last {
                rect.y + rect.h - y
            } else {
                it.area / strip_w
            };
            out.push(Tile {
                label: it.label.clone(),
                size: it.size,
                path: it.path.clone(),
                is_dir: it.is_dir,
                x: rect.x as f32,
                y: y as f32,
                w: strip_w as f32,
                h: th as f32,
            });
            y += th;
        }
        Rect {
            x: rect.x + strip_w,
            y: rect.y,
            w: (rect.w - strip_w).max(0.0),
            h: rect.h,
        }
    } else {
        let strip_h = (s / rect.w).min(rect.h);
        let mut x = rect.x;
        let last = row.len() - 1;
        for (idx, it) in row.iter().enumerate() {
            let tw = if idx == last {
                rect.x + rect.w - x
            } else {
                it.area / strip_h
            };
            out.push(Tile {
                label: it.label.clone(),
                size: it.size,
                path: it.path.clone(),
                is_dir: it.is_dir,
                x: x as f32,
                y: rect.y as f32,
                w: tw as f32,
                h: strip_h as f32,
            });
            x += tw;
        }
        Rect {
            x: rect.x,
            y: rect.y + strip_h,
            w: rect.w,
            h: (rect.h - strip_h).max(0.0),
        }
    }
}

fn render_svg(tree: &SizeTree, opts: &TreemapOptions) -> String {
    let tiles = layout(tree, opts);
    let mut s = String::with_capacity(256 + 96 * tiles.len());
    s.push_str(&format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"##,
        w = opts.width,
        h = opts.height,
    ));
    for t in &tiles {
        let (r, g, b) = svg_color(&t.label, opts.color_by_extension);
        s.push_str(&format!(
            r##"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="#{:02x}{:02x}{:02x}" stroke="#000" stroke-opacity="0.4"/>"##,
            t.x,
            t.y,
            t.w.max(0.5),
            t.h.max(0.5),
            r,
            g,
            b,
        ));
    }
    s.push_str("</svg>");
    s
}

fn svg_color(label: &str, color_by_extension: bool) -> (u8, u8, u8) {
    let key: &str = if color_by_extension {
        match label.rsplit_once('.') {
            Some((_, ext)) if !ext.is_empty() => ext,
            _ => label,
        }
    } else {
        label
    };
    let h: u32 = key
        .bytes()
        .fold(2166136261u32, |a, b| (a ^ b as u32).wrapping_mul(16777619));
    (
        ((h >> 16) & 0xff) as u8,
        ((h >> 8) & 0xff) as u8,
        (h & 0xff) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::tree::{DirNode, FileNode};
    use std::path::PathBuf;

    fn make_tree(files: &[(&str, u64)]) -> SizeTree {
        let mut root = DirNode {
            path: PathBuf::from("/"),
            ..DirNode::default()
        };
        for (name, size) in files {
            root.files.push(FileNode {
                path: PathBuf::from(name),
                size: *size,
            });
        }
        let total: u64 = files.iter().map(|(_, s)| *s).sum();
        root.size = total;
        root.file_count = files.len() as u64;
        SizeTree {
            root,
            total_bytes: total,
        }
    }

    #[test]
    fn empty_tree_yields_no_tiles() {
        let tree = SizeTree::default();
        let tiles = layout(&tree, &TreemapOptions::default());
        assert!(tiles.is_empty());
    }

    #[test]
    fn single_tile_fills_rect() {
        let tree = make_tree(&[("only.bin", 1000)]);
        let opts = TreemapOptions {
            width: 200,
            height: 100,
            max_tiles: 100,
            color_by_extension: true,
        };
        let tiles = layout(&tree, &opts);
        assert_eq!(tiles.len(), 1);
        let t = &tiles[0];
        assert!((t.x - 0.0).abs() < 1e-3);
        assert!((t.y - 0.0).abs() < 1e-3);
        assert!((t.w - 200.0).abs() < 1e-3);
        assert!((t.h - 100.0).abs() < 1e-3);
        assert_eq!(t.label, "only.bin");
        assert_eq!(t.size, 1000);
    }

    #[test]
    fn tiles_cover_canvas_exactly() {
        let tree = make_tree(&[
            ("a.txt", 100),
            ("b.txt", 80),
            ("c.txt", 60),
            ("d.txt", 40),
            ("e.txt", 20),
            ("f.txt", 10),
            ("g.txt", 5),
        ]);
        let opts = TreemapOptions {
            width: 400,
            height: 250,
            max_tiles: 100,
            color_by_extension: false,
        };
        let tiles = layout(&tree, &opts);
        assert_eq!(tiles.len(), 7);

        let total_area: f64 = tiles
            .iter()
            .map(|t| t.w as f64 * t.h as f64)
            .sum();
        let expected = opts.width as f64 * opts.height as f64;
        assert!(
            (total_area - expected).abs() / expected < 1e-3,
            "tiles area {} differs from canvas {}",
            total_area,
            expected
        );

        for t in &tiles {
            assert!(t.x >= -1e-3 && t.y >= -1e-3, "tile outside canvas: {:?}", t);
            assert!(
                t.x + t.w <= opts.width as f32 + 1e-2
                    && t.y + t.h <= opts.height as f32 + 1e-2,
                "tile overflows canvas: {:?}",
                t
            );
            assert!(t.w > 0.0 && t.h > 0.0, "degenerate tile: {:?}", t);
        }
    }

    #[test]
    fn tile_areas_are_proportional_to_size() {
        let tree = make_tree(&[("big", 800), ("small", 200)]);
        let opts = TreemapOptions {
            width: 100,
            height: 100,
            max_tiles: 100,
            color_by_extension: false,
        };
        let tiles = layout(&tree, &opts);
        let mut by_label: std::collections::HashMap<&str, f64> = Default::default();
        for t in &tiles {
            by_label.insert(t.label.as_str(), t.w as f64 * t.h as f64);
        }
        let big = by_label["big"];
        let small = by_label["small"];
        assert!(
            (big / (big + small) - 0.8).abs() < 1e-2,
            "big area share {} not ~0.8",
            big / (big + small)
        );
    }

    #[test]
    fn max_tiles_aggregates_tail() {
        let files: Vec<(String, u64)> =
            (0..20).map(|i| (format!("f{}.dat", i), (20 - i) as u64 * 10)).collect();
        let pairs: Vec<(&str, u64)> = files.iter().map(|(n, s)| (n.as_str(), *s)).collect();
        let tree = make_tree(&pairs);
        let opts = TreemapOptions {
            width: 300,
            height: 200,
            max_tiles: 5,
            color_by_extension: false,
        };
        let tiles = layout(&tree, &opts);
        assert_eq!(tiles.len(), 5);
        assert!(tiles.iter().any(|t| t.label == "(other)"));
    }

    #[test]
    fn render_svg_writes_file() {
        let tree = make_tree(&[("a.rs", 100), ("b.rs", 50), ("c.md", 25)]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("tm.svg");
        render(&tree, &TreemapOptions::default(), &out).unwrap();
        let body = std::fs::read_to_string(&out).unwrap();
        assert!(body.starts_with("<?xml"));
        assert!(body.contains("<svg"));
        assert!(body.contains("<rect"));
    }

    #[test]
    fn render_rejects_non_svg_extension() {
        let tree = make_tree(&[("a", 1)]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("tm.png");
        let err = render(&tree, &TreemapOptions::default(), &out).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("svg"), "unexpected error: {msg}");
    }
}
