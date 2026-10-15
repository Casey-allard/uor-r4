//! Writing receipts — the first half of the port that produces state rather than reading it.
//!
//! **Why this exists before more reading.** Every check so far compares the two implementations on
//! state that *Python wrote*. That proves Rust can read the shared record; it says nothing about
//! whether Rust can *add to* it. A port is not a port until a record written by one implementation
//! is read correctly by the other, and a writer that is subtly wrong is worse than no writer —
//! it corrupts the record the labs coordinate through.
//!
//! The contract it must satisfy, taken from the Python implementation:
//!
//! * **Sharded by lab** — `docs/harness/ledger/<slug>-<YYYY-MM>.jsonl`. Two labs never write the
//!   same file. This is the sharding the whole plane depends on, so the port must get it identically
//!   right, including the slug rules.
//! * **One JSON object per line**, keys sorted, `ensure_ascii` off. Python writes
//!   `json.dumps(record, ensure_ascii=False, sort_keys=True)`; a reader parses by line and does not
//!   care, but a *diff* does, and two implementations writing one file differently would make every
//!   ledger change unreadable.
//! * **Append, never rewrite.** A receipt write must not be able to lose an existing line.
//! * **`kind`, `lab`, `schema`, `ts` always present** — the contract's `required_always` for this
//!   format. A row missing any of them fails `statecheck`.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::state::{slug, slug_stem, StatePaths};

/// Bumped when the receipt shape changes. Must match what the Python implementation writes.
pub const RECEIPT_SCHEMA: u32 = 1;

/// One receipt, in the shape the contract requires.
#[derive(Debug, Clone)]
pub struct Receipt {
    /// Always present. `skill`, `heartbeat`, `claim`, `release`, `team`, `prep`, `receipt`.
    pub kind: String,
    /// Always present. Slugs to the shard it lands in.
    pub lab: String,
    /// Always present. RFC3339 with the local offset, as the Python side writes.
    pub ts: String,
    /// Every other field the caller supplied, rendered as JSON.
    pub fields: Map<String, Value>,
}

impl Receipt {
    /// Build a receipt, stamping the schema and defaulting the timestamp to now.
    pub fn new(kind: impl Into<String>, lab: impl Into<String>) -> Self {
        Receipt {
            kind: kind.into(),
            lab: lab.into(),
            ts: now_rfc3339(),
            fields: Map::new(),
        }
    }

