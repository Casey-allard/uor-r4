//! Offline protected pooled-margin Jacobians and four same-epoch joint trials.
use super::*;
#[path = "protected_discrete_feedback.rs"]
mod protected_discrete_feedback;
#[path = "protected_legal_set.rs"]
mod protected_legal_set;
const DIM: usize = 2 * COUNT;
const SWEEPS: usize = 256;
const TOL: f64 = 1e-10;
const RADII: [u8; 4] = [1, 2, 4, 7];
pub(super) fn policy() -> Value {
    json!({"jacobians":380,"families":[PREFIX,GENERATE],"parameter_coordinates":DIM,
        "margin":"native ln(original winner mass / strongest other token mass); ascending token-ID rival ties; unweighted",
        "passes":SWEEPS,"projection":"f64 unit-L2 normalized nonzero rows, cyclic halfspace correction; not a nearest-QP or convergence guarantee",
        "residual_tolerance":"1e-10 * direction L2, no absolute floor; checked after all passes and after quantization",
        "radii":RADII,"joint_normalization":true,"no_follow_on_coordinate_pass":true})
}
pub(super) fn discrete_policy() -> Value {
    json!({"outer_rounds":32,"inner_passes":32,"rho":1,"target_max":0.25,"coordinates":1920,
 "initial_direction":"legacy256 normalized cyclic protection projection; fixed original epoch",
 "legal":"original fractional bits for unchanged code; all other signed codes-7..7 canonical quarter",
 "cost_ties":"noop then smallest signed code","duplicate":"exact destination bits; retain receipt, score first eligible occurrence only",
 "screen":"original380 unit margins at1e-10*actualdeltaL2 and strict original objective gdotdelta<0",
 "projection":"fixed residual-feedback heuristic, not exact ADMM/convergence guarantee","maximum_alternatives":32,"no_follow_on_coordinate_pass":true})
}
pub(super) fn legal_policy() -> Value {
    json!({"backend":"microlp=0.6.0","branch_node_limit":4096,"root_counts_as_node":false,"time_limit":null,"warm_start":"original noop; advisory backend hint",
        "coordinates":1920,"legal":"centered k=q-q0 integer[-7-q0,7-q0]; fractional original noop bits or14 canonical changed quarter codes; signedzero preserved",
        "fractional_encoding":"l,r binary; l+r<=1; k>=r-(q0+7)l; k<=-l+(7-q0)r; delta=k/4+(l+r)(q0/4-m)",
        "canonical_encoding":"k=q-q0 integer[-7-q0,7-q0], no side binaries; original bits retained at unchanged code",
        "integer_decode":"raw backend values within1e-6 of a unique integer; reject illegal side/code combination; exact f32 destination bits then independent screens","objective":"minimize original gdotactualdelta","construction_target":"all380 original unitJdelta>=0; prospectively stricter than norm-tolerant admission",
        "screen":"recompute actual legal-bit delta, all380 unitJdelta>=-1e-10*deltaL2 and strict gdotdelta<0",
        "maximum_alternatives":1,"no_follow_on_coordinate_pass":true,"backend_status":"observational bounded floating search, not a mathematical optimality/infeasibility certificate"})
}
fn maximum_alternatives(mode: PrefixTransaction) -> usize {
    if mode == PrefixTransaction::ProtectedLegalSet {
        1
    } else if discrete_mode(mode) {
        32
    } else {
        4
    }
}
fn discrete_mode(mode: PrefixTransaction) -> bool {
    mode == PrefixTransaction::ProtectedDiscreteFeedback
}
fn schema(mode: PrefixTransaction) -> &'static str {
    if mode == PrefixTransaction::ProtectedLegalSet {
        "uor-r4.protected-legal-set-centered/1"
    } else if discrete_mode(mode) {
        "uor-r4.protected-discrete-feedback/1"
    } else {
        "uor-r4.protected-joint-vector/1"
    }
}
fn identity(p: &Proposal) -> &'static str {
    if p.receipt.get("candidate").is_some() {
        "candidate"
    } else if p.receipt.get("round").is_some() {
        "round"
    } else {
        "radius"
    }
}
fn duplicate(p: &Proposal) -> bool {
    p.receipt.get("duplicate_of").is_some_and(|v| !v.is_null())
}
fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}
fn norm(a: &[f64]) -> f64 {
    dot(a, a).sqrt()
}
fn unit_rows(j: &[Vec<f32>]) -> Result<Vec<Vec<f64>>> {
    j.iter()
        .map(|row| {
            replay_require(
                row.len() == DIM && row.iter().all(|x| x.is_finite()),
                "protected Jacobian malformed/missing",
            )?;
            let mut row = row.iter().map(|x| f64::from(*x)).collect::<Vec<_>>();
            let n = norm(&row);
            replay_require(n.is_finite(), "protected Jacobian norm overflow")?;
            if n > 0. {
                for x in &mut row {
                    *x /= n;
                }
            }
            Ok(row)
        })
        .collect()
}
fn residuals(rows: &[Vec<f64>], d: &[f64]) -> Result<(Vec<f64>, f64, bool)> {
    replay_require(
        d.len() == DIM && d.iter().all(|x| x.is_finite()),
        "protected direction nonfinite/shape",
    )?;
    let tolerance = TOL * norm(d);
    let values = rows.iter().map(|r| dot(r, d)).collect::<Vec<_>>();
    replay_require(
        tolerance.is_finite() && values.iter().all(|x| x.is_finite()),
        "protected residual overflow",
    )?;
    let okay = values.iter().all(|x| *x >= -tolerance);
    Ok((values, tolerance, okay))
}
fn project(gradient: &[f32], jacobians: &[Vec<f32>]) -> Result<(Vec<f64>, Value)> {
    replay_require(
        gradient.len() == DIM && gradient.iter().all(|x| x.is_finite()),
        "protected objective gradient malformed",
    )?;
    let rows = unit_rows(jacobians)?;
    let mut direction = gradient.iter().map(|x| -f64::from(*x)).collect::<Vec<_>>();
    let mut corrections = vec![0usize; rows.len()];
    for _ in 0..SWEEPS {
        for (i, row) in rows.iter().enumerate() {
            let v = dot(row, &direction);
            if v < 0. {
                for (d, j) in direction.iter_mut().zip(row) {
                    *d -= v * j;
                }
                corrections[i] += 1;
            }
        }
    }
    let (values, tolerance, okay) = residuals(&rows, &direction)?;
    let receipt = json!({"passes":SWEEPS,"unit_row_zero":rows.iter().map(|r|norm(r)==0.).collect::<Vec<_>>(),
        "correction_counts":corrections,"direction":direction,"final_residuals":values,"tolerance":tolerance,"linear_constraints_passed":okay});
    Ok((direction, receipt))
}
fn contrast(pool: &shared::Pool, winner: u32) -> Result<(u32, u64, u64)> {
    replay_require(
        pool.trace.summary.chosen_token_id == winner,
        "protected original winner differs",
    )?;
    let wm = pool
        .trace
        .token_masses
        .iter()
        .find(|m| m.token_id == winner)
        .ok_or_else(|| bad("protected winner mass absent"))?
        .weight_q31;
    let rival = pool
        .trace
        .token_masses
        .iter()
        .filter(|m| m.token_id != winner)
        .max_by(|a, b| {
            a.weight_q31
                .cmp(&b.weight_q31)
                .then_with(|| b.token_id.cmp(&a.token_id))
        })
        .ok_or_else(|| bad("protected rival absent"))?;
    replay_require(wm > 0 && rival.weight_q31 > 0, "protected zero mass")?;
    Ok((rival.token_id, wm, rival.weight_q31))
}
fn margin_utility(
    f: &shared::Frame,
    pool: &shared::Pool,
    row: usize,
    p: &ContinuationParent,
    donor: &mut shared::DonorCache,
    w: u32,
    r: u32,
) -> Result<DonorUtility> {
    replay_require(
        !f.ids.is_empty() && pool.donor == earliest(&pool.base_copy)?,
        "protected physical donor authority",
    )?;
    let mut red = NativeVocabularyActions::new(p.integer.binding().clone(), &p.exp)?;
    let mut margins = Vec::new();
    let mut records = Vec::new();
    for i in 0..f.ids.len() {
        let (post, g) = donor.get(row, i, f, p)?;
        let full = g
            .iter()
            .zip(&f.u)
            .map(|(g, u)| {
                g.checked_add(*u)
                    .ok_or_else(|| bad("protected forced donor overflow"))
            })
            .collect::<Result<Vec<_>>>()?;
        let trace = red.reduce_trace(&full, &f.ids, &pool.copy)?;
        let wm = trace
            .token_masses
            .iter()
            .find(|m| m.token_id == w)
            .ok_or_else(|| bad("forced winner missing"))?
            .weight_q31;
        let rm = trace
            .token_masses
            .iter()
            .find(|m| m.token_id == r)
            .ok_or_else(|| bad("forced rival missing"))?
            .weight_q31;
        replay_require(wm > 0 && rm > 0, "forced margin zero mass")?;
        let margin = (wm as f64 / rm as f64).ln();
        replay_require(margin.is_finite(), "forced margin nonfinite")?;
        if i == pool.donor {
            replay_require(
                post == pool.post
                    && full == pool.generate
                    && trace.summary == pool.trace.summary
                    && trace.token_masses == pool.trace.token_masses,
                "protected factual full-pool parity",
            )?;
        }
        margins.push(margin);
        records.push(json!([
            i,
            post.iter().map(|x| x.index()).collect::<Vec<_>>(),
            wm,
            rm,
            trace.summary.total_weight_q31,
            trace.summary.chosen_token_id,
            margin
        ]));
    }
    let factual = margins[pool.donor];
    let losses = margins
        .iter()
        .map(|m| (m - factual) as f32)
        .collect::<Vec<_>>();
    replay_require(
        losses.iter().all(|x| x.is_finite()),
        "forced margin contrast nonfinite",
    )?;
    Ok(DonorUtility {
        losses,
        receipt: json!({"row":row,"winner":w,"rival":r,"factual_donor":pool.donor,
        "factual_margin_f64":factual,"donors":records,"encoding":"ordinal,post8,winnerMass,rivalMass,totalMass,chosen,nativeMarginF64",
        "forced_donor_reductions":f.ids.len(),"unit_margin_weight":1,"physical_aliases":f.ids.len()}),
    })
}
pub(super) fn margin_pass(
    a: &Args,
    start: Instant,
    p: &ContinuationParent,
    frames: &[shared::Frame],
    pools: &[shared::Pool],
    pw: &PrefixAngularWeights,
    g: &GenerateLearningWeights,
    prepared: &uor_r4_training::geometric_generate_learning::PreparedGenerateLearning,
    pv: &candle_core::Var,
    donor: &mut shared::DonorCache,
    backward: bool,
) -> Result<()> {
    replay_require(
        frames.len() == GUARDS && pools.len() == GUARDS,
        "protected complete380 missing",
    )?;
    let mut terms = Vec::new();
    for (i, (f, saved_pool)) in frames.iter().zip(pools).enumerate() {
        let mut restored = saved_pool.clone();
        let mut reducer = NativeVocabularyActions::new(p.integer.binding().clone(), &p.exp)?;
        restored.trace = reducer.reduce_trace(&restored.generate, &f.ids, &restored.copy)?;
        replay_require(
            restored.trace.summary == saved_pool.trace.summary
                && restored.trace.token_masses == saved_pool.trace.token_masses,
            "protected compact-to-complete native trace parity",
        )?;
        let pool = &restored;
        shared::progress(a, start)?;
        replay_require(
            f.prefix_trace.is_some()
                && pool.donor == earliest(&pool.base_copy)?
                && pool.copy
                    == pool
                        .base_copy
                        .iter()
                        .zip(&f.ids)
                        .map(|(x, id)| x.checked_add(f.u[*id as usize]))
                        .collect::<Option<Vec<_>>>()
                        .ok_or_else(|| bad("protected alias U overflow"))?,
            "protected Prefix/U trace authority",
        )?;
        let (r, wm, rm) = contrast(pool, f.target)?;
        // Separate cache row namespace from31 objective frames.
        let utility = margin_utility(f, pool, 31 + i, p, donor, f.target, r)?;
        let loss = graph_quantity(
            a,
            f,
            pool,
            31 + i,
            p,
            pw,
            g,
            prepared,
            donor,
            Some(&utility),
            Some((f.target, r)),
        )?;
        replay_require(
            loss.to_scalar::<f32>()?.to_bits() == ((wm as f64 / rm as f64).ln() as f32).to_bits(),
            "protected native margin forward anchor differs",
        )?;
        if backward {
            let gr = loss.backward()?;
            let mut all = Vec::with_capacity(DIM);
            for var in [pv, &g.unary] {
                let values = gr
                    .get(var.as_tensor())
                    .ok_or_else(|| bad("protected family gradient MISSING; zero fill forbidden"))?
                    .flatten_all()?
                    .to_device(&Device::Cpu)?
                    .to_vec1::<f32>()?;
                replay_require(
                    values.len() == COUNT && values.iter().all(|x| x.is_finite()),
                    "protected gradient shape/nonfinite",
                )?;
                all.extend(values);
            }
            let leaf = format!("protected-margin-{i:03}.f32le");
            let bytes = all.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>();
            fs::write(a.out.join(&leaf), &bytes)?;
            let mut native_masses = vec![0u64; VOCAB];
            for m in &pool.trace.token_masses {
                native_masses[m.token_id as usize] = m.weight_q31;
            }
            let ml = format!("protected-original-masses-{i:03}.u64le");
            let mb = native_masses
                .iter()
                .flat_map(|m| m.to_le_bytes())
                .collect::<Vec<_>>();
            fs::write(a.out.join(&ml), &mb)?;
            let ul = format!("protected-donor-margin-{i:03}.json");
            write(a, &ul, &utility.receipt)?;
            terms.push(json!({"guard_index":i,"input_index":f.input,"position":f.position,"id":f.id,"actual_prefix_ids":f.prefix,
                "winner":f.target,"rival":r,"winner_mass":wm,"rival_mass":rm,"native_margin_f64":(wm as f64/rm as f64).ln(),
                "file":leaf,"sha256":sha256_bytes(&bytes),"shape":[DIM],"status":"PRESENT","all_zero":all.iter().all(|x|*x==0.),
                "original_masses_file":ml,"original_masses_sha256":sha256_bytes(&mb),"utility_file":ul,"utility_sha256":sha256_file(&a.out.join(&ul))?}));
        }
    }
    if backward {
        write(
            a,
            "protected-margin-receipt.json",
            &json!({"schema":"uor-r4.protected-margin-jacobians/1","policy":policy(),"terms":terms,
        "backward_calls":GUARDS,"fresh_objective_backward_calls":31,"total_fresh_backward_calls":411,"physical_gradients_shape":[GUARDS,DIM],
        "protected_weight":1,"objective_guard_CE_weight":0,"training_encoder_calls":0,"all380_parity_before_any_backward":true,
        "donor_cache_peak_bytes":donor.peak,"donor_cache_calls_cumulative":donor.calls}),
        )?;
    } else {
        write(
            a,
            "protected-forward-parity.json",
            &json!({"guards":GUARDS,"all_before_any_backward":true,"native_margin_factual_donor_fullpool":true,
        "forced_donor_reductions":frames.iter().map(|f|f.ids.len()).sum::<usize>(),"graph_forwards":GUARDS,"new_training_encoder_calls":0}),
        )?;
    }
    Ok(())
}
pub(super) fn resource_projection(
    a: &Args,
    guards: &[shared::Frame],
    pools: &[shared::Pool],
) -> Result<()> {
    replay_require(
        guards.len() == GUARDS && pools.len() == GUARDS,
        "protected resource population",
    )?;
    let trace = guards.iter().try_fold(0u64, |n, f| {
        Ok::<_, Box<dyn std::error::Error>>(
            n + serde_json::to_vec(
                f.prefix_trace
                    .as_ref()
                    .ok_or_else(|| bad("protected typed trace missing"))?,
            )?
            .len() as u64,
        )
    })?;
    let utilities = guards
        .iter()
        .map(|f| 2048 + f.ids.len() as u64 * 512)
        .sum::<u64>();
    let jacobian = (GUARDS * DIM * 4) as u64;
    let native_mass_bytes = (GUARDS * VOCAB * 8) as u64;
    let prior = read(&a.out.join("coupled-pregradient-resource-projection.json"))?;
    let numeric = prior["numeric_upper_bound"]
        .as_u64()
        .ok_or_else(|| bad("numeric projection absent"))?
        + trace * 8
        + jacobian * 4
        + 8 * 1024 * 1024;
    let process = prior["process_ram_projection_bytes"]
        .as_u64()
        .ok_or_else(|| bad("process projection absent"))?
        + trace * 8
        + 128 * 1024 * 1024
        + jacobian * 4;
    let report = prior["report_upper_bound"]
        .as_u64()
        .ok_or_else(|| bad("report projection absent"))?
        + utilities
        + jacobian
        + native_mass_bytes
        + 8 * 1024 * 1024;
    write(
        a,
        "protected-resource-projection.json",
        &json!({"stage":"BEFORE_ANY_BACKWARD","typed_prefix_trace_serialized_bytes":trace,"typed_trace_expansion":8,
        "jacobian_bytes":jacobian,"utility_report_bound":utilities,"numeric_upper_bound":numeric,"process_upper_bound":process,
        "report_upper_bound":report,"numeric_cap":512*1024*1024u64,"process_cap":8*1024*1024*1024u64,"guard_graphs":"sequential; one forced donor utility and one graph live;64MiB donor cache; no380 device graphs retained"}),
    )?;
    replay_require(
        numeric <= 512 * 1024 * 1024
            && process <= 8 * 1024 * 1024 * 1024
            && report + 1024 * 1024 < a.maximum_report_bytes,
        "protected complete resource projection exceeded",
    )
}
fn decode_jacobian(bytes: &[u8]) -> Result<Vec<f32>> {
    replay_require(
        bytes.len() == DIM * 4,
        "protected Jacobian byte shape differs",
    )?;
    let values = bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect::<Vec<_>>();
    replay_require(
        values.iter().all(|x| x.is_finite()),
        "protected Jacobian nonfinite",
    )?;
    Ok(values)
}
fn jacobians(root: &Path) -> Result<Vec<Vec<f32>>> {
    let receipt = read(&root.join("protected-margin-receipt.json"))?;
    replay_require(
        receipt["policy"] == policy() && receipt["backward_calls"] == GUARDS,
        "protected Jacobian policy/count",
    )?;
    let terms = receipt["terms"]
        .as_array()
        .ok_or_else(|| bad("protected terms absent"))?;
    replay_require(terms.len() == GUARDS, "protected Jacobian population")?;
    terms
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let leaf = format!("protected-margin-{i:03}.f32le");
            replay_require(
                t["guard_index"] == i
                    && t["file"] == leaf
                    && t["status"] == "PRESENT"
                    && t["shape"] == json!([DIM])
                    && sha256_file(&root.join(&leaf))? == t["sha256"],
                "protected Jacobian identity",
            )?;
            decode_jacobian(&fs::read(root.join(leaf))?)
        })
        .collect()
}
#[derive(Clone)]
struct Proposal {
    radius: u8,
    prefix: Vec<f32>,
    unary: Vec<f32>,
    receipt: Value,
    eligible: bool,
}
fn proposals(
    pm: &[f32],
    pg: &[f32],
    gm: &[f32],
    gg: &[f32],
    j: &[Vec<f32>],
) -> Result<(Vec<Proposal>, Value)> {
    let master = pm.iter().chain(gm).copied().collect::<Vec<_>>();
    let gradient = pg.iter().chain(gg).copied().collect::<Vec<_>>();
    replay_require(
        master.len() == DIM && master.iter().all(|x| x.is_finite()),
        "joint master shape/nonfinite",
    )?;
    let (d, projection) = project(&gradient, j)?;
    let rows = unit_rows(j)?;
    let max = d.iter().map(|x| x.abs()).fold(0., f64::max);
    let mut proposals = Vec::new();
    for radius in RADII {
        let eta = if max == 0. {
            0.
        } else {
            radius as f64 * 0.25 / max
        };
        let mut dest = Vec::new();
        let mut delta = Vec::new();
        for (&m, &direction) in master.iter().zip(&d) {
            let x = (f64::from(m) + eta * direction).clamp(-1.75, 1.75) as f32;
            let q = generate::code(x)?;
            let v = if q == generate::code(m)? {
                m
            } else {
                f32::from(q) * 0.25
            };
            dest.push(v);
            delta.push(f64::from(v) - f64::from(m));
        }
        let (res, tol, okay) = residuals(&rows, &delta)?;
        let linear = gradient
            .iter()
            .zip(&delta)
            .map(|(g, d)| f64::from(*g) * d)
            .sum::<f64>();
        let eligible = max > 0.
            && projection["linear_constraints_passed"] == true
            && okay
            && linear < 0.
            && linear.is_finite();
        let receipt = json!({"radius":radius,"eta":eta,"destination_master_bits":dest.iter().map(|x|x.to_bits()).collect::<Vec<_>>(),
            "actual_delta":delta,"actual_CE_linear_delta":linear,"quantized_margin_residuals":res,"quantized_tolerance":tol,
            "quantized_constraints_passed":okay,"eligible":eligible,"incumbent_epoch":0});
        proposals.push(Proposal {
            radius,
            prefix: dest[..COUNT].to_vec(),
            unary: dest[COUNT..].to_vec(),
            receipt,
            eligible,
        });
    }
    Ok((proposals, projection))
}
fn legal_proposals(
    pm: &[f32],
    pg: &[f32],
    gm: &[f32],
    gg: &[f32],
    j: &[Vec<f32>],
    saved: Option<&Value>,
) -> Result<(Vec<Proposal>, Value)> {
    replay_require(
        [pm.len(), pg.len(), gm.len(), gg.len()]
            .iter()
            .all(|n| *n == COUNT),
        "legal1920 domain",
    )?;
    let m = pm.iter().chain(gm).copied().collect::<Vec<_>>();
    let g = pg.iter().chain(gg).copied().collect::<Vec<_>>();
    let rows = unit_rows(j)?;
    let receipt: protected_legal_set::Receipt = if let Some(saved) = saved {
        serde_json::from_value(saved.clone())?
    } else {
        protected_legal_set::run(&m, &g, &rows)?
    };
    protected_legal_set::authenticate(&m, &g, &rows, &receipt)?;
    let projection = json!({"legal_solver":receipt});
    let mut proposals = Vec::new();
    if let Some(bits) = receipt.destination_master_bits {
        let dest = bits.iter().map(|b| f32::from_bits(*b)).collect::<Vec<_>>();
        let r = json!({"candidate":0,"destination_master_bits":bits,"actual_delta":receipt.actual_delta,
            "quantized_margin_residuals":receipt.construction_residuals,"quantized_tolerance":receipt.residual_tolerance,
            "actual_CE_linear_delta":receipt.gradient_dot,"quantized_constraints_passed":receipt.unchanged_screen_passed,
            "recomputed_zero_margin_target_passed":receipt.recomputed_zero_margin_target_passed,"eligible":receipt.eligible,"incumbent_epoch":0});
        proposals.push(Proposal {
            radius: 0,
            prefix: dest[..COUNT].to_vec(),
            unary: dest[COUNT..].to_vec(),
            receipt: r,
            eligible: receipt.eligible,
        });
    }
    Ok((proposals, projection))
}
fn mode_proposals(
    mode: PrefixTransaction,
    pm: &[f32],
    pg: &[f32],
    gm: &[f32],
    gg: &[f32],
    j: &[Vec<f32>],
) -> Result<(Vec<Proposal>, Value)> {
    if mode == PrefixTransaction::ProtectedLegalSet {
        return legal_proposals(pm, pg, gm, gg, j, None);
    }
    if !discrete_mode(mode) {
        return proposals(pm, pg, gm, gg, j);
    }
    let master = pm.iter().chain(gm).copied().collect::<Vec<_>>();
    let gradient = pg.iter().chain(gg).copied().collect::<Vec<_>>();
    replay_require(
        [pm.len(), pg.len(), gm.len(), gg.len()]
            .iter()
            .all(|n| *n == COUNT)
            && master.len() == DIM
            && gradient.len() == DIM,
        "feedback1920 domain",
    )?;
    let (d, mut projection) = project(&gradient, j)?;
    let run = protected_discrete_feedback::run(&master, &gradient, &unit_rows(j)?, &d)?;
    projection["feedback_status"] = json!(format!("{:?}", run.status));
    projection["feedback_target"] = json!(run.target);
    let mut output = Vec::new();
    for r in run.rounds {
        let dest = r
            .destination_bits
            .iter()
            .map(|&b| f32::from_bits(b))
            .collect::<Vec<_>>();
        let receipt = json!({"round":r.index,"destination_master_bits":r.destination_bits,"actual_delta":r.actual_delta,
   "quantized_margin_residuals":r.residuals,"quantized_tolerance":r.tolerance,"actual_CE_linear_delta":r.gradient_dot,
   "quantized_constraints_passed":r.residuals.iter().all(|x|*x>=-r.tolerance),"eligible":r.eligible,
   "feedback_norm":r.feedback_norm,"primal_norm":r.primal_norm,"duplicate_of":r.duplicate_of,"incumbent_epoch":0});
        output.push(Proposal {
            radius: r.index as u8,
            prefix: dest[..COUNT].to_vec(),
            unary: dest[COUNT..].to_vec(),
            receipt,
            eligible: r.eligible,
        });
    }
    Ok((output, projection))
}
struct Stage {
    rows: BTreeMap<usize, Replacement>,
    objective: Value,
    receipt: Value,
}
fn prepare_rows(
    count: usize,
    mut build: impl FnMut(usize) -> Result<Replacement>,
) -> Result<BTreeMap<usize, Replacement>> {
    (0..count).map(|row| build(row).map(|r| (row, r))).collect()
}
fn evaluate(
    proposals: Vec<Proposal>,
    mut stage: impl FnMut(&Proposal) -> Result<Stage>,
) -> Result<(Vec<Value>, Option<(f64, Proposal, Value)>, usize)> {
    let mut alternatives = Vec::new();
    let mut best = None;
    let mut reductions = 0;
    for proposal in proposals {
        let mut receipt = proposal.receipt.clone();
        if duplicate(&proposal) {
            receipt["native"] = json!({"guard_status":"NOT_CHECKED_DUPLICATE","feasible":false});
        } else if proposal.eligible {
            let s = stage(&proposal)?;
            reductions += s.rows.len();
            receipt["native"] = s.receipt.clone();
            if s.receipt["feasible"] == true {
                let ce = s.objective["combined"]
                    .as_f64()
                    .filter(|x| x.is_finite())
                    .ok_or_else(|| bad("joint CE absent/nonfinite"))?;
                if best
                    .as_ref()
                    .is_none_or(|(old, _, _): &(f64, Proposal, Value)| ce < *old)
                {
                    best = Some((ce, proposal.clone(), s.receipt));
                }
            }
        } else {
            receipt["native"] =
                json!({"guard_status":"NOT_CHECKED_LINEAR_INELIGIBLE","feasible":false});
        }
        alternatives.push(receipt);
    }
    Ok((alternatives, best, reductions))
}
fn stage(
    proposal: &Proposal,
    p: &ContinuationParent,
    frames: &[shared::Frame],
    map: &[usize],
    spec: &shared::ObjectiveSpec,
    pm: &[f32],
    caches: &[GeneratePatchCache],
    posts: &[Vec<H4Code>],
    incidences: &[RowIncidence],
    legal: &[u32],
    red: &mut NativeVocabularyActions,
    pc: &mut PostCache,
    baseline: &Value,
) -> Result<Stage> {
    replay_require(
        frames.len() == UNION
            && caches.len() == UNION
            && posts.len() == UNION
            && incidences.len() == UNION,
        "joint stage391 domain",
    )?;
    let native = native_with_unary(p, &proposal.unary)?;
    let rows = prepare_rows(UNION, |row| {
        replacement(
            row,
            &frames[row],
            pm,
            &proposal.prefix,
            &native,
            p,
            pc,
            red,
            legal,
            &posts[row],
        )
    })?;
    let objective = replacement_objective(frames, map, spec, caches, &rows)?;
    let objective_gate = generate::valid_objective(baseline, &objective)?;
    let mut checked = Vec::new();
    let mut failure = Value::Null;
    if objective_gate {
        for row in 0..GUARDS {
            checked.push(row);
            let chosen = rows[&row].cache.summary().chosen_token_id;
            if chosen != frames[row].target {
                failure = json!({"guard_index":row,"required":frames[row].target,"chosen":chosen});
                break;
            }
        }
    }
    let changes = rows
        .iter()
        .map(|(&row, r)| {
            Ok::<_, Box<dyn std::error::Error>>(json!([
                row,
                r.donor,
                r.post.iter().map(|x| x.index()).collect::<Vec<_>>(),
                r.incidence.as_ref().unwrap_or(&incidences[row]).digest()?
            ]))
        })
        .collect::<Result<Vec<_>>>()?;
    let digest = replacement_digest(&rows, frames)?;
    let receipt = json!({"radius":proposal.radius,"incumbent_epoch":0,"objective":journal_objective(&objective),"strict_current_CE_and17":objective_gate,
        "affected_guard_indices":(0..GUARDS).collect::<Vec<_>>(),"checked_guard_indices":checked,"first_failure":failure,
        "guard_status":if !objective_gate{"NOT_CHECKED_OBJECTIVE_GATE_FALSE"}else if !failure.is_null(){"FIRST_VETO"}else{"FULL_PASS"},
        "feasible":objective_gate && failure.is_null(),"changed_rows":changes,"staged_summary_digest":digest,"staged_rows_count":UNION});
    let mut receipt = receipt;
    if identity(proposal) != "radius" {
        receipt
            .as_object_mut()
            .ok_or_else(|| bad("stage identity"))?
            .remove("radius");
        receipt[identity(proposal)] = json!(proposal.radius);
    }
    Ok(Stage {
        rows,
        objective,
        receipt,
    })
}
fn apply(
    stage: Stage,
    caches: &mut [GeneratePatchCache],
    posts: &mut [Vec<H4Code>],
    donors: &mut [usize],
    incidences: &mut [RowIncidence],
) -> Result<Value> {
    replay_require(
        stage.rows.len() == caches.len()
            && stage.rows.keys().copied().eq(0..caches.len())
            && [caches.len(), posts.len(), donors.len(), incidences.len()]
                .iter()
                .all(|x| *x == caches.len()),
        "joint atomic replacement domain",
    )?;
    // All fallible validation/restaging precedes these ownership swaps.
    for (i, r) in stage.rows {
        caches[i] = r.cache;
        posts[i] = r.post;
        donors[i] = r.donor;
        if let Some(v) = r.incidence {
            incidences[i] = v;
        }
    }
    Ok(stage.objective)
}
pub(super) fn run(
    a: &Args,
    start: Instant,
    p: &ContinuationParent,
    frames: &[shared::Frame],
    map: &[usize],
    spec: &shared::ObjectiveSpec,
    pm: &[f32],
    pg: &[f32],
    gm: &[f32],
    gg: &[f32],
    caches: &mut [GeneratePatchCache],
    posts: &mut [Vec<H4Code>],
    donors: &mut [usize],
    incidences: &mut [RowIncidence],
    legal: &[u32],
    red: &mut NativeVocabularyActions,
) -> Result<(Vec<f32>, Vec<f32>, Value, Value)> {
    // All17 references are already among original380, including the coalesced role.
    for role in spec
        .roles
        .as_ref()
        .ok_or_else(|| bad("protected roles absent"))?
    {
        if !role.task {
            let row = *map
                .iter()
                .find(|&&row| {
                    frames[row].input == role.input && frames[row].position == role.position
                })
                .ok_or_else(|| bad("protected reference absent"))?;
            replay_require(
                row < GUARDS,
                "protected original reference not covered by380",
            )?;
        }
    }
    let j = jacobians(&a.out)?;
    let mode = transaction_mode(a);
    let (proposals, projection) = mode_proposals(mode, pm, pg, gm, gg, &j)?;
    let baseline = generate::objective(frames, map, spec, caches, &generate::Patches::new())?;
    let mut pc = PostCache::new(p);
    let (alternatives, best, reductions) = evaluate(proposals, |proposal| {
        shared::progress(a, start)?;
        stage(
            proposal, p, frames, map, spec, pm, caches, posts, incidences, legal, red, &mut pc,
            &baseline,
        )
    })?;
    let mut selected = json!({"status":"unchanged","incumbent_epoch":0,"epoch_after":0});
    let mut current = baseline.clone();
    let mut cp = pm.to_vec();
    let mut cg = gm.to_vec();
    let mut restage = 0;
    if let Some((_, proposal, expected)) = best {
        let s = stage(
            &proposal, p, frames, map, spec, pm, caches, posts, incidences, legal, red, &mut pc,
            &baseline,
        )?;
        restage = UNION;
        replay_require(
            s.receipt == expected,
            "joint selected restage mismatch; no mutation",
        )?;
        current = apply(s, caches, posts, donors, incidences)?;
        let key = identity(&proposal);
        cp = proposal.prefix;
        cg = proposal.unary;
        selected = json!({"status":"committed","incumbent_epoch":0,"epoch_after":1,"selected_receipt":expected,"selected_restage_exact":true});
        selected[key] = json!(proposal.radius);
    }
    let summary = json!({"coordinates":0,"maximum_alternatives":maximum_alternatives(mode),"evaluated_alternatives":reductions/UNION,"accepted_prefix":usize::from(selected["status"]=="committed"),
        "accepted_generate":usize::from(selected["status"]=="committed"),"accepted_joint":usize::from(selected["status"]=="committed"),"accepted_epoch":usize::from(selected["status"]=="committed"),
        "initial":baseline,"final":current,"revisited":0,"proposal_stage_pool_reductions":reductions,"selected_restage_pool_reductions":restage});
    write(
        a,
        "coupled-construction.json",
        &json!({"schema":schema(mode),"policy":policy_for_args(a),
        "protected_margin_receipt_sha256":sha256_file(&a.out.join("protected-margin-receipt.json"))?,"projection":projection,"joint_vectors":alternatives,"selected":selected,"coordinate_records":[],"summary":summary}),
    )?;
    Ok((cp, cg, current, summary))
}

