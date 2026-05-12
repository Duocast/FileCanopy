use filecanopy::analysis::duplicates;
use filecanopy::cli::args::HashAlgo;
use filecanopy::scanner::ScanReport;

#[test]
fn empty_report_yields_empty_duplicates() {
    let report = ScanReport::default();
    let dups = duplicates::find(&report, 1024, HashAlgo::Blake3).unwrap();
    assert!(dups.groups.is_empty());
    assert_eq!(dups.reclaimable_bytes, 0);
}