    pub fn with(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    /// The line this receipt becomes: `json.dumps(record, ensure_ascii=False, sort_keys=True)`.
    ///
    /// **The claim in this comment was false until the `team dispatch` unit diffed a written line
    /// against Python's.** `Value::to_string()` is COMPACT - `{"count":2}` - while Python's
    /// `json.dumps` writes the default separators, `{"count": 2}`. Both parse to the same record and
    /// every reader in this crate parses, so nothing failed; **the module header says two
    /// implementations writing one file differently "would make every ledger change unreadable", and
    /// that is what was happening.** Reading a line back is not comparing it.
    pub fn to_line(&self) -> String {
        // serde_json::Map preserves insertion order by default, so the keys are inserted sorted
        // rather than relying on a feature flag.
        let mut all: Vec<(&str, Value)> = vec![
            ("kind", Value::String(self.kind.clone())),
            ("lab", Value::String(self.lab.clone())),
            ("schema", Value::from(RECEIPT_SCHEMA)),
            ("ts", Value::String(self.ts.clone())),
        ];
        for (k, v) in &self.fields {
            all.push((k.as_str(), v.clone()));
        }
        all.sort_by(|a, b| a.0.cmp(b.0));
        let mut obj = Map::new();
        for (k, v) in all {
            obj.insert(k.to_string(), v);
        }
        py_dumps_sorted(&Value::Object(obj))
    }

    /// A malformed receipt is rejected before it can reach the record: the four required keys are
    /// the contract's `required_always`, and a row without them fails `statecheck`.
    ///
    /// **An unnamed lab is NOT malformed.** Python warns loudly, keeps `"lab": ""` in the line and
    /// records it in the `unknown-<month>` shard: a receipt that cannot say who wrote it still says
    /// what happened, and refusing to write it would lose the record where the warning does not.
    /// Thirteen receipts accumulated unattributed before that warning existed. [unattributed] tells
    /// the caller to print it.
    ///
    /// **The kind IS checked, against the frozen list, and this is the only place that can do it for
    /// every writer.** It was not checked at all: `receipt --kind <anything>` wrote the line, and
    /// `check` then hard-errored on it at parse (`receipts.parse`) - so the harness accepted a record
    /// its own reader rejects, and the only way to fix it was to find and delete the line by hand.
    /// Reported by the Stage-A sweep-record agent on #2028. A writer that emits what its reader
    /// refuses is worse than a writer that refuses.
    pub fn validate(&self) -> Result<(), String> {
        if self.kind.trim().is_empty() {
            return Err("a receipt needs a kind".into());
        }
        if !crate::receiptsrules::KINDS.contains(&self.kind.as_str()) {
            return Err(format!(
                "unknown receipt kind {:?}; the frozen list is {}",
                self.kind,
                python_tuple(&crate::receiptsrules::KINDS)
            ));
        }
        if self.ts.trim().is_empty() {
            return Err("a receipt needs a timestamp".into());
        }
        Ok(())
    }

    /// True when no lab named itself, so this line lands in the `unknown` shard.
    pub fn unattributed(&self) -> bool {
        slug_stem(&self.lab).is_empty()
    }
}

/// The warning a writer prints when it records an unattributed receipt.
///
/// Once per process, on stderr, in Python's words - the wording is the fix instruction, and two
/// implementations telling a lab different things about the same mistake is a divergence in the
/// only place it matters.
pub const UNATTRIBUTED_WARNING: &str = "uor-harness: WARNING - no lab identity, so this record is attributed to nobody and no\n             other lab can see who did it. Fix it for this session with:\n             export UOR_LAB=<kimi|claude|codex|deepseek>";

/// `json.dumps(value, ensure_ascii=False, sort_keys=True)` - Python's DEFAULT separators.
///
/// `serde_json`'s compact form is `{"a":1,"b":[1,2]}` and Python's is `{"a": 1, "b": [1, 2]}`.
/// The difference is invisible to every reader and obvious in a diff, and the ledger is a file labs
/// read as a diff.
///
/// `ensure_ascii` is off here (Python's ledger call passes `ensure_ascii=False`), which is why this
/// is not the same renderer as `registries::ensure_ascii`: the ledger keeps UTF-8, `--json` escapes
/// it. Two call sites, two Python defaults, stated rather than shared by accident.
///
/// Shared beyond the ledger: `presence::publish_live` writes the same shape, and `migrate_ledger`
/// re-serialises every row it moves with it — a migrated row must be byte-identical to the row
/// Python would have written, or the migration itself changes the record.
pub fn py_dumps_sorted(v: &Value) -> String {
    match v {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        // serde_json's string escaping already matches Python's for a lone string: the same short
        // escapes for control characters, no escaping of `/`, and raw UTF-8.
        Value::String(s) => Value::String(s.clone()).to_string(),
        Value::Array(items) => {
            let parts: Vec<String> = items.iter().map(py_dumps_sorted).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Object(map) => {
            // UTF-8 byte order IS code-point order, so Rust's sort is Python's `sort_keys=True`.
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let parts: Vec<String> = keys
                .iter()
                .map(|k| {
                    format!(
                        "{}: {}",
                        Value::String((*k).clone()),
                        py_dumps_sorted(&map[*k])
                    )
                })
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
    }
}

/// Where a lab's shard for a month lives.
pub fn shard_path(paths: &StatePaths, lab: &str, month: &str) -> PathBuf {
    // `slug` already falls back to `unknown`, which is what an unattributed row is filed under.
    paths
        .ledger_dir()
        .join(format!("{}-{month}.jsonl", slug(lab)))
}

/// Append one receipt to its lab's shard. Returns the shard written.
///
/// Appends with `O_APPEND`, so a concurrent writer from another lab cannot interleave into this
/// file — different labs write different shards by construction, and within a shard the append is
/// atomic for the line sizes involved.
pub fn append(paths: &StatePaths, receipt: &Receipt, month: &str) -> Result<PathBuf, String> {
    receipt.validate()?;
    write_line(paths, receipt, month)
}

/// The write half, without the lab check.
///
/// `record` needs it: **Python's `append_receipt` writes an unattributed receipt to the
/// `unknown-<month>` shard while KEEPING `"lab": ""` in the line**, and `Receipt::validate`
/// refuses an empty lab. Refusing here would be a divergence in the wrong direction - the port
/// would error where Python records the receipt, warns, and moves on.
fn write_line(paths: &StatePaths, receipt: &Receipt, month: &str) -> Result<PathBuf, String> {
    // R10: a committed line goes into the caller's own checkout, never the shared one.
    crate::state::refuse_shared_checkout(paths, "a ledger receipt")?;
    let shard = shard_path(paths, &receipt.lab, month);
    if let Some(parent) = shard.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&shard)
        .map_err(|e| format!("cannot open {}: {e}", shard.display()))?;
    writeln!(file, "{}", receipt.to_line())
        .map_err(|e| format!("cannot write {}: {e}", shard.display()))?;
    Ok(shard)
}

// ----------------------------------------------------------------------------------- record

/// The event a receipt kind carries, when the kind has one. `receipts.EVENT_TYPES`.
pub fn event_type(kind: &str) -> Option<&'static str> {
    match kind {
        "skill" => Some("harness_skill_used"),
        "team" => Some("harness_team_dispatched"),
        "heartbeat" => Some("harness_heartbeat"),
        _ => None,
    }
}

/// Field names a receipt never carries - and must never carry if they look secret.
const NEVER_FIELDS: [&str; 5] = ["token", "api_key", "apikey", "secret", "password"];

/// One recorded receipt: Python's insertion-ordered pairs, and where the line landed.
#[derive(Debug, Clone)]
pub struct Recorded {
    /// In Python's insertion order, which is what `--json` prints. **Not a `serde_json::Map`:**
    /// the default map is a `BTreeMap` and would sort them, and the ledger's sorted order and the
    /// JSON the caller sees are two different requirements on the same record.
    pub pairs: Vec<(String, Value)>,
    /// The shard the line was appended to (`unknown-<month>` when nothing identified the lab).
    pub shard: PathBuf,
    /// True when no lab identity was available: the line keeps `"lab": ""`, as Python's does.
    pub unattributed: bool,
}

impl Recorded {
    /// A field's value.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// A field as text, empty when it is not a string (Python's `rec.get(key, "")`).
    pub fn get_str(&self, key: &str) -> String {
        self.get(key)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    }

    fn to_receipt(&self) -> Receipt {
        let mut r = Receipt {
            kind: self.get_str("kind"),
            lab: self.get_str("lab"),
            ts: self.get_str("ts"),
            fields: Map::new(),
        };
        for (k, v) in &self.pairs {
            if !matches!(k.as_str(), "kind" | "lab" | "ts" | "schema") {
                r.fields.insert(k.clone(), v.clone());
            }
        }
        r
    }

