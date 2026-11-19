//! Activate a pinned passive S2 carrier without rewriting its retained table or trunk.
use serde::Deserialize;
use serde_json::value::RawValue;
use std::{
    env,
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
use uor_r4_integer::discrete_d11::{
    IntegerDiscreteModel, DISCRETE_S2_EXPERT_SCHEMA, DISCRETE_S2_SCHEMA,
};

#[derive(Deserialize)]
struct SchemaField<'a> {
    #[serde(borrow)]
    schema: &'a RawValue,
}

fn activate(bytes: &mut [u8], expected: &str) -> Result<(), Box<dyn Error>> {
    let model = IntegerDiscreteModel::from_bytes(bytes)?;
    if model.artifact_schema() != DISCRETE_S2_SCHEMA || model.artifact_sha256() != expected {
        return Err("input must be the pinned passive S2 carrier".into());
    }
    drop(model);
    let len = usize::try_from(u64::from_le_bytes(bytes[8..16].try_into()?))?;
    let header = &bytes[16..16 + len];
    let field: SchemaField = serde_json::from_slice(header)?;
    // RawValue borrows the schema string from this exact header. Both literal
    // '1' and its JSON Unicode escape end in the same ASCII byte.
    let raw = field.schema.get();
    let start = (raw.as_ptr() as usize)
        .checked_sub(header.as_ptr() as usize)
        .ok_or("schema is outside the header")?;
    let final_digit = start + raw.len() - 2;
    if header.get(final_digit..=final_digit + 1) != Some(b"1\"") {
        return Err("unexpected passive S2 schema spelling".into());
    }
    bytes[16 + final_digit] = b'2';
    if IntegerDiscreteModel::from_bytes(bytes)?.artifact_schema() != DISCRETE_S2_EXPERT_SCHEMA {
        return Err("invalid active S2 schema".into());
    }
    Ok(())
}

fn publish(output: &Path, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let tmp = output.with_file_name(format!(".activate-s2-{}-{stamp}.tmp", std::process::id()));
    let mut f = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
    let written = (|| -> std::io::Result<()> {
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        // The destination becomes visible only as a complete file, and a
        // concurrent activation or an existing destination is never overwritten.
        fs::hard_link(&tmp, output)
    })();
    let cleanup = fs::remove_file(tmp);
    written?;
    cleanup?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [input, pin, output] = args.as_slice() else {
        return Err("usage: activate-s2-expert PASSIVE.d11 EXPECTED_SHA256 NEW_ACTIVE.d11".into());
    };
    let mut bytes = fs::read(input)?;
    activate(&mut bytes, pin.to_str().ok_or("invalid SHA-256 text")?)?;
    publish(Path::new(output), &bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use uor_r4_integer::discrete_d11::{
        spiralcore_orbit_roots, MAGIC, S2_SECTION, SNAPSHOT_SCHEMA,
    };

    #[test]
    fn pinned_passive_carrier_with_unsupported_vocabulary_cannot_activate() {
        let mut payload = Vec::new();
        let mut tables = Vec::new();
        let mut section = |name: &str, kind: &str, words: &[i32]| {
            payload.resize(payload.len().next_multiple_of(64), 0);
            tables.push(json!({
                "name": name, "kind": kind, "len": words.len(),
                "span": {"offset": payload.len(), "bytes": words.len() * 4},
            }));
            payload.extend(words.iter().flat_map(|word| word.to_le_bytes()));
        };
        let root = spiralcore_orbit_roots()[0].map(i32::from);
        section("discrete.table", "i32", &root.repeat(8));
        section("discrete.cells", "u32", &[0; 2]);
        section("discrete.routes", "u32", &[0; 4]);
        section("discrete.gates", "i32", &[0; 4]);
        section("discrete.trunk.step_candidates", "u32", &[0; 64]);
        section("discrete.trunk.norm_candidates", "u32", &[0; 4]);
        section("discrete.trunk.unigram", "u32", &[1, 0, 1]);
        for name in [
            "read_table",
            "rec_table",
            "read_rec_table",
            "rec_shuf_table",
            "read_rec_shuf_table",
            "rec_unit_table",
            "read_rec_unit_table",
            "query_table",
            "query_read_table",
            "query_rec_table",
            "query_rec_shuf_table",
            "query_rec_unit_table",
        ] {
            section(&format!("discrete.trunk.{name}"), "u32", &[]);
        }
        section(S2_SECTION, "u32", &[0; 24]);

        let header = serde_json::to_vec(&json!({
            "schema": DISCRETE_S2_SCHEMA,
            "shape": {
                "vocabulary": 2, "blocks": 4, "context": 1, "kept_cells": 1, "stages": 1,
                "read_mode": "route", "read_kept": 0, "read_sweeps": 0,
                "objective_policy": "nearest_address_margin_v1",
                "read_initialization": "zero_gates_v1", "seed": 0, "epoch": 0,
            },
            "matrices": [], "tables": tables,
            "source": {
                "snapshot_schema": SNAPSHOT_SCHEMA, "snapshot_sha256": "",
                "operator_kappa": "blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b",
                "composition_kappa": "blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0",
                "kernel_sha256": "", "source_sha256": "",
                "geometry_sha256": "",
            },
        }))
        .unwrap();
        let mut bytes = MAGIC.to_vec();
        bytes.extend((header.len() as u64).to_le_bytes());
        bytes.extend(header);
        bytes.resize(bytes.len().next_multiple_of(64), 0);
        bytes.extend(payload);

        let passive = IntegerDiscreteModel::from_bytes(&bytes).unwrap();
        assert_eq!(passive.artifact_schema(), DISCRETE_S2_SCHEMA);
        let error = activate(&mut bytes, passive.artifact_sha256()).unwrap_err();
        assert!(error.to_string().contains("standalone 4096-token trunk"));
    }
}
