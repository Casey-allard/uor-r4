//! Byte-only usage; pass a caller-owned path to demonstrate explicit persistence.
use uor_r4_object_store::{Limits, Manifest, Store};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = b"An immutable exact quotation. An immutable exact quotation.\0\0";
    let mut writer = Store::new(Limits::default());
    let folded = writer.fold(bytes)?;
    let pinned_version = folded.manifest.version();
    let mut reader = Store::new(Limits::default());
    reader.import_records(&writer.export_records()?)?;
    let manifest = Manifest::decode(&folded.manifest.encode(), reader.limits())?;
    assert_eq!(manifest.version(), pinned_version);
    let whole = reader.rebuild(&manifest)?;
    assert_eq!(whole.bytes(), bytes);
    let quotation = reader.range(&manifest, 3..12)?;
    assert_eq!(quotation.text()?, "immutable");
    assert!(!quotation.whole_object_digest_verified());
    if let Some(path) = std::env::args_os().nth(1) {
        writer.save(&path)?;
        let mut restarted = Store::new(Limits::default());
        restarted.load(path)?;
        assert_eq!(restarted.rebuild(&manifest)?.bytes(), bytes);
    }
    println!(
        "PASS: exact byte quotation; raw={} retained={} new={} visits={}",
        folded.stats.raw_bytes,
        folded.stats.retained_record_bytes,
        folded.stats.new_record_bytes,
        whole.stats().visits
    );
    Ok(())
}