    /// The line written to the shard: `json.dumps(record, ensure_ascii=False, sort_keys=True)`.
    pub fn line(&self) -> String {
        self.to_receipt().to_line()
    }

    /// `json.dumps(record, indent=2)` - Python's KEY ORDER and Python's non-ASCII escaping.
    ///
    /// A `serde_json::Value` would sort the keys and emit raw UTF-8, so the record is rendered
    /// from the ordered pairs instead: **the two implementations are compared as bytes here, which
    /// is what found the escaping difference in the `team roster` unit.**
    pub fn to_pretty(&self) -> String {
        pretty_ordered(&self.pairs)
    }
}

/// `json.dumps(dict(pairs), indent=2)` — Python's KEY ORDER and Python's non-ASCII escaping.
///
/// A `serde_json::Value` would sort the keys, so the document is rendered from an ordered list
/// instead. **The key order is the caller's insertion order, which is what Python prints**, and
/// the escaping is `ensure_ascii=True`, which is Python's default and the opposite of the ledger's.
pub fn pretty_ordered(pairs: &[(String, Value)]) -> String {
    let mut out = String::from("{\n");

    let last = pairs.len().saturating_sub(1);

    for (i, (k, v)) in pairs.iter().enumerate() {
        let comma = if i == last { "" } else { "," };

        let value =
            crate::registries::ensure_ascii(&serde_json::to_string_pretty(v).unwrap_or_default());

        // Python indents a nested container relative to its key, so every line after the first

        // moves over by the key's own indent.

        let value = value.replace('\n', "\n  ");

        out.push_str(&format!("  {}: {value}{comma}\n", Value::String(k.clone())));
    }

    out.push('}');

    out
}

/// Python's `value in (None, "", [], {})`: the values a receipt does NOT store.
///
/// Note what it is not: `0` and `false` are stored, because Python compares with `==` against
/// those four values and neither equals any of them. A "falsy" test would drop them.
fn py_empty(v: &Value) -> bool {
    match v {
        Value::Null => true,
        Value::String(s) => s.is_empty(),
        Value::Array(a) => a.is_empty(),
        Value::Object(o) => o.is_empty(),
        _ => false,
    }
}

/// `receipts.record`: one receipt, appended to its lab's shard and returned.
///
/// The fields arrive as ordered pairs because **Python's `record` builds a dict and `--json`
/// prints it in insertion order** - `ts`, `kind`, `schema`, `event`, then the caller's fields,
/// then `lab`.
///
/// Two behaviours are copied rather than corrected, both visible in the ledger:
///
/// * **A string is redacted; anything else is stored as given.** Python tests `isinstance(value,
///   str)`, so a list is NOT redacted element by element.
/// * **No lab identity is a warning, not a refusal.** The line keeps `"lab": ""` and lands in
///   `unknown-<month>`, which is why the shard is named for a gap rather than shared.
pub fn record(
    paths: &StatePaths,
    kind: &str,
    fields: Vec<(&str, Value)>,
    lab_env: Option<&str>,
) -> Result<Recorded, String> {
    if !crate::receiptsrules::KINDS.contains(&kind) {
        return Err(format!(
            "unknown receipt kind {}; expected one of {}",
            crate::prgates::py_repr_public(kind),
            python_tuple(&crate::receiptsrules::KINDS)
        ));
    }
    let mut pairs: Vec<(String, Value)> = vec![
        ("ts".to_string(), Value::String(now_rfc3339())),
        ("kind".to_string(), Value::String(kind.to_string())),
        ("schema".to_string(), Value::from(RECEIPT_SCHEMA)),
    ];
    if let Some(event) = event_type(kind) {
        pairs.push(("event".to_string(), Value::String(event.to_string())));
    }
    for (key, value) in fields {
        if NEVER_FIELDS.contains(&key.to_ascii_lowercase().as_str()) {
            continue;
        }
        if py_empty(&value) {
            continue;
        }
        let value = match &value {
            Value::String(s) => Value::String(crate::prgates::redact_public(s)),
            other => other.clone(),
        };
        pairs.push((key.to_string(), value));
    }
    let mut lab = pairs
        .iter()
        .find(|(k, _)| k == "lab")
        .and_then(|(_, v)| v.as_str())
        .unwrap_or("")
        .to_string();
    if lab.is_empty() {
        lab = lab_env.unwrap_or("").trim().to_string();
        pairs.push(("lab".to_string(), Value::String(lab.clone())));
    }
    let unattributed = lab.is_empty();
    let month = current_month();
    let recorded = Recorded {
        pairs,
        shard: shard_path(paths, &lab, &month),
        unattributed,
    };
    // LIVE shard, like the check-in and the other coordination receipts: `team dispatch` records that
    // a role was deployed, which every lab should see immediately and which must not dirty a checkout
    // (R10). `ledger-sync` carries the line into the committed shard before the PR.
    let shard = write_live_line(paths, &recorded.to_receipt(), &month)?;
    Ok(Recorded { shard, ..recorded })
}

/// Python's tuple repr, for the one error message that quotes the kind list.
fn python_tuple(items: &[&str]) -> String {
    let body: Vec<String> = items.iter().map(|s| format!("'{s}'")).collect();
    format!("({})", body.join(", "))
}

/// `YYYY-MM` for a Unix second count, in UTC — the month a shard belongs to.
///
/// UTC rather than local time on purpose: the shard name must be derivable identically by both
/// implementations on any machine, and a lab's local offset would put the same instant in different
/// months in different timezones. The receipt's own `ts` keeps the local offset for readability.
pub fn month_for(unix: i64) -> String {
    let days = unix.div_euclid(86_400);
    let (y, m, _d) = civil_from_days(days);
    format!("{y:04}-{m:02}")
}

/// Inverse of `days_from_civil`: days since the epoch to a Gregorian date. Howard Hinnant's algorithm.
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Now, as RFC3339 in UTC. The Python side writes a local offset; both are valid RFC3339 and the
/// reader parses either, so the port does not need to know the machine's timezone.
pub fn now_rfc3339() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let (y, m, d) = civil_from_days(secs.div_euclid(86_400));
    let tod = secs.rem_euclid(86_400);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        tod / 3600,
        (tod % 3600) / 60,
        tod % 60
    )
}

