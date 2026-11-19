use super::*;

use fixture::bits_q12;

type Row = [u64; LANES];
const LIVE: [usize; 3] = [0, 1, LANES - 1];
const GRID: u64 = 8;
const STEP: u64 = B as u64 / GRID;

fn row(kn: u128, ml: u128, uniform: u128) -> Row {
    let mut masses = [0; LANES];
    for (lane, mass) in LIVE.into_iter().zip([kn, ml, uniform]) {
        masses[lane] = mass as u64;
    }
    masses
}

fn mixed(masses: Row, weights: Row) -> u64 {
    let product_sum: u128 = LIVE
        .iter()
        .map(|&lane| u128::from(masses[lane]) * u128::from(weights[lane]))
        .sum();
    (product_sum / B).max(1) as u64
}

fn score(rows: &[Row], weights: Row) -> u64 {
    rows.iter()
        .map(|&masses| bits_q12(mixed(masses, weights)).unwrap())
        .sum()
}

fn directions(rows: &[Row], weights: Row, reverse_ceil: bool) -> [Option<String>; LANES] {
    std::array::from_fn(|lane| {
        LIVE.contains(&lane).then(|| {
            rows.iter()
                .map(|&masses| {
                    let numerator = B * (u128::from(masses[lane]) + 1);
                    let denominator = u128::from(mixed(masses, weights));
                    let whole = numerator / denominator;
                    let fraction = u128::from(numerator % denominator != 0);
                    if reverse_ceil {
                        whole
                    } else {
                        whole + fraction
                    }
                })
                .sum::<u128>()
                .to_string()
        })
    })
}

fn receipt(rows: &[Row], weights: Row) -> Bucket {
    let sums_q32 = directions(rows, weights, false);
    let max_lane = *LIVE
        .iter()
        .min_by_key(|&&lane| {
            std::cmp::Reverse(sums_q32[lane].as_ref().unwrap().parse::<u128>().unwrap())
        })
        .unwrap();
    Bucket {
        bucket: 0,
        targets: rows.len() as u64,
        mask: mask(0),
        max_lane,
        sums_q32,
    }
}

fn finite_match(bucket: &Bucket, rows: &[Row], weights: Row) -> Result<()> {
    require(
        bucket.sums_q32 == directions(rows, weights, false),
        "finite row oracle direction mismatch",
    )?;
    bucket_residual(bucket, 0, rows.len() as u64, mask(0))?;
    Ok(())
}

fn grid_min(rows: &[Row]) -> u64 {
    (0..=GRID)
        .flat_map(|a| (0..=GRID - a).map(move |b| (a, b)))
        .map(|(a, b)| {
            score(
                rows,
                row(
                    (a * STEP).into(),
                    (b * STEP).into(),
                    ((GRID - a - b) * STEP).into(),
                ),
            )
        })
        .min()
        .unwrap()
}

fn rowmax_bits(rows: &[Row]) -> u64 {
    rows.iter()
        .map(|masses| bits_q12(*masses.iter().max().unwrap().max(&1)).unwrap())
        .sum()
}

#[test]
fn bound_encloses_enumerated_simplex_and_live_floor_optimum() {
    let conflicting = [row(M, 0, M / 4096), row(M / 4, M, M / 4096)];
    let floor_live = [row(M / 16, M, M / 4096), row(M / 4, M, M / 4096)];
    assert!(rowmax_bits(&conflicting) < grid_min(&conflicting));
    assert_eq!(grid_min(&floor_live), 0);
    assert_eq!(mixed(conflicting[0], row(0, B, 0)), 1);

    let anchors = [
        row(B, 0, 0),
        row(0, B, 0),
        row(B - 2, 1, 1),
        row(B / 2, B / 4, B / 4),
    ];
    assert_eq!(receipt(&floor_live, anchors[2]).max_lane, 1);
    let cases = [
        conflicting,
        floor_live,
        [row(M, M, M / 4096), row(M, M, M / 4096)],
        [row(1, M, M / 4096), row(2, M - 1, M / 4096)],
        [row(3, 0, M / 4096), row(7, 0, M / 4096)],
    ];
    let min_total: u64 = cases.iter().map(|rows| grid_min(rows)).sum();
    let mut strongest = cases.iter().map(|rows| rowmax_bits(rows)).sum::<u64>();
    let mut best_feasible = u64::MAX;
    for weights in anchors {
        let mut resid = 0;
        let mut score_q12 = 0;
        for rows in &cases {
            let bucket = receipt(rows, weights);
            finite_match(&bucket, rows, weights).unwrap();
            resid += bucket_residual(&bucket, 0, rows.len() as u64, mask(0)).unwrap();
            score_q12 += score(rows, weights);
        }
        let lower = tangent_bound(score_q12, 10, resid).unwrap();
        strongest = strongest.max(lower);
        best_feasible = best_feasible.min(score_q12);
        assert!(lower <= min_total);
        assert!(min_total <= score_q12);
    }
    assert!(strongest <= min_total && min_total <= best_feasible);
}

