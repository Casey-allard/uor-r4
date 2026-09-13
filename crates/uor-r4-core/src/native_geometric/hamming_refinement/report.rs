use super::{
    metric::{census, Metric},
    runtime::refine,
    tests::{fixture, selected_bytes},
};
use crate::native_geometric::addressed_attention::artifact::BoundGeometry;
use crate::report_output;
#[test]
#[ignore = "exclusive finite Hamming mechanism report, no language fitting"]
fn hamming_refinement_report() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from(std::env::var("UOR_HAMMING_REPORT")?);
    if path.as_os_str().is_empty() {
        return Err("empty report path".into());
    }
    report_output::claim(&path)?;
    let run = (|| -> Result<(), Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        let g = BoundGeometry::canonical()?;
        let metric = Metric::new(&g)?;
        let geometry = census(&g)?;
        let mut rows = Vec::new();
        for (name, changed, reordered, freeze, thin) in [
            ("full", false, false, false, false),
            ("changed-fourth-root", true, false, false, false),
            ("relocated-records", false, true, false, false),
            ("update-disabled", false, false, true, false),
            ("first-root-only", false, false, false, true),
        ] {
            let (m, mut p, initial, _) = fixture(&g, changed, reordered);
            p.suppress_update = freeze;
            p.first_root_only = thin;
            let r = refine(&m, &g, &metric, initial, 2, &mut p)?;
            let hops=r.hops[..2].iter().flatten().map(|h|serde_json::json!({"round":h.round,"query":h.query.roots,"null":h.query.null,"distance":h.distance,"candidate_scores":h.candidate_scores,"before":h.before,"delta":h.delta,"after":h.after,"selected_reference":h.selected.map(|l|format!("{:?}",l.reference())),"selected_roots":h.selected.map(|l|l.roots()),"selected_payload":h.selected.map(|l|l.payload().to_vec()),"parent_digest":h.parent_digest,"digest":h.digest})).collect::<Vec<_>>();
            let selected = selected_bytes(&r);
            let expected = if freeze || thin {
                b"LL"
            } else if changed {
                b"LS"
            } else {
                b"LR"
            };
            let pass = selected
                .iter()
                .zip(expected)
                .all(|(bytes, expected)| bytes.as_deref() == Some(&[*expected]));
            rows.push(serde_json::json!({"case":name,"pass":pass,"selected_bytes":selected,"hops":hops,"trace_digest":r.digest,"memory_revision":m.revision(),"policy":"authored finite causal fixture; NOT_TRAINED"}));
        }
        let pass = rows.iter().all(|r| r["pass"] == true);
        let summary = serde_json::json!({"status":if pass{"PASS_HAMMING_REFINEMENT_PRIMITIVES"}else{"FAIL_HAMMING_REFINEMENT_PRIMITIVES"},"scope":"actual metric and generic query/read/update kernel with authored finite policy; no emission model or fitted policy","cases":rows,"geometry":geometry,"elapsed_us":start.elapsed().as_micros(),"fit_calls":0,"parameter_updates":0,"generated_language":"NOT_RUN","learned_refinement":"NOT_IMPLEMENTED","retained_model":"15baec48","promotion":false});
        std::fs::write(
            path.join("summary.json"),
            serde_json::to_vec_pretty(&summary)?,
        )?;
        if !pass {
            return Err("finite causal case failed".into());
        }
        Ok(())
    })();
    if let Err(e) = &run {
        std::fs::write(path.join("error.txt"), e.to_string())?;
    }
    report_output::seal(&path)?;
    report_output::verify(&path)?;
    run
}
