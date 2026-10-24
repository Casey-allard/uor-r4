use sha2::{Digest, Sha256};
use std::{error::Error, fs::OpenOptions, io::Write, path::Path};
use uor_r4_object_store::{
    address_read::{
        AddressDirectory, AddressReadOutcome, ObjectTarget, PrimeAddress, VersionSelect,
    },
    Store,
};

fn generate_panel() -> Result<String, Box<dyn Error>> {
    const ENTITY: [u32; 10] = [2, 5, 11, 17, 23, 29, 37, 43, 53, 59];
    const FACT_A: [u32; 10] = [3, 7, 13, 19, 31, 41, 47, 61, 67, 73];
    const FACT_B: [u32; 10] = [101, 103, 107, 109, 113, 127, 131, 137, 139, 149];
    const FACT_C: [u32; 10] = [383, 389, 397, 401, 409, 419, 421, 431, 433, 439];
    const FACT_D: [u32; 10] = [443, 449, 457, 461, 463, 467, 479, 487, 491, 499];
    const HISTORY_A: [u32; 10] = [151, 157, 163, 167, 173, 179, 181, 191, 193, 197];
    const HISTORY_B: [u32; 10] = [317, 331, 337, 347, 349, 353, 359, 367, 373, 379];
    const UNWRITTEN: [u32; 10] = [199, 211, 223, 227, 229, 233, 239, 241, 251, 257];
    const EVICTED: [u32; 10] = [263, 269, 271, 277, 281, 283, 293, 307, 311, 313];

    let mut panel = String::new();
    for i in 0..10 {
        for arm in 0..4 {
            let relation = match arm {
                0 => FACT_A[i],
                1 => FACT_B[i],
                2 => FACT_C[i],
                _ => FACT_D[i],
            };
            let value = format!("entity-{i:02}-fact-{arm}-v3");
            let prompt = format!("Remember entity-{i:02} fact {arm}: {value}.");
            panel.push_str(&emit_stated(
                i * 4 + arm,
                ENTITY[i],
                relation,
                prompt,
                value,
            )?);
        }
        for arm in 0..2 {
            let relation = if arm == 0 { HISTORY_A[i] } else { HISTORY_B[i] };
            let before = format!("entity-{i:02}-status-{arm}-v1");
            let after = format!("entity-{i:02}-status-{arm}-v2");
            let prompt = format!("Remember entity-{i:02} status {arm} changed from {before} to {after}; what was previous?");
            panel.push_str(&emit_historical(
                i * 2 + arm,
                ENTITY[i],
                relation,
                prompt,
                before,
                after,
            )?);
        }
    }
    for i in 0..10 {
        let prompt = format!("Recall unwritten entity-{i:02} fact 0.");
        panel.push_str(&emit_unwritten(i, UNWRITTEN[i], FACT_A[i], prompt)?);
        let prompt = format!("Recall evicted entity-{i:02} fact 0.");
        panel.push_str(&emit_evicted(i, EVICTED[i], FACT_A[i], prompt)?);
    }
    Ok(panel)
}

fn address(entity: u32, relation: u32) -> Result<PrimeAddress, Box<dyn Error>> {
    PrimeAddress::with_pairs(&[entity, relation], &[(entity, relation)])
        .map_err(|error| format!("invalid address {entity}/{relation}: {error}").into())
}

fn write_object(
    dir: &mut AddressDirectory,
    store: &mut Store,
    address: &PrimeAddress,
    bytes: &[u8],
) -> Result<(), Box<dyn Error>> {
    let folded = store.fold(bytes)?;
    dir.write(
        address.clone(),
        ObjectTarget::WholeObject {
            manifest: folded.manifest,
        },
    )?;
    Ok(())
}

fn emit_stated(
    index: usize,
    entity: u32,
    relation: u32,
    prompt: String,
    value: String,
) -> Result<String, Box<dyn Error>> {
    let address = address(entity, relation)?;
    let mut store = Store::new(uor_r4_object_store::Limits::default());
    let mut dir = AddressDirectory::new(4)?;
    write_object(&mut dir, &mut store, &address, value.as_bytes())?;
    match dir.read_exact(&store, None, &address, VersionSelect::Current)? {
        AddressReadOutcome::Hit {
            bytes,
            whole_object_verified,
            ..
        } if bytes == value.as_bytes() && whole_object_verified => Ok(json_row(
            &format!("stated-{index:03}"),
            "stated",
            &prompt,
            &address,
            "current",
            "hit",
            Some(&value),
            None,
        )),
        other => Err(format!("stated read mismatch: {other:?}").into()),
    }
}

fn emit_historical(
    index: usize,
    entity: u32,
    relation: u32,
    prompt: String,
    before: String,
    after: String,
) -> Result<String, Box<dyn Error>> {
    let address = address(entity, relation)?;
    let mut store = Store::new(uor_r4_object_store::Limits::default());
    let mut dir = AddressDirectory::new(4)?;
    write_object(&mut dir, &mut store, &address, before.as_bytes())?;
    write_object(&mut dir, &mut store, &address, after.as_bytes())?;
    match dir.read_exact(&store, None, &address, VersionSelect::Previous)? {
        AddressReadOutcome::Previous {
            bytes,
            whole_object_verified,
            ..
        } if bytes == before.as_bytes() && whole_object_verified => Ok(json_row(
            &format!("historical-{index:03}"),
            "historical",
            &prompt,
            &address,
            "previous",
            "previous",
            Some(&before),
            None,
        )),
        other => Err(format!("historical read mismatch: {other:?}").into()),
    }
}

