//! #395 E8/icosian spike support: dump real store content for the research
//! prototype. Writes a sample of centered context-bundle vectors (the exact
//! f32 objects the shipped path sign-codes into 36 bytes) plus the artifact
//! thresholds, as little-endian f32 binaries under /tmp/e8_spike/.
//!
//! Research tooling only (research/395-e8); never part of certify rows.
//!
//! Run:
//!   cargo test --release -p uor-r4-graph-certify --test e8_dump_bundles -- --ignored --nocapture

use std::io::Write;

use uor_r4_core::transformerless::compiler;
use uor_r4_core::transformerless::runtime;

fn fixture(name: &str) -> String {
    format!(
        "{}/../uor-r4-core/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

#[test]
#[ignore = "spike tooling; run explicitly with --ignored"]
fn dump_bundles() {
    const SAMPLE_EVERY: usize = 10; // 500k records -> 50k vectors
    let c = compiler::load_corpus_from(&fixture("c_meta.bin"), &fixture("c_recs.bin"))
        .expect("checked-in fixture corpus");
    let art = compiler::load_artifacts_from(&fixture("tless_artifacts.bin"))
        .expect("checked-in fixture artifacts");
    let rot = compiler::derive_rotations();

    std::fs::create_dir_all("/tmp/e8_spike").expect("mkdir");
    let mut out =
        std::io::BufWriter::new(std::fs::File::create("/tmp/e8_spike/bundles.f32").expect("open"));
    let mut count = 0usize;
    let mut dim = 0usize;
    for i in (0..c.n).step_by(SAMPLE_EVERY) {
        let bundle = runtime::bundle_plain(&art, &rot, &c, i);
        let centered = runtime::centered_work(&art, &bundle);
        dim = centered.len();
        for v in centered.iter() {
            out.write_all(&(*v as f32).to_le_bytes()).expect("write");
        }
        count += 1;
    }
    drop(out);
    let mut thr = std::io::BufWriter::new(
        std::fs::File::create("/tmp/e8_spike/thresholds.f32").expect("open"),
    );
    for t in art.thresholds.iter() {
        thr.write_all(&(*t as f32).to_le_bytes()).expect("write");
    }
    println!("dumped {count} centered bundle vectors, dim {dim}, to /tmp/e8_spike/");
}
