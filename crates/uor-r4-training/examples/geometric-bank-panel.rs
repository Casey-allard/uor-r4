//! Prospectively freeze preserved single-source and query-conditioned bank data.
//! Rust token preparation only; all labels and typed facts stay outside serving packets.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{
    answer_oracle::{FrozenAnswers, RecordedValueIntent},
    report_output,
};
use uor_r4_integer::{
    geometric_source_actions::SourceActionBinding, geometric_source_realizer::NativeArtifactBinding,
};
use uor_r4_tokenizer::{dialogue::SCHEMA_V2, ByteBpeTokenizer};
use uor_r4_training::{
    geometric_source_emission_view::SourceEmissionCompiler, sha256_bytes, sha256_file,
};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const LIMIT: usize = 64 * 1024 * 1024;
const JOB: &str = "Remind me what my job is.";
const WHERE: &str = "Remind me where I live.";
const STRATA: [&str; 4] = [
    "roles-order",
    "distractors",
    "repetition",
    "currentversions",
];
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    // Kept as a validated, unused placeholder in fresh-only mode.
    dev_out: PathBuf,
    #[serde(default)]
    fresh_only: bool,
    /// Prospective source-only pool expansion after retained tuples are exhausted.
    /// Never selected using model predictions or target support.
    #[serde(default)]
    fresh_literal_pool: Vec<String>,
    fresh_out: PathBuf,
    tokenizer: PathBuf,
    trusted_binding: PathBuf,
    compiled_panel: PathBuf,
    seed: u64,
    exposed_panel_roots: Vec<Exposure>,
    exposed_bank_spec: PathBuf,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Exposure {
    root: PathBuf,
    manifest_sha256: String,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Packet {
    id: String,
    segments: Vec<Segment>,
    query_ids: Vec<u32>,
    actual_prefix_ids: Vec<u32>,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
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
#[derive(Deserialize)]
struct Compiled {
    training64: Vec<Episode>,
}
#[derive(Deserialize)]
struct Episode {
    id: String,
    record: u64,
    commit: u64,
    entity: Vec<u32>,
    relation: u32,
    original_source_ids: Vec<u32>,
    query_ids: Vec<u32>,
    target_ids_labels_only: Vec<u32>,
    source_view: Value,
    accepted: Vec<String>,
    literal: String,
}
#[derive(Clone, Serialize)]
struct Fact {
    role: String,
    literal: String,
    event: u64,
    record: u64,
    commit: u64,
}
struct Authored {
    packet: Packet,
    kind: &'static str,
    stratum: &'static str,
    pair_id: Option<String>,
    query_role: &'static str,
    source_tuple: Vec<String>,
    source_views: Vec<Value>,
    facts: Vec<Fact>,
    accepted: Vec<String>,
    parent_id: Option<String>,
    original_target: Option<Vec<u32>>,
}
struct Prepared {
    packets: Vec<Packet>,
    labels: Vec<Value>,
    data: Vec<Value>,
}
fn invalid(s: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, s.into())
}
fn read<T: serde::de::DeserializeOwned>(p: &Path) -> Result<T> {
    let b = fs::read(p)?;
    if b.len() > LIMIT {
        return Err(invalid("input cap exceeded").into());
    }
    Ok(serde_json::from_slice(&b)?)
}
fn write(root: &Path, name: &str, v: &Value) -> Result<()> {
    let b = serde_json::to_vec(v)?;
    let used = fs::read_dir(root)?.try_fold(0usize, |sum, e| -> io::Result<usize> {
        Ok(sum + e?.metadata()?.len() as usize)
    })?;
    if used.saturating_add(b.len()) > LIMIT - 128 * 1024 {
        return Err(invalid("preparation report cap").into());
    }
    fs::write(root.join(name), b)?;
    Ok(())
}
fn limit(start: Instant) -> Result<()> {
    if start.elapsed().as_secs() >= 300 {
        return Err(invalid("prospective preparation300s cap").into());
    }
    Ok(())
}
fn horizon(projected: usize, query: usize) -> Result<usize> {
    let n = projected
        .checked_add(query)
        .and_then(|x| x.checked_add(32))
        .ok_or_else(|| invalid("horizon overflow"))?;
    if n > 128 {
        return Err(invalid("context+query+32generation exceeds128").into());
    }
    Ok(n)
}
fn encoded(tok: &ByteBpeTokenizer, text: &str, vocab: usize) -> Result<Vec<u32>> {
    let ids = tok.encode(text);
    if ids.is_empty() || ids.iter().any(|i| *i as usize >= vocab) || tok.decode(&ids) != text {
        return Err(invalid("tokenizer roundtrip/admission failed").into());
    }
    Ok(ids)
}
fn valid_sha(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}
fn args() -> Result<Args> {
    let mut argv = std::env::args().skip(1);
    let p = argv
        .next()
        .ok_or_else(|| invalid("one JSON config required"))?;
    if argv.next().is_some() {
        return Err(invalid("one config only").into());
    }
    let a: Args = read(Path::new(&p))?;
    let dev = output_support::prospective_output(&a.dev_out)?;
    let fresh = output_support::prospective_output(&a.fresh_out)?;
    if dev.starts_with(&fresh)
        || fresh.starts_with(&dev)
        || a.seed == 0
        || a.exposed_panel_roots.is_empty()
    {
        return Err(invalid("distinct roots/nonzero seed/exposures required").into());
    }
    if (!a.fresh_only && !a.fresh_literal_pool.is_empty())
        || a.fresh_literal_pool.len() > 64
        || a.fresh_literal_pool.iter().any(|s| {
            s.is_empty() || s.len() > 256 || s.trim() != s || s.chars().any(char::is_control)
        })
    {
        return Err(invalid(
            "fresh literal expansion requires fresh-only and bounded clean literals",
        )
        .into());
    }
    for input in [
        &a.tokenizer,
        &a.trusted_binding,
        &a.compiled_panel,
        &a.exposed_bank_spec,
    ]
    .into_iter()
    .chain(a.exposed_panel_roots.iter().map(|e| &e.root))
    {
        let p = fs::canonicalize(input)?;
        if dev.starts_with(&p) || fresh.starts_with(&p) {
            return Err(invalid("output beneath input").into());
        }
    }
    if a.exposed_panel_roots
        .iter()
        .any(|e| !valid_sha(&e.manifest_sha256))
    {
        return Err(invalid("exposure manifest digest invalid").into());
    }
    Ok(a)
}
fn nearest_seal(path: &Path) -> Result<PathBuf> {
    path.ancestors()
        .find(|p| p.join(report_output::MANIFEST_FILE).is_file())
        .map(Path::to_path_buf)
        .ok_or_else(|| invalid("sealed input ancestor absent").into())
}
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn index(&mut self, n: usize) -> Result<usize> {
        if n == 0 {
            return Err(invalid("empty literal pool").into());
        }
        Ok((self.next() % n as u64) as usize)
    }
}
fn tuple_key(values: &[String]) -> Result<String> {
    Ok(sha256_bytes(&serde_json::to_vec(values)?))
}
fn bank_key(packet: &Packet) -> Result<String> {
    Ok(sha256_bytes(&serde_json::to_vec(&packet.segments)?))
}
fn exposure_tuples(
    value: &Value,
    tok: &ByteBpeTokenizer,
    tuples: &mut BTreeSet<String>,
) -> Result<usize> {
    let mut found = 0;
    if let Some(cases) = value.get("cases").and_then(Value::as_array) {
        for case in cases {
            let mut tuple = Vec::new();
            if let Some(segments) = case.get("segments").and_then(Value::as_array) {
                for s in segments {
                    if s["kind"] == "Source" {
                        let ids: Vec<u32> =
                            serde_json::from_value(s["original_source_ids"].clone())?;
                        tuple.push(tok.decode(&ids));
                    }
                }
            } else if let Some(ids) = case.get("original_source_ids") {
                tuple.push(tok.decode(&serde_json::from_value::<Vec<u32>>(ids.clone())?));
            }
            if !tuple.is_empty() {
                tuples.insert(tuple_key(&tuple)?);
                found += 1;
            }
        }
    }
    if let Some(rows) = value.get("rows").and_then(Value::as_array) {
        for row in rows {
            if let Some(segments) = row.get("segments").and_then(Value::as_array) {
                let tuple = segments
                    .iter()
                    .filter(|s| s["kind"] == "source")
                    .map(|s| {
                        s["literal"]
                            .as_str()
                            .map(str::to_owned)
                            .ok_or_else(|| invalid("exposed literal missing"))
                    })
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                if !tuple.is_empty() {
                    tuples.insert(tuple_key(&tuple)?);
                    found += 1;
                }
            }
        }
    }
    Ok(found)
}
fn typed_answer(facts: &[Fact], role: &str) -> Result<String> {
    facts
        .iter()
        .rev()
        .find(|f| f.role == role)
        .map(|f| format!("{}.", f.literal))
        .ok_or_else(|| invalid("typed current fact absent").into())
}
fn add_fact(
    segments: &mut Vec<Segment>,
    views: &mut Vec<Value>,
    facts: &mut Vec<Fact>,
    role: &str,
    literal: &str,
    event: u64,
    commit: u64,
    tok: &ByteBpeTokenizer,
    compiler: &SourceEmissionCompiler,
    vocab: usize,
) -> Result<()> {
    let (cue, record, relation) = match role {
        "job" => ("I work as a", 2, 7),
        "where" => ("I live in", 3, 8),
        "other" => ("I mentioned", 4, 9),
        _ => return Err(invalid("unknown authored role").into()),
    };
    segments.push(Segment::Context {
        event,
        role: 1,
        token_ids: encoded(tok, cue, vocab)?,
    });
    let original = encoded(tok, literal, vocab)?;
    let view = compiler.compile(&original)?;
    let index = segments.len();
    views.push(json!({"segment_index":index,"source_view":view}));
    segments.push(Segment::Source {
        event,
        record,
        commit,
        scope: "m-world-v2".into(),
        entity: vec![644, 284],
        relation,
        view: 0,
        original_source_ids: original,
    });
    facts.push(Fact {
        role: role.into(),
        literal: literal.into(),
        event,
        record,
        commit,
    });
    Ok(())
}
fn project(segments: &[Segment], views: &[Value]) -> Result<usize> {
    let mut n = 0usize;
    for (i, s) in segments.iter().enumerate() {
        let count = match s {
            Segment::Context { token_ids, .. } => token_ids.len(),
            Segment::Source { .. } => views
                .iter()
                .find(|v| v["segment_index"] == i)
                .and_then(|v| v["source_view"]["emitted_token_ids"].as_array())
                .ok_or_else(|| invalid("projection missing"))?
                .len(),
        };
        n = n
            .checked_add(count)
            .ok_or_else(|| invalid("projection overflow"))?;
    }
    Ok(n)
}
fn finish(
    rows: Vec<Authored>,
    tok: &ByteBpeTokenizer,
    binding: &SourceActionBinding,
) -> Result<Prepared> {
    // Source packets have already been fully constructed; labels/targets never
    // enter source emission compilation or runtime frame admission.
    let mut packets = Vec::new();
    let mut labels = Vec::new();
    let mut data = Vec::new();
    for row in rows {
        let answers = FrozenAnswers {
            intent: RecordedValueIntent::Current,
            accepted: row.accepted,
        };
        answers.validate()?;
        let mut target = encoded(
            tok,
            &format!(" {}", answers.accepted[0]),
            binding.vocab_size(),
        )?;
        target.push(binding.eos_token_id());
        if target.len() > 32 {
            return Err(invalid("complete answerEOS exceeds32").into());
        }
        if row
            .original_target
            .as_ref()
            .is_some_and(|old| *old != target)
        {
            return Err(invalid("preserved64 target differs").into());
        }
        let projected = project(&row.packet.segments, &row.source_views)?;
        let admitted = horizon(projected, row.packet.query_ids.len())?;
        labels.push(json!({"id":row.packet.id,"answers":answers}));
        data.push(json!({"id":row.packet.id,"kind":row.kind,"stratum":row.stratum,"pair_id":row.pair_id,"query_role":row.query_role,"source_tuple":row.source_tuple,"source_views":row.source_views,"typed_facts":row.facts,"target_ids_labels_only":target,"parent_id":row.parent_id,"projected_context_tokens":projected,"admitted_horizon_including32":admitted}));
        packets.push(row.packet);
    }
    Ok(Prepared {
        packets,
        labels,
        data,
    })
}
fn pairs_valid(packets: &[Packet], data: &[Value], expected_pairs: usize) -> Result<()> {
    if packets.len() != data.len() {
        return Err(invalid("pair packet/receipt lengths differ").into());
    }
    let mut pairs: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, d) in data.iter().enumerate() {
        if let Some(pair) = d["pair_id"].as_str() {
            pairs.entry(pair.into()).or_default().push(i);
        }
    }
    if pairs.len() != expected_pairs {
        return Err(invalid("pair count differs").into());
    }
    for indices in pairs.values() {
        if indices.len() != 2 {
            return Err(invalid("two queries perbank required").into());
        }
        let (a, b) = (indices[0], indices[1]);
        if packets[a].segments != packets[b].segments
            || packets[a].actual_prefix_ids != packets[b].actual_prefix_ids
            || packets[a].query_ids == packets[b].query_ids
            || data[a]["source_views"] != data[b]["source_views"]
            || data[a]["source_tuple"] != data[b]["source_tuple"]
            || data[a]["query_role"] != "job"
            || data[b]["query_role"] != "where"
        {
            return Err(invalid("samebank queryswap invariant differs").into());
        }
    }
    Ok(())
}
fn bank_rows(
    split: &str,
    per_stratum: usize,
    rng: &mut Rng,
    single: &[String],
    multi: &[String],
    excluded: &BTreeSet<String>,
    used: &mut BTreeSet<String>,
    fingerprints: &mut BTreeSet<String>,
    tok: &ByteBpeTokenizer,
    compiler: &SourceEmissionCompiler,
    binding: &SourceActionBinding,
    start: Instant,
    rejections: &mut Vec<Value>,
) -> Result<Vec<Authored>> {
    let mut rows = Vec::new();
    for stratum in STRATA {
        for bank in 0..per_stratum {
            let pool = if bank % 2 == 0 { single } else { multi };
            let mut accepted = None;
            for attempt in 0..4096 {
                limit(start)?;
                let mut values = Vec::new();
                let need = if stratum == "currentversions" {
                    4
                } else if stratum == "distractors" {
                    3
                } else {
                    2
                };
                for _draw in 0..128 {
                    if values.len() == need {
                        break;
                    }
                    let value = pool[rng.index(pool.len())?].clone();
                    if !values.contains(&value) {
                        values.push(value);
                    }
                }
                if values.len() != need {
                    rejections.push(json!({"split":split,"stratum":stratum,"bank":bank,"attempt":attempt,"reason":"bounded_distinct_pool_draw_exhausted"}));
                    continue;
                }
                let mut segments = Vec::new();
                let mut views = Vec::new();
                let mut facts = Vec::new();
                let mut sequence = Vec::new();
                match stratum {
                    "roles-order" => {
                        sequence.push(("job", values[0].clone(), 10, 2));
                        sequence.push(("where", values[1].clone(), 20, 2));
                        if bank % 4 >= 2 {
                            sequence.swap(0, 1);
                        }
                    }
                    "distractors" => {
                        sequence.push(("job", values[0].clone(), 10, 2));
                        sequence.push(("where", values[1].clone(), 20, 2));
                        if bank % 4 >= 2 {
                            sequence.swap(0, 1);
                        }
                        let other = ("other", values[2].clone(), 30, 2);
                        if bank % 4 < 2 {
                            sequence.push(other);
                        } else {
                            sequence.insert(0, other);
                        }
                    }
                    "repetition" => {
                        sequence.push(("job", format!("{} {}", values[0], values[0]), 10, 2));
                        sequence.push(("where", format!("{} {}", values[1], values[1]), 20, 2));
                        if bank % 4 >= 2 {
                            sequence.swap(0, 1);
                        }
                    }
                    "currentversions" => {
                        sequence.extend([
                            ("job", values[0].clone(), 10, 2),
                            ("where", values[1].clone(), 20, 2),
                            ("job", values[2].clone(), 40, 3),
                            ("where", values[3].clone(), 50, 3),
                        ]);
                        if bank % 4 >= 2 {
                            sequence.swap(0, 1);
                            sequence.swap(2, 3);
                        }
                    }
                    _ => return Err(invalid("stratum unavailable").into()),
                }
                for (role, literal, event, commit) in sequence {
                    add_fact(
                        &mut segments,
                        &mut views,
                        &mut facts,
                        role,
                        &literal,
                        event,
                        commit,
                        tok,
                        compiler,
                        binding.vocab_size(),
                    )?;
                }
                let tuple = facts.iter().map(|f| f.literal.clone()).collect::<Vec<_>>();
                let tuple_sha = tuple_key(&tuple)?;
                let packet = Packet {
                    id: String::new(),
                    segments,
                    query_ids: Vec::new(),
                    actual_prefix_ids: Vec::new(),
                };
                let bank_sha = bank_key(&packet)?;
                let reason = if excluded.contains(&tuple_sha) {
                    Some("exposed_source_tuple")
                } else if used.contains(&tuple_sha) {
                    Some("dev_or_fresh_source_tuple_duplicate")
                } else if fingerprints.contains(&bank_sha) {
                    Some("bank_packet_duplicate")
                } else {
                    None
                };
                if let Some(reason) = reason {
                    rejections.push(json!({"split":split,"stratum":stratum,"bank":bank,"attempt":attempt,"source_tuple":tuple,"reason":reason}));
                    continue;
                }
                let projected = project(&packet.segments, &views)?;
                let mut candidate = Vec::new();
                let pair_id = format!("{split}-{stratum}-{bank:02}");
                let mut invalid_admission = None;
                for (role, query) in [("job", JOB), ("where", WHERE)] {
                    let query_ids = encoded(tok, query, binding.vocab_size())?;
                    let answer = typed_answer(&facts, role)?;
                    let target_length =
                        encoded(tok, &format!(" {answer}"), binding.vocab_size())?.len() + 1;
                    if horizon(projected, query_ids.len()).is_err() || target_length > 32 {
                        invalid_admission = Some("token_horizon_or_completeanswer32");
                        break;
                    }
                    let mut p = packet.clone();
                    p.id = format!("{pair_id}-{role}");
                    p.query_ids = query_ids;
                    candidate.push(Authored {
                        packet: p,
                        kind: "bank",
                        stratum,
                        pair_id: Some(pair_id.clone()),
                        query_role: role,
                        source_tuple: tuple.clone(),
                        source_views: views.clone(),
                        facts: facts.clone(),
                        accepted: vec![answer],
                        parent_id: None,
                        original_target: None,
                    });
                }
                if let Some(reason) = invalid_admission {
                    rejections.push(json!({"split":split,"stratum":stratum,"bank":bank,"attempt":attempt,"source_tuple":tuple,"reason":reason}));
                    continue;
                }
                used.insert(tuple_sha);
                fingerprints.insert(bank_sha);
                accepted = Some(candidate);
                break;
            }
            rows.extend(accepted.ok_or_else(|| {
                invalid(format!(
                    "finite prospective sampling exhausted {split}/{stratum}/{bank}"
                ))
            })?);
        }
    }
    Ok(rows)
}
fn run(a: &Args, start: Instant) -> Result<()> {
    let mut hashes = BTreeMap::new();
    for p in [
        &a.tokenizer,
        &a.trusted_binding,
        &a.compiled_panel,
        &a.exposed_bank_spec,
    ] {
        hashes.insert(p.to_string_lossy().into_owned(), sha256_file(p)?);
    }
    let compiled_seal = nearest_seal(&a.compiled_panel)?;
    report_output::verify(&compiled_seal)?;
    let expected: NativeArtifactBinding = read(&a.trusted_binding)?;
    let bytes = fs::read(&a.tokenizer)?;
    if sha256_file(&a.tokenizer)? != expected.identity.tokenizer_sha256 {
        return Err(invalid("trusted tokenizer differs").into());
    }
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("tokenizer JSON invalid"))?;
    let binding = SourceActionBinding::new(&bytes)?;
    if binding.protocol().schema != SCHEMA_V2 {
        return Err(invalid("dialogue2 required").into());
    }
    let compiler = SourceEmissionCompiler::new(&bytes)?;
    let compiled: Compiled = read(&a.compiled_panel)?;
    if compiled.training64.len() != 64 {
        return Err(invalid("unchanged64 source rows required").into());
    }
    let mut authored = Vec::new();
    let mut literals = BTreeSet::new();
    let mut excluded = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut golden = Vec::new();
    for e in compiled.training64 {
        limit(start)?;
        if !ids.insert(e.id.clone())
            || e.accepted.is_empty()
            || tok.decode(&e.original_source_ids) != e.literal
            || e.query_ids.is_empty()
        {
            return Err(invalid("compiled64 source/query/answer identity differs").into());
        }
        let view = compiler.compile(&e.original_source_ids)?;
        if serde_json::to_value(&view)? != e.source_view {
            return Err(invalid("compiled64 emissionview differs").into());
        }
        let packet = Packet {
            id: e.id.clone(),
            segments: vec![Segment::Source {
                event: 10,
                record: e.record,
                commit: e.commit,
                scope: "m-world-v2".into(),
                entity: e.entity,
                relation: e.relation,
                view: 0,
                original_source_ids: e.original_source_ids.clone(),
            }],
            query_ids: e.query_ids,
            actual_prefix_ids: vec![],
        };
        horizon(view.emitted_token_ids().len(), packet.query_ids.len())?;
        literals.insert(e.literal.clone());
        excluded.insert(tuple_key(&[e.literal.clone()])?);
        golden.push(json!({"id":e.id,"packet_sha256":sha256_bytes(&serde_json::to_vec(&packet)?),"original_source_ids_sha256":sha256_bytes(&serde_json::to_vec(&e.original_source_ids)?),"query_ids_sha256":sha256_bytes(&serde_json::to_vec(&packet.query_ids)?),"source_view_sha256":sha256_bytes(&serde_json::to_vec(&view)?),"accepted_sha256":sha256_bytes(&serde_json::to_vec(&e.accepted)?),"target_ids_labels_only_sha256":sha256_bytes(&serde_json::to_vec(&e.target_ids_labels_only)?)}));
        authored.push(Authored {
            packet,
            kind: "single-source",
            stratum: "preservation",
            pair_id: None,
            query_role: "job",
            source_tuple: vec![e.literal],
            source_views: vec![json!({"segment_index":0,"source_view":view})],
            facts: vec![],
            accepted: e.accepted,
            parent_id: Some(e.id),
            original_target: Some(e.target_ids_labels_only),
        });
    }
    let mut exposures = Vec::new();
    for e in &a.exposed_panel_roots {
        report_output::verify(&e.root)?;
        if sha256_file(&e.root.join(report_output::MANIFEST_FILE))? != e.manifest_sha256 {
            return Err(invalid("exposed manifest differs").into());
        }
        let mut count = 0;
        let mut files = Vec::new();
        for name in ["inputs.json", "source-inputs.json"] {
            let path = e.root.join(name);
            if path.is_file() {
                count += exposure_tuples(&read::<Value>(&path)?, &tok, &mut excluded)?;
                files.push(json!({"path":name,"sha256":sha256_file(&path)?}));
            }
        }
        if count == 0 {
            return Err(invalid("declared exposure has no source-only packets").into());
        }
        exposures.push(json!({"root":e.root,"manifest_sha256":e.manifest_sha256,"source_packets":count,"files":files}));
    }
    exposure_tuples(&read::<Value>(&a.exposed_bank_spec)?, &tok, &mut excluded)?;
    for literal in &a.fresh_literal_pool {
        // Validate surface encoding only; no model or supported-answer probe.
        let ids = encoded(&tok, literal, binding.vocab_size())?;
        compiler.compile(&ids)?;
        literals.insert(literal.clone());
    }
    let single = literals
        .iter()
        .filter(|s| !s.contains(' '))
        .cloned()
        .collect::<Vec<_>>();
    let multi = literals
        .iter()
        .filter(|s| s.contains(' '))
        .cloned()
        .collect::<Vec<_>>();
    if single.len() < 4 || multi.len() < 4 {
        return Err(invalid("four distinct singleton/multiword values required").into());
    }
    let mut rng = Rng(a.seed);
    let mut used = BTreeSet::new();
    let mut fingerprints = BTreeSet::new();
    let mut rejections = Vec::new();
    let dev = if a.fresh_only {
        // Validate the retained typed answers and exact canonical targets without
        // drawing new banks, consuming RNG state, or creating development output.
        let preserved = finish(authored, &tok, &binding)?;
        if preserved.packets.len() != 64 {
            return Err(invalid("preserved64 input validation required").into());
        }
        None
    } else {
        authored.extend(bank_rows(
            "development",
            8,
            &mut rng,
            &single,
            &multi,
            &excluded,
            &mut used,
            &mut fingerprints,
            &tok,
            &compiler,
            &binding,
            start,
            &mut rejections,
        )?);
        let dev = finish(authored, &tok, &binding)?;
        if dev.packets.len() != 128 {
            return Err(invalid("development128 required").into());
        }
        pairs_valid(&dev.packets, &dev.data, 32)?;
        Some(dev)
    };
    let dev_tuple_keys = used.clone();
    let fresh_rows = bank_rows(
        "fresh",
        4,
        &mut rng,
        &single,
        &multi,
        &excluded,
        &mut used,
        &mut fingerprints,
        &tok,
        &compiler,
        &binding,
        start,
        &mut rejections,
    )?;
    let fresh = finish(fresh_rows, &tok, &binding)?;
    if fresh.packets.len() != 32 {
        return Err(invalid("fresh32 required").into());
    }
    pairs_valid(&fresh.packets, &fresh.data, 16)?;
    for d in &fresh.data {
        let tuple: Vec<String> = serde_json::from_value(d["source_tuple"].clone())?;
        let key = tuple_key(&tuple)?;
        if dev_tuple_keys.contains(&key) || excluded.contains(&key) {
            return Err(invalid("fresh tuple exposure collision").into());
        }
    }
    let exe = std::env::current_exe()?;
    let lookup = "current_exe";
    let common = json!({"schema":"uor-r4.geometric-bank-panel/1","status":"completed","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":sha256_file(&exe)?,"executable_lookup":lookup,"seed":a.seed,"rng":if a.fresh_only {"xorshift64;lexicallysorted_unique_compiled_literal_pool;fresh-only/2"} else {"xorshift64;lexicallysorted_unique_compiled_literal_pool;devthenfresh/1"},"fresh_only":a.fresh_only,"prospective_fresh_literal_pool":a.fresh_literal_pool,"all64_input_golden_validated":true,"development_output_created":!a.fresh_only,"queries":{"job":JOB,"where":WHERE},"trusted_binding":expected,"input_files_sha256":hashes,"exposed_roots":exposures,"source_tuple_exclusion":"ordered query-independent source literal sequence; not semantic metric","fresh_source_tuples_disjoint_declared_exposures_and_development":true,"samebank_queryswap_segments_exact":true,"all64_golden_preserved":if a.fresh_only {Value::Bool(false)} else {json!(golden)},"maximum_context_and_generation":128,"reserved_generation":32,"model_calls":0,"support_filters":false,"predictions":"NOT_RUN","labels_outside_serving_inputs":true,"metadata_roles_relations_are_opaque_provenance":true,"pool":{"singletons":single,"multiword":multi},"rejections":rejections,"elapsed_seconds":start.elapsed().as_secs_f64()});
    let mut outputs = Vec::new();
    if let Some(dev) = dev {
        outputs.push(("development", &a.dev_out, dev, 32));
    }
    outputs.push(("fresh", &a.fresh_out, fresh, 16));
    for (split, out, panel, pairs) in outputs {
        write(
            out,
            "inputs.json",
            &json!({"schema":"uor-r4.native-source-bank-probe-input/1","cases":panel.packets}),
        )?;
        write(
            out,
            "labels.json",
            &json!({"schema":"uor-r4.native-source-bank-labels/1","protocol":SCHEMA_V2,"membership_only":true,"cases":panel.labels}),
        )?;
        write(
            out,
            "context-data.json",
            &json!({"schema":"uor-r4.geometric-bank-context-data/1","split":split,"cases":panel.data}),
        )?;
        let mut report = common.clone();
        report["split"] = json!(split);
        report["cases"] = json!(if split == "development" { 128 } else { 32 });
        report["bank_pairs"] = json!(pairs);
        report["single_source_preservation_cases"] =
            json!(if split == "development" { 64 } else { 0 });
        report["bank_query_counts"] = json!({"job":pairs,"where":pairs});
        report["bank_stratum_episode_counts"] = json!(STRATA
            .iter()
            .map(|s| ((*s).to_owned(), pairs / 2))
            .collect::<BTreeMap<_, _>>());
        report["bank_base_value_class_counts"] =
            json!({"singleton_base_episodes":pairs,"multiword_base_episodes":pairs});
        report["inputs_sha256"] = json!(sha256_file(&out.join("inputs.json"))?);
        report["labels_sha256"] = json!(sha256_file(&out.join("labels.json"))?);
        report["context_data_sha256"] = json!(sha256_file(&out.join("context-data.json"))?);
        write(out, "report.json", &report)?;
    }
    for (path, expected) in common["input_files_sha256"]
        .as_object()
        .ok_or_else(|| invalid("input receipts absent"))?
    {
        if sha256_file(Path::new(path))?
            != expected
                .as_str()
                .ok_or_else(|| invalid("input digest absent"))?
        {
            return Err(invalid("input changed during preparation").into());
        }
    }
    limit(start)?;
    Ok(())
}
fn output_roots(a: &Args) -> Vec<&PathBuf> {
    if a.fresh_only {
        vec![&a.fresh_out]
    } else {
        vec![&a.dev_out, &a.fresh_out]
    }
}
fn main() -> Result<()> {
    let a = args()?;
    if !a.fresh_only {
        report_output::claim(&a.dev_out)?;
    }
    if let Err(e) = report_output::claim(&a.fresh_out) {
        if a.fresh_only {
            return Err(e.into());
        }
        write(&a.dev_out, "failure.json", &json!({"error":e.to_string()}))?;
        report_output::seal(&a.dev_out)?;
        return Err(e.into());
    }
    let start = Instant::now();
    let result = run(&a, start);
    if let Err(e) = &result {
        for out in output_roots(&a) {
            write(
                out,
                "failure.json",
                &json!({"status":"failed","error":e.to_string(),"model_calls":0}),
            )?;
        }
    }
    for out in output_roots(&a) {
        report_output::seal(out)?;
        report_output::verify(out)?;
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fresh_only_omits_development_root_and_is_opt_in() -> Result<()> {
        let config = json!({
            "dev_out":"unused-development-placeholder", "fresh_out":"fresh-output",
            "tokenizer":"tokenizer", "trusted_binding":"binding",
            "compiled_panel":"compiled", "seed":20261012,
            "exposed_panel_roots":[], "exposed_bank_spec":"bank-spec"
        });
        let default: Args = serde_json::from_value(config.clone())?;
        assert!(!default.fresh_only);
        assert_eq!(
            output_roots(&default),
            vec![&default.dev_out, &default.fresh_out]
        );
        let mut opted = config;
        opted["fresh_only"] = json!(true);
        let fresh: Args = serde_json::from_value(opted)?;
        assert_eq!(output_roots(&fresh), vec![&fresh.fresh_out]);
        Ok(())
    }
    #[test]
    fn preservation_keeps_all_answer_aliases_and_original_canonical_target() -> Result<()> {
        const TOKENIZER: &str = r#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5,"Ġ":6,"Ġa":7},"merges":["Ġ a"]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
        let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(TOKENIZER.as_bytes())
            .ok_or_else(|| invalid("fixture tokenizer invalid"))?;
        let binding = SourceActionBinding::new(TOKENIZER.as_bytes())?;
        let compiler = SourceEmissionCompiler::new(TOKENIZER.as_bytes())?;
        let view = compiler.compile(&[4])?;
        // The retained first twenty controls have these five rendering forms;
        // membership must preserve all of them, while CE uses the first only.
        let accepted: Vec<String> = [
            "a.",
            "It's a.",
            "It is a.",
            "You work as a a.",
            "Your job is a.",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        let canonical = vec![7, 3, 1];
        let prepared = finish(
            vec![Authored {
                packet: Packet {
                    id: "preserved-five-aliases".into(),
                    segments: vec![Segment::Source {
                        event: 10,
                        record: 2,
                        commit: 2,
                        scope: "m-world-v2".into(),
                        entity: vec![4],
                        relation: 7,
                        view: 0,
                        original_source_ids: vec![4],
                    }],
                    query_ids: vec![5],
                    actual_prefix_ids: vec![],
                },
                kind: "single-source",
                stratum: "preservation",
                pair_id: None,
                query_role: "job",
                source_tuple: vec!["a".into()],
                source_views: vec![json!({"segment_index":0,"source_view":view})],
                facts: vec![],
                accepted: accepted.clone(),
                parent_id: Some("old-control".into()),
                original_target: Some(canonical.clone()),
            }],
            &tok,
            &binding,
        )?;
        assert_eq!(prepared.labels[0]["answers"]["accepted"], json!(accepted));
        assert_eq!(prepared.data[0]["target_ids_labels_only"], json!(canonical));
        assert!(serde_json::to_value(&prepared.packets[0])?
            .get("answers")
            .is_none());
        Ok(())
    }
    #[test]
    fn bank_horizon_and_target_free_packet_contract() -> Result<()> {
        assert_eq!(horizon(87, 9)?, 128);
        assert!(horizon(88, 9).is_err());
        let packet = json!({"id":"a","segments":[{"kind":"Context","event":7,"role":1,"token_ids":[4]}],"query_ids":[5],"actual_prefix_ids":[]});
        let _: Packet = serde_json::from_value(packet.clone())?;
        let mut bad = packet;
        bad["target"] = json!("singer");
        assert!(serde_json::from_value::<Packet>(bad).is_err());
        Ok(())
    }
    #[test]
    fn typed_current_uses_supplied_fact_order_not_event_sort() -> Result<()> {
        let facts = vec![
            Fact {
                role: "job".into(),
                literal: "singer".into(),
                event: 50,
                record: 2,
                commit: 2,
            },
            Fact {
                role: "where".into(),
                literal: "Louston".into(),
                event: 60,
                record: 3,
                commit: 2,
            },
            Fact {
                role: "job".into(),
                literal: "dancer".into(),
                event: 7,
                record: 2,
                commit: 3,
            },
        ];
        assert_eq!(typed_answer(&facts, "job")?, "dancer.");
        assert_eq!(typed_answer(&facts, "where")?, "Louston.");
        Ok(())
    }
    #[test]
    fn pair_requires_exact_sources_and_two_questions() -> Result<()> {
        let a = Packet {
            id: "a".into(),
            segments: vec![Segment::Context {
                event: 7,
                role: 1,
                token_ids: vec![4],
            }],
            query_ids: vec![5],
            actual_prefix_ids: vec![],
        };
        let mut b = a.clone();
        b.id = "b".into();
        b.query_ids = vec![6];
        let data = vec![
            json!({"pair_id":"p","query_role":"job","source_views":[],"source_tuple":["singer"]}),
            json!({"pair_id":"p","query_role":"where","source_views":[],"source_tuple":["singer"]}),
        ];
        pairs_valid(&[a.clone(), b.clone()], &data, 1)?;
        b.segments.reverse();
        b.segments.push(Segment::Context {
            event: 7,
            role: 1,
            token_ids: vec![4],
        });
        assert!(pairs_valid(&[a, b], &data, 1).is_err());
        Ok(())
    }
}
