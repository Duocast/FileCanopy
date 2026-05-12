use std::path::Path;

use crate::analysis::SizeTree;
use crate::Result;

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

/// Render a treemap of `tree` to `out`. Output format (SVG or PNG) is inferred
/// from the file extension on `out`.
pub fn render(_tree: &SizeTree, _opts: &TreemapOptions, _out: &Path) -> Result<()> {
    // TODO: squarified treemap layout + plotters SVG/PNG backend
    Ok(())
}

/// Compute squarified rectangles without drawing. Useful for embedding in
/// interactive UIs or HTML reports.
#[derive(Debug, Clone)]
pub struct Tile {
    pub label: String,
    pub size: u64,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub fn layout(_tree: &SizeTree, _opts: &TreemapOptions) -> Vec<Tile> {
    // TODO: squarified treemap algorithm (Bruls, Huijsen, van Wijk, 2000)
    Vec::new()
}
