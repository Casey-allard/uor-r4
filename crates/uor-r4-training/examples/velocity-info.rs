//! Diagnostic (pivot card D2, #2032): how much does each recurrence lane's
//! per-token *turn* r_t (the unit quaternion the token applies to the state),
//! its angle, its keep factor lambda_t, and the state h_t itself ("position")
//! say about the next token?
//!
//! For every recurrence layer and lane it replays the scan on the host from
//! the captured normalized input (checked against the captured core output),
//! classes r_t and h_t by their nearest of the 120 unit icosians (h_t by
//! direction), bins the angle and lambda into 16 levels, and reports the
//! plug-in mutual information with the next token minus the same estimate
//! with the next tokens shuffled (bias control), in bits.
//!
//! Usage: velocity-info model=DIR tokens=FILE.u16 [blocks=256] [context=256]
use std::collections::HashMap;
use std::path::PathBuf;

use candle_core::{DType, Device, Tensor};
use uor_r4_training::geometric_stack::{StackModel, StackSite};

type Res<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn icosians() -> Vec<[f32; 4]> {
    let phi = (1.0 + 5f64.sqrt()) / 2.0;
    let mut out: Vec<[f64; 4]> = Vec::new();
    for i in 0..4 {
        for s in [1.0, -1.0] {
            let mut v = [0.0; 4];
            v[i] = s;
            out.push(v);
        }
    }
    for m in 0..16 {
        out.push(std::array::from_fn(|i| {
            if m >> i & 1 == 1 {
                -0.5
            } else {
                0.5
            }
        }));
    }
    let base = [phi / 2.0, 0.5, 1.0 / (2.0 * phi), 0.0];
    let even: Vec<[usize; 4]> = permutations()
        .into_iter()
        .filter(|p| {
            let mut inv = 0;
            for i in 0..4 {
                for j in i + 1..4 {
                    if p[i] > p[j] {
                        inv += 1;
                    }
                }
            }
            inv % 2 == 0
        })
        .collect();
    for p in even {
        for m in 0..8 {
            let mut v = [0.0; 4];
            for k in 0..3 {
                v[p[k]] = if m >> k & 1 == 1 { -base[k] } else { base[k] };
            }
            v[p[3]] = 0.0;
            out.push(v);
        }
    }
    assert_eq!(out.len(), 120);
    out.iter().map(|v| v.map(|x| x as f32)).collect()
}

fn permutations() -> Vec<[usize; 4]> {
    let mut out = Vec::new();
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                for d in 0..4 {
                    let p = [a, b, c, d];
                    let mut seen = [false; 4];
                    if p.iter().all(|&x| !std::mem::replace(&mut seen[x], true)) {
                        out.push(p);
                    }
                }
            }
        }
    }
    out
}

fn nearest(v: [f32; 4], roots: &[[f32; 4]]) -> usize {
    let mut best = (f32::NEG_INFINITY, 0);
    for (i, r) in roots.iter().enumerate() {
        let d = (0..4).map(|c| v[c] * r[c]).sum::<f32>();
        if d > best.0 {
            best = (d, i);
        }
    }
    best.1
}

fn product(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
        a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
    ]
}

fn gelu(x: f32) -> f32 {
    0.5 * x * (1.0 + (0.797_884_6 * (x + 0.044_715 * x * x * x)).tanh())
}

/// Plug-in mutual information (bits) between classes and next tokens.
fn mutual_information(classes: &[u32], next: &[u32]) -> f64 {
    let n = classes.len() as f64;
    let mut joint: HashMap<(u32, u32), f64> = HashMap::new();
    let mut a: HashMap<u32, f64> = HashMap::new();
    let mut b: HashMap<u32, f64> = HashMap::new();
    for (&c, &t) in classes.iter().zip(next) {
        *joint.entry((c, t)).or_default() += 1.0;
        *a.entry(c).or_default() += 1.0;
        *b.entry(t).or_default() += 1.0;
    }
    joint
        .iter()
        .map(|(&(c, t), &k)| k / n * ((k * n) / (a[&c] * b[&t])).log2())
        .sum()
}

