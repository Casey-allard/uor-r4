//! One recorded temporal-credit displacement at authenticated held-fixed inputs.
//! No reranking, encoder advance, backward, artifact export or model rollout.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::report_output;
#[path = "native_prefix_transition_credit/margins.rs"]
mod margins;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const DERIVED_REPORT: &str = "3a0c30dff9c8bd9b1f5c7a30251de432b90ef07b62f77ff79ab1d3b00bca0722";
const DERIVED_SEAL: &str = "a9b29208992b009fc55362116490d5ebe5746cd934f03ebd382b6b86c4b11b8b";
const PROBE_REPORT: &str = "1f7a51fe58e56862f6e8cd269225445bf9d42354d3cfe7eaf96a5910707568c9";
const PROBE_SEAL: &str = "c43bbbe81eeea18338b2ea541c50f8f01d24dcfbe9a32662b73e2d2ebc03b332";
const PARENT_REPORT: &str = "c9b9fe10b6fbb4332cf919a5df7ba31403ad3d51ac6f877d94daeabd99672bee";
const PARENT_SEAL: &str = "de90ba0ba2809ed37ca868ef2bcf94b6ceb8b176a60b86ae88801876dafbd0e5";
const SOURCE: &str = "9f0b272e7852a47bbbad0f549e8157a44ff3212af86905907501ec11f3e7989b";
const REPORT_CAP: u64 = 2 * 1024 * 1024;
fn bad(s: &str) -> Box<dyn std::error::Error> {
    std::io::Error::other(s).into()
}
fn need(b: bool, s: &str) -> Result<()> {
    if b {
        Ok(())
    } else {
        Err(bad(s))
    }
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn read(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn pinned(root: &Path, report: &str, seal: &str) -> Result<Value> {
    report_output::verify(root)?;
    need(
        hash(&fs::read(root.join("report.json"))?) == report
            && hash(&fs::read(root.join("manifest.json"))?) == seal,
        "temporal margin authority report/seal differs",
    )?;
    let value = read(&root.join("report.json"))?;
    need(
        value["status"] == "COMPLETED",
        "temporal margin authority incomplete",
    )?;
    Ok(value)
}
fn admitted(candidate: &Value) -> Result<()> {
    need(
        candidate["name"] == "consumer.context.self_transition"
            && candidate["index"] == 3610
            && candidate["before"] == -1
            && candidate["after"] == -2
            && candidate["decoded"] == json!({"head":1,"lane":3,"class":62,"basis_component":2}),
        "recorded temporal displacement identity differs",
    )?;
    for field in ["dot_on", "dot_off", "temporal_dot"] {
        need(
            candidate[field]
                .as_f64()
                .is_some_and(|v| v.is_finite() && v < 0.),
            "recorded temporal displacement must have three negative dots",
        )?;
    }
    let before = candidate["master_before"]
        .as_f64()
        .ok_or_else(|| bad("recorded fractional master absent"))?;
    let after = candidate["master_after"]
        .as_f64()
        .ok_or_else(|| bad("recorded target master absent"))?;
    need(
        before == f64::from(-0.1445201337337494f32)
            && after == -0.5
            && candidate["actual_delta"].as_f64() == Some(after - before),
        "recorded actual fractional displacement differs",
    )
}
fn nonoverlap(output: &Path, inputs: &[PathBuf]) -> Result<()> {
    need(
        inputs
            .iter()
            .all(|p| !output.starts_with(p) && !p.starts_with(output)),
        "output overlaps sealed input authority",
    )
}
struct Writer {
    root: PathBuf,
    written: u64,
}
impl Writer {
    fn write(&mut self, name: &str, value: &Value) -> Result<Value> {
        let data = serde_json::to_vec(value)?;
        need(
            self.written + data.len() as u64 + 65536 <= REPORT_CAP,
            "temporal margin report cap exceeded",
        )?;
        fs::write(self.root.join(name), &data)?;
        self.written += data.len() as u64;
        Ok(json!({"file":name,"bytes":data.len(),"sha256":hash(&data)}))
    }
}
fn task_carriers(probe: &Path, rows: &[Value]) -> Result<Value> {
    // Posthoc interpretation only: every local score row has already been
    // computed without selecting a task donor, word, target or carrier.
    let prev = read(&probe.join("baseline-row-0245-position-03.json"))?;
    let next = read(&probe.join("baseline-row-0245-position-04.json"))?;
    need(
        prev["input_index"] == 245
            && prev["position"] == 3
            && next["input_index"] == 245
            && next["position"] == 4
            && prev["actual_prefix_ids"] == json!([617, 2097, 315])
            && next["actual_prefix_ids"] == json!([617, 2097, 315, 1057]),
        "task actual prefix identity differs",
    )?;
    let donor = &prev["native"]["bridge"]["selected_candidate"];
    need(
        donor["occurrence"]["token_offset"] == 0 && donor["occurrence"]["token_id"] == 1057,
        "preceding factual donor is not recorded warm occurrence0",
    )?;
    let segment = donor["segment_index"]
        .as_u64()
        .ok_or_else(|| bad("preceding donor Source segment absent"))?;
    let source_scope = format!("Source segment{segment} token0");
    let source = rows
        .iter()
        .filter(|r| r["input_index"] == 245 && r["scope"] == source_scope)
        .cloned()
        .collect::<Vec<_>>();
    let response = rows
        .iter()
        .filter(|r| {
            r["input_index"] == 245
                && r["position"] == 4
                && r["scope"] == "Response last token at position4; old state from saved position3"
        })
        .cloned()
        .collect::<Vec<_>>();
    need(
        source.len() == 2
            && response.len() == 1
            && source
                .iter()
                .chain(&response)
                .all(|r| r["token_id"] == 1057),
        "recorded warm Source/response local row cardinality differs",
    )?;
    Ok(
        json!({"input_index":245,"actual_prefix_position3":prev["actual_prefix_ids"],"actual_prefix_position4":next["actual_prefix_ids"],"preceding_actual_donor":donor,
        "source_offset0_token":1057,"token_literal_posthoc":"Ġwarm","source_offset0_rows":source,"response_position4_rows":response,
        "source_old_state_scope":"isolated Source-before token0; selected segment comes from preceding factual bridge witness, not target filtering",
        "response_old_state":prev["native"]["bank_trace"]["prefix"]["response"]["states"],
        "annotation_scope":"attached after all181 held-input margins; preceding actual donor identity is distinct from independent expected-source-role authority; no claim that local crossing propagates to a fullpool fix"}),
    )
}
fn run(derived: &Path, probe: &Path, parent: &Path, w: &mut Writer) -> Result<Value> {
    let clock = Instant::now();
    let d = pinned(derived, DERIVED_REPORT, DERIVED_SEAL)?;
    let p = pinned(probe, PROBE_REPORT, PROBE_SEAL)?;
    let ancestor = pinned(parent, PARENT_REPORT, PARENT_SEAL)?;
    need(
        d["schema"] == "uor-r4.prefix-transition-credit-decomposition/1"
            && d["saved_run_report_sha256"] == PROBE_REPORT
            && d["saved_run_manifest_sha256"] == PROBE_SEAL
            && d["source_metadata_sha256"] == SOURCE
            && p["schema"] == "uor-r4.prefix-context-credit-probe/1"
            && p["parent_report_sha256"] == PARENT_REPORT
            && p["parent_manifest_sha256"] == PARENT_SEAL
            && ancestor["mode"] == "readout_intermediate_candidate"
            && ancestor["selected_model"] == true
            && ancestor["final_receipt"]["parent"]["metadata_sha256"] == SOURCE,
        "temporal margin fixed ancestry differs",
    )?;
    let rankings = read(&derived.join("rankings.json"))?;
    need(
        rankings == d["rankings"],
        "recorded ranking file/report differs",
    )?;
    let candidate = d["rankings"]["on_directed"]["top20_temporal_dot"]
        .as_array()
        .and_then(|r| r.first())
        .ok_or_else(|| bad("recorded temporal candidate absent"))?
        .clone();
    admitted(&candidate)?;
    let mut result = margins::analyze(probe, parent, &json!([candidate]))?;
    need(
        result["hypothetical_candidates"] == 1 && result["local_input_rows"] == 181,
        "declared one-candidate local margin population differs",
    )?;
    let rows = result["rows"]
        .as_array()
        .ok_or_else(|| bad("local margin rows absent"))?;
    let mut task_changes = 0;
    let mut reference_changes = 0;
    let mut task_rows = 0;
    let mut reference_rows = 0;
    for row in rows {
        need(
            row["name"] == candidate["name"] && row["index"] == candidate["index"],
            "local margin displacement identity differs",
        )?;
        let task = row["input_index"] == 245;
        if task {
            task_rows += 1;
            task_changes += usize::from(row["frozen_input_action_changed"] == true);
        } else {
            reference_rows += 1;
            reference_changes += usize::from(row["frozen_input_action_changed"] == true);
        }
    }
    let carriers = task_carriers(probe, rows)?;
    let saved = w.write("saved-input-action-margins.json", &result)?;
    result
        .as_object_mut()
        .ok_or_else(|| bad("local margin summary object absent"))?
        .remove("rows");
    result["saved_rows"] = saved;
    w.write("task-carriers.json", &carriers)?;
    Ok(
        json!({"schema":"uor-r4.prefix-temporal-margin/1","status":"COMPLETED","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"derived_report_sha256":DERIVED_REPORT,"derived_manifest_sha256":DERIVED_SEAL,
        "probe_report_sha256":PROBE_REPORT,"probe_manifest_sha256":PROBE_SEAL,"parent_report_sha256":PARENT_REPORT,"parent_manifest_sha256":PARENT_SEAL,
        "candidate":candidate,"candidate_authority":"recorded first on_directed.top20_temporal_dot entry; no rerank","saved_input_margins":result,
        "task245":{"local_rows":task_rows,"local_action_changes":task_changes,"scope":"both savedpos3/4 task carriers"},"references":{"local_rows":reference_rows,"local_action_changes":reference_changes,"scope":"16 source-control frames"},"task_carriers":carriers,
        "elapsed_seconds":clock.elapsed().as_secs_f64(),"maximum_report_bytes":REPORT_CAP,"maximum_process_ram_bytes":1073741824u64,"maximum_cpu_threads":2,
        "reranking":"NOT_RUN","backward":"NOT_RUN","encoder_advance":"NOT_RUN","finite_artifact":"NOT_RUN","model_forward":"NOT_RUN","autoregressive_rollout":"NOT_RUN",
        "scope":"exact native local operator margins at authenticated fixed inputs for one already recorded displacement; local crossings do not establish recurrent/fullpool/objective/complete-reply improvement"}),
    )
}
fn main() -> Result<()> {
    let argv = std::env::args().collect::<Vec<_>>();
    need(argv.len()==5,"usage: native-prefix-temporal-margin SAVED_DECOMPOSITION_ROOT PROBE_ROOT PARENT_ROOT NEW_OUTPUT_ROOT")?;
    let inputs = argv[1..4]
        .iter()
        .map(fs::canonicalize)
        .collect::<std::io::Result<Vec<_>>>()?;
    need(
        inputs.iter().all(|p| p.is_dir()),
        "input authority root is not a directory",
    )?;
    let path = PathBuf::from(&argv[4]);
    let parent = path.parent().ok_or_else(|| bad("output parent absent"))?;
    let output = fs::canonicalize(parent)?.join(
        path.file_name()
            .ok_or_else(|| bad("output filename absent"))?,
    );
    nonoverlap(&output, &inputs)?;
    report_output::claim(&output)?;
    let mut writer = Writer {
        root: output.clone(),
        written: 0,
    };
    let result = run(&inputs[0], &inputs[1], &inputs[2], &mut writer);
    let mut report = match &result {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.prefix-temporal-margin/1","status":"FAILED","error":e.to_string(),"scope":"saved arithmetic setup/execution failure; no scientific negative"})
        }
    };
    report["attempt_argv"] = json!(argv);
    let exceeded = writer.written + serde_json::to_vec(&report)?.len() as u64 > REPORT_CAP;
    if exceeded {
        report = json!({"schema":"uor-r4.prefix-temporal-margin/1","status":"FAILED","error":"final report cap exceeded"});
    }
    fs::write(output.join("report.json"), serde_json::to_vec(&report)?)?;
    report_output::seal(&output)?;
    report_output::verify(&output)?;
    if exceeded {
        return Err(bad("final report cap exceeded"));
    }
    result.map(|_| ())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn candidate() -> Value {
        json!({"name":"consumer.context.self_transition","index":3610,"before":-1,"after":-2,"decoded":{"head":1,"lane":3,"class":62,"basis_component":2},"dot_on":-0.002501638380482049,"dot_off":-0.0004952498372175033,"temporal_dot":-0.0020063885432645458,"master_before":f64::from(-0.1445201337337494f32),"master_after":-0.5,"actual_delta":-0.5-f64::from(-0.1445201337337494f32)})
    }
    #[test]
    fn temporal_margin_rejects_wrong_recorded_coordinate_or_ascent() -> Result<()> {
        let mut c = candidate();
        admitted(&c)?;
        c["index"] = json!(1660);
        assert!(admitted(&c).is_err());
        c = candidate();
        c["dot_on"] = json!(0.001);
        assert!(admitted(&c).is_err());
        c = candidate();
        c["actual_delta"] = json!(-0.25);
        assert!(admitted(&c).is_err());
        Ok(())
    }
    #[test]
    fn temporal_margin_rejects_output_ancestry_overlap() -> Result<()> {
        let inputs = vec![PathBuf::from("/saved/run"), PathBuf::from("/saved/parent")];
        nonoverlap(Path::new("/attempts/new"), &inputs)?;
        assert!(nonoverlap(Path::new("/saved/run/child"), &inputs).is_err());
        assert!(nonoverlap(Path::new("/saved"), &inputs).is_err());
        Ok(())
    }
}
