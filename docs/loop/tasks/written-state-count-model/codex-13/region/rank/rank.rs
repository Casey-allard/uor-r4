use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::io;
use uor_r4_integer::paired_role::{role_candidates, PairedRoleState, RoleSpan};
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
struct Grade {
    id: String,
    prefix: Vec<u32>,
    assertions: Vec<Assertion>,
}
#[derive(Deserialize)]
struct Assertion {
    person: String,
}
#[derive(Deserialize)]
struct Audit {
    id: String,
    prefix_sha256: String,
    region: [usize; 2],
    expected_request: [Option<usize>; 2],
    expected_excluded: [Option<usize>; 2],
    baseline_request: [Option<usize>; 2],
    masked_request: [Option<usize>; 2],
    baseline_words: [u32; 8],
    masked_words: [u32; 8],
}

#[derive(Clone, Copy)]
struct Candidate {
    span: RoleSpan,
    scores: [i32; 2],
}

fn rank(candidates: &[Candidate], target: RoleSpan, role: usize) -> Value {
    let Some(found) = candidates.iter().find(|c| c.span == target) else {
        return Value::Null;
    };
    let score = found.scores[role];
    if score == i32::MIN {
        return Value::Null;
    }
    let above = candidates.iter().filter(|c| c.scores[role] > score).count();
    let equal = candidates
        .iter()
        .filter(|c| c.scores[role] == score)
        .count();
    json!({"rank":above+1,"ties":equal,"score":score})
}

fn endpoints(ends: [Option<usize>; 2]) -> Result<Option<RoleSpan>> {
    match ends {
        [None, None] => Ok(None),
        [Some(start), Some(end)] if start < end => Ok(Some(RoleSpan { start, end })),
        _ => Err("invalid paired endpoint".into()),
    }
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [tokenizer_path, bank_path, grade_path, audit_path] = args.as_slice() else {
        return Err("expected TOKENIZER BANK GRADE PAIRED_ROLE_ROWS".into());
    };
    let tokenizer_bytes = fs::read(tokenizer_path)?;
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes).ok_or("invalid tokenizer")?;
    let bank = PairedRoleState::from_bytes(&fs::read(bank_path)?)?;
    if bank.bindings()[1] != <[u8; 32]>::from(Sha256::digest(tokenizer_bytes)) {
        return Err("ranking tokenizer/bank binding differs".into());
    }
    let grades: Vec<Grade> = serde_json::from_slice(&fs::read(grade_path)?)?;
    let audits: Vec<Audit> = serde_json::from_slice(&fs::read(audit_path)?)?;
    if grades.len() != audits.len() {
        return Err("ranking grade/row length differs".into());
    }
    let mut entities: BTreeSet<_> = grades
        .iter()
        .flat_map(|g| &g.assertions)
        .map(|a| a.person.to_ascii_lowercase())
        .collect();
    // Named distractors embedded as literals in the registered question templates.
    entities.extend(["cedar", "harbor", "summit"].map(str::to_owned));
    for (grade, audit) in grades.iter().zip(&audits) {
        for label in [audit.expected_request, audit.expected_excluded] {
            if let Some(span) = endpoints(label)? {
                let surface = tokenizer.decode(
                    grade
                        .prefix
                        .get(span.start..span.end)
                        .ok_or("label outside prompt")?,
                );
                entities.insert(surface.trim().to_ascii_lowercase());
            }
        }
    }
    let mut output = Vec::new();
    for (grade, audit) in grades.into_iter().zip(audits) {
        let mut hash = Sha256::new();
        for &word in &grade.prefix {
            hash.update(word.to_le_bytes());
        }
        let [start, end] = audit.region;
        if grade.id != audit.id
            || format!("{:x}", hash.finalize()) != audit.prefix_sha256
            || bank.predict(&grade.prefix)?.words() != audit.baseline_words
            || bank.predict_in_region(&grade.prefix, start..end)?.words() != audit.masked_words
        {
            return Err(format!("ranking raw decision identity differs: {}", grade.id).into());
        }
        let mut candidates = Vec::new();
        for (span, _) in role_candidates(bank.sites(), &grade.prefix)? {
            // A whole-word candidate is unique inside its own exact interval.
            let d = bank.predict_in_region(&grade.prefix, span.start..span.end)?;
            candidates.push(Candidate {
                span,
                scores: [d.requested_score, d.excluded_score],
            });
        }
        let masked: Vec<_> = candidates
            .iter()
            .copied()
            .filter(|c| c.span.start >= start && c.span.end <= end)
            .collect();
        let nominated = |span: Option<RoleSpan>| -> Value {
            span.map_or(Value::Null, |s| {
                let text = tokenizer.decode(&grade.prefix[s.start..s.end]);
                json!({"span":[s.start,s.end],"literal":text,"entity":entities.contains(&text.trim().to_ascii_lowercase())})
            })
        };
        let requested = endpoints(audit.expected_request)?;
        for span in [requested, endpoints(audit.expected_excluded)?]
            .into_iter()
            .flatten()
        {
            if !masked.iter().any(|c| c.span == span) {
                return Err(
                    format!("annotated role is not a whole candidate: {}", grade.id).into(),
                );
            }
        }
        let expected = requested.map(|span| {
            json!({"word":nominated(Some(span)),
                "original_requested_rank":rank(&candidates, span, 0),
                "original_excluded_rank":rank(&candidates, span, 1),
                "region_requested_rank":rank(&masked, span, 0),
                "region_excluded_rank":rank(&masked, span, 1)})
        });
        output.push(
            json!({"id":grade.id,"prefix_sha256":audit.prefix_sha256,"expected":expected,
            "original_nominee":nominated(endpoints(audit.baseline_request)?),
            "region_nominee":nominated(endpoints(audit.masked_request)?)}),
        );
    }
    serde_json::to_writer(io::stdout().lock(), &output)?;
    Ok(())
}
