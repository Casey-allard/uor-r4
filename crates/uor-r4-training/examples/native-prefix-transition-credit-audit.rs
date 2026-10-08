//! Decompose authenticated saved Prefix temporal transition gradients.
//! No model forward, encoder, backward, export or finite displacement is run.
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::report_output;
use uor_r4_integer::geometric_context_q4::{unpack_coefficients, ContextQ4Config};
#[path = "native_prefix_transition_credit/margins.rs"]
mod native_prefix_transition_margins;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REPORT: &str = "1f7a51fe58e56862f6e8cd269225445bf9d42354d3cfe7eaf96a5910707568c9";
const SEAL: &str = "c43bbbe81eeea18338b2ea541c50f8f01d24dcfbe9a32662b73e2d2ebc03b332";
const PARENT_REPORT: &str = "c9b9fe10b6fbb4332cf919a5df7ba31403ad3d51ac6f877d94daeabd99672bee";
const PARENT_SEAL: &str = "de90ba0ba2809ed37ca868ef2bcf94b6ceb8b176a60b86ae88801876dafbd0e5";
const SOURCE: &str = "9f0b272e7852a47bbbad0f549e8157a44ff3212af86905907501ec11f3e7989b";
const FAMILIES: [&str; 2] = [
    "consumer.context.self_transition",
    "consumer.context.neighbor_transition",
];
const BASIS: [&str; 6] = [
    "self_transition",
    "neighbor_transition",
    "self_root",
    "neighbor_root",
    "self_category",
    "neighbor_category",
];
const CAP: u64 = 16 * 1024 * 1024;
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
fn hash(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn read(p: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(p)?)?)
}
fn pinned(root: &Path, report: &str, seal: &str) -> Result<Value> {
    report_output::verify(root)?;
    need(
        hash(&fs::read(root.join("report.json"))?) == report
            && hash(&fs::read(root.join("manifest.json"))?) == seal,
        "saved authority report/seal pin differs",
    )?;
    let r = read(&root.join("report.json"))?;
    need(r["status"] == "COMPLETED", "saved authority incomplete")?;
    Ok(r)
}
fn number(v: &Value) -> Result<usize> {
    Ok(v.as_u64()
        .ok_or_else(|| bad("receipt number absent"))?
        .try_into()?)
}
fn raw(root: &Path, r: &Value, file: &str, shape: &[usize]) -> Result<Vec<f32>> {
    let count = shape.iter().product::<usize>();
    need(
        r["file"] == file
            && r["shape"]
                .as_array()
                .is_none_or(|_| r["shape"] == json!(shape)),
        "raw file/shape receipt differs",
    )?;
    need(
        number(&r["elements"])? == count && number(&r["bytes"])? == count * 4,
        "raw receipt element/byte count differs",
    )?;
    let b = fs::read(root.join(file))?;
    need(
        b.len() == count * 4 && r["sha256"] == hash(&b),
        "raw f32 length/hash differs",
    )?;
    let values = b
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect::<Vec<_>>();
    need(values.iter().all(|v| v.is_finite()), "raw nonfinite f32")?;
    Ok(values)
}
#[derive(Clone, Serialize, Debug, PartialEq, Eq)]
struct Coordinate {
    head: usize,
    lane: usize,
    class: usize,
    basis_component: usize,
}
fn coordinate(index: usize) -> Result<Coordinate> {
    need(index < 3840, "transition flattened coordinate out of range")?;
    Ok(Coordinate {
        head: index / (4 * 120 * 4),
        lane: (index / (120 * 4)) % 4,
        class: (index / 4) % 120,
        basis_component: index % 4,
    })
}
fn q4(master: f32) -> Result<i8> {
    need(master.is_finite(), "nonfinite parent master")?;
    let q = (master * 4.).round();
    need(
        (-7.0..=7.0).contains(&q),
        "parent master rounds outside native signed Q4",
    )?;
    Ok(q as i8)
}
#[derive(Clone, Serialize)]
struct Row {
    name: String,
    index: usize,
    decoded: Coordinate,
    before: i8,
    after: i8,
    master_before: f32,
    master_after: f32,
    actual_delta: f64,
    g_off: f32,
    g_on: f32,
    g_difference_f64: f64,
    raw_difference_f32: f32,
    dot_on: f64,
    dot_off: f64,
    temporal_dot: f64,
}
#[derive(Default, Serialize)]
struct Counts {
    prospective_scalars: usize,
    missing_scalars: usize,
    eligible_scalars: usize,
    zero_direction_gradient: usize,
    saturated: usize,
    legal_changed: usize,
    negative_temporal_dot: usize,
    negative_on_dot: usize,
    negative_off_dot: usize,
}
fn make(
    name: &str,
    index: usize,
    m: f32,
    off: f32,
    on: f32,
    difference: f32,
    direction: f64,
) -> Result<Option<Row>> {
    need(
        off.is_finite() && on.is_finite() && direction.is_finite() && difference.is_finite(),
        "nonfinite decomposition gradient",
    )?;
    let before = q4(m)?;
    let after = (before
        + if direction > 0. {
            -1
        } else if direction < 0. {
            1
        } else {
            0
        })
    .clamp(-7, 7);
    if after == before {
        return Ok(None);
    }
    let master_after = f32::from(after) * 0.25;
    let delta = f64::from(master_after) - f64::from(m);
    let g_difference = f64::from(on) - f64::from(off);
    Ok(Some(Row {
        name: name.into(),
        index,
        decoded: coordinate(index)?,
        before,
        after,
        master_before: m,
        master_after,
        actual_delta: delta,
        g_off: off,
        g_on: on,
        g_difference_f64: g_difference,
        raw_difference_f32: difference,
        dot_on: f64::from(on) * delta,
        dot_off: f64::from(off) * delta,
        temporal_dot: g_difference * delta,
    }))
}
fn top(rows: &[Row], score: impl Fn(&Row) -> f64) -> Vec<Row> {
    let mut ranked = rows.to_vec();
    ranked.sort_by(|a, b| {
        score(a)
            .total_cmp(&score(b))
            .then(a.name.cmp(&b.name))
            .then(a.index.cmp(&b.index))
    });
    ranked.truncate(20);
    ranked
}
struct Writer {
    root: PathBuf,
    written: u64,
}
impl Writer {
    fn write(&mut self, name: &str, v: &Value) -> Result<Value> {
        let b = serde_json::to_vec(v)?;
        need(
            self.written + b.len() as u64 + 1_048_576 <= CAP,
            "decomposition report cap exceeded",
        )?;
        fs::write(self.root.join(name), &b)?;
        self.written += b.len() as u64;
        Ok(json!({"file":name,"bytes":b.len(),"sha256":hash(&b)}))
    }
}
fn arm(root: &Path, r: &Value, name: &str, enabled: bool) -> Result<Option<Vec<f32>>> {
    need(
        r["prefix_temporal_utility"] == enabled
            && r["continuation_context_credit"] == true
            && r["read_selector_credit"] == true
            && r["categorical_bridge_credit"] == true
            && r["backward_calls"] == 18
            && r["full_cohort_validated_before_any_backward"] == true,
        "saved gradient arm scope differs",
    )?;
    let family = &r["families"][name];
    let label = if enabled { "on" } else { "off" };
    need(
        family["shape"] == json!([2, 4, 120, 4]) && family["missing_gradient_filled_zero"] == false,
        "transition gradient shape/missing policy differs",
    )?;
    let terms = r["terms"]
        .as_array()
        .ok_or_else(|| bad("gradient term inventory absent"))?;
    need(terms.len() == 18, "gradient term population differs")?;
    let mut missing = false;
    for term in terms {
        match term["gradient_presence"][name].as_str() {
            Some("PRESENT") => {}
            Some("MISSING") => missing = true,
            _ => return Err(bad("gradient presence state invalid")),
        }
    }
    need(
        family["status"] == if missing { "MISSING" } else { "PRESENT" },
        "gradient status disagrees with term connectivity",
    )?;
    let values = if family["raw"].is_object() {
        Some(raw(
            root,
            &family["raw"],
            &format!("context-gradients/{label}/{name}.f32le"),
            &[2, 4, 120, 4],
        )?)
    } else {
        None
    };
    if missing {
        Ok(None)
    } else {
        Ok(Some(
            values.ok_or_else(|| bad("present gradient raw file absent"))?,
        ))
    }
}
fn run(root: &Path, parent: &Path, w: &mut Writer) -> Result<Value> {
    let clock = Instant::now();
    let r = pinned(root, REPORT, SEAL)?;
    need(
        r["schema"] == "uor-r4.prefix-context-credit-probe/1"
            && r["mode"] == "prefix_context_credit"
            && r["parent_report_sha256"] == PARENT_REPORT
            && r["parent_manifest_sha256"] == PARENT_SEAL
            && r["parent_master_bits_restored"] == true
            && r["selected_model"] == false
            && r["optimizer_updates"] == 0,
        "fixed Prefix Context probe identity differs",
    )?;
    let parent_report = pinned(parent, PARENT_REPORT, PARENT_SEAL)?;
    let metadata = read(&parent.join("checkpoint-0001/native/metadata.json"))?;
    need(
        parent_report["final_receipt"]["parent"]["metadata_sha256"] == SOURCE
            && hash(&fs::read(
                parent.join("checkpoint-0001/native/metadata.json"),
            )?) == SOURCE,
        "original Source metadata identity differs",
    )?;
    let cfg = ContextQ4Config {
        vocab_size: 4096,
        heads: 2,
        lanes_per_head: 4,
    };
    let shapes = cfg.coefficient_shapes()?;
    let initial = read(&root.join("context-initial-masters.json"))?;
    need(
        initial["native_layout"] == serde_json::to_value(&shapes)?
            && initial["proposal_scalars"] == 17472
            && initial["parent_report_sha256"] == PARENT_REPORT,
        "original Context packed layout/master receipt differs",
    )?;
    let master_map = initial["families"]
        .as_object()
        .ok_or_else(|| bad("initial master inventory absent"))?;
    need(
        master_map.len() == 6
            && BASIS
                .iter()
                .all(|n| master_map.contains_key(&format!("consumer.context.{n}"))),
        "six master family population differs",
    )?;
    let packed = fs::read(parent.join("checkpoint-0001/native/consumer/context-q4.bin"))?;
    let codes = unpack_coefficients(cfg.coefficient_count()?, &packed)?;
    let mut masters = BTreeMap::new();
    let mut offsets = BTreeMap::new();
    let mut offset = 0;
    let mut scalars = 0;
    for (short, shape) in &shapes {
        let name = format!("consumer.context.{short}");
        let count = shape.iter().product::<usize>();
        if let Some(receipt) = master_map.get(&name) {
            need(
                receipt["shape"] == json!(shape)
                    && receipt["sha256"] == metadata["source_parameters"][&name]["f32_sha256"]
                    && metadata["source_parameters"][&name]["shape"] == json!(shape),
                "initial fractional master Source identity differs",
            )?;
            let values = raw(
                root,
                receipt,
                &format!("context-initial-masters/{name}.f32le"),
                shape,
            )?;
            for (i, m) in values.iter().enumerate() {
                need(
                    q4(*m)? == codes[offset + i],
                    "actual fractional master/native quarter code differs",
                )?;
            }
            scalars += values.len();
            offsets.insert(name.clone(), offset);
            masters.insert(name, values);
        }
        offset += count;
    }
    need(
        scalars == 17472 && offset == codes.len(),
        "original native/master scalar count differs",
    )?;
    let off = read(&root.join("gradient-off-receipt.json"))?;
    let on = read(&root.join("gradient-on-receipt.json"))?;
    need(
        off["families"] == r["gradient_off"] && on["families"] == r["gradient_on"],
        "report/raw gradient receipt inventory differs",
    )?;
    let difference = read(&root.join("gradient-difference-receipt.json"))?;
    let mut on_rows = Vec::new();
    let mut difference_rows = Vec::new();
    let mut on_count = Counts::default();
    let mut difference_count = Counts::default();
    let mut eligibility = Vec::new();
    for name in FAMILIES {
        on_count.prospective_scalars += 3840;
        difference_count.prospective_scalars += 3840;
        let x = arm(root, &off, name, false)?;
        let y = arm(root, &on, name, true)?;
        let receipt = &difference[name];
        need(
            receipt["shape"] == json!([2, 4, 120, 4]),
            "saved difference shape differs",
        )?;
        let (Some(x), Some(y)) = (x, y) else {
            need(
                receipt["status"] == "UNAVAILABLE"
                    && receipt["missing_gradient_filled_zero"] == false,
                "missing gradient difference fabricated",
            )?;
            on_count.missing_scalars += 3840;
            difference_count.missing_scalars += 3840;
            eligibility.push(json!({"name":name,"status":"UNAVAILABLE","eligible_scalars":0}));
            continue;
        };
        need(
            receipt["status"] == "PRESENT"
                && receipt["subtraction"] == "f32 ON-minus-OFF after weighted f32 accumulation",
            "saved difference scope differs",
        )?;
        let delta = raw(
            root,
            receipt,
            &format!("context-gradients/difference/{name}.f32le"),
            &[2, 4, 120, 4],
        )?;
        need(
            y.iter()
                .zip(&x)
                .zip(&delta)
                .all(|((y, x), d)| (*y - *x).to_bits() == d.to_bits()),
            "saved f32 ON-minus-OFF differs",
        )?;
        eligibility.push(json!({"name":name,"status":"PRESENT","eligible_scalars":3840,"initial_master":master_map[name],"off":off["families"][name],"on":on["families"][name],"difference":receipt,"native_packed_family_offset":offsets[name]}));
        for i in 0..3840 {
            for (direction, rows, counts) in [
                (f64::from(y[i]), &mut on_rows, &mut on_count),
                (
                    f64::from(y[i]) - f64::from(x[i]),
                    &mut difference_rows,
                    &mut difference_count,
                ),
            ] {
                counts.eligible_scalars += 1;
                if direction == 0. {
                    counts.zero_direction_gradient += 1;
                    continue;
                }
                match make(name, i, masters[name][i], x[i], y[i], delta[i], direction)? {
                    None => counts.saturated += 1,
                    Some(row) => {
                        counts.legal_changed += 1;
                        counts.negative_temporal_dot += usize::from(row.temporal_dot < 0.);
                        counts.negative_on_dot += usize::from(row.dot_on < 0.);
                        counts.negative_off_dot += usize::from(row.dot_off < 0.);
                        rows.push(row);
                    }
                }
            }
        }
    }
    need(
        on_rows.len() <= 7680 && difference_rows.len() <= 7680,
        "transition row population exceeds declared bound",
    )?;
    let on_file = w.write(
        "on-directed-rows.json",
        &json!({"direction":"sign(g_ON)","rows":on_rows}),
    )?;
    let difference_file=w.write("difference-directed-rows.json",&json!({"direction":"sign(f64(g_ON)-f64(g_OFF))","diagnostic_only":true,"rows":difference_rows}))?;
    let rankings = json!({"on_directed":{"counts":on_count,"top20_dot_on":top(&on_rows,|r|r.dot_on),"top20_dot_off":top(&on_rows,|r|r.dot_off),"top20_temporal_dot":top(&on_rows,|r|r.temporal_dot)},
        "difference_directed":{"diagnostic_only":true,"counts":difference_count,"top20_temporal_dot":top(&difference_rows,|r|r.temporal_dot),"top20_dot_on":top(&difference_rows,|r|r.dot_on),"top20_dot_off":top(&difference_rows,|r|r.dot_off)}});
    w.write("rankings.json", &rankings)?;
    let mut margins = native_prefix_transition_margins::analyze(
        root,
        parent,
        &rankings["on_directed"]["top20_dot_on"],
    )?;
    let margin_file = w.write("saved-input-action-margins.json", &margins)?;
    margins
        .as_object_mut()
        .ok_or_else(|| bad("margin report object absent"))?
        .remove("rows");
    margins["saved_rows"] = margin_file;
    Ok(
        json!({"schema":"uor-r4.prefix-transition-credit-decomposition/1","status":"COMPLETED","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"saved_run_report_sha256":REPORT,"saved_run_manifest_sha256":SEAL,"source_metadata_sha256":SOURCE,
        "authenticated_master_basis_scalars":17472,"analyzed_transition_scalars":7680,"families":eligibility,"rows":{"on_directed":on_file,"difference_directed":difference_file},"rankings":rankings,
        "numerics":{"quarter_round":"f32(master*4).round ties-away; exact parent native packing; adjacent code clamp[-7,7]","actual_delta":"f64(q_after/4)-f64(original fractional master)","temporal_dot":"(f64(g_ON)-f64(g_OFF))*same actual_delta; recorded f32 difference is separately authenticated","rank_ties":"utility ascending, family name, flattened index","coordinate_shape":[2,4,120,4]},
        "saved_input_margins":margins,"model_forward":"NOT_RUN","encoder":"NOT_RUN","backward":"NOT_RUN","finite_displacement":"NOT_RUN","model_selection":"NOT_RUN","artifact_export":"NOT_RUN","elapsed_seconds":clock.elapsed().as_secs_f64(),"maximum_ranking_cache_bytes":67108864,"maximum_process_ram_bytes":1073741824,"maximum_report_bytes":CAP,
        "scope":"saved gradient decomposition only; ON-directed rankings and independent difference-directed diagnostics do not select or test a model; no new gradient, dose, serving behavior or predictive result"}),
    )
}
fn main() -> Result<()> {
    let argv = std::env::args().collect::<Vec<_>>();
    need(
        argv.len() == 4,
        "usage: native-prefix-transition-credit-audit RUN_ROOT PARENT_ROOT NEW_OUTPUT_ROOT",
    )?;
    let root = fs::canonicalize(&argv[1])?;
    need(root.is_dir(), "saved run root is not a directory")?;
    let source_parent = fs::canonicalize(&argv[2])?;
    need(
        source_parent.is_dir(),
        "Source parent root is not a directory",
    )?;
    let requested = PathBuf::from(&argv[3]);
    let parent = requested
        .parent()
        .ok_or_else(|| bad("output parent absent"))?;
    let output = fs::canonicalize(parent)?.join(
        requested
            .file_name()
            .ok_or_else(|| bad("output filename absent"))?,
    );
    need(
        !output.starts_with(&root)
            && !root.starts_with(&output)
            && !output.starts_with(&source_parent)
            && !source_parent.starts_with(&output),
        "output overlaps sealed input",
    )?;
    report_output::claim(&output)?;
    let mut writer = Writer {
        root: output.clone(),
        written: 0,
    };
    let result = run(&root, &source_parent, &mut writer);
    let mut report = match &result {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.prefix-transition-credit-decomposition/1","status":"FAILED","error":e.to_string(),"scope":"saved diagnostic execution failure; no model-quality verdict"})
        }
    };
    report["attempt_argv"] = json!(argv);
    let data = serde_json::to_vec(&report)?;
    let exceeded = writer.written + data.len() as u64 > CAP;
    if exceeded {
        report = json!({"schema":"uor-r4.prefix-transition-credit-decomposition/1","status":"FAILED","error":"final report cap exceeded"});
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
    #[test]
    fn transition_credit_fractional_actual_delta_and_separate_directions() -> Result<()> {
        let on = make(FAMILIES[0], 0, 0.1393287, -0.4, 0.1, 0.5, 0.1)?
            .ok_or_else(|| bad("test ON row absent"))?;
        assert_eq!((on.before, on.after), (1, 0));
        assert_eq!(on.actual_delta, -f64::from(0.1393287f32));
        assert!(on.dot_on < 0. && on.dot_off > 0. && on.temporal_dot < 0.);
        let opposite = make(FAMILIES[0], 0, 0.1393287, -0.4, -0.1, 0.3, 0.3)?
            .ok_or_else(|| bad("test difference row absent"))?;
        assert_eq!(opposite.after, 0);
        assert!(opposite.dot_on > 0. && opposite.temporal_dot < 0.);
        Ok(())
    }
    #[test]
    fn transition_credit_flattening_and_tie_order() -> Result<()> {
        assert_eq!(
            coordinate(3839)?,
            Coordinate {
                head: 1,
                lane: 3,
                class: 119,
                basis_component: 3
            }
        );
        assert!(coordinate(3840).is_err());
        for i in 0..3840 {
            let c = coordinate(i)?;
            assert_eq!(
                (((c.head * 4 + c.lane) * 120 + c.class) * 4) + c.basis_component,
                i
            );
        }
        let a = make("a", 1, 0.12, 1., -2., -3., -2.)?.ok_or_else(|| bad("test row absent"))?;
        let mut b = a.clone();
        b.index = 0;
        let mut c = b.clone();
        c.name = "b".into();
        let ranked = top(&[a, b, c], |r| r.dot_on);
        assert_eq!((ranked[0].name.as_str(), ranked[0].index), ("a", 0));
        Ok(())
    }
    #[test]
    fn transition_credit_missing_and_saturation_are_not_zero_proposals() -> Result<()> {
        let r = json!({"prefix_temporal_utility":false,"continuation_context_credit":true,"read_selector_credit":true,"categorical_bridge_credit":true,"backward_calls":18,"full_cohort_validated_before_any_backward":true,
            "families":{"consumer.context.self_transition":{"shape":[2,4,120,4],"status":"MISSING","missing_gradient_filled_zero":false}},"terms":(0..18).map(|_|json!({"gradient_presence":{"consumer.context.self_transition":"MISSING"}})).collect::<Vec<_>>()});
        assert!(arm(Path::new("/not-read-for-missing"), &r, FAMILIES[0], false)?.is_none());
        assert!(make(FAMILIES[0], 0, 1.75, 0., -1., -1., -1.)?.is_none());
        assert!(make(FAMILIES[0], 0, 0.12, 0., 0., 0., 0.)?.is_none());
        Ok(())
    }
}