fn emit_unwritten(
    index: usize,
    entity: u32,
    relation: u32,
    prompt: String,
) -> Result<String, Box<dyn Error>> {
    let address = address(entity, relation)?;
    let store = Store::new(uor_r4_object_store::Limits::default());
    let dir = AddressDirectory::new(4)?;
    match dir.read_exact(&store, None, &address, VersionSelect::Current)? {
        AddressReadOutcome::Absent => Ok(json_row(
            &format!("unwritten-{index:03}"),
            "unwritten",
            &prompt,
            &address,
            "current",
            "absent",
            None,
            None,
        )),
        other => Err(format!("unwritten read mismatch: {other:?}").into()),
    }
}

fn emit_evicted(
    index: usize,
    entity: u32,
    relation: u32,
    prompt: String,
) -> Result<String, Box<dyn Error>> {
    let replacement = address(relation, entity)?;
    let address = address(entity, relation)?;
    let mut store = Store::new(uor_r4_object_store::Limits::default());
    let mut dir = AddressDirectory::new(1)?;
    write_object(&mut dir, &mut store, &address, b"old")?;
    write_object(&mut dir, &mut store, &replacement, b"new")?;
    match dir.read_exact(&store, None, &address, VersionSelect::Current)? {
        AddressReadOutcome::Evicted {
            evicted_at_sequence,
            ..
        } if evicted_at_sequence == 2 => Ok(json_row(
            &format!("evicted-{index:03}"),
            "evicted",
            &prompt,
            &address,
            "current",
            "evicted",
            None,
            Some(evicted_at_sequence),
        )),
        other => Err(format!("evicted read mismatch: {other:?}").into()),
    }
}

fn json_row(
    id: &str,
    family: &str,
    prompt: &str,
    address: &PrimeAddress,
    version: &str,
    outcome: &str,
    expected: Option<&str>,
    evicted_at: Option<u64>,
) -> String {
    let primes = json_numbers(address.primes());
    let pairs = address
        .ordered_pairs()
        .iter()
        .map(|(a, b)| format!("[{a},{b}]"))
        .collect::<Vec<_>>()
        .join(",");
    let expected = expected.map(json_string).unwrap_or_else(|| "null".into());
    let evicted_at = evicted_at.map_or_else(|| "null".into(), |value| value.to_string());
    format!(
        "{{\"row_id\":{},\"family\":{},\"prompt\":{},\"primes\":{},\"ordered_pairs\":{},\"version_select\":{},\"outcome\":{},\"expected_bytes\":{},\"evicted_at_sequence\":{}}}\n",
        json_string(id),
        json_string(family),
        json_string(prompt),
        primes,
        format!("[{pairs}]"),
        json_string(version),
        json_string(outcome),
        expected,
        evicted_at,
    )
}

fn json_numbers(values: &[u32]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('\"');
    out
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let output = args.next().ok_or("usage: address-panel-freeze OUT")?;
    if args.next().is_some() {
        return Err("usage: address-panel-freeze OUT".into());
    }
    let panel = generate_panel()?;
    let path = Path::new(&output);
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(panel.as_bytes())?;
    file.sync_all()?;
    println!("rows 80");
    println!("bytes {}", panel.len());
    println!("sha256 {}", hex(&Sha256::digest(panel.as_bytes())));
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn registered_composition_and_byte_reproduction() {
        let first = generate_panel().unwrap();
        let second = generate_panel().unwrap();
        assert_eq!(first, second);
        assert_eq!(first.lines().count(), 80);
        assert_eq!(first.matches("\"family\":\"stated\"").count(), 40);
        assert_eq!(first.matches("\"family\":\"historical\"").count(), 20);
        assert_eq!(first.matches("\"family\":\"unwritten\"").count(), 10);
        assert_eq!(first.matches("\"family\":\"evicted\"").count(), 10);
        assert_eq!(first.matches("\"outcome\":\"absent\"").count(), 10);
        assert_eq!(first.matches("\"outcome\":\"evicted\"").count(), 10);
    }

    #[test]
    fn addresses_and_prompts_are_unique() {
        let panel = generate_panel().unwrap();
        let addresses = panel
            .lines()
            .map(|line| {
                line.split("\"primes\":")
                    .nth(1)
                    .unwrap()
                    .split(",\"version_select\"")
                    .next()
                    .unwrap()
            })
            .collect::<BTreeSet<_>>();
        let prompts = panel
            .lines()
            .map(|line| {
                line.split("\"prompt\":")
                    .nth(1)
                    .unwrap()
                    .split(",\"primes\"")
                    .next()
                    .unwrap()
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(addresses.len(), 80);
        assert_eq!(prompts.len(), 80);
    }
}
