//! The prompt bank (claude-400): user prompts drawn from licensed sources, guarded
//! against the project's panels, deduplicated and seed-shuffled. Only user turns are
//! read; every assistant text in a source is ignored.
//!
//!   donor-corpus bank --sources DIR --panels DIR --seed S --out FILE
//!       [--two-turn-permille 100] [--cap-<source> N] [--min-chars 16] [--max-chars 4000]

use crate::prompts::{normalize, PanelGuard, Rng};
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::{Field, Row};
use parquet::schema::types::Type;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Sources in processing order (the order duplicates are judged in), with each one's
/// default cap and the fixed salt of its cap shuffle.
pub const SOURCES: [(&str, usize, u64); 5] = [
    ("wildchat", 650_000, 0x5749_4C44_4348_4154),
    ("numina", 120_000, 0x4E55_4D49_4E41_0001),
    ("openthoughts", 114_000, 0x4F50_454E_5448_4F55),
    ("oasst2", 30_000, 0x4F41_5353_5432_0001),
    ("gsm8k", 7_473, 0x4753_4D38_4B00_0001),
];
const TWO_TURN_SALT: u64 = 0x5457_4F54_5552_4E00;
const INTERLEAVE_SALT: u64 = 0x494E_5445_524C_4541;
/// Sources whose kept prompts may get a second user message.
const TWO_TURN_SOURCES: [&str; 2] = ["wildchat", "oasst2"];
const REASONS: [&str; 5] = ["filter", "length", "markup", "dup", "panel"];
/// Channel and reasoning markup that marks a prompt as a model transcript, not a user turn.
const MARKUP: [&str; 6] = ["<\u{FF5C}", "\u{FF5C}>", "<think>", "</think>", "<|", "|>"];
/// Names of the wrapper groups a nested list carries between a column and its fields.
const WRAPPERS: [&str; 5] = ["list", "element", "item", "array", "bag"];

/// Second user messages: short, generic, and sensible after almost any answer.
pub const FOLLOW_UPS: [&str; 40] = [
    "Can you say that more simply?",
    "Give me one concrete example.",
    "What is the most important point?",
    "Can you make it shorter?",
    "Why does that work?",
    "What would you do differently?",
    "Can you explain that another way?",
    "What is the first step I should take?",
    "What are the main trade-offs here?",
    "Is there a common mistake to avoid?",
    "Can you summarize that in one sentence?",
    "What would a beginner find hardest about this?",
    "How sure are you about that?",
    "What is a good next step after this?",
    "Can you put that as a short list?",
    "Which part matters most in practice?",
    "What assumptions are you making?",
    "Could you give me an analogy?",
    "What would change if I had less time?",
    "Can you go into a bit more detail?",
    "What is the simplest version of this?",
    "Are there any exceptions I should know about?",
    "How would you check that it is right?",
    "What is one thing people often get wrong here?",
    "Can you rephrase that for a child?",
    "What should I remember from this?",
    "Is there another way to do it?",
    "What are the pros and cons?",
    "Can you make that more formal?",
    "Can you make that more casual?",
    "What would an expert add to that?",
    "Which part of your answer is least certain?",
    "Can you give me a quick checklist?",
    "What question should I ask next?",
    "How would you explain this to a friend?",
    "What is the key idea in plain words?",
    "Can you turn that into three short steps?",
    "What could go wrong with this?",
    "Why is that better than the alternative?",
    "Thanks. Can you recap the main points?",
];

type Res<T> = Result<T, String>;

fn sha_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn need<'a>(m: &'a BTreeMap<String, String>, k: &str) -> Res<&'a str> {
    m.get(k)
        .map(String::as_str)
        .ok_or_else(|| format!("bank: --{k} is required"))
}

fn num<T: std::str::FromStr>(m: &BTreeMap<String, String>, k: &str, default: T) -> Res<T> {
    match m.get(k) {
        None => Ok(default),
        Some(v) => v
            .parse()
            .map_err(|_| format!("bank: --{k} {v} is not a number")),
    }
}

/// The leaf columns a source's adapter reads, as logical paths (list wrappers left out).
fn wanted(source: &str) -> &'static [&'static [&'static str]] {
    match source {
        "wildchat" => &[
            &["language"],
            &["toxic"],
            &["redacted"],
            &["conversation", "role"],
            &["conversation", "content"],
        ],
        "numina" => &[&["problem"]],
        "openthoughts" => &[&["conversations", "from"], &["conversations", "value"]],
        "oasst2" => &[
            &["role"],
            &["parent_id"],
            &["lang"],
            &["deleted"],
            &["text"],
        ],
        "gsm8k" => &[&["question"]],
        _ => &[],
    }
}

