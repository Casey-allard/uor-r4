//! Target-free attribution of prospectively authenticated actual reached-prefix frames.
//! Offline saved witnesses only; no fit, intervention or model promotion.
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{
    native_geometric::learner::{
        geometric_continuation_field::NativeContinuationField,
        geometric_generate::GenerateReadCounts,
        native_bank_generate::{
            BankPin, BoundNativeBytes, NativeBankArtifacts, NativeBankGenerator, OwnedBankSegment,
            OwnedBankSource, PinnedBankSnapshot, SnapshotSourceStatus,
        },
    },
    report_output,
};
use uor_r4_integer::{
    geometric_cue_carrier::CueCarrierMetadata, geometric_prefix_transport::PrefixTransportMetadata,
    geometric_source_actions::SourceActionBinding,
    geometric_source_realizer::NativeArtifactBinding,
    geometric_vocabulary_actions::NativeVocabularyActions,
};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    #[serde(default)]
    endpoint_kind: EndpointKind,
    #[serde(default)]
    frames: Option<Vec<FrameRequest>>,
    compensation_root: PathBuf,
    #[serde(default)]
    qualification_root: Option<PathBuf>,
    #[serde(default)]
    candidate_authority: Option<CandidateAuthority>,
    expected_report_sha256: String,
    expected_manifest_sha256: String,
    expected_source_metadata_sha256: String,
    expected_generate_sha256: String,
    expected_continuation_sha256: String,
    inputs: PathBuf,
    inputs_seal_root: PathBuf,
    expected_inputs_sha256: String,
    labels: PathBuf,
    expected_labels_sha256: String,
    output: PathBuf,
    maximum_cache_bytes: u64,
    maximum_report_bytes: u64,
}
#[derive(Clone, Copy, Default, Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum EndpointKind {
    #[default]
    ZeroCompensation,
    SelectedReachedU,
    SelectedReadoutIntermediate,
    UnselectedPrefixFragment,
    UnselectedPrefixTrajectory,
    UnselectedPrefixCandidate,
    SelectedOriginalTrajectorySupplement,
    SelectedOriginalCanonicalConditional,
}
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct CandidateAuthority {
    family: CandidateFamily,
    expected_prefix_packed_sha256: String,
    expected_prefix_native_metadata_sha256: String,
    expected_prefix_master_sha256: String,
    expected_qualification_report_sha256: String,
    expected_qualification_manifest_sha256: String,
}
#[derive(Clone, Copy, Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum CandidateFamily {
    PrefixJointFragment,
}
fn validate_candidate_authority(
    kind: EndpointKind,
    authority: Option<&CandidateAuthority>,
) -> Result<()> {
    require(
        (kind == EndpointKind::UnselectedPrefixCandidate) == authority.is_some(),
        "typed candidate authority exclusive to generic candidate endpoint",
    )?;
    if let Some(a) = authority {
        for h in [
            &a.expected_prefix_packed_sha256,
            &a.expected_prefix_native_metadata_sha256,
            &a.expected_prefix_master_sha256,
            &a.expected_qualification_report_sha256,
            &a.expected_qualification_manifest_sha256,
        ] {
            require(
                h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()),
                "typed candidate identity malformed",
            )?;
        }
    }
    Ok(())
}
fn authenticate_candidate(
    report: &Value,
    receipt: &Value,
    authority: &CandidateAuthority,
) -> Result<()> {
    match authority.family {
        CandidateFamily::PrefixJointFragment => require(
            report["status"] == "COMPLETED"
                && report["mode"] == "prefix_joint_fragment_learning"
                && report["candidate_receipt"] == *receipt
                && receipt["step"] == 1
                && report["selected_model"] == false
                && report["finite_prefix_positive"] == true
                && report["qualified_fragment"] == true
                && report["all3_phase_targets_correct"] == true
                && report["all_original380_preserved"] == true
                && report["prefix_backward_calls"] == 20
                && report["candidate_native_steps"] == 383,
            "typed joint Prefix candidate authority differs",
        ),
    }
}
fn configured_qualification_pins(c: &Config) -> Result<(&str, &str)> {
    if c.endpoint_kind == EndpointKind::UnselectedPrefixCandidate {
        let a = c
            .candidate_authority
            .as_ref()
            .ok_or_else(|| bad("typed candidate authority absent"))?;
        Ok((
            &a.expected_qualification_report_sha256,
            &a.expected_qualification_manifest_sha256,
        ))
    } else {
        Ok(qualification_pins(c.endpoint_kind))
    }
}
#[derive(Clone, Copy, Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum FrameRole {
    LegacyEntry,
    FactualFailure,
    SourceControl,
    GrammarFailure,
    GrammarControl,
}
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct FrameRequest {
    input_index: usize,
    position: usize,
    expected_id: String,
    #[serde(default)]
    expected_saved_row_sha256: Option<String>,
    expected_actual_prefix_ids: Vec<u32>,
    role: FrameRole,
}
fn is_prefix_candidate(kind: EndpointKind) -> bool {
    matches!(
        kind,
        EndpointKind::UnselectedPrefixFragment
            | EndpointKind::UnselectedPrefixTrajectory
            | EndpointKind::UnselectedPrefixCandidate
    )
}
fn qualification_pins(kind: EndpointKind) -> (&'static str, &'static str) {
    if kind == EndpointKind::UnselectedPrefixTrajectory {
        (
            "8a4dbd8c13488df2397a8e01a6af885851ca87d8d0edc62cb18e7257b5b5732d",
            "ec0235c3e357c4dd5bbdf609602436aa8627e8e8192ca40cd515120ccc583379",
        )
    } else {
        (
            "d120dd6948e440abadf66ac47c0e5e01f3a90069d5691f49159fa7100bae89c9",
            "3ddfc9d701b38da86d933ba2e50459940d6b7a2e7f11d0c09305d360fd6a5f15",
        )
    }
}
fn qualification_row<'a>(q: &'a Value, id: &str) -> Result<&'a Value> {
    let mut matches = q["evaluation"]["rows"]
        .as_array()
        .ok_or_else(|| bad("qualification rows absent"))?
        .iter()
        .filter(|r| r["id"] == id);
    let row = matches.next().ok_or_else(|| bad("qualified row absent"))?;
    require(matches.next().is_none(), "duplicate qualified row identity")?;
    Ok(row)
}
fn safe_row_leaf(row: &Value) -> Result<&str> {
    let leaf = text(&row["row_file"])?;
    require(
        Path::new(leaf).components().count() == 1
            && matches!(
                Path::new(leaf).components().next(),
                Some(std::path::Component::Normal(_))
            ),
        "qualification row leaf invalid",
    )?;
    Ok(leaf)
}
// This function accesses canonical labels only AFTER every target-free capture.
fn authorize_first_divergence(saved: &Value, request: &FrameRequest) -> Result<Value> {
    let actual: Vec<u32> = serde_json::from_value(saved["generated_ids"].clone())?;
    let targets: Vec<u32> =
        serde_json::from_value(saved["canonical_target_ids_labels_only"].clone())?;
    let first = actual
        .iter()
        .zip(&targets)
        .position(|(a, t)| a != t)
        .ok_or_else(|| bad("requested row has no actual token divergence"))?;
    require(
        first == request.position
            && saved["id"] == request.expected_id
            && actual.get(..first) == Some(request.expected_actual_prefix_ids.as_slice()),
        "postcapture requested frame is not first actual divergence",
    )?;
    Ok(
        json!({"position":first,"actual_prefix_ids":request.expected_actual_prefix_ids,
        "actual_token":actual[first],"canonical_target_label_only":targets[first],
        "derived_after_all_captures":true,"scope":"offline canonical-token boundary; typed wholeanswer qualification remains distinct"}),
    )
}
fn validate_frames(frames: &[FrameRequest], kind: EndpointKind) -> Result<()> {
    let mut seen = std::collections::BTreeSet::new();
    for f in frames {
        require(
            f.input_index < 512
                && f.position < 32
                && f.position == f.expected_actual_prefix_ids.len()
                && !f.expected_id.is_empty()
                && !f.expected_actual_prefix_ids.contains(&1)
                && seen.insert((f.input_index, f.position)),
            "illegal/duplicate frame request",
        )?;
    }
    if kind == EndpointKind::SelectedOriginalCanonicalConditional {
        require(
            frames.len() == 1
                && frames[0]
                    .expected_saved_row_sha256
                    .as_ref()
                    .is_some_and(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit())),
            "conditional training witness requires one pinned generic frame",
        )?;
    } else if matches!(
        kind,
        EndpointKind::UnselectedPrefixTrajectory | EndpointKind::UnselectedPrefixCandidate
    ) {
        require(
            frames.len() == 1,
            "trajectory first-divergence capture requires exactly one frame",
        )?;
        for f in frames {
            require(
                f.role == FrameRole::FactualFailure
                    && f.expected_saved_row_sha256
                        .as_ref()
                        .is_some_and(|h| h.len() == 64 && h.bytes().all(|x| x.is_ascii_hexdigit())),
                "trajectory prospective actual frame hash/role differs",
            )?;
        }
    } else if kind == EndpointKind::SelectedReachedU {
        let expected = [
            (97, 3),
            (156, 3),
            (151, 3),
            (245, 3),
            (392, 3),
            (0, 3),
            (1, 3),
            (4, 3),
            (5, 3),
            (8, 3),
            (9, 3),
            (12, 3),
            (13, 3),
            (399, 2),
            (5, 2),
            (13, 2),
        ]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
        require(
            frames.len() == 16 && seen == expected,
            "prospective16frame cohort differs",
        )?;
        for f in frames {
            require(
                f.expected_saved_row_sha256
                    .as_ref()
                    .is_some_and(|h| h.len() == 64 && h.bytes().all(|x| x.is_ascii_hexdigit())),
                "selectedframe expected rowhash absent/invalid",
            )?;
            let role = if [97, 156, 151, 245, 392].contains(&f.input_index) {
                FrameRole::FactualFailure
            } else if f.position == 3 {
                FrameRole::SourceControl
            } else if f.input_index == 399 {
                FrameRole::GrammarFailure
            } else {
                FrameRole::GrammarControl
            };
            require(f.role == role, "prospective frame group differs")?;
        }
    } else if kind == EndpointKind::SelectedReadoutIntermediate {
        let expected = [245usize, 0, 1, 4, 5, 8, 9, 12, 13]
            .into_iter()
            .flat_map(|index| {
                [3usize, 4]
                    .into_iter()
                    .map(move |position| (index, position))
            })
            .collect::<std::collections::BTreeSet<_>>();
        require(
            frames.len() == 18 && seen == expected,
            "prospective18word-boundary cohort differs",
        )?;
        for f in frames {
            require(
                f.expected_saved_row_sha256
                    .as_ref()
                    .is_some_and(|h| h.len() == 64 && h.bytes().all(|x| x.is_ascii_hexdigit())),
                "intermediate frame row hash absent/invalid",
            )?;
            require(
                f.role
                    == if f.input_index == 245 {
                        FrameRole::FactualFailure
                    } else {
                        FrameRole::SourceControl
                    },
                "word-boundary cohort role differs",
            )?;
            if f.input_index == 245 {
                let prefix = if f.position == 3 {
                    vec![617, 2097, 315]
                } else {
                    vec![617, 2097, 315, 1057]
                };
                require(
                    f.expected_actual_prefix_ids == prefix,
                    "recovered factual actual word-boundary prefix differs",
                )?;
            }
        }
    } else if kind == EndpointKind::SelectedOriginalTrajectorySupplement {
        require(
            frames.len() == 2 && seen == [(3, 1), (455, 0)].into_iter().collect(),
            "prospective original trajectory supplement differs",
        )?;
        for f in frames {
            let (prefix, digest) = if f.input_index == 3 {
                (
                    vec![617],
                    "5d527e1e881f3b5937be974309eb6b984099356774f7e73120250b6d6a0cd897",
                )
            } else {
                (
                    vec![],
                    "871f1265ccd2fa58d250e80bbd348391e0be2aaa73ef2e59375accbe481e613b",
                )
            };
            require(
                f.expected_actual_prefix_ids == prefix
                    && f.expected_saved_row_sha256.as_deref() == Some(digest)
                    && f.role == FrameRole::SourceControl,
                "original trajectory supplement prefix/hash/role differs",
            )?;
        }
    } else if kind == EndpointKind::UnselectedPrefixFragment {
        require(
            frames.len() == 2 && seen == [(245, 2), (13, 7)].into_iter().collect(),
            "prospective first-divergence cohort differs",
        )?;
        for f in frames {
            let expected = if f.input_index == 245 {
                vec![617, 2097]
            } else {
                vec![617, 2097, 315, 261, 92, 607, 770]
            };
            require(
                f.expected_actual_prefix_ids == expected
                    && f.expected_saved_row_sha256
                        .as_ref()
                        .is_some_and(|h| h.len() == 64 && h.bytes().all(|x| x.is_ascii_hexdigit())),
                "first-divergence actual prefix/hash differs",
            )?;
            require(
                f.role == FrameRole::FactualFailure,
                "first-divergence role differs",
            )?;
        }
    } else {
        require(
            frames.len() == 3 && seen == [(399, 1), (5, 1), (13, 1)].into_iter().collect(),
            "legacy cohort differs",
        )?;
    }
    Ok(())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Panel {
    schema: String,
    cases: Vec<Packet>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Packet {
    id: String,
    segments: Vec<Segment>,
    query_ids: Vec<u32>,
    actual_prefix_ids: Vec<u32>,
}
#[derive(Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Segment {
    Source {
        event: u64,
        record: u64,
        commit: u64,
        scope: String,
        entity: Vec<u32>,
        relation: u32,
        view: u32,
        original_source_ids: Vec<u32>,
    },
    Context {
        event: u64,
        role: u32,
        token_ids: Vec<u32>,
    },
}
fn bad(s: &str) -> Box<dyn std::error::Error> {
    std::io::Error::other(s).into()
}
fn require(ok: bool, s: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(bad(s))
    }
}
fn bytes(p: &Path) -> Result<Vec<u8>> {
    Ok(fs::read(p)?)
}
fn hash(b: &[u8]) -> String {
    hex::encode(Sha256::digest(b))
}
fn file_hash(p: &Path) -> Result<String> {
    Ok(hash(&bytes(p)?))
}
fn read(p: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&bytes(p)?)?)
}
fn text(v: &Value) -> Result<&str> {
    v.as_str().ok_or_else(|| bad("required string absent"))
}
fn write(c: &Config, name: &str, value: &Value, written: &mut u64) -> Result<()> {
    let b = serde_json::to_vec(value)?;
    *written = written
        .checked_add(b.len() as u64)
        .ok_or_else(|| bad("report byte overflow"))?;
    require(
        *written <= c.maximum_report_bytes,
        "report byte cap exhausted",
    )?;
    fs::write(c.output.join(name), b)?;
    Ok(())
}
fn snapshot(p: &Packet) -> Result<PinnedBankSnapshot> {
    require(
        p.actual_prefix_ids.is_empty(),
        "supplied input prefix excluded",
    )?;
    let mut scope = None;
    let mut commit = 0;
    for segment in &p.segments {
        if let Segment::Source {
            scope: s,
            commit: c,
            ..
        } = segment
        {
            require(
                !s.is_empty() && scope.map_or(true, |old: &String| old == s),
                "mixed/empty bank scope",
            )?;
            scope = Some(s);
            commit = commit.max(*c);
        }
    }
    let scope = scope.ok_or_else(|| bad("required Source bank absent"))?;
    Ok(PinnedBankSnapshot {
        pin: BankPin {
            lineage: 0,
            commit,
            scope: scope.as_bytes().to_vec(),
        },
        query_ids: p.query_ids.clone(),
        segments: p
            .segments
            .iter()
            .map(|s| match s {
                Segment::Source {
                    event,
                    record,
                    commit,
                    scope,
                    entity,
                    relation,
                    view,
                    original_source_ids,
                } => OwnedBankSegment::Source(OwnedBankSource {
                    event: *event,
                    record: *record,
                    commit: *commit,
                    scope: scope.as_bytes().to_vec(),
                    entity: entity.clone(),
                    relation: *relation,
                    view: *view,
                    status: SnapshotSourceStatus::Found,
                    original_token_ids: original_source_ids.clone(),
                }),
                Segment::Context {
                    event,
                    role,
                    token_ids,
                } => OwnedBankSegment::Context {
                    event: *event,
                    role: *role,
                    token_ids: token_ids.clone(),
                },
            })
            .collect(),
    })
}
fn admit_paths(c: &mut Config) -> Result<()> {
    if let Some(frames) = &c.frames {
        validate_frames(frames, c.endpoint_kind)?;
    } else {
        require(
            c.endpoint_kind == EndpointKind::ZeroCompensation,
            "nonlegacy endpoint requires explicit prospective frames before model loading",
        )?;
    }
    for p in [
        &c.compensation_root,
        &c.inputs,
        &c.inputs_seal_root,
        &c.labels,
        &c.output,
    ] {
        require(
            p.is_absolute()
                && !p
                    .components()
                    .any(|x| matches!(x, std::path::Component::ParentDir)),
            "absolute paths without traversal required",
        )?;
    }
    validate_candidate_authority(c.endpoint_kind, c.candidate_authority.as_ref())?;
    require(
        is_prefix_candidate(c.endpoint_kind) == c.qualification_root.is_some(),
        "qualification authority exclusive to Prefix endpoint",
    )?;
    if let Some(root) = &c.qualification_root {
        require(
            root.is_absolute()
                && !root
                    .components()
                    .any(|x| matches!(x, std::path::Component::ParentDir)),
            "qualification path invalid",
        )?;
    }
    let (cache_cap, report_cap) = if matches!(
        c.endpoint_kind,
        EndpointKind::UnselectedPrefixTrajectory
            | EndpointKind::UnselectedPrefixCandidate
            | EndpointKind::SelectedOriginalCanonicalConditional
    ) {
        (64 * 1024 * 1024, 64 * 1024 * 1024)
    } else if c.endpoint_kind == EndpointKind::SelectedReadoutIntermediate {
        (256 * 1024 * 1024, 128 * 1024 * 1024)
    } else {
        (128 * 1024 * 1024, 128 * 1024 * 1024)
    };
    require(
        c.maximum_cache_bytes > 0
            && c.maximum_cache_bytes <= cache_cap
            && c.maximum_report_bytes >= 1024 * 1024
            && c.maximum_report_bytes <= report_cap,
        "diagnostic caps exceed admitted endpoint resources",
    )?;
    c.compensation_root = fs::canonicalize(&c.compensation_root)?;
    c.inputs = fs::canonicalize(&c.inputs)?;
    c.inputs_seal_root = fs::canonicalize(&c.inputs_seal_root)?;
    c.labels = fs::canonicalize(&c.labels)?;
    c.output = output_support::prospective_output(&c.output)?;
    require(
        c.inputs.starts_with(&c.inputs_seal_root) && c.labels.starts_with(&c.inputs_seal_root),
        "inputs/labels outside declared seal",
    )?;
    for p in [&c.compensation_root, &c.inputs_seal_root] {
        require(
            !c.output.starts_with(p) && !p.starts_with(&c.output),
            "output overlaps sealed authority",
        )?;
    }
    if let Some(root) = &mut c.qualification_root {
        *root = fs::canonicalize(&*root)?;
        require(
            !c.output.starts_with(&*root) && !root.starts_with(&c.output),
            "output overlaps qualification seal",
        )?;
    }
    Ok(())
}
fn authenticate_actual_prefix(saved: &Value, prefix: &[u32], position: usize) -> Result<()> {
    let generated: Vec<u32> = serde_json::from_value(saved["generated_ids"].clone())?;
    let eos = generated.last() == Some(&1);
    require(
        saved["eos"].as_bool() == Some(eos)
            && !generated[..generated.len().saturating_sub(usize::from(eos))].contains(&1),
        "savedEOSflag/placement differs",
    )?;
    let steps = saved["generation"]
        .as_array()
        .ok_or_else(|| bad("saved actual trajectory absent"))?;
    require(
        position == prefix.len()
            && position < generated.len()
            && steps.len() == generated.len()
            && generated.get(..position) == Some(prefix)
            && !prefix.contains(&1),
        "requested frame absent/pastEOS/notactualprefix",
    )?;
    for (i, step) in steps.iter().enumerate().take(position + 1) {
        require(
            step["actual_prefix_ids"] == json!(&generated[..i])
                && step["pool"]["summary"]["chosen_token_id"] == generated[i],
            "saved winner/prefix chain differs",
        )?;
    }
    Ok(())
}
fn authenticate_reached_prefix(saved: &Value, prefix: &[u32], position: usize) -> Result<()> {
    authenticate_actual_prefix(saved, prefix, position)?;
    require(
        saved["canonical"][position]["native"]["pool"]["summary"]
            == saved["generation"][position]["pool"]["summary"],
        "saved actual frame is not canonical reached prefix",
    )
}
fn check_saved(
    step: &uor_r4_core::native_geometric::learner::native_bank_generate::NativeBankGenerateStep,
    saved: &Value,
    position: usize,
    actual_only: bool,
) -> Result<()> {
    let actual = &saved["generation"][position];
    let canonical = &saved["canonical"][position]["native"];
    authenticate_actual_prefix(saved, &step.actual_prefix_ids, position)?;
    if actual_only {
        require(
            actual["pool"]["summary"] == serde_json::to_value(&step.actions.summary)?,
            "actual-only full pool parity differs",
        )?;
        let u = step
            .continuation
            .as_ref()
            .ok_or_else(|| bad("actual-only U missing"))?;
        return require(
            actual["continuation"]["state_codes"]
                == json!(u.state_codes.iter().map(|x| x.index()).collect::<Vec<_>>())
                && actual["continuation"]["delta_scores_q24_sha256"]
                    == hash(&serde_json::to_vec(&u.delta_scores_q24)?),
            "actual-only U parity differs",
        );
    }
    authenticate_reached_prefix(saved, &step.actual_prefix_ids, position)?;
    require(
        actual["pool"]["summary"] == serde_json::to_value(&step.actions.summary)?
            && canonical["generate_raw_scores_sha256"]
                == hash(&serde_json::to_vec(&step.generate_raw_scores_q24)?)
            && canonical["post_state_codes"]
                == json!(step
                    .post_state
                    .iter()
                    .map(|x| x.index())
                    .collect::<Vec<_>>())
            && canonical["copy_token_ids"] == json!(step.copy_token_ids),
        "saved native actual position parity differs",
    )?;
    let u = step
        .continuation
        .as_ref()
        .ok_or_else(|| bad("U witness missing"))?;
    require(
        canonical["continuation"] == actual["continuation"]
            && actual["continuation"]["state_codes"]
                == json!(u.state_codes.iter().map(|x| x.index()).collect::<Vec<_>>())
            && actual["continuation"]["delta_scores_q24_sha256"]
                == hash(&serde_json::to_vec(&u.delta_scores_q24)?),
        "actual U state/delta digest parity differs",
    )?;
    Ok(())
}
fn check_saved_canonical(
    step: &uor_r4_core::native_geometric::learner::native_bank_generate::NativeBankGenerateStep,
    saved: &Value,
    position: usize,
) -> Result<()> {
    let n = &saved["canonical"][position]["native"];
    let u = step
        .continuation
        .as_ref()
        .ok_or_else(|| bad("conditional U missing"))?;
    require(
        n["pool"]["summary"] == serde_json::to_value(&step.actions.summary)?
            && n["generate_raw_scores_sha256"]
                == hash(&serde_json::to_vec(&step.generate_raw_scores_q24)?)
            && n["post_state_codes"]
                == json!(step
                    .post_state
                    .iter()
                    .map(|x| x.index())
                    .collect::<Vec<_>>())
            && n["copy_token_ids"] == json!(step.copy_token_ids)
            && n["continuation"]["state_codes"]
                == json!(u.state_codes.iter().map(|x| x.index()).collect::<Vec<_>>())
            && n["continuation"]["delta_scores_q24_sha256"]
                == hash(&serde_json::to_vec(&u.delta_scores_q24)?)
            && n["continuation"]["query_tokens"] == u.query_tokens
            && n["continuation"]["actual_prefix_tokens"] == u.actual_prefix_tokens
            && n["continuation"]["encoding_coefficient_reads"] == u.encoding_coefficient_reads
            && n["continuation"]["field_counts"] == serde_json::to_value(&u.counts)?
            && n["continuation"]["delta_scores"] == u.delta_scores_q24.len()
            && n["continuation"]["minimum_delta_q24"] == json!(u.delta_scores_q24.iter().min())
            && n["continuation"]["maximum_delta_q24"] == json!(u.delta_scores_q24.iter().max()),
        "conditional original native parity differs",
    )
}
fn report_schema(kind: EndpointKind) -> &'static str {
    if kind == EndpointKind::SelectedReadoutIntermediate
        || kind == EndpointKind::SelectedOriginalTrajectorySupplement
        || kind == EndpointKind::SelectedOriginalCanonicalConditional
        || is_prefix_candidate(kind)
    {
        "uor-r4.native-reached-prefix-attribution/3"
    } else {
        "uor-r4.native-reached-prefix-attribution/2"
    }
}
fn compact_factor_layout() -> Value {
    json!({"format":"token_ordered_tuples/1","tokens":4096,"generate_columns":["relative_codes","logical_factor_keys","unary_codes","pair_codes","bias_code","total_q24","u_total_q24"],"u_columns":["relative_codes","coefficient_codes","total_q24"],"generate_score_shift":20,"u_score_shift":22,"prototype_codes":"bound immutable Generate artifact; token row index"})
}
fn authenticate_endpoint(report: &Value, receipt: &Value, kind: EndpointKind) -> Result<()> {
    require(
        kind != EndpointKind::UnselectedPrefixCandidate,
        "generic candidate requires typed family authority",
    )?;
    if is_prefix_candidate(kind) {
        return require(
            report["status"] == "COMPLETED"
                && report["mode"]
                    == if kind == EndpointKind::UnselectedPrefixTrajectory {
                        "prefix_trajectory_learning"
                    } else {
                        "prefix_fragment_learning"
                    }
                && (kind != EndpointKind::UnselectedPrefixTrajectory
                    || (report["all_original380_preserved"] == true
                        && report["protected_population"] == 380
                        && report["new_prefix_gradients"] == 0
                        && report["selected_model"] == false))
                && report["candidate_receipt"] == *receipt
                && receipt["step"] == 1
                && report["finite_prefix_positive"] == true
                && report["qualified_fragment"] == true,
            "unselected Prefix candidate construction authority differs",
        );
    }
    require(
        report["status"] == "COMPLETED"
            && report["final_receipt"] == *receipt
            && receipt["step"] == 1,
        "endpoint complete receipt differs",
    )?;
    if kind == EndpointKind::SelectedReadoutIntermediate
        || kind == EndpointKind::SelectedOriginalTrajectorySupplement
        || kind == EndpointKind::SelectedOriginalCanonicalConditional
    {
        require(
            report["mode"] == "readout_intermediate_candidate"
                && report["selected_model"] == true
                && report["candidate_artifact_status"] == "QUALIFIED_NATIVE_GATE_AND_RETENTION"
                && report["useful_candidate"] == false
                && report["reply_qualification"]["status"] == "PASSED"
                && report["reply_qualification"]["retained_original8"] == true
                && report["reply_qualification"]["candidate512"] == "COMPLETED",
            "recovered intermediate qualification differs",
        )?;
    } else {
        require(
            report["mode"]
                == if kind == EndpointKind::ZeroCompensation {
                    "prototype_compensation"
                } else {
                    "reached_u"
                }
                && report["native_code_proposals"]["winner"] == 0,
            "historical selected endpoint differs",
        )?;
    }
    Ok(())
}
fn run(c: &Config, written: &mut u64) -> Result<Value> {
    let clock = Instant::now();
    report_output::verify(&c.compensation_root)?;
    report_output::verify(&c.inputs_seal_root)?;
    require(
        file_hash(&c.compensation_root.join("report.json"))? == c.expected_report_sha256
            && file_hash(&c.compensation_root.join("manifest.json"))? == c.expected_manifest_sha256
            && file_hash(&c.inputs)? == c.expected_inputs_sha256
            && file_hash(&c.labels)? == c.expected_labels_sha256,
        "sealed authority pins differ",
    )?;
    let report = read(&c.compensation_root.join("report.json"))?;
    let cp = c.compensation_root.join("checkpoint-0001");
    let receipt = read(&cp.join("receipt.json"))?;
    if let Some(a) = &c.candidate_authority {
        authenticate_candidate(&report, &receipt, a)?;
    } else {
        authenticate_endpoint(&report, &receipt, c.endpoint_kind)?;
    }
    let binding: NativeArtifactBinding = serde_json::from_value(receipt["parent"].clone())?;
    require(
        binding.metadata_sha256 == c.expected_source_metadata_sha256
            && c.expected_source_metadata_sha256
                == "9f0b272e7852a47bbbad0f549e8157a44ff3212af86905907501ec11f3e7989b"
            && c.expected_generate_sha256
                == "4248245471db609b1fc19482e8f180380b292c5832fc90c81ce69947bc4b7737"
            && c.expected_continuation_sha256
                == if c.endpoint_kind == EndpointKind::ZeroCompensation {
                    "a42cc8d9a9d04bdcddb9f1a513128f66361611556f0e6d768de37da0bc9d1030"
                } else {
                    "82ae9daeb402b288e64492d5b299110b36849907019c952609a1cae6612673ee"
                },
        "fixed successor artifact pins differ",
    )?;
    if c.endpoint_kind == EndpointKind::SelectedReachedU {
        require(
            c.expected_report_sha256
                == "2a9f967a955c6dc40b422c94f1b6d1d54f2511a24fb601af97218d49440b9923"
                && c.expected_manifest_sha256
                    == "6cbfabf807427f3baa2bfcfda7187e1b00f3a205ec9466b30a7437a410baad94",
            "selectedUreport/seal differs",
        )?;
    }
    if c.endpoint_kind == EndpointKind::SelectedReadoutIntermediate
        || c.endpoint_kind == EndpointKind::SelectedOriginalTrajectorySupplement
        || c.endpoint_kind == EndpointKind::SelectedOriginalCanonicalConditional
    {
        require(
            c.expected_report_sha256
                == "c9b9fe10b6fbb4332cf919a5df7ba31403ad3d51ac6f877d94daeabd99672bee"
                && c.expected_manifest_sha256
                    == "de90ba0ba2809ed37ca868ef2bcf94b6ceb8b176a60b86ae88801876dafbd0e5",
            "recovered intermediate report/seal differs",
        )?;
    }
    let qualification = if is_prefix_candidate(c.endpoint_kind) {
        let (report_pin, seal_pin, packed_pin, metadata_pin, master_pin) =
            if let Some(a) = &c.candidate_authority {
                (
                    c.expected_report_sha256.as_str(),
                    c.expected_manifest_sha256.as_str(),
                    a.expected_prefix_packed_sha256.as_str(),
                    a.expected_prefix_native_metadata_sha256.as_str(),
                    a.expected_prefix_master_sha256.as_str(),
                )
            } else if c.endpoint_kind == EndpointKind::UnselectedPrefixTrajectory {
                (
                    "77227984640fc55e29067bb6dae935068c7a9faf2bf8c8f0b296b5f0b9c05355",
                    "c1309528bcf45f7ceac54643e35657eedac679ff9b682deec3b93cffcf483442",
                    "9cd7eff1af78cdc9f9ae032cf26b46630208e027fcdf5b76a16f30bd7606ff96",
                    "31c31b31da847ce79d913d713edb36bb45b78b0cebf727a11d0da54fd05d7116",
                    "a2d0ec5b2b6957afba4c311d2ce2eeb8c4836fe00c20c7c9551f503c2aa28d30",
                )
            } else {
                (
                    "71301b77d9d4606dda1501385e43f8e114c4e72af9a502b799f363b604b14cfe",
                    "12c1412d049724e6dd7ba1e1cd8fc0383be65f7a3c9f43251a206fc23ba76d32",
                    "a6ea6299cec8b5739b2e20002488989f932392054ec28c24dde5057eb2de2b86",
                    "d8820c74192cee59c397c6708bbab2fdf69105d8623f4fad4146638cc81dea68",
                    "900f1e23d31369fc0a7f78d6db23cf7777f95226b06ae4ebce6685111075ad36",
                )
            };
        require(
            c.expected_report_sha256 == report_pin && c.expected_manifest_sha256 == seal_pin,
            "Prefix learning report/seal differs",
        )?;
        require(
            file_hash(&cp.join("prefix/prefix-q4.bin"))? == packed_pin
                && file_hash(&cp.join("prefix/native-metadata.json"))? == metadata_pin
                && file_hash(&cp.join("prefix/prefix-source-f32.bin"))? == master_pin,
            "Prefix candidate payload/metadata/master identity differs",
        )?;
        if c.endpoint_kind == EndpointKind::UnselectedPrefixCandidate {
            require(
                file_hash(&cp.join("prefix-source/prefix.coefficients.f32le"))? == master_pin,
                "typed candidate source master identity differs",
            )?;
        }
        let root = c
            .qualification_root
            .as_ref()
            .ok_or_else(|| bad("qualification root absent"))?;
        report_output::verify(root)?;
        let (q_report, q_manifest) = configured_qualification_pins(c)?;
        require(
            file_hash(&root.join("report.json"))? == q_report
                && file_hash(&root.join("manifest.json"))? == q_manifest,
            "cheap actual report/seal differs",
        )?;
        let q = read(&root.join("report.json"))?;
        require(
            q["mode"] == "prefix_artifact_check"
                && q["status"] == "COMPLETED"
                && q["candidate_report_sha256"] == c.expected_report_sha256
                && q["candidate_manifest_sha256"] == c.expected_manifest_sha256
                && q["qualification_positive"] == false
                && (!matches!(
                    c.endpoint_kind,
                    EndpointKind::UnselectedPrefixTrajectory
                        | EndpointKind::UnselectedPrefixCandidate
                ) || (q["selected_model"] == false
                    && q["retained_original8"] == true
                    && q["actual_ownprefix_rows"] == 9)),
            "unselected cheap qualification authority differs",
        )?;
        Some(q)
    } else {
        None
    };
    if matches!(
        c.endpoint_kind,
        EndpointKind::SelectedOriginalTrajectorySupplement
            | EndpointKind::SelectedOriginalCanonicalConditional
    ) {
        require(
            file_hash(&cp.join("prefix/prefix-q4.bin"))?
                == "c2e8ec992996055450f77237ec64730c28b2e7cd53f9ae49cdb7a28128236d0a"
                && file_hash(&cp.join("prefix/prefix-source-f32.bin"))?
                    == "1e47a7dff9134d393043a2313a7da50ea234d1b1ae7888d895e3604855ac0fe3",
            "original trajectory Prefix authority differs",
        )?;
    }
    let gen = bytes(&cp.join("generate.bin"))?;
    let bridge = bytes(&cp.join("read-state-bridge-categorical.bin"))?;
    let exp = bytes(&cp.join("native/consumer/exp-q31.bin"))?;
    let exp_hash = hash(&exp);
    let cue = bytes(&cp.join("cue/cue-q4.bin"))?;
    let prefix = bytes(&cp.join("prefix/prefix-q4.bin"))?;
    let joint = if cp.join("cue/cue-joint-q4.bin").exists() {
        Some(bytes(&cp.join("cue/cue-joint-q4.bin"))?)
    } else {
        None
    };
    let cue_metadata: CueCarrierMetadata =
        serde_json::from_value(read(&cp.join("cue/native-metadata.json"))?)?;
    let prefix_metadata: PrefixTransportMetadata =
        serde_json::from_value(read(&cp.join("prefix/native-metadata.json"))?)?;
    require(
        hash(&gen) == c.expected_generate_sha256
            && receipt["generate_sha256"] == c.expected_generate_sha256,
        "Generate endpoint pin differs",
    )?;
    let mut generator = NativeBankGenerator::load(NativeBankArtifacts {
        native_directory: &cp.join("native"),
        source_binding: &binding,
        generate: BoundNativeBytes {
            bytes: &gen,
            sha256: &c.expected_generate_sha256,
        },
        bridge: Some(BoundNativeBytes {
            bytes: &bridge,
            sha256: text(&receipt["categorical_sha256"])?,
        }),
        cue_packed: &cue,
        cue_joint_packed: joint.as_deref(),
        cue_metadata: &cue_metadata,
        prefix_packed: &prefix,
        prefix_metadata: &prefix_metadata,
        exp: BoundNativeBytes {
            bytes: &exp,
            sha256: &exp_hash,
        },
    })?;
    require(
        generator.generate_model().lanes() == 8 && generator.generate_model().vocab_size() == 4096,
        "retained Generate shape differs",
    )?;
    let u = bytes(&cp.join("continuation-field.bin"))?;
    require(
        hash(&u) == c.expected_continuation_sha256
            && receipt["continuation_sha256"] == c.expected_continuation_sha256,
        "U endpoint pin differs",
    )?;
    let field = NativeContinuationField::from_bytes(&u, &binding, generator.generate_model())?;
    require(field.applies_to_copy(), "shared-actionv2 field required")?;
    if c.endpoint_kind == EndpointKind::ZeroCompensation {
        for lane in 0..8 {
            for relative in 0..120 {
                require(
                    field.coefficient_unary(lane, relative)? == 0,
                    "prototype-dependent U is nonzero",
                )?;
            }
        }
    }
    generator = generator.with_continuation_field(BoundNativeBytes {
        bytes: &u,
        sha256: &c.expected_continuation_sha256,
    })?;
    let action_binding = SourceActionBinding::new(&bytes(&cp.join("native/tokenizer.json"))?)?;
    let mut reducer = NativeVocabularyActions::new(action_binding, &exp)?;
    require(
        reducer.legal_token_ids().iter().copied().eq(0u32..4096),
        "full legal vocabulary differs",
    )?;
    let panel: Panel = serde_json::from_slice(&bytes(&c.inputs)?)?;
    require(
        panel.schema == "uor-r4.native-source-bank-probe-input/1" && panel.cases.len() == 512,
        "frozen input panel differs",
    )?;
    // Selection is fixed by the prospective card, not by runtime scores or labels.
    let frames = c.frames.clone().unwrap_or_else(|| {
        [399usize, 5, 13]
            .into_iter()
            .map(|i| FrameRequest {
                input_index: i,
                position: 1,
                expected_id: panel.cases[i].id.clone(),
                expected_saved_row_sha256: None,
                expected_actual_prefix_ids: vec![617],
                role: FrameRole::LegacyEntry,
            })
            .collect()
    });
    validate_frames(&frames, c.endpoint_kind)?;
    let mut summaries = Vec::new();
    for request in &frames {
        let index = request.input_index;
        let position = request.position;
        let packet = &panel.cases[index];
        let row = if let Some(q) = &qualification {
            qualification_row(q, &packet.id)?
        } else {
            &report["final_evaluation"]["rows"][index]
        };
        let saved_root = c
            .qualification_root
            .as_ref()
            .unwrap_or(&c.compensation_root);
        require(
            row["id"] == packet.id && packet.id == request.expected_id,
            "report/input/request case identity differs",
        )?;
        let row_name = text(&row["row_file"])?;
        require(
            if matches!(
                c.endpoint_kind,
                EndpointKind::UnselectedPrefixTrajectory | EndpointKind::UnselectedPrefixCandidate
            ) {
                safe_row_leaf(row)? == row_name
            } else if qualification.is_some() {
                row_name
                    == if index == 245 {
                        "cheap-ownprefix-row-0008.json"
                    } else {
                        "cheap-ownprefix-row-0007.json"
                    }
            } else {
                row_name == format!("development-0001-row-{index:04}.json")
            },
            "saved row path differs",
        )?;
        let saved_bytes = bytes(&saved_root.join(row_name))?;
        require(
            hash(&saved_bytes) == text(&row["row_sha256"])?
                && request
                    .expected_saved_row_sha256
                    .as_ref()
                    .map_or(true, |h| h == &hash(&saved_bytes)),
            "saved actual row digest differs",
        )?;
        let saved: Value = serde_json::from_slice(&saved_bytes)?;
        require(
            saved["id"] == packet.id
                && (c.endpoint_kind == EndpointKind::UnselectedPrefixFragment
                    || saved["generated_ids"] == row["generated_ids"])
                && saved["complete"] == row["complete"]
                && saved["continuation_sha256"] == c.expected_continuation_sha256,
            "saved actual row/report binding differs",
        )?;
        if qualification.is_some() {
            authenticate_actual_prefix(&saved, &request.expected_actual_prefix_ids, position)?;
        } else if c.endpoint_kind != EndpointKind::SelectedOriginalCanonicalConditional {
            authenticate_reached_prefix(&saved, &request.expected_actual_prefix_ids, position)?;
        }
        let bank = generator.admit_bank(snapshot(packet)?)?;
        let step = generator.step(&bank, &request.expected_actual_prefix_ids)?;
        if c.endpoint_kind == EndpointKind::SelectedOriginalCanonicalConditional {
            check_saved_canonical(&step, &saved, position)?;
        } else {
            check_saved(
                &step,
                &saved,
                position,
                is_prefix_candidate(c.endpoint_kind),
            )?;
        }
        let replay = reducer.reduce_trace(
            &step.generate_raw_scores_q24,
            &step.copy_token_ids,
            &step.copy_raw_scores_q24,
        )?;
        require(
            replay == step.actions,
            "full production reducer replay differs",
        )?;
        let trace = step
            .bank_trace
            .as_ref()
            .ok_or_else(|| bad("bank trace absent"))?;
        let bridge = step
            .bridge
            .as_ref()
            .ok_or_else(|| bad("bridge witness absent"))?;
        let continuation = step
            .continuation
            .as_ref()
            .ok_or_else(|| bad("continuation witness absent"))?;
        let (base_generate, base_copy) = remove_shared_u(
            &step.generate_raw_scores_q24,
            &step.copy_token_ids,
            &step.copy_raw_scores_q24,
            &continuation.delta_scores_q24,
        )?;
        let mut summed_copy = vec![0i64; step.copy_token_ids.len()];
        for head in &trace.cue_bank.bank.heads {
            require(
                head.scores_q24.len() == summed_copy.len(),
                "physical Copy head shape differs",
            )?;
            for (sum, value) in summed_copy.iter_mut().zip(&head.scores_q24) {
                *sum = sum
                    .checked_add(*value)
                    .ok_or_else(|| bad("physical Copy sum overflow"))?;
            }
        }
        require(
            summed_copy == base_copy,
            "physical Copy differs from already combined bank heads",
        )?;
        let selected = earliest_physical_max(&summed_copy)?;
        require(
            selected == bridge.selected_ordinal
                && trace.cue_bank.bank.candidates.get(selected) == Some(&bridge.selected_candidate),
            "bridge differs from earliest physical raw maximum",
        )?;
        if c.endpoint_kind == EndpointKind::ZeroCompensation {
            require(
                continuation.delta_scores_q24.iter().all(|x| *x == 0),
                "legacyUdelta notzero",
            )?;
        }
        let model = generator.generate_model();
        let mut factor_counts = GenerateReadCounts::default();
        let mut factors = Vec::with_capacity(4096);
        let mut u_factors = Vec::with_capacity(4096);
        for token in 0..4096usize {
            let mut relative = Vec::with_capacity(8);
            let mut unary = Vec::with_capacity(8);
            let mut keys = vec![0u32; 8 + model.energy().edges().len()];
            model.factor_incidence_into(&step.post_state, token, &mut keys, &mut factor_counts)?;
            for lane in 0..8usize {
                let code = model.algebra().compose(
                    model.algebra().inverse(step.post_state[lane].index())?,
                    model.prototypes()[token * 8 + lane],
                )?;
                require(
                    keys[lane] == (lane * 120 + usize::from(code)) as u32,
                    "unary logical/finite algebra incidence differs",
                )?;
                relative.push(code);
                unary.push(model.energy().get_unary(lane as u8, code)?);
            }
            let mut pairs = Vec::new();
            for (edge_index, edge) in model.energy().edges().iter().enumerate() {
                let left = relative[usize::from(edge.left)];
                let right = relative[usize::from(edge.right)];
                require(
                    keys[8 + edge_index]
                        == (960 + edge_index * 14400 + usize::from(left) * 120 + usize::from(right))
                            as u32,
                    "pair logical/finite algebra incidence differs",
                )?;
                pairs.push(model.energy().get_pair(edge_index, left, right)?);
            }
            let bias = model.token_bias(token)?;
            let total = component_sum_q24(&unary, &pairs, bias)?;
            require(
                total == base_generate[token],
                "factor decomposition does not equal complete Generate score",
            )?;
            let mut u_relative = Vec::new();
            let mut u_coefficients = Vec::new();
            for lane in 0..8 {
                let r = model.algebra().compose(
                    model
                        .algebra()
                        .inverse(continuation.state_codes[lane].index())?,
                    model.prototypes()[token * 8 + lane],
                )?;
                u_relative.push(r);
                u_coefficients.push(field.coefficient_unary(lane, r)?);
            }
            let u_total = shifted_sum(&u_coefficients, 22)?;
            require(
                u_total == continuation.delta_scores_q24[token],
                "alltokenUfactor sum differs",
            )?;
            if c.endpoint_kind == EndpointKind::SelectedReadoutIntermediate
                || c.endpoint_kind == EndpointKind::SelectedOriginalTrajectorySupplement
                || c.endpoint_kind == EndpointKind::SelectedOriginalCanonicalConditional
                || is_prefix_candidate(c.endpoint_kind)
            {
                u_factors.push(json!([u_relative, u_coefficients, u_total]));
                factors.push(json!([relative, keys, unary, pairs, bias, total, u_total]));
            } else {
                u_factors.push(json!({"token_id":token,"relative_codes":u_relative,"coefficient_codes":u_coefficients,"score_shift":22,"total_q24":u_total}));
                factors.push(json!({"token_id":token,"prototype_codes":&model.prototypes()[token*8..(token+1)*8],"relative_codes":relative,"logical_factor_keys":keys,"unary_codes":unary,"pair_codes":pairs,"bias_code":bias,"score_shift":20,"total_q24":total,"total_scope":"BASEGenerate beforeU","u_total_q24":u_total}));
            }
        }
        let (generate_only, undo_u_copy) = if qualification.is_some()
            || c.endpoint_kind == EndpointKind::SelectedOriginalTrajectorySupplement
            || c.endpoint_kind == EndpointKind::SelectedOriginalCanonicalConditional
        {
            (json!("NOT_RUN"), json!("NOT_RUN"))
        } else {
            (
                serde_json::to_value(reducer.reduce_trace(
                    &step.generate_raw_scores_q24,
                    &[],
                    &[],
                )?)?,
                serde_json::to_value(reducer.reduce_trace(
                    &step.generate_raw_scores_q24,
                    &step.copy_token_ids,
                    &base_copy,
                )?)?,
            )
        };
        let mut frame = json!({"input_index":index,"id":packet.id,"position":position,"request":request,"actual_prefix_ids":request.expected_actual_prefix_ids,"saved_row_sha256":hash(&saved_bytes),
          "capture_target_free":true,"label_access_before_capture":false,"query_ids":packet.query_ids,"snapshot_pin":{"lineage":0,"commit":packet.segments.iter().filter_map(|s|if let Segment::Source{commit,..}=s{Some(*commit)}else{None}).max(),"scope":packet.segments.iter().find_map(|s|if let Segment::Source{scope,..}=s{Some(scope)}else{None})},"bank_trace":trace,
          "bridge":{"selected_ordinal":bridge.selected_ordinal,"selected_candidate":bridge.selected_candidate,
            "state_roles":{"query_state":"complete bank final prebridge state; not query-only","source_state":"cumulative bank replay state at selected physical occurrence; not isolated source embedding","post_state":"actual Generate scorer input after categorical bridge","continuation_state":"separate query+actualprefix encoding; shared-actionU, distinct from bridge"},"selection_policy":"default earliest PHYSICAL raw Copy maximum before token alias pooling","physical_copy_sum_verified":true,"query_state":bridge.query_state.iter().map(|x|x.index()).collect::<Vec<_>>(),"source_state":bridge.source_state.iter().map(|x|x.index()).collect::<Vec<_>>(),
            "action_codes":bridge.action_codes.iter().map(|x|x.index()).collect::<Vec<_>>(),"action_scores_q24":bridge.action_scores_q24,"counts":bridge.counts},
          "post_state":step.post_state.iter().map(|x|x.index()).collect::<Vec<_>>(),
          "continuation":{"query_tokens":continuation.query_tokens,"actual_prefix_tokens":continuation.actual_prefix_tokens,"state_codes":continuation.state_codes.iter().map(|x|x.index()).collect::<Vec<_>>(),"delta_scores_q24":continuation.delta_scores_q24,"encoding_coefficient_reads":continuation.encoding_coefficient_reads,"counts":continuation.counts},
          "base_generate_q24":base_generate,"base_copy_q24":base_copy,"generate_q24":step.generate_raw_scores_q24,"generate_sha256":hash(&serde_json::to_vec(&step.generate_raw_scores_q24)?),"copy_ids":step.copy_token_ids,"copy_q24":step.copy_raw_scores_q24,
          "pool":step.actions,"generate_counts":step.generate_counts,"factor_attribution_counts":factor_counts,"declared_pair_edges":model.energy().edges(),"factors":factors,"u_factors":u_factors,
          "saved_actual":saved["generation"][position],"saved_canonical_native":saved["canonical"][position]["native"],"controls":{"generate_only":generate_only,"copy_u_removed":undo_u_copy},"control_scope":"savedvector reducer diagnostics; not servingoptions or generatedcounterfactuals","expected_record_query_role":"NOT_LOADED: separately authenticated reference joined posthoc by reader; never inferred from ID/target/bridge donor"});
        if c.endpoint_kind == EndpointKind::SelectedReadoutIntermediate
            || c.endpoint_kind == EndpointKind::SelectedOriginalTrajectorySupplement
            || c.endpoint_kind == EndpointKind::SelectedOriginalCanonicalConditional
            || is_prefix_candidate(c.endpoint_kind)
        {
            frame["schema"] = json!("uor-r4.native-reached-prefix-frame/3");
            frame["factor_layout"] = compact_factor_layout();
            frame["request_role_scope"] = json!(if c.endpoint_kind
                == EndpointKind::SelectedOriginalTrajectorySupplement
            {
                "two authenticated original correct-prefix supplements; labels attached separately after both captures"
            } else if matches!(
                c.endpoint_kind,
                EndpointKind::UnselectedPrefixTrajectory | EndpointKind::UnselectedPrefixCandidate
            ) {
                "one authenticated actual-prefix frame; first-divergence label comparison validated postcapture; offline only"
            } else if qualification.is_some() {
                "two authenticated actual first-divergence frames; labels attached separately after both captures"
            } else {
                "offline row cohort only; factual_failure includes preceding correct position3; labels attached separately after all captures"
            });
        }
        if qualification.is_some() {
            frame["endpoint_admission"]=json!("UNSELECTED_PREFIX_CANDIDATE; diagnostic capture only; failed cheap qualification retained");
            frame["saved_canonical_native"] = json!("NOT_RUN");
            frame["control_scope"]=json!("NOT_RUN: causal question is original versus candidate same actual prefix, not serving ablation");
        }
        if c.endpoint_kind == EndpointKind::SelectedOriginalCanonicalConditional {
            frame["endpoint_admission"] = json!(
                "selected original parent; generic canonical-conditional training witness only"
            );
            frame["control_scope"] =
                json!("NOT_RUN: missing ordered-phase training witness, not serving ablation");
            frame["request_role_scope"]=json!("one owner-declared canonical-conditional prefix; model receives no target; canonical prefix authorization after capture; not actual ownfeedback");
            frame["prefix_authority"]=json!("CANONICAL_CONDITIONAL_TRAINING; configured prefix may be label-derived before execution; label file/target array accessed after capture");
            frame["saved_actual"] = json!("NOT_APPLICABLE: conditional prefix, not actual rollout");
        }
        if c.endpoint_kind == EndpointKind::SelectedOriginalTrajectorySupplement {
            frame["endpoint_admission"] =
                json!("selected original parent; missing correct-prefix witness supplement only");
            frame["control_scope"] =
                json!("NOT_RUN: complete trajectory guard recovery, not serving ablation");
            frame["request_role_scope"]=json!("two authenticated original correct-prefix supplements; labels attached separately after both captures");
        }
        let payload = serde_json::to_vec(&frame)?;
        require(
            payload.len() as u64 <= c.maximum_cache_bytes,
            "oneframe payload exceeds cache cap",
        )?;
        let name = if c.endpoint_kind == EndpointKind::ZeroCompensation {
            format!("frame-{index:04}.json")
        } else {
            format!("frame-{index:04}-position-{position:02}.json")
        };
        summaries.push(json!({"input_index":index,"position":position,"id":packet.id,"role":request.role,"file":name,"sha256":hash(&payload),"winner":step.actions.summary.chosen_token_id,"post_state":frame["post_state"]}));
        write(c, &name, &frame, written)?;
    }
    // No label fields are accessed until every admitted production capture is saved.
    let labels = read(&c.labels)?;
    require(
        labels["schema"] == "uor-r4.native-source-bank-labels/1"
            && labels["cases"].as_array().is_some_and(|x| x.len() == 512),
        "labelauthority shape differs",
    )?;
    let mut posthoc = Vec::new();
    for request in &frames {
        let saved = if matches!(
            c.endpoint_kind,
            EndpointKind::UnselectedPrefixTrajectory | EndpointKind::UnselectedPrefixCandidate
        ) {
            let q = qualification
                .as_ref()
                .ok_or_else(|| bad("qualification missing"))?;
            let row = qualification_row(q, &request.expected_id)?;
            read(
                &c.qualification_root
                    .as_ref()
                    .ok_or_else(|| bad("qualification root missing"))?
                    .join(safe_row_leaf(row)?),
            )?
        } else if qualification.is_some() {
            read(
                &c.qualification_root
                    .as_ref()
                    .ok_or_else(|| bad("qualification root absent"))?
                    .join(if request.input_index == 245 {
                        "cheap-ownprefix-row-0008.json"
                    } else {
                        "cheap-ownprefix-row-0007.json"
                    }),
            )?
        } else {
            read(&c.compensation_root.join(format!(
                "development-0001-row-{:04}.json",
                request.input_index
            )))?
        };
        require(
            labels["cases"][request.input_index]["id"] == request.expected_id,
            "posthoclabel ID differs",
        )?;
        let mut annotation = json!({"input_index":request.input_index,"position":request.position,"request":request,"authority":labels["cases"][request.input_index],"saved_canonical_target_ids":saved["canonical_target_ids_labels_only"],"current_target_label_only":if qualification.is_some(){saved["canonical_target_ids_labels_only"][request.position].clone()} else {saved["canonical"][request.position]["target_label_only"].clone()},"attached_after_all_captures":true,"expected_record_query_role":"NOT_LOADED: external authenticated posthoc authority"});
        if c.endpoint_kind == EndpointKind::SelectedOriginalCanonicalConditional {
            let canonical: Vec<u32> =
                serde_json::from_value(saved["canonical_target_ids_labels_only"].clone())?;
            require(
                canonical.get(..request.position)
                    == Some(request.expected_actual_prefix_ids.as_slice()),
                "postcapture conditional prefix differs from canonical training authority",
            )?;
            let captured = read(&c.output.join(format!(
                "frame-{:04}-position-{:02}.json",
                request.input_index, request.position
            )))?;
            let target = &saved["canonical"][request.position]["target_label_only"];
            let mass = captured["pool"]["token_masses"]
                .as_array()
                .ok_or_else(|| bad("conditional masses absent"))?
                .iter()
                .find(|m| m["token_id"] == *target)
                .ok_or_else(|| bad("conditional target mass absent"))?;
            require(
                mass["weight_q31"] == saved["canonical"][request.position]["native_target_mass"]
                    && captured["pool"]["summary"]["total_weight_q31"]
                        == saved["canonical"][request.position]["native_denominator"],
                "postcapture conditional target mass/denominator differs",
            )?;
            annotation["canonical_target_mass_parity"] = json!({"target":target,"native_target_mass":mass["weight_q31"],"native_denominator":captured["pool"]["summary"]["total_weight_q31"],"checked_after_all_captures":true});
            annotation["prefix_authority"] = json!("CANONICAL_CONDITIONAL_TRAINING; owner-declared label-derived prefix, not actual ownfeedback; checked after target-free step");
        }
        if matches!(
            c.endpoint_kind,
            EndpointKind::UnselectedPrefixTrajectory | EndpointKind::UnselectedPrefixCandidate
        ) {
            annotation["actual_first_divergence"] = authorize_first_divergence(&saved, request)?;
        }
        posthoc.push(annotation);
    }
    write(
        c,
        "posthoc-labels.json",
        &json!({"frames":posthoc,"labels_sha256":c.expected_labels_sha256,"scope":"labels separate from targetfree production captures"}),
        written,
    )?;
    Ok(
        json!({"schema":report_schema(c.endpoint_kind),"status":"COMPLETED","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"runtime":if c.endpoint_kind==EndpointKind::SelectedOriginalCanonicalConditional {"production native bank generator; owner-declared canonical conditional training prefix; full Copy+Generate reducer; not actual ownfeedback"}else{"production native bank generator; exact admittedframes at authenticated saved actualprefixes; full Copy+Generate reducer"},"source_binding":binding,"generate_sha256":c.expected_generate_sha256,"continuation_sha256":c.expected_continuation_sha256,"compensation_report_sha256":c.expected_report_sha256,"compensation_manifest_sha256":c.expected_manifest_sha256,"inputs_sha256":c.expected_inputs_sha256,"labels_sha256":c.expected_labels_sha256,"categorical_sha256":receipt["categorical_sha256"],"exp_sha256":exp_hash,"endpoint_kind":c.endpoint_kind,"qualification_authority":if qualification.is_some(){json!({"root":c.qualification_root,"report_sha256":configured_qualification_pins(c)?.0,"manifest_sha256":configured_qualification_pins(c)?.1,"candidate_authority":c.candidate_authority,"candidate_selected":false,"controls":"NOT_RUN"})}else{Value::Null},"frame_count":frames.len(),"frames":summaries,"elapsed_seconds":clock.elapsed().as_secs_f64(),"scope":"exposed reached-prefix attribution; source role authority joined separately posthoc; no generated counterfactual, fit, gradient, model promotion, transfer or chat claim"}),
    )
}
fn earliest_physical_max(scores: &[i64]) -> Result<usize> {
    scores
        .iter()
        .enumerate()
        .max_by(|(ia, a), (ib, b)| a.cmp(b).then_with(|| ib.cmp(ia)))
        .map(|(i, _)| i)
        .ok_or_else(|| bad("physicalCopy candidates absent"))
}
fn shifted_sum(coefficients: &[i8], shift: u32) -> Result<i64> {
    let sum = coefficients.iter().try_fold(0i64, |n, x| {
        n.checked_add(i64::from(*x))
            .ok_or_else(|| bad("coefficient sum overflow"))
    })?;
    sum.checked_mul(1i64.checked_shl(shift).ok_or_else(|| bad("invalidshift"))?)
        .ok_or_else(|| bad("shiftedscore overflow"))
}
fn remove_shared_u(
    generate: &[i64],
    copy_ids: &[u32],
    copy: &[i64],
    delta: &[i64],
) -> Result<(Vec<i64>, Vec<i64>)> {
    require(
        generate.len() == delta.len() && copy_ids.len() == copy.len(),
        "sharedU vectorshape differs",
    )?;
    let base_g = generate
        .iter()
        .zip(delta)
        .map(|(g, u)| {
            g.checked_sub(*u)
                .ok_or_else(|| bad("baseGenerate subtraction overflow"))
        })
        .collect::<Result<Vec<_>>>()?;
    let base_c = copy
        .iter()
        .zip(copy_ids)
        .map(|(c, id)| {
            let u = delta
                .get(*id as usize)
                .ok_or_else(|| bad("physicalalias outofdomain"))?;
            c.checked_sub(*u)
                .ok_or_else(|| bad("baseCopy subtraction overflow"))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok((base_g, base_c))
}
fn component_sum_q24(unary: &[i8], pair: &[i8], bias: i8) -> Result<i64> {
    let sum = unary.iter().chain(pair).try_fold(i64::from(bias), |n, x| {
        n.checked_add(i64::from(*x))
            .ok_or_else(|| bad("factor sum overflow"))
    })?;
    sum.checked_mul(1i64 << 20)
        .ok_or_else(|| bad("Q24 attribution overflow"))
}
fn main() -> Result<()> {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() == 3 && argv[1] == "verify-report" {
        report_output::verify(Path::new(&argv[2]))?;
        return Ok(());
    }
    require(
        argv.len() == 2,
        "usage: native-reached-prefix-attribution CONFIG.json",
    )?;
    let raw = bytes(Path::new(&argv[1]))?;
    let mut c: Config = serde_json::from_slice(&raw)?;
    admit_paths(&mut c)?;
    report_output::claim(&c.output)?;
    let mut written = 0u64;
    let outcome = (|| -> Result<Value> {
        write(
            &c,
            "config.json",
            &serde_json::from_slice(&raw)?,
            &mut written,
        )?;
        run(&c, &mut written)
    })();
    let mut report = match &outcome {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":report_schema(c.endpoint_kind),"status":"FAILED","error":e.to_string(),"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"scope":"execution failure; no model-quality verdict"})
        }
    };
    let mut data = serde_json::to_vec(&report)?;
    let exceeded = written
        .checked_add(data.len() as u64)
        .map_or(true, |n| n > c.maximum_report_bytes);
    if exceeded {
        report = json!({"status":"FAILED","error":"final report exceeds byte cap","scope":"execution failure"});
        data = serde_json::to_vec(&report)?;
    }
    fs::write(c.output.join("report.json"), data)?;
    report_output::seal(&c.output)?;
    report_output::verify(&c.output)?;
    if exceeded {
        return Err(bad("final report exceeds byte cap"));
    }
    outcome.map(|_| ())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_conditional_request_is_generic_and_pinned() -> Result<()> {
        let mut f = FrameRequest {
            input_index: 7,
            position: 2,
            expected_id: "generic-training-row".into(),
            expected_saved_row_sha256: Some("a".repeat(64)),
            expected_actual_prefix_ids: vec![9, 10],
            role: FrameRole::FactualFailure,
        };
        validate_frames(
            &[f.clone()],
            EndpointKind::SelectedOriginalCanonicalConditional,
        )?;
        f.expected_saved_row_sha256 = None;
        assert!(validate_frames(
            &[f.clone()],
            EndpointKind::SelectedOriginalCanonicalConditional
        )
        .is_err());
        f.expected_saved_row_sha256 = Some("a".repeat(64));
        assert!(validate_frames(
            &[f.clone(), f],
            EndpointKind::SelectedOriginalCanonicalConditional
        )
        .is_err());
        Ok(())
    }
    #[test]
    fn original_trajectory_supplement_binds_both_missing_correct_prefixes() -> Result<()> {
        let mk = |i, prefix: Vec<u32>, sha: &str| FrameRequest {
            input_index: i,
            position: prefix.len(),
            expected_id: format!("id{i}"),
            expected_saved_row_sha256: Some(sha.into()),
            expected_actual_prefix_ids: prefix,
            role: FrameRole::SourceControl,
        };
        let mut frames = vec![
            mk(
                3,
                vec![617],
                "5d527e1e881f3b5937be974309eb6b984099356774f7e73120250b6d6a0cd897",
            ),
            mk(
                455,
                vec![],
                "871f1265ccd2fa58d250e80bbd348391e0be2aaa73ef2e59375accbe481e613b",
            ),
        ];
        validate_frames(&frames, EndpointKind::SelectedOriginalTrajectorySupplement)?;
        frames[0].expected_saved_row_sha256 = Some("a".repeat(64));
        assert!(
            validate_frames(&frames, EndpointKind::SelectedOriginalTrajectorySupplement).is_err()
        );
        frames.pop();
        assert!(
            validate_frames(&frames, EndpointKind::SelectedOriginalTrajectorySupplement).is_err()
        );
        Ok(())
    }
    #[test]
    fn unselected_prefix_capture_requires_exact_actual_two_frame_chain() -> Result<()> {
        let mk = |index, prefix: Vec<u32>| FrameRequest {
            input_index: index,
            position: prefix.len(),
            expected_id: format!("case{index}"),
            expected_saved_row_sha256: Some("a".repeat(64)),
            expected_actual_prefix_ids: prefix,
            role: FrameRole::FactualFailure,
        };
        let mut frames = vec![
            mk(245, vec![617, 2097]),
            mk(13, vec![617, 2097, 315, 261, 92, 607, 770]),
        ];
        validate_frames(&frames, EndpointKind::UnselectedPrefixFragment)?;
        frames[0].expected_actual_prefix_ids[1] = 315;
        assert!(validate_frames(&frames, EndpointKind::UnselectedPrefixFragment).is_err());
        let mut saved = json!({"generated_ids":[617,2097,1717],"eos":false,"canonical":"NOT_RUN","generation":[{"actual_prefix_ids":[],"pool":{"summary":{"chosen_token_id":617}}},{"actual_prefix_ids":[617],"pool":{"summary":{"chosen_token_id":2097}}},{"actual_prefix_ids":[617,2097],"pool":{"summary":{"chosen_token_id":1717}}}]});
        authenticate_actual_prefix(&saved, &[617, 2097], 2)?;
        assert!(authenticate_reached_prefix(&saved, &[617, 2097], 2).is_err());
        saved["generation"][1]["pool"]["summary"]["chosen_token_id"] = json!(315);
        assert!(authenticate_actual_prefix(&saved, &[617, 2097], 2).is_err());
        let receipt = json!({"step":1});
        let mut report = json!({"status":"COMPLETED","mode":"prefix_fragment_learning","candidate_receipt":receipt,"finite_prefix_positive":true,"qualified_fragment":true});
        authenticate_endpoint(&report, &receipt, EndpointKind::UnselectedPrefixFragment)?;
        report["candidate_receipt"]["step"] = json!(0);
        assert!(
            authenticate_endpoint(&report, &receipt, EndpointKind::UnselectedPrefixFragment)
                .is_err()
        );
        Ok(())
    }
    #[test]
    fn recovered_endpoint_requires_qualified_selected_receipt_without_legacy_winner() -> Result<()>
    {
        let receipt = json!({"step":1});
        let mut report = json!({"status":"COMPLETED","mode":"readout_intermediate_candidate","final_receipt":receipt,"selected_model":true,"candidate_artifact_status":"QUALIFIED_NATIVE_GATE_AND_RETENTION","useful_candidate":false,"reply_qualification":{"status":"PASSED","retained_original8":true,"candidate512":"COMPLETED"}});
        authenticate_endpoint(&report, &receipt, EndpointKind::SelectedReadoutIntermediate)?;
        assert!(authenticate_endpoint(&report, &receipt, EndpointKind::SelectedReachedU).is_err());
        report["selected_model"] = json!(false);
        assert!(authenticate_endpoint(
            &report,
            &receipt,
            EndpointKind::SelectedReadoutIntermediate
        )
        .is_err());
        report["selected_model"] = json!(true);
        report["final_receipt"]["step"] = json!(0);
        assert!(authenticate_endpoint(
            &report,
            &receipt,
            EndpointKind::SelectedReadoutIntermediate
        )
        .is_err());
        Ok(())
    }
    #[test]
    fn recovered_word_boundary_requires_exact_eighteen_frames_and_actual_extension() -> Result<()> {
        let mut frames = Vec::new();
        for i in [245usize, 0, 1, 4, 5, 8, 9, 12, 13] {
            for p in [3usize, 4] {
                frames.push(FrameRequest {
                    input_index: i,
                    position: p,
                    expected_id: format!("id{i}"),
                    expected_saved_row_sha256: Some("a".repeat(64)),
                    expected_actual_prefix_ids: if i == 245 {
                        if p == 3 {
                            vec![617, 2097, 315]
                        } else {
                            vec![617, 2097, 315, 1057]
                        }
                    } else {
                        vec![617; p]
                    },
                    role: if i == 245 {
                        FrameRole::FactualFailure
                    } else {
                        FrameRole::SourceControl
                    },
                });
            }
        }
        validate_frames(&frames, EndpointKind::SelectedReadoutIntermediate)?;
        frames[1].expected_actual_prefix_ids[3] = 307;
        assert!(validate_frames(&frames, EndpointKind::SelectedReadoutIntermediate).is_err());
        frames[1].expected_actual_prefix_ids[3] = 1057;
        frames.pop();
        assert!(validate_frames(&frames, EndpointKind::SelectedReadoutIntermediate).is_err());
        assert_eq!(
            report_schema(EndpointKind::SelectedReadoutIntermediate),
            "uor-r4.native-reached-prefix-attribution/3"
        );
        assert_eq!(
            report_schema(EndpointKind::SelectedReachedU),
            "uor-r4.native-reached-prefix-attribution/2"
        );
        Ok(())
    }
    #[test]
    fn newly_reached_position_four_requires_recorded_preceding_word() -> Result<()> {
        let generated = [617u32, 2097, 315, 1057, 307];
        let steps=generated.iter().enumerate().map(|(p,t)|json!({"actual_prefix_ids":&generated[..p],"pool":{"summary":{"chosen_token_id":t}}})).collect::<Vec<_>>();
        let mut canonical = vec![json!({}); 5];
        canonical[4] = json!({"native":{"pool":{"summary":{"chosen_token_id":307}}}});
        let mut saved =
            json!({"generated_ids":generated,"eos":false,"generation":steps,"canonical":canonical});
        authenticate_reached_prefix(&saved, &generated[..4], 4)?;
        saved["generation"][3]["pool"]["summary"]["chosen_token_id"] = json!(267);
        assert!(authenticate_reached_prefix(&saved, &generated[..4], 4).is_err());
        Ok(())
    }
    #[test]
    fn signed_q4_factor_contributions_preserve_native_q24_sum() -> Result<()> {
        // Opposite extremes, cancellation and nonzero bias catch unsigned nibble/scaling drift.
        assert_eq!(component_sum_q24(&[-7, 7, -1], &[7, -7], 3)?, 2 << 20);
        assert_eq!(component_sum_q24(&[-7; 8], &[-7; 4], -7)?, -91 << 20);
        Ok(())
    }
    #[test]
    fn padded_pair_and_signed_nibbles_preserve_ordered_contributions() -> Result<()> {
        use uor_r4_core::native_geometric::learner::integrated_attention::geometry::{
            EnergyReadCounts, EnergyTables, LanePair,
        };
        let mut energy = EnergyTables::zeroed(2, vec![LanePair { left: 0, right: 1 }])?;
        energy.set_unary(0, 119, -7)?;
        energy.set_unary(1, 1, 7)?;
        energy.set_pair(0, 119, 1, -3)?;
        energy.set_pair(0, 1, 119, 6)?;
        let unary = [energy.get_unary(0, 119)?, energy.get_unary(1, 1)?];
        let pair = [energy.get_pair(0, 119, 1)?];
        assert_eq!(pair, [-3]);
        assert_eq!(energy.get_pair(0, 1, 119)?, 6);
        let mut counts = EnergyReadCounts::default();
        assert_eq!(
            component_sum_q24(&unary, &pair, 2)?,
            (i64::from(energy.score(&[119, 1], &mut counts)?) + 2) << 20
        );
        assert_eq!(energy.get_unary(0, 118)?, 0);
        Ok(())
    }
    #[test]
    fn teacher_only_prefix_cannot_authorize_actual_frame() -> Result<()> {
        // Reject actual-prefix mismatch before accessing any native arrays.
        let saved = json!({"generated_ids":[617],"generation":[{}, {"actual_prefix_ids":[2997]}],"canonical":[{}, {"native":{}}]});
        assert!(authenticate_reached_prefix(&saved, &[617], 1).is_err());
        let missing_actual =
            json!({"generated_ids":[617],"canonical":[{}, {"native":{"actual_prefix_ids":[617]}}]});
        assert!(authenticate_reached_prefix(&missing_actual, &[617], 1).is_err());
        Ok(())
    }
    #[test]
    fn shared_u_removal_preserves_every_duplicate_copy_alias_and_base_donor() -> Result<()> {
        let delta = [0, 10, -4];
        let (g, c) = remove_shared_u(&[2, 13, 0], &[1, 1, 2], &[15, 14, 5], &delta)?;
        assert_eq!(g, vec![2, 3, 4]);
        assert_eq!(c, vec![5, 4, 9]);
        assert_eq!(earliest_physical_max(&c)?, 2);
        assert_eq!(earliest_physical_max(&[15, 14, 5])?, 0);
        assert_eq!(earliest_physical_max(&[9, 9, 8])?, 0);
        assert!(remove_shared_u(&[i64::MIN], &[], &[], &[1]).is_err());
        assert!(remove_shared_u(&[0], &[1], &[0], &[0]).is_err());
        assert_eq!(shifted_sum(&[-7, 7, 1], 22)?, 1 << 22);
        Ok(())
    }
    #[test]
    fn actual_frame_authority_rejects_wrong_chain_past_eos_and_cutoff() -> Result<()> {
        let summary0 = json!({"chosen_token_id":617});
        let summary1 = json!({"chosen_token_id":2097});
        let mut saved = json!({"generated_ids":[617,2097],"eos":false,"generation":[{"actual_prefix_ids":[],"pool":{"summary":summary0}},{"actual_prefix_ids":[617],"pool":{"summary":summary1}}],"canonical":[{}, {"native":{"pool":{"summary":summary1}}}]});
        authenticate_reached_prefix(&saved, &[617], 1)?;
        assert!(authenticate_reached_prefix(&saved, &[617, 2097], 2).is_err());
        saved["generation"][0]["pool"]["summary"]["chosen_token_id"] = json!(2997);
        assert!(authenticate_reached_prefix(&saved, &[617], 1).is_err());
        saved["generated_ids"] = json!([1, 2097]);
        assert!(authenticate_reached_prefix(&saved, &[1], 1).is_err());
        Ok(())
    }
    #[test]
    fn frame_identity_includes_position_and_duplicate_positions_rejected() -> Result<()> {
        let request = |i, p| FrameRequest {
            input_index: i,
            position: p,
            expected_id: format!("id{i}"),
            expected_saved_row_sha256: Some("a".repeat(64)),
            expected_actual_prefix_ids: vec![617; p],
            role: FrameRole::LegacyEntry,
        };
        let valid = vec![request(399, 1), request(5, 1), request(13, 1)];
        validate_frames(&valid, EndpointKind::ZeroCompensation)?;
        let invalid = vec![request(399, 1), request(5, 1), request(5, 1)];
        assert!(validate_frames(&invalid, EndpointKind::ZeroCompensation).is_err());
        let mut cohort = Vec::new();
        for i in [97, 156, 151, 245, 392] {
            let mut f = request(i, 3);
            f.role = FrameRole::FactualFailure;
            cohort.push(f);
        }
        for i in [0, 1, 4, 5, 8, 9, 12, 13] {
            let mut f = request(i, 3);
            f.role = FrameRole::SourceControl;
            cohort.push(f);
        }
        for i in [399, 5, 13] {
            let mut f = request(i, 2);
            f.role = if i == 399 {
                FrameRole::GrammarFailure
            } else {
                FrameRole::GrammarControl
            };
            cohort.push(f);
        }
        validate_frames(&cohort, EndpointKind::SelectedReachedU)?;
        cohort[15].position = 3;
        cohort[15].expected_actual_prefix_ids.push(315);
        assert!(validate_frames(&cohort, EndpointKind::SelectedReachedU).is_err());
        Ok(())
    }
    #[test]
    fn trajectory_first_divergence_is_generic_and_postcapture_authorized() -> Result<()> {
        let mut request = FrameRequest {
            input_index: 42,
            position: 2,
            expected_id: "arbitrary-case".into(),
            expected_saved_row_sha256: Some("a".repeat(64)),
            expected_actual_prefix_ids: vec![8, 9],
            role: FrameRole::FactualFailure,
        };
        validate_frames(&[request.clone()], EndpointKind::UnselectedPrefixTrajectory)?;
        let saved = json!({"id":"arbitrary-case","generated_ids":[8,9,11,12],
            "canonical_target_ids_labels_only":[8,9,10,12]});
        let witness = authorize_first_divergence(&saved, &request)?;
        assert_eq!(witness["position"], 2);
        assert_eq!(witness["derived_after_all_captures"], true);
        request.position = 3;
        request.expected_actual_prefix_ids.push(11);
        assert!(authorize_first_divergence(&saved, &request).is_err());
        request.position = 2;
        request.expected_actual_prefix_ids = vec![8, 10];
        assert!(authorize_first_divergence(&saved, &request).is_err());
        assert!(validate_frames(
            &[request.clone(), request],
            EndpointKind::UnselectedPrefixTrajectory
        )
        .is_err());
        Ok(())
    }
    #[test]
    fn trajectory_epoch_and_saved_row_identity_fail_closed() -> Result<()> {
        let receipt = json!({"step":1});
        let mut report = json!({"status":"COMPLETED","mode":"prefix_trajectory_learning",
            "candidate_receipt":receipt,"finite_prefix_positive":true,"qualified_fragment":true,
            "all_original380_preserved":true,"protected_population":380,"new_prefix_gradients":0,
            "selected_model":false});
        authenticate_endpoint(&report, &receipt, EndpointKind::UnselectedPrefixTrajectory)?;
        report["selected_model"] = json!(true);
        assert!(
            authenticate_endpoint(&report, &receipt, EndpointKind::UnselectedPrefixTrajectory)
                .is_err()
        );
        let q = json!({"evaluation":{"rows":[{"id":"one","row_file":"../foreign.json"}]}});
        assert!(safe_row_leaf(qualification_row(&q, "one")?).is_err());
        let duplicate = json!({"evaluation":{"rows":[{"id":"one"},{"id":"one"}]}});
        assert!(qualification_row(&duplicate, "one").is_err());
        Ok(())
    }
    #[test]
    fn generic_candidate_authority_is_exclusive_and_pinned() -> Result<()> {
        let mut a = CandidateAuthority {
            family: CandidateFamily::PrefixJointFragment,
            expected_prefix_packed_sha256: "a".repeat(64),
            expected_prefix_native_metadata_sha256: "b".repeat(64),
            expected_prefix_master_sha256: "c".repeat(64),
            expected_qualification_report_sha256: "d".repeat(64),
            expected_qualification_manifest_sha256: "e".repeat(64),
        };
        validate_candidate_authority(EndpointKind::UnselectedPrefixCandidate, Some(&a))?;
        assert!(
            validate_candidate_authority(EndpointKind::UnselectedPrefixTrajectory, Some(&a))
                .is_err()
        );
        assert!(
            validate_candidate_authority(EndpointKind::UnselectedPrefixCandidate, None).is_err()
        );
        let receipt = json!({"step":1});
        let mut report = json!({"status":"COMPLETED","mode":"prefix_joint_fragment_learning",
            "candidate_receipt":receipt,"selected_model":false,"finite_prefix_positive":true,
            "qualified_fragment":true,"all3_phase_targets_correct":true,
            "all_original380_preserved":true,"prefix_backward_calls":20,"candidate_native_steps":383});
        authenticate_candidate(&report, &receipt, &a)?;
        report["selected_model"] = json!(true);
        assert!(authenticate_candidate(&report, &receipt, &a).is_err());
        report["selected_model"] = json!(false);
        report["candidate_receipt"] = json!({"step":2});
        assert!(authenticate_candidate(&report, &receipt, &a).is_err());
        a.expected_prefix_master_sha256 = "not-a-digest".into();
        assert!(
            validate_candidate_authority(EndpointKind::UnselectedPrefixCandidate, Some(&a))
                .is_err()
        );
        Ok(())
    }
    #[test]
    fn generic_candidate_request_has_no_fixed_case_or_position() -> Result<()> {
        let mut f = FrameRequest {
            input_index: 42,
            position: 2,
            expected_id: "generic-case".into(),
            expected_saved_row_sha256: Some("a".repeat(64)),
            expected_actual_prefix_ids: vec![7, 9],
            role: FrameRole::FactualFailure,
        };
        validate_frames(&[f.clone()], EndpointKind::UnselectedPrefixCandidate)?;
        assert!(validate_frames(
            &[f.clone(), f.clone()],
            EndpointKind::UnselectedPrefixCandidate
        )
        .is_err());
        f.role = FrameRole::SourceControl;
        assert!(validate_frames(&[f], EndpointKind::UnselectedPrefixCandidate).is_err());
        Ok(())
    }
}