/// The month a shard should use right now.
pub fn current_month() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    month_for(secs)
}

/// Read the receipts from one shard file, for verification and tests.
pub fn read_shard(path: &Path) -> Vec<Value> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths() -> StatePaths {
        StatePaths::resolve(
            "/tmp/harness-writer-test",
            Some(Path::new("/tmp/none")),
            Some("/tmp/none"),
        )
    }

    #[test]
    fn a_receipt_line_is_sorted_json() {
        let mut r = Receipt::new("heartbeat", "kimi");
        r.ts = "2026-10-13T00:00:00-04:00".into();
        r = r.with("milestone", "m1").with("count", "2/3");
        // **Asserted as the exact line, not as a key list.** The previous version split on `","`,
        // which the PORT's compact form happened to produce - so when the separators were corrected
        // to Python's `", "` the split found nothing and the test would have passed on an empty key
        // list. A test that cannot fail is worse than no test, so this pins the whole line: sorted
        // keys, Python's separators, and Python's `ensure_ascii=False`.
        assert_eq!(
            r.to_line(),
            "{\"count\": \"2/3\", \"kind\": \"heartbeat\", \"lab\": \"kimi\", \"milestone\": \"m1\", \"schema\": 1, \"ts\": \"2026-10-13T00:00:00-04:00\"}"
        );
    }

    /// **Python's line, captured from a scratch repository by the unit's comparison script.** The
    /// timestamp is replaced by a fixed value so the fixture asserts FORMAT rather than the moment
    /// it was captured - the trap this port has fallen into three times.
    #[test]
    fn a_recorded_line_is_byte_for_byte_pythons() {
        let r = Recorded {
            pairs: vec![
                (
                    "ts".into(),
                    Value::String("2026-10-14T04:31:43-04:00".into()),
                ),
                ("kind".into(), Value::String("team".into())),
                ("schema".into(), Value::from(1)),
                (
                    "event".into(),
                    Value::String("harness_team_dispatched".into()),
                ),
                ("role".into(), Value::String("architecture-reviewer".into())),
                (
                    "route".into(),
                    Value::String("deepseek-account/deepseek-v4-pro".into()),
                ),
                ("count".into(), Value::from(2)),
                ("state".into(), Value::String("running".into())),
                (
                    "scope".into(),
                    Value::Array(vec![
                        Value::String("src/a.rs".into()),
                        Value::String("src/b.rs".into()),
                    ]),
                ),
                ("lab".into(), Value::String("deepseek".into())),
            ],
            shard: PathBuf::from("unused"),
            unattributed: false,
        };
        assert_eq!(
            r.line(),
            r#"{"count": 2, "event": "harness_team_dispatched", "kind": "team", "lab": "deepseek", "role": "architecture-reviewer", "route": "deepseek-account/deepseek-v4-pro", "schema": 1, "scope": ["src/a.rs", "src/b.rs"], "state": "running", "ts": "2026-10-14T04:31:43-04:00"}"#
        );
    }

    /// The `--json` form is Python's `json.dumps(rec, indent=2)`: insertion order, and `ensure_ascii`
    /// ON, which is the opposite of the ledger's.
    #[test]
    fn the_pretty_form_is_pythons_insertion_order_with_escaped_non_ascii() {
        let r = Recorded {
            pairs: vec![
                ("ts".into(), Value::String("T".into())),
                ("kind".into(), Value::String("team".into())),
                ("task".into(), Value::String("a\u{2014}b".into())),
                (
                    "scope".into(),
                    Value::Array(vec![Value::String("x".into())]),
                ),
            ],
            shard: PathBuf::from("unused"),
            unattributed: false,
        };
        assert_eq!(
            r.to_pretty(),
            concat!(
                "{\n",
                "  \"ts\": \"T\",\n",
                "  \"kind\": \"team\",\n",
                "  \"task\": \"a\\u2014b\",\n",
                "  \"scope\": [\n    \"x\"\n  ]\n",
                "}"
            )
        );
    }

    /// A scratch repo AND a writable scratch live tier: `record` writes the live shard (R10), so a
    /// fixture with the live tier pointed at nothing would now fail on the write rather than on the
    /// behaviour under test.
    fn scratch() -> (tempfile::TempDir, StatePaths) {
        let dir = tempfile::tempdir().expect("tempdir");
        let ledger = dir.path().join("docs").join("harness").join("ledger");
        std::fs::create_dir_all(&ledger).expect("mkdir");
        let live = dir.path().join("live");
        std::fs::create_dir_all(&live).expect("mkdir");
        let paths = StatePaths::resolve(dir.path(), None, Some(live.to_str().expect("utf8")));
        (dir, paths)
    }

    /// Python's `value in (None, "", [], {})` and the `_NEVER` key list, and the asymmetry that a
    /// string is redacted while a list is stored as given.
    #[test]
    fn record_skips_what_python_skips_and_redacts_what_python_redacts() {
        let (_d, paths) = scratch();
        let r = record(
            &paths,
            "team",
            vec![
                ("role", Value::String("claude".into())),
                ("milestone", Value::String("".into())),
                ("task", Value::Null),
                ("scope", Value::Array(vec![])),
                ("count", Value::from(0)),
                ("note", Value::String("ghp_abcdefghijkl".into())),
                ("secret", Value::String("leak".into())),
                (
                    "scope2",
                    Value::Array(vec![Value::String("sk-abcdefghij".into())]),
                ),
            ],
            Some("deepseek"),
        )
        .expect("recorded");
        assert_eq!(r.get_str("milestone"), "");
        assert!(r.get("task").is_none(), "an empty value is not stored");
        assert!(
            r.get("note").is_some(),
            "a non-empty, non-secret field is stored"
        );
        assert_eq!(r.get_str("note"), "<redacted>");
        assert!(
            r.get("secret").is_none(),
            "a secret-named field is never stored"
        );
        assert_eq!(r.get_str("count"), "");
        assert_eq!(
            r.get("count").and_then(Value::as_i64),
            Some(0),
            "zero is stored"
        );
        assert_eq!(
            r.get("scope2").and_then(Value::as_array).map(|a| a.len()),
            Some(1),
            "a list is stored as given, NOT redacted element by element"
        );
    }

    /// The order `--json` prints: Python builds `ts`, `kind`, `schema`, `event`, the caller's
    /// fields, then `lab`.
    #[test]
    fn record_keeps_pythons_insertion_order() {
        let (_d, paths) = scratch();
        let r = record(
            &paths,
            "team",
            vec![("role", Value::String("claude".into()))],
            Some("kimi"),
        )
        .expect("recorded");
        let keys: Vec<&str> = r.pairs.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, vec!["ts", "kind", "schema", "event", "role", "lab"]);
    }

    /// No lab identity is a warning, not a refusal: `"lab": ""` is written, to the shard named for
    /// the gap.
    #[test]
    fn an_unattributed_record_keeps_an_empty_lab_and_lands_in_unknown() {
        let (_d, paths) = scratch();
        let r = record(&paths, "team", vec![], None).expect("recorded");
        assert!(r.unattributed);
        assert_eq!(r.get_str("lab"), "");
        assert!(
            r.shard
                .ends_with(format!("unknown-{}.jsonl", current_month())),
            "{}",
            r.shard.display()
        );
        // LIVE tier, like the check-in: `team dispatch` records coordination, and coordination does
        // not dirty a checkout (R10). `ledger-sync` carries the line into the committed shard.
        assert!(
            r.shard.starts_with(live_ledger_dir(&paths)),
            "recorded into {} rather than the live tier",
            r.shard.display()
        );
        assert!(r.line().contains("\"lab\": \"\""));
    }

    #[test]
    fn an_unknown_kind_is_refused() {
        let (_d, paths) = scratch();
        let err = record(&paths, "nonsense", vec![], Some("kimi")).expect_err("refused");
        assert!(err.contains("unknown receipt kind 'nonsense'"), "{err}");
    }
    /// The contract's `required_always` for the ledger format. A row missing any of these fails
    /// `statecheck`, so the writer must never emit one.
    #[test]
    fn every_line_carries_the_four_required_keys() {
        let r = Receipt::new("receipt", "claude");
        let v: Value = serde_json::from_str(&r.to_line()).expect("valid json");
        for key in ["kind", "lab", "schema", "ts"] {
            assert!(v.get(key).is_some(), "missing {key} in {}", r.to_line());
        }
    }

    #[test]
    fn the_shard_is_named_for_the_lab_and_month() {
        let p = shard_path(&paths(), "Kimi", "2026-10");
        assert!(p.ends_with("ledger/kimi-2026-10.jsonl"), "{}", p.display());
        // An unidentified lab goes to `unknown`, not to a shared file — the conflict is still
        // avoided and the gap stays visible.
        let u = shard_path(&paths(), "", "2026-10");
        assert!(
            u.ends_with("ledger/unknown-2026-10.jsonl"),
            "{}",
            u.display()
        );
    }

    /// **The frozen kind list is enforced at WRITE time.** `receipt --kind <anything>` used to write
    /// the line and `check` then hard-errored on it at parse (`receipts.parse`), so the harness
    /// accepted a record its own reader rejects and the only fix was to find the line and delete it
    /// (reported on #2028). The check lives here because every writer passes through `validate`.
    #[test]
    fn an_unknown_kind_is_refused_before_it_reaches_the_record() {
        let err = Receipt::new("nonsense_kind", "kimi")
            .validate()
            .expect_err("an unknown kind must be refused");
        assert!(err.contains("unknown receipt kind"), "{err}");
        assert!(
            err.contains("heartbeat"),
            "the refusal must name the frozen list: {err}"
        );
        // And every kind the contract carries IS accepted, so this check cannot drift from it.
        for kind in crate::receiptsrules::KINDS {
            assert!(
                Receipt::new(kind, "kimi").validate().is_ok(),
                "{kind} is in the frozen list and must be accepted"
            );
        }
    }

    #[test]
    fn a_receipt_without_a_kind_or_timestamp_is_rejected_before_it_reaches_the_record() {
        assert!(Receipt::new("", "kimi").validate().is_err());
        let mut no_ts = Receipt::new("heartbeat", "kimi");
        no_ts.ts = String::new();
        assert!(no_ts.validate().is_err());
        assert!(Receipt::new("heartbeat", "kimi").validate().is_ok());
    }

    /// **An unnamed lab is warned about, not refused.** Python keeps `"lab": ""` in the line and
    /// files it under `unknown-<month>`: the record survives and the gap is visible, where a refusal
    /// would have lost it. The shard name is the fallback this task list called for.
    #[test]
    fn an_unnamed_lab_is_recorded_under_unknown_rather_than_refused() {
        let unattributed = Receipt::new("heartbeat", "");
        assert!(unattributed.validate().is_ok(), "the record must survive");
        assert!(unattributed.unattributed(), "and the caller must be told");
        assert!(
            unattributed.to_line().contains("\"lab\": \"\""),
            "{}",
            unattributed.to_line()
        );
        // A whitespace name cannot name a lab either; like Python, the line keeps what it was given.
        assert!(Receipt::new("heartbeat", "  ").unattributed());
        let named = Receipt::new("heartbeat", "kimi");
        assert!(!named.unattributed());
        // The file it lands in, through the same path a write uses.
        let p = live_shard_path(&paths(), "", "2026-10");
        assert!(p.ends_with("unknown-2026-10.jsonl"), "{}", p.display());
    }

    #[test]
    fn months_are_derived_in_utc() {
        // Epochs computed independently (`datetime.fromisoformat(...).timestamp()`), because a
        // hand-written constant is exactly the kind of thing that is wrong by a year and still
        // looks plausible: the first version of this test asserted 1_760_313_600 was October 2026
        // when it is October 2025.
        assert_eq!(month_for(1_791_849_600), "2026-10", "2026-10-13T00:00:00Z");
        assert_eq!(month_for(0), "1970-01", "the epoch");
        assert_eq!(month_for(1_767_225_599), "2025-12", "2025-12-31T23:59:59Z");
        assert_eq!(
            month_for(1_767_225_600),
            "2026-01",
            "2026-01-01T00:00:00Z, the next second"
        );
    }

    /// The property that matters: a write must never lose an existing line.
    #[test]
    fn appending_preserves_earlier_receipts() {
        let dir = std::env::temp_dir().join(format!("harness-writer-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let p = StatePaths::resolve(&dir, Some(Path::new("/tmp/none")), Some("/tmp/none"));
        let first = Receipt::new("heartbeat", "kimi").with("milestone", "m1");
        let second = Receipt::new("skill", "kimi").with("skill", "expert-dispatch");
        append(&p, &first, "2026-10").expect("first write");
        append(&p, &second, "2026-10").expect("second write");
        let rows = read_shard(&shard_path(&p, "kimi", "2026-10"));
        assert_eq!(
            rows.len(),
            2,
            "the first receipt must survive the second write"
        );
        assert_eq!(rows[0]["kind"], "heartbeat");
        assert_eq!(rows[1]["kind"], "skill");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn two_labs_never_write_the_same_shard() {
        assert_ne!(
            shard_path(&paths(), "kimi", "2026-10"),
            shard_path(&paths(), "codex", "2026-10")
        );
    }

    #[test]
    fn a_timestamp_is_rfc3339_with_a_z() {
        let ts = now_rfc3339();
        assert!(ts.ends_with('Z'), "{ts}");
        assert_eq!(ts.len(), 20, "{ts}");
        assert!(
            crate::state::parse_iso_unix(&ts).is_some(),
            "must be parseable: {ts}"
        );
    }
}

// ------------------------------------------------------------------------ live tier and sync
//
// A receipt is written to the LIVE tier first, for the same reason presence is: a check-in every few
// minutes must not dirty a working checkout. The committed shard is then updated at DELIVERY, from a
// command the lab runs before opening its PR.
//
// This is not "the ledger moved out of tree". The committed shard remains the authority and my
// shard-integrity guard still compares it against `origin/main`; the live tier is the write path, and
// the sync is what carries receipts into the record review actually sees.

/// The live ledger directory: `~/.local/share/uor-r4/harness/ledger`.
pub fn live_ledger_dir(paths: &StatePaths) -> PathBuf {
    paths.live.join("ledger")
}

/// The live shard for a lab and month.
pub fn live_shard_path(paths: &StatePaths, lab: &str, month: &str) -> PathBuf {
    live_ledger_dir(paths).join(format!("{}-{month}.jsonl", slug(lab)))
}

/// Append a receipt to the LIVE shard.
///
/// Appends rather than rewriting, so two labs checking in at once cannot clobber each other's lines.
pub fn append_live(paths: &StatePaths, receipt: &Receipt, month: &str) -> Result<PathBuf, String> {
    receipt.validate()?;
    write_live_line(paths, receipt, month)
}

/// The live write, without the lab check - `record` needs it for the same reason `write_line` exists:
/// an unattributed receipt is WARNED about and preserved, never refused.
fn write_live_line(paths: &StatePaths, receipt: &Receipt, month: &str) -> Result<PathBuf, String> {
    let shard = live_shard_path(paths, &receipt.lab, month);
    if let Some(parent) = shard.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    let line = receipt.to_line();
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&shard)
        .map_err(|e| format!("cannot open {}: {e}", shard.display()))?;
    use std::io::Write;
    writeln!(file, "{line}").map_err(|e| format!("cannot append to {}: {e}", shard.display()))?;
    Ok(shard)
}

/// What a sync did, so the caller can report it rather than guess.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncReport {
    pub shard: String,
    /// Receipts the live tier held that the committed shard did not.
    pub added: usize,
    /// Live receipts already present in the committed shard, by timestamp and kind.
    pub already_committed: usize,
    /// False when there was nothing live to sync.
    pub wrote: bool,
}