pub(super) fn authenticate(
    root: &Path,
    journal: &Value,
    gradient: &Value,
    checkpoint: &Path,
) -> Result<()> {
    authenticate_mode(
        root,
        journal,
        gradient,
        checkpoint,
        PrefixTransaction::ProtectedJointVector,
    )
}
pub(super) fn authenticate_mode(
    root: &Path,
    journal: &Value,
    gradient: &Value,
    checkpoint: &Path,
    mode: PrefixTransaction,
) -> Result<()> {
    replay_require(mode.is_protected(), "protected authentication mode")?;
    replay_require(
        journal["schema"] == schema(mode)
            && journal["policy"] == policy_for_modes(DonorCredit::FullPoolUtility, mode)
            && gradient["prefix_transaction"] == json!(PrefixTransaction::ProtectedJointVector)
            && gradient["protected_margin_backward_calls"] == 380
            && gradient["total_fresh_backward_calls"] == 411
            && gradient["protected_margin_receipt_sha256"]
                == sha256_file(&root.join("protected-margin-receipt.json"))?
            && journal["protected_margin_receipt_sha256"]
                == gradient["protected_margin_receipt_sha256"],
        "protected artifact mode/raw Jacobian authority",
    )?;
    let pm = shared::floats(&root.join(format!("coupled-{PREFIX}-initial-master.f32le")))?;
    let gm = shared::floats(&root.join(format!("coupled-{GENERATE}-initial-master.f32le")))?;
    let pg = shared::floats(&root.join(format!("coupled-{PREFIX}-gradient.f32le")))?;
    let gg = shared::floats(&root.join(format!("coupled-{GENERATE}-gradient.f32le")))?;
    for (family, aggregate) in [(PREFIX, &pg), (GENERATE, &gg)] {
        let mut sum = vec![0f32; COUNT];
        for i in 0..31 {
            let term =
                shared::floats(&root.join(format!("coupled-gradient-{family}-term-{i:02}.f32le")))?;
            replay_require(
                term.len() == COUNT && term.iter().all(|x| x.is_finite()),
                "protected objective raw term shape",
            )?;
            for (s, v) in sum.iter_mut().zip(term) {
                *s += v;
            }
        }
        replay_require(
            sum.iter()
                .map(|x| x.to_bits())
                .eq(aggregate.iter().map(|x| x.to_bits())),
            "protected objective ordered aggregate bits differ",
        )?;
    }
    let j = jacobians(root)?;
    let margin = read(&root.join("protected-margin-receipt.json"))?;
    let population = read(&root.join("coupled-population.json"))?;
    let guards = population["guard_population"]["terms"]
        .as_array()
        .ok_or_else(|| bad("protected guard identity missing"))?;
    let terms = margin["terms"]
        .as_array()
        .ok_or_else(|| bad("protected margin terms missing"))?;
    replay_require(
        guards.len() == GUARDS && population["guards"] == GUARDS,
        "protected authority population",
    )?;
    for (i, (t, guard)) in terms.iter().zip(guards).enumerate() {
        replay_require(
            t["guard_index"] == i
                && t["input_index"] == guard["input_index"]
                && t["position"] == guard["position"]
                && t["id"] == guard["id"]
                && t["actual_prefix_ids"] == guard["actual_prefix_ids"]
                && t["winner"] == guard["required_original_winner"]
                && t["rival"] != t["winner"],
            "protected mechanical identity differs",
        )?;
        let mass_leaf = format!("protected-original-masses-{i:03}.u64le");
        let mb = fs::read(root.join(&mass_leaf))?;
        replay_require(
            mb.len() == VOCAB * 8
                && t["original_masses_file"] == mass_leaf
                && t["original_masses_sha256"] == sha256_bytes(&mb),
            "protected original native masses authority",
        )?;
        let masses = mb
            .chunks_exact(8)
            .map(|x| u64::from_le_bytes([x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7]]))
            .collect::<Vec<_>>();
        let winner = shared::idx(&t["winner"])?;
        replay_require(
            winner < VOCAB && masses[winner] > 0 && t["winner_mass"] == masses[winner],
            "protected winner original mass",
        )?;
        let rival = (0..VOCAB)
            .filter(|i| *i != winner)
            .max_by(|a, b| masses[*a].cmp(&masses[*b]).then_with(|| b.cmp(a)))
            .ok_or_else(|| bad("protected mechanical rival absent"))?;
        replay_require(
            t["rival"] == rival
                && masses[rival] > 0
                && t["rival_mass"] == masses[rival]
                && (0..VOCAB).all(|i| {
                    masses[i] < masses[winner] || (masses[i] == masses[winner] && i >= winner)
                }),
            "protected original winner/strongest rival tie mismatch",
        )?;
        let leaf = format!("protected-donor-margin-{i:03}.json");
        replay_require(
            t["utility_file"] == leaf && t["utility_sha256"] == sha256_file(&root.join(&leaf))?,
            "protected donor margin authority",
        )?;
        let u = read(&root.join(leaf))?;
        replay_require(
            u["row"] == 31 + i
                && u["winner"] == t["winner"]
                && u["rival"] == t["rival"]
                && u["unit_margin_weight"] == 1,
            "protected donor margin quantity differs",
        )?;
    }
    let (projected, projection) = if mode == PrefixTransaction::ProtectedLegalSet {
        legal_proposals(
            &pm,
            &pg,
            &gm,
            &gg,
            &j,
            Some(&journal["projection"]["legal_solver"]),
        )?
    } else {
        mode_proposals(mode, &pm, &pg, &gm, &gg, &j)?
    };
    replay_require(
        journal["projection"] == projection,
        "protected direction projection differs",
    )?;
    let alternatives = journal["joint_vectors"]
        .as_array()
        .ok_or_else(|| bad("protected vectors absent"))?;
    replay_require(
        alternatives.len() == projected.len()
            && (!discrete_mode(mode) || projected.len() == 32 || projected.is_empty()),
        "protected round population",
    )?;
    let mut best: Option<(f64, u8, &Value)> = None;
    for (proposal, actual) in projected.iter().zip(alternatives) {
        let mut mathematical = actual.clone();
        mathematical
            .as_object_mut()
            .ok_or_else(|| bad("protected vector record malformed"))?
            .remove("native");
        replay_require(
            mathematical == proposal.receipt,
            "protected actual projection/master/delta differs",
        )?;
        let native = &actual["native"];
        if duplicate(proposal) {
            replay_require(
                native == &json!({"guard_status":"NOT_CHECKED_DUPLICATE","feasible":false}),
                "feedback duplicate was rescored",
            )?;
            continue;
        }
        if !proposal.eligible {
            replay_require(
                native == &json!({"guard_status":"NOT_CHECKED_LINEAR_INELIGIBLE","feasible":false}),
                "protected ineligible vector was evaluated",
            )?;
            continue;
        }
        replay_require(
            native["incumbent_epoch"] == 0
                && native[identity(proposal)] == proposal.radius
                && native["affected_guard_indices"] == json!((0..GUARDS).collect::<Vec<_>>())
                && native["staged_rows_count"] == UNION,
            "protected same epoch all391 scope differs",
        )?;
        let checked = native["checked_guard_indices"]
            .as_array()
            .ok_or_else(|| bad("protected traversal missing"))?;
        replay_require(
            checked
                .iter()
                .enumerate()
                .all(|(i, v)| v.as_u64() == Some(i as u64)),
            "protected ascending traversal differs",
        )?;
        let gate = native["strict_current_CE_and17"] == true;
        if !gate {
            replay_require(
                checked.is_empty()
                    && native["first_failure"].is_null()
                    && native["guard_status"] == "NOT_CHECKED_OBJECTIVE_GATE_FALSE"
                    && native["feasible"] == false,
                "protected objective rejection mislabeled",
            )?;
        } else if native["first_failure"].is_null() {
            replay_require(
                checked.len() == GUARDS
                    && native["guard_status"] == "FULL_PASS"
                    && native["feasible"] == true,
                "protected complete guard pass missing",
            )?;
            let ce = native["objective"]["combined"]
                .as_f64()
                .filter(|x| x.is_finite())
                .ok_or_else(|| bad("protected feasible CE nonfinite"))?;
            replay_require(
                generate::valid_objective(&journal["summary"]["initial"], &native["objective"])?,
                "protected native objective gate contradicts CE/reference",
            )?;
            if best.as_ref().is_none_or(|(old, _, _)| ce < *old) {
                best = Some((ce, proposal.radius, native));
            }
        } else {
            replay_require(
                native["guard_status"] == "FIRST_VETO"
                    && native["feasible"] == false
                    && !checked.is_empty()
                    && native["first_failure"]["guard_index"] == checked[checked.len() - 1]
                    && native["first_failure"]["required"] != native["first_failure"]["chosen"],
                "protected first-veto traversal differs",
            )?;
        }
        let rows = native["changed_rows"]
            .as_array()
            .ok_or_else(|| bad("protected row identity receipt absent"))?;
        replay_require(
            rows.len() == UNION
                && rows.iter().enumerate().all(|(i, r)| {
                    r[0].as_u64() == Some(i as u64)
                        && r[2].as_array().is_some_and(|p| {
                            p.len() == 8 && p.iter().all(|v| v.as_u64().is_some_and(|x| x < 120))
                        })
                        && r[3].as_str().is_some_and(|s| {
                            s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
                        })
                }),
            "protected complete donor/post/incidence identity differs",
        )?;
    }
    let selected = &journal["selected"];
    let mut cp = pm;
    let mut cg = gm;
    if let Some((_, radius, native)) = best {
        replay_require(
            selected["status"] == "committed"
                && selected[identity(
                    projected
                        .iter()
                        .find(|p| p.radius == radius)
                        .ok_or_else(|| bad("selected protected identity missing"))?,
                )] == radius
                && selected["incumbent_epoch"] == 0
                && selected["epoch_after"] == 1
                && selected["selected_restage_exact"] == true
                && selected["selected_receipt"] == *native,
            "protected minimum feasible/restage/epoch mismatch",
        )?;
        let proposal = projected
            .iter()
            .find(|p| p.radius == radius)
            .ok_or_else(|| bad("protected selected radius missing"))?;
        cp = proposal.prefix.clone();
        cg = proposal.unary.clone();
    } else {
        replay_require(
            selected == &json!({"status":"unchanged","incumbent_epoch":0,"epoch_after":0}),
            "protected no feasible proposal committed",
        )?;
    }
    for (expected, path) in [
        (cp, checkpoint.join("prefix/prefix-source-f32.bin")),
        (cg, checkpoint.join("generate-source/generate.unary.f32le")),
    ] {
        let bytes = expected
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>();
        replay_require(
            fs::read(path)? == bytes,
            "protected selected final fractional masters differ",
        )?;
    }
    replay_require(
        journal["coordinate_records"] == json!([])
            && journal["summary"]["coordinates"] == 0
            && journal["summary"]["maximum_alternatives"] == maximum_alternatives(mode),
        "protected unexpected follow-on coordinate pass",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legal_mode_native_pool_and_incidence_keeps_same_epoch_on_veto() -> Result<()> {
        let (mut red, native, legal, a, b, binding) = fixture()?;
        let mut factors = [0u32; 8];
        native.factor_incidence_into(&a, 5, &mut factors, &mut Default::default())?;
        let key = factors[0] as usize;
        let mut pg = vec![0.; COUNT];
        pg[0] = -1.;
        pg[1] = 1.;
        let mut gg = vec![0.; COUNT];
        gg[key] = -1.;
        let mut pm = vec![-1.75; COUNT];
        pm[0] = 0.;
        pm[1] = 0.;
        let gm = vec![0.; COUNT];
        let (proposals, receipt) = mode_proposals(
            PrefixTransaction::ProtectedLegalSet,
            &pm,
            &pg,
            &gm,
            &gg,
            &[vec![0.; DIM]],
        )?;
        replay_require(
            proposals.len() == 1 && proposals[0].eligible,
            "legal fixture incumbent",
        )?;
        let (saved, saved_receipt) = legal_proposals(
            &pm,
            &pg,
            &gm,
            &gg,
            &[vec![0.; DIM]],
            Some(&receipt["legal_solver"]),
        )?;
        replay_require(
            saved_receipt == receipt && saved[0].receipt == proposals[0].receipt,
            "saved legal authentication must not solve",
        )?;
        let mut before = vec![0; native.vocab_size()];
        native.score_into(&b, &mut before, &mut Default::default())?;
        let caches = vec![
            red.prepare_generate_patch_cache(before.clone(), vec![4, 4, 5], vec![0, 0, 1 << 23])?,
            red.prepare_generate_patch_cache(before, vec![4, 5], vec![0, 1 << 23])?,
        ];
        let before_masses = caches
            .iter()
            .map(|c| c.token_masses().to_vec())
            .collect::<Vec<_>>();
        let old_inc = RowIncidence::new(&native, &b, &legal)?.digest()?;
        let (records, best, count) = evaluate(proposals, |p| {
            let mut e = native.energy().clone();
            for lane in 0usize..8 {
                for bin in 0usize..120 {
                    e.set_unary(
                        lane as u8,
                        bin as u8,
                        generate::code(p.unary[lane * 120 + bin])?,
                    )?;
                }
            }
            let g = NativeGeometricGenerate::compile(
                &binding,
                8,
                native.prototypes(),
                native.packed_biases(),
                e,
            )?;
            let rows = prepare_rows(2, |row| {
                let ids = if row == 0 { vec![4, 4, 5] } else { vec![4, 5] };
                let base = if row == 0 {
                    vec![0, 0, 1 << 23]
                } else {
                    vec![0, 1 << 23]
                };
                let keys = (0..ids.len())
                    .map(|i| {
                        (0..8)
                            .map(|lane| {
                                Some(
                                    lane * 120
                                        + if lane == 0 {
                                            if row == 0 {
                                                usize::from(i == 2)
                                            } else {
                                                2 * i
                                            }
                                        } else {
                                            0
                                        },
                                )
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
                let copy = shared::staged_base(&base, &keys, &pm, &p.prefix)?;
                let donor = earliest(&copy)?;
                let post = if donor == 0 { a.clone() } else { b.clone() };
                let mut raw = vec![0; g.vocab_size()];
                g.score_into(&post, &mut raw, &mut Default::default())?;
                let inc = RowIncidence::new(&g, &post, &legal)?;
                if row == 0 {
                    replay_require(inc.digest()? != old_inc, "changed native donor incidence")?;
                }
                Ok(Replacement {
                    cache: red.prepare_generate_patch_cache(raw, ids, copy)?,
                    post,
                    donor,
                    incidence: Some(inc),
                })
            })?;
            let required = caches[1].summary().chosen_token_id;
            let chosen = rows[&1].cache.summary().chosen_token_id;
            replay_require(
                required == 5 && chosen != required,
                "actual native late guard veto",
            )?;
            replay_require(rows[&0].donor == 0, "actual BASE donor changes")?;
            // A measured native protected-row veto drops the complete batch.
            Ok(Stage {
                rows,
                objective: json!({"combined":0.}),
                receipt: json!({"feasible":false,"guard_status":"FIRST_VETO","first_failure":{"guard_index":1,"required":required,"chosen":chosen},"candidate":0,"incumbent_epoch":0}),
            })
        })?;
        replay_require(
            best.is_none() && count == 2 && records[0]["native"]["incumbent_epoch"] == 0,
            "legal same epoch veto",
        )?;
        replay_require(
            caches
                .iter()
                .zip(before_masses)
                .all(|(c, m)| c.token_masses() == m),
            "veto incumbent masses unchanged",
        )?;
        Ok(())
    }
    #[test]
    fn feedback1920_recovery_and_duplicate_receipts() -> Result<()> {
        let mut pm = vec![0.; COUNT];
        pm[1] = 0.124;
        let mut pg = vec![0.; COUNT];
        pg[0] = -1.;
        pg[1] = -0.6;
        let mut j = vec![0.; DIM];
        j[0] = -0.59;
        j[1] = 1.;
        let (rounds, projection) = mode_proposals(
            PrefixTransaction::ProtectedDiscreteFeedback,
            &pm,
            &pg,
            &vec![0.; COUNT],
            &vec![0.; COUNT],
            &[j],
        )?;
        replay_require(
            rounds.len() == 32 && projection["feedback_status"] == "Completed",
            "feedback fixture rounds",
        )?;
        replay_require(
            !rounds[0].eligible && rounds.iter().any(|p| p.eligible),
            "feedback compensation fixture",
        )?;
        replay_require(rounds.iter().any(duplicate), "feedback duplicate fixture")?;
        for (i, r) in rounds.iter().enumerate() {
            replay_require(
                r.receipt["round"] == i
                    && r.receipt["destination_master_bits"]
                        .as_array()
                        .is_some_and(|x| x.len() == DIM),
                "feedback exact1920 receipt",
            )?;
        }
        Ok(())
    }
    #[test]
    fn joint_jacobian_reader_checks_1920_finite_gradients_without_weight_domain() -> Result<()> {
        let mut expected = vec![0.0f32; DIM];
        expected[0] = -3.0;
        expected[DIM - 1] = 2.0;
        let mut bytes = expected
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>();
        assert_eq!(decode_jacobian(&bytes)?, expected);
        assert!(decode_jacobian(&bytes[..COUNT * 4]).is_err());
        assert!(decode_jacobian(&bytes[..bytes.len() - 1]).is_err());
        bytes[..4].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(decode_jacobian(&bytes).is_err());
        Ok(())
    }

    #[test]
    fn cyclic_projection_rechecks_later_opposition_zero_rows_and_scale() -> Result<()> {
        let mut gradient = vec![0.; DIM];
        gradient[0] = 1.;
        gradient[1] = -2.;
        let mut a = vec![0.; DIM];
        a[0] = 1.;
        let mut b = vec![0.; DIM];
        b[0] = -1.;
        b[1] = -1.;
        let j = vec![a, b, vec![0.; DIM]];
        let (d, r) = project(&gradient, &j)?;
        assert_eq!(r["passes"], 256);
        assert_eq!(r["unit_row_zero"][2], true);
        assert_eq!(r["linear_constraints_passed"], false); // fixed passes do not certify convergence
        let (zero, z) = project(&vec![0.; DIM], &j)?;
        assert!(zero.iter().all(|x| *x == 0.));
        assert_eq!(z["tolerance"], 0.);
        let (_, tiny, _) = residuals(
            &unit_rows(&j)?,
            &d.iter().map(|x| x * 1e-20).collect::<Vec<_>>(),
        )?;
        assert!(tiny < 1e-25);
        assert!(project(&vec![f32::NAN; DIM], &j).is_err());
        assert!(unit_rows(&[vec![0.; DIM - 1]]).is_err());
        Ok(())
    }
    #[test]
    fn quantized_joint_direction_can_fail_even_when_float_halfspace_passes() -> Result<()> {
        // Float d=(1,.6) is feasible for row(-.59,1), but radius1 rounds its
        // second coordinate back to0; quantized delta(.25,0) breaks protection.
        let mut j = vec![0.; DIM];
        j[0] = -0.59;
        j[COUNT] = 1.;
        let mut pg = vec![0.; COUNT];
        pg[0] = -1.;
        let mut gg = vec![0.; COUNT];
        gg[0] = -0.6;
        let (p, r) = proposals(&vec![0.; COUNT], &pg, &vec![0.; COUNT], &gg, &[j])?;
        assert_eq!(r["linear_constraints_passed"], true);
        // Radius1 can round .15 to.25, so use an initial fractional master
        // retaining its original native code to demonstrate actualdelta loss.
        let mut gm = vec![0.; COUNT];
        gm[0] = 0.124;
        let (p2, _) = proposals(
            &vec![0.; COUNT],
            &pg,
            &gm,
            &gg,
            &[{
                let mut x = vec![0.; DIM];
                x[0] = -0.59;
                x[COUNT] = 1.;
                x
            }],
        )?;
        assert!(p
            .iter()
            .chain(&p2)
            .any(|x| x.receipt["quantized_constraints_passed"] == false));
        Ok(())
    }
    #[test]
    fn projection_preserves_fractional_bits_and_rejects_reverse_recovery() -> Result<()> {
        let declared = policy_for_modes(
            DonorCredit::FullPoolUtility,
            PrefixTransaction::ProtectedJointVector,
        );
        assert_eq!(declared["total_fresh_backward_calls"], 411);
        assert_eq!(declared["total_training_graph_forwards"], 822);
        assert_eq!(
            declared["generate"],
            "same joint transaction as Prefix; no follow-on coordinate pass"
        );
        assert!(declared["prefix"]
            .as_str()
            .unwrap_or("")
            .contains("originalm+eta*d"));
        let pm = vec![0.124f32; COUNT];
        let gm = vec![-0.124f32; COUNT];
        let (p, _) = proposals(
            &pm,
            &vec![0.; COUNT],
            &gm,
            &vec![0.; COUNT],
            &[vec![0.; DIM]],
        )?;
        assert!(p.iter().all(|p| !p.eligible
            && p.prefix.iter().all(|x| x.to_bits() == pm[0].to_bits())
            && p.unary.iter().all(|x| x.to_bits() == gm[0].to_bits())));
        assert!(validate_transaction(
            PrefixTransaction::ProtectedJointVector,
            DonorCredit::FullPoolUtility,
            true
        )
        .is_err());
        assert!(validate_transaction(
            PrefixTransaction::ProtectedJointVector,
            DonorCredit::StateTangent,
            false
        )
        .is_err());
        assert!(validate_inherited_transaction(
            PrefixTransaction::CoordinateAdjacent,
            PrefixTransaction::ProtectedJointVector,
            &json!({"prefix_transaction":"protected_joint_vector"})
        )
        .is_err());
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
        let a = shared::codes(&json!(vec![1; 8]))?;
        let mut b = a.clone();
        b[0] = shared::codes(&json!(vec![2; 8]))?[0];
        Ok((red, native, legal, a, b, binding))
    }
    #[test]
    fn native_joint_driver_restage_changes_donor_unary_incidence_and_late_veto_rolls_back(
    ) -> Result<()> {
        let (mut red, native, legal, a, b, binding) = fixture()?;
        let mut factor = [0u32; 8];
        native.factor_incidence_into(&a, 5, &mut factor, &mut Default::default())?;
        let unary_key = factor[0] as usize;
        let pm = vec![0.; COUNT];
        let mut pg = vec![0.; COUNT];
        pg[0] = -1.;
        pg[1] = 1.;
        let gm = vec![0.; COUNT];
        let mut gg = vec![0.; COUNT];
        gg[unary_key] = -1.;
        let (proposals, _) = proposals(&pm, &pg, &gm, &gg, &[vec![0.; DIM]])?;
        let original_inc = RowIncidence::new(&native, &b, &legal)?;
        let original_digest = original_inc.digest()?;
        let mut scores = vec![0; native.vocab_size()];
        native.score_into(&b, &mut scores, &mut Default::default())?;
        let mut caches = vec![
            red.prepare_generate_patch_cache(scores.clone(), vec![4, 4, 5], vec![0, 0, 1 << 23])?,
            red.prepare_generate_patch_cache(scores.clone(), vec![4, 5], vec![-1677722, 11744051])?,
        ];
        let original = caches
            .iter()
            .map(|c| {
                (
                    c.generate_scores().to_vec(),
                    c.copy_scores().to_vec(),
                    c.summary().clone(),
                )
            })
            .collect::<Vec<_>>();
        let mut posts = vec![b.clone(), b.clone()];
        let mut donors = vec![2, 1];
        let mut incidence = vec![RowIncidence::new(&native, &b, &legal)?, original_inc];
        let keys = |n: usize| {
            (0..n)
                .map(|i| {
                    (0..8)
                        .map(|lane| {
                            Some(
                                lane * 120
                                    + if lane == 0 {
                                        if i + 1 == n {
                                            if n == 3 {
                                                1
                                            } else {
                                                2
                                            }
                                        } else {
                                            0
                                        }
                                    } else {
                                        0
                                    },
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        };
        let build = |proposal: &Proposal, red: &mut NativeVocabularyActions| -> Result<Stage> {
            let mut energy = native.energy().clone();
            energy.set_unary(
                0,
                (unary_key % 120) as u8,
                generate::code(proposal.unary[unary_key])?,
            )?;
            let changed = NativeGeometricGenerate::compile(
                &binding,
                8,
                native.prototypes(),
                native.packed_biases(),
                energy,
            )?;
            let rows = prepare_rows(2, |row| {
                let base = if row == 0 {
                    vec![0, 0, 1 << 23]
                } else {
                    vec![-1677722, 11744051]
                };
                let ids = if row == 0 { vec![4, 4, 5] } else { vec![4, 5] };
                let copy = shared::staged_base(&base, &keys(ids.len()), &pm, &proposal.prefix)?;
                let donor = earliest(&copy)?;
                let post = if donor == 0 { a.clone() } else { b.clone() };
                let mut g = vec![0; changed.vocab_size()];
                changed.score_into(&post, &mut g, &mut Default::default())?;
                let cache = red.prepare_generate_patch_cache(g, ids, copy)?;
                Ok(Replacement {
                    cache,
                    post: post.clone(),
                    donor,
                    incidence: Some(RowIncidence::new(&changed, &post, &legal)?),
                })
            })?;
            let c = &rows[&0].cache;
            let ce = -(c.token_masses()[4] as f64 / c.summary().total_weight_q31 as f64).ln();
            let guard = rows[&1].cache.summary().chosen_token_id == 5;
            let receipt = json!({"radius":proposal.radius,"incumbent_epoch":0,"feasible":guard,"guard_status":if guard{"FULL_PASS"}else{"FIRST_VETO"},
                "objective":{"combined":ce},"rows":rows.iter().map(|(i,r)|Ok::<_,Box<dyn std::error::Error>>(json!([i,r.donor,r.post.iter().map(|x|x.index()).collect::<Vec<_>>(),r.cache.summary(),r.incidence.as_ref().map(|x|x.digest()).transpose()?]))).collect::<Result<Vec<_>>>()?});
            Ok(Stage {
                rows,
                objective: json!({"combined":ce}),
                receipt,
            })
        };
        let (records, best, count) = evaluate(proposals, |p| build(p, &mut red))?;
        assert_eq!(records.len(), 4);
        assert_eq!(count, 8);
        assert!(records.iter().any(|r| r["native"]["feasible"] == false));
        assert!(records.iter().any(|r| r["native"]["feasible"] == true));
        // No attempted radius changes either incumbent family/cache/donor/incidence.
        for (i, c) in caches.iter().enumerate() {
            assert_eq!(c.generate_scores(), original[i].0);
            assert_eq!(c.copy_scores(), original[i].1);
            assert_eq!(c.summary(), original[i].2);
        }
        assert_eq!(donors, vec![2, 1]);
        assert_eq!(incidence[0].digest()?, original_digest);
        let (_, selected, receipt) = best.ok_or_else(|| bad("fixture no feasible radius"))?;
        let restaged = build(&selected, &mut red)?;
        assert_eq!(restaged.receipt, receipt);
        assert_eq!(restaged.rows[&0].donor, 0);
        assert_ne!(
            restaged.rows[&0]
                .incidence
                .as_ref()
                .ok_or_else(|| bad("fixture incidence"))?
                .digest()?,
            original_digest
        );
        apply(
            restaged,
            &mut caches,
            &mut posts,
            &mut donors,
            &mut incidence,
        )?;
        assert_eq!(posts[0], a);
        assert_eq!(donors[0], 0);
        assert_ne!(caches[0].generate_scores(), original[0].0);
        assert!(incidence[0].matching(unary_key).contains(&5));
        Ok(())
    }
    #[test]
    fn invalid_joint_batch_prevalidation_preserves_both_cache_domains() -> Result<()> {
        let (mut red, native, legal, a, _, _) = fixture()?;
        let mut raw = vec![0; native.vocab_size()];
        native.score_into(&a, &mut raw, &mut Default::default())?;
        let mut caches =
            vec![red.prepare_generate_patch_cache(raw.clone(), vec![4, 4], vec![0, 0])?];
        let before = caches[0].summary().clone();
        let mut posts = vec![a.clone()];
        let mut donors = vec![0];
        let mut incidence = vec![RowIncidence::new(&native, &a, &legal)?];
        let mut rows = BTreeMap::new();
        rows.insert(
            1,
            Replacement {
                cache: red.prepare_generate_patch_cache(raw, vec![5], vec![0])?,
                post: a,
                donor: 0,
                incidence: None,
            },
        );
        assert!(apply(
            Stage {
                rows,
                objective: json!({}),
                receipt: json!({})
            },
            &mut caches,
            &mut posts,
            &mut donors,
            &mut incidence
        )
        .is_err());
        assert_eq!(caches[0].summary(), before);
        assert_eq!(donors, vec![0]);
        Ok(())
    }
}