/// MI minus the shuffled-next baseline (same marginals, no dependence).
fn corrected(classes: &[u32], next: &[u32], shuffled: &[u32]) -> f64 {
    mutual_information(classes, next) - mutual_information(classes, shuffled)
}

fn main() -> Res<()> {
    let mut args: HashMap<String, String> = HashMap::new();
    for a in std::env::args().skip(1) {
        let (k, v) = a.split_once('=').ok_or("arguments are key=value")?;
        args.insert(k.into(), v.into());
    }
    let model_dir = PathBuf::from(args.get("model").ok_or("model=")?);
    let blocks: usize = args.get("blocks").map_or(Ok(256), |v| v.parse())?;
    let context: usize = args.get("context").map_or(Ok(256), |v| v.parse())?;
    let path = PathBuf::from(args.get("tokens").ok_or("tokens=")?);
    let reader = uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader::open(&path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let tokens: Vec<u32> = reader.as_slice().iter().map(|&id| u32::from(id)).collect();
    let device = Device::Cpu;
    let model = StackModel::load(&model_dir, &device)?;
    let config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(model_dir.join("config.json"))?)?;
    let pattern = config["pattern"].as_str().ok_or("pattern")?.to_owned();
    let width = config["width"].as_u64().ok_or("width")? as usize;
    let lanes = width / 4;
    let vars = model.variables();
    let get = |name: &str| -> Res<Vec<f32>> {
        Ok(vars
            .get(name)
            .ok_or_else(|| format!("missing {name}"))?
            .as_tensor()
            .to_dtype(DType::F32)?
            .flatten_all()?
            .to_vec1::<f32>()?)
    };
    let roots = icosians();
    let rec_layers: Vec<usize> = pattern
        .chars()
        .enumerate()
        .filter(|(_, c)| *c == 'r')
        .map(|(i, _)| i)
        .collect();
    // Per (layer, lane, feature): the class of every scored position.
    let features = ["turn_r", "state_h", "angle", "lambda", "token120"];
    let mut classes: HashMap<(usize, usize, usize), Vec<u32>> = HashMap::new();
    let mut next_tokens: Vec<u32> = Vec::new();
    let mut max_gap = 0f32;
    let batch = 8;
    let usable = (tokens.len() - 1) / context;
    let blocks = blocks.min(usable) / batch * batch;
    for chunk in 0..blocks / batch {
        let start = chunk * batch * context;
        let ids: Vec<u32> = tokens[start..start + batch * context].to_vec();
        let mut captured: HashMap<(bool, usize), Tensor> = HashMap::new();
        model.hidden_with_capture(&ids, batch, context, &mut |site, t| {
            match site {
                StackSite::Recurrence(l) => {
                    captured.insert((false, l), t.clone());
                }
                StackSite::RecurrenceOut(l) => {
                    captured.insert((true, l), t.clone());
                }
                _ => {}
            }
            Ok(())
        })?;
        for b in 0..batch {
            for t in 0..context {
                let i = start + b * context + t + 1;
                next_tokens.push(tokens[i]);
            }
        }
        for &l in &rec_layers {
            let name = |s: &str| format!("layers.{l:02}.{s}");
            let gain = get(&name("rec_norm.weight"))?;
            let w_in = get(&name("rec.in.weight"))?; // [2w, w]
            let w_gate = get(&name("rec.gate.weight"))?; // [lanes + w, w]
            let b_gate = get(&name("rec.gate.bias"))?;
            let taps = get(&name("rec.conv.weight"))?; // [4, w]
            let conv_bias = get(&name("rec.conv.bias"))?;
            let decay = get(&name("rec.decay"))?;
            let log_a: Vec<f32> = decay
                .iter()
                .map(|&d| -((-f64::from(d)).exp().ln_1p()) as f32)
                .collect();
            let xhat = captured[&(false, l)]
                .to_dtype(DType::F32)?
                .to_vec2::<f32>()?;
            let out = captured[&(true, l)]
                .to_dtype(DType::F32)?
                .to_vec2::<f32>()?;
            let gate_width = lanes + width;
            for b in 0..batch {
                let mut state = vec![[0f32; 4]; lanes];
                let mut drive_in: Vec<Vec<f32>> = Vec::new();
                for t in 0..context {
                    let row = &xhat[b * context + t];
                    let u: Vec<f32> = row.iter().zip(&gain).map(|(x, g)| x * g).collect();
                    let lin = |w: &[f32], r: usize| -> f32 {
                        w[r * width..(r + 1) * width]
                            .iter()
                            .zip(&u)
                            .map(|(a, b)| a * b)
                            .sum()
                    };
                    let branches: Vec<f32> = (0..2 * width).map(|r| lin(&w_in, r)).collect();
                    drive_in.push(branches[..width].to_vec());
                    let gates: Vec<f32> = (0..gate_width)
                        .map(|r| lin(&w_gate, r) + b_gate[r])
                        .collect();
                    let mut c = conv_bias.clone();
                    for shift in 0..4.min(t + 1) {
                        let a = &drive_in[t - shift];
                        for k in 0..width {
                            c[k] += taps[shift * width + k] * a[k];
                        }
                    }
                    let pos = (chunk * batch + b) * context + t;
                    for lane in 0..lanes {
                        let opening = 1.0 / (1.0 + (-gates[lane]).exp());
                        let lambda = (8.0 * opening * log_a[lane]).exp();
                        let complement = 1.0 - lambda * lambda;
                        let keep = if complement < 1e-6 {
                            1e-3
                        } else {
                            complement.sqrt()
                        };
                        let raw: [f32; 4] = std::array::from_fn(|k| gates[lanes + 4 * lane + k]);
                        let n = (raw.iter().map(|v| v * v).sum::<f32>() + 1e-6).sqrt();
                        let r = raw.map(|v| v / n);
                        let q = r.map(|v| v * lambda);
                        let moved = product(q, state[lane]);
                        let h: [f32; 4] =
                            std::array::from_fn(|k| moved[k] + keep * c[4 * lane + k]);
                        state[lane] = h;
                        // Check the replay against the captured core output.
                        for k in 0..4 {
                            let g = branches[width + 4 * lane + k];
                            let mine = h[k] * gelu(g);
                            let theirs = out[b * context + t][4 * lane + k];
                            max_gap = max_gap.max((mine - theirs).abs());
                        }
                        let hn = (h.iter().map(|v| v * v).sum::<f32>() + 1e-12).sqrt();
                        let turn = nearest(r, &roots) as u32;
                        let place = nearest(h.map(|v| v / hn), &roots) as u32;
                        let angle = 2.0 * r[0].abs().min(1.0).acos(); // 0..pi
                        let angle_bin = ((angle / std::f32::consts::PI) * 16.0).min(15.0) as u32;
                        let lambda_bin = (lambda * 16.0).min(15.0) as u32;
                        // Control: the current token itself, hashed to 120 classes
                        // (same class count as the turn).
                        let current = tokens[start + b * context + t];
                        let token_class = (current.wrapping_mul(2_654_435_761) >> 7) % 120;
                        let values = [turn, place, angle_bin, lambda_bin, token_class];
                        for (f, v) in values.into_iter().enumerate() {
                            let entry = classes.entry((l, lane, f)).or_default();
                            if entry.len() <= pos {
                                entry.resize(pos + 1, 0);
                            }
                            entry[pos] = v;
                        }
                    }
                }
            }
        }
    }
    // Shuffled baseline: a fixed permutation of the next tokens.
    let n = next_tokens.len();
    let mut shuffled = next_tokens.clone();
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    for i in (1..n).rev() {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        shuffled.swap(i, (seed % (i as u64 + 1)) as usize);
    }
    println!(
        "{{\"positions\": {n}, \"replay_max_abs_gap\": {max_gap}, \"layers\": {rec_layers:?}, \"lanes\": {lanes}}}"
    );
    for &l in &rec_layers {
        let mut line = format!("layer {l}:");
        for (f, name) in features.iter().enumerate() {
            let per_lane: Vec<f64> = (0..lanes)
                .map(|lane| corrected(&classes[&(l, lane, f)], &next_tokens, &shuffled))
                .collect();
            let mean = per_lane.iter().sum::<f64>() / lanes as f64;
            let best = per_lane.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            line += &format!(" {name} mean {mean:.4} best {best:.4} |");
        }
        println!("{line}");
    }
    Ok(())
}
