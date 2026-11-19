//! Finite memory and previously exposed source/cursor reproduction on Wick.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    env, fs,
    hint::black_box,
    mem::size_of_val,
    ops::Range,
    path::Path,
    sync::atomic::{AtomicUsize, Ordering::SeqCst},
};
use uor_r4_integer::discrete_d11::{ExactCopyState, IntegerDiscreteModel, ROLE_COPY_STOPS};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

struct CountingSystem;

#[global_allocator]
static ALLOCATOR: CountingSystem = CountingSystem;

// The fixture counts requested sizes and otherwise forwards the identical
// pointer and layout to System. Rust's default realloc is counted as the
// simultaneous old/new blocks while it copies.
unsafe impl GlobalAlloc for CountingSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let address = unsafe { System.alloc(layout) };
        if !address.is_null() {
            let live = LIVE.fetch_add(layout.size(), SeqCst) + layout.size();
            PEAK.fetch_max(live, SeqCst);
        }
        address
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), SeqCst);
        unsafe { System.dealloc(ptr, layout) };
    }
}

fn scope<T>(run: impl FnOnce() -> T) -> (T, usize, usize) {
    let before = LIVE.load(SeqCst);
    PEAK.store(before, SeqCst);
    let result = black_box(run());
    let after = LIVE.load(SeqCst);
    let peak = PEAK.load(SeqCst);
    (
        result,
        after
            .checked_sub(before)
            .expect("scope released previous allocations"),
        peak - before,
    )
}

