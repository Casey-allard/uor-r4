#![forbid(unsafe_code)]
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;
const ARMS: [&str; 5] = [
    "baseline",
    "fact_true",
    "fact_reversed",
    "candidate_true",
    "candidate_reversed",
];
const MODES: [&str; 2] = ["ordinary", "supplied"];
const NIL: u32 = u32::MAX;
const EOS: u32 = 1;
const COPY_STOPS: [u32; 4] = [16, EOS, 0, 201];
const COPY_CAP: u32 = 8;
const TOKENIZER_SHA256: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";
type Span = [usize; 2];

#[derive(Deserialize)]
struct Key {
    row: u32,
    group: Value,
    stratum: String,
    original_ids: Vec<u32>,
    query: Option<Span>,
    fact: Option<Span>,
    excluded: Option<Span>,
    factors: Option<[usize; 4]>,
    gold_source: Option<Span>,
    expected_tape: Option<Vec<u32>>,
}

#[derive(Deserialize)]
struct Selection {
    source: Option<usize>,
}
#[derive(Deserialize)]
struct Paired {
    role_words: Option<[u32; 8]>,
}
#[derive(Deserialize)]
struct Capture {
    artifact_sha256: String,
    tokenizer_sha256: String,
    prompt_ids: Vec<u32>,
    generated_ids: Vec<u32>,
    selected: Selection,
    paired: Paired,
    stop: String,
    role_replay: Value,
    copy_step_words: Vec<[u32; 8]>,
}

fn check(ok: bool, reason: impl Into<String>) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(reason.into().into())
    }
}
fn first_difference(a: &[u32], b: &[u32]) -> Option<Value> {
    (0..a.len().max(b.len()))
        .find(|&i| a.get(i) != b.get(i))
        .map(|i| json!({"offset":i,"expected":a.get(i),"actual":b.get(i)}))
}
fn class_span(words: &Option<[u32; 8]>, offset: usize) -> Option<Span> {
    words.as_ref().and_then(|w| {
        (w[offset] != NIL && w[offset + 1] != NIL)
            .then_some([w[offset] as usize, w[offset + 1] as usize])
    })
}
fn valid_key(k: &Key) -> Result<()> {
    for [begin, end] in [k.query, k.fact, k.gold_source].into_iter().flatten() {
        check(
            begin < end && end <= k.original_ids.len(),
            "gold span outside original prompt",
        )?;
    }
    if let (Some(q), Some(f)) = (k.query, k.fact) {
        check(
            k.original_ids[q[0]..q[1]] == k.original_ids[f[0]..f[1]],
            "query and factual person words differ",
        )?;
    }
    match (k.gold_source, &k.expected_tape) {
        (Some([s, e]), Some(t)) => check(
            !t.is_empty()
                && t.last() == Some(&EOS)
                && !t[..t.len() - 1].contains(&EOS)
                && k.original_ids[s..e] == t[..t.len() - 1],
            "gold phrase fails original-token writer check",
        )?,
        (None, None) => (),
        _ => return Err("onset and keyed reply presence differ".into()),
    }
    Ok(())
}

fn audit_writer(c: &Capture) -> Result<()> {
    let prompt_end = u32::try_from(c.prompt_ids.len())?;
    let mut active = c.selected.source.is_some();
    let source = match c.selected.source {
        Some(source) => {
            check(
                source < c.prompt_ids.len(),
                "copy source outside original prompt",
            )?;
            u32::try_from(source)?
        }
        None => NIL,
    };
    let mut cursor = source;
    for (offset, (&id, step)) in c.generated_ids.iter().zip(&c.copy_step_words).enumerate() {
        // The ordinary predictor is outside the writer audit. Only a copy or
        // delimiter-driven EOS forces its output independently of that predictor.
        let mut emitted = id;
        let mut exact_id = NIL;
        let mut copied = 0;
        let mut release = 0;
        if active {
            if cursor - source == COPY_CAP {
                release = 3;
            } else if cursor == prompt_end {
                release = 2;
            } else {
                let token = c.prompt_ids[cursor as usize];
                if COPY_STOPS.contains(&token) {
                    release = 1;
                    if cursor > source {
                        emitted = EOS;
                    }
                } else {
                    cursor += 1;
                    emitted = token;
                    exact_id = token;
                    copied = 1;
                }
            }
            active = release == 0;
        }
        let expected = [
            u32::from(active),
            source,
            cursor,
            prompt_end,
            emitted,
            exact_id,
            copied,
            release,
        ];
        check(
            emitted == id && *step == expected,
            format!("invalid original writer transition at generated offset {offset}"),
        )?;
    }
    check(
        c.selected.source.is_none() || cursor > source,
        "claimed copy without an emitted source token",
    )
}

