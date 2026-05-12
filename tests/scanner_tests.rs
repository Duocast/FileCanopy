use std::fs;
use std::io::Write;

use filecanopy::analysis::tree;
use filecanopy::scanner::metadata::EntryKind;
use filecanopy::scanner::{ScanOptions, walker};

#[test]
fn scan_empty_dir_yields_zero_files() {
    let tmp = tempfile::tempdir().unwrap();
    let opts = ScanOptions {
        roots: vec![tmp.path().to_path_buf()],
        ..ScanOptions::default()
    };
    let report = walker::scan(&opts).unwrap();
    assert_eq!(report.file_count, 0);
    assert_eq!(report.dir_count, 0);
    assert_eq!(report.total_bytes, 0);
}

#[test]
fn scan_counts_files_and_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join("a/b")).unwrap();
    write_file(&root.join("a/one.txt"), b"hello");
    write_file(&root.join("a/b/two.bin"), &vec![0u8; 1024]);
    write_file(&root.join("top.log"), b"x");

    let report = walker::scan(&ScanOptions {
        roots: vec![root.to_path_buf()],
        ..ScanOptions::default()
    })
    .unwrap();

    assert_eq!(report.file_count, 3);
    assert_eq!(report.total_bytes, 5 + 1024 + 1);
    // `a` and `a/b` -> 2 directories.
    assert_eq!(report.dir_count, 2);
    assert!(report.errors.is_empty(), "errors: {:?}", report.errors);

    let files: Vec<_> = report
        .entries
        .iter()
        .filter(|e| e.kind == EntryKind::File)
        .collect();
    assert_eq!(files.len(), 3);
}

#[test]
fn scan_honors_excludes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("node_modules/pkg")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    write_file(&root.join("node_modules/pkg/big.bin"), &vec![0u8; 4096]);
    write_file(&root.join("src/main.rs"), b"fn main() {}");

    let report = walker::scan(&ScanOptions {
        roots: vec![root.to_path_buf()],
        excludes: vec!["node_modules".into()],
        ..ScanOptions::default()
    })
    .unwrap();

    assert_eq!(report.file_count, 1);
    assert_eq!(report.total_bytes, "fn main() {}".len() as u64);
    assert!(
        !report
            .entries
            .iter()
            .any(|e| e.path.components().any(|c| c.as_os_str() == "node_modules")),
        "excluded directory leaked into entries: {:?}",
        report.entries
    );
}

#[test]
fn build_tree_aggregates_sizes_up_the_hierarchy() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("a/b")).unwrap();
    write_file(&root.join("a/one.txt"), &vec![0u8; 100]);
    write_file(&root.join("a/b/two.txt"), &vec![0u8; 200]);

    let report = walker::scan(&ScanOptions {
        roots: vec![root.to_path_buf()],
        ..ScanOptions::default()
    })
    .unwrap();
    let tree = tree::build(&report, &[root.to_path_buf()]).unwrap();

    assert_eq!(tree.total_bytes, 300);
    assert_eq!(tree.root.path, root);
    assert_eq!(tree.root.file_count, 2);

    // Largest child first.
    let a = tree
        .root
        .children
        .iter()
        .find(|c| c.path.file_name().map(|n| n == "a").unwrap_or(false))
        .expect("expected child `a`");
    assert_eq!(a.size, 300);
    assert_eq!(a.file_count, 2);
}

fn write_file(path: &std::path::Path, contents: &[u8]) {
    let mut f = fs::File::create(path).unwrap();
    f.write_all(contents).unwrap();
}
