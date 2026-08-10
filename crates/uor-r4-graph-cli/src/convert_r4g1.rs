use std::path::PathBuf;
use uor_r4_core::transformerless::compiler::{self, HammingCalibrationReport};
use uor_r4_core::transformerless::convert_r4g1;
use uor_r4_core::transformerless::runtime;
use uor_r4_model_source::SourceUnavailable;

pub fn run(args: &[String]) -> Result<(), SourceUnavailable> {
    let mut artifacts_path: Option<PathBuf> = None;
    let mut store_path: Option<PathBuf> = None;
    let mut calibration_path: Option<PathBuf> = None;
    let mut out_path: Option<PathBuf> = None;
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| SourceUnavailable::new(format!("missing value for {flag}")))?;
        match flag.as_str() {
            "--artifacts" => artifacts_path = Some(PathBuf::from(value)),
            "--store" => store_path = Some(PathBuf::from(value)),
            "--calibration" => calibration_path = Some(PathBuf::from(value)),
            "--out" => out_path = Some(PathBuf::from(value)),
            _ => {
                return Err(SourceUnavailable::new(format!(
                    "unknown convert-r4g1 option: {flag}"
                )));
            }
        }
        index += 2;
    }
    let artifacts_path =
        artifacts_path.ok_or_else(|| SourceUnavailable::new("pass --artifacts <TLA container>"))?;
    let store_path =
        store_path.ok_or_else(|| SourceUnavailable::new("pass --store <TLS1 container>"))?;
    let out_path =
        out_path.ok_or_else(|| SourceUnavailable::new("pass --out <R4G1 output path>"))?;

    let artifact_bytes = std::fs::read(&artifacts_path).map_err(|error| {
        SourceUnavailable::new(format!("{}: {error}", artifacts_path.display()))
    })?;
    let artifacts = compiler::parse_artifacts(&artifact_bytes).ok_or_else(|| {
        SourceUnavailable::new(format!(
            "{}: not a TLA3/TLA4/TLA5 artifact container",
            artifacts_path.display()
        ))
    })?;
    let store_bytes = std::fs::read(&store_path)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", store_path.display())))?;
    // Both store eras are accepted: the current 8-byte-entry TLS1 and
    // the legacy 6-byte-entry (u16 token) variant.
    #[allow(deprecated)]
    let store = runtime::parse_store(&store_bytes)
        .or_else(|| runtime::parse_store_legacy_u16(&store_bytes))
        .ok_or_else(|| {
            SourceUnavailable::new(format!(
                "{}: not a TLS1 store (either era)",
                store_path.display()
            ))
        })?;
    let calibration = match calibration_path {
        Some(path) => {
            let text = std::fs::read_to_string(&path)
                .map_err(|error| SourceUnavailable::new(format!("{}: {error}", path.display())))?;
            let report: HammingCalibrationReport = serde_json::from_str(&text)
                .map_err(|error| SourceUnavailable::new(format!("{}: {error}", path.display())))?;
            Some(report)
        }
        None => None,
    };

    let (bytes, report) = convert_r4g1::convert(
        &artifact_bytes,
        &artifacts,
        &store,
        &store_bytes,
        calibration.as_ref(),
    )
    .ok_or_else(|| {
        SourceUnavailable::new(
            "R4G1 conversion failed: token codes, vocabulary, or class-signature books are malformed",
        )
    })?;

    // Fail closed: the converter must never emit an artifact its own
    // two-stage validator or the integrity CIDs reject. A rejection here is
    // a defect in the converter over bytes it just produced, not a host
    // condition — so it panics rather than surfacing SourceUnavailable.
    let view = uor_r4_graph_format::GraphView::parse(&bytes)
        .expect("converter produced an invalid R4G1 artifact");
    view.verify_cids()
        .expect("converter produced an artifact with bad CIDs");

    std::fs::write(&out_path, &bytes)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", out_path.display())))?;
    println!(
        "convert-r4g1: {} -> {}",
        artifacts_path.display(),
        out_path.display()
    );
    println!(
        "  nodes {} (1 root + {} class regions), edges {} ({} observed refinement + {} root fallback)",
        report.node_count,
        report.node_count - 1,
        report.edge_count,
        report.observed_refinement_edges,
        report.root_fallback_edges
    );
    println!(
        "  store keys migrated {}, root prior entries {}, calibrated radii {}",
        report.observed_prefix_keys, report.root_prior_entries, report.calibrated_radii
    );
    println!(
        "  HEAD bounds: A {} (max frontier width), E {} (max emission entries), W 5 x 36B signatures",
        report.max_frontier_width, report.max_emission_entries
    );
    println!(
        "  wrote {} bytes, κ blake3:{}",
        report.artifact_bytes,
        blake3::hash(&bytes).to_hex()
    );
    Ok(())
}
