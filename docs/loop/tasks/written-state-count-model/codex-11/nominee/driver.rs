use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use uor_r4_integer::discrete_d11::{spiralcore_orbit_roots, IntegerDiscreteModel};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const X: u32 = u32::MAX;
const W: usize = 192;
fn hash(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn load(p: &str) -> Result<(Value, String)> {
    let b = fs::read(p)?;
    Ok((serde_json::from_slice(&b)?, hash(&b)))
}
fn save(p: impl AsRef<Path>, v: &Value) -> Result<()> {
    let mut f = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(p)?;
    serde_json::to_writer(&mut f, v)?;
    f.write_all(b"\n")?;
    Ok(())
}
fn words(v: &Value) -> Result<Vec<u32>> {
    Ok(serde_json::from_value(v.clone())?)
}
fn idx(v: &Value) -> u32 {
    v.as_u64().and_then(|n| u32::try_from(n).ok()).unwrap_or(X)
}
fn opt(x: u32) -> Option<u32> {
    (x != X).then_some(x)
}
fn array(v: &Value) -> Result<&Vec<Value>> {
    Ok(v.as_array().ok_or("expected array")?)
}
fn packet(exe: &str, words: &[u32], width: usize, n: usize) -> Result<Vec<Vec<u32>>> {
    let b: Vec<_> = words.iter().flat_map(|i| i.to_le_bytes()).collect();
    let mut child = Command::new(exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("stdin")?.write_all(&b)?;
    let out = child.wait_with_output()?;
    if !out.status.success() || out.stdout.len() != width * n * 4 {
        return Err("device protocol".into());
    }
    Ok(out
        .stdout
        .chunks_exact(4 * width)
        .map(|r| {
            r.chunks_exact(4)
                .map(|u| u32::from_le_bytes(u.try_into().unwrap()))
                .collect()
        })
        .collect())
}
fn native_metric() -> Vec<u32> {
    let a = spiralcore_orbit_roots();
    let mut m = vec![];
    for x in a {
        for y in a {
            m.push(
                x.iter()
                    .zip(y)
                    .map(|(i, j)| u32::from(i.abs_diff(*j)))
                    .sum(),
            )
        }
    }
    m
}
fn full_query(g: &Value, vocab: &Value) -> Result<u32> {
    if g.get("query_start").is_some() {
        return Ok(idx(&g["query_start"]));
    }
    if let Some(x) = g["token_positions"].get("query_start") {
        return Ok(idx(x));
    }
    let name = match g["kind"].as_str() {
        Some("absent") => g["absent_name"].as_str(),
        Some("positive") => g["role"].as_str(),
        _ => return Ok(X),
    }
    .ok_or("missing oracle name")?;
    let id = idx(&vocab[format!("Ġ{name}")]);
    if id == X {
        return Err("mixed oracle requires an exact one-token name".into());
    }
    words(&g["prefix"])?
        .iter()
        .rposition(|&t| t == id)
        .map(|n| n as u32)
        .ok_or_else(|| "missing oracle word".into())
}
fn check_model(id: &str, role: &str, span: &str, tok: &str, gate: &str) -> bool {
    [
        (
            "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d",
            "1d98243a79f9b68537add8445397316773a028260c39997fa9d9b1dc04dec48e",
            "3451b35c018687cd4ae37f80f306ffe2af3724eb4969d2f7271828cf1c67f870",
            "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377",
            "7a74e9c03219f4027891ee324c09e11fcc805c6e73ac5d89aa6b7e20925e3530",
        ),
        (
            "1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de",
            "32ac74888a382198f737c98d1c67d8352a93ead9d884a98e15f2768e6e86457b",
            "090c53c07a3c16e7d017c931b02e6749bca98affe71c5f8ca19e89f55d57436f",
            "f804ecbee34e3c4e0001a737d003e3d620ba2c40722d416e7f3dd64843e2b30e",
            "10bdc94da4abe4722193f69e497a784aa616d54cb0839ff21571483b067e0e0e",
        ),
    ]
    .contains(&(id, role, span, tok, gate))
}
fn run(mode: &str, a: &[String]) -> Result<()> {
    let [exe, model, span, role, token, gate, tape, handoff, extra, oracle, out] = a else {
        return Err("unexpected inputs".into());
    };
    if !matches!(mode, "fit" | "blind" | "oracle") || (mode != "oracle" && oracle != "-") {
        return Err("invalid label boundary".into());
    }
    let m = IntegerDiscreteModel::load(Path::new(model))?;
    let id = m.artifact_sha256();
    let (s, sh) = load(span)?;
    let (r, rh) = load(role)?;
    let (t, th) = load(token)?;
    let (g, gh) = load(gate)?;
    let (b, bh) = load(tape)?;
    let (h, hh) = load(handoff)?;
    let (extra, xh) = load(extra)?;
    if !check_model(id, &rh, &sh, &th, &gh)
        || h["arm"] != "role"
        || h["bare_sha256"] != bh
        || h["artifact_sha256"] != id
        || t["stack_sha256"] != id
    {
        return Err("seed, artifact or handoff binding".into());
    }
    let n = array(&b)?.len();
    if n == 0 || n > 8192 || array(&h["rows"])?.len() != n {
        return Err("invalid row counts".into());
    }
    let mut delimiter = vec![0; 4096];
    let mut upper = vec![0; 4096];
    let vocab = &t["model"]["vocab"];
    for (w, v) in vocab.as_object().ok_or("token vocab")? {
        let i = idx(v) as usize;
        if i >= 4096 {
            return Err("vocab ID outside geometry".into());
        }
        upper[i] = u32::from(
            w.trim_start_matches('Ġ')
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_uppercase),
        );
    }
    let grade = if mode == "fit" {
        Some(extra.clone())
    } else if mode == "oracle" {
        Some(load(oracle)?.0)
    } else {
        None
    };
    if grade
        .as_ref()
        .is_some_and(|g| g.as_array().map(Vec::len) != Some(n))
    {
        return Err("grade row counts".into());
    }
    let mut rows = vec![];
    let mut prefixes = vec![];
    for i in 0..n {
        if b[i].as_object().map(|a| a.len()) != Some(1) {
            return Err("blind input contains additional fields".into());
        }
        let ids = words(&b[i]["prefix"])?;
        let v = &h["rows"][i];
        if ids.is_empty()
            || ids.len() > W
            || ids[0] != 0
            || ids.iter().skip(1).any(|&u| !(2..4096).contains(&u))
        {
            return Err("token tape bounds".into());
        }
        let sha = hash(&ids.iter().flat_map(|i| i.to_le_bytes()).collect::<Vec<_>>());
        if v["prefix_sha256"] != sha {
            return Err("handoff prefix mismatch".into());
        }
        prefixes.push(sha);
        let mut row = [0u32; 584];
        row[..ids.len()].copy_from_slice(&ids);
        row[576] = ids.len() as u32;
        row[577] = X;
        row[579] = idx(&v["frozen"]["query"]);
        row[580] = idx(&v["continuation"]["source"]);
        if let Some(grade) = &grade {
            if grade[i]["prefix"] != b[i]["prefix"] {
                return Err("grade tape mismatch".into());
            }
            row[577] = full_query(&grade[i], vocab)?;
            if row[577] != X && row[577] as usize >= ids.len() {
                return Err("query bounds".into());
            }
            if mode == "fit" {
                for support in array(&grade[i]["supports"])? {
                    let f = idx(&support["fact_start"]) as usize;
                    if f == 0 || f >= ids.len() {
                        return Err("invalid TRAIN fact".into());
                    }
                    row[W + 2 * f] = 1;
                }
            }
        }
        rows.extend(row);
    }
    let mut ends = vec![];
    let mut role_words = vec![];
    for group in array(&r["groups"])? {
        for site in array(group)? {
            let v = words(site)?;
            if v.len() != 6 {
                return Err("bad role descriptor".into());
            }
            role_words.extend(v)
        }
        ends.push((role_words.len() / 6) as u32)
    }
    let mut allow = vec![];
    for k in ["positive", "negative"] {
        for key in array(&g[k])? {
            let v = words(key)?;
            if v.len() != 14 {
                return Err("bad action descriptor".into());
            }
            allow.extend(v)
        }
    }
    let mut pos = vec![];
    let mut neg = vec![];
    if mode != "fit" {
        if extra["schema"] != "uor-r4.requested-word/1"
            || extra["model"] != id
            || extra["role"] != rh
            || extra["span"] != sh
            || extra["token"] != th
        {
            return Err("incompatible fitted state".into());
        }
        for d in words(&extra["delimiters"])? {
            *delimiter.get_mut(d as usize).ok_or("bad delimiter")? = 1
        }
        for (k, bank) in [("positive", &mut pos), ("negative", &mut neg)] {
            for p in array(&extra[k])? {
                let v = words(p)?;
                if v.len() != 8 {
                    return Err("bad nominee descriptor".into());
                }
                bank.extend(v)
            }
        }
    }
    let cls = words(&s["bank"]["classes"])?;
    let mut stream = vec![
        u32::from(mode != "fit"),
        n as u32,
        role_words.len() as u32 / 6,
        pos.len() as u32 / 8,
        (pos.len() + neg.len()) as u32 / 8,
        array(&g["positive"])?.len() as u32,
        allow.len() as u32 / 14,
    ];
    stream.extend(m.packed_codebook_roots());
    stream.extend(cls);
    stream.extend(native_metric());
    stream.extend(ends);
    stream.extend(&upper);
    let dstart = stream.len();
    stream.extend(&delimiter);
    stream.extend(role_words);
    stream.extend(pos);
    stream.extend(neg);
    stream.extend(allow);
    stream.extend(rows);
    if mode == "fit" {
        let first = packet(exe, &stream, 1920, n)?;
        let ds: BTreeSet<_> = first
            .iter()
            .flatten()
            .collect::<Vec<_>>()
            .chunks_exact(10)
            .filter_map(|w| opt(*w[9]))
            .collect();
        if ds.is_empty() {
            return Err("no TRAIN fact delimiters".into());
        }
        for &d in &ds {
            stream[dstart + d as usize] = 1;
        }
        let second = packet(exe, &stream, 1920, n)?;
        let (mut positive, mut negative) = (BTreeSet::new(), BTreeSet::new());
        for row in &second {
            for x in row.chunks_exact(10) {
                if x[8] == 1 {
                    positive.insert(x[..8].to_vec());
                } else if x[8] == 0 {
                    negative.insert(x[..8].to_vec());
                }
            }
        }
        if positive.is_empty() || negative.is_empty() {
            return Err("empty bank".into());
        }
        return save(
            out,
            &json!({"schema":"uor-r4.requested-word/1","model":id,"role":rh,"span":sh,"token":th,"train_bare":bh,"train_grade":xh,"train_role":hh,"delimiters":ds,"positive":positive,"negative":negative}),
        );
    }
    let result = packet(exe, &stream, 18, n)?;
    if mode == "blind" {
        save(
            Path::new(out).join("words.json"),
            &json!({"schema":"uor-r4.requested-words/1","model":id,"bank":xh,"bare":bh,"rows":result}),
        )?;
    }
    let arms = if mode == "blind" {
        vec!["nominee", "support", "both"]
    } else {
        vec!["query"]
    };
    for arm in arms {
        let mut hs = vec![];
        let mut trace = vec![];
        for (i, v) in result.iter().enumerate() {
            let at = match arm {
                "support" => 0,
                "query" => 8,
                _ => 4,
            };
            let q = opt(v[at]);
            let mut f = opt(v[at + 1]);
            let count = v[at + 2];
            let allowed = if arm == "nominee" {
                v[14] == 1
            } else {
                count == 1
            };
            if !allowed {
                f = None
            }
            let bound_q = if arm == "nominee" && allowed {
                opt(idx(&h["rows"][i]["frozen"]["query"]))
            } else {
                f.and(q)
            };
            let bound_f = if arm == "nominee" && allowed {
                opt(idx(&h["rows"][i]["frozen"]["fact"]))
            } else {
                f
            };
            hs.push(json!({"q":bound_q,"f":bound_f,"boundary":null,"forced":null,"prefix_sha256":prefixes[i]}));
            trace.push(json!({"prefix_sha256":prefixes[i],"nominee":q,"support":opt(v[at+1]),"count":count,"all_earlier":v[at+3],"forwarded_fact":bound_f,"allowed":allowed,"copy_distance":opt(v[12]),"generate_distance":opt(v[13])}));
        }
        save(
            Path::new(out).join(format!("{arm}-override.json")),
            &json!({"arm":"role","bare":bh,"artifact":id,"state":xh,"rows":hs}),
        )?;
        save(
            Path::new(out).join(format!("{arm}-diagnostic.json")),
            &json!({"arm":arm,"bare":bh,"bank":xh,"artifact":id,"rows":trace}),
        )?;
    }
    Ok(())
}
fn main() -> Result<()> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    if let [mode, exe] = a.as_slice() {
        if mode == "fixture" {
            return fixture(exe);
        }
    }
    run(a.first().ok_or("missing command")?, &a[1..])
}
// This packet exercises extraction and reloaded native context words on a tiny invented alphabet.
fn fixture(exe: &str) -> Result<()> {
    let roots: Vec<u32> = (0..4096).collect();
    let mut cls = vec![0; 4096];
    cls[5] = 1;
    cls[6] = 1;
    let mut uppers = vec![0; 4096];
    uppers[5] = 1;
    let mut delim = vec![0; 4096];
    delim[8] = 1;
    let examples = [
        (vec![0, 8, 5, 9, 3, 5, 4], 5, 1),
        (vec![0, 8, 6, 9, 3, 5, 4], 5, 0),
        (vec![0, 8, 5, 8, 5, 9, 3, 5, 4], 7, 2),
    ];
    let mut raw = vec![];
    for (t, q, _) in &examples {
        let mut row = [0; 584];
        row[..t.len()].copy_from_slice(t);
        row[576] = t.len() as u32;
        row[577] = *q;
        row[579] = *q;
        row[580] = 3;
        row[W + 4] = 1;
        raw.extend(row);
    }
    let mut tables = roots.clone();
    tables.extend(cls);
    tables.extend(native_metric());
    tables.extend([1, 2, 2, 2, 2]);
    tables.extend(uppers);
    tables.extend(delim);
    tables.extend([0; 12]);
    let mut req = vec![0, 3, 2, 0, 0, 0, 0];
    req.extend(&tables);
    req.extend(&raw);
    let fit = packet(exe, &req, 1920, examples.len())?;
    if fit[0][5 * 10 + 8] != 1 || fit[0][2 * 10 + 8] != 0 || fit[0][2 * 10 + 9] != 8 {
        return Err("finite fit labels".into());
    }
    let banks = vec![fit[0][50..58].to_vec(), fit[0][20..28].to_vec()];
    let reloaded: Vec<Vec<u32>> = serde_json::from_slice(&serde_json::to_vec(&banks)?)?;
    if banks != reloaded {
        return Err("finite reload".into());
    }
    let mut req = vec![1, 3, 2, 1, 2, 0, 0];
    req.extend(tables);
    req.extend(reloaded.into_iter().flatten());
    req.extend(raw);
    let result = packet(exe, &req, 18, examples.len())?;
    for (r, (_, q, count)) in result.iter().zip(&examples) {
        if r[4] != *q || r[8] != *q || r[6] != *count || r[10] != *count {
            return Err(format!("finite support or nominee mismatch {r:?}").into());
        }
    }
    println!("PASS synthetic two candidate words, missing, singular, repeated support; TRAIN extraction, raw integer serialize/reload, CUDA/host equality (all fields)");
    Ok(())
}
