#![forbid(unsafe_code)]

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    env,
    error::Error,
    fs::{self, File},
    io::Write,
    path::Path,
};

type Fail = Box<dyn Error>;
const CAP: usize = 125_998_708;
const BASE_SHA: &str = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";
const SOURCE_SHA: &str = "6da3ac9fcfd8456f803284042f2eece9eca9d471e57affbe2cb3e1b726745be7";
const FIXTURE_SHA: &str = "eebb8ff66ca5190e0ddfbe0ae4c6d701b3adcbbe52162c77681dd3d67cae03e7";
const FIXTURE_CAP: usize = 86_538_000;

const PROVENANCE: usize = 96;
const ROW: usize = 20;
const DOMAIN: &[u8] = b"uor-r4/codex-27/s2-retention/v1";

fn require(ok: bool, detail: &str) -> Result<(), Fail> {
    if !ok {
        return Err(detail.into());
    }
    Ok(())
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn parts(artifact: &[u8]) -> Result<(Value, &[u8]), Fail> {
    require(artifact.starts_with(b"UORLUT01"), "invalid magic")?;
    let len = u64::from_le_bytes(artifact.get(8..16).ok_or("short header")?.try_into()?);
    let end = 16_usize
        .checked_add(len.try_into()?)
        .ok_or("header overflow")?;
    let header = serde_json::from_slice(artifact.get(16..end).ok_or("short JSON")?)?;
    let start = end
        .checked_next_multiple_of(64)
        .ok_or("alignment overflow")?;
    Ok((header, artifact.get(start..).ok_or("short D11 prefix")?))
}

#[derive(Clone)]
struct Group {
    key: u128,
    first: usize,
    rows: usize,
}

fn groups(rows: &[u8]) -> Result<[Vec<Group>; 11], Fail> {
    require(rows.len().is_multiple_of(ROW), "partial source row")?;
    let mut orders: [Vec<Group>; 11] = std::array::from_fn(|_| Vec::new());
    let mut previous = None;
    for (index, row) in rows.chunks_exact(ROW).enumerate() {
        let key = u128::from_le_bytes(row[..16].try_into()?);
        let order = orders.get_mut((key >> 120) as usize).ok_or("bad order")?;
        require(previous.is_none_or(|prior| prior <= key), "unordered key")?;
        if previous == Some(key) {
            let group = order.last_mut().ok_or("missing group start")?;
            group.rows += 1;
            require(group.rows <= 4, "oversized group")?;
        } else {
            order.push(Group {
                key,
                first: index,
                rows: 1,
            });
        }
        previous = Some(key);
    }
    Ok(orders)
}

fn keep_prefix(order: &mut Vec<Group>, allowance: usize) {
    let mut ranked: Vec<_> = order
        .drain(..)
        .map(|group| {
            let mut digest = Sha256::new();
            digest.update(DOMAIN);
            digest.update(group.key.to_le_bytes());
            (<[u8; 32]>::from(digest.finalize()), group.key, group)
        })
        .collect();
    ranked.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    let mut remaining = allowance;
    for (_, _, group) in ranked {
        if group.rows > remaining {
            break;
        }
        remaining -= group.rows;
        order.push(group);
    }
    order.sort_unstable_by_key(|group| group.first);
}

fn row_count(order: &[Group]) -> usize {
    order.iter().map(|group| group.rows).sum()
}

fn section_size(header: &mut Value, bytes: usize) -> Result<Vec<u8>, Fail> {
    let descriptor = header["tables"]
        .as_array_mut()
        .and_then(|tables| tables.last_mut())
        .ok_or("missing S2 descriptor")?;
    descriptor["len"] = json!(bytes / 4);
    descriptor["span"]["bytes"] = json!(bytes);
    Ok(serde_json::to_vec(header)?)
}

fn pack(base: &[u8], source: &[u8], cap: usize) -> Result<(Vec<u8>, Value), Fail> {
    let (base_header, trunk) = parts(base)?;
    let (mut header, payload) = parts(source)?;
    require(
        header["schema"] == "uor-r4.lut-discrete-trunk-s2/2",
        "expected active S2",
    )?;
    let mut trunk_header = header.clone();
    let descriptor = trunk_header["tables"]
        .as_array_mut()
        .and_then(Vec::pop)
        .ok_or("missing S2 descriptor")?;
    require(
        descriptor["name"] == "discrete.trunk.s2.top4.log2.v1",
        "unexpected final section",
    )?;
    trunk_header["schema"] = base_header["schema"].clone();
    let tokenizer = trunk_header
        .as_object_mut()
        .ok_or("invalid header")?
        .remove("tokenizer_sha256")
        .ok_or("unbound tokenizer")?;
    require(
        trunk_header == base_header,
        "trunk descriptors or binds changed",
    )?;
    let offset: usize = descriptor["span"]["offset"]
        .as_u64()
        .ok_or("missing offset")?
        .try_into()?;
    let section = payload.get(offset..).ok_or("missing S2 payload")?;
    require(
        descriptor["span"]["bytes"].as_u64() == Some(section.len() as u64),
        "unexpected section length",
    )?;
    let prefix = &payload[..offset];
    require(
        prefix.starts_with(trunk) && prefix[trunk.len()..].iter().all(|&byte| byte == 0),
        "trunk bytes changed",
    )?;
    let provenance = section.get(..PROVENANCE).ok_or("short provenance")?;
    require(
        provenance[..32] == Sha256::digest(base)[..],
        "different base provenance",
    )?;
    let rows = &section[PROVENANCE..];
    let mut orders = groups(rows)?;
    let conservative_header = (16 + section_size(&mut header, cap)?.len()).next_multiple_of(64);
    let available = cap
        .checked_sub(conservative_header + offset + PROVENANCE)
        .ok_or("cap too small for envelope")?
        / ROW;
    let fixed: usize = orders[..3].iter().map(|order| row_count(order)).sum();
    let remaining = available
        .checked_sub(fixed)
        .ok_or("cap too small for low orders")?;
    let mut census = Vec::new();
    for (index, order) in orders.iter_mut().enumerate() {
        let input_groups = order.len();
        let input_rows = row_count(order);
        let allowance = if index < 3 {
            input_rows
        } else {
            remaining / 8 + usize::from(index - 3 < remaining % 8)
        };
        if input_rows > allowance {
            keep_prefix(order, allowance);
        }
        census.push(json!({
            "order": index, "inputGroups": input_groups, "inputRows": input_rows,
            "allowanceRows": allowance, "retainedGroups": order.len(), "retainedRows": row_count(order)
        }));
    }
    let retained_rows: usize = orders.iter().map(|order| row_count(order)).sum();
    let header = section_size(&mut header, PROVENANCE + retained_rows * ROW)?;
    let mut artifact = Vec::new();
    artifact.extend_from_slice(b"UORLUT01");
    artifact.extend_from_slice(&(header.len() as u64).to_le_bytes());
    artifact.extend_from_slice(&header);
    let payload_start = artifact.len().next_multiple_of(64);
    artifact.resize(payload_start, 0);
    artifact.extend_from_slice(prefix);
    artifact.extend_from_slice(provenance);
    for group in orders.iter().flatten() {
        artifact.extend_from_slice(&rows[group.first * ROW..(group.first + group.rows) * ROW]);
    }
    require(artifact.len() <= cap, "output exceeds cap")?;
    let report = json!({
        "outputSha256": sha(&artifact), "capBytes": cap, "outputBytes": artifact.len(),
        "unusedBytes": cap - artifact.len(), "headerAndAlignmentBytes": payload_start,
        "trunkBytes": trunk.len(), "trunkPaddingBytes": offset - trunk.len(),
        "countProvenanceBytes": PROVENANCE, "countRecordBytes": retained_rows * ROW,
        "tokenizerSha256": tokenizer, "orders": census
    });
    Ok((artifact, report))
}

fn main() -> Result<(), Fail> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [base, source, output] = args.as_slice() else {
        return Err("usage: budget-s2-pack BASE.d11 PINNED_SOURCE.d11 NEW_OUTPUT.d11".into());
    };
    let base = fs::read(base)?;
    let source = fs::read(source)?;
    require(sha(&base) == BASE_SHA, "unexpected base digest")?;
    let cap = match sha(&source).as_str() {
        SOURCE_SHA => CAP,
        FIXTURE_SHA => FIXTURE_CAP,
        _ => return Err("unexpected source digest".into()),
    };

    let (artifact, report) = pack(&base, &source, cap)?;
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(Path::new(output))?;
    file.write_all(&artifact)?;
    file.sync_all()?;
    drop(file);
    require(fs::read(output)? == artifact, "readback bytes differ")?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
