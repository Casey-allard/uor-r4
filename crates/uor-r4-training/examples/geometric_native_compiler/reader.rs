//! Native record-to-answer execution. Labels never enter these reader inputs.
use serde_json::{json, Value};
use std::{fs, path::Path};
use uor_r4_integer::{
    geometric_cue_carrier::CueAngularQ4,
    geometric_occurrence_read::{FrameMetadata, FrameStatus, SelectedRecordFrame, SourceIdentity},
    geometric_prefix_transport::PrefixAngularQ4,
    geometric_source_end_transport::SourceEndAngularQ4,
    geometric_source_realizer::{NativeSourceRealizer, SourceBankSegment},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub struct Record {
    pub event: u64,
    pub record: u64,
    pub commit: u64,
    pub relation: u32,
    pub cue: Vec<u32>,
    pub value: Vec<u32>,
}
fn metadata(root: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(
        root.join("native-metadata.json"),
    )?)?)
}
fn mismatch() -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "native reader donor metadata mismatch",
    )
}
pub fn generate(
    parent: &NativeSourceRealizer,
    cue_root: &Path,
    prefix_root: &Path,
    end_root: &Path,
    query: &[u32],
    records: &[Record],
    maximum: usize,
) -> Result<Value> {
    if records.is_empty() || maximum == 0 || maximum > 32 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "reader needs records and a bounded output window",
        )
        .into());
    }
    let cm = metadata(cue_root)?;
    let cue = parent.compile_cue_carrier(CueAngularQ4::new(
        serde_json::from_value(cm["potential"].clone())?,
        &fs::read(cue_root.join("cue-q4.bin"))?,
    )?)?;
    if serde_json::to_value(cue.metadata())? != cm {
        return Err(mismatch().into());
    }
    let pm = metadata(prefix_root)?;
    let prefix = parent.compile_prefix_transport(
        &cue,
        PrefixAngularQ4::new(
            serde_json::from_value(pm["potential"].clone())?,
            &fs::read(prefix_root.join("prefix-q4.bin"))?,
        )?,
    )?;
    if serde_json::to_value(prefix.metadata())? != pm {
        return Err(mismatch().into());
    }
    let em = metadata(end_root)?;
    let end = parent.compile_source_end_transport(
        &cue,
        &prefix,
        SourceEndAngularQ4::new(
            serde_json::from_value(em["potential"].clone())?,
            &fs::read(end_root.join("source-end-period-q4.bin"))?,
            &fs::read(end_root.join("source-end-stop-q4.bin"))?,
        )?,
    )?;
    if serde_json::to_value(end.metadata())? != em {
        return Err(mismatch().into());
    }
    let views = records
        .iter()
        .map(|r| parent.compile_view(&r.value))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut segments = Vec::new();
    for (r, view) in records.iter().zip(&views) {
        segments.push(SourceBankSegment::Context {
            token_ids: &r.cue,
            event: r.event,
            role: 1,
        });
        segments.push(SourceBankSegment::Source {
            frame: SelectedRecordFrame {
                identity: SourceIdentity {
                    record: r.record,
                    commit: r.commit,
                },
                metadata: FrameMetadata {
                    scope: b"compiler-probe",
                    entity: &[1],
                    relation: r.relation,
                    view: 0,
                    status: FrameStatus::Found,
                },
                token_ids: &r.value,
            },
            view,
            event: r.event,
        });
    }
    let mut ids = Vec::new();
    let mut traces = Vec::new();
    for step in 0..maximum {
        let trace = parent
            .read_bank_with_source_end_transport(&segments, query, &ids, &cue, &prefix, &end)?;
        let chosen = trace.actions.chosen_token_id;
        traces.push(json!({"step":step,"actual_prefix":ids,"native":trace}));
        ids.push(chosen);
        if chosen == parent.binding().eos_token_id() {
            break;
        }
    }
    Ok(
        json!({"generated_ids_including_eos":ids,"eos":ids.last()==Some(&parent.binding().eos_token_id()),
        "tokens":traces,"record_count":records.len(),"query":query,
        "policy":"original-assertion-cue;actual-store-value;own-prefix;native-global-alias-argmax"}),
    )
}
