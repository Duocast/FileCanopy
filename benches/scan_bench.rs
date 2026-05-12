use criterion::{Criterion, criterion_group, criterion_main};

use filecanopy::scanner::{ScanOptions, walker};

fn bench_scan_empty(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let opts = ScanOptions {
        roots: vec![tmp.path().to_path_buf()],
        ..ScanOptions::default()
    };
    c.bench_function("scan empty dir", |b| {
        b.iter(|| walker::scan(&opts).unwrap());
    });
}

criterion_group!(benches, bench_scan_empty);
criterion_main!(benches);
