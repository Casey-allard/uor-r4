use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
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
fn row_hash(ids: &[u32]) -> String {
    hash(&ids.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>())
}
fn row_check(n: usize, p: &Value, g: &Value, b: &str) -> Result<()> {
    if (g["bare_sha256"] != b && g["bare"] != b) || array(&g["rows"])?.len() != n {
        return Err("trace/prefix binding".into());
    }
    for i in 0..n {
        if g["rows"][i]["prefix_sha256"] != row_hash(&words(&p[i]["prefix"])?) {
            return Err("row binding".into());
        }
    }
    Ok(())
}
fn enumerate(a: &[String]) -> Result<()> {
    let [exe, span, bank, tape, nominee, out] = a else {
        return Err("enumerate dependencies".into());
    };
    let (s, sh) = load(span)?;
    let (b, bh) = load(bank)?;
    let (t, th) = load(tape)?;
    let (n, nh) = load(nominee)?;
    if n["artifact_sha256"] != b["model"]
        || n["state_sha256"] != bh
        || b["span"] != sh
        || !matches!(n["arm"].as_str(), Some("both" | "query"))
    {
        return Err("nominee or frozen state identity".into());
    }
    let ct = array(&t)?.len();
    row_check(ct, &t, &n, &th)?;
    let mut req = vec![0, ct as u32];
    req.extend(words(&s["bank"]["classes"])?);
    let mut ds = vec![0; 4096];
    for i in words(&b["delimiters"])? {
        *ds.get_mut(i as usize).ok_or("delimiter ID")? = 1
    }
    req.extend(ds);
    for i in 0..ct {
        let mut r = [0; 584];
        let ids = words(&t[i]["prefix"])?;
        if ids.len() > W {
            return Err("tape too long".into());
        }
        r[..ids.len()].copy_from_slice(&ids);
        r[576] = ids.len() as u32;
        r[577] = idx(&n["rows"][i]["nominee"]);
        req.extend(r);
    }
    let got = packet(exe, &req, W, ct)?;
    let sites: Vec<Vec<u32>> = got
        .iter()
        .map(|r| r.iter().copied().take_while(|&u| u != X).collect())
        .collect();
    let max = sites.iter().map(Vec::len).max().ok_or("empty panel")?;
    for (i, s) in sites.iter().enumerate() {
        if Some(s.len() as u64) != n["rows"][i]["count"].as_u64() {
            return Err("alternate addresses disagree with primary count".into());
        }
    }
    for rank in 0..max {
        let rows:Vec<_>=sites.iter().enumerate().map(|(i,f)|{let fact=f.get(rank);let q=fact.map(|_|idx(&n["rows"][i]["nominee"]));json!({"q":q,"f":fact,"boundary":null,"forced":null,"prefix_sha256":n["rows"][i]["prefix_sha256"]})}).collect();
        save(
            Path::new(out).join(format!("rank-{rank}-override.json")),
            &json!({"arm":"role","artifact":b["model"],"bare":th,"state":bh,"rows":rows}),
        )?;
    }
    save(
        Path::new(out).join("sites.json"),
        &json!({"sites":sites,"max":max,"bare":th,"nominee":nh,"artifact":b["model"],"bank":bh}),
    )
}
fn pack_run(source: &Value, w: &Value) -> Result<Vec<u32>> {
    if &w["source"] != source {
        return Err("writer source/chain mismatch".into());
    }
    let ids = words(&w["generated"])?;
    let steps = array(&w["steps"])?;
    if ids.len() != steps.len() || steps.len() > 16 {
        return Err("invalid realized length".into());
    }
    let mut r = vec![idx(source), steps.len() as u32];
    for (s, &id) in steps.iter().zip(&ids) {
        if Some(id as u64) != s["emitted"].as_u64() {
            return Err("realized ID mismatch".into());
        }
        r.extend([
            idx(&s["copied"]),
            idx(&s["release"]),
            id,
            idx(&s["source_id"]),
        ]);
        for k in ["before", "after"] {
            let p = words(&s[k])?;
            if p.len() != 4 {
                return Err("invalid cursor".into());
            }
            r.extend(p)
        }
    }
    r.resize(2 + 16 * 12, 0);
    Ok(r)
}
fn adjudicate(a: &[String]) -> Result<()> {
    let [exe, tape, nominee, dir, arm, out] = a else {
        return Err("consensus dependencies".into());
    };
    let d = Path::new(dir);
    let (sites, _) = load(d.join("sites.json").to_str().ok_or("utf8")?)?;
    let (t, th) = load(tape)?;
    let (n, nh) = load(nominee)?;
    let ct = array(&t)?.len();
    row_check(ct, &t, &n, &th)?;
    if sites["bare"] != th || sites["nominee"] != nh || sites["artifact"] != n["artifact_sha256"] {
        return Err("enumerated site identity".into());
    }
    let mut candidate = vec![];
    for k in 0..idx(&sites["max"]) {
        let (c, _) = load(
            d.join(format!("rank-{k}-cont.json"))
                .to_str()
                .ok_or("utf8")?,
        )?;
        let (w, _) = load(
            d.join(format!("rank-{k}-writer.json"))
                .to_str()
                .ok_or("utf8")?,
        )?;
        row_check(ct, &t, &c, &th)?;
        row_check(ct, &t, &w, &th)?;
        if c["artifact_sha256"] != sites["artifact"] || w["model"] != sites["artifact"] {
            return Err("writer artifact".into());
        }
        candidate.push((c, w));
    }
    let mut states = vec![];
    let mut runs = vec![];
    let mut count = 0;
    for i in 0..ct {
        let ids = words(&t[i]["prefix"])?;
        let ss = words(&sites["sites"][i])?;
        let mut s = vec![ids.len() as u32];
        s.extend(ids);
        s.resize(W + 1, 0);
        s.extend([count, ss.len() as u32]);
        states.extend(s);
        for (k, f) in ss.into_iter().enumerate() {
            let (c, w) = &candidate[k];
            let row = &c["rows"][i];
            if idx(&row["frozen"]["fact"]) != f || row["frozen"]["query"] != n["rows"][i]["nominee"]
            {
                return Err("chain selected alternating address".into());
            }
            runs.extend(pack_run(&row["continuation"]["source"], &w["rows"][i])?);
            count += 1;
        }
    }
    let mut req = vec![1, ct as u32, count];
    req.extend(states);
    req.extend(runs);
    let got = packet(exe, &req, 4, ct)?;
    let rows:Vec<_>=got.iter().enumerate().map(|(i,v)|json!({"prefix_sha256":n["rows"][i]["prefix_sha256"],"nominee":n["rows"][i]["nominee"],"count":v[3],"complete":v[1],"disagree":v[2]!=0,"source":opt(v[0])})).collect();
    save(
        out,
        &json!({"arm":arm,"artifact_sha256":sites["artifact"],"bare_sha256":th,"nominee_sha256":nh,"rows":rows}),
    )
}
fn fixture(exe: &str) -> Result<()> {
    // Different earlier addresses holding equal complete value sequences are equivalent.
    let mut req = vec![1, 6, 8];
    let counts = [0, 1, 2, 2, 2, 1];
    let vals = [6, 6, 6, 7, 6, 6, 6, 6];
    let mut off = 0;
    for c in counts {
        let mut row = vec![18, 0, 6, 16, 7, 16, 6, 16];
        row.resize(193, 0);
        row.extend([off, c]);
        req.extend(row);
        off += c;
    }
    for (i, id) in vals.into_iter().enumerate() {
        let src = if i == 2 || i == 4 {
            5
        } else if id == 7 {
            3
        } else {
            1
        };
        let mut run = vec![
            src,
            2,
            1,
            0,
            id,
            id,
            1,
            src,
            src,
            18,
            1,
            src,
            src + 1,
            18,
            0,
            1,
            1,
            X,
            1,
            src,
            src + 1,
            18,
            0,
            src,
            src + 1,
            18,
        ];
        if i == 6 {
            run[2] = 0
        }
        if i == 7 {
            run[15] = 3
        }
        run.resize(194, 0);
        req.extend(run);
    }
    let v = packet(exe, &req, 4, 6)?;
    if v != vec![
        vec![X, 0, 0, 0],
        vec![1, 1, 0, 1],
        vec![1, 2, 0, 2],
        vec![X, 2, 1, 2],
        vec![X, 1, 0, 2],
        vec![X, 0, 0, 1],
    ] {
        return Err(format!("agreement finite {v:?}").into());
    }
    println!("PASS zero/single/equal/conflict/missing copy/cap release, all four CUDA and host decision words");
    Ok(())
}
fn main() -> Result<()> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    match a.as_slice() {
        [m, exe] if m == "fixture" => fixture(exe),
        [m, rest @ ..] if m == "enumerate" => enumerate(rest),
        [m, rest @ ..] if m == "adjudicate" => adjudicate(rest),
        _ => Err("fixture | enumerate | adjudicate".into()),
    }
}
