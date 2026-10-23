//! Case-loader tests. Binary32 values are written by the test harness only, so
//! the crate's `no_floats` scan (src/*.rs) still sees binary floating point only in
//! `exact.rs`: these tests live outside `src/`.

use std::path::{Path, PathBuf};

use legal_exact_solver::bb::Assignment;
use legal_exact_solver::case::{self, FAMILIES, GUARDS};
use legal_exact_solver::check;

fn tiny_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("uor-legal-case-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_bits(dir: &Path, name: &str, values: &[f32]) {
    let bytes: Vec<u8> = values
        .iter()
        .flat_map(|v| v.to_bits().to_le_bytes())
        .collect();
    std::fs::write(dir.join(name), bytes).unwrap();
}

fn tiny_case(dir: &Path, guards: usize, row_len: usize) {
    for (i, (stem, count)) in FAMILIES.iter().enumerate() {
        let base = if i == 0 { 0.25 } else { -0.5 };
        let m: Vec<f32> = (0..*count).map(|j| base + 0.25 * (j % 3) as f32).collect();
        let g: Vec<f32> = (0..*count)
            .map(|j| 0.125 * ((j % 5) as f32 - 2.0))
            .collect();
        write_bits(dir, &format!("coupled-{stem}-initial-master.f32le"), &m);
        write_bits(dir, &format!("coupled-{stem}-gradient.f32le"), &g);
    }
    for i in 0..guards {
        let row: Vec<f32> = (0..row_len).map(|j| ((i + j) % 7) as f32 * 0.25).collect();
        write_bits(dir, &format!("protected-margin-{i:03}.f32le"), &row);
    }
}

#[test]
fn loads_the_expected_shape_and_counts_fractional() {
    let dir = tiny_dir("shape");
    tiny_case(&dir, GUARDS, 1920);
    let case = case::load(&dir).unwrap();
    assert_eq!(case.problem.n(), 1920);
    assert_eq!(case.guard_rows, GUARDS);
    assert_eq!(
        case.fractional, 0,
        "the tiny fixture masters are all on the quarter grid"
    );
    let report = check::check(&case.problem, &Assignment::zero(1920));
    assert!(report.legal, "zero assignment is legal");
    assert!(report.guards_hold, "all-zero guard values hold");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn refuses_a_nan_value() {
    let dir = tiny_dir("nan");
    tiny_case(&dir, GUARDS, 1920);
    write_bits(&dir, "protected-margin-007.f32le", &vec![f32::NAN; 1920]);
    let err = case::load(&dir).unwrap_err();
    assert!(err.contains("NaN or infinite"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn refuses_a_wrong_guard_row_length() {
    let dir = tiny_dir("rowlen");
    tiny_case(&dir, GUARDS, 1920);
    write_bits(&dir, "protected-margin-123.f32le", &vec![1.0; 1919]);
    let err = case::load(&dir).unwrap_err();
    assert!(err.contains("1919 coefficients, expected 1920"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn families_keep_their_order_by_value() {
    let dir = tiny_dir("order");
    // Distinct per-family values: prefix masters 0.25 / gradients 0.125;
    // generate masters -0.5 / gradients -0.25.
    for (stem, count) in FAMILIES.iter() {
        // Keyed by family NAME, not index: a loader that swaps the families must fail this test.
        let (mval, gval) = if *stem == "prefix.coefficients" {
            (0.25f32, 0.125f32)
        } else {
            (-0.5, -0.25)
        };
        write_bits(
            &dir,
            &format!("coupled-{stem}-initial-master.f32le"),
            &vec![mval; *count],
        );
        write_bits(
            &dir,
            &format!("coupled-{stem}-gradient.f32le"),
            &vec![gval; *count],
        );
    }
    for i in 0..GUARDS {
        write_bits(
            &dir,
            &format!("protected-margin-{i:03}.f32le"),
            &vec![0.0; 1920],
        );
    }
    let case = case::load(&dir).unwrap();
    let rat = |x: f32| legal_exact_solver::exact::f32_to_rational(x.to_bits()).unwrap();
    assert_eq!(case.problem.masters[0], rat(0.25));
    assert_eq!(case.problem.masters[959], rat(0.25));
    assert_eq!(case.problem.masters[960], rat(-0.5));
    assert_eq!(case.problem.g[0], rat(0.125));
    assert_eq!(case.problem.g[960], rat(-0.25));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn master_above_the_legal_range_is_refused() {
    let dir = tiny_dir("range");
    tiny_case(&dir, GUARDS, 1920);
    write_bits(&dir, "coupled-prefix.coefficients-initial-master.f32le", &{
        let mut v = vec![0.25f32; 960];
        v[0] = 2.0;
        v
    });
    let err = case::load(&dir).unwrap_err();
    assert!(err.contains("|m| > 1.75"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn term_identity_matches_and_catches_a_corrupted_aggregate() {
    let dir = tiny_dir("identity");
    for (stem, count) in FAMILIES {
        write_bits(
            &dir,
            &format!("coupled-{stem}-gradient.f32le"),
            &vec![31.0f32; count],
        );
        for i in 0..31 {
            write_bits(
                &dir,
                &format!("coupled-gradient-{stem}-term-{i:02}.f32le"),
                &vec![1.0f32; count],
            );
        }
    }
    let ok = case::term_identity(&dir).unwrap();
    assert!(ok.iter().all(|(_, matches)| *matches), "{ok:?}");
    // Corrupt one aggregate entry; the bit-exact f32 sum must no longer match.
    let mut aggregate = vec![31.0f32; 960];
    aggregate[0] = 30.0;
    write_bits(&dir, "coupled-generate.unary-gradient.f32le", &aggregate);
    let bad = case::term_identity(&dir).unwrap();
    assert!(
        bad.iter()
            .any(|(family, matches)| *family == "generate.unary" && !*matches),
        "{bad:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