/// Copy live receipts into the committed shard, APPEND-ONLY.
///
/// Identity is `(ts, kind)` within the lab's own shard. A lab does not record two receipts of the
/// same kind in the same second through the harness — every receipt carries a distinct `event` or
/// action — so this does not collapse distinct work, and it makes the sync idempotent: running it
/// twice adds nothing the second time.
///
/// **Committed lines are never rewritten or removed.** The sync only appends, which is the property
/// the shard guard enforces and the reason #2305's data loss was possible at all.
pub fn sync_live_to_committed(
    paths: &StatePaths,
    lab: &str,
    month: &str,
) -> Result<SyncReport, String> {
    // R10: the sync is a deliberate write into a checkout, and it is the caller's own checkout that
    // carries the lines into review. The shared checkout stays clean; the live tier is where a
    // receipt goes when there is no checkout to write.
    //
    // Refused BEFORE any work, so a dry run and a real sync agree about permission: a no-op sync
    // from the shared checkout is still the wrong checkout to have run it in.
    crate::state::refuse_shared_checkout(paths, "the committed ledger shard")?;
    let (mut report, add) = plan_sync(paths, lab, month)?;
    if add.is_empty() {
        return Ok(report);
    }
    let committed = std::path::PathBuf::from(&report.shard);
    if let Some(parent) = committed.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&committed)
        .map_err(|e| format!("cannot open {}: {e}", committed.display()))?;
    use std::io::Write;
    for line in &add {
        writeln!(file, "{line}").map_err(|e| format!("cannot append: {e}"))?;
    }
    report.added = add.len();
    report.wrote = true;
    Ok(report)
}