#[test]
fn rejects_rounding_census_lane_and_wrong_tape_faults() {
    let rows = [row(3, M, M / 4096), row(9, 0, M / 4096)];
    let weights = row(B, 0, 0);
    let mut bucket = receipt(&rows, weights);
    finite_match(&bucket, &rows, weights).unwrap();

    bucket.sums_q32 = directions(&rows, weights, true);
    assert!(finite_match(&bucket, &rows, weights).is_err());
    bucket = receipt(&rows, weights);
    bucket.sums_q32.swap(0, 1);
    assert!(finite_match(&bucket, &rows, weights).is_err());
    bucket = receipt(&rows, weights);
    bucket.sums_q32[1] = None;
    assert!(bucket_residual(&bucket, 0, 2, mask(0)).is_err());
    bucket = receipt(&rows, weights);
    bucket.sums_q32[2] = Some("0".into());
    assert!(bucket_residual(&bucket, 0, 2, mask(0)).is_err());
    bucket = receipt(&rows, weights);
    assert!(bucket_residual(&bucket, 0, 3, mask(0)).is_err());
    assert!(bucket_residual(&bucket, 1, 2, mask(1)).is_err());

    let changed = [row(3, M, M / 4096), row(9, 1, M / 4096)];
    assert!(finite_match(&receipt(&changed, weights), &rows, weights).is_err());
}

#[test]
fn validates_ties_and_wide_numerator_and_descent_rounding() {
    let one = [row(1, M, M / 4096)];
    let weights = row(B, 0, 0);
    let bucket = receipt(&one, weights);
    assert_eq!(
        bucket.sums_q32[1].as_deref(),
        Some("1208925819614633469673472")
    );
    assert_eq!(bucket_residual(&bucket, 0, 1, mask(0)).unwrap(), B * M);
    assert_eq!(tangent_bound(2_000, 1, 1).unwrap(), 1995);
    assert_eq!(tangent_bound(20_000, 2, B + 1).unwrap(), 13_847);
    assert!(tangent_bound(1, 1, u128::MAX).is_err());

    let rows = [row(M, M, M / 4096)];
    let mut tied = receipt(&rows, weights);
    assert_eq!(tied.max_lane, 0);
    tied.max_lane = 1;
    assert!(bucket_residual(&tied, 0, 1, mask(0)).is_err());
    assert_eq!(
        decimal("000").unwrap_err().to_string(),
        "noncanonical directional sum"
    );
}

#[test]
fn cutoffs_compare_exact_integer_products() {
    assert_eq!(branch(85, 100), "counts carry");
    assert_eq!(branch(86, 100), "both open");
    assert_eq!(branch(90, 100), "both open");
    assert_eq!(branch(91, 100), "geometry critical");
    assert_eq!(branch(86, 101), "both open");
    assert_eq!(branch(90, 101), "both open");
    assert_eq!(branch(91, 101), "geometry critical");
    assert_eq!(branch(u64::MAX, u64::MAX), "geometry critical");
}

#[test]
fn missing_loose_and_resolved_intervals_have_distinct_status() {
    assert_eq!(classification(None, 80, 100, true), ("missing", None));
    assert_eq!(
        classification(Some(0), 85, 100, true),
        ("resolved", Some("counts carry"))
    );
    assert_eq!(
        classification(Some(86), 90, 100, true),
        ("resolved", Some("both open"))
    );
    assert_eq!(
        classification(Some(91), 100, 100, true),
        ("resolved", Some("geometry critical"))
    );
    assert_eq!(classification(Some(90), 100, 100, true), ("loose", None));
    assert_eq!(
        classification(Some(91), 100, 100, false),
        ("same_cut_only", None)
    );
}

#[test]
fn truncated_ordered_bucket_receipt_cannot_supply_even_a_vacuous_bound() {
    let manifest = Manifest {
        schema: String::new(),
        frame_sha256: String::new(),
        kn_mass_sha256: String::new(),
        source_sha256: String::new(),
        binary_sha256: String::new(),
        target_count: 1,
        per_mille: 1,
        bucket_targets: vec![0; BUCKETS],
        bucket_masks: (0..BUCKETS).map(mask).collect(),
        inputs: Value::Null,
        synthetic_fixture: true,
    };
    let point = Point {
        mode: "kn_only",
        pass: 0,
        score_q12: 0,
    };
    let receipt = DeviceReceipt {
        schema: String::new(),
        manifest_sha256: String::new(),
        frame_sha256: String::new(),
        targets: 1,
        fit_result_sha256: String::new(),
        verified_fit_sha256: String::new(),
        certificate_binary_sha256: String::new(),
        certificate_host_sha256: String::new(),
        anchors: ANCHORS
            .into_iter()
            .map(|name| Anchor {
                name: name.into(),
                pass: 0,
                status: "evaluated".into(),
                score_q12: Some(0),
                buckets: Some(
                    (0..BUCKETS - 1)
                        .map(|bucket| Bucket {
                            bucket,
                            targets: 0,
                            mask: mask(bucket),
                            max_lane: 0,
                            sums_q32: std::array::from_fn(|e| {
                                (mask(bucket) & (1 << e) != 0).then(|| "0".into())
                            }),
                        })
                        .collect(),
                ),
            })
            .collect(),
        rowmax_q12: Some(0),
    };
    let error = verify_bounds(receipt, &manifest, &Value::Null, &[point; 4], None).unwrap_err();
    assert_eq!(error.to_string(), "incomplete ordered bucket receipts");
}