fn audit(k: &Key, c: &Capture, digest: &str, mode: &str) -> Result<Value> {
    check(c.artifact_sha256 == digest, "artifact identity")?;
    check(c.tokenizer_sha256 == TOKENIZER_SHA256, "tokenizer identity")?;
    check(c.prompt_ids == k.original_ids, "prefix identity")?;
    let ids = &c.generated_ids;
    check(
        !ids.is_empty() && ids.len() == c.copy_step_words.len(),
        "token/step count differs",
    )?;
    check(
        !ids[..ids.len() - 1].contains(&EOS)
            && c.stop
                == if ids.last() == Some(&EOS) {
                    "eos"
                } else {
                    "max_tokens"
                },
        "EOS/stop mismatch",
    )?;
    audit_writer(c)?;
    let start = c.selected.source;
    let query = class_span(&c.paired.role_words, 0);
    let excluded = class_span(&c.paired.role_words, 2);
    let assertions = c
        .role_replay
        .get("assertions")
        .and_then(Value::as_array)
        .ok_or("missing factual trace")?;
    let bound = k.fact.is_some_and(|f| {
        assertions
            .iter()
            .any(|a| a.get("fact_span") == Some(&json!(f)))
    });
    let fence = k.fact.and_then(|f| {
        assertions
            .iter()
            .find(|a| a.get("fact_span") == Some(&json!(f)))
            .and_then(|a| a.pointer("/assertion/fence")?.as_u64())
    });
    let writer_envelope = k.gold_source.map(|[s, e]| match fence {
        None => "no_bound_fence",
        Some(f) if f != e as u64 => "wrong_fence",
        Some(_) if e - s >= COPY_CAP as usize => "over_budget",
        Some(_) => "within_budget",
    });
    let contextual = c
        .role_replay
        .get("contextual_sources")
        .and_then(Value::as_array)
        .ok_or("missing source trace")?;
    let onset = k.gold_source.map(|s| s[0]);
    let eligible = k.fact.zip(onset).is_some_and(|(f, g)| {
        contextual
            .iter()
            .filter(|a| a.get("fact_span") == Some(&json!(f)))
            .filter_map(|a| a.get("candidates")?.as_array())
            .flatten()
            .any(|c| {
                c.pointer("/1/eligible") == Some(&Value::Bool(true))
                    && c.pointer("/1/source") == Some(&json!(g))
            })
    });
    let eligible_complete = k.fact.zip(k.gold_source).is_some_and(|(f, g)| {
        contextual
            .iter()
            .filter(|a| a.get("fact_span") == Some(&json!(f)))
            .filter_map(|a| a.get("candidates")?.as_array())
            .flatten()
            .any(|c| {
                c.pointer("/1/eligible") == Some(&Value::Bool(true))
                    && c.pointer("/0/completed_source_range") == Some(&json!(g))
            })
    });
    let expected = k.expected_tape.as_ref();
    let mismatch = expected.and_then(|t| first_difference(t, ids));
    let exact = expected.map(|_| mismatch.is_none() && c.stop == "eos");
    let cursor_eos = c
        .copy_step_words
        .last()
        .is_some_and(|s| s[7] == 1 && s[2] > s[1] && s[4] == EOS);
    Ok(
        json!({"mode":mode,"role_words":c.paired.role_words,"requested":query,"requested_correct":k.query.map(|q|query==Some(q)),
         "fence":fence,"writer_envelope":writer_envelope,"excluded":excluded,"excluded_correct":excluded==k.excluded,"eligible_complete":onset.map(|_|eligible_complete),
         "assertion_bound":k.fact.map(|_|bound),"eligible_gold":onset.map(|_|eligible),
         "selected_source":start,"correct_source":onset.map(|s|start==Some(s)),
         "wrong_source":start.is_some() && onset.is_some() && start!=onset,
         "unsupported_copy":start.is_some() && onset.is_none(),"withheld":start.is_none(),
         "first_wrong":mismatch,"generated":ids,"exact_eos":exact,"stop":c.stop,"copy_released_at_eos":cursor_eos}),
    )
}

