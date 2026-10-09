//! Offline four-radius Prefix vector transaction; no serving policy changes.
use super::*;

const RADII: [u8; 4] = [1, 2, 4, 7];
#[derive(Clone)]
struct Proposal {
    radius: u8,
    masters: Vec<f32>,
    changed: Vec<usize>,
    max_abs_gradient: f64,
    eta: f64,
    linear_delta: f64,
    status: &'static str,
}
fn proposals(pm: &[f32], pg: &[f32]) -> Result<Vec<Proposal>> {
    replay_require(
        pm.len() == COUNT && pg.len() == COUNT,
        "Prefix vector shape",
    )?;
    replay_require(
        pm.iter().chain(pg).all(|x| x.is_finite()),
        "Prefix vector nonfinite input",
    )?;
    let max_abs = pg.iter().map(|g| f64::from(*g).abs()).fold(0., f64::max);
    RADII
        .iter()
        .map(|&radius| {
            let eta = if max_abs == 0. {
                0.
            } else {
                (f64::from(radius) * 0.25) / max_abs
            };
            let mut masters = Vec::with_capacity(COUNT);
            let mut changed = Vec::new();
            let mut linear = 0.;
            for (i, (&m, &g)) in pm.iter().zip(pg).enumerate() {
                let old = generate::code(m)?;
                let value = if max_abs == 0. {
                    m
                } else {
                    let x = (f64::from(m) - eta * f64::from(g)).clamp(-1.75, 1.75) as f32;
                    let q = generate::code(x)?;
                    if q == old {
                        m
                    } else {
                        f32::from(q) * 0.25
                    }
                };
                if value.to_bits() != m.to_bits() {
                    changed.push(i);
                }
                linear += f64::from(g) * (f64::from(value) - f64::from(m));
                masters.push(value);
            }
            replay_require(
                linear.is_finite(),
                "Prefix vector linearized utility overflow",
            )?;
            let status = if max_abs == 0. {
                "zero_gradient"
            } else if changed.is_empty() {
                "unchanged_native_codes"
            } else if linear < 0. {
                "eligible"
            } else {
                "nonnegative_actual_linear_delta"
            };
            Ok(Proposal {
                radius,
                masters,
                changed,
                max_abs_gradient: max_abs,
                eta,
                linear_delta: linear,
                status,
            })
        })
        .collect()
}
fn affected_union(frames: &[shared::Frame], changed: &[usize]) -> Result<Vec<usize>> {
    let mut affected = Vec::new();
    for (row, f) in frames.iter().enumerate() {
        if key_union_affected(validated_prefix_keys(f.ids.len(), &f.cue_keys)?, changed) {
            affected.push(row);
        }
    }
    Ok(affected)
}
fn key_union_affected(keys: &[Vec<Option<usize>>], changed: &[usize]) -> bool {
    keys.iter()
        .flatten()
        .flatten()
        .any(|key| changed.binary_search(key).is_ok())
}
struct Stage {
    rows: BTreeMap<usize, Replacement>,
    objective: Value,
    receipt: Value,
}
#[allow(clippy::too_many_arguments)]
fn stage(
    proposal: &Proposal,
    baseline: &Value,
    p: &ContinuationParent,
    frames: &[shared::Frame],
    map: &[usize],
    spec: &shared::ObjectiveSpec,
    pm: &[f32],
    native: &NativeGeometricGenerate,
    caches: &[GeneratePatchCache],
    posts: &[Vec<H4Code>],
    donors: &[usize],
    incidences: &[RowIncidence],
    legal: &[u32],
    red: &mut NativeVocabularyActions,
    pc: &mut PostCache,
) -> Result<Stage> {
    let affected = affected_union(frames, &proposal.changed)?;
    let affected_guards = affected
        .iter()
        .copied()
        .filter(|i| *i < GUARDS)
        .collect::<Vec<_>>();
    let mut rows = BTreeMap::new();
    for &row in map {
        if affected.binary_search(&row).is_ok() {
            rows.insert(
                row,
                replacement(
                    row,
                    &frames[row],
                    pm,
                    &proposal.masters,
                    native,
                    p,
                    pc,
                    red,
                    legal,
                    &posts[row],
                )?,
            );
        }
    }
    let objective = replacement_objective(frames, map, spec, caches, &rows)?;
    let objective_gate = generate::valid_objective(baseline, &objective)?;
    let mut checked = Vec::new();
    let mut failure = Value::Null;
    if objective_gate {
        for &row in &affected_guards {
            if !rows.contains_key(&row) {
                rows.insert(
                    row,
                    replacement(
                        row,
                        &frames[row],
                        pm,
                        &proposal.masters,
                        native,
                        p,
                        pc,
                        red,
                        legal,
                        &posts[row],
                    )?,
                );
            }
            checked.push(row);
            failure = guard_failure(row, frames[row].target, &rows[&row].cache);
            if !failure.is_null() {
                break;
            }
        }
    }
    let changes = rows
        .iter()
        .map(|(&row, r)| {
            Ok(json!([
                row,
                donors[row],
                r.donor,
                r.post.iter().map(|x| x.index()).collect::<Vec<_>>(),
                r.incidence.is_some(),
                match &r.incidence {
                    Some(inc) => inc.digest()?,
                    None => incidences[row].digest()?,
                }
            ]))
        })
        .collect::<Result<Vec<_>>>()?;
    let digest = replacement_digest(&rows, frames)?;
    let receipt = json!({"radius":proposal.radius,"incumbent_epoch":0,"objective":journal_objective(&objective),
        "strict_current_CE_and17":objective_gate,"affected_rows":affected,"affected_guard_indices":affected_guards,
        "checked_guard_indices":checked,"first_failure":failure,
        "guard_status":if !objective_gate{"NOT_CHECKED_OBJECTIVE_GATE_FALSE"}else if !failure.is_null(){"FIRST_VETO"}else{"FULL_PASS"},
        "feasible":objective_gate && failure.is_null(),"changed_rows":changes,"staged_summary_digest":digest,
        "staged_rows_count":rows.len()});
    Ok(Stage {
        rows,
        objective,
        receipt,
    })
}
fn guard_failure(row: usize, target: u32, cache: &GeneratePatchCache) -> Value {
    let chosen = cache.summary().chosen_token_id;
    if chosen == target {
        Value::Null
    } else {
        json!({"guard_index":row,"required":target,"chosen":chosen})
    }
}
struct Evaluation {
    alternatives: Vec<Value>,
    best: Option<(f64, Proposal, Value)>,
    evaluated: u64,
    reductions: u64,
}
fn evaluate(
    mut stage: impl FnMut(&Proposal) -> Result<Stage>,
    candidates: Vec<Proposal>,
) -> Result<Evaluation> {
    let mut result = Evaluation {
        alternatives: Vec::with_capacity(4),
        best: None,
        evaluated: 0,
        reductions: 0,
    };
    for proposal in candidates {
        let projection = json!({"radius":proposal.radius,"max_abs_gradient":proposal.max_abs_gradient,"eta":proposal.eta,
            "actual_linear_delta":proposal.linear_delta,"status":proposal.status,"changed_indices":proposal.changed,
            "destination_master_bits":proposal.masters.iter().map(|x|x.to_bits()).collect::<Vec<_>>()});
        if proposal.status != "eligible" {
            result.alternatives.push(json!({"projection":projection,"native_stage":"NOT_RUN_LINEAR_FILTER","incumbent_epoch":0}));
            continue;
        }
        result.evaluated += 1;
        let staged = stage(&proposal)?;
        result.reductions += staged.rows.len() as u64;
        let ce = staged.objective["combined"]
            .as_f64()
            .filter(|x| x.is_finite())
            .ok_or_else(|| bad("vector finite CE absent"))?;
        result
            .alternatives
            .push(json!({"projection":projection,"native_stage":staged.receipt}));
        if staged.receipt["feasible"] == true
            && result.best.as_ref().is_none_or(|(old, previous, _)| {
                ce < *old || (ce == *old && proposal.radius < previous.radius)
            })
        {
            result.best = Some((ce, proposal, staged.receipt));
        }
        drop(staged.rows);
    }
    Ok(result)
}
fn verify_restage(expected: &Value, actual: &Value) -> Result<()> {
    replay_require(
        expected == actual,
        "Prefix vector selected restage authority differs",
    )
}
fn apply(
    rows: BTreeMap<usize, Replacement>,
    caches: &mut [GeneratePatchCache],
    posts: &mut [Vec<H4Code>],
    donors: &mut [usize],
    incidences: &mut [RowIncidence],
) -> Result<()> {
    // Validate the complete batch before the first infallible ownership swap.
    replay_require(
        rows.keys().all(|&i| {
            i < caches.len() && i < posts.len() && i < donors.len() && i < incidences.len()
        }),
        "Prefix vector complete replacement domain",
    )?;
    for (row, r) in rows {
        caches[row] = r.cache;
        posts[row] = r.post;
        donors[row] = r.donor;
        if let Some(incidence) = r.incidence {
            incidences[row] = incidence;
        }
    }
    Ok(())
}
fn commit_selected(
    expected: &Value,
    staged: Stage,
    caches: &mut [GeneratePatchCache],
    posts: &mut [Vec<H4Code>],
    donors: &mut [usize],
    incidences: &mut [RowIncidence],
) -> Result<Value> {
    verify_restage(expected, &staged.receipt)?;
    let next = staged.objective;
    apply(staged.rows, caches, posts, donors, incidences)?;
    Ok(next)
}
pub(super) struct Outcome {
    pub masters: Vec<f32>,
    pub current: Value,
    pub committed: bool,
    pub evaluated: u64,
    pub journal: Value,
}
#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    a: &Args,
    start: Instant,
    p: &ContinuationParent,
    frames: &[shared::Frame],
    map: &[usize],
    spec: &shared::ObjectiveSpec,
    pm: &[f32],
    pg: &[f32],
    native: &NativeGeometricGenerate,
    caches: &mut [GeneratePatchCache],
    posts: &mut [Vec<H4Code>],
    donors: &mut [usize],
    incidences: &mut [RowIncidence],
    legal: &[u32],
    red: &mut NativeVocabularyActions,
    pc: &mut PostCache,
    baseline: &Value,
) -> Result<Outcome> {
    let Evaluation {
        alternatives,
        best,
        evaluated,
        reductions: stage_reductions,
    } = evaluate(
        |proposal| {
            shared::progress(a, start)?;
            stage(
                proposal, baseline, p, frames, map, spec, pm, native, caches, posts, donors,
                incidences, legal, red, pc,
            )
        },
        proposals(pm, pg)?,
    )?;
    let mut masters = pm.to_vec();
    let mut current = baseline.clone();
    let mut selected = json!({"status":"unchanged","incumbent_epoch":0,"epoch_after":0});
    let mut restage_reductions = 0u64;
    let committed = if let Some((_, proposal, receipt)) = best {
        let staged = stage(
            &proposal, baseline, p, frames, map, spec, pm, native, caches, posts, donors,
            incidences, legal, red, pc,
        )?;
        restage_reductions = staged.rows.len() as u64;
        // No family, cache, donor, post, incidence or epoch changes before this point.
        let selected_receipt = staged.receipt.clone();
        let next = commit_selected(&receipt, staged, caches, posts, donors, incidences)?;
        masters = proposal.masters;
        current = next;
        selected = json!({"status":"committed","radius":proposal.radius,"incumbent_epoch":0,"epoch_after":1,
            "selected_receipt":selected_receipt,"selected_restage_exact":true});
        true
    } else {
        false
    };
    Ok(Outcome {
        masters,
        current,
        committed,
        evaluated,
        journal: json!({"schema":"uor-r4.gradient-vector-prefix/1","radii":RADII,"alternatives":alternatives,
            "selected":selected,"proposal_stage_pool_reductions":stage_reductions,
            "selected_restage_pool_reductions":restage_reductions,"selected_restage_count":usize::from(committed),
            "all_radius_incumbent_epochs":0,"best_retention":"compact masters+receipt only; opaque rows dropped between alternatives"}),
    })
}
fn validate_rank_bits(values: &[f32], bytes: &[u8]) -> Result<()> {
    replay_require(
        bytes
            == values
                .iter()
                .flat_map(|x| x.to_le_bytes())
                .collect::<Vec<_>>(),
        "vector rank differs authenticated raw f32 bits",
    )
}
pub(super) fn authenticate(
    journal: &Value,
    ranked: &[Coordinate],
    population: &Value,
    root: &Path,
    gradient: &Value,
    checkpoint: &Path,
) -> Result<()> {
    replay_require(
        journal["schema"] == "uor-r4.gradient-vector-prefix-construction/1"
            && journal["prefix_vector"]["radii"] == json!(RADII)
            && journal["summary"]["coordinates"] == 960
            && journal["summary"]["maximum_alternatives"] == 13444
            && journal["summary"]["revisited"] == 0,
        "vector constructor population authority",
    )?;
    let alternatives = journal["prefix_vector"]["alternatives"]
        .as_array()
        .ok_or_else(|| bad("vector radii missing"))?;
    replay_require(alternatives.len() == 4, "four radius receipts")?;
    replay_require(
        ranked.len() == 2 * COUNT,
        "vector full original rank population",
    )?;
    let mut seen = BTreeSet::new();
    for row in ranked {
        replay_require(
            row.index < COUNT
                && [PREFIX, GENERATE].contains(&row.family.as_str())
                && seen.insert((row.family.as_str(), row.index)),
            "vector rank family/index authority",
        )?;
    }
    let mut pm = vec![0.; COUNT];
    let mut pg = vec![0.; COUNT];
    for row in ranked.iter().filter(|r| r.family == PREFIX) {
        pm[row.index] = row.original_master;
        pg[row.index] = row.gradient;
    }
    let mut gm = vec![0.; COUNT];
    let mut gg = vec![0.; COUNT];
    for row in ranked.iter().filter(|r| r.family == GENERATE) {
        gm[row.index] = row.original_master;
        gg[row.index] = row.gradient;
    }
    replay_require(
        serde_json::to_value(order(&pm, &pg, &gm, &gg)?)? == serde_json::to_value(ranked)?,
        "vector frozen full rank calculation authority",
    )?;
    for (family, kind, values) in [
        (PREFIX, "initial-master", &pm),
        (PREFIX, "gradient", &pg),
        (GENERATE, "initial-master", &gm),
        (GENERATE, "gradient", &gg),
    ] {
        let files = gradient["files"]
            .as_array()
            .ok_or_else(|| bad("vector raw authority files"))?;
        let file = files
            .iter()
            .find(|f| f["family"] == family && f["kind"] == kind)
            .ok_or_else(|| bad("vector rank raw binding missing"))?;
        let leaf = file["file"]
            .as_str()
            .ok_or_else(|| bad("vector raw filename"))?;
        let bytes = fs::read(root.join(leaf))?;
        validate_rank_bits(values, &bytes)?;
    }
    let population_rows = population["rows"]
        .as_array()
        .ok_or_else(|| bad("vector population missing"))?;
    replay_require(
        population_rows.len() == UNION && population["guards"] == GUARDS,
        "vector complete population",
    )?;
    let mut unions = Vec::with_capacity(UNION);
    let mut targets = Vec::with_capacity(UNION);
    for (i, row) in population_rows.iter().enumerate() {
        replay_require(row["row"] == i, "vector population row ordering")?;
        let keys: Vec<usize> = serde_json::from_value(row["prefix_key_union"].clone())?;
        replay_require(
            !keys.is_empty()
                && keys.iter().all(|k| *k < COUNT)
                && keys.windows(2).all(|w| w[0] < w[1]),
            "vector sorted complete key union",
        )?;
        unions.push(keys);
        targets.push(
            row["target_label_only"]
                .as_u64()
                .ok_or_else(|| bad("vector guard target"))?,
        );
    }
    let mut best: Option<(f64, u64, &Value)> = None;
    for (saved, projected) in alternatives.iter().zip(proposals(&pm, &pg)?) {
        let v = &saved["projection"];
        replay_require(
            v["radius"] == projected.radius
                && v["eta"] == projected.eta
                && v["max_abs_gradient"] == projected.max_abs_gradient
                && v["actual_linear_delta"] == projected.linear_delta
                && v["status"] == projected.status
                && v["changed_indices"] == json!(projected.changed)
                && v["destination_master_bits"]
                    == json!(projected
                        .masters
                        .iter()
                        .map(|x| x.to_bits())
                        .collect::<Vec<_>>()),
            "vector actual displacement/projection authority",
        )?;
        let stage = &saved["native_stage"];
        if projected.status != "eligible" {
            replay_require(
                stage == "NOT_RUN_LINEAR_FILTER" && saved["incumbent_epoch"] == 0,
                "vector filtered scope",
            )?;
            continue;
        }
        replay_require(
            stage["incumbent_epoch"] == 0 && stage["radius"] == projected.radius,
            "vector original epoch authority",
        )?;
        let affected = unions
            .iter()
            .enumerate()
            .filter(|(_, keys)| {
                keys.iter()
                    .any(|k| projected.changed.binary_search(k).is_ok())
            })
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        let guards = affected
            .iter()
            .copied()
            .filter(|i| *i < GUARDS)
            .collect::<Vec<_>>();
        replay_require(
            stage["affected_rows"] == json!(affected)
                && stage["affected_guard_indices"] == json!(guards),
            "vector declared changed-key union authority",
        )?;
        let checked: Vec<usize> = serde_json::from_value(stage["checked_guard_indices"].clone())?;
        replay_require(
            checked.len() <= guards.len() && checked == guards[..checked.len()],
            "vector ascending guard traversal",
        )?;
        let gate = generate::valid_objective(&journal["summary"]["initial"], &stage["objective"])?;
        replay_require(
            stage["strict_current_CE_and17"] == gate
                && stage["feasible"] == (gate && stage["first_failure"].is_null()),
            "vector gate authority",
        )?;
        if !gate {
            replay_require(
                checked.is_empty()
                    && stage["first_failure"].is_null()
                    && stage["guard_status"] == "NOT_CHECKED_OBJECTIVE_GATE_FALSE",
                "vector unavailable guard scope",
            )?;
        } else if !stage["first_failure"].is_null() {
            let last = checked
                .last()
                .ok_or_else(|| bad("vector veto without traversal"))?;
            replay_require(
                stage["guard_status"] == "FIRST_VETO"
                    && stage["first_failure"]["guard_index"] == *last
                    && stage["first_failure"]["required"] == targets[*last]
                    && stage["first_failure"]["chosen"] != targets[*last],
                "vector first-veto authority",
            )?;
        }
        if stage["feasible"] == true {
            replay_require(
                stage["guard_status"] == "FULL_PASS"
                    && stage["affected_guard_indices"] == stage["checked_guard_indices"],
                "vector complete guard scope",
            )?;
            let ce = stage["objective"]["combined"]
                .as_f64()
                .filter(|x| x.is_finite())
                .ok_or_else(|| bad("vector finite CE missing"))?;
            let radius = u64::from(projected.radius);
            if best
                .as_ref()
                .is_none_or(|(old, r, _)| ce < *old || (ce == *old && radius < *r))
            {
                best = Some((ce, radius, saved));
            }
        }
    }
    let selected = &journal["prefix_vector"]["selected"];
    if let Some((_, _, best)) = best {
        replay_require(
            selected["status"] == "committed"
                && selected["incumbent_epoch"] == 0
                && selected["epoch_after"] == 1
                && selected["radius"] == best["projection"]["radius"]
                && selected["selected_restage_exact"] == true
                && selected["selected_receipt"] == best["native_stage"],
            "selected vector/restage authority",
        )?;
    } else {
        replay_require(
            selected["status"] == "unchanged"
                && selected["incumbent_epoch"] == 0
                && selected["epoch_after"] == 0,
            "vector no feasible alternative",
        )?;
    }
    let committed = selected["status"] == "committed";
    replay_require(
        journal["prefix_vector"]["selected_restage_count"] == usize::from(committed)
            && journal["summary"]["accepted_prefix"] == usize::from(committed)
            && journal["summary"]["selected_restage_count"] == usize::from(committed),
        "vector selected restage/commit counts",
    )?;
    let mut current = if committed {
        selected["selected_receipt"]["objective"].clone()
    } else {
        journal_objective(&journal["summary"]["initial"])
    };
    let mut epoch = u64::from(committed);
    let mut final_pm = pm.clone();
    let mut final_gm = gm.clone();
    if committed {
        let chosen = alternatives
            .iter()
            .find(|a| a["projection"]["radius"] == selected["radius"])
            .ok_or_else(|| bad("selected radius missing"))?;
        let bits: Vec<u32> =
            serde_json::from_value(chosen["projection"]["destination_master_bits"].clone())?;
        final_pm = bits.into_iter().map(f32::from_bits).collect();
    }
    let records = journal["coordinate_records"]
        .as_array()
        .ok_or_else(|| bad("vector Generate journal"))?;
    let g_order = ranked
        .iter()
        .filter(|r| r.family == GENERATE)
        .collect::<Vec<_>>();
    replay_require(
        records.len() == COUNT && g_order.len() == COUNT,
        "vector Generate complete order",
    )?;
    for (i, (rec, rank)) in records.iter().zip(g_order).enumerate() {
        replay_require(
            rec["order"] == i
                && rec["family"] == GENERATE
                && rec["index"] == rank.index
                && rec["incumbent_epoch"] == epoch
                && rec["original_master_bits"] == rank.original_master.to_bits()
                && rec["gradient_bits"] == rank.gradient.to_bits()
                && rec["priority_bits"] == rank.priority.to_bits()
                && rec["rank_code"] == rank.rank_code,
            "vector frozen Generate relative order/epoch",
        )?;
        let alts = rec["alternatives"]
            .as_array()
            .ok_or_else(|| bad("vector G alternatives missing"))?;
        let mut best: Option<(f64, i64, &Value)> = None;
        for alt in alts {
            let gate = generate::valid_objective(&current, &alt["objective"])?;
            replay_require(
                alt["incumbent_epoch"] == epoch
                    && alt["strict_current_CE_and17"] == gate
                    && alt["feasible"] == (gate && alt["first_failure"].is_null()),
                "vector G current objective gate",
            )?;
            if alt["feasible"] == true {
                replay_require(
                    alt["guard_status"] == "FULL_PASS"
                        && alt["affected_guard_indices"] == alt["checked_guard_indices"],
                    "vector G full guard traversal",
                )?;
                let ce = alt["objective"]["combined"]
                    .as_f64()
                    .filter(|x| x.is_finite())
                    .ok_or_else(|| bad("vector G finite CE"))?;
                let q = alt["code"].as_i64().ok_or_else(|| bad("vector G code"))?;
                if best
                    .as_ref()
                    .is_none_or(|(b, c, _)| ce < *b || (ce == *b && q < *c))
                {
                    best = Some((ce, q, alt));
                }
            }
        }
        if let Some((_, q, alt)) = best {
            replay_require(
                rec["selected"]["status"] == "committed"
                    && rec["selected"]["code"] == q
                    && rec["selected"]["staged_summary_digest"] == alt["staged_summary_digest"],
                "vector G minimum feasible selection",
            )?;
            final_gm[rank.index] = q as f32 * 0.25;
            current = alt["objective"].clone();
            epoch += 1;
        } else {
            replay_require(
                rec["selected"]["status"] == "unchanged",
                "vector G no feasible selection",
            )?;
        }
        replay_require(rec["epoch_after"] == epoch, "vector G epoch advancement")?;
    }
    replay_require(
        journal["summary"]["accepted_epoch"] == epoch
            && journal_objective(&journal["summary"]["final"]) == current,
        "vector complete terminal objective/epoch",
    )?;
    for (leaf, values) in [
        ("prefix/prefix-source-f32.bin", final_pm),
        ("generate-source/generate.unary.f32le", final_gm),
    ] {
        replay_require(
            fs::read(checkpoint.join(leaf))?
                == values
                    .iter()
                    .flat_map(|m| m.to_le_bytes())
                    .collect::<Vec<_>>(),
            "vector final committed master bits",
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projected_fractional_noop_and_zero_gradient_preserve_bits() -> Result<()> {
        let mut pm = vec![0.01f32; COUNT];
        let mut pg = vec![0.; COUNT];
        for p in proposals(&pm, &pg)? {
            assert_eq!(p.status, "zero_gradient");
            assert!(p
                .masters
                .iter()
                .zip(&pm)
                .all(|(a, b)| a.to_bits() == b.to_bits()));
        }
        pm[0] = 1.75;
        pg[0] = -1.;
        pg[1] = 1e-9;
        let ps = proposals(&pm, &pg)?;
        assert_eq!(ps[0].masters[0].to_bits(), pm[0].to_bits());
        assert_eq!(ps[0].masters[1].to_bits(), pm[1].to_bits());
        pg[2] = 1.;
        assert!(proposals(&pm, &pg)?.iter().all(|p| p.linear_delta < 0.));
        pg[4] = f32::NAN;
        assert!(proposals(&pm, &pg).is_err());
        Ok(())
    }
    #[test]
    fn selected_restage_mismatch_fails_before_any_commit() -> Result<()> {
        let expected = json!({"objective":{"combined":1.},"changed_rows":[[1,0,1,vec![2;8],true,"digest"]],"staged_summary_digest":"a"});
        let mut actual = expected.clone();
        actual["changed_rows"][0][2] = json!(2);
        assert!(verify_restage(&expected, &actual).is_err());
        assert!(verify_restage(&expected, &expected).is_ok());
        Ok(())
    }
    fn fixture() -> Result<(
        NativeVocabularyActions,
        NativeGeometricGenerate,
        Vec<u32>,
        Vec<H4Code>,
        Vec<H4Code>,
        uor_r4_integer::geometric_source_actions::SourceActionBinding,
    )> {
        use uor_r4_core::native_geometric::learner::integrated_attention::geometry::EnergyTables;
        use uor_r4_integer::geometric_source_actions::SourceActionBinding;
        const TOK:&[u8]=br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>","special":true},{"id":1,"content":"<|eos|>","special":true},{"id":2,"content":"<|unk|>","special":true}]}"#;
        let binding = SourceActionBinding::new(TOK)?;
        let exp = (0..uor_r4_integer::geometric_read::EXP_TABLE_LEN)
            .flat_map(|i| {
                (((-(i as f64) / 256.).exp() * (1u64 << 31) as f64).round() as u32).to_le_bytes()
            })
            .collect::<Vec<_>>();
        let red = NativeVocabularyActions::new(binding.clone(), &exp)?;
        let legal = red.legal_token_ids().to_vec();
        let prototypes = (0..binding.vocab_size())
            .flat_map(|i| vec![(i + 1) as u8; 8])
            .collect::<Vec<_>>();
        let native = NativeGeometricGenerate::compile(
            &binding,
            8,
            &prototypes,
            &vec![0; binding.vocab_size().div_ceil(2)],
            EnergyTables::zeroed(8, vec![])?,
        )?;
        let old = shared::codes(&json!(vec![1; 8]))?;
        let mut next = old.clone();
        next[0] = shared::codes(&json!(vec![2; 8]))?[0];
        Ok((red, native, legal, old, next, binding))
    }
    #[test]
    fn native_four_radius_driver_guard_veto_restage_and_updated_incidence() -> Result<()> {
        let (mut red, native, legal, old, next, binding) = fixture()?;
        let pm = vec![0f32; COUNT];
        let mut pg = vec![0f32; COUNT];
        pg[0] = 1.;
        pg[1] = -1.;
        let keys = (0..2)
            .map(|j| {
                (0..8)
                    .map(|lane| Some(lane * 120 + if lane == 0 { j } else { 0 }))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let old_inc = RowIncidence::new(&native, &old, &legal)?;
        let old_digest = old_inc.digest()?;
        let mut raw = vec![0; native.vocab_size()];
        native.score_into(&old, &mut raw, &mut Default::default())?;
        let mut caches = vec![
            red.prepare_generate_patch_cache(raw.clone(), vec![4, 5], vec![1 << 20, 0])?,
            red.prepare_generate_patch_cache(
                raw.clone(),
                vec![4, 4, 5],
                vec![5 << 22, 5 << 22, 0],
            )?,
        ];
        let original = caches
            .iter()
            .map(|c| {
                (
                    c.generate_scores().to_vec(),
                    c.copy_scores().to_vec(),
                    c.summary().chosen_token_id,
                    c.revision(),
                )
            })
            .collect::<Vec<_>>();
        let mut posts = vec![old.clone(), old.clone()];
        let mut donors = vec![0, 0];
        let mut incidences = vec![
            RowIncidence::new(&native, &old, &legal)?,
            RowIncidence::new(&native, &old, &legal)?,
        ];
        let make = |proposal: &Proposal, red: &mut NativeVocabularyActions| -> Result<Stage> {
            let mut rows = BTreeMap::new();
            let mut changes = Vec::new();
            for row in 0..2 {
                let base = shared::staged_base(
                    if row == 0 {
                        &[1 << 20, 0]
                    } else {
                        &[5 << 22, 0]
                    },
                    &keys,
                    &pm,
                    &proposal.masters,
                )?;
                let donor = earliest(&base)?;
                let post = if donor == 0 {
                    old.clone()
                } else {
                    next.clone()
                };
                let mut scores = vec![0; native.vocab_size()];
                native.score_into(&post, &mut scores, &mut Default::default())?;
                let (ids, copy) = if row == 0 {
                    (vec![4, 5], base)
                } else {
                    (vec![4, 4, 5], vec![base[0], base[0], base[1]])
                };
                let cache = red.prepare_generate_patch_cache(scores, ids, copy)?;
                let inc = RowIncidence::new(&native, &post, &legal)?;
                changes.push(json!([
                    row,
                    0,
                    donor,
                    post.iter().map(|h| h.index()).collect::<Vec<_>>(),
                    inc.digest()?
                ]));
                rows.insert(
                    row,
                    Replacement {
                        cache,
                        post,
                        donor,
                        incidence: Some(inc),
                    },
                );
            }
            let failure = guard_failure(1, 4, &rows[&1].cache);
            let mass = rows[&0].cache.token_masses()[5];
            let denom = rows[&0].cache.summary().total_weight_q31;
            let ce = -(mass as f64 / denom as f64).ln();
            let receipt = json!({"incumbent_epoch":0,"feasible":failure.is_null(),"first_failure":failure,"changed_rows":changes,"objective":{"combined":ce}});
            Ok(Stage {
                rows,
                objective: json!({"combined":ce}),
                receipt,
            })
        };
        let evaluation = evaluate(|proposal| make(proposal, &mut red), proposals(&pm, &pg)?)?;
        assert_eq!(evaluation.evaluated, 4);
        assert!(evaluation
            .alternatives
            .iter()
            .all(|a| a["native_stage"]["incumbent_epoch"] == 0));
        assert!(evaluation
            .alternatives
            .iter()
            .any(|a| !a["native_stage"]["first_failure"].is_null()));
        assert_eq!(
            caches
                .iter()
                .map(|c| (
                    c.generate_scores().to_vec(),
                    c.copy_scores().to_vec(),
                    c.summary().chosen_token_id,
                    c.revision()
                ))
                .collect::<Vec<_>>(),
            original
        );
        let (_, selected, receipt) = evaluation
            .best
            .ok_or_else(|| bad("native fixture no feasible radius"))?;
        let mut bad_receipt = receipt.clone();
        bad_receipt["changed_rows"][0][2] = json!(99);
        assert!(commit_selected(
            &bad_receipt,
            make(&selected, &mut red)?,
            &mut caches,
            &mut posts,
            &mut donors,
            &mut incidences
        )
        .is_err());
        assert_eq!(donors, vec![0, 0]);
        assert_eq!(incidences[0].digest()?, old_digest);
        commit_selected(
            &receipt,
            make(&selected, &mut red)?,
            &mut caches,
            &mut posts,
            &mut donors,
            &mut incidences,
        )?;
        assert_eq!(donors[0], 1);
        assert_eq!(posts[0], next);
        assert_ne!(incidences[0].digest()?, old_digest);
        // Subsequent Generate staging uses the newly committed post incidence.
        let mut factor = [0u32; 8];
        native.factor_incidence_into(&posts[0], 5, &mut factor, &mut Default::default())?;
        let key = factor[0] as usize;
        assert!(incidences[0].matching(key).contains(&5));
        assert!(!old_inc.matching(key).contains(&5));
        let atoms = incidences[0]
            .matching(key)
            .iter()
            .map(|&t| {
                (
                    u32::from(t),
                    caches[0].generate_scores()[t as usize] + (1 << 20),
                )
            })
            .collect::<Vec<_>>();
        let pending = red.evaluate_generate_patch(&caches[0], &atoms)?;
        let mut exact = caches[0].generate_scores().to_vec();
        for &(id, score) in &atoms {
            exact[id as usize] = score;
        }
        let mut energy = native.energy().clone();
        energy.set_unary(0, (key % 120) as u8, 1)?;
        let next_generate = NativeGeometricGenerate::compile(
            &binding,
            8,
            native.prototypes(),
            native.packed_biases(),
            energy,
        )?;
        let mut actual = vec![0; native.vocab_size()];
        next_generate.score_into(&posts[0], &mut actual, &mut Default::default())?;
        assert_eq!(actual, exact);
        let full = red.reduce_trace(&exact, caches[0].copy_token_ids(), caches[0].copy_scores())?;
        assert_eq!(
            pending.summary().total_weight_q31,
            full.summary.total_weight_q31
        );
        assert_eq!(
            pending.summary().chosen_token_id,
            full.summary.chosen_token_id
        );
        for mass in &full.token_masses {
            assert_eq!(
                pending.token_mass(&caches[0], mass.token_id)?,
                mass.weight_q31
            );
        }
        Ok(())
    }
    #[test]
    fn driver_exact_ce_ties_choose_earlier_radius() -> Result<()> {
        let (mut red, native, legal, old, _, _) = fixture()?;
        let pm = vec![0.; COUNT];
        let mut pg = vec![0.; COUNT];
        pg[0] = 1.;
        let mut raw = vec![0; native.vocab_size()];
        native.score_into(&old, &mut raw, &mut Default::default())?;
        let result = evaluate(
            |_| {
                let cache =
                    red.prepare_generate_patch_cache(raw.clone(), vec![4, 4], vec![0, 0])?;
                let mut rows = BTreeMap::new();
                rows.insert(
                    0,
                    Replacement {
                        cache,
                        post: old.clone(),
                        donor: 0,
                        incidence: None,
                    },
                );
                Ok(Stage {
                    rows,
                    objective: json!({"combined":1.}),
                    receipt: json!({"feasible":true}),
                })
            },
            proposals(&pm, &pg)?,
        )?;
        assert_eq!(result.best.ok_or_else(|| bad("tie fixture"))?.1.radius, 1);
        Ok(())
    }
    #[test]
    fn cancellation_keeps_declared_key_union_affected() -> Result<()> {
        let keys = vec![(0..8).map(|lane| Some(lane * 120)).collect::<Vec<_>>()];
        let original = vec![0.; COUNT];
        let mut destination = original.clone();
        destination[0] = 0.25;
        destination[120] = -0.25;
        assert_eq!(
            shared::staged_base(&[5], &keys, &original, &destination)?,
            vec![5]
        );
        assert!(key_union_affected(&keys, &[0, 120]));
        Ok(())
    }
    #[test]
    fn vector_mode_default_recovery_and_credit_are_exclusive() -> Result<()> {
        assert_eq!(
            PrefixTransaction::default(),
            PrefixTransaction::CoordinateAdjacent
        );
        assert_eq!(
            policy_for_modes(DonorCredit::StateTangent, PrefixTransaction::default()),
            policy()
        );
        assert_eq!(
            policy_for_modes(DonorCredit::FullPoolUtility, PrefixTransaction::default()),
            policy_for(DonorCredit::FullPoolUtility)
        );
        assert!(validate_transaction(
            PrefixTransaction::GradientVectorPrefix,
            DonorCredit::StateTangent,
            false
        )
        .is_err());
        assert!(validate_transaction(
            PrefixTransaction::GradientVectorPrefix,
            DonorCredit::FullPoolUtility,
            true
        )
        .is_err());
        validate_transaction(
            PrefixTransaction::GradientVectorPrefix,
            DonorCredit::FullPoolUtility,
            false,
        )?;
        validate_inherited_transaction(
            PrefixTransaction::CoordinateAdjacent,
            PrefixTransaction::CoordinateAdjacent,
            &json!({}),
        )?;
        assert!(validate_inherited_transaction(
            PrefixTransaction::CoordinateAdjacent,
            PrefixTransaction::GradientVectorPrefix,
            &json!({"prefix_transaction":"gradient_vector_prefix"})
        )
        .is_err());
        Ok(())
    }
    #[test]
    fn replacement_batch_late_invalid_domain_preserves_all_incumbents() -> Result<()> {
        let (mut red, native, legal, old, next, _) = fixture()?;
        let mut raw = vec![0; native.vocab_size()];
        native.score_into(&old, &mut raw, &mut Default::default())?;
        let mut caches =
            vec![red.prepare_generate_patch_cache(raw.clone(), vec![4, 4], vec![0, 0])?];
        let before = caches[0].copy_scores().to_vec();
        let mut posts = vec![old.clone()];
        let mut donors = vec![0];
        let mut incidences = vec![RowIncidence::new(&native, &old, &legal)?];
        let digest = incidences[0].digest()?;
        let mut rows = BTreeMap::new();
        for row in [0, 1] {
            rows.insert(
                row,
                Replacement {
                    cache: red.prepare_generate_patch_cache(raw.clone(), vec![5], vec![1 << 24])?,
                    post: next.clone(),
                    donor: 1,
                    incidence: Some(RowIncidence::new(&native, &next, &legal)?),
                },
            );
        }
        assert!(apply(rows, &mut caches, &mut posts, &mut donors, &mut incidences).is_err());
        assert_eq!(caches[0].copy_scores(), before);
        assert_eq!(posts[0], old);
        assert_eq!(donors[0], 0);
        assert_eq!(incidences[0].digest()?, digest);
        Ok(())
    }
    #[test]
    fn vector_projection_clamps_before_quantizing_and_binds_actual_delta() -> Result<()> {
        let mut pm = vec![0.139f32; COUNT];
        let mut pg = vec![0.; COUNT];
        pm[0] = -1.7;
        pg[0] = 1.;
        pg[1] = -1.;
        pm[2] = -1.6;
        pg[2] = 1.;
        let result = proposals(&pm, &pg)?;
        // -1.7 already has native code -7: saturation is a fractional NOOP.
        assert!(result
            .iter()
            .all(|p| p.masters[0].to_bits() == pm[0].to_bits()));
        // -1.6 has code -6, so the clamped changed code becomes canonical -7.
        assert!(result.iter().all(|p| p.masters[2] == -1.75));
        for p in result {
            let delta = pg
                .iter()
                .zip(pm.iter().zip(&p.masters))
                .map(|(g, (m, n))| f64::from(*g) * (f64::from(*n) - f64::from(*m)))
                .sum::<f64>();
            assert_eq!(delta, p.linear_delta);
        }
        Ok(())
    }
    #[test]
    fn rank_raw_bit_binding_rejects_fractional_same_code_and_gradient_tamper() -> Result<()> {
        let masters = vec![0.139f32; COUNT];
        let bytes = masters
            .iter()
            .flat_map(|m| m.to_le_bytes())
            .collect::<Vec<_>>();
        validate_rank_bits(&masters, &bytes)?;
        let mut changed = masters.clone();
        changed[37] = 0.14;
        assert_eq!(generate::code(changed[37])?, generate::code(masters[37])?);
        assert!(validate_rank_bits(&changed, &bytes).is_err());
        let mut gradient = vec![0f32; COUNT];
        let raw = gradient
            .iter()
            .flat_map(|g| g.to_le_bytes())
            .collect::<Vec<_>>();
        gradient[8] = f32::from_bits(1);
        assert!(validate_rank_bits(&gradient, &raw).is_err());
        Ok(())
    }
}
