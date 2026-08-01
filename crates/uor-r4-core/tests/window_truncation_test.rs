use uor_r4_core::transformerless::compiler;
use uor_r4_core::transformerless::runtime::{self, bundle_window_plain, derive_rotations};

fn fixture_art() -> compiler::Compiled {
    let dir = env!("CARGO_MANIFEST_DIR");
    let bytes = std::fs::read(format!("{dir}/tests/fixtures/tless_artifacts.bin")).unwrap();
    compiler::parse_artifacts(&bytes).expect("fixture container parses")
}

#[test]
fn test_window_truncation_to_last_8_tokens() {
    let art = fixture_art();
    let rot = derive_rotations();

    // 12 tokens context
    let long_context: Vec<u32> = (1..=12).collect();
    // last 8 tokens context
    let truncated_context: Vec<u32> = (5..=12).collect();

    let bundle_long = bundle_window_plain(&art, &rot, &long_context);
    let bundle_truncated = bundle_window_plain(&art, &rot, &truncated_context);

    assert_eq!(bundle_long, bundle_truncated);
}

#[test]
fn test_window_truncation_equivalence_on_signature() {
    let art = fixture_art();
    let rot = derive_rotations();

    let long_input: [u32; 15] = [1, 2, 3, 4, 5, 6, 7, 10, 20, 30, 40, 50, 60, 70, 80];
    let short_input: [u32; 8] = [10, 20, 30, 40, 50, 60, 70, 80];

    let bundle_long = bundle_window_plain(&art, &rot, &long_input);
    let bundle_short = bundle_window_plain(&art, &rot, &short_input);

    let sig_long = runtime::sig_plain(&art, &bundle_long);
    let sig_short = runtime::sig_plain(&art, &bundle_short);

    assert_eq!(sig_long, sig_short);
}
