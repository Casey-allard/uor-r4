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
        validate_role_ids, PairedRoleDecision, PairedRoleState, RequestedContext,
        RequestedOccurrenceTrace, RoleSpan, ROLE_CONTEXT, ROLE_ROOTS, ROLE_VOCAB,
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
    context: Option<RequestedContext>,
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
            match context {
                None => 0,
                Some(RequestedContext::FullPrefix) => 1,
                Some(RequestedContext::UserTextOnly) => 2,
                Some(RequestedContext::ClauseToEntity(_)) => 3,
            },
            rows.len() as u32,
            roles.cells().len() as u32,
        ],
    );
    for (id, [&root, &class]) in roles.sites().iter().map(|s| [&s[0], &s[1]]).enumerate() {
        let clause = match context {
            Some(RequestedContext::ClauseToEntity(separators)) => separators.contains(&(id as u32)),
            _ => false,
        };
        put(
            &mut packet,
            [
                root,
                class,
                classes[id],
                u32::from(delimiters.contains(&(id as u32))) | (u32::from(clause) << 1),
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
            .filter_map(|c| {
                let [requested, excluded] = c.excluded_decision_scores.unwrap_or(c.scores);
                (excluded > requested).then_some((excluded, c.span))
            })
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
        native.requested_context,
        &rows,
        &expected,
    )?;
    if let Some(context) = native.requested_context {
        attach_census(
            &mut device_rows,
            &rows,
            native.roles,
            native.delimiters,
            native.assertion_classes,
            context,
        )?;
    }
    save(
        output,
        &serde_json::json!({
            "schema": "uor-r4.requested-role-device/1",
            "policy": match native.requested_context {
                None => "legacy",
                Some(RequestedContext::FullPrefix) => "assertion_agreement",
                Some(RequestedContext::UserTextOnly) => "user_text_only",
                Some(RequestedContext::ClauseToEntity(_)) => "clause_to_entity",
            },
            "artifact": model.artifact_sha256(),
            "blind": format!("{:x}", Sha256::digest(&raw)),
            "rows": device_rows
        }),
    )
}

fn attach_census(
    device_rows: &mut [RoleRow],
    rows: &[(&[u32], Range<usize>)],
    roles: &PairedRoleState,
    delimiters: &[u32],
    classes: &[u32],
    context: RequestedContext,
) -> Result<(), Failure> {
    for (result, (ids, content)) in device_rows.iter_mut().zip(rows) {
        let trace = roles.trace_requested_occurrence(
            ids,
            content.clone(),
            0..ids.len(),
            delimiters,
            classes,
            context,
        )?;
        if ranked_census(&trace) != result.role_words {
            return Err(format!("row {} census differs from device words", result.index).into());
        }
        result.requested_occurrence = Some(trace);
    }
    Ok(())
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
        None,
        &rows,
        &original_expected,
    )?;
    let assertion_agreement = replay(
        executable,
        &state,
        &classes,
        &DELIMITERS,
        Some(RequestedContext::FullPrefix),
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

fn user_text_fixture_state() -> Result<(PairedRoleState, Vec<u32>), Failure> {
    let (state, classes) = fixture_state()?;
    let offset = |i| i * ROLE_ROOTS as u32;
    let state = PairedRoleState::new(
        *state.bindings(),
        state.sites().to_vec(),
        vec![
            [offset(2) + POS, 0, 4, 0],
            [offset(3) + POS, 0, 32, 0],
            [offset(3) + NEG, 0, 0, 32],
            [offset(4) - 1, 1, 0, 1],
        ],
    )?;
    Ok((state, classes))
}

fn user_text_fixture_tapes() -> [Vec<u32>; 2] {
    let content = [
        0, FENCE, A[0], A[1], 7, FENCE, B[0], B[1], 7, FENCE, POS, A[0], A[1], 10, B[0], B[1],
    ];
    [POS, NEG].map(|suffix| [content.as_slice(), &[suffix]].concat())
}

fn user_text_fixture_rows(tapes: &[Vec<u32>; 2]) -> [(&[u32], Range<usize>); 2] {
    tapes.each_ref().map(|ids| (ids.as_slice(), 2..16))
}

fn user_text_fixture_expectations(
    state: &PairedRoleState,
    classes: &[u32],
    rows: &[(&[u32], Range<usize>); 2],
) -> Result<[[[u32; 8]; 2]; 2], Failure> {
    let mut all_words = [[[0; 8]; 2]; 2];
    for (i, (ids, content)) in rows.iter().enumerate() {
        for (arm, context) in [RequestedContext::FullPrefix, RequestedContext::UserTextOnly]
            .into_iter()
            .enumerate()
        {
            all_words[arm][i] = state
                .predict_requested_occurrence(
                    ids,
                    content.clone(),
                    0..ids.len(),
                    &DELIMITERS,
                    classes,
                    context,
                )?
                .words();
        }
        let full = all_words[0][i];
        let scoped = all_words[1][i];
        let planted = if i == 0 {
            (14, 16, 56, 40)
        } else {
            (11, 13, 40, -56)
        };
        if (full[0], full[1], full[4] as i32, full[6] as i32) != planted
            || (scoped[0], scoped[1], scoped[4] as i32, scoped[6] as i32) != (11, 13, 40, -12)
            || [full[2], full[3], full[5], full[7]] != [scoped[2], scoped[3], scoped[5], scoped[7]]
        {
            return Err(format!(
                "fixture {i} disagrees with planted full/scoped rank: {full:?}, {scoped:?}"
            )
            .into());
        }
        let census = state.trace_requested_occurrence(
            ids,
            content.clone(),
            0..ids.len(),
            &DELIMITERS,
            classes,
            RequestedContext::UserTextOnly,
        )?;
        let second = census
            .candidates
            .iter()
            .find(|c| c.span == [14, 16])
            .ok_or("missing second query candidate")?;
        if second.feature_keys[3..] != [4, 5, 6].map(|offset| offset * ROLE_ROOTS as u32 - 1) {
            return Err(
                "the synthetic suffix was not replaced with the missing-root sentinel".into(),
            );
        }
    }
    Ok(all_words)
}

fn finite_user_text_only(executable: &Path, output: &Path) -> Result<(), Failure> {
    let (state, classes) = user_text_fixture_state()?;
    let tapes = user_text_fixture_tapes();
    let rows = user_text_fixture_rows(&tapes);
    let expected = user_text_fixture_expectations(&state, &classes, &rows)?;
    let capture = |context, expected: &[[u32; 8]; 2]| -> Result<Vec<RoleRow>, Failure> {
        let mut device = replay(
            executable,
            &state,
            &classes,
            &DELIMITERS,
            Some(context),
            &rows,
            expected,
        )?;
        attach_census(&mut device, &rows, &state, &DELIMITERS, &classes, context)?;
        Ok(device)
    };
    save(
        output,
        &serde_json::json!({
            "schema": "uor-r4.user-text-only-role-finite/1",
            "assertion_agreement": capture(RequestedContext::FullPrefix, &expected[0])?,
            "user_text_only": capture(RequestedContext::UserTextOnly, &expected[1])?,
            "fixtures": ["requested_root_outside_content", "excluded_root_outside_content"],
        }),
    )
}

const CLAUSE_FENCES: [u32; 2] = [10, 11];

fn clause_fixture_tapes() -> [Vec<u32>; 8] {
    let facts = [0, FENCE, A[0], A[1], 7, FENCE, B[0], B[1], 7, FENCE];
    let clause = |name: [u32; 2], cue| [cue, 12, 12, 12, name[0], name[1]];
    let pair = |left: [u32; 6], right: [u32; 6], delimiter| {
        [facts.as_slice(), &left, &[delimiter], &right, &[FENCE]].concat()
    };
    [
        pair(clause(A, POS), clause(B, NEG), CLAUSE_FENCES[0]),
        pair(clause(A, NEG), clause(B, POS), CLAUSE_FENCES[1]),
        pair(clause(B, POS), clause(A, NEG), CLAUSE_FENCES[1]),
        pair(clause(B, NEG), clause(A, POS), CLAUSE_FENCES[0]),
        pair(clause(A, POS), clause(B, POS), CLAUSE_FENCES[0]),
        pair(clause(A, NEG), clause(B, NEG), CLAUSE_FENCES[0]),
        pair(clause(A, POS), clause(B, NEG), 12),
        pair(clause([7, 12], POS), clause(B, NEG), CLAUSE_FENCES[0]),
    ]
}

fn clause_fixture_expectations(
    state: &PairedRoleState,
    classes: &[u32],
    rows: &[(&[u32], Range<usize>)],
) -> Result<Vec<[u32; 8]>, Failure> {
    let outcomes = [
        (14, 16, 56, -56),
        (21, 23, 56, -56),
        (14, 16, 56, -56),
        (21, 23, 56, -56),
        (ABSENT, ABSENT, 56, 56),
        (ABSENT, ABSENT, -56, -56),
        (ABSENT, ABSENT, i32::MIN, i32::MIN),
        (21, 23, -56, i32::MIN),
    ];
    rows.iter()
        .zip(outcomes)
        .map(|((ids, content), outcome)| {
            let context = RequestedContext::ClauseToEntity(CLAUSE_FENCES);
            let words = |context| {
                state
                    .predict_requested_occurrence(
                        ids, content.clone(), 0..ids.len(), &DELIMITERS, classes, context,
                    )
                    .map(PairedRoleDecision::words)
            };
            let baseline = words(RequestedContext::FullPrefix)?;
            let treatment = words(context)?;
            if (treatment[0], treatment[1], treatment[4] as i32, treatment[6] as i32) != outcome
                || baseline[4] != 0
                || [2, 3, 5, 7].into_iter().any(|i| baseline[i] != treatment[i])
            {
                return Err(format!(
                    "fixture disagrees with planted clause rank {outcome:?}: {baseline:?}, {treatment:?}"
                )
                .into());
            }
            Ok(treatment)
        })
        .collect()
}

fn finite_clause_to_entity(executable: &Path, output: &Path) -> Result<(), Failure> {
    let (state, classes) = fixture_state()?;
    let tapes = clause_fixture_tapes();
    let rows = tapes.each_ref().map(|ids| (ids.as_slice(), 2..ids.len()));
    let expected = clause_fixture_expectations(&state, &classes, &rows)?;
    let context = RequestedContext::ClauseToEntity(CLAUSE_FENCES);
    let mut device = replay(
        executable,
        &state,
        &classes,
        &DELIMITERS,
        Some(context),
        &rows,
        &expected,
    )?;
    attach_census(&mut device, &rows, &state, &DELIMITERS, &classes, context)?;
    save(
        output,
        &serde_json::json!({
            "schema": "uor-r4.clause-to-entity-role-finite/1",
            "clause_to_entity": device,
            "fixtures": ["positive_a_first", "positive_b_second", "positive_b_first", "positive_a_second",
                         "tied_positive", "negative_only", "shared_clause", "unsupported_positive"],
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
        [flag, executable, output] if flag == "--finite-user-text-only" => {
            finite_user_text_only(Path::new(executable), Path::new(output))
        }
        [flag, executable, output] if flag == "--finite-clause-to-entity" => {
            finite_clause_to_entity(Path::new(executable), Path::new(output))
        }
        _ => Err(
            "usage: requested-role-device ARTIFACT BLIND EXECUTABLE OUT | --finite EXECUTABLE OUT | --finite-user-text-only EXECUTABLE OUT | --finite-clause-to-entity EXECUTABLE OUT"
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

    #[test]
    fn planted_user_text_direction_and_off_text_perturbation() -> Result<(), Failure> {
        let (state, classes) = user_text_fixture_state()?;
        let tapes = user_text_fixture_tapes();
        user_text_fixture_expectations(&state, &classes, &user_text_fixture_rows(&tapes))?;
        Ok(())
    }

    #[test]
    fn clause_contrast_follows_evidence_across_name_and_clause_orders() -> Result<(), Failure> {
        let (state, classes) = fixture_state()?;
        let tapes = clause_fixture_tapes();
        let rows = tapes.each_ref().map(|ids| (ids.as_slice(), 2..ids.len()));
        clause_fixture_expectations(&state, &classes, &rows)?;
        Ok(())
    }
}