fn prepare(root: &Path, mode: &str, arm: &str, ids: &BTreeSet<u32>) -> Result<PathBuf> {
    let p = root.join(mode).join(arm);
    let mut found = BTreeSet::new();
    for ent in fs::read_dir(&p)? {
        let path = ent?.path();
        if path.extension().is_some_and(|s| s == "json") {
            let id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or("invalid row file")?
                .parse::<u32>()?;
            check(found.insert(id), "duplicate row filename")?;
        }
    }
    check(
        &found == ids,
        format!("capture row set differs in {}", p.display()),
    )?;
    Ok(p)
}

fn run() -> Result<Value> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    let [key, primary, cold, artifacts] = a.as_slice() else {
        return Err("usage: d35-value-grade HELD_KEY PRIMARY COLD ARM_HASHES_JSON".into());
    };
    let keys: Vec<Key> = serde_json::from_slice(&fs::read(key)?)?;
    let hashes: BTreeMap<String, String> = serde_json::from_slice(&fs::read(artifacts)?)?;
    for arm in ARMS {
        check(
            hashes
                .get(arm)
                .is_some_and(|h| h.len() == 64 && h.bytes().all(|c| c.is_ascii_hexdigit())),
            "arm digest missing",
        )?
    }
    check(
        hashes["fact_true"] == hashes["fact_reversed"]
            && hashes["candidate_true"] == hashes["candidate_reversed"],
        "orientation control uses different state",
    )?;
    let mut all = BTreeSet::new();
    for k in &keys {
        valid_key(k)?;
        check(all.insert(k.row), "duplicate key row")?
    }
    check(!all.is_empty(), "empty held panel")?;
    let mut results = Vec::new();
    for mode in MODES {
        let valid: BTreeSet<_> = keys
            .iter()
            .filter(|k| mode == "ordinary" || (k.query.is_some() && k.fact.is_some()))
            .map(|k| k.row)
            .collect();
        let mut folders = BTreeMap::new();
        for arm in ARMS {
            folders.insert(
                arm,
                (
                    prepare(Path::new(primary), mode, arm, &valid)?,
                    prepare(Path::new(cold), mode, arm, &valid)?,
                ),
            );
        }
        for k in keys.iter().filter(|k| valid.contains(&k.row)) {
            let mut arms = BTreeMap::new();
            let mut role = None;
            for arm in ARMS {
                let (p, c) = &folders[arm];
                let name = format!("{}.json", k.row);
                let bytes = fs::read(p.join(&name))?;
                check(
                    bytes == fs::read(c.join(&name))?,
                    format!("cold replay differs for {mode}/{arm}/{name}"),
                )?;
                let capture: Capture = serde_json::from_slice(&bytes)?;
                if arm == "baseline" {
                    role = Some(capture.paired.role_words)
                }
                check(
                    role == Some(capture.paired.role_words),
                    format!("rank changed role in {mode}/{arm}/{name}"),
                )?;
                arms.insert(
                    arm,
                    audit(k, &capture, &hashes[arm], mode)
                        .map_err(|e| format!("{mode}/{arm}/{name}: {e}"))?,
                );
            }
            results.push(json!({"row":k.row,"mode":mode,"group":k.group,"stratum":k.stratum,"query":k.query,"fact":k.fact,
                "excluded":k.excluded,"factors":k.factors,"gold_source":k.gold_source,"expected":k.expected_tape,"arms":arms}));
        }
    }
    Ok(json!({"schema":"uor-r4.d35-value-first-wrong/1","n_held":all.len(),"rows":results}))
}
fn main() -> Result<()> {
    println!("{}", serde_json::to_string_pretty(&run()?)?);
    Ok(())
}
