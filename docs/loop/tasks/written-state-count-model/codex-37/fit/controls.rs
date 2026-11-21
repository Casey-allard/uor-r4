//! Reconstruct the registered C1 rows from the sealed ARM-A1 feature tape.
//! The Pick class stays fixed and the Pick lane carries no mass.

use super::*;
use std::io::{BufReader, Read};

pub fn masked(identity: &Path, identity_pin: &str, out: &Path) -> Result<(), Failure> {
    let prepared = fs::read(identity.join("prepare.json"))?;
    if digest(&prepared) != identity_pin {
        return Err(fail("wrong sealed identity preparation digest"));
    }
    let prepared: Value = serde_json::from_slice(&prepared)?;
    let source = identity.join("a1.rows");
    let rule = PickClassRule::Three;
    let mut read = BufReader::with_capacity(1 << 20, fs::File::open(source)?);
    let mut tape_header = [0; 24];
    read.read_exact(&mut tape_header)?;
    if tape_header != header(b"R37EM001", REAL_ROWS, rule)
        || prepared["format"] != PREPARED
        || prepared["pick_mode"] != "identity"
        || prepared["r_sha256"] != R_SHA
        || prepared["rows"] != REAL_ROWS
        || prepared["c_sha256"] != C_SHA
        || prepared["compact_sha256"] != COMPACT_SHA
        || prepared["trunk_sha256"] != TRUNK_SHA
    {
        return Err(fail("wrong identity preparation or row header for C1"));
    }
    let mut upstream = Sha256::new();
    upstream.update(tape_header);
    fs::create_dir(out)?;
    let path = out.join("a1.rows");
    let mut masked = writer(&path)?;
    masked.write_all(&tape_header)?;
    let mut row = [0; 8 * (EXPERTS + 1)];
    for ordinal in 0..REAL_ROWS {
        read.read_exact(&mut row)?;
        upstream.update(row);
        let pick = word(&row[8 * EXPERTS..])?;
        if (pick != 0 && pick != 1 << 48) || word(&row[..8])? >= buckets(rule) as u64 {
            return Err(fail(format!("invalid identity row at ordinal {ordinal}")));
        }
        row[8 * EXPERTS..].fill(0);
        masked.write_all(&row)?;
    }
    if read.read(&mut [0])? != 0
        || prepared["arms"]["a1"]["rows_sha256"] != format!("{:x}", upstream.finalize())
    {
        return Err(fail("consumed identity rows differ"));
    }
    finish(masked)?;
    let ordinals = fs::read(identity.join("probe-ordinals.u64"))?;
    let probes = fs::read(identity.join("probes.json"))?;
    if prepared["probe_ordinals_sha256"] != digest(&ordinals)
        || prepared["probes_sha256"] != digest(&probes)
    {
        return Err(fail("sealed identity probe inputs changed"));
    }
    let mut probes: Vec<Value> = serde_json::from_slice(&probes)?;
    for probe in &mut probes {
        probe["original_hit"] = probe["hit"].clone();
        probe["hit"] = json!(false);
    }
    let probes = serde_json::to_vec_pretty(&probes)?;
    write_new(&out.join("probes.json"), &probes)?;
    write_new(&out.join("probe-ordinals.u64"), &ordinals)?;
    let receipt = json!({
        "format": PREPARED, "rows": REAL_ROWS, "records": RECORDS,
        "c_sha256": C_SHA, "r_sha256": R_SHA, "trunk_sha256": TRUNK_SHA,
        "compact_sha256": COMPACT_SHA, "count_tape_sha256": prepared["count_tape_sha256"],
        "pick_mode": "masked", "pick_tape_sha256": prepared["pick_tape_sha256"],
        "identity_preparation_sha256": identity_pin,
        "arms": {"a1": {"rows_sha256": file_digest(&path)?,
            "classes": rule.classes(), "class_counts": prepared["arms"]["a1"]["class_counts"],
            "hit_counts": [0, 0, 0, 0, 0]}},
        "probes_sha256": digest(&probes), "probe_ordinals_sha256": digest(&ordinals),
        "host_source_sha256": digest(include_bytes!("main.rs")),
        "prep_source_sha256": digest(include_bytes!("prepare.rs")),
        "device_source_sha256": digest(include_bytes!("device.cu")),
        "masked_source_sha256": digest(include_bytes!("controls.rs")),
    });
    write_new(
        &out.join("prepare.json"),
        &serde_json::to_vec_pretty(&receipt)?,
    )?;
    Ok(())
}