/// The file schema cut down to the wanted leaves, so no other column is decoded.
fn project(
    t: &Type,
    path: &mut Vec<String>,
    want: &[&[&str]],
    found: &mut BTreeSet<String>,
) -> Option<Type> {
    if t.is_primitive() {
        let p: Vec<&str> = path.iter().map(String::as_str).collect();
        if want.contains(&p.as_slice()) {
            found.insert(p.join("."));
            return Some(t.clone());
        }
        return None;
    }
    let mut kept = Vec::new();
    for c in t.get_fields() {
        let structural = !path.is_empty() && WRAPPERS.contains(&c.name());
        if !structural {
            path.push(c.name().to_string());
        }
        if let Some(k) = project(c, path, want, found) {
            kept.push(Arc::new(k));
        }
        if !structural {
            path.pop();
        }
    }
    if kept.is_empty() {
        return None;
    }
    let info = t.get_basic_info();
    let mut b = Type::group_type_builder(info.name())
        .with_fields(kept)
        .with_converted_type(info.converted_type())
        .with_logical_type(info.logical_type_ref().cloned());
    if info.has_repetition() {
        b = b.with_repetition(info.repetition());
    }
    b.build().ok()
}

fn field<'a>(row: &'a Row, name: &str) -> Option<&'a Field> {
    row.get_column_iter()
        .find(|(k, _)| k.as_str() == name)
        .map(|(_, v)| v)
}

fn str_of(f: &Field) -> Option<&str> {
    match f {
        Field::Str(s) => Some(s.as_str()),
        Field::Bytes(b) => b.as_utf8().ok(),
        _ => None,
    }
}

fn bool_of(f: &Field) -> Option<bool> {
    match f {
        Field::Bool(b) => Some(*b),
        _ => None,
    }
}

/// The `text_key` of the first list element whose `role_key` equals `role`.
fn first_turn(list: &Field, role_key: &str, role: &str, text_key: &str) -> Option<String> {
    let Field::ListInternal(l) = list else {
        return None;
    };
    let first = l.elements().iter().find(|e| match e {
        Field::Group(g) => field(g, role_key).and_then(str_of) == Some(role),
        _ => false,
    })?;
    let Field::Group(g) = first else {
        return None;
    };
    field(g, text_key).and_then(str_of).map(str::to_string)
}

/// The user prompt a source row holds, or None when the row fails the source's filter.
fn extract(source: &str, row: &Row) -> Option<String> {
    let s = |k: &str| field(row, k).and_then(str_of);
    let b = |k: &str| field(row, k).and_then(bool_of);
    match source {
        "wildchat" => {
            if s("language")? != "English" || b("toxic")? || b("redacted")? {
                return None;
            }
            first_turn(field(row, "conversation")?, "role", "user", "content")
        }
        "numina" => s("problem").map(str::to_string),
        "openthoughts" => first_turn(field(row, "conversations")?, "from", "user", "value"),
        "oasst2" => {
            let root = matches!(field(row, "parent_id"), None | Some(Field::Null));
            if s("role")? != "prompter" || !root || s("lang")? != "en" || b("deleted")? {
                return None;
            }
            s("text").map(str::to_string)
        }
        "gsm8k" => s("question").map(str::to_string),
        _ => None,
    }
}

/// Reads one parquet file of a source, calling `f` with each row's trimmed prompt (None
/// when the row is filtered). Returns the rows seen.
pub fn scan_file(source: &str, path: &Path, f: &mut dyn FnMut(Option<String>)) -> Res<u64> {
    let file = fs::File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    let reader =
        SerializedFileReader::new(file).map_err(|e| format!("parquet {}: {e}", path.display()))?;
    let want = wanted(source);
    let mut found = BTreeSet::new();
    let proj = project(
        reader.metadata().file_metadata().schema(),
        &mut Vec::new(),
        want,
        &mut found,
    );
    if found.len() != want.len() {
        let missing: Vec<String> = want
            .iter()
            .map(|w| w.join("."))
            .filter(|w| !found.contains(w))
            .collect();
        return Err(format!(
            "{}: columns {missing:?} not found for {source}",
            path.display()
        ));
    }
    let iter = reader
        .get_row_iter(proj)
        .map_err(|e| format!("rows {}: {e}", path.display()))?;
    let mut rows = 0u64;
    for r in iter {
        let row = r.map_err(|e| format!("row {} in {}: {e}", rows, path.display()))?;
        rows += 1;
        let p = extract(source, &row)
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty());
        f(p);
    }
    Ok(rows)
}

