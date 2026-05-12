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
}