/// What a sync WOULD append, and the lines themselves - **writing nothing**.
///
/// **This exists because `--dry-run` used to write.** The flag was documented on `ledger-sync` and
/// never read: the command went straight to `sync_live_to_committed`, so a lab asking what the sync
/// would do got the effect instead of the answer, and a checkout that should have stayed clean was
/// appended to. Measured on 15 October, on the delivery run for this very change. A read-only flag
/// that writes is worse than a missing flag: it teaches a lab the wrong thing about R10.
///
/// The refusal is deliberately NOT here: planning reads, and reading the shared checkout is exactly
/// what R10 permits. The write half asks `refuse_shared_checkout` before it touches anything.
pub fn plan_sync(
    paths: &StatePaths,
    lab: &str,
    month: &str,
) -> Result<(SyncReport, Vec<String>), String> {
    let live = live_shard_path(paths, lab, month);
    let committed = shard_path(paths, lab, month);
    let mut report = SyncReport {
        shard: committed.display().to_string(),
        ..Default::default()
    };
    let Ok(live_text) = std::fs::read_to_string(&live) else {
        return Ok((report, Vec::new())); // nothing live: not an error, just nothing to carry
    };
    let committed_text = std::fs::read_to_string(&committed).unwrap_or_default();
    let have: std::collections::BTreeSet<String> = committed_text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str::<serde_json::Value>(l)
                .ok()
                .map(|v| format!("{}|{}", key_of(&v, "ts"), key_of(&v, "kind")))
                .unwrap_or_else(|| l.trim().to_string())
        })
        .collect();

    let mut add: Vec<String> = Vec::new();
    for line in live_text.lines().filter(|l| !l.trim().is_empty()) {
        let key = serde_json::from_str::<serde_json::Value>(line)
            .ok()
            .map(|v| format!("{}|{}", key_of(&v, "ts"), key_of(&v, "kind")))
            .unwrap_or_else(|| line.trim().to_string());
        if have.contains(&key) {
            report.already_committed += 1;
        } else {
            add.push(line.trim().to_string());
        }
    }
    report.added = add.len();
    Ok((report, add))
}

