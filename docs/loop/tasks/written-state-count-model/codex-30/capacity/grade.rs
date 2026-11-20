use super::Result;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, io::Write, path::Path};
use uor_r4_integer::{
    discrete_d11::contextual_source::{SourceArm, SourceBank, SourceCandidate, SourceOutcome},
    discrete_d11::spiralcore_orbit_roots,
    paired_role::RoleSpan,
};

const CODES: usize = 65536;
type Rows = BTreeMap<u32, Vec<SourceCandidate>>;

fn load_rows(path: &Path) -> Result<Rows> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some("D25ROWS1") {
        return Err("missing device packet header".into());
    }
    let mut rows = Rows::new();
    for line in lines {
        let w = line
            .split_ascii_whitespace()
            .map(str::parse::<i32>)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if w.len() != 69 || w[4] != 0 || !(0..=1).contains(&w[3]) {
            return Err("invalid prediction row".into());
        }
        let group = rows.entry(w[0].try_into()?).or_default();
        if w[3] == 0 {
            if w[1..].iter().any(|&x| x != 0) {
                return Err("nonempty masked row".into());
            }
            continue;
        }
        let address = usize::try_from(w[1])?;
        let local: Vec<_> = w[5..37]
            .iter()
            .map(|&x| i16::try_from(x))
            .collect::<std::result::Result<_, _>>()?;
        let context: [i32; 32] = w[37..].try_into()?;
        if group
            .last()
            .is_some_and(|r| r.span.start >= address || r.contextual != context)
        {
            return Err("inconsistent nominal candidates".into());
        }
        // The selector consumes only the start, eligibility and native coordinates.
        group.push(SourceCandidate {
            span: RoleSpan {
                start: address,
                end: address + 1,
            },
            narrow: w[2] == 1,
            local: local.try_into().map_err(|_| "wrong local width")?,
            contextual: context,
            completed_source_range: [address as u32, address as u32 + 1],
        });
    }
    Ok(rows)
}
fn decode(out: u32) -> Result<(&'static str, Option<u32>)> {
    let address = out & 0xffffff;
    match (out >> 24, address) {
        (0, a) => Ok(("pick", Some(a))),
        (1, 0) => Ok(("empty", None)),
        (3, 0) => Ok(("nonpositive", None)),
        (4, 0) => Ok(("tie", None)),
        _ => Err("invalid frozen two-polarity bank result".into()),
    }
}
fn distances(a: u16, b: u16) -> u32 {
    (0..4).filter(|i| ((a ^ b) >> (4 * i)) & 15 != 0).count() as u32
}
fn context(code: u16) -> [i32; 32] {
    let roots = spiralcore_orbit_roots();
    std::array::from_fn(|i| i32::from(roots[((code >> (4 * (i / 8))) & 15) as usize][i % 8]))
}
fn code(ctx: &[i32; 32]) -> Result<u16> {
    let mut out = 0;
    for (i, block) in ctx.chunks_exact(8).enumerate() {
        let roots = spiralcore_orbit_roots();
        let k = roots
            .iter()
            .position(|v| v.iter().zip(block).all(|(&x, &y)| i32::from(x) == y))
            .ok_or("observed context outside recurrence lattice")?;
        out |= (k as u16) << (4 * i);
    }
    Ok(out)
}
fn native(bank: &SourceBank, rows: &[SourceCandidate], w: u16) -> Result<u32> {
    let rows = rows.iter().cloned().map(|mut r| {
        r.contextual = context(w);
        Ok(r)
    });
    let selection = bank.select(SourceArm::WideRecurrence, rows, |_, _| {})?;
    Ok(match selection.outcome {
        SourceOutcome::Pick => selection.source.ok_or("source missing")?,
        SourceOutcome::Empty => 1 << 24,
        SourceOutcome::Nonpositive => 3 << 24,
        SourceOutcome::Tie => 4 << 24,
    })
}
fn read_map(path: &Path) -> Result<BTreeMap<u32, Vec<u32>>> {
    let bytes = fs::read(path)?;
    if bytes.len() < 16 || &bytes[..8] != b"D30FIB01" || bytes.len() % 4 != 0 {
        return Err("invalid capacity map".into());
    }
    let words = bytes[8..]
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
        .collect::<Vec<_>>();
    if words[1] != CODES as u32 || words.len() != 2 + (1 + CODES) * words[0] as usize {
        return Err("capacity shape differs".into());
    }
    let mut out = BTreeMap::new();
    for g in words[2..].chunks_exact(CODES + 1) {
        if out.insert(g[0], g[1..].to_vec()).is_some() {
            return Err("duplicate capacity row".into());
        }
    }
    Ok(out)
}
#[derive(Deserialize)]
struct Key {
    row: u32,
    gold: Option<u32>,
    fact_and_query: Option<[[u32; 2]; 2]>,
}
fn read_decisions(p: &Path) -> Result<BTreeMap<u32, (&'static str, Option<u32>)>> {
    let mut out = BTreeMap::new();
    for line in fs::read_to_string(p)?.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() != 4 {
            return Err("bad prediction".into());
        }
        if fields[1] != "3" {
            continue;
        }
        let choice = match fields[2] {
            "pick" => ("pick", Some(fields[3].parse()?)),
            "empty" => ("empty", None),
            "nonpositive" => ("nonpositive", None),
            "tie" => ("tie", None),
            _ => return Err("invalid score reason".into()),
        };
        if out.insert(fields[0].parse()?, choice).is_some() {
            return Err("duplicate score".into());
        }
    }
    Ok(out)
}
pub fn grade(args: &[String]) -> Result<()> {
    let [rows, bank, map, nominal, swapped, donors, key, out] = args else {
        return Err("expected NOMINAL.rows BANK MAP NOMINAL.results SWAPPED.results DONORS.tsv KEY.json NEW_OUT".into());
    };
    let rows = load_rows(rows.as_ref())?;
    let bank = SourceBank::from_bytes(&fs::read(bank)?)?;
    let maps = read_map(map.as_ref())?;
    let nominal = read_decisions(nominal.as_ref())?;
    let swapped = read_decisions(swapped.as_ref())?;
    let mut pairs = BTreeMap::new();
    for line in fs::read_to_string(donors)?.lines() {
        let (r, d) = line.split_once('\t').ok_or("bad donor pair")?;
        if pairs.insert(r.parse::<u32>()?, d.parse::<u32>()?).is_some() {
            return Err("duplicate pair".into());
        }
    }
    let keys: Vec<Key> = serde_json::from_slice(&fs::read(key)?)?;
    if rows.len() != maps.len() || keys.len() != maps.len() || nominal.len() != maps.len() {
        return Err("unequal row sets".into());
    }
    let mut report = Vec::<Value>::new();
    for key in &keys {
        let id = key.row;
        let rows_here = rows.get(&id).ok_or("candidate group missing")?;
        let codes = maps.get(&id).ok_or("missing exhaustive group")?;
        let observed = rows_here.first().map(|r| code(&r.contextual)).transpose()?;
        let (actual, actual_source) = decode(codes[observed.unwrap_or(0) as usize])?;
        if nominal.get(&id) != Some(&(actual, actual_source)) {
            return Err(format!("nominal differs {id}").into());
        }
        let donor = pairs.get(&id).copied();
        if let Some(d) = donor {
            let other = rows
                .get(&d)
                .and_then(|r| r.first())
                .ok_or("missing donor vector")?;
            let donor_code = code(&other.contextual)?;
            if swapped.get(&id) != Some(&decode(codes[usize::from(donor_code)])?) {
                return Err(format!("donor differs {id}").into());
            }
            let other_key = keys
                .iter()
                .find(|k| k.row == d)
                .ok_or("missing donor key")?;
            let head = key.fact_and_query.ok_or("missing fact")?[0][0];
            let other_head = other_key.fact_and_query.ok_or("missing donor fact")?[0][0];
            let relatives = |v, head: u32| -> Result<_> {
                let (reason, s) = decode(v)?;
                Ok((reason, s.map(|x| i64::from(x) - i64::from(head))))
            };
            let other_map = maps.get(&d).ok_or("missing donor map")?;
            for (&a, &b) in codes.iter().zip(other_map) {
                if relatives(a, head)? != relatives(b, other_head)? {
                    return Err(format!("paired maps differ at relative addresses {id}/{d}").into());
                }
            }
        }
        // Audit all distinct outcomes with the production Rust selector at a nearest witness.
        let mut counts = BTreeMap::<u32, (u32, u32, u16)>::new();
        for (w, &result) in codes.iter().enumerate() {
            decode(result)?;
            let d = distances(observed.unwrap_or(0), w as u16);
            let entry = counts.entry(result).or_insert((0, u32::MAX, 0));
            entry.0 += 1;
            if d < entry.1 {
                entry.1 = d;
                entry.2 = w as u16;
            }
        }
        let mut outcomes = Vec::new();
        for (&result, &(n, changes, w)) in &counts {
            let (reason, source) = decode(result)?;
            if native(&bank, rows_here, w)? != result {
                return Err("production witness disagrees".into());
            }
            outcomes.push(json!({"reason":reason,"source":source,"count":n,"witness_code":w,"min_block_changes":observed.map(|_|changes)}));
        }
        if let Some(obs) = observed {
            if native(&bank, rows_here, obs)? != codes[usize::from(obs)] {
                return Err("actual native decision disagrees".into());
            }
        }
        report.push(json!({"row":id,"gold":key.gold,"donor":donor,"nominal_code":observed,"nominal_source":actual_source,"eligible":rows_here.iter().map(|r|r.span.start).collect::<Vec<_>>(),"gold_codes":key.gold.map(|s|counts.get(&s).map_or(0,|t|t.0)),"gold_min_block_changes":key.gold.and_then(|s|counts.get(&s)).map(|t|t.1),"outcomes":outcomes}));
    }
    let mut f = fs::File::create_new(out)?;
    serde_json::to_writer_pretty(&mut f, &report)?;
    writeln!(f)?;
    Ok(())
}
