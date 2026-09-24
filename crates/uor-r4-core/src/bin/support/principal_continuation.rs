//! Principal continuation harness: frozen full-tune adjudication, integer readout,
//! observation-depth controls, fresh-source evaluation and actual warm starts.
use super::*;
use std::{path::Path, time::Instant};
use uor_r4_core::native_geometric::learner::lexical_residual as lr;
use uor_r4_core::native_geometric::learner::realtext_support::DocRec;
const ALPHA: [u8; 7] = [28, 30, 32, 34, 36, 38, 40];
const GAMMAS: usize = 9;
const KINDS: [&str; 5] = [
    "full_state",
    "last_two_only",
    "older_reversed",
    "unigram",
    "cur_only",
];
fn lse(v: &[f64]) -> f64 {
    let m = v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    m + v.iter().map(|x| (x - m).exp2()).sum::<f64>().log2()
}
fn logdist(model: &TlModel, h: &[i32], f: &[i32], event: usize) -> (Vec<i32>, Vec<f64>, f64) {
    let z = model.readout(h, &vec![0; model.h_dim], f, event);
    let scale = 2f64.powi(-(model.score_shift as i32));
    let a: Vec<f64> = z[..model.vocab]
        .iter()
        .map(|x| f64::from(*x) * scale)
        .collect();
    let lg = lse(&a);
    let stop = ((f64::from(z[model.stop_row()]) * scale - lg).exp2() + 1.0).log2();
    (z, a.iter().map(|x| x - lg).collect(), stop)
}
fn grids(a: &[f64], c: &[f64], target: usize, stop: f64) -> Vec<f64> {
    let mut sums = [0.0f64; 63];
    for (&av, &cv) in a.iter().zip(c) {
        let ac = (av / 32.0).exp2();
        let cc = (cv / 16.0).exp2();
        let mut cp = cc.powi(14);
        for ai in 0..7 {
            let mut ap = 1.0;
            for gi in 0..GAMMAS {
                sums[ai * GAMMAS + gi] += cp * ap;
                ap *= ac;
            }
            cp *= cc;
        }
    }
    sums.iter()
        .enumerate()
        .map(|(i, s)| {
            s.log2()
                - f64::from(ALPHA[i / GAMMAS]) / 32.0 * c[target]
                - (i % GAMMAS) as f64 / 32.0 * a[target]
                + stop
        })
        .collect()
}
struct GridResult {
    totals: Vec<Vec<Eval>>,
    head: Vec<Vec<(f64, usize)>>,
    tail: Vec<Vec<(f64, usize)>>,
    native: Eval,
    count: Eval,
}
fn grid_eval(
    model: &TlModel,
    ws: &[ProseWindow],
    docs: usize,
    est: &CountEstimate,
    l: (f64, f64),
    top: &[bool],
) -> Result<GridResult, String> {
    let ratio = std::env::var("UOR_PRINCIPAL_RATIO").as_deref() == Ok("1");
    let mut result = GridResult {
        totals: (0..5)
            .map(|_| (0..63).map(|_| eval_new(docs)).collect())
            .collect(),
        head: vec![vec![(0.0, 0); 63]; 5],
        tail: vec![vec![(0.0, 0); 63]; 5],
        native: eval_new(docs),
        count: eval_new(docs),
    };
    let m = vec![0; model.h_dim];
    let f = model.typed_block(&[], &[], SlFacts::default());
    let u: Vec<f64> = (0..VOCAB).map(|v| est.uni.p(v as u32).log2()).collect();
    let mut probs = vec![0.; VOCAB];
    let mut curprob = probs.clone();
    let started = Instant::now();
    for (wi, w) in ws.iter().enumerate() {
        let mut h = model.init_state(&m, &f);
        for (j, &t) in w.tokens.iter().enumerate() {
            if j >= PREFIX {
                let event = if j == PREFIX {
                    TL_EV_OBSERVE
                } else {
                    TL_EV_GENERATE
                };
                let (_, a, stop) = logdist(model, &h, &f, event);
                let p = w.tokens[j - 2] as usize;
                let c = w.tokens[j - 1] as usize;
                pc_dense_into(&est.row1, &est.row2, &est.uni, p, c, l, &mut probs);
                let logs: Vec<f64> = probs.iter().map(|p| p.log2()).collect();
                e1_dense_into(&est.row1, &est.uni, c, l.0, &mut curprob);
                let one: Vec<f64> = curprob.iter().map(|p| p.log2()).collect();
                let mut th = model.init_state(&m, &f);
                for k in j - 2..j {
                    th = model.transition(
                        &th,
                        if k < PREFIX {
                            TL_EV_OBSERVE
                        } else {
                            TL_EV_GENERATE
                        },
                        Some(w.tokens[k]),
                        &m,
                        &f,
                    );
                }
                let (_, tail, _) = logdist(model, &th, &f, event);
                let mut rh = model.init_state(&m, &f);
                for (k, &v) in w.tokens[..j - 2]
                    .iter()
                    .rev()
                    .chain(w.tokens[j - 2..j].iter())
                    .enumerate()
                {
                    rh = model.transition(
                        &rh,
                        if k < PREFIX {
                            TL_EV_OBSERVE
                        } else {
                            TL_EV_GENERATE
                        },
                        Some(v),
                        &m,
                        &f,
                    );
                }
                let (_, rev, _) = logdist(model, &rh, &f, event);
                for (kind, dist) in [&a, &tail, &rev, &u, &one].iter().enumerate() {
                    let relative: Vec<f64>;
                    let observed: &[f64] = if ratio {
                        relative = dist.iter().zip(&u).map(|(a, u)| a - u).collect();
                        &relative
                    } else {
                        dist.as_slice()
                    };
                    let curve = grids(observed, &logs, t as usize, stop);
                    for (i, &bits) in curve.iter().enumerate() {
                        eval_add(&mut result.totals[kind][i], w.doc, bits);
                        let strata = if top[t as usize] {
                            &mut result.head
                        } else {
                            &mut result.tail
                        };
                        strata[kind][i].0 += bits;
                        strata[kind][i].1 += 1;
                    }
                }
                eval_add(&mut result.native, w.doc, -a[t as usize] + stop);
                eval_add(&mut result.count, w.doc, -logs[t as usize] + stop);
            }
            h = model.transition(
                &h,
                if j < PREFIX {
                    TL_EV_OBSERVE
                } else {
                    TL_EV_GENERATE
                },
                Some(t),
                &m,
                &f,
            );
        }
        if wi % 128 == 0 {
            println!(
                "GRID windows={}/{} seconds={:.3}",
                wi + 1,
                ws.len(),
                started.elapsed().as_secs_f64()
            );
        }
    }
    Ok(result)
}
fn pick(z: &[i32], rows: &[usize]) -> usize {
    let mut best = rows[0];
    for &r in rows.iter().skip(1) {
        if z[r] > z[best] {
            best = r
        }
    }
    best
}
fn qbits(z: &[i32], rows: &[usize], target: usize) -> f64 {
    let xs: Vec<f64> = rows
        .iter()
        .map(|r| f64::from(z[*r]) / lr::ONE as f64)
        .collect();
    lse(&xs) - f64::from(z[target]) / lr::ONE as f64
}
fn eval_integer(
    model: &TlModel,
    prior: &lr::IntegerPrior,
    ws: &[ProseWindow],
    docs: usize,
    est: &CountEstimate,
    l: (f64, f64),
    a: u8,
    g: u8,
    ratio: bool,
    top: &[bool],
) -> Result<serde_json::Value, String> {
    let mut native = eval_new(docs);
    let mut integer = eval_new(docs);
    let mut ideal = eval_new(docs);
    let mut max_error = 0.0f64;
    let mut mass_error = 0i64;
    let mut raw_changed = 0;
    let mut head = (0.0, 0);
    let mut tail = (0.0, 0);
    let m = vec![0; model.h_dim];
    let f = model.typed_block(&[], &[], SlFacts::default());
    let rows = model.legal_rows(false);
    let mut c = vec![0.; VOCAB];
    let timer = Instant::now();
    for w in ws {
        let mut h = model.init_state(&m, &f);
        for (j, &t) in w.tokens.iter().enumerate() {
            if j >= PREFIX {
                let ev = if j == PREFIX {
                    TL_EV_OBSERVE
                } else {
                    TL_EV_GENERATE
                };
                let (z, log_a, stop) = logdist(model, &h, &f, ev);
                let qs = prior.score(w.tokens[j - 2], w.tokens[j - 1])?;
                let out =
                    compose_selected(prior, ratio, &z, &qs, model.score_shift, a, g, &prior.math)?;
                pc_dense_into(
                    &est.row1,
                    &est.row2,
                    &est.uni,
                    w.tokens[j - 2] as usize,
                    w.tokens[j - 1] as usize,
                    l,
                    &mut c,
                );
                let ideal_z: Vec<f64> = c
                    .iter()
                    .zip(&log_a)
                    .enumerate()
                    .map(|(i, (c, x))| {
                        f64::from(a) / 32. * c.log2()
                            + f64::from(g) / 32.
                                * (x - if ratio {
                                    est.uni.p(i as u32).log2()
                                } else {
                                    0.0
                                })
                    })
                    .collect();
                let ib = lse(&ideal_z) - ideal_z[t as usize] + stop;
                let qb = qbits(&out, &rows, t as usize);
                max_error = max_error.max((ib - qb).abs());
                eval_add(&mut native, w.doc, -log_a[t as usize] + stop);
                eval_add(&mut ideal, w.doc, ib);
                eval_add(&mut integer, w.doc, qb);
                let r = if top[t as usize] {
                    &mut head
                } else {
                    &mut tail
                };
                r.0 += qb;
                r.1 += 1;
                raw_changed += usize::from(pick(&z, &rows) != pick(&out, &rows));
                let native_q: Vec<i32> = z[..VOCAB]
                    .iter()
                    .map(|x| x << (lr::FRAC - model.score_shift))
                    .collect();
                mass_error = mass_error.max(
                    (i64::from(prior.math.sum(&out[..VOCAB]))
                        - i64::from(prior.math.sum(&native_q)))
                    .abs(),
                );
            }
            h = model.transition(
                &h,
                if j < PREFIX {
                    TL_EV_OBSERVE
                } else {
                    TL_EV_GENERATE
                },
                Some(t),
                &m,
                &f,
            );
        }
    }
    if max_error > 0.02 || mass_error != 0 {
        return Err(format!(
            "integer parity envelope exceeded {max_error} / {mass_error}"
        ));
    }
    Ok(
        serde_json::json!({"native_bits":native.micro(),"integer_bits":integer.micro(),"ideal_bits":ideal.micro(),"integer_minus_native":paired_interval(&integer,&native,20260923),"max_per_target_quantization_error_bits":max_error,"generate_logmass_error_codes":mass_error,"raw_top1_changes":raw_changed,"head":head,"tail":tail,"integer_documents":integer.per_doc,"ideal_documents":ideal.per_doc,"seconds":timer.elapsed().as_secs_f64()}),
    )
}
fn generations(
    model: &TlModel,
    prior: &lr::IntegerPrior,
    ws: &[ProseWindow],
    tok: &HfBpeTokenizer,
    a: u8,
    g: u8,
    ratio: bool,
) -> Result<serde_json::Value, String> {
    let mut output = Vec::new();
    let m = vec![0; model.h_dim];
    let f = model.typed_block(&[], &[], SlFacts::default());
    let rows = model.legal_rows(false);
    for w in ws.iter().step_by(4).take(6) {
        for composed in [false, true] {
            let mut history = w.tokens[..PREFIX].to_vec();
            let mut h = model.init_state(&m, &f);
            for t in &history {
                h = model.transition(&h, TL_EV_OBSERVE, Some(*t), &m, &f);
            }
            let mut generated = Vec::new();
            let mut stopped = false;
            for j in 0..48 {
                let z = model.readout(
                    &h,
                    &m,
                    &f,
                    if j == 0 {
                        TL_EV_OBSERVE
                    } else {
                        TL_EV_GENERATE
                    },
                );
                let n = history.len();
                let out = if composed {
                    compose_selected(
                        prior,
                        ratio,
                        &z,
                        &prior.score(history[n - 2], history[n - 1])?,
                        model.score_shift,
                        a,
                        g,
                        &prior.math,
                    )?
                } else {
                    z
                };
                let v = pick(&out, &rows);
                if v == model.stop_row() {
                    stopped = true;
                    break;
                }
                let t = v as u32;
                generated.push(t);
                history.push(t);
                h = model.transition(&h, TL_EV_GENERATE, Some(t), &m, &f);
            }
            output.push(serde_json::json!({"doc":w.doc,"composite":composed,"prompt":tok.decode(&w.tokens[..PREFIX]),"generated":tok.decode(&generated),"tokens":generated,"stopped":stopped}));
        }
    }
    Ok(serde_json::json!(output))
}
fn depth_probe(model: &TlModel, ws: &[ProseWindow]) -> Result<serde_json::Value, String> {
    let f = model.typed_block(&[], &[], SlFacts::default());
    let m = vec![0; model.h_dim];
    let mut extra = m.clone();
    extra.extend_from_slice(&f);
    extra.extend((0..TL_EVENTS).map(|e| i32::from(e == TL_EV_GENERATE)));
    let mut b = model.wf.forward_i32(&extra)?;
    for (i, x) in b.iter_mut().enumerate() {
        *x += model.bh[i];
    }
    let embeddings: Vec<Vec<i32>> = (0..VOCAB)
        .map(|t| (0..model.h_dim).map(|i| model.e.value(t, i)).collect())
        .collect();
    let mut banks = vec![embeddings.clone()];
    let mut next: Vec<Vec<i32>> = embeddings
        .iter()
        .map(|x| x.iter().zip(&b).map(|(a, b)| a + b).collect())
        .collect();
    for _ in 0..3 {
        next = next
            .iter()
            .map(|x| {
                model
                    .wh
                    .forward_i32(x)
                    .map(|v| v.iter().map(|x| x >> model.recurrent_shift).collect())
            })
            .collect::<Result<_, _>>()?;
        banks.push(next.clone());
    }
    let nearest = |x: &[i32], bank: &[Vec<i32>]| {
        let (mut best, mut score) = (0, i64::MAX);
        for (i, v) in bank.iter().enumerate() {
            let mut d = 0;
            for (a, b) in x.iter().zip(v) {
                let z = i64::from(*a) - i64::from(*b);
                d += z * z;
                if d > score {
                    break;
                }
            }
            if d < score {
                score = d;
                best = i;
            }
        }
        best
    };
    let mut correct = [0usize; 4];
    let mut wrong = [0usize; 4];
    let mut rotated = [0usize; 4];
    let mut n = 0;
    for w in ws {
        let mut h = model.init_state(&m, &f);
        for (j, &t) in w.tokens.iter().enumerate() {
            if j >= PREFIX + 4 {
                n += 1;
                let mut residual: Vec<i32> = h.iter().zip(&b).map(|(x, b)| x - b).collect();
                for lag in 0..4 {
                    let id = nearest(&residual, &banks[lag]);
                    correct[lag] += usize::from(id == w.tokens[j - 1 - lag] as usize);
                    wrong[lag] += usize::from(
                        nearest(&residual, &banks[0]) == w.tokens[j - 1 - lag] as usize,
                    );
                    rotated[lag] +=
                        usize::from(id == w.tokens[(j + 17 - lag) % w.tokens.len()] as usize);
                    for i in 0..residual.len() {
                        residual[i] -= banks[lag][id][i];
                    }
                }
            }
            h = model.transition(
                &h,
                if j < PREFIX {
                    TL_EV_OBSERVE
                } else {
                    TL_EV_GENERATE
                },
                Some(t),
                &m,
                &f,
            );
        }
    }
    Ok(
        serde_json::json!({"positions":n,"lag_tokens":[1,2,3,4],"correct":correct,"wrong_frame":wrong,"rotated_label":rotated,"side_information":"hidden state and fixed Generate event offset only; no true token passed to self-decoder","scope":"approximate finite transported prototypes, open-development lower bound on recoverable identity, not a universal absence test"}),
    )
}
fn write_json(root: &Path, name: &str, value: &serde_json::Value) -> Result<(), String> {
    write_checked(
        root,
        name,
        &serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
}
fn bundle(
    model: &TlModel,
    prior: &lr::IntegerPrior,
    a: u8,
    g: u8,
    ratio: bool,
) -> Result<Vec<u8>, String> {
    let m = model.to_bytes();
    let p = prior.to_bytes();
    let mut b = b"PRC2".to_vec();
    b.extend_from_slice(&(m.len() as u32).to_le_bytes());
    b.extend_from_slice(&(p.len() as u32).to_le_bytes());
    b.extend([a, g, u8::from(ratio)]);
    b.extend(m);
    b.extend(p);
    Ok(b)
}
fn load_bundle(b: &[u8]) -> Result<(TlModel, lr::IntegerPrior, u8, u8, bool), String> {
    if b.len() < 14 || b.len() > 64 * 1024 * 1024 {
        return Err("invalid composite length".into());
    }
    let (head, ratio) = match &b[..4] {
        b"PRC1" => (14, false),
        b"PRC2" if b.len() >= 15 && b[14] <= 1 => (15, b[14] == 1),
        _ => return Err("invalid composite format or correction frame".into()),
    };
    let n = u32::from_le_bytes(b[4..8].try_into().map_err(|_| "model size")?) as usize;
    let p = u32::from_le_bytes(b[8..12].try_into().map_err(|_| "prior size")?) as usize;
    if n.checked_add(p).and_then(|x| x.checked_add(head)) != Some(b.len())
        || b[12] > 64
        || b[13] > 32
    {
        return Err("invalid composite shape".into());
    }
    let native = &b[head..head + n];
    let model = TlModel::from_bytes(native)?;
    let binding: [u8; 32] = Sha256::digest(native).into();
    let prior = lr::IntegerPrior::from_bytes(&b[head + n..], binding)?;
    if model.vocab != prior.vocab {
        return Err("composite vocabulary mismatch".into());
    }
    Ok((model, prior, b[12], b[13], ratio))
}
fn compose_selected(
    prior: &lr::IntegerPrior,
    ratio: bool,
    z: &[i32],
    c: &[i32],
    shift: u32,
    a: u8,
    g: u8,
    math: &lr::LogAdd,
) -> Result<Vec<i32>, String> {
    if ratio {
        lr::compose_ratio(z, c, prior.unigram(), shift, a, g, math)
    } else {
        lr::compose(z, c, shift, a, g, math)
    }
}
fn panel_summary(model: &TlModel, cases: &[Grounded]) -> serde_json::Value {
    let p = grounded_panel(model, cases);
    let correct = p
        .iter()
        .filter(|r| r["ok"] == true || r["exact"] == true)
        .count();
    serde_json::json!({"correct":correct,"total":p.len(),"rows":p})
}
pub fn run(args: &Args) -> Result<ExitCode, String> {
    let mode = std::env::var("UOR_PRINCIPAL_RUN").map_err(|e| e.to_string())?;
    let root = args
        .state_probe
        .clone()
        .ok_or("new --state-probe report root required")?;
    claim(&root).map_err(|e| e.to_string())?;
    let started = Instant::now();
    let artifact = args.artifact.clone().ok_or("artifact required")?;
    let mb = std::fs::read(&artifact).map_err(|e| e.to_string())?;
    let model = TlModel::from_bytes(&mb)?;
    if model.vocab != VOCAB || model.score_shift > lr::FRAC {
        return Err("diagnostic requires pinned 4096-vocabulary/dyadic scale".into());
    }
    let tb = std::fs::read(&args.tokenizer).map_err(|e| e.to_string())?;
    let tsha = sha256_hex(&Sha256::digest(&tb));
    if tsha != EXPECTED_TOKENIZER_SHA256 {
        return Err("tokenizer identity mismatch".into());
    }
    let tok = derive_tokenizer(&tb, VOCAB).map_err(|e| e.to_string())?;
    let (uniq, collected, duplicates) = reconstruct_corpus(&args.docs);
    let mut fit = Vec::new();
    let mut tune = Vec::new();
    let mut devpool = Vec::new();
    let mut tune_names = Vec::new();
    for (i, d) in uniq.iter().enumerate() {
        let ts = tok.encode(&d.text);
        match d.split {
            Split::Fit => fit.extend(prose_windows(&ts, i)),
            Split::Tune => {
                let n = tune_names.len();
                tune_names.push(d.path.clone());
                tune.extend(prose_windows(&ts, n));
            }
            Split::Dev => devpool.push((i, ts.len())),
        }
    }
    if fit.len() > FIT_MAX_WINDOWS {
        let stride = fit.len() as f64 / FIT_MAX_WINDOWS as f64;
        fit = (0..FIT_MAX_WINDOWS)
            .map(|k| fit[(k as f64 * stride) as usize].clone())
            .collect();
    }
    let tune_all: Vec<Vec<u32>> = tune.iter().map(|w| w.tokens.clone()).collect();
    if tune.len() > 2048 {
        let stride = tune.len() as f64 / 2048.;
        tune = (0..2048)
            .map(|k| tune[(k as f64 * stride) as usize].clone())
            .collect();
    }
    devpool.sort_by_key(|(_, len)| *len);
    let mut dev = Vec::new();
    let mut names = Vec::new();
    for k in 0..24.min(devpool.len()) {
        let i = devpool[k * devpool.len() / 24.min(devpool.len())].0;
        names.push(uniq[i].path.clone());
        dev.extend(
            prose_windows(&tok.encode(&uniq[i].text), k)
                .into_iter()
                .take(4),
        );
    }
    let est = build_count_estimate(&fit);
    let (legacy_l, _) = tune_lambdas(&est.c1, &est.c2, &est.uni, &tune_all);
    let matched = std::env::var("UOR_PRINCIPAL_MATCHED_COUNTS").as_deref() == Ok("1");
    let l = if matched {
        let c = matched_count_calibration(&est, &tune, &dev, legacy_l);
        write_json(&root, "matched-count-tuning.json", &c)?;
        (
            c["selected_lambdas"][0].as_f64().ok_or("lambda1")?,
            c["selected_lambdas"][1].as_f64().ok_or("lambda2")?,
        )
    } else {
        legacy_l
    };
    let mut ids: Vec<usize> = (0..VOCAB).collect();
    ids.sort_by_key(|i| (std::cmp::Reverse(est.uni.counts[*i]), *i));
    let mut top = vec![false; VOCAB];
    for i in &ids[..1024] {
        top[*i] = true;
    }
    let triples: Vec<(u32, u32, u32)> = fit
        .iter()
        .flat_map(|w| {
            (PREFIX..w.tokens.len()).map(|k| (w.tokens[k - 2], w.tokens[k - 1], w.tokens[k]))
        })
        .collect();
    let binding: [u8; 32] = Sha256::digest(&mb).into();
    let prior = lr::IntegerPrior::compile(VOCAB, &triples, l, binding)?;
    let meta = serde_json::json!({"source_revision":args.source_rev,"mode":mode,"artifact":artifact,"artifact_sha256":sha256_hex(&binding),"tokenizer_sha256":tsha,"collected":collected,"duplicates":duplicates,"fit_targets":triples.len(),"tune_targets":tune.iter().map(|w|w.tokens.len()-PREFIX).sum::<usize>(),"tune_windows":tune.len(),"dev_targets":dev.iter().map(|w|w.tokens.len()-PREFIX).sum::<usize>(),"count_lambdas":l,"legacy_count_lambdas":legacy_l,"matched_count_tuning":matched,"prior_serialized_bytes":prior.to_bytes().len(),"sparse_count_entries":prior.count_entries(),"source_sha256":sha256_hex(&Sha256::digest(include_bytes!("principal_continuation.rs")))});
    write_json(&root, "attempt.json", &meta)?;
    write_json(&root,"corpus-identity.json",&serde_json::json!(uniq.iter().map(|d|serde_json::json!({"path":d.path,"sha256":sha256_hex(&d.sha256),"split":d.split.name()})).collect::<Vec<_>>()))?;
    let result = match mode.as_str() {
        "adjudicate" => adjudicate(
            &root,
            &model,
            &prior,
            &fit,
            &tune,
            &dev,
            &names,
            &tune_names,
            &est,
            l,
            &top,
            &tok,
        )?,
        "causal" => super::causal_continuation::run(&root, &model, &fit, &dev, &tok)?,
        "depth" => depth_probe(&model, &dev)?,
        "count-calibration" => matched_count_calibration(&est, &tune, &dev, legacy_l),
        "realize" => realize_selected(&root, &model, &prior, &dev, &names, &est, l, &top, &tok)?,
        "score" => score_recorded(&model, &dev, &names, &top, &tok)?,
        "warm" => warm_run(&root, &model, &fit, &dev, &tok)?,
        "grounded" => grounded_transfer(&root, &model, &prior, &dev, &tok, &est, l, &top)?,
        "cost" => cost_probe(&model, &prior, &dev)?,
        "fresh" => fresh_run(&root, &model, &prior, &uniq, &est, l, &top, &tok)?,
        _ => return Err("unknown principal mode".into()),
    };
    write_json(
        &root,
        "receipt.json",
        &serde_json::json!({"metadata":meta,"result":result,"seconds":started.elapsed().as_secs_f64(),"scope":"Research component only. No broad language, geometric superiority or energy qualification."}),
    )?;
    seal(&root).map_err(|e| e.to_string())?;
    let errors = verify(&root).map_err(|e| e.to_string())?;
    if !errors.is_empty() {
        return Err(format!("seal verification {errors:?}"));
    }
    println!(
        "FINISHED {} seconds={:.3}",
        root.display(),
        started.elapsed().as_secs_f64()
    );
    Ok(ExitCode::SUCCESS)
}
#[allow(clippy::too_many_arguments)]
fn adjudicate(
    root: &Path,
    model: &TlModel,
    prior: &lr::IntegerPrior,
    _fit: &[ProseWindow],
    tune: &[ProseWindow],
    dev: &[ProseWindow],
    names: &[String],
    tn: &[String],
    est: &CountEstimate,
    l: (f64, f64),
    top: &[bool],
    tok: &HfBpeTokenizer,
) -> Result<serde_json::Value, String> {
    let ratio = std::env::var("UOR_PRINCIPAL_RATIO").as_deref() == Ok("1");
    let t = grid_eval(model, tune, tn.len(), est, l, top)?;
    if t.native.per_doc.iter().map(|x| x.1).sum::<usize>() != 114364 {
        return Err("full Tune population does not match 114364".into());
    }
    let selected: Vec<usize> = t
        .totals
        .iter()
        .map(|g| {
            (0..g.len())
                .min_by(|a, b| g[*a].micro().total_cmp(&g[*b].micro()))
                .unwrap_or(0)
        })
        .collect();
    let ci = (0..7)
        .min_by(|a, b| {
            t.totals[0][*a * 9]
                .micro()
                .total_cmp(&t.totals[0][*b * 9].micro())
        })
        .unwrap_or(0)
        * 9;
    let spec = serde_json::json!({"selection":"Tune only; fixed dyadic 7x9 grid separately for five explanatory inputs","likelihood_ratio_to_fit_unigram":ratio,"alpha_numerators":ALPHA,"gamma_numerators":[0,1,2,3,4,5,6,7,8],"denominator":32,"kinds":KINDS,"selected_indices":selected,"calibrated_count_index":ci,"calibrated_count_alpha_numerator":ALPHA[ci/9],"alpha":ALPHA[selected[0]/9],"gamma":selected[0]%9,"tune_losses":t.totals.iter().map(|g|g.iter().map(Eval::micro).collect::<Vec<_>>()).collect::<Vec<_>>(),"tune_native":t.native.micro(),"tune_counts":t.count.micro()});
    write_json(root, "selection.json", &spec)?;
    let a = ALPHA[selected[0] / 9];
    let g = (selected[0] % 9) as u8;
    let bytes = bundle(model, prior, a, g, ratio)?;
    let (loaded, lp, la, lg, lr_ratio) = load_bundle(&bytes)?;
    if loaded != *model || lp != *prior || la != a || lg != g || lr_ratio != ratio {
        return Err("bundle round-trip mismatch".into());
    }
    write_checked(root, "candidate.prc", &bytes)?;
    let d = grid_eval(model, dev, names.len(), est, l, top)?;
    let base = &d.totals[0][ci];
    let rows:Vec<_>=(0..5).map(|k|{let i=selected[k];let h=d.head[k][i];let tail=d.tail[k][i];let rh=d.head[0][18];let rt=d.tail[0][18];serde_json::json!({"kind":KINDS[k],"alpha":ALPHA[i/9],"gamma":i%9,"denominator":32,"bits":d.totals[k][i].micro(),"minus_calibrated_count":paired_interval(&d.totals[k][i],base,20260923),"minus_full_state":paired_interval(&d.totals[k][i],&d.totals[0][selected[0]],20260923),"head_bits":h.0/h.1 as f64,"head_n":h.1,"tail_bits":tail.0/tail.1 as f64,"tail_n":tail.1,"head_delta_raw_count":h.0/h.1 as f64-rh.0/rh.1 as f64,"tail_delta_raw_count":tail.0/tail.1 as f64-rt.0/rt.1 as f64,"documents":d.totals[k][i].per_doc})}).collect();
    let integer = eval_integer(&loaded, &lp, dev, names.len(), est, l, a, g, ratio, top)?;
    let words = read_words(tok)?;
    let (train, class, held) = authored_world(&words);
    let grounding = serde_json::json!({"native_train":panel_summary(model,&train),"native_class":panel_summary(model,&class),"native_heldout":panel_summary(model,&held),"composite_train":ground_composite(&loaded,&lp,&train,a,g,ratio)?,"composite_class":ground_composite(&loaded,&lp,&class,a,g,ratio)?,"composite_heldout":ground_composite(&loaded,&lp,&held,a,g,ratio)?});
    write_json(
        root,
        "generation.json",
        &generations(&loaded, &lp, dev, tok, a, g, ratio)?,
    )?;
    write_json(root, "grounded.json", &grounding)?;
    Ok(
        serde_json::json!({"selection":spec,"grid_scope":"bounded coefficients; an endpoint optimum is not an unrestricted optimum","native_bits":d.native.micro(),"raw_count_bits":d.count.micro(),"calibrated_count_bits":base.micro(),"comparisons":rows,"integer":integer,"bundle_bytes":bytes.len(),"bundle_sha256":sha256_hex(&Sha256::digest(&bytes)),"depth":depth_probe(model,dev)?,"grounded":grounding}),
    )
}
fn ground_composite(
    model: &TlModel,
    prior: &lr::IntegerPrior,
    cases: &[Grounded],
    a: u8,
    g: u8,
    ratio: bool,
) -> Result<serde_json::Value, String> {
    let mut rs = Vec::new();
    let mut correct = 0;
    for c in cases {
        let m = model.content_feature(&c.sel, &c.res);
        let f = model.typed_block(&c.sel, &c.res, c.facts);
        let mut h = model.init_state(&m, &f);
        let mut history = Vec::<u32>::new();
        let mut actions = Vec::new();
        let mut cursor = 0;
        let mut ev = TL_EV_OBSERVE;
        for _ in 0..c.accepted.len() + 2 {
            let z = model.readout(&h, &m, &f, ev);
            let rows = model.legal_rows(cursor < c.sel.len());
            let scores = if history.len() >= 2
                && history[history.len() - 2..]
                    .iter()
                    .all(|v| (*v as usize) < VOCAB)
            {
                compose_selected(
                    prior,
                    ratio,
                    &z,
                    &prior.score(history[history.len() - 2], history[history.len() - 1])?,
                    model.score_shift,
                    a,
                    g,
                    &prior.math,
                )?
            } else {
                z
            };
            let r = pick(&scores, &rows);
            let action = if r < VOCAB {
                TlAction::Generate(r as u32)
            } else if r == model.copy_row() {
                TlAction::Copy
            } else {
                TlAction::Stop
            };
            actions.push(action);
            let token = match action {
                TlAction::Generate(v) => Some(v),
                TlAction::Copy => {
                    let v = c.sel[cursor];
                    cursor += 1;
                    Some(v)
                }
                TlAction::Stop => None,
            };
            if let Some(v) = token {
                history.push(v);
            } else {
                break;
            }
            ev = action.event();
            h = model.transition(&h, ev, token, &m, &f);
        }
        let exact = actions == c.accepted;
        correct += usize::from(exact);
        rs.push(serde_json::json!({"name":c.name,"exact":exact,"actions":actions.iter().map(action_name).collect::<Vec<_>>(),"tokens":history}));
    }
    Ok(serde_json::json!({"correct":correct,"total":cases.len(),"rows":rs}))
}
fn native_generations(
    model: &TlModel,
    dev: &[ProseWindow],
    tok: &HfBpeTokenizer,
) -> serde_json::Value {
    serde_json::json!(dev.iter().step_by(4).take(6).map(|w|{let prompt=&w.tokens[..PREFIX];let r=model.rollout(&[],&[],SlFacts::default(),prompt,&[],48,false,false);serde_json::json!({"doc":w.doc,"prompt":tok.decode(prompt),"generated":tok.decode(&r.tokens),"tokens":r.tokens,"stopped":r.stopped})}).collect::<Vec<_>>())
}
fn warm_run(
    root: &Path,
    model: &TlModel,
    fit: &[ProseWindow],
    dev: &[ProseWindow],
    tok: &HfBpeTokenizer,
) -> Result<serde_json::Value, String> {
    let words = read_words(tok)?;
    let (train, class, held) = authored_world(&words);
    let steps = 128usize;
    let mut order: Vec<usize> = (0..fit.len()).collect();
    let run_seed: u64 = std::env::var("UOR_PRINCIPAL_WARM_SEED")
        .unwrap_or_else(|_| "20260923".into())
        .parse()
        .map_err(|_| "invalid warm seed")?;
    let mut seed = run_seed;
    shuffle(&mut order, &mut seed);
    let initial = serde_json::json!({"prose_bits":evaluate_single(model,dev),"train":panel_summary(model,&train),"class":panel_summary(model,&class),"heldout":panel_summary(model,&held),"crossed_feedback":crossed_feedback(model,&words)});
    write_json(
        root,
        "warm-plan.json",
        &serde_json::json!({"warm_seed":run_seed,"steps_per_arm":steps,"prose_draws_per_step":4,"ground_draws_per_step":2,"ground_weight":4.0,"schedules":["constant 0.01","linear 0.02 to 0.002"],"initialization":"Exact served-weight expansion with fresh zero Adam moments; not a checkpoint resume or an imported refitted head","selection":"No dev-based winner promotion; both artifacts and all panels retained"}),
    )?;
    let mut result = Vec::new();
    for schedule in 0..2 {
        let timer = Instant::now();
        let mut tr = TlTrainer::from_model_fresh_optimizer(
            model,
            TlTrainConfig {
                lr: 0.01,
                seed: run_seed,
                ..TlTrainConfig::default()
            },
        )?;
        if tr.model()?.to_bytes() != model.to_bytes() {
            return Err("warm-start step-zero drift".into());
        }
        let mut curve = Vec::new();
        let (mut pseen, mut gseen) = (0usize, 0usize);
        for step in 0..steps {
            tr.tcfg.lr = if schedule == 0 {
                0.01
            } else {
                0.02 - 0.018 * step as f64 / (steps - 1) as f64
            };
            let mut batch = Vec::new();
            for j in 0..4 {
                batch.push(prose_example(&fit[order[(step * 4 + j) % order.len()]]));
                pseen += 1;
            }
            for j in 0..2 {
                batch.push(train[(step * 2 + j) % train.len()].example(4.0, 0));
                gseen += 1;
            }
            let loss = tr.train_batch(&batch);
            if step % 32 == 0 {
                curve.push(serde_json::json!({"step":step+1,"lr":tr.tcfg.lr,"batch_bits":loss.bits_per_target()}));
                println!(
                    "WARM schedule={schedule} step={}/{} seconds={:.3}",
                    step + 1,
                    steps,
                    timer.elapsed().as_secs_f64()
                );
            }
        }
        let candidate = tr.model()?;
        let bytes = candidate.to_bytes();
        let loaded = TlModel::from_bytes(&bytes)?;
        if loaded != candidate {
            return Err("trained artifact roundtrip mismatch".into());
        }
        let file = format!("warm-schedule-{schedule}.tlx");
        write_checked(root, &file, &bytes)?;
        let generations = native_generations(&loaded, dev, tok);
        write_json(
            root,
            &format!("warm-generation-{schedule}.json"),
            &generations,
        )?;
        result.push(serde_json::json!({"schedule":schedule,"steps":tr.step,"prose_examples":pseen,"ground_examples":gseen,"initialization_exact":true,"curve":curve,"prose_bits":evaluate_single(&loaded,dev),"train":panel_summary(&loaded,&train),"class":panel_summary(&loaded,&class),"heldout":panel_summary(&loaded,&held),"crossed_feedback":crossed_feedback(&loaded,&words),"artifact":file,"sha256":sha256_hex(&Sha256::digest(&bytes)),"seconds":timer.elapsed().as_secs_f64()}));
    }
    Ok(
        serde_json::json!({"warm_seed":run_seed,"initial":initial,"arms":result,"capability_scope":"128-update matched fresh-optimizer experiments, not convergence or a global optimizer limitation"}),
    )
}
fn fresh_run(
    root: &Path,
    model: &TlModel,
    native_prior: &lr::IntegerPrior,
    old: &[DocRec],
    est: &CountEstimate,
    l: (f64, f64),
    top: &[bool],
    tok: &HfBpeTokenizer,
) -> Result<serde_json::Value, String> {
    let path = std::env::var("UOR_PRINCIPAL_BUNDLE").map_err(|_| "frozen bundle path required")?;
    let source_root = Path::new(&path).parent().ok_or("bundle source root")?;
    let source_errors = verify(source_root).map_err(|e| e.to_string())?;
    if !source_errors.is_empty() {
        return Err(format!("frozen source report changed: {source_errors:?}"));
    }
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    let (loaded, prior, a, g, ratio) = load_bundle(&bytes)?;
    let native_only = std::env::var("UOR_PRINCIPAL_FRESH_NATIVE").as_deref() == Ok("1");
    let (loaded, prior, a, g, ratio) = if native_only {
        (model.clone(), native_prior.clone(), 0, 32, false)
    } else {
        (loaded, prior, a, g, ratio)
    };

    if loaded.to_bytes() != model.to_bytes() {
        return Err("fresh evaluation supplied a different native artifact".into());
    }
    let dir = std::env::var("UOR_PRINCIPAL_FRESH_DIR").map_err(|_| "fresh directory required")?;
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("txt"))
        .collect();
    files.sort();
    if files.is_empty() || files.len() > 32 {
        return Err("fresh source population outside 1..32 files".into());
    }
    let mut ws = Vec::new();
    let mut identities = Vec::new();
    for p in &files {
        let raw = std::fs::read(p).map_err(|e| e.to_string())?;
        let hash: [u8; 32] = Sha256::digest(&raw).into();
        if old.iter().any(|d| d.sha256 == hash) {
            return Err("fresh source duplicates old corpus".into());
        }
        let text = std::str::from_utf8(&raw).map_err(|e| e.to_string())?;
        let body = if let Some(i) = text.find("*** START OF THE PROJECT GUTENBERG") {
            let tail = &text[i..];
            tail.split_once('\n').map(|(_, b)| b).unwrap_or(tail)
        } else {
            text
        };
        let body = body
            .split("*** END OF THE PROJECT GUTENBERG")
            .next()
            .unwrap_or(body);
        let n = identities.len();
        let windows = prose_windows(&tok.encode(body), n);
        if windows.len() < 4 {
            return Err("fresh source too short".into());
        }
        let stride = (windows.len() / 16).max(1);
        ws.extend(windows.into_iter().step_by(stride).take(16));
        identities.push(serde_json::json!({"filename":p.file_name().and_then(|x|x.to_str()),"sha256":sha256_hex(&hash),"bytes":raw.len(),"selection":"up to16 evenly-spaced nonoverlapping64-token windows, excluding PG wrappers"}));
    }
    write_json(
        root,
        "fresh-identities.json",
        &serde_json::json!(&identities),
    )?;
    write_json(
        root,
        "generation.json",
        &generations(&loaded, &prior, &ws, tok, a, g, ratio)?,
    )?;
    let integer = eval_integer(&loaded, &prior, &ws, files.len(), est, l, a, g, ratio, top)?;
    let frozen_selection: serde_json::Value = serde_json::from_slice(
        &std::fs::read(
            Path::new(&path)
                .parent()
                .ok_or("bundle parent")?
                .join("selection.json"),
        )
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let ca = frozen_selection["calibrated_count_alpha_numerator"]
        .as_u64()
        .ok_or("missing frozen count coefficient")?;
    if ca > 64 {
        return Err("invalid frozen count coefficient".into());
    }
    let calibrated = eval_integer(
        &loaded,
        &prior,
        &ws,
        files.len(),
        est,
        l,
        ca as u8,
        0,
        false,
        top,
    )?;
    Ok(
        serde_json::json!({"native_only":native_only,"evaluated_native_sha256":sha256_hex(&Sha256::digest(model.to_bytes())),"frozen_bundle_path":path,"frozen_bundle_sha256":sha256_hex(&Sha256::digest(&bytes)),"sources":identities,"candidate":integer,"calibrated_count":calibrated,"calibrated_count_alpha_numerator":ca,"calibration_note":"Full-tune selected count exponent frozen before source loading; not selected on fresh text","scope":"Source-disjoint classical prose, no claim of broad domain transfer; old model may have encountered analogous public text in tokenizer source, not trained model weights"}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_core::native_geometric::learner::hamilton_transport as ht;
    #[test]
    fn factored_grid_equals_direct_normalization() {
        let a = [0.1f64.log2(), 0.2f64.log2(), 0.7f64.log2()];
        let c = [0.5f64.log2(), 0.4f64.log2(), 0.1f64.log2()];
        let s = grids(&a, &c, 1, 0.03);
        for (i, &v) in s.iter().enumerate() {
            let z: Vec<f64> = a
                .iter()
                .zip(c)
                .map(|(a, c)| ALPHA[i / 9] as f64 / 32. * c + (i % 9) as f64 / 32. * a)
                .collect();
            assert!((v - (lse(&z) - z[1] + 0.03)).abs() < 1e-12);
        }
    }
    #[test]
    fn hamilton_axis_attention_needs_relative_frames() {
        let v = [1, 2, 3, 4];
        assert_eq!(ht::relative(2, 2, v).unwrap(), v);
        assert_ne!(
            ht::relative(2, 4, v).unwrap(),
            ht::relative(4, 2, v).unwrap()
        );
    }
}

fn selected_coefficients() -> Result<(u8, u8, bool), String> {
    let path = std::env::var("UOR_PRINCIPAL_BUNDLE").map_err(|_| "frozen bundle path required")?;
    let dir = Path::new(&path).parent().ok_or("bundle parent missing")?;
    let errors = verify(dir).map_err(|e| e.to_string())?;
    if !errors.is_empty() {
        return Err(format!("source report verification {errors:?}"));
    }
    let b = std::fs::read(path).map_err(|e| e.to_string())?;
    let (_, _, a, g, ratio) = load_bundle(&b)?;
    Ok((a, g, ratio))
}
#[allow(clippy::too_many_arguments)]
fn grounded_transfer(
    root: &Path,
    model: &TlModel,
    prior: &lr::IntegerPrior,
    dev: &[ProseWindow],
    tok: &HfBpeTokenizer,
    est: &CountEstimate,
    l: (f64, f64),
    top: &[bool],
) -> Result<serde_json::Value, String> {
    let (a, g, ratio) = selected_coefficients()?;
    let bytes = bundle(model, prior, a, g, ratio)?;
    let (m, p, aa, gg, rr) = load_bundle(&bytes)?;
    if m != *model || p != *prior || aa != a || gg != g || rr != ratio {
        return Err("grounded bundle roundtrip mismatch".into());
    }
    write_checked(root, "grounded-candidate.prc", &bytes)?;
    let parent = std::env::var("UOR_PRINCIPAL_BUNDLE").map_err(|_| "source bundle required")?;
    let selection = std::fs::read(
        Path::new(&parent)
            .parent()
            .ok_or("source parent")?
            .join("selection.json"),
    )
    .map_err(|e| e.to_string())?;
    write_checked(root, "selection.json", &selection)?;

    let words = read_words(tok)?;
    let (train, class, held) = authored_world(&words);
    let temporal: Vec<Grounded> = train
        .iter()
        .filter(|c| c.role == "preflight_temporal")
        .cloned()
        .collect();
    write_json(
        root,
        "generation.json",
        &generations(&m, &p, dev, tok, a, g, ratio)?,
    )?;
    Ok(
        serde_json::json!({"alpha":a,"gamma":g,"denominator":32,"selection":"Transferred unchanged from the full-Tune prose selection; not retuned on grounded panels","native_temporal":panel_summary(&m,&temporal),"composite_temporal":ground_composite(&m,&p,&temporal,a,g,ratio)?,"native_train":panel_summary(&m,&train),"composite_train":ground_composite(&m,&p,&train,a,g,ratio)?,"native_class":panel_summary(&m,&class),"composite_class":ground_composite(&m,&p,&class,a,g,ratio)?,"native_heldout":panel_summary(&m,&held),"composite_heldout":ground_composite(&m,&p,&held,a,g,ratio)?,"integer_prose":eval_integer(&m,&p,dev,24,est,l,a,g,ratio,top)?,"bundle_bytes":bytes.len(),"bundle_sha256":sha256_hex(&Sha256::digest(&bytes)),"scope":"Same loaded grounded model and readout; altered Generate ranking can change action choices despite preserved group mass. No automatic replacement."}),
    )
}
fn cost_probe(
    model: &TlModel,
    prior: &lr::IntegerPrior,
    ws: &[ProseWindow],
) -> Result<serde_json::Value, String> {
    let (a, g, ratio) = selected_coefficients()?;
    let m = vec![0; model.h_dim];
    let f = model.typed_block(&[], &[], SlFacts::default());
    let rows = model.legal_rows(false);
    let mut samples = Vec::new();
    for w in ws.iter().take(4) {
        let mut h = model.init_state(&m, &f);
        for (j, &token) in w.tokens.iter().enumerate() {
            if j >= 8 && j < 24 {
                samples.push((
                    h.clone(),
                    w.tokens[j - 2],
                    w.tokens[j - 1],
                    if j == 8 {
                        TL_EV_OBSERVE
                    } else {
                        TL_EV_GENERATE
                    },
                ));
            }
            h = model.transition(
                &h,
                if j < PREFIX {
                    TL_EV_OBSERVE
                } else {
                    TL_EV_GENERATE
                },
                Some(token),
                &m,
                &f,
            );
        }
    }
    let mut times = vec![Vec::new(), Vec::new()];
    let mut checksum = 0u64;
    for round in 0..9 {
        for turn in 0..2 {
            let which = (round + turn) % 2;
            let start = Instant::now();
            for (h, p, c, e) in &samples {
                let z = model.readout(std::hint::black_box(h), &m, &f, *e);
                let out = if which == 1 {
                    compose_selected(
                        prior,
                        ratio,
                        &z,
                        &prior.score(*p, *c)?,
                        model.score_shift,
                        a,
                        g,
                        &prior.math,
                    )?
                } else {
                    z
                };
                let selected = pick(&out, &rows);
                let action = if selected < VOCAB {
                    TlAction::Generate(selected as u32)
                } else {
                    TlAction::Stop
                };
                let token = if selected < VOCAB {
                    Some(selected as u32)
                } else {
                    None
                };
                let state = model.transition(h, action.event(), token, &m, &f);
                checksum = checksum.wrapping_add(std::hint::black_box(state[0]) as u64);
            }
            times[which].push(start.elapsed().as_secs_f64() * 1e6 / samples.len() as f64);
        }
    }
    let mut medians = Vec::new();
    for t in &times {
        let mut v = t.clone();
        v.sort_by(f64::total_cmp);
        medians.push(v[v.len() / 2]);
    }
    Ok(
        serde_json::json!({"conditions":["native_readout_plus_update","integer_prior_composite_plus_update"],"microseconds_per_step_rounds":times,"median_microseconds":medians,"context_count":samples.len(),"interleaved_rounds":9,"checksum":checksum,"scope":"Warm CPU microbenchmark of readout plus one update over stored contexts, excluding prompt ingestion and allocation setup; includes per-step allocations. Not whole-system energy or equal-quality incumbent comparison."}),
    )
}
#[cfg(test)]
#[test]
fn composite_bundle_roundtrips_and_rejects_invalid_binding() {
    let model = TlTrainer::new(TlConfig::new(8), TlTrainConfig::default())
        .unwrap()
        .model()
        .unwrap();
    let bytes = model.to_bytes();
    let binding: [u8; 32] = Sha256::digest(&bytes).into();
    let prior = lr::IntegerPrior::compile(8, &[(0, 1, 2), (0, 1, 3)], (0.7, 0.4), binding).unwrap();
    let b = bundle(&model, &prior, 32, 5, false).unwrap();
    let (m, p, a, g, ratio) = load_bundle(&b).unwrap();
    assert!(!ratio);
    assert_eq!(m, model);
    assert_eq!(p, prior);
    assert_eq!((a, g), (32, 5));
    assert!(load_bundle(&b[..b.len() - 1]).is_err());
    let mut foreign = b.clone();
    foreign[15 + bytes.len() + 8] ^= 1;
    assert!(load_bundle(&foreign).is_err());
    let mut invalid = b;
    invalid[12] = 255;
    assert!(load_bundle(&invalid).is_err());
}

fn score_recorded(
    model: &TlModel,
    dev: &[ProseWindow],
    names: &[String],
    top: &[bool],
    tok: &HfBpeTokenizer,
) -> Result<serde_json::Value, String> {
    let states = record_served_states(model, dev);
    let mut e = eval_new(names.len());
    let (mut head, mut tail) = ((0.0, 0usize), (0.0, 0usize));
    let mut clipped = 0usize;
    for s in &states {
        eval_add(&mut e, s.doc, s.bits);
        let r = if top[s.target as usize] {
            &mut head
        } else {
            &mut tail
        };
        r.0 += s.bits;
        r.1 += 1;
        clipped += s.h.iter().filter(|v| v.abs() >= model.h_clamp).count();
    }
    let independent = evaluate_single(model, dev);
    if (e.micro() - independent).abs() > 1e-10 {
        return Err("score recording and native scorer disagree".into());
    }
    let words = read_words(tok)?;
    let (train, class, held) = authored_world(&words);
    let temporal: Vec<Grounded> = train
        .iter()
        .filter(|x| x.role == "preflight_temporal")
        .cloned()
        .collect();
    Ok(
        serde_json::json!({"bits":e.micro(),"documents":e.per_doc.iter().enumerate().map(|(i,(bits,n))|serde_json::json!({"name":names[i],"bits_sum":bits,"targets":n})).collect::<Vec<_>>(),"head_bits":head.0/head.1 as f64,"head_n":head.1,"tail_bits":tail.0/tail.1 as f64,"tail_n":tail.1,"state_sha256":recorded_states_sha256(&states),"saturated_coordinates":clipped,"temporal":panel_summary(model,&temporal),"train":panel_summary(model,&train),"class":panel_summary(model,&class),"heldout":panel_summary(model,&held),"crossed_feedback":crossed_feedback(model,&words),"generated":native_generations(model,dev,tok)}),
    )
}

fn matched_count_calibration(
    est: &CountEstimate,
    tune: &[ProseWindow],
    dev: &[ProseWindow],
    legacy: (f64, f64),
) -> serde_json::Value {
    let mut candidates = Vec::new();
    let mut best = (f64::INFINITY, 0.0, 0.0);
    let n: usize = tune.iter().map(|w| w.tokens.len() - PREFIX).sum();
    for x in 0..10 {
        for y in 0..10 {
            let l = (x as f64 / 10.0, y as f64 / 10.0);
            let mut bits = 0.;
            for w in tune {
                for k in PREFIX..w.tokens.len() {
                    bits -= family_p(
                        &est.c1,
                        &est.c2,
                        &est.uni,
                        w.tokens[k - 2] as usize,
                        w.tokens[k - 1] as usize,
                        w.tokens[k],
                        l,
                    )
                    .log2();
                }
            }
            let mean = bits / n as f64;
            if mean < best.0 {
                best = (mean, l.0, l.1)
            }
            candidates.push(serde_json::json!({"lambda1":l.0,"lambda2":l.1,"tune_bits":mean}));
        }
    }
    let mut rows = Vec::new();
    for l in [legacy, (best.1, best.2)] {
        let mut eval = eval_new(24);
        for w in dev {
            for k in PREFIX..w.tokens.len() {
                eval_add(
                    &mut eval,
                    w.doc,
                    -family_p(
                        &est.c1,
                        &est.c2,
                        &est.uni,
                        w.tokens[k - 2] as usize,
                        w.tokens[k - 1] as usize,
                        w.tokens[k],
                        l,
                    )
                    .log2(),
                );
            }
        }
        rows.push(
            serde_json::json!({"lambdas":l,"dev_bits":eval.micro(),"documents":eval.per_doc}),
        );
    }
    serde_json::json!({"tune_targets":n,"selected_lambdas":[best.1,best.2],"legacy_lambdas":legacy,"equal_to_legacy":legacy==(best.1,best.2),"candidates":candidates,"evaluation":rows,"scope":"Interpolation selected on exactly the declared full served-conditioning Tune population; no development selection."})
}

#[allow(clippy::too_many_arguments)]
fn realize_selected(
    root: &Path,
    model: &TlModel,
    prior: &lr::IntegerPrior,
    dev: &[ProseWindow],
    names: &[String],
    est: &CountEstimate,
    l: (f64, f64),
    top: &[bool],
    tok: &HfBpeTokenizer,
) -> Result<serde_json::Value, String> {
    let path =
        std::env::var("UOR_PRINCIPAL_SELECTION").map_err(|_| "frozen selection path required")?;
    let dir = Path::new(&path).parent().ok_or("selection report parent")?;
    let errors = verify(dir).map_err(|e| e.to_string())?;
    if !errors.is_empty() {
        return Err(format!("frozen selection report modified {errors:?}"));
    }
    let sb = std::fs::read(&path).map_err(|e| e.to_string())?;
    let selected: serde_json::Value = serde_json::from_slice(&sb).map_err(|e| e.to_string())?;
    let prior_report: serde_json::Value = serde_json::from_slice(
        &std::fs::read(dir.join("receipt.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if prior_report["metadata"]["artifact_sha256"] != sha256_hex(&Sha256::digest(model.to_bytes()))
        || prior_report["metadata"]["count_lambdas"] != serde_json::json!([l.0, l.1])
    {
        return Err("frozen selection artifact or count interpolation mismatch".into());
    }
    let a = selected["alpha"].as_u64().ok_or("alpha")?;
    let g = selected["gamma"].as_u64().ok_or("gamma")?;
    if a > 64 || g > 32 {
        return Err("selected coefficients outside format bounds".into());
    }
    let (a, g) = (a as u8, g as u8);
    let ratio = selected["likelihood_ratio_to_fit_unigram"]
        .as_bool()
        .ok_or("correction frame missing")?;
    let bytes = bundle(model, prior, a, g, ratio)?;
    let (m, p, aa, gg, rr) = load_bundle(&bytes)?;
    if m != *model || p != *prior || aa != a || gg != g || rr != ratio {
        return Err("PRC2 reload mismatch".into());
    }
    write_checked(root, "selection.json", &sb)?;
    write_checked(root, "candidate.prc", &bytes)?;
    let eval = eval_integer(&m, &p, dev, names.len(), est, l, a, g, ratio, top)?;
    let expected = prior_report["result"]["comparisons"][0]["bits"]
        .as_f64()
        .ok_or("source ideal loss missing")?;
    if (eval["ideal_bits"].as_f64().ok_or("ideal")? - expected).abs() > 1e-9 {
        return Err("independent float realization differs from frozen grid".into());
    }
    write_json(
        root,
        "generation.json",
        &generations(&m, &p, dev, tok, a, g, ratio)?,
    )?;
    Ok(
        serde_json::json!({"alpha":a,"gamma":g,"relative_to_fit_unigram":ratio,"denominator":32,"frozen_selection_source":path,"selection_sha256":sha256_hex(&Sha256::digest(&sb)),"bundle_sha256":sha256_hex(&Sha256::digest(&bytes)),"bundle_bytes":bytes.len(),"integer":eval,"scope":"Frozen selected function implemented in integer lookup arithmetic; no new fit or source-disjoint selection"}),
    )
}
#[cfg(test)]
#[test]
fn prc2_records_ratio_and_reads_prc1_without_changing_semantics() {
    let m = TlTrainer::new(TlConfig::new(8), TlTrainConfig::default())
        .unwrap()
        .model()
        .unwrap();
    let hash: [u8; 32] = Sha256::digest(m.to_bytes()).into();
    let p = lr::IntegerPrior::compile(8, &[(0, 1, 2)], (0.7, 0.4), hash).unwrap();
    let b = bundle(&m, &p, 28, 8, true).unwrap();
    assert!(load_bundle(&b).unwrap().4);
    let mut bad = b.clone();
    bad[14] = 2;
    assert!(load_bundle(&bad).is_err());
    let mut old = bundle(&m, &p, 32, 5, false).unwrap();
    old[..4].copy_from_slice(b"PRC1");
    old.remove(14);
    let (mm, pp, a, g, r) = load_bundle(&old).unwrap();
    assert_eq!(mm, m);
    assert_eq!(pp, p);
    assert_eq!((a, g, r), (32, 5, false));
}