fn key_of(v: &serde_json::Value, field: &str) -> String {
    v.get(field)
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod live_tier_tests {
    use super::*;

    struct Fixture {
        /// Held so the scratch directory outlives the test; never read.
        _dir: tempfile::TempDir,
        paths: StatePaths,
    }

    impl Fixture {
        fn new() -> Fixture {
            let dir = tempfile::tempdir().expect("tempdir");
            let repo = dir.path().join("repo");
            let live = dir.path().join("live");
            std::fs::create_dir_all(&repo).expect("repo");
            std::fs::create_dir_all(&live).expect("live");
            let paths = StatePaths::resolve(&repo, None, Some(live.to_str().expect("utf8")));
            Fixture { _dir: dir, paths }
        }
    }

    fn receipt(lab: &str, kind: &str, ts: &str) -> Receipt {
        Receipt {
            ts: ts.into(),
            lab: lab.into(),
            kind: kind.into(),
            fields: Default::default(),
        }
    }

    /// The reason the live tier exists: a check-in every five minutes must not dirty a checkout.
    #[test]
    fn a_receipt_appends_to_the_live_shard_not_a_checkout() {
        let f = Fixture::new();
        let r = receipt("deepseek", "heartbeat", "2026-10-13T09:00:00-04:00");
        let shard = append_live(&f.paths, &r, "2026-10").expect("append");
        assert!(
            shard.starts_with(live_ledger_dir(&f.paths)),
            "{}",
            shard.display()
        );
        assert!(shard.is_file());
        // And nothing was created inside the repo.
        let committed = shard_path(&f.paths, "deepseek", "2026-10");
        assert!(!committed.exists(), "the checkout must stay untouched");
    }

    /// Appending twice keeps both lines: two labs checking in at once must not clobber each other.
    #[test]
    fn the_live_shard_only_ever_grows() {
        let f = Fixture::new();
        for ts in ["2026-10-13T09:00:00-04:00", "2026-10-13T09:05:00-04:00"] {
            append_live(&f.paths, &receipt("deepseek", "heartbeat", ts), "2026-10")
                .expect("append");
        }
        let text = std::fs::read_to_string(live_shard_path(&f.paths, "deepseek", "2026-10"))
            .expect("read");
        assert_eq!(text.lines().filter(|l| !l.trim().is_empty()).count(), 2);
    }

    /// **A `--dry-run` must not write.** The flag was documented on `ledger-sync` and never read, so
    /// asking what the sync would do created and appended to the committed shard - measured on the
    /// delivery run for this very change, when the dry run added 101 lines.
    #[test]
    fn planning_a_sync_writes_nothing() {
        let f = Fixture::new();
        append_live(
            &f.paths,
            &receipt("deepseek", "heartbeat", "2026-10-13T09:00:00-04:00"),
            "2026-10",
        )
        .expect("append");
        let committed = shard_path(&f.paths, "deepseek", "2026-10");
        let (report, add) = plan_sync(&f.paths, "deepseek", "2026-10").expect("plan");
        assert_eq!(add.len(), 1, "the plan names the line it would carry");
        assert_eq!(report.added, 1);
        assert!(!report.wrote, "a plan is not a write");
        assert!(
            !committed.exists(),
            "a dry run must not create the committed shard: {} exists",
            committed.display()
        );
        // And the real sync then writes exactly what the plan promised.
        let done = sync_live_to_committed(&f.paths, "deepseek", "2026-10").expect("sync");
        assert!(done.wrote);
        assert_eq!(done.added, add.len());
        assert!(committed.is_file());
    }

    /// The delivery step: live receipts reach the committed shard that review sees.
    #[test]
    fn sync_carries_live_receipts_into_the_committed_shard() {
        let f = Fixture::new();
        append_live(
            &f.paths,
            &receipt("deepseek", "heartbeat", "2026-10-13T09:00:00-04:00"),
            "2026-10",
        )
        .expect("append");
        let report = sync_live_to_committed(&f.paths, "deepseek", "2026-10").expect("sync");
        assert_eq!(report.added, 1);
        assert!(report.wrote);
        let committed =
            std::fs::read_to_string(shard_path(&f.paths, "deepseek", "2026-10")).expect("read");
        assert!(
            committed.contains("2026-10-13T09:00:00-04:00"),
            "{committed}"
        );
    }

    /// Running the sync twice adds nothing the second time. A delivery step that is not idempotent
    /// gets run twice by someone in a hurry and duplicates the record.
    #[test]
    fn sync_is_idempotent() {
        let f = Fixture::new();
        append_live(
            &f.paths,
            &receipt("deepseek", "heartbeat", "2026-10-13T09:00:00-04:00"),
            "2026-10",
        )
        .expect("append");
        let first = sync_live_to_committed(&f.paths, "deepseek", "2026-10").expect("sync");
        assert_eq!(first.added, 1);
        let second = sync_live_to_committed(&f.paths, "deepseek", "2026-10").expect("sync");
        assert_eq!(second.added, 0, "the second run must add nothing");
        assert_eq!(second.already_committed, 1);
        let committed =
            std::fs::read_to_string(shard_path(&f.paths, "deepseek", "2026-10")).expect("read");
        assert_eq!(
            committed.lines().filter(|l| l.contains("09:00:00")).count(),
            1,
            "the receipt must not be duplicated: {committed}"
        );
    }

    /// **The property the shard guard enforces.** A sync must never rewrite or drop committed lines —
    /// appending is what makes the ledger safe, and rewriting is how #2305 lost Kimi's receipts.
    #[test]
    fn sync_never_rewrites_or_drops_committed_lines() {
        let f = Fixture::new();
        let committed_path = shard_path(&f.paths, "deepseek", "2026-10");
        std::fs::create_dir_all(committed_path.parent().expect("parent")).expect("dir");
        let existing =
            "{\"ts\":\"2026-10-01T00:00:00-04:00\",\"kind\":\"skill\",\"lab\":\"deepseek\"}\n";
        std::fs::write(&committed_path, existing).expect("seed");
        append_live(
            &f.paths,
            &receipt("deepseek", "heartbeat", "2026-10-13T09:00:00-04:00"),
            "2026-10",
        )
        .expect("append");
        sync_live_to_committed(&f.paths, "deepseek", "2026-10").expect("sync");
        let after = std::fs::read_to_string(&committed_path).expect("read");
        assert!(
            after.starts_with(existing),
            "existing committed lines must be untouched and first: {after}"
        );
        assert!(
            after.contains("09:00:00"),
            "and the live line appended after them"
        );
    }

    /// Nothing live is not an error: a lab that has not checked in has nothing to carry.
    #[test]
    fn syncing_with_no_live_shard_is_a_no_op_not_a_failure() {
        let f = Fixture::new();
        let report = sync_live_to_committed(&f.paths, "deepseek", "2026-10").expect("sync");
        assert_eq!(report.added, 0);
        assert!(!report.wrote);
    }

    /// Each lab syncs its OWN shard. A sync that wrote into another lab's shard would attribute one
    /// lab's work to another.
    #[test]
    fn sync_is_scoped_to_one_lab() {
        let f = Fixture::new();
        append_live(
            &f.paths,
            &receipt("kimi", "heartbeat", "2026-10-13T09:00:00-04:00"),
            "2026-10",
        )
        .expect("append");
        let report = sync_live_to_committed(&f.paths, "deepseek", "2026-10").expect("sync");
        assert_eq!(
            report.added, 0,
            "kimi's receipts are not deepseek's to carry"
        );
        assert!(report.shard.contains("deepseek"));
    }
}
