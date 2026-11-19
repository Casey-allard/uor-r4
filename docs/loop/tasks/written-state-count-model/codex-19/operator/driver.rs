use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    env,
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    ops::Range,
    path::Path,
    process::{Command, Stdio},
};
use uor_r4_integer::{
    discrete_d11::IntegerDiscreteModel,
    paired_role::{
        validate_role_ids, PairedRoleDecision, PairedRoleState, RequestedOccurrenceTrace, RoleSpan,
        ROLE_CONTEXT, ROLE_ROOTS, ROLE_VOCAB,
    },
};

const ABI: u32 = 0x3141524f;
type Failure = Box<dyn Error>;

#[derive(Deserialize)]
struct Prompt {
    prefix: Vec<u32>,
}

#[derive(Serialize)]
struct RoleRow {
    index: usize,
    prefix_sha256: String,
    role_words: [u32; 8],
    #[serde(skip_serializing_if = "Option::is_none")]
    requested_occurrence: Option<RequestedOccurrenceTrace>,
}

fn put(bytes: &mut Vec<u8>, words: impl IntoIterator<Item = u32>) {
    for word in words {
        bytes.extend(word.to_le_bytes());
    }
}

fn replay(
    executable: &Path,
    roles: &PairedRoleState,
    classes: &[u32],
    delimiters: &[u32],
    treatment: bool,
    rows: &[(&[u32], Range<usize>)],
    expected: &[[u32; 8]],
) -> Result<Vec<RoleRow>, Failure> {
    if rows.is_empty() || rows.len() != expected.len() || classes.len() != ROLE_VOCAB {
        return Err("invalid replay dimensions".into());
    }
    let mut packet = Vec::new();
    put(
        &mut packet,
        [
            ABI,
            u32::from(treatment),
            rows.len() as u32,
            roles.cells().len() as u32,
        ],
    );
    for (id, [&root, &class]) in roles.sites().iter().map(|s| [&s[0], &s[1]]).enumerate() {
        put(
            &mut packet,
            [
                root,
                class,
                classes[id],
                u32::from(delimiters.contains(&(id as u32))),
            ],
        );
    }
    for cell in roles.cells() {
        put(&mut packet, *cell);
    }
    for (ids, content) in rows {
        validate_role_ids(ids)?;
        put(
            &mut packet,
            ids.iter()
                .copied()
                .chain(std::iter::repeat(0))
                .take(ROLE_CONTEXT),
        );
        put(
            &mut packet,
            [ids.len() as u32, content.start as u32, content.end as u32],
        );
    }

    let mut child = Command::new(executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("missing device stdin")?
        .write_all(&packet)?;
    let output = child.wait_with_output()?;
    if !output.status.success() || output.stdout.len() != rows.len() * 32 {
        return Err(format!(
            "device failed with status {} and {} bytes",
            output.status,
            output.stdout.len()
        )
        .into());
    }
    rows.iter()
        .zip(expected)
        .zip(output.stdout.chunks_exact(32))
        .enumerate()
        .map(|(index, (((ids, _), expected), output))| {
            let role_words = std::array::from_fn(|word| {
                let start = word * 4;
                u32::from_le_bytes(output[start..start + 4].try_into().unwrap())
            });
            if &role_words != expected {
                return Err(format!(
                    "row {index} device {role_words:?} differs from Rust {expected:?}"
                )
                .into());
            }
            let mut hash = Sha256::new();
            for id in *ids {
                hash.update(id.to_le_bytes());
            }
            Ok(RoleRow {
                index,
                prefix_sha256: format!("{:x}", hash.finalize()),
                role_words,
                requested_occurrence: None,
            })
        })
        .collect()
}

fn ranked_census(trace: &RequestedOccurrenceTrace) -> [u32; 8] {
    let rank = |mut candidates: Vec<(i32, [usize; 2])>, floor| {
        candidates.sort_unstable_by_key(|&(score, _)| std::cmp::Reverse(score));
        let best = candidates.first().map_or(i32::MIN, |row| row.0);
        let next = candidates.get(1).map_or(i32::MIN, |row| row.0);
        let span = candidates.first().and_then(|&(_, [start, end])| {
            (best >= floor && best > next).then_some(RoleSpan { start, end })
        });
        (span, best, next)
    };
    let (requested, requested_score, requested_runner_up) = rank(
        trace
            .candidates
            .iter()
            .filter_map(|c| c.contrast.map(|m| (m, c.span)))
            .collect(),
        i32::MIN,
    );
    let (excluded, excluded_score, excluded_runner_up) = rank(
        trace
            .candidates
            .iter()
            .filter_map(|c| (c.scores[1] > c.scores[0]).then_some((c.scores[1], c.span)))
            .collect(),
        12,
    );
    PairedRoleDecision {
        requested,
        excluded,
        requested_score,
        excluded_score,
        requested_runner_up,
        excluded_runner_up,
    }
    .words()
}

fn save(path: &Path, value: &impl Serialize) -> Result<(), Failure> {
    let mut output = OpenOptions::new().write(true).create_new(true).open(path)?;
    serde_json::to_writer_pretty(&mut output, value)?;
    output.write_all(b"\n")?;
    output.sync_all()?;
    Ok(())
}

fn artifact(executable: &Path, carrier: &Path, blind: &Path, output: &Path) -> Result<(), Failure> {
    let model = IntegerDiscreteModel::load(carrier)?;
    let native = model.frozen_role_replay()?;
    let raw = fs::read(blind)?;
    let prompts: Vec<Prompt> = serde_json::from_slice(&raw)?;
    let mut rows = Vec::new();
    let mut expected = Vec::new();
    for prompt in &prompts {
        let ids = prompt.prefix.as_slice();
        let content = native
            .requested_framing
            .map_or_else(|| Ok(0..ids.len()), |f| f.content(ids))?;
        rows.push((ids, content));
        expected.push(
            model
                .paired_roles_diagnostic(ids, 0..ids.len(), false)?
                .words(),
        );
    }
    let mut device_rows = replay(
        executable,
        native.roles,
        native.assertion_classes,
        native.delimiters,
        native.requested_framing.is_some(),
        &rows,
        &expected,
    )?;
    if native.requested_framing.is_some() {
        for (result, (ids, content)) in device_rows.iter_mut().zip(rows) {
            let trace = native.roles.trace_assertion_agreement(
                ids,
                content,
                0..ids.len(),
                native.delimiters,
                native.assertion_classes,
            )?;
            if ranked_census(&trace) != result.role_words {
                return Err(
                    format!("row {} census differs from device words", result.index).into(),
                );
            }
            result.requested_occurrence = Some(trace);
        }
    }
    save(
        output,
        &serde_json::json!({
            "schema": "uor-r4.requested-role-device/1",
            "policy": if native.requested_framing.is_some() {"assertion_agreement"} else {"legacy"},
            "artifact": model.artifact_sha256(),
            "blind": format!("{:x}", Sha256::digest(&raw)),
            "rows": device_rows
        }),
    )
}

const POS: u32 = 8;
const NEG: u32 = 9;
const A: [u32; 2] = [3, 4];
const B: [u32; 2] = [5, 6];
const DELIMITERS: [u32; 1] = [2];
const FENCE: u32 = DELIMITERS[0];
const ABSENT: u32 = u32::MAX;

fn fixture_state() -> Result<(PairedRoleState, Vec<u32>), Failure> {
    let mut sites = vec![[0, 0]; ROLE_VOCAB];
    for [start, continuation] in [A, B] {
        sites[start as usize][1] = 1;
        sites[continuation as usize][1] = 2;
    }
    sites[7][1] = 1;
    sites[POS as usize][0] = POS;
    sites[NEG as usize][0] = NEG;
    let classes = sites.iter().map(|site| site[1]).collect();
    let state = PairedRoleState::new(
        [[0; 32]; 4],
        sites,
        vec![
            [2 * ROLE_ROOTS as u32 + POS, 0, 32, 0],
            [2 * ROLE_ROOTS as u32 + NEG, 0, 0, 32],
        ],
    )?;
    Ok((state, classes))
}

fn fixture_tapes() -> Vec<Vec<u32>> {
    let facts = [0, FENCE, A[0], A[1], 7, FENCE, B[0], B[1], 7, FENCE];
    let paired = |left, right| {
        [
            facts.as_slice(),
            &[left, A[0], A[1], right, B[0], B[1], FENCE],
        ]
        .concat()
    };
    vec![
        paired(POS, NEG),
        paired(NEG, POS),
        paired(POS, POS),
        [facts.as_slice(), &[NEG, A[0], A[1], FENCE]].concat(),
        [facts.as_slice(), &[POS, A[0], B[1], FENCE]].concat(),
        [facts.as_slice(), &[POS, 7, FENCE]].concat(),
    ]
}

fn fixture_treatment(
    state: &PairedRoleState,
    classes: &[u32],
    tapes: &[Vec<u32>],
) -> Result<Vec<[u32; 8]>, Failure> {
    let expected: Vec<_> = tapes
        .iter()
        .map(|ids| {
            state
                .predict_assertion_agreement(ids, 2..ids.len(), 0..ids.len(), &DELIMITERS, classes)
                .map(|decision| decision.words())
        })
        .collect::<Result<_, _>>()?;
    let planted = [
        (11, 13, 56, -56),
        (14, 16, 56, -56),
        (ABSENT, ABSENT, 56, 56),
        (11, 13, -56, i32::MIN),
        (ABSENT, ABSENT, i32::MIN, i32::MIN),
        (ABSENT, ABSENT, i32::MIN, i32::MIN),
    ];
    for (row, &(start, end, best, runner_up)) in expected.iter().zip(&planted) {
        if (row[0], row[1], row[4] as i32, row[6] as i32) != (start, end, best, runner_up) {
            return Err(
                format!("finite fixture did not produce planted role outcome {row:?}").into(),
            );
        }
    }
    Ok(expected)
}

fn finite(executable: &Path, output: &Path) -> Result<(), Failure> {
    let (state, classes) = fixture_state()?;
    let tapes = fixture_tapes();
    let rows: Vec<_> = tapes
        .iter()
        .map(|ids| (ids.as_slice(), 2..ids.len()))
        .collect();
    let original_expected: Vec<_> = tapes
        .iter()
        .map(|ids| state.predict(ids).map(|d| d.words()))
        .collect::<Result<_, _>>()?;
    let treatment_expected = fixture_treatment(&state, &classes, &tapes)?;
    let legacy = replay(
        executable,
        &state,
        &classes,
        &DELIMITERS,
        false,
        &rows,
        &original_expected,
    )?;
    let assertion_agreement = replay(
        executable,
        &state,
        &classes,
        &DELIMITERS,
        true,
        &rows,
        &treatment_expected,
    )?;
    save(
        output,
        &serde_json::json!({
            "schema": "uor-r4.requested-role-finite/1", "legacy": legacy, "assertion_agreement": assertion_agreement,
            "fixtures": ["first_name", "second_name", "tie", "sole_negative_contrast",
                "different_continuation", "no_query_support"]
        }),
    )
}

fn main() -> Result<(), Failure> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    match args.as_slice() {
        [carrier, blind, executable, output] => artifact(
            Path::new(executable),
            Path::new(carrier),
            Path::new(blind),
            Path::new(output),
        ),
        [flag, executable, output] if flag == "--finite" => {
            finite(Path::new(executable), Path::new(output))
        }
        _ => Err(
            "usage: requested-role-device ARTIFACT BLIND EXECUTABLE OUT | --finite EXECUTABLE OUT"
                .into(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planted_direction_and_abstention() -> Result<(), Failure> {
        let (state, classes) = fixture_state()?;
        fixture_treatment(&state, &classes, &fixture_tapes())?;
        Ok(())
    }
}
