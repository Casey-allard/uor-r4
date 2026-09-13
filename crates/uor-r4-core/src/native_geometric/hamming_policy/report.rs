use super::{
    artifact::Artifact,
    credit::{generate, trajectory},
    policy::{Intervention, Params, GATES},
};
use crate::{
    native_geometric::{
        addressed_attention::{artifact::BoundGeometry, pilot_data},
        hamming_refinement::metric::Metric,
    },
    report_output,
};
use serde_json::json;
#[test]
#[ignore = "exclusive initialized policy conformance and actual generation; zero fitting"]
fn hamming_policy_report() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from(std::env::var("UOR_HAMMING_POLICY_REPORT")?);
    if path.as_os_str().is_empty() {
        return Err("empty report".into());
    }
    report_output::claim(&path)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let params = Params::seeded(973);
        let initial = Artifact::new(params.clone(), &g)?;
        std::fs::write(path.join("initialized.json"), initial.encode()?)?;
        let restored = Artifact::decode(&initial.encode()?, &g)?;
        let training = pilot_data::training();
        let development = pilot_data::development();
        // Fixed before execution: two construction records, two interventions.
        // No search over cells, no candidate acceptance, no optimizer update.
        let examples = [&training[0], &training[12]];
        let mut contrasts = Vec::new();
        let mut calls = 0u64;
        let mut selected_changes = 0usize;
        for (label, gates) in [
            ("hidden-gate-zero-complement", 0..1),
            ("geometric-output-first-16-complement", 1280..1296),
        ] {
            let mut changed = params.clone();
            for gate in gates.clone() {
                changed.tables[gate] ^= u16::MAX;
            }
            let changed = Artifact::new(changed, &g)?;
            std::fs::write(path.join(format!("{label}.json")), changed.encode()?)?;
            for e in examples {
                let parent = trajectory(&initial, &g, &m, e, Intervention::Full)?;
                let candidate = trajectory(&changed, &g, &m, e, Intervention::Full)?;
                let replay = trajectory(&restored, &g, &m, e, Intervention::Full)?;
                if parent.trace != replay.trace
                    || parent.cells != replay.cells
                    || parent.loss != replay.loss
                {
                    return Err("artifact trajectory mismatch".into());
                }
                let delta = candidate.loss - parent.loss;
                let independent = candidate.reference_loss - parent.reference_loss;
                if (delta - independent).abs() > 1e-9 {
                    return Err("finite contrast reference mismatch".into());
                }
                let changed_steps = parent
                    .trace
                    .iter()
                    .zip(&candidate.trace)
                    .filter(|(a, b)| a != b)
                    .count();
                selected_changes += changed_steps;
                let queried_cells: Vec<_> = gates
                    .clone()
                    .flat_map(|gate| (0..16).map(move |row| (gate << 4) | row))
                    .collect();
                let visits = queried_cells
                    .iter()
                    .map(|&i| u64::from(parent.cells[i]))
                    .sum::<u64>();
                calls += parent.calls + candidate.calls + replay.calls;
                let record = json!({"intervention":label,"record":e.id,"parent_loss":parent.loss,"changed_loss":candidate.loss,"delta":delta,"reference_delta":independent,"changed_trajectory_steps":changed_steps,"parameter_cell_visits":visits,"parent_calls":parent.calls,"changed_calls":candidate.calls,"artifact_replay_equal":true,"parent":parent.trace,"changed":candidate.trace,"parent_cell_counts":parent.cells,"changed_cell_counts":candidate.cells});
                std::fs::write(
                    path.join(format!("contrast-{}.json", contrasts.len())),
                    serde_json::to_vec(&record)?,
                )?;
                contrasts.push(json!({"intervention":label,"record":e.id,"delta":delta,"reference_delta":independent,"changed_trajectory_steps":changed_steps,"parameter_cell_visits":visits,"parent_calls":parent.calls,"changed_calls":candidate.calls}));
            }
        }
        let mut generation = Vec::new();
        let mut exact = 0usize;
        let mut full_exact = 0usize;
        let mut full_symbols = 0usize;
        let mut full_correct = 0usize;
        let mut full_loss = 0.;
        for e in training.iter().chain(&development) {
            let teacher = trajectory(&initial, &g, &m, e, Intervention::Full)?;
            full_symbols += teacher.positions;
            full_correct += teacher.correct;
            full_loss += teacher.loss;
            calls += teacher.calls;
            for control in [
                Intervention::Full,
                Intervention::ReadDisabled,
                Intervention::UpdateDisabled,
                Intervention::FirstRootOnly,
                Intervention::PayloadMasked,
            ] {
                let row = generate(&initial, &g, &m, e, control)?;
                let pass = row["exact"] == true;
                exact += usize::from(pass);
                if control == Intervention::Full {
                    full_exact += usize::from(pass);
                }
                calls += row["calls"].as_u64().ok_or("calls")?;
                std::fs::write(
                    path.join(format!("generation-{}.json", generation.len())),
                    serde_json::to_vec(&row)?,
                )?;
                generation.push(json!({"id":e.id,"control":format!("{control:?}"),"tokens":row["tokens"],"exact":pass,"eos":row["eos"],"calls":row["calls"]}));
            }
        }
        let pass = selected_changes > 0;
        let summary = json!({"integration_gate":if pass{"PASS_HAMMING_POLICY_TRAJECTORY_CONFORMANCE"}else{"FAIL_NO_FINITE_TRAJECTORY_EFFECT"},"language_gate":"UNQUALIFIED_INITIALIZED_NO_FIT","initialized_artifact":initial.id(),"parameter_digest":params.digest(),"gates":GATES,"contrasts":contrasts,"generation":generation,"full_exact":full_exact,"full_records":24,"all_controls_exact":exact,"full_teacher_symbols":full_symbols,"full_teacher_correct":full_correct,"full_teacher_mean_ce":full_loss/full_symbols as f64,"calls":calls,"elapsed_us":start.elapsed().as_micros(),"fit_calls":0,"optimizer_updates":0,"fixed_conformance_variants":2,"credit":"exact discrete finite contrast; not an analytic gradient, scalable optimizer or learned policy","heldout":"NOT_RUN; reused authored construction/development","generated_program_execution":"NOT_RUN; initialized path unqualified; see raw outputs","retained_model":"15baec48","promotion":false});
        std::fs::write(
            path.join("summary.json"),
            serde_json::to_vec_pretty(&summary)?,
        )?;
        if !pass {
            return Err("no finite trajectory effect".into());
        }
        Ok(())
    })();
    if let Err(e) = &result {
        std::fs::write(path.join("error.txt"), e.to_string())?;
    }
    report_output::seal(&path)?;
    report_output::verify(&path)?;
    result
}
