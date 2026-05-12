use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;
use crate::scanner::ScanReport;
use crate::scanner::metadata::EntryKind;

/// A hierarchical view of disk usage, suitable for treemap rendering or
/// drill-down navigation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SizeTree {
    pub root: DirNode,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DirNode {
    pub path: PathBuf,
    pub size: u64,
    pub file_count: u64,
    pub children: Vec<DirNode>,
    pub files: Vec<FileNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub path: PathBuf,
    pub size: u64,
}

/// Build a hierarchical tree from a flat scan report.
///
/// `roots` is the list of paths originally passed to the scanner. They are
/// preserved as non-collapsible nodes so the tree's root is the deepest
/// shared scan root rather than `/`.
pub fn build(report: &ScanReport, roots: &[PathBuf]) -> Result<SizeTree> {
    let mut builder = Builder::default();
    for r in roots {
        builder.pin_root(r);
    }
    for entry in &report.entries {
        match entry.kind {
            EntryKind::File => builder.insert_file(&entry.path, entry.size),
            EntryKind::Directory => builder.insert_dir(&entry.path),
            // Symlinks and other entries don't contribute to disk usage in
            // the tree (they're already counted via their target if
            // `follow_symlinks` is on; otherwise excluded).
            _ => {}
        }
    }
    Ok(builder.finish())
}

#[derive(Default)]
struct Builder {
    /// Trie keyed on path components. The empty key represents the root.
    nodes: BTreeMap<PathBuf, DirEntryAcc>,
    /// Scan roots — kept as anchors so we never collapse past them.
    pinned: BTreeSet<PathBuf>,
}

#[derive(Default)]
struct DirEntryAcc {
    files: Vec<FileNode>,
    children: Vec<PathBuf>,
    file_count: u64,
    /// Bytes from files directly inside this directory (not recursive).
    own_bytes: u64,
}

impl Builder {
    fn pin_root(&mut self, path: &Path) {
        self.insert_dir(path);
        self.pinned.insert(path.to_path_buf());
    }

    fn insert_dir(&mut self, path: &Path) {
        self.nodes.entry(path.to_path_buf()).or_default();
        // Make sure each ancestor is registered as a child of its parent.
        let mut cur = path.to_path_buf();
        while let Some(parent) = cur.parent().map(Path::to_path_buf) {
            if parent == cur {
                break;
            }
            let parent_acc = self.nodes.entry(parent.clone()).or_default();
            if !parent_acc.children.iter().any(|c| c == &cur) {
                parent_acc.children.push(cur.clone());
            }
            cur = parent;
        }
    }

    fn insert_file(&mut self, path: &Path, size: u64) {
        let parent = path.parent().map(Path::to_path_buf).unwrap_or_default();
        let acc = self.nodes.entry(parent.clone()).or_default();
        acc.files.push(FileNode {
            path: path.to_path_buf(),
            size,
        });
        acc.file_count += 1;
        acc.own_bytes += size;

        // Register parent chain so the file's directory is reachable from
        // the synthetic root even if no Directory entry was emitted for it.
        let mut cur = parent;
        while let Some(grandparent) = cur.parent().map(Path::to_path_buf) {
            if grandparent == cur {
                break;
            }
            let g = self.nodes.entry(grandparent.clone()).or_default();
            if !g.children.iter().any(|c| c == &cur) {
                g.children.push(cur.clone());
            }
            cur = grandparent;
        }
    }

    fn finish(mut self) -> SizeTree {
        // Start from the shortest registered path (often `/` because we
        // register every ancestor up to the filesystem root) and descend
        // through single-child synthetic ancestors until we hit a node
        // that actually carries content — files of its own, or more than
        // one child. That gives us the deepest meaningful root.
        let Some(mut root_path) = self
            .nodes
            .keys()
            .min_by_key(|p| p.components().count())
            .cloned()
        else {
            return SizeTree::default();
        };

        loop {
            if self.pinned.contains(&root_path) {
                break;
            }
            let acc = match self.nodes.get(&root_path) {
                Some(a) => a,
                None => break,
            };
            if !acc.files.is_empty() || acc.children.len() != 1 {
                break;
            }
            root_path = acc.children[0].clone();
        }

        let root = self.build_node(&root_path);
        let total_bytes = root.size;
        SizeTree { root, total_bytes }
    }

    fn build_node(&mut self, path: &Path) -> DirNode {
        let acc = self.nodes.remove(path).unwrap_or_default();
        let mut children: Vec<DirNode> = acc
            .children
            .iter()
            .map(|c| self.build_node(c))
            .collect();
        children.sort_by(|a, b| b.size.cmp(&a.size));

        let mut size = acc.own_bytes;
        let mut file_count = acc.file_count;
        for c in &children {
            size += c.size;
            file_count += c.file_count;
        }

        let mut files = acc.files;
        files.sort_by(|a, b| b.size.cmp(&a.size));

        DirNode {
            path: path.to_path_buf(),
            size,
            file_count,
            children,
            files,
        }
    }
}