/// The text and each of its sentences, the parts the panel guard is asked about.
fn sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in text.chars() {
        cur.push(c);
        if matches!(c, '.' | '?' | '!' | '\n') {
            if !cur.trim().is_empty() {
                out.push(cur.trim().to_string());
            }
            cur.clear();
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

fn panel_hit(guard: &PanelGuard, text: &str) -> bool {
    guard.hits(&[text.to_string()]) || guard.hits(&sentences(text))
}

fn fisher_yates<T>(xs: &mut [T], rng: &mut Rng) {
    for i in (1..xs.len()).rev() {
        let j = rng.below(i + 1);
        xs.swap(i, j);
    }
}

fn json_str(s: &str) -> Res<String> {
    serde_json::to_string(s).map_err(|e| e.to_string())
}

/// Loads every panel file under `dir` into a guard, with each file's name, size and hash.
fn load_panels(dir: &Path) -> Res<(PanelGuard, Vec<Value>, Vec<String>)> {
    let mut guard = PanelGuard::new();
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("read_dir {}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            return Err(format!(
                "{} is a folder: the guard reads files only",
                p.display()
            ));
        }
        let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("");
        let name = p.file_name().and_then(|x| x.to_str()).unwrap_or("");
        let bytes = fs::read(&p).map_err(|e| format!("read {}: {e}", p.display()))?;
        if guard.add_file(ext, &String::from_utf8_lossy(&bytes)) {
            files.push(json!({"name": name, "bytes": bytes.len(), "sha256": sha_hex(&bytes)}));
        } else {
            skipped.push(name.to_string());
        }
    }
    if guard.is_empty() {
        return Err(format!("no panel strings found under {}", dir.display()));
    }
    Ok((guard, files, skipped))
}

struct Kept {
    source: &'static str,
    text: String,
    follow_up: Option<&'static str>,
}

