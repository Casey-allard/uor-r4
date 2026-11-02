use std::path::Path;

use uor_r4_cpu_train::{
    CpuFbsForward, CpuFbsHead, CpuPanelEvaluator, CpuRankMarginTrainer, RankMarginConfig,
    RankMarginSample,
};
use uor_r4_integer::stack::IntegerStackModel;

const PUBLIC_FILES: [(&str, &str); 2] = [
    (
        "dual-c131-seed-1-raw_both.uorlut",
        "a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd",
    ),
    (
        "dual-c131-seed-2-raw_both.uorlut",
        "da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1",
    ),
];

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-public-fbs-parity: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let directory = std::env::args()
        .nth(1)
        .ok_or_else(|| "usage: cpu-public-fbs-parity <public-fixture-dir>".to_owned())?;
    let directory = Path::new(&directory);
    let mut results = Vec::new();
    for (seed, (name, expected_sha256)) in PUBLIC_FILES.into_iter().enumerate() {
        let started = std::time::Instant::now();
        let artifact =
            std::fs::read(directory.join(name)).map_err(|error| format!("{name}: {error}"))?;
        let actual_sha256 = uor_r4_discrete::corpus::sha256(&artifact);
        if actual_sha256 != expected_sha256 {
            return Err(format!(
                "{name}: sha256 {actual_sha256}, expected {expected_sha256}"
            ));
        }
        let model = IntegerStackModel::parse(&artifact).map_err(|error| error.to_string())?;
        let shape = model.shape();
        if !shape.raw_gates || !shape.raw_transition || !shape.projection_fbs() {
            return Err(format!("{name}: expected a raw FBS artifact"));
        }

        let mut cpu = CpuFbsForward::from_artifact(&artifact).map_err(|error| error.to_string())?;
        let mut serving = model.session();
        let prefix_started = std::time::Instant::now();
        let mut prefixes = 0u64;
        for position in 0..shape.context {
            let token = ((position * 1_103 + seed * 97 + 17) % shape.vocab) as u32;
            let expected = serving
                .step_native(token)
                .map_err(|error| error.to_string())?;
            let actual = cpu
                .step(token, CpuFbsHead::Writer)
                .map_err(|error| error.to_string())?;
            if actual.query_q12 != expected.query_q12
                || actual.token != expected.token
                || actual.kept_cells != expected.kept_cells
            {
                return Err(format!(
                    "{name}: CPU/serving mismatch at prefix position {position}"
                ));
            }
            prefixes += 1;
        }
        let prefix_elapsed_ms = prefix_started.elapsed().as_secs_f64() * 1_000.0;

        let mut probe = CpuRankMarginTrainer::from_artifact(
            &artifact,
            vec![RankMarginSample {
                input: vec![0],
                target: 1,
            }],
            RankMarginConfig::default(),
            seed as u64 + 1,
        )
        .map_err(|error| error.to_string())?;
        let mut samples = Vec::new();
        for row in 0u32..16 {
            let input = (row * 257 + 3) % shape.vocab as u32;
            let reading = probe
                .evaluate_sample(&RankMarginSample {
                    input: vec![input],
                    target: 0,
                })
                .map_err(|error| error.to_string())?;
            let target = reading
                .top8
                .iter()
                .rev()
                .copied()
                .find(|&target| target != reading.top1)
                .ok_or_else(|| format!("{name}: model has no writer rival"))?;
            samples.push(RankMarginSample {
                input: vec![input],
                target,
            });
        }
        let mut baseline_panel = CpuPanelEvaluator::from_artifact(&artifact, samples.clone(), 1)
            .map_err(|error| error.to_string())?;
        let baseline_panel = baseline_panel
            .evaluate()
            .map_err(|error| error.to_string())?;
        let mut trainer = CpuRankMarginTrainer::from_artifact(
            &artifact,
            samples.clone(),
            RankMarginConfig::default(),
            seed as u64 + 1,
        )
        .map_err(|error| error.to_string())?;
        let training_steps = trainer
            .train_steps(samples.len())
            .map_err(|error| error.to_string())?;
        let updated_steps = training_steps.iter().filter(|step| step.updated).count();
        let mut panel = CpuPanelEvaluator::from_artifact(trainer.artifact(), samples, 1)
            .map_err(|error| error.to_string())?;
        let updated_panel = panel.evaluate().map_err(|error| error.to_string())?;
        if !baseline_panel.export.is_exact() || !updated_panel.export.is_exact() {
            return Err(format!("{name}: CPU/serving panel export gap is nonzero"));
        }

        results.push(serde_json::json!({
            "seed": seed + 1,
            "artifact": name,
            "sha256": actual_sha256,
            "raw_gates": shape.raw_gates,
            "raw_transition": shape.raw_transition,
            "prefixes_checked": prefixes,
            "prefix_elapsed_ms": prefix_elapsed_ms,
            "training_steps": training_steps.len(),
            "training_steps_updated": updated_steps,
            "panel_baseline": {
                "samples": baseline_panel.samples,
                "target_admitted": baseline_panel.target_admitted,
                "top1_correct": baseline_panel.top1_correct,
                "top8_correct": baseline_panel.top8_correct,
                "export_exact": baseline_panel.export.is_exact(),
            },
            "panel_updated": {
                "samples": updated_panel.samples,
                "target_admitted": updated_panel.target_admitted,
                "top1_correct": updated_panel.top1_correct,
                "top8_correct": updated_panel.top8_correct,
                "export_exact": updated_panel.export.is_exact(),
            },
            "elapsed_ms": started.elapsed().as_secs_f64() * 1_000.0,
        }));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "uor-r4.cpu-public-fbs-parity/1",
            "substrate": "cpu-development-research",
            "qualification": "gpu-required",
            "results": results,
        }))
        .map_err(|error| error.to_string())?
    );
    Ok(())
}
