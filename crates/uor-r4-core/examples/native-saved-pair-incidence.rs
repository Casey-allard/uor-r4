//! Saved factor incidence and historical bridge transport; no generator, token scoring, reduction, gradient or proposals.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{
    native_geometric::learner::{
        geometric_generate::{GenerateReadCounts, NativeGeometricGenerate},
        geometric_read_state_bridge::{BridgeReadCounts, NativeGeometricReadStateBridge},
    },
    report_output,
};
use uor_r4_integer::{
    geometric_source_actions::SourceActionBinding,
    geometric_vocabulary_actions::NativeVocabularyActions, h4_tables::H4Code,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REPORT: &str = "0aa9b3c0c2d1defaaa6d93611a78fb348be0016aefe5d7548563250b9a687b8c";
const MANIFEST: &str = "e7667312b7d18b502e72b0fc41bf2d7d52a4977a020501ecd527da6ef628f9aa";
fn require(ok: bool, message: &str) -> Result<()> {
    if !ok {
        return Err(io::Error::new(io::ErrorKind::InvalidData, message).into());
    }
    Ok(())
}
fn read(p: &Path) -> Result<Value> {
    Ok(serde_json::from_reader(io::BufReader::new(
        fs::File::open(p)?,
    ))?)
}
fn hash(p: &Path) -> Result<String> {
    use std::io::Read;
    let mut f = fs::File::open(p)?;
    let mut h = Sha256::new();
    let mut b = [0; 65536];
    loop {
        let n = f.read(&mut b)?;
        if n == 0 {
            break;
        }
        h.update(&b[..n]);
    }
    Ok(hex::encode(h.finalize()))
}
fn index(v: &Value) -> Result<usize> {
    Ok(usize::try_from(v.as_u64().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "missing unsigned index")
    })?)?)
}
fn token(v: &Value) -> Result<u32> {
    Ok(u32::try_from(index(v)?)?)
}
fn array(v: &Value) -> Result<&Vec<Value>> {
    v.as_array()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing array").into())
}
fn codes(v: &Value) -> Result<Vec<H4Code>> {
    let a = array(v)?;
    require(a.len() == 8, "state must contain eight H4 codes")?;
    a.iter()
        .map(|x| Ok(H4Code::try_from(u8::try_from(index(x)?)?)?))
        .collect()
}
fn raw_codes(v: &[H4Code]) -> Vec<u8> {
    v.iter().map(|x| x.index()).collect()
}
fn keys(
    g: &NativeGeometricGenerate,
    post: &[H4Code],
    t: u32,
    counts: &mut GenerateReadCounts,
) -> Result<[u32; 12]> {
    let mut k = [0; 12];
    g.factor_incidence_into(post, t as usize, &mut k, counts)?;
    Ok(k)
}
fn contrast(
    g: &NativeGeometricGenerate,
    post: &[H4Code],
    target: u32,
    rival: u32,
    counts: &mut GenerateReadCounts,
) -> Result<Value> {
    let a = keys(g, post, target, counts)?;
    let b = keys(g, post, rival, counts)?;
    let mut delta = BTreeMap::<u32, i32>::new();
    for &k in &a {
        *delta.entry(k).or_default() += 1;
    }
    for &k in &b {
        *delta.entry(k).or_default() -= 1;
    }
    delta.retain(|_, v| *v != 0);
    Ok(
        json!({"target":target,"rival":rival,"post_state":raw_codes(post),"target_keys":a,"rival_keys":b,
        "unary_distinct_lanes":a[..8].iter().zip(&b[..8]).filter(|(a,b)|a!=b).count(),
        "pair_distinct_edges":a[8..].iter().zip(&b[8..]).filter(|(a,b)|a!=b).count(),
        "signed_target_minus_rival_incidence":delta.into_iter().map(|(k,n)|json!([k,n])).collect::<Vec<_>>()}),
    )
}
#[derive(Clone)]
struct Row {
    identity: Value,
    path: PathBuf,
    target: u32,
    winner: u32,
    competitor: u32,
    donor: usize,
    post: Vec<H4Code>,
    original_donor: usize,
    original_post: Vec<H4Code>,
    guard: bool,
    reference: bool,
}
fn bridge_post(
    row: &Row,
    donor: usize,
    bridge: &NativeGeometricReadStateBridge,
) -> Result<Vec<H4Code>> {
    let v = read(&row.path)?;
    let n = &v["native"];
    let bank = &n["bank_trace"]["cue_bank"]["bank"];
    let candidate = array(&bank["candidates"])?
        .get(donor)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "physical donor missing"))?;
    require(
        index(&candidate["bank_index"])? == donor,
        "physical donor ordinal mismatch",
    )?;
    let source = array(&bank["context"]["states"])?
        .get(index(&candidate["context_position"])?)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "source context position missing",
            )
        })?;
    let query = codes(&n["bridge"]["query_state"])?;
    let source = codes(source)?;
    let mut post = vec![H4Code::IDENTITY; 8];
    let mut actions = post.clone();
    let mut scores = vec![0; 960];
    bridge.apply_into(
        &query,
        &source,
        &mut post,
        &mut actions,
        &mut scores,
        &mut BridgeReadCounts::default(),
    )?;
    Ok(post)
}
fn selected_alt(record: &Value) -> Result<Option<&Value>> {
    match record["selected"]["status"].as_str() {
        Some("committed") => {
            let code = &record["selected"]["code"];
            let found = array(&record["alternatives"])?
                .iter()
                .filter(|a| &a["code"] == code)
                .collect::<Vec<_>>();
            require(
                found.len() == 1 && found[0]["feasible"] == true,
                "selected alternative not uniquely feasible",
            )?;
            Ok(Some(found[0]))
        }
        Some("unchanged") | Some("noop") => Ok(None),
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, "unknown selected status").into()),
    }
}
// Reverse only committed Prefix changes; rejected alternatives never alter history.
fn undo_change(current_donor: usize, current_post: &[H4Code], change: &Value) -> Result<usize> {
    let a = array(change)?;
    require(a.len() == 6, "changed_rows tuple shape")?;
    require(
        current_donor == index(&a[2])? && current_post == codes(&a[3])?,
        "reverse committed donor/post differs",
    )?;
    index(&a[1])
}
fn event_state(incumbent: &[H4Code], row: usize, family: &str, alt: &Value) -> Result<Vec<H4Code>> {
    if family == "prefix.coefficients" {
        for c in array(&alt["changed_rows"])? {
            if index(&c[0])? == row {
                return codes(&c[3]);
            }
        }
    }
    Ok(incumbent.to_vec())
}
fn analyze(run: &Path, out: &Path) -> Result<Value> {
    let start = Instant::now();
    require(
        hash(&run.join("report.json"))? == REPORT && hash(&run.join("manifest.json"))? == MANIFEST,
        "completed run pins differ",
    )?;
    report_output::verify(run)?;
    let report = read(&run.join("report.json"))?;
    require(
        report["status"] == "COMPLETED"
            && report["candidate_native_steps"] == 391
            && report["selected_model"] == false,
        "completed unselected391 authority",
    )?;
    let cp = run.join("checkpoint-0001");
    let binding = SourceActionBinding::new(&fs::read(cp.join("native/tokenizer.json"))?)?;
    let g = NativeGeometricGenerate::from_bytes(&fs::read(cp.join("generate.bin"))?, &binding)?;
    let bridge = NativeGeometricReadStateBridge::from_bytes(
        &fs::read(cp.join("read-state-bridge-categorical.bin"))?,
        &binding,
    )?;
    let reducer =
        NativeVocabularyActions::new(binding, &fs::read(cp.join("native/consumer/exp-q31.bin"))?)?;
    require(
        g.lanes() == 8
            && g.energy().edges().len() == 4
            && reducer.legal_token_ids().iter().copied().eq(0..4096),
        "eight lanes/four declared edges/full legal vocabulary required",
    )?;
    let population = read(&run.join("coupled-population.json"))?;
    let p_rows = array(&population["rows"])?;
    require(
        p_rows.len() == 391
            && population["guards"] == 380
            && population["weighted_roles"] == 32
            && population["physical_frames"] == 31,
        "population shape",
    )?;
    let map = array(&population["objective_row_map"])?
        .iter()
        .map(index)
        .collect::<Result<Vec<_>>>()?;
    require(
        map.len() == 31 && map.iter().all(|&i| i < 391),
        "objective mapping",
    )?;
    let roles = array(&population["roles"])?;
    let reference_ids = roles
        .iter()
        .filter(|r| r["task"] == false)
        .map(|r| Ok((index(&r["input"])?, index(&r["position"])?)))
        .collect::<Result<BTreeSet<_>>>()?;
    require(reference_ids.len() == 17, "17 reference identities")?;
    let guards = array(&population["guard_population"]["terms"])?;
    require(guards.len() == 380, "guard authority380")?;
    for i in 0..380 {
        for key in ["input_index", "position", "id", "actual_prefix_ids"] {
            require(
                guards[i][key] == p_rows[i][key],
                "guard index/union identity mismatch",
            )?;
        }
        require(
            guards[i]["required_original_winner"] == p_rows[i]["target_label_only"],
            "guard required winner differs",
        )?;
    }
    let mut rows = Vec::new();
    let mut receipts = Vec::new();
    let mut alias_records = Vec::new();
    let mut seen = BTreeSet::new();
    for (i, p) in p_rows.iter().enumerate() {
        require(index(&p["row"])? == i, "population row ordering")?;
        let input = index(&p["input_index"])?;
        let position = index(&p["position"])?;
        require(seen.insert((input, position)), "duplicate physical frame")?;
        let path = run.join(format!(
            "candidate-row-{input:04}-position-{position:02}.json"
        ));
        let v = read(&path)?;
        for key in [
            "input_index",
            "position",
            "id",
            "actual_prefix_ids",
            "target_label_only",
        ] {
            require(v[key] == p[key], "snapshot population identity mismatch")?;
        }
        let n = &v["native"];
        let target = token(&p["target_label_only"])?;
        let winner = token(&n["pool"]["summary"]["chosen_token_id"])?;
        let masses = array(&n["pool"]["token_masses"])?;
        require(masses.len() == 4096, "complete saved mass vector")?;
        let mut mass_ids = BTreeSet::new();
        let mut best: Option<(u64, u32)> = None;
        for m in masses {
            let t = token(&m["token_id"])?;
            require(t < 4096 && mass_ids.insert(t), "token mass identity")?;
            if t != target {
                let w = m["weight_q31"]
                    .as_u64()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "mass absent"))?;
                if best.is_none_or(|(bw, bt)| w > bw || (w == bw && t < bt)) {
                    best = Some((w, t));
                }
            }
        }
        let competitor = best
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "competitor absent"))?
            .1;
        if i < 380 {
            require(winner == target, "final protected winner differs")?;
        }
        let ids = array(&n["copy_ids"])?;
        let candidates = array(&n["bank_trace"]["cue_bank"]["bank"]["candidates"])?;
        require(ids.len() == candidates.len(), "physical alias count")?;
        let aliases = |t: u32| -> Result<Vec<Value>> {
            ids.iter()
                .enumerate()
                .filter_map(|(j, id)| match token(id) {
                    Ok(x) if x == t => Some(Ok(json!({"ordinal":j,"candidate":candidates[j]}))),
                    Ok(_) => None,
                    Err(e) => Some(Err(e)),
                })
                .collect()
        };
        alias_records.push(json!({"row":i,"target":target,"winner":winner,"competitor":competitor,"target_aliases":aliases(target)?,"winner_aliases":aliases(winner)?,"competitor_aliases":aliases(competitor)?}));
        receipts.push(json!({"row":i,"file":path.file_name().and_then(|s|s.to_str()),"sha256":hash(&path)?,"bytes":fs::metadata(&path)?.len()}));
        rows.push(Row {
            identity: p.clone(),
            path,
            target,
            winner,
            competitor,
            donor: index(&n["bridge"]["selected_ordinal"])?,
            post: codes(&n["post_state"])?,
            original_donor: index(&p["donor"])?,
            original_post: codes(&p["post_state"])?,
            guard: i < 380,
            reference: reference_ids.contains(&(input, position)),
        });
    }
    let journal = read(&run.join("coupled-construction.json"))?;
    let records = array(&journal["coordinate_records"])?;
    require(records.len() == 1920, "complete journal1920")?;
    let task_rows = &map[..15];
    for (pos, &i) in task_rows.iter().enumerate() {
        require(
            index(&rows[i].identity["position"])? == pos,
            "ordered15 task physical mapping",
        )?;
    }
    let eos = task_rows[14];
    let eos_target = rows[eos].target;
    let mut events = BTreeMap::<usize, Vec<(usize, usize, u32)>>::new();
    let mut wanted = BTreeSet::from([eos]);
    let mut event_count = 0;
    let mut epoch = 0;
    let mut current_objective = journal["summary"]["initial"].clone();
    for (order, rec) in records.iter().enumerate() {
        require(
            index(&rec["order"])? == order && index(&rec["incumbent_epoch"])? == epoch,
            "journal order/epoch",
        )?;
        let selected = selected_alt(rec)?;
        require(
            index(&rec["epoch_after"])? == epoch + usize::from(selected.is_some()),
            "journal epoch advancement",
        )?;
        epoch = index(&rec["epoch_after"])?;
        for (a, alt) in array(&rec["alternatives"])?.iter().enumerate() {
            if token(&alt["objective"]["objective_masses"][14][2])? == eos_target
                && alt["guard_status"] == "FIRST_VETO"
            {
                let guard = index(&alt["first_failure"]["guard_index"])?;
                require(guard < 380, "historical veto guard domain")?;
                wanted.insert(guard);
                events.entry(order).or_default().push((
                    a,
                    guard,
                    token(&current_objective["objective_masses"][14][2])?,
                ));
                event_count += 1;
            }
        }
        if let Some(alt) = selected {
            current_objective = alt["objective"].clone();
        }
    }
    require(
        event_count == 7,
        "pinned journal seven EOS first-veto observations",
    )?;
    let mut history = wanted
        .iter()
        .map(|&r| (r, (rows[r].donor, rows[r].post.clone())))
        .collect::<BTreeMap<_, _>>();
    let mut historical = Vec::new();
    let mut factor_counts = GenerateReadCounts::default();
    let mut bridge_calls = 0;
    for (order, rec) in records.iter().enumerate().rev() {
        let family = rec["family"]
            .as_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "family absent"))?;
        require(
            ["prefix.coefficients", "generate.unary"].contains(&family),
            "journal family",
        )?;
        if family == "prefix.coefficients" {
            if let Some(alt) = selected_alt(rec)? {
                for change in array(&alt["changed_rows"])? {
                    let r = index(&change[0])?;
                    require(r < 391, "changed row domain")?;
                    if let Some((donor, post)) = history.get(&r) {
                        let previous = undo_change(*donor, post, change)?;
                        let old = bridge_post(&rows[r], previous, &bridge)?;
                        bridge_calls += 1;
                        history.insert(r, (previous, old));
                    }
                }
            }
        }
        if let Some(items) = events.get(&order) {
            for &(a, guard, incumbent_winner) in items {
                let alt = &rec["alternatives"][a];
                let (_, ep) = &history[&eos];
                let (_, gp) = &history[&guard];
                let es = event_state(ep, eos, family, alt)?;
                let gs = event_state(gp, guard, family, alt)?;
                let rival = token(&alt["first_failure"]["chosen"])?;
                historical.push(json!({"order":order,"family":family,"index":rec["index"],"alternative_code":alt["code"],"incumbent_epoch":rec["incumbent_epoch"],
                "guard_index":guard,"guard_identity":rows[guard].identity,"guard_contrast":contrast(&g,&gs,rows[guard].target,rival,&mut factor_counts)?,
                "EOS_incumbent_post":raw_codes(ep),"guard_incumbent_post":raw_codes(gp),"EOS_alternative_post":raw_codes(&es),
                "EOS_target_keys":keys(&g,&es,eos_target,&mut factor_counts)?,"EOS_actual_incumbent_winner":incumbent_winner,
                "EOS_actual_incumbent_winner_keys":keys(&g,&es,incumbent_winner,&mut factor_counts)?,
                "EOS_target_minus_incumbent_contrast":contrast(&g,&es,eos_target,incumbent_winner,&mut factor_counts)?,
                "EOS_actual_alternative_runner_up":"UNAVAILABLE_IN_COMPACT_JOURNAL","state_scope":"reconstructed historical incumbent/staged post; not final-state substitution"}));
            }
        }
    }
    for (&r, (donor, post)) in &history {
        require(
            *donor == rows[r].original_donor && *post == rows[r].original_post,
            "reverse history differs from authenticated original population state",
        )?;
    }
    historical.sort_by_key(|v| v["order"].as_u64());
    let mut contrasts = Vec::new();
    let mut focus = BTreeSet::new();
    for &r in task_rows {
        if rows[r].winner != rows[r].target {
            let c = contrast(
                &g,
                &rows[r].post,
                rows[r].target,
                rows[r].winner,
                &mut factor_counts,
            )?;
            for field in ["target_keys", "rival_keys"] {
                for k in array(&c[field])? {
                    focus.insert(u32::try_from(index(k)?)?);
                }
            }
            contrasts.push(json!({"row":r,"identity":rows[r].identity,"contrast":c}));
        }
    }
    for event in &historical {
        for field in ["EOS_target_keys", "EOS_actual_incumbent_winner_keys"] {
            for k in array(&event[field])? {
                focus.insert(u32::try_from(index(k)?)?);
            }
        }
        for field in ["target_keys", "rival_keys"] {
            for k in array(&event["guard_contrast"][field])? {
                focus.insert(u32::try_from(index(k)?)?);
            }
        }
    }
    let audit_path = run
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "recovery parent absent"))?
        .join("audit/result.json");
    require(
        hash(&audit_path)? == "8152fbb044bec0d7416370aeaa3a74626ba28f294745227f3e21a4ebf061938f",
        "prior numerical audit result identity",
    )?;
    let audit = read(&audit_path)?;
    require(
        audit["status"] == "PASS"
            && audit["report_sha256"] == REPORT
            && audit["reader_sha256"]
                == "d47937a9606cda264fc49a5a38744c8fe801c963148d80232308af3ecac4bc91",
        "prior geometry continuity audit authority",
    )?;
    let mut original_contrasts = Vec::new();
    for &r in task_rows {
        if rows[r].winner != rows[r].target {
            let c = contrast(
                &g,
                &rows[r].original_post,
                rows[r].target,
                rows[r].winner,
                &mut factor_counts,
            )?;
            for field in ["target_keys", "rival_keys"] {
                for k in array(&c[field])? {
                    focus.insert(u32::try_from(index(k)?)?);
                }
            }
            original_contrasts.push(json!({"row":r,"identity":rows[r].identity,"contrast":c,"rival_scope":"final candidate winner used as matched token anchor; original fullpool competitor not inferred"}));
        }
    }
    // Every focused pair also requests its component unary incidence. A pair is a
    // conjunction of retained relative codes, not a new distinguishable state.
    let mut descriptions = Vec::new();
    for k in focus.clone() {
        if k >= 960 {
            let e = ((k - 960) / 14400) as usize;
            let relative = (k - 960) % 14400;
            let edge = &g.energy().edges()[e];
            let left = u32::from(edge.left) * 120 + relative / 120;
            let right = u32::from(edge.right) * 120 + relative % 120;
            focus.insert(left);
            focus.insert(right);
            descriptions.push(json!({"pair_key":k,"edge":e,"component_unary_keys":[left,right]}));
        }
    }
    let mut matched_original = 0;
    for row in &rows {
        require(
            bridge_post(row, row.original_donor, &bridge)? == row.original_post,
            "original population bridge post mismatch",
        )?;
        bridge_calls += 1;
        matched_original += 1;
    }
    let mut scopes = Vec::new();
    for original in [false, true] {
        let mut shared = BTreeMap::<u32, Vec<Value>>::new();
        let mut totals = vec![0u64; 58560];
        for (r, row) in rows.iter().enumerate() {
            let post = if original {
                &row.original_post
            } else {
                &row.post
            };
            let target_keys = keys(&g, post, row.target, &mut factor_counts)?;
            let competitor_keys = keys(&g, post, row.competitor, &mut factor_counts)?;
            let mut counts = BTreeMap::<u32, u32>::new();
            for &t in reducer.legal_token_ids() {
                for k in keys(&g, post, t, &mut factor_counts)? {
                    totals[k as usize] += 1;
                    if focus.contains(&k) {
                        *counts.entry(k).or_default() += 1;
                    }
                }
            }
            for (k, n) in counts {
                let td = i32::from(target_keys.contains(&k));
                let rd = i32::from(competitor_keys.contains(&k));
                shared.entry(k).or_default().push(json!({"row":r,"input_index":row.identity["input_index"],"position":row.identity["position"],"guard":row.guard,"reference":row.reference,"all_legal_Generate_token_incidence":n,
                    "required_target_has_key":td==1,"final_candidate_competitor_anchor_has_key":rd==1,"signed_target_minus_competitor_anchor_incidence":td-rd}));
            }
        }
        let totals = totals
            .iter()
            .enumerate()
            .filter(|(_, n)| **n > 0)
            .map(|(k, n)| json!([k, n]))
            .collect::<Vec<_>>();
        scopes.push(json!({"state_scope":if original {"original selected population states; fixed prototype/algebra/edge continuity reused from prior numerical audit"}else{"unselected final candidate states"},
            "shared_incidence":shared,"global_nonzero_factor_token_incidence":totals,"all_token_calls":391*4096,
            "competitor_scope":if original {"matched final candidate competitor token anchors; original fullpool competitor UNANALYZED"}else{"highest saved final pooled non-target mass, smallest token ID on ties"}}));
    }
    let mut interaction_witnesses = Vec::new();
    for event in &historical {
        let signed = |v: &Value| -> Result<BTreeMap<u32, i64>> {
            array(&v["signed_target_minus_rival_incidence"])?
                .iter()
                .map(|x| {
                    Ok((
                        u32::try_from(index(&x[0])?)?,
                        x[1].as_i64().ok_or_else(|| {
                            io::Error::new(io::ErrorKind::InvalidData, "signed incidence absent")
                        })?,
                    ))
                })
                .collect()
        };
        let eos_delta = signed(&event["EOS_target_minus_incumbent_contrast"])?;
        let guard_delta = signed(&event["guard_contrast"])?;
        let witnesses=eos_delta.iter().filter_map(|(&k,&e)| {let guard=*guard_delta.get(&k).unwrap_or(&0);
            if (e>0 && guard>=0)||(e<0 && guard<=0) {Some(json!({"key":k,"family":if k<960{"unary"}else{"ordered_pair"},"EOS_signed_incidence":e,"guard_signed_incidence":guard,"algebraic_direction":if e>0{"positive"}else{"negative"}}))}else{None}}).collect::<Vec<_>>();
        interaction_witnesses.push(json!({"order":event["order"],"incumbent_epoch":event["incumbent_epoch"],"alternative_code":event["alternative_code"],"guard_index":event["guard_index"],
            "unary937_signed_contrasts":{"EOS":eos_delta.get(&937).copied().unwrap_or(0),"guard":guard_delta.get(&937).copied().unwrap_or(0)},
            "direct_gap_direction_compatible_coordinates":witnesses,"scope":"signed incidence only, not coefficients/proposals or fullpool guard feasibility; Copy masses and all other rivals remain coupled"}));
    }
    let historical_states = historical
        .iter()
        .map(|v| {
            serde_json::to_string(&json!([
                v["order"],
                v["incumbent_epoch"],
                v["EOS_incumbent_post"],
                v["guard_incumbent_post"]
            ]))
        })
        .collect::<std::result::Result<BTreeSet<_>, _>>()?;
    Ok(
        json!({"schema":"uor-r4.native-saved-pair-incidence/1","status":"COMPLETED","input_report_sha256":REPORT,"input_manifest_sha256":MANIFEST,"input_complete_file_verification":"PASS",
        "artifact_receipts":{"generate_sha256":hash(&cp.join("generate.bin"))?,"tokenizer_sha256":hash(&cp.join("native/tokenizer.json"))?,"bridge_sha256":hash(&cp.join("read-state-bridge-categorical.bin"))?,"population_sha256":hash(&run.join("coupled-population.json"))?,"journal_sha256":hash(&run.join("coupled-construction.json"))?},
        "snapshot_inputs":receipts,"final_state_scope":"unselected candidate391 saved final native posts only","original_population_scope":{"bridge_validated_states":matched_original,"incidence":"COMPLETE391_UNDER_PRIOR_AUDITED_FROZEN_GEOMETRY","prior_audit_result_sha256":"8152fbb044bec0d7416370aeaa3a74626ba28f294745227f3e21a4ebf061938f","prior_audit":"REUSED_NOT_RERUN","full_numerical_scores":"UNANALYZED"},
        "rows":391,"guards":380,"references":17,"legal_generate_tokens":4096,"lanes":8,"ordered_edges":g.energy().edges(),"pair_key_encoding":"960+edge*14400+left_relative*120+right_relative",
        "final_residual_contrasts":contrasts,"historical_EOS_guard_vetoes":historical,"original_residual_matched_token_contrasts":original_contrasts,"incidence_scopes":scopes,"pair_component_unary_keys":descriptions,"historical_veto_alternatives":7,"distinct_historical_veto_states":historical_states.len(),"historical_signed_interaction_witnesses":interaction_witnesses,
        "physical_Copy_aliases":alias_records,"Copy_scope":"exact physical occurrences retained separately; no Generate pair attribution assigned to Copy",
        "saved_bridge_reconstruction_calls":bridge_calls,"new_model_steps":0,"new_encoders":0,"new_gradients":0,"new_proposals":0,"new_Generate_scores_or_pool_reductions":0,"saved_bridge_action_scores":bridge_calls*960,
        "limits":"Distinct keys show representational incidence, not legal-code/global-pool/380 feasibility. Historical EOS runner-up is unavailable; original competitor tokens are explicit final-candidate anchors. No capacity, serving, geometry advantage or language claim.",
        "elapsed_seconds":start.elapsed().as_secs_f64(),"output":out}),
    )
}
fn main() -> Result<()> {
    let argv = std::env::args_os().skip(1).collect::<Vec<_>>();
    require(
        argv.len() == 2,
        "usage: native-saved-pair-incidence RUN OUTPUT",
    )?;
    let run = fs::canonicalize(PathBuf::from(&argv[0]))?;
    let out = PathBuf::from(&argv[1]);
    let parent = out
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let absolute = fs::canonicalize(parent)?.join(
        out.file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "output leaf missing"))?,
    );
    require(
        !absolute.starts_with(&run) && !run.starts_with(&absolute),
        "output/input ancestry forbidden",
    )?;
    report_output::claim(&out)?;
    let result = analyze(&run, &out);
    let value = match &result {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.native-saved-pair-incidence/1","status":"FAILED","error":e.to_string(),"new_model_steps":0})
        }
    };
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&value)?)?;
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result.map(|_| ())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_h4_is_rejected() {
        assert!(codes(&json!([0, 1, 2, 3, 4, 5, 6, 120])).is_err());
        assert!(codes(&json!([1, 2])).is_err());
    }
    #[test]
    fn rejected_prefix_does_not_reverse_history() -> Result<()> {
        let rec = json!({"selected":{"status":"unchanged"},"alternatives":[{"code":1,"feasible":false,"changed_rows":[[299,0,1,[1,1,1,1,1,1,1,1],true,"digest"]]}]});
        assert!(selected_alt(&rec)?.is_none());
        Ok(())
    }
    #[test]
    fn committed_donor_change_has_exact_reverse_and_shape_guard() -> Result<()> {
        let post = codes(&json!([2, 2, 2, 2, 2, 2, 2, 2]))?;
        let change = json!([299, 3, 7, [2, 2, 2, 2, 2, 2, 2, 2], true, "digest"]);
        assert_eq!(undo_change(7, &post, &change)?, 3);
        assert!(undo_change(6, &post, &change).is_err());
        assert!(undo_change(7, &codes(&json!([1, 1, 1, 1, 1, 1, 1, 1]))?, &change).is_err());
        Ok(())
    }
    #[test]
    fn staged_prefix_and_generate_incumbents_are_distinct() -> Result<()> {
        let post = codes(&json!([1, 1, 1, 1, 1, 1, 1, 1]))?;
        let alt = json!({"changed_rows":[[299,0,1,[2,2,2,2,2,2,2,2],true,"digest"]]});
        assert_eq!(
            raw_codes(&event_state(&post, 299, "prefix.coefficients", &alt)?),
            vec![2; 8]
        );
        assert_eq!(event_state(&post, 299, "generate.unary", &alt)?, post);
        assert_eq!(event_state(&post, 298, "prefix.coefficients", &alt)?, post);
        Ok(())
    }
}