pub fn cmd_bank(m: &BTreeMap<String, String>) -> Result<(), String> {
    let sources = PathBuf::from(need(m, "sources")?);
    let panels = PathBuf::from(need(m, "panels")?);
    let seed: u64 = need(m, "seed")?
        .parse()
        .map_err(|_| "bank: --seed is not a number".to_string())?;
    let out = PathBuf::from(need(m, "out")?);
    let permille: usize = num(m, "two-turn-permille", 100)?;
    let min_chars: usize = num(m, "min-chars", 16)?;
    let max_chars: usize = num(m, "max-chars", 4000)?;
    if permille > 1000 {
        return Err("bank: --two-turn-permille is above 1000".into());
    }
    let (guard, panel_files, panels_not_read) = load_panels(&panels)?;
    if let Some(f) = FOLLOW_UPS.iter().find(|f| panel_hit(&guard, f)) {
        return Err(format!("bank: follow-up {f:?} is a panel string"));
    }
    let mut flags = BTreeMap::new();
    flags.insert("two-turn-permille".to_string(), permille);
    flags.insert("min-chars".to_string(), min_chars);
    flags.insert("max-chars".to_string(), max_chars);

    let mut seen: HashSet<String> = HashSet::new();
    let mut kept: Vec<Kept> = Vec::new();
    let mut stats: BTreeMap<String, Value> = BTreeMap::new();
    let mut two_rng = Rng::new(seed ^ TWO_TURN_SALT);
    for (name, default_cap, salt) in SOURCES {
        let cap: usize = num(m, &format!("cap-{name}"), default_cap)?;
        flags.insert(format!("cap-{name}"), cap);
        let dir = sources.join(name);
        let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
            .map_err(|e| format!("read_dir {}: {e}", dir.display()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("parquet"))
            .collect();
        paths.sort();
        if paths.is_empty() {
            return Err(format!("no parquet files under {}", dir.display()));
        }
        let mut dropped: BTreeMap<&str, u64> = REASONS.iter().map(|r| (*r, 0)).collect();
        let mut survivors: Vec<String> = Vec::new();
        let mut files = Vec::new();
        let mut rows = 0u64;
        let mut extracted = 0u64;
        for p in &paths {
            let bytes = fs::read(p).map_err(|e| format!("read {}: {e}", p.display()))?;
            files.push(json!({
                "name": p.file_name().and_then(|x| x.to_str()).unwrap_or(""),
                "bytes": bytes.len(),
                "sha256": sha_hex(&bytes),
            }));
            drop(bytes);
            rows += scan_file(name, p, &mut |prompt| {
                let reason = match prompt {
                    None => "filter",
                    Some(t) => {
                        extracted += 1;
                        let n = t.chars().count();
                        if n < min_chars || n > max_chars {
                            "length"
                        } else if MARKUP.iter().any(|k| t.contains(k)) {
                            "markup"
                        } else if !seen.insert(normalize(&t)) {
                            "dup"
                        } else if panel_hit(&guard, &t) {
                            "panel"
                        } else {
                            survivors.push(t);
                            return;
                        }
                    }
                };
                *dropped.entry(reason).or_insert(0) += 1;
            })?;
        }
        let n_survivors = survivors.len();
        if n_survivors > cap {
            fisher_yates(&mut survivors, &mut Rng::new(seed ^ salt));
            survivors.truncate(cap);
        }
        let mut two_turn = 0u64;
        for text in survivors {
            let mut follow_up = None;
            if TWO_TURN_SOURCES.contains(&name) {
                let roll = two_rng.below(1000);
                let pick = two_rng.below(FOLLOW_UPS.len());
                if roll < permille {
                    follow_up = Some(FOLLOW_UPS[pick]);
                    two_turn += 1;
                }
            }
            kept.push(Kept {
                source: name,
                text,
                follow_up,
            });
        }
        let kept_here = kept.iter().filter(|k| k.source == name).count();
        stats.insert(
            name.to_string(),
            json!({
                "files": files, "rows_seen": rows, "extracted": extracted, "dropped": dropped,
                "survivors": n_survivors, "cap": cap, "kept": kept_here, "two_turn": two_turn,
            }),
        );
    }
    fisher_yates(&mut kept, &mut Rng::new(seed ^ INTERLEAVE_SALT));

    let mut ids: BTreeSet<String> = BTreeSet::new();
    let mut text = String::new();
    for k in &kept {
        let mut id = format!(
            "{}-{}",
            k.source,
            &sha_hex(normalize(&k.text).as_bytes())[..16]
        );
        let mut msgs = format!("{{\"role\":\"user\",\"content\":{}}}", json_str(&k.text)?);
        let kind = match k.follow_up {
            None => "single",
            Some(f) => {
                id.push_str("-t2");
                msgs.push_str(&format!(
                    ",{{\"role\":\"user\",\"content\":{}}}",
                    json_str(f)?
                ));
                "two-turn"
            }
        };
        if !ids.insert(id.clone()) {
            return Err(format!("bank: id {id} is not unique"));
        }
        text.push_str(&format!(
            "{{\"id\":{},\"kind\":{},\"source\":{},\"messages\":[{msgs}]}}\n",
            json_str(&id)?,
            json_str(kind)?,
            json_str(k.source)?
        ));
    }
    fs::write(&out, &text).map_err(|e| format!("write {}: {e}", out.display()))?;
    let manifest = json!({
        "seed": seed, "flags": flags, "output_sha256": sha_hex(text.as_bytes()),
        "panel_files": panel_files, "panel_files_not_read": panels_not_read,
        "panel_strings": guard.len(), "sources": stats, "total_prompts": kept.len(),
    });
    let mpath = PathBuf::from(format!("{}.manifest.json", out.display()));
    let mtext = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())? + "\n";
    fs::write(&mpath, mtext).map_err(|e| format!("write {}: {e}", mpath.display()))?;
    println!(
        "bank: {} prompts -> {} (sha256 {})",
        kept.len(),
        out.display(),
        sha_hex(text.as_bytes())
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use parquet::basic::Compression;
    use parquet::data_type::{BoolType, ByteArray, ByteArrayType};
    use parquet::file::properties::WriterProperties;
    use parquet::file::writer::SerializedFileWriter;
    use parquet::schema::parser::parse_message_type;

    enum Col {
        S(Vec<String>, Option<Vec<i16>>, Option<Vec<i16>>),
        B(Vec<bool>, Option<Vec<i16>>),
    }

    fn write_pq(path: &Path, schema: &str, cols: Vec<Col>) {
        let schema = Arc::new(parse_message_type(schema).unwrap());
        let props = Arc::new(
            WriterProperties::builder()
                .set_compression(Compression::SNAPPY)
                .build(),
        );
        let f = fs::File::create(path).unwrap();
        let mut w = SerializedFileWriter::new(f, schema, props).unwrap();
        let mut rg = w.next_row_group().unwrap();
        let mut it = cols.into_iter();
        while let Some(mut cw) = rg.next_column().unwrap() {
            match it.next().unwrap() {
                Col::S(v, d, r) => {
                    let v: Vec<ByteArray> = v.iter().map(|s| ByteArray::from(s.as_str())).collect();
                    cw.typed::<ByteArrayType>()
                        .write_batch(&v, d.as_deref(), r.as_deref())
                        .unwrap();
                }
                Col::B(v, d) => {
                    cw.typed::<BoolType>()
                        .write_batch(&v, d.as_deref(), None)
                        .unwrap();
                }
            }
            cw.close().unwrap();
        }
        rg.close().unwrap();
        w.close().unwrap();
    }

    type Turns = Vec<(&'static str, String)>;

    /// A list-of-struct column pair (role, text) with its definition and repetition levels.
    fn turn_cols(rows: &[Turns]) -> (Col, Col) {
        let (mut roles, mut texts, mut def, mut rep) = (vec![], vec![], vec![], vec![]);
        for t in rows {
            if t.is_empty() {
                def.push(0);
                rep.push(0);
            }
            for (k, (r, x)) in t.iter().enumerate() {
                roles.push(r.to_string());
                texts.push(x.clone());
                def.push(1);
                rep.push(if k == 0 { 0 } else { 1 });
            }
        }
        (
            Col::S(roles, Some(def.clone()), Some(rep.clone())),
            Col::S(texts, Some(def), Some(rep)),
        )
    }

    fn list_schema(list: &str, role: &str, text: &str) -> String {
        format!(
            "required group {list} (LIST) {{ repeated group list {{ required group element {{ \
             required binary {role} (UTF8); required binary {text} (UTF8); }} }} }}"
        )
    }

    struct Fx {
        wildchat: Vec<(&'static str, bool, bool, Turns)>,
        numina: Vec<String>,
        openthoughts: Vec<Turns>,
        oasst2: Vec<(
            &'static str,
            Option<&'static str>,
            &'static str,
            bool,
            String,
        )>,
        gsm8k: Vec<String>,
    }

    fn base() -> Fx {
        Fx {
            wildchat: vec![],
            numina: vec![],
            openthoughts: vec![],
            oasst2: vec![],
            gsm8k: vec![],
        }
    }

    fn dir(name: &str) -> PathBuf {
        let exe = std::env::current_exe().unwrap();
        let d = exe.parent().unwrap().join("bank-fixtures").join(name);
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    /// Writes all five sources and a panels folder; returns (sources, panels).
    fn write_fx(root: &Path, fx: &Fx, panel_json: &str) -> (PathBuf, PathBuf) {
        let src = root.join("src");
        for (n, _, _) in SOURCES {
            fs::create_dir_all(src.join(n)).unwrap();
        }
        let w = &fx.wildchat;
        let (role, content) = turn_cols(&w.iter().map(|r| r.3.clone()).collect::<Vec<_>>());
        write_pq(
            &src.join("wildchat/train-00000.parquet"),
            &format!(
                "message schema {{ required binary language (UTF8); required boolean toxic; \
                 required boolean redacted; {} }}",
                list_schema("conversation", "role", "content")
            ),
            vec![
                Col::S(w.iter().map(|r| r.0.to_string()).collect(), None, None),
                Col::B(w.iter().map(|r| r.1).collect(), None),
                Col::B(w.iter().map(|r| r.2).collect(), None),
                role,
                content,
            ],
        );
        let flat = |p: &str, col: &str, v: &[String]| {
            write_pq(
                &src.join(p),
                &format!(
                    "message schema {{ required binary {col} (UTF8); required binary other (UTF8); }}"
                ),
                vec![
                    Col::S(v.to_vec(), None, None),
                    Col::S(v.iter().map(|_| "assistant answer".to_string()).collect(), None, None),
                ],
            )
        };
        flat("numina/train-00000.parquet", "problem", &fx.numina);
        flat("gsm8k/train-00000.parquet", "question", &fx.gsm8k);
        let (from, value) = turn_cols(&fx.openthoughts);
        write_pq(
            &src.join("openthoughts/train-00000.parquet"),
            &format!(
                "message schema {{ {} }}",
                list_schema("conversations", "from", "value")
            ),
            vec![from, value],
        );
        let o = &fx.oasst2;
        write_pq(
            &src.join("oasst2/train-00000.parquet"),
            "message schema { required binary role (UTF8); optional binary parent_id (UTF8); \
             required binary lang (UTF8); required boolean deleted; required binary text (UTF8); }",
            vec![
                Col::S(o.iter().map(|r| r.0.to_string()).collect(), None, None),
                Col::S(
                    o.iter().filter_map(|r| r.1.map(str::to_string)).collect(),
                    Some(o.iter().map(|r| r.1.is_some() as i16).collect()),
                    None,
                ),
                Col::S(o.iter().map(|r| r.2.to_string()).collect(), None, None),
                Col::B(o.iter().map(|r| r.3).collect(), None),
                Col::S(o.iter().map(|r| r.4.clone()).collect(), None, None),
            ],
        );
        let panels = root.join("panels");
        fs::create_dir_all(&panels).unwrap();
        fs::write(panels.join("panel.json"), panel_json).unwrap();
        (src, panels)
    }

    const PANEL: &str = r#"{"rows": [{"q": "What is the capital of France?"}]}"#;

    fn run(src: &Path, panels: &Path, out: &Path, extra: &[(&str, &str)]) -> Res<()> {
        let mut m: BTreeMap<String, String> = [
            ("sources", src.to_str().unwrap()),
            ("panels", panels.to_str().unwrap()),
            ("seed", "7"),
            ("out", out.to_str().unwrap()),
        ]
        .iter()
        .chain(extra.iter())
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        m.entry("seed".into()).or_insert_with(|| "7".into());
        cmd_bank(&m)
    }

    fn lines(out: &Path) -> Vec<Value> {
        fs::read_to_string(out)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    fn manifest(out: &Path) -> Value {
        let p = format!("{}.manifest.json", out.display());
        serde_json::from_str(&fs::read_to_string(p).unwrap()).unwrap()
    }

    fn s(x: &str) -> String {
        x.to_string()
    }

    fn contents(v: &[Value]) -> BTreeSet<String> {
        v.iter()
            .map(|l| l["messages"][0]["content"].as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn bank_row_filters_per_source() {
        let root = dir("filters");
        let mut fx = base();
        let a = |x: &str| ("assistant", s(x));
        let u = |x: &str| ("user", s(x));
        fx.wildchat = vec![
            (
                "English",
                false,
                false,
                vec![
                    a("Assistant text to ignore here"),
                    u("Wildchat user prompt number one"),
                    u("A later user turn that is not first"),
                ],
            ),
            (
                "French",
                false,
                false,
                vec![u("Wildchat prompt in another language")],
            ),
            (
                "English",
                true,
                false,
                vec![u("Wildchat prompt that is toxic here")],
            ),
            (
                "English",
                false,
                true,
                vec![u("Wildchat prompt that is redacted ok")],
            ),
            (
                "English",
                false,
                false,
                vec![a("Only an assistant turn in this row")],
            ),
            ("English", false, false, vec![]),
        ];
        fx.numina = vec![s("Numina problem about a triangle area"), s("   ")];
        fx.openthoughts = vec![
            vec![
                ("assistant", s("Assistant text before the user")),
                ("user", s("Openthoughts user question about primes")),
            ],
            vec![("assistant", s("Assistant only conversation row"))],
        ];
        fx.oasst2 = vec![
            (
                "prompter",
                None,
                "en",
                false,
                s("Oasst root prompt in English here"),
            ),
            (
                "assistant",
                None,
                "en",
                false,
                s("Oasst assistant reply is ignored"),
            ),
            (
                "prompter",
                Some("abc"),
                "en",
                false,
                s("Oasst prompter reply in a thread"),
            ),
            (
                "prompter",
                None,
                "de",
                false,
                s("Oasst root prompt in German here"),
            ),
            (
                "prompter",
                None,
                "en",
                true,
                s("Oasst root prompt that was deleted"),
            ),
        ];
        fx.gsm8k = vec![s("Gsm8k question about apples and pears")];
        let (src, panels) = write_fx(&root, &fx, PANEL);
        let out = root.join("bank.jsonl");
        run(&src, &panels, &out, &[]).unwrap();
        let want: BTreeSet<String> = [
            "Wildchat user prompt number one",
            "Numina problem about a triangle area",
            "Openthoughts user question about primes",
            "Oasst root prompt in English here",
            "Gsm8k question about apples and pears",
        ]
        .iter()
        .map(|x| s(x))
        .collect();
        let got = lines(&out);
        assert_eq!(contents(&got), want);
        let man = manifest(&out);
        for (name, rows, filtered) in [
            ("wildchat", 6, 5),
            ("numina", 2, 1),
            ("openthoughts", 2, 1),
            ("oasst2", 5, 4),
            ("gsm8k", 1, 0),
        ] {
            let st = &man["sources"][name];
            assert_eq!(st["rows_seen"], rows, "{name}");
            assert_eq!(st["dropped"]["filter"], filtered, "{name}");
            assert_eq!(st["kept"], 1, "{name}");
            assert_eq!(st["files"].as_array().unwrap().len(), 1);
        }
        assert_eq!(man["total_prompts"], 5);
        for l in &got {
            let src = l["source"].as_str().unwrap();
            assert!(l["id"].as_str().unwrap().starts_with(&format!("{src}-")));
        }
    }

    #[test]
    fn bank_length_markup_and_dup_drops_across_sources() {
        let root = dir("drops");
        let mut fx = base();
        let u = |x: &str| vec![("user", s(x))];
        fx.wildchat = vec![
            ("English", false, false, u("too short")),
            (
                "English",
                false,
                false,
                u("Please think hard <think>step by step</think> ok"),
            ),
            (
                "English",
                false,
                false,
                u("Some prompt <|im_start|>user with a tag"),
            ),
            (
                "English",
                false,
                false,
                u("Some prompt <\u{FF5C}User\u{FF5C}> with a wide tag"),
            ),
            ("English", false, false, u(&"long ".repeat(30))),
            (
                "English",
                false,
                false,
                u("Tell me about the moon and the stars"),
            ),
        ];
        fx.numina = vec![s("TELL me about the moon, and the stars!")];
        fx.openthoughts = vec![u("An openthoughts prompt that is unique")];
        fx.gsm8k = vec![s("tell me about the MOON and the stars??")];
        let (src, panels) = write_fx(&root, &fx, PANEL);
        let out = root.join("bank.jsonl");
        run(&src, &panels, &out, &[("max-chars", "60")]).unwrap();
        let man = manifest(&out);
        let w = &man["sources"]["wildchat"];
        assert_eq!(w["dropped"]["length"], 2);
        assert_eq!(w["dropped"]["markup"], 3);
        assert_eq!(w["kept"], 1);
        assert_eq!(man["sources"]["numina"]["dropped"]["dup"], 1);
        assert_eq!(man["sources"]["gsm8k"]["dropped"]["dup"], 1);
        assert_eq!(man["sources"]["numina"]["kept"], 0);
        assert_eq!(man["total_prompts"], 2);
    }

    #[test]
    fn bank_panel_strings_are_dropped_in_any_case_and_punctuation() {
        let root = dir("panel");
        let mut fx = base();
        fx.gsm8k = vec![
            s("what IS the capital,, of france"),
            s("Hello there my friend. WHAT is the capital of France!! Thanks a lot."),
            s("What is the capital of Spain today?"),
        ];
        let (src, panels) = write_fx(&root, &fx, PANEL);
        let out = root.join("bank.jsonl");
        run(&src, &panels, &out, &[]).unwrap();
        let man = manifest(&out);
        assert_eq!(man["sources"]["gsm8k"]["dropped"]["panel"], 2);
        assert_eq!(
            contents(&lines(&out)),
            [s("What is the capital of Spain today?")]
                .into_iter()
                .collect()
        );
        assert_eq!(man["panel_files"][0]["name"], "panel.json");
        assert_eq!(man["panel_files"][0]["bytes"], PANEL.len());
    }

    #[test]
    fn bank_caps_and_two_turn_permille() {
        let root = dir("caps");
        let mut fx = base();
        fx.wildchat = (0..400)
            .map(|i| {
                (
                    "English",
                    false,
                    false,
                    vec![("user", format!("Wildchat prompt number {i} about things"))],
                )
            })
            .collect();
        fx.gsm8k = (0..10)
            .map(|i| format!("Gsm8k question number {i} about sums"))
            .collect();
        let (src, panels) = write_fx(&root, &fx, PANEL);
        let out = root.join("bank.jsonl");
        let args = [
            ("cap-wildchat", "300"),
            ("cap-gsm8k", "3"),
            ("two-turn-permille", "250"),
        ];
        run(&src, &panels, &out, &args).unwrap();
        let man = manifest(&out);
        let w = &man["sources"]["wildchat"];
        assert_eq!(w["survivors"], 400);
        assert_eq!(w["kept"], 300);
        assert_eq!(man["sources"]["gsm8k"]["kept"], 3);
        assert_eq!(man["sources"]["gsm8k"]["two_turn"], 0);
        let two = w["two_turn"].as_u64().unwrap();
        assert!(
            (40..=110).contains(&two),
            "two-turn {two} of 300 at 250 permille"
        );
        let got = lines(&out);
        assert_eq!(got.len(), 303);
        let mut n_two = 0;
        for l in &got {
            let msgs = l["messages"].as_array().unwrap();
            let id = l["id"].as_str().unwrap();
            if l["kind"] == "two-turn" {
                n_two += 1;
                assert_eq!(msgs.len(), 2);
                assert!(id.ends_with("-t2"));
                assert!(FOLLOW_UPS.contains(&msgs[1]["content"].as_str().unwrap()));
            } else {
                assert_eq!(l["kind"], "single");
                assert_eq!(msgs.len(), 1);
                assert!(!id.ends_with("-t2"));
            }
            assert!(msgs.iter().all(|m| m["role"] == "user"));
        }
        assert_eq!(n_two, two);
        // The interleave mixes sources: the gsm8k prompts are not all at one end.
        let first_gsm = got.iter().position(|l| l["source"] == "gsm8k").unwrap();
        assert!(first_gsm < 300);
    }

    #[test]
    fn bank_output_is_byte_identical_and_ids_are_unique() {
        let root = dir("same");
        let mut fx = base();
        fx.wildchat = (0..50)
            .map(|i| {
                (
                    "English",
                    false,
                    false,
                    vec![("user", format!("Wildchat prompt number {i} again"))],
                )
            })
            .collect();
        fx.numina = (0..20)
            .map(|i| format!("Numina problem number {i} here"))
            .collect();
        fx.oasst2 = (0..20)
            .map(|i| {
                (
                    "prompter",
                    None,
                    "en",
                    false,
                    format!("Oasst prompt number {i} here"),
                )
            })
            .collect();
        let (src, panels) = write_fx(&root, &fx, PANEL);
        let (a, b, c) = (
            root.join("a.jsonl"),
            root.join("b.jsonl"),
            root.join("c.jsonl"),
        );
        let args = [("two-turn-permille", "500")];
        run(&src, &panels, &a, &args).unwrap();
        run(&src, &panels, &b, &args).unwrap();
        run(
            &src,
            &panels,
            &c,
            &[("two-turn-permille", "500"), ("seed", "8")],
        )
        .unwrap();
        assert_eq!(fs::read(&a).unwrap(), fs::read(&b).unwrap());
        let ma = fs::read(format!("{}.manifest.json", a.display())).unwrap();
        let mb = fs::read(format!("{}.manifest.json", b.display())).unwrap();
        assert_eq!(ma, mb);
        assert_ne!(fs::read(&a).unwrap(), fs::read(&c).unwrap());
        let got = lines(&a);
        let ids: BTreeSet<&str> = got.iter().map(|l| l["id"].as_str().unwrap()).collect();
        assert_eq!(ids.len(), got.len());
        assert_eq!(got.len(), 90);
        let man = manifest(&a);
        assert_eq!(man["output_sha256"], sha_hex(&fs::read(&a).unwrap()));
        assert_eq!(man["seed"], 7);
    }

    #[test]
    fn bank_follow_ups_pass_the_guard() {
        let distinct: BTreeSet<String> = FOLLOW_UPS.iter().map(|f| normalize(f)).collect();
        assert_eq!(distinct.len(), FOLLOW_UPS.len());
        let mut guard = PanelGuard::new();
        guard.add_file("json", PANEL);
        for f in FOLLOW_UPS {
            assert!(!panel_hit(&guard, f), "{f}");
            assert!(f.chars().count() >= 16, "{f}");
            assert!(!MARKUP.iter().any(|k| f.contains(k)));
        }
        // A follow-up that is a panel string stops the build.
        let root = dir("follow");
        let mut fx = base();
        fx.gsm8k = vec![s("Gsm8k question about apples and pears")];
        let (src, panels) = write_fx(&root, &fx, r#"["give me ONE concrete example"]"#);
        let err = run(&src, &panels, &root.join("bank.jsonl"), &[]).unwrap_err();
        assert!(err.contains("follow-up"), "{err}");
    }

    /// BANK_REAL_SOURCE=<name> BANK_REAL_FILE=<parquet> reads one real file.
    #[test]
    #[ignore]
    fn bank_real_file_reads() {
        let source = std::env::var("BANK_REAL_SOURCE").unwrap();
        let path = PathBuf::from(std::env::var("BANK_REAL_FILE").unwrap());
        let start = std::time::Instant::now();
        let mut extracted = 0u64;
        let mut sample = None;
        let rows = scan_file(&source, &path, &mut |p| {
            if let Some(t) = p {
                extracted += 1;
                sample.get_or_insert(t);
            }
        })
        .unwrap();
        println!(
            "{source}: rows {rows}, extracted {extracted}, {} ms, first {:?}",
            start.elapsed().as_millis(),
            sample.map(|t| t.chars().take(80).collect::<String>())
        );
        assert!(rows > 0 && extracted > 0);
    }
}