fn load(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn hash_ids(ids: &[u32]) -> String {
    let mut hash = Sha256::new();
    for id in ids {
        hash.update(id.to_le_bytes());
    }
    format!("{:x}", hash.finalize())
}

fn memory(root: &Path, trunk: &Path, seed: u8) -> Result<Value> {
    let inputs = load(&root.join(format!("s{seed}-pack.json")))?;
    let ids: Vec<u32> = serde_json::from_value(inputs["prefix"].clone())?;
    let raw_trunk = fs::read(trunk)?;
    let (baseline, trunk_live, trunk_parse_peak) =
        scope(|| IntegerDiscreteModel::from_bytes(&raw_trunk));
    let baseline = baseline?;
    assert_eq!(baseline.artifact_sha256(), inputs["trunk_sha256"]);
    let inline_model = size_of_val(&baseline);
    drop(baseline);
    let mut arms = Vec::new();
    for mode in 0..2 {
        let path = root.join(format!("positive-s{seed}-{mode}.d11"));
        let bytes = fs::read(&path)?;
        let (model, loaded_live, load_peak) = scope(|| IntegerDiscreteModel::from_bytes(&bytes));
        let model = model?;
        let framing = model.ordered_role_framing().ok_or("missing framing")?;
        let bank = model.ordered_role_bank().ok_or("missing positive bank")?;
        assert_eq!(bank.mode as usize, mode);
        let prompt = model
            .ordered_role_sites()?
            .prompt(&ids, framing.content(&ids)?)?;
        let (roles, operator_live, operator_peak) =
            scope(|| prompt.predict(bank.mode, &bank.prototypes, 0..ids.len()));
        let roles = roles?;
        let requested = roles.requested.ok_or("finite positive bank abstained")?;
        let (decision, public_live, public_peak) = scope(|| model.paired_copy_decision(&ids, true));
        let decision = decision?.ok_or("missing public decision")?;
        assert_eq!(decision.choice.source, Some(7));
        assert_eq!(decision.role_words, Some(roles.words()));
        let clipped = requested.start..requested.end;
        assert_eq!(
            prompt.feature_positions(requested, clipped.clone())?,
            [None; 24]
        );
        assert_eq!(prompt.feature(bank.mode, requested, clipped)?, [0; 32]);
        // The same real prompt, shifted left of the requested span, drops
        // that complete candidate. Only membership/features are intervened on.
        let shifted = 0..requested.start;
        let shifted_rows = model.ordered_role_trace(&ids, shifted.clone())?;
        let dropped = shifted_rows
            .iter()
            .find(|item| item.span == [requested.start, requested.end])
            .ok_or("shifted trace lost candidate non-membership")?;
        assert!(!dropped.admitted);
        assert!(dropped.feature.is_none());
        let shifted_role = prompt.predict(bank.mode, &bank.prototypes, shifted)?;
        assert_ne!(shifted_role.requested, Some(requested));
        arms.push(json!({
            "mode":mode,"artifact":model.artifact_sha256(),"artifact_bytes":bytes.len(),
            "inline_bank":size_of_val(bank),"model_live_heap":loaded_live,"model_load_peak_heap":load_peak,
            "added_live_heap":loaded_live-trunk_live,"operator_retained_heap":operator_live,
            "operator_peak_heap":operator_peak,"read_retained_heap":public_live,"read_peak_heap":public_peak,
            "full_role":roles.words(),"full_source":decision.choice.source,"shifted_role":shifted_role.words(),
            "positive_clipped_witness":{"complete_span":[requested.start,requested.end],"all_slots_masked":true},
            "negative_shifted_witness":{"region":[0,requested.start],"complete_span_admitted":dropped.admitted},
        }));
    }
    for field in [
        "artifact_bytes",
        "inline_bank",
        "model_live_heap",
        "model_load_peak_heap",
        "added_live_heap",
        "operator_retained_heap",
        "operator_peak_heap",
        "read_retained_heap",
        "read_peak_heap",
    ] {
        assert_eq!(arms[0][field], arms[1][field], "mode mismatch at {field}");
    }
    assert_eq!(arms[0]["operator_peak_heap"], 0);
    Ok(
        json!({"seed":seed,"base":inputs["trunk_sha256"],"base_live_heap":trunk_live,
              "inline_model":inline_model,"base_load_peak_heap":trunk_parse_peak,"arms":arms}),
    )
}

fn range(row: &Value) -> Result<Range<usize>> {
    Ok(serde_json::from_value::<usize>(row["start"].clone())?
        ..serde_json::from_value::<usize>(row["end"].clone())?)
}

fn exposed(study: &Path, artifact: &Path, seed: u8, split: &str, id: &str) -> Result<Value> {
    let panels = study.join("region/panels");
    let grades = load(&panels.join(format!("{split}-grade.json")))?;
    let row = grades
        .as_array()
        .ok_or("historical grade shape")?
        .iter()
        .position(|g| g["id"] == id)
        .ok_or("historical fixture id missing")?;
    let blind = load(&panels.join(format!("{split}-blind.json")))?;
    let bounds = load(&panels.join(format!("{split}-regions.json")))?;
    let adverse = load(&panels.join(format!("{split}-adverse.json")))?;
    let ids: Vec<u32> = serde_json::from_value(blind[row]["prefix"].clone())?;
    let original = hash_ids(&ids);
    for source in [&bounds, &adverse, &grades] {
        assert_eq!(source[row]["prefix"], blind[row]["prefix"]);
    }
    let model = IntegerDiscreteModel::load(artifact)?;
    let runs = study.join(format!("writer/primary/seed{seed}-{split}"));
    let old_roles = study.join("region/primary");
    let mut arms = Vec::new();
    for (arm, region) in [
        ("original", 0..ids.len()),
        ("identity", 0..ids.len()),
        ("region", range(&bounds[row])?),
        ("adverse", range(&adverse[row])?),
    ] {
        let rf = old_roles.join(format!("seed{seed}-{split}-{arm}.json"));
        let wf = runs.join(arm).join("chosen.json");
        let frozen_role = load(&rf)?;
        let frozen_trajectory = load(&wf)?;
        let historical = &frozen_trajectory["rows"][row];
        let roles = model.paired_roles_diagnostic(&ids, region.clone(), false)?;
        let current = model.paired_copy_diagnostic(&ids, roles, None)?;
        if arm == "original" {
            assert_eq!(
                serde_json::to_value(model.paired_copy_decision(&ids, true)?)?,
                serde_json::to_value(&current)?
            );
        }
        assert_eq!(frozen_role["rows"][row]["prefix_sha256"], original);
        assert_eq!(frozen_role["rows"][row]["words"], json!(roles.words()));
        assert_eq!(historical["prefix_sha256"], original);
        assert_eq!(historical["source"], json!(current.choice.source));
        let mut context = ids.clone();
        let mut cursor = ExactCopyState::new(current.choice.source, ids.len() as u32);
        let mut generated = Vec::new();
        let steps = historical["steps"]
            .as_array()
            .ok_or("missing historical steps")?;
        for step in steps {
            let ordinary = model.evaluate_prefix(&context)?;
            let observed = cursor
                .write(&ids, ordinary.prediction, ROLE_COPY_STOPS)?
                .finish_with_eos(true, 1);
            assert_eq!(step["before"], json!(cursor.words()));
            assert_eq!(step["native_id"], ordinary.prediction);
            assert_eq!(step["native_top8"], json!(ordinary.top8));
            assert_eq!(step["emitted"], observed.emitted);
            assert_eq!(step["release"], observed.release);
            assert_eq!(step["copied"], observed.copied);
            assert_eq!(step["after"], json!(observed.state.words()));
            assert_eq!(
                step["source_id"],
                json!((observed.copied == 1).then(|| ids[cursor.cursor as usize]))
            );
            cursor = observed.state;
            generated.push(observed.emitted);
            context.push(observed.emitted);
        }
        assert_eq!(historical["generated"], json!(generated));
        assert!(generated.last() == Some(&1) || generated.len() == 16);
        arms.push(json!({
            "arm":arm,"region":[region.start,region.end],
            "frozen_roles_sha256":format!("{:x}",Sha256::digest(fs::read(rf)?)),
            "frozen_writer_sha256":format!("{:x}",Sha256::digest(fs::read(wf)?)),
            "words":roles.words(),"selected":current.choice,"generated":generated,"steps":steps.len(),
        }));
    }
    Ok(
        json!({"seed":seed,"panel":split,"id":id,"row":row,"prefix_sha256":original,
              "carrier":model.artifact_sha256(),"historical_expected":grades[row]["expected"],"arms":arms}),
    )
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let out = match args.as_slice() {
        [op, directory, trunk, seed] if op == "memory" => {
            memory(Path::new(directory), Path::new(trunk), seed.parse()?)?
        }
        [op, study, artifact, seed, split, id] if op == "exposed" => exposed(
            Path::new(study),
            Path::new(artifact),
            seed.parse()?,
            split,
            id,
        )?,
        _ => return Err("use memory DIR TRUNK SEED or exposed PRIOR CARRIER SEED SPLIT ID".into()),
    };
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}
