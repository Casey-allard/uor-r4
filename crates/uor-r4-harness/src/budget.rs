//! `budget` — cumulative cost and live compute (D30 port of `budget.py`).
//!
//! # What this measures, and what it refuses to
//!
//! A budget view is easy to make wrong in the flattering direction, so the Python this ports is explicit
//! about three refusals, and they are the reason the module is worth porting carefully:
//!
//! * **A rate is not a charge.** `usd_per_h` is reported separately and never summed as spend.
//! * **A bool is not a number.** `isinstance(True, int)` is True in Python, so `_num` refuses bools
//!   *before* the numeric branch rather than counting a JSON `true` as 1.
//! * **An unreadable line is not a lost file.** A ledger is append-only evidence, so the tolerant reader
//!   costs a corrupt line that line only.

use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;
use serde_json::Value;

use crate::ordered::Ordered;

/// The word the harness prints where a figure is absent - never a zero standing in for one.
pub const ABSENT: &str = "ABSENT";

/// Owner caps (AGENTS.md): running pods and dollars per hour, across all labs.
pub const POD_CAP: i64 = 4;
pub const RATE_CAP_USD_PER_H: f64 = 8.00;
/// The owner's local working band, in GB, and the retained-evidence stop margin.
///
/// **Integers, because Python's `DISK_BAND_GB = (30, 70)` is a tuple of ints** and every place it is
/// printed - `owner band 30-70 GB` and `"band_gb": [30, 70]` - shows no decimal. The comparison against
/// Python caught `30.0-70.0` on the first run.
pub const DISK_BAND_GB: (i64, i64) = (30, 70);
pub const STORAGE_MARGIN_MIB: i64 = 128;

/// The cumulative ledger the task names, plus the per-experiment chain the repo keeps.
pub const MODEL_TIME_LEDGER: &str = ".uor-models/native-joint-learning-2026-09-04/model-time.json";
pub const RESOURCE_LEDGER_GLOB: &str = "*/resource-ledger.json";

/// Receipts may record a cost under any of these names.
pub const COST_KEYS: [&str; 5] = ["cost_usd", "usd", "usd_total", "spend_usd", "cost"];
/// **Rates are not totals** - they are reported separately so they cannot be summed as spend.
pub const RATE_KEYS: [&str; 3] = ["usd_per_h", "rate_usd_per_h", "usd_per_hour"];

pub const POD_SCRIPT_REL: &str = "scripts/pod/uor-pod";
pub const HYGIENE_COMMAND: &str = "scripts/storage/uor-hygiene --apply";

/// Python's `str()` for a JSON value.
pub fn py_str_of(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "None".to_string(),
        Value::Bool(b) => if *b { "True" } else { "False" }.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

/// `_num`: a number, or `None`.
///
/// **The bool check comes FIRST**, because `isinstance(True, int)` is True in Python: without it a JSON
/// `true` in a cost field would be counted as one dollar.
pub fn num_of(value: &Value) -> Option<f64> {
    match value {
        Value::Bool(_) => None,
        Value::Number(n) => n.as_f64(),
        // `re.search(r"-?[0-9]+(?:\.[0-9]+)?", value.replace(",", ""))` - a number out of a string,
        // so "$1,234.50" is 1234.5 and "TBD" is None (uncounted, never zero).
        Value::String(s) => first_number(&s.replace(',', "")),
        _ => None,
    }
}

fn first_number(s: &str) -> Option<f64> {
    static NUMBER: OnceLock<Regex> = OnceLock::new();
    let rx = NUMBER.get_or_init(|| Regex::new(r"-?[0-9]+(?:\.[0-9]+)?").expect("a valid pattern"));
    rx.find(s).and_then(|m| m.as_str().parse::<f64>().ok())
}

/// `_read_jsonl_tolerant`: read JSONL, **skipping the lines that do not parse**.
///
/// The strict reader opens UTF-8 and loses every record after one undecodable byte; a ledger is
/// append-only evidence, so a corrupt line must cost that line only. A missing file is empty, not an
/// error, and **only objects are kept** - a bare `[1, 2]` line is dropped.
pub fn read_jsonl_tolerant(path: &Path) -> Vec<Value> {
    if !path.is_file() {
        return Vec::new();
    }
    let Ok(bytes) = std::fs::read(path) else {
        return Vec::new();
    };
    // `errors="replace"`: one bad byte must not lose the file.
    let text = String::from_utf8_lossy(&bytes);
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(rec) = serde_json::from_str::<Value>(line) {
            if rec.is_object() {
                out.push(rec);
            }
        }
    }
    out
}

/// Python's `round(x, places)`.
///
/// **`round_ties_even`, not `round`.** Python rounds a tie to the EVEN digit; Rust's `f64::round` rounds
/// it away from zero. On a single value that is invisible - on this ledger it is not: 211 pod events are
/// each rounded to four decimals as they accumulate, and the two rules disagreed enough times to make the
/// total `371.9637` where Python says `371.9626`. The comparison against Python is what found it.
pub fn round_to(x: f64, places: i32) -> f64 {
    // **Round the DECIMAL, as Python does.** `(x * 10^p).round_ties_even() / 10^p` multiplies first,
    // which introduces binary error before the tie is even seen: it left this ledger's pod total at
    // `371.9625` against Python's `371.9626`. Formatting with a precision is correctly rounded on the
    // decimal value, ties to even, which is Python's rule.
    format!("{:.*}", places.max(0) as usize, x)
        .parse::<f64>()
        .unwrap_or(x)
}

/// Python's `str(row.get(key) or "(unset)")`: **a falsy value groups with the missing ones**, so a `0`,
/// an empty string, `false` and `null` all land under `(unset)` rather than under their own key.
pub fn group_key(row: &Value, key: &str) -> String {
    let v = row.get(key);
    let truthy = match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|f| f != 0.0),
        Some(Value::String(s)) => !s.is_empty(),
        Some(_) => true,
    };
    match (truthy, v) {
        (true, Some(v)) => py_str_of(v),
        _ => "(unset)".to_string(),
    }
}

/// `_sum_by`: accumulate `usd` per group, **rounding to four decimals at every step**, then sort by key.
///
/// Rounding per step is not the same number as summing and rounding once, and the sorted order is part of
/// the output: a plain dict would have been insertion-ordered.
pub fn sum_by(rows: &[Value], key: &str) -> Vec<(String, f64)> {
    let mut out: Vec<(String, f64)> = Vec::new();
    for row in rows {
        let k = group_key(row, key);
        let usd = row.get("usd").and_then(num_of).unwrap_or(0.0);
        match out.iter_mut().find(|(existing, _)| *existing == k) {
            Some((_, total)) => *total = round_to(*total + usd, 4),
            None => out.push((k, round_to(usd, 4))),
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// `_fmt_map`: `"k $v.vv"` joined with commas, or `ABSENT` when there is nothing to report.
///
/// **`%.2f`, fixed two decimals** - not `%g`, which the score lines use.
pub fn fmt_map(mapping: &[(String, f64)]) -> String {
    if mapping.is_empty() {
        return ABSENT.to_string();
    }
    mapping
        .iter()
        .map(|(k, v)| format!("{k} ${v:.2}"))
        .collect::<Vec<_>>()
        .join(", ")
}

// ------------------------------------------------------------------------------- timestamps

/// `_gb`: bytes to gibibytes, the unit the disk band is stated in.
pub fn gb(bytes: f64) -> f64 {
    bytes / (1024.0f64.powi(3))
}

/// Days from 1970-01-01 for a civil date (Howard Hinnant's algorithm) - no dependency needed.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// `_parse_ts` as epoch seconds, or `None`.
///
/// Python does `datetime.fromisoformat(ts.replace("Z", "+00:00"))` and **assumes UTC when the string
/// carries no offset**. A naive timestamp is therefore NOT local time, and a pod ledger written in local
/// time would be read two to five hours off - which is why the assumption is copied rather than made
/// clever: it is what the durations in `pod_ledger_spend` were computed with.
pub fn parse_ts(ts: &str) -> Option<i64> {
    let s = ts.trim();
    if s.len() < 19 {
        return None;
    }
    let b = s.as_bytes();
    if b[4] != b'-' || b[7] != b'-' || b[10] != b'T' {
        return None;
    }
    let num = |a: usize, z: usize| s.get(a..z)?.parse::<i64>().ok();
    let (y, mo, d) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    let (h, mi, sec) = (num(11, 13)?, num(14, 16)?, num(17, 19)?);
    let mut epoch = days_from_civil(y, mo, d) * 86400 + h * 3600 + mi * 60 + sec;
    let mut rest = &s[19..];
    // Fractional seconds are accepted and dropped: only differences are used.
    if rest.starts_with('.') {
        let digits = rest[1..].chars().take_while(|c| c.is_ascii_digit()).count();
        rest = &rest[1 + digits..];
    }
    if rest.starts_with('+') || rest.starts_with('-') {
        let sign = if rest.starts_with('-') { -1 } else { 1 };
        let body = &rest[1..];
        let parts: Vec<&str> = body.split(':').collect();
        let oh: i64 = parts
            .first()
            .and_then(|p| p.get(0..2))
            .and_then(|x| x.parse().ok())
            .unwrap_or(0);
        let om: i64 = parts
            .get(1)
            .and_then(|p| p.get(0..2))
            .and_then(|x| x.parse().ok())
            .unwrap_or(0);
        epoch -= sign * (oh * 3600 + om * 60);
    }
    Some(epoch)
}

// ------------------------------------------------------------------- resource ledger

/// One `resource-ledger.json`, as `resource_ledger` records it.
#[derive(Debug, Clone, Default)]
pub struct LedgerEntry {
    /// **Repo-relative**, which is what `chain` then matches pointers against.
    pub path: String,
    pub schema: String,
    pub status: String,
    pub updated_utc: String,
    pub increment_seconds: Value,
    pub projection_seconds: Value,
    pub cumulative_projected_seconds: Value,
    pub lab_cumulative_projection_seconds: Value,
    pub previous_ledger: String,
    pub new_pods: Value,
    pub approx_gpu_cost_usd: Value,
}

/// The result of walking the pointer chain.
#[derive(Debug, Clone, Default)]
pub struct Chain {
    pub head: String,
    pub links: usize,
    pub files: usize,
    pub increment_seconds: f64,
    pub increment_hours: f64,
    pub cumulative_projected_seconds: Option<f64>,
    pub cumulative_projected_hours: Option<f64>,
    pub note: String,
}

const CHAIN_NOTE: &str =
    "cumulative_projected_seconds is the newest ledger's own projection; the walked \
increments are shown as an internal-consistency check, and a missing previous_ledger ends the walk";

/// `_chain`: walk `previous_ledger` back from the newest head to recover the cumulative total.
///
/// **`cumulative_projected_seconds` comes from the HEAD ONLY.** The docstring is explicit that the walked
/// increments are an internal-consistency check and not a new total, so a port must not sum its way to a
/// cumulative figure - which is exactly the mistake a helpful implementation would make.
pub fn chain(entries: &[LedgerEntry]) -> Chain {
    if entries.is_empty() {
        return Chain {
            head: String::new(),
            links: 0,
            files: 0,
            increment_seconds: 0.0,
            increment_hours: 0.0,
            cumulative_projected_seconds: None,
            cumulative_projected_hours: None,
            note: "no resource-ledger.json found".to_string(),
        };
    }
    // `follow`: an exact path, else a three-way SUFFIX match, because the repo writes pointers as
    // repo-relative paths, absolute pod paths, or a bare name. First match in entry order wins.
    let follow = |pointer: &str| -> Option<usize> {
        let pointer = pointer.trim();
        if pointer.is_empty() {
            return None;
        }
        if let Some(i) = entries.iter().position(|e| e.path == pointer) {
            return Some(i);
        }
        entries.iter().position(|e| {
            let base = e.path.rsplit('/').next().unwrap_or(e.path.as_str());
            pointer.ends_with(e.path.as_str())
                || e.path.ends_with(pointer)
                || pointer.ends_with(base)
        })
    };
    // Heads: entries no other entry's pointer resolves to (an identity test, as Python does).
    let heads: Vec<usize> = (0..entries.len())
        .filter(|i| {
            !(0..entries.len()).any(|j| {
                !entries[j].previous_ledger.trim().is_empty()
                    && follow(&entries[j].previous_ledger) == Some(*i)
            })
        })
        .collect();
    let pool: Vec<usize> = if heads.is_empty() {
        (0..entries.len()).collect()
    } else {
        heads
    };
    // `sorted(pool, key=updated_utc, reverse=True)[0]` - a STRING comparison, and the first entry wins a
    // tie, which is why this keeps the strictly-greater one rather than the last.
    let mut head = pool[0];
    for i in &pool[1..] {
        if entries[*i].updated_utc > entries[head].updated_utc {
            head = *i;
        }
    }
    let mut node = Some(head);
    let mut links = 0usize;
    let mut increment = 0.0f64;
    let mut seen: Vec<String> = Vec::new();
    while let Some(i) = node {
        if seen.contains(&entries[i].path) {
            break;
        }
        seen.push(entries[i].path.clone());
        links += 1;
        if let Some(inc) = num_of(&entries[i].increment_seconds) {
            increment += inc;
        }
        let pointer = entries[i].previous_ledger.trim().to_string();
        if pointer.is_empty() {
            break;
        }
        match follow(&pointer) {
            Some(next) if !seen.contains(&entries[next].path) => node = Some(next),
            _ => break,
        }
    }
    let cumulative = entries[head].cumulative_projected_seconds.clone();
    Chain {
        head: entries[head].path.clone(),
        links,
        files: entries.len(),
        increment_seconds: round_to(increment, 1),
        increment_hours: round_to(increment / 3600.0, 2),
        cumulative_projected_seconds: num_of(&cumulative),
        cumulative_projected_hours: num_of(&cumulative).map(|c| round_to(c / 3600.0, 2)),
        note: CHAIN_NOTE.to_string(),
    }
}

/// `_latest_cumulative`: the highest cumulative projection any ledger states, with its path and field.
///
/// Two schemas coexist and **neither is summed, because each is already a cumulative figure**. The newer
/// `lab_cumulative_projection_seconds` is checked first, and a tie keeps the EARLIER entry (`>` not `>=`).
pub fn latest_cumulative(entries: &[LedgerEntry]) -> (Option<f64>, String, String, Option<f64>) {
    let mut best: (Option<f64>, String, String) = (None, String::new(), String::new());
    for e in entries {
        for (field, value) in [
            (
                "lab_cumulative_projection_seconds",
                &e.lab_cumulative_projection_seconds,
            ),
            (
                "cumulative_projected_seconds",
                &e.cumulative_projected_seconds,
            ),
        ] {
            let Some(v) = num_of(value) else { continue };
            if best.0.is_none() || v > best.0.unwrap_or(f64::MIN) {
                best = (Some(v), e.path.clone(), field.to_string());
            }
        }
    }
    let hours = best.0.map(|s| round_to(s / 3600.0, 2));
    (best.0, best.1, best.2, hours)
}

// ------------------------------------------------------------------------------ disk + pods

fn disk_error(path: &Path, error: &str) -> Value {
    serde_json::json!({
        "ok": false,
        "error": error,
        "path": path.display().to_string(),
    })
}

/// `disk_free`: free space where the work happens, **measured - never assumed**.
///
/// Python calls `shutil.disk_usage`. Rust's std has no equivalent and this crate depends only on
/// `regex`/`serde`/`serde_json`, so the figure comes from `df -k <path>`: the same numbers (1024-byte
/// blocks, the `Avail` column is `shutil`'s `free`), no new dependency, and no `unsafe` block in a crate
/// that holds `forbid(unsafe_code)` elsewhere.
///
/// **The band comparisons use the UNROUNDED figure.** Rounding first would flip a boundary case at exactly
/// the floor, which is the one place this number is read as a decision.
pub fn disk_free(path: &Path) -> Value {
    let out = std::process::Command::new("df")
        .arg("-k")
        .arg(path)
        .output();
    let Ok(o) = out else {
        return disk_error(path, "OSError: df could not be run");
    };
    if !o.status.success() {
        return disk_error(
            path,
            &format!("OSError: df exited {}", o.status.code().unwrap_or(-1)),
        );
    }
    let text = String::from_utf8_lossy(&o.stdout);
    let Some(line) = text.lines().last() else {
        return disk_error(path, "OSError: no df output");
    };
    // Columns are Filesystem, 1024-blocks, Used, Avail - the mount point may contain spaces, so only
    // the leading numeric columns are read.
    let cols: Vec<&str> = line.split_whitespace().collect();
    if cols.len() < 4 {
        return disk_error(path, "OSError: unexpected df output");
    }
    let blocks = |s: &str| s.parse::<i64>().unwrap_or(0);
    let total = blocks(cols[1]) * 1024;
    let free = blocks(cols[3]) * 1024;
    let free_gb = gb(free as f64);
    let (low_i, high_i) = DISK_BAND_GB;
    let (low, high) = (low_i as f64, high_i as f64);
    serde_json::json!({
        "ok": true,
        "path": path.display().to_string(),
        "free_bytes": free,
        "total_bytes": total,
        "free_gb": round_to(free_gb, 2),
        "band_gb": [low_i, high_i],
        "in_band": low <= free_gb && free_gb <= high,
        "below_floor": free_gb < low,
        "above_band": free_gb > high,
        "margin_mib": STORAGE_MARGIN_MIB,
        "hygiene": HYGIENE_COMMAND,
    })
}

/// `pod_state_dir`: `$UOR_POD_STATE`, else the lab's compute directory.
pub fn pod_state_dir() -> std::path::PathBuf {
    if let Ok(env) = std::env::var("UOR_POD_STATE") {
        if !env.is_empty() {
            return std::path::PathBuf::from(env);
        }
    }
    let home = std::env::var("HOME").unwrap_or_default();
    std::path::PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("uor-r4")
        .join("compute")
}

/// `pod_ledger_spend`: actual spend from the local pod event ledger - **local machine state, not committed
/// evidence**, which is why the result carries an explicit `scope`.
#[derive(Debug, Clone, Default)]
pub struct PodSpend {
    pub present: bool,
    pub reason: String,
    pub checked: String,
    pub records: usize,
    pub closed_events: usize,
    pub usd_total: Option<f64>,
    pub by_month: Vec<(String, f64)>,
    pub by_lab: Vec<(String, f64)>,
    pub pods: Vec<(String, f64)>,
    pub method: String,
    pub scope: String,
}

const POD_METHOD: &str =
    "sum over down events of usd_per_h x duration; pods still running are not charged here";
const POD_SCOPE: &str =
    "local machine state outside the repo (UOR_POD_STATE), not committed evidence";

/// `pod_ledger_spend`.
///
/// Three details decide the number: **a pod's rate is remembered from an earlier row** (a `down` event may
/// carry none, and with nothing to charge the event is skipped rather than charged at zero); the duration
/// falls back `start_ts` -> `uptime_s` -> 0, each clamped at zero; and `usd_total` is the **months** total,
/// not the labs or pods one.
pub fn pod_ledger_spend(state: &Path) -> PodSpend {
    let path = state.join("ledger.jsonl");
    let checked = path.display().to_string();
    if !path.is_file() {
        return PodSpend {
            present: false,
            reason: ABSENT.to_string(),
            checked,
            usd_total: None,
            ..PodSpend::default()
        };
    }
    let records = read_jsonl_tolerant(&path);
    let mut rate_by_pod: Vec<(String, f64)> = Vec::new();
    let mut months: Vec<(String, f64)> = Vec::new();
    let mut labs: Vec<(String, f64)> = Vec::new();
    let mut by_pod: Vec<(String, f64)> = Vec::new();
    let mut events = 0usize;
    for rec in &records {
        let pod = rec.get("pod").map(py_str_of).unwrap_or_default();
        let ts = rec.get("ts").map(py_str_of).unwrap_or_default();
        let rate = rec.get("usd_per_h").and_then(num_of);
        if pod.is_empty() || ts.is_empty() {
            continue;
        }
        let Some(when) = parse_ts(&ts) else { continue };
        if let Some(r) = rate {
            match rate_by_pod.iter_mut().find(|(p, _)| *p == pod) {
                Some((_, existing)) => *existing = r,
                None => rate_by_pod.push((pod.clone(), r)),
            }
        }
        if rec.get("event").and_then(Value::as_str) != Some("down") {
            continue;
        }
        events += 1;
        let start = rec
            .get("start_ts")
            .map(py_str_of)
            .and_then(|s| parse_ts(&s));
        let rate_used =
            rate.or_else(|| rate_by_pod.iter().find(|(p, _)| *p == pod).map(|(_, r)| *r));
        let Some(rate_used) = rate_used else { continue };
        let seconds = match start {
            Some(st) => (when - st).max(0) as f64,
            None => rec
                .get("uptime_s")
                .and_then(num_of)
                .map(|u| u.max(0.0))
                .unwrap_or(0.0),
        };
        let usd = rate_used * seconds / 3600.0;
        let month: String = ts.chars().take(7).collect();
        accumulate(&mut months, &month, usd);
        let lab = group_key(rec, "lab");
        accumulate(&mut labs, &lab, usd);
        accumulate(&mut by_pod, &pod, usd);
    }
    let total: f64 = months.iter().map(|(_, v)| *v).sum();
    months.sort_by(|a, b| a.0.cmp(&b.0));
    let mut by_lab = labs;
    by_lab.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut pods = by_pod;
    pods.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    pods.truncate(10);
    PodSpend {
        present: true,
        reason: String::new(),
        checked,
        records: records.len(),
        closed_events: events,
        usd_total: Some(round_to(total, 4)),
        by_month: months,
        by_lab,
        pods,
        method: POD_METHOD.to_string(),
        scope: POD_SCOPE.to_string(),
    }
}

/// Python's per-step accumulation: `d[k] = round(d.get(k, 0.0) + v, 4)`.
fn accumulate(map: &mut Vec<(String, f64)>, key: &str, value: f64) {
    match map.iter_mut().find(|(k, _)| k == key) {
        Some((_, total)) => *total = round_to(*total + value, 4),
        None => map.push((key.to_string(), round_to(value, 4))),
    }
}

// ------------------------------------------------------------------------- resource ledger

/// `resource_ledger`: the per-experiment ledger chain the repo keeps.
#[derive(Debug, Clone, Default)]
pub struct ResourceLedger {
    pub present: bool,
    pub reason: String,
    pub checked: String,
    pub count: usize,
    pub chain: Chain,
    pub latest_seconds: Option<f64>,
    pub latest_path: String,
    pub latest_field: String,
    pub latest_hours: Option<f64>,
    /// `(path, usd)` - only entries whose cost parses, so a `true` or a `"TBD"` never becomes a row.
    pub gpu_cost_records: Vec<(String, Value)>,
}

/// The path of `child` relative to `root`, as Python's `_rel` writes it.
fn rel(child: &Path, root: &Path) -> String {
    child
        .strip_prefix(root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| child.display().to_string())
}

pub fn resource_ledger(root: &Path) -> ResourceLedger {
    if !root.is_dir() {
        return ResourceLedger {
            present: false,
            reason: ABSENT.to_string(),
            checked: root.display().to_string(),
            ..ResourceLedger::default()
        };
    }
    // `root.glob("*/resource-ledger.json")` - exactly one level deep, then sorted.
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for e in entries.flatten() {
            let p = e.path().join("resource-ledger.json");
            if p.is_file() {
                files.push(p);
            }
        }
    }
    files.sort();
    let mut entries: Vec<LedgerEntry> = Vec::new();
    for f in &files {
        let Ok(text) = std::fs::read_to_string(f) else {
            continue;
        };
        let Ok(data) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        if !data.is_object() {
            continue;
        }
        let s = |k: &str| {
            data.get(k)
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        };
        // `status or orchestration.status`: an EMPTY string falls through, which is how the two schemas
        // in this repo coexist.
        let status = {
            let a = s("status");
            if !a.is_empty() {
                a
            } else {
                data.get("orchestration")
                    .and_then(|o| o.get("status"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string()
            }
        };
        let updated_utc = {
            let a = s("updated_utc");
            if !a.is_empty() {
                a
            } else {
                let b = s("started_utc");
                if !b.is_empty() {
                    b
                } else {
                    s("complete_accounting_start_utc")
                }
            }
        };
        let pick = |keys: &[&str]| -> Value {
            for k in keys {
                if let Some(v) = data.get(*k) {
                    return v.clone();
                }
            }
            Value::Null
        };
        entries.push(LedgerEntry {
            path: rel(f, root),
            schema: s("schema"),
            status,
            updated_utc,
            increment_seconds: pick(&["increment_seconds", "increment"]),
            projection_seconds: pick(&[
                "complete_projection_seconds",
                "increment_seconds",
                "increment",
            ]),
            cumulative_projected_seconds: data
                .get("cumulative_projected_seconds")
                .cloned()
                .unwrap_or(Value::Null),
            lab_cumulative_projection_seconds: data
                .get("lab_cumulative_projection_seconds")
                .cloned()
                .unwrap_or(Value::Null),
            previous_ledger: s("previous_ledger"),
            new_pods: data.get("new_pods").cloned().unwrap_or(Value::Null),
            approx_gpu_cost_usd: data
                .get("approximate_gpu_cost_usd")
                .cloned()
                .unwrap_or(Value::Null),
        });
    }
    let chain = chain(&entries);
    let (latest_seconds, latest_path, latest_field, latest_hours) = latest_cumulative(&entries);
    let gpu_cost_records: Vec<(String, Value)> = entries
        .iter()
        .filter(|e| num_of(&e.approx_gpu_cost_usd).is_some())
        .map(|e| (e.path.clone(), e.approx_gpu_cost_usd.clone()))
        .collect();
    ResourceLedger {
        present: !entries.is_empty(),
        reason: if entries.is_empty() {
            ABSENT.to_string()
        } else {
            String::new()
        },
        checked: format!("{}/**/resource-ledger.json", root.display()),
        count: entries.len(),
        chain,
        latest_seconds,
        latest_path,
        latest_field,
        latest_hours,
        gpu_cost_records,
    }
}

// ----------------------------------------------------------------------------- receipts

/// One receipt that carries a cost - an estimate a session recorded, not a charge.
#[derive(Debug, Clone, Default)]
pub struct ReceiptCost {
    pub ts: Value,
    pub month: String,
    pub kind: Value,
    pub milestone: String,
    pub field: String,
    pub usd: f64,
    pub file: Value,
}

/// One receipt that carries only a rate. **A rate is not a charge.**
#[derive(Debug, Clone, Default)]
pub struct ReceiptRate {
    pub ts: Value,
    pub month: String,
    pub milestone: String,
    pub field: String,
    pub usd_per_h: f64,
}

/// A receipt field with Python's `get(key, "")` default - **not `Null`**, because `str(None)` is the
/// four-character string `"None"` and would make a missing timestamp into a month called `None`.
fn field_or_empty(rec: &Value, key: &str) -> Value {
    rec.get(key)
        .cloned()
        .unwrap_or(Value::String(String::new()))
}

/// `_receipt_costs`: split receipts into cost entries and rate-only entries.
///
/// The first `COST_KEYS` name that **parses** wins; only when none of them parse does it look at
/// `RATE_KEYS`, and it stops at the first rate key present. The docstring is blunt about why: counting
/// `usd_per_h` as money would overstate the ledger.
pub fn receipt_costs(recs: &[Value]) -> (Vec<ReceiptCost>, Vec<ReceiptRate>) {
    let mut costs = Vec::new();
    let mut rates = Vec::new();
    for rec in recs {
        let ts = field_or_empty(rec, "ts");
        let month = {
            let s = py_str_of(&ts);
            if s.is_empty() {
                "?".to_string()
            } else {
                s.chars().take(7).collect()
            }
        };
        let milestone = rec
            .get("milestone")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let mut amount: Option<f64> = None;
        let mut key_used = String::new();
        for key in COST_KEYS {
            if let Some(v) = rec.get(key) {
                amount = num_of(v);
                key_used = key.to_string();
                if amount.is_some() {
                    break;
                }
            }
        }
        if let Some(a) = amount {
            costs.push(ReceiptCost {
                ts,
                month,
                kind: field_or_empty(rec, "kind"),
                milestone,
                field: key_used,
                usd: round_to(a, 4),
                file: field_or_empty(rec, "_file"),
            });
            continue;
        }
        for key in RATE_KEYS {
            if let Some(v) = rec.get(key) {
                if let Some(rate) = num_of(v) {
                    rates.push(ReceiptRate {
                        ts: ts.clone(),
                        month: month.clone(),
                        milestone,
                        field: key.to_string(),
                        usd_per_h: rate,
                    });
                }
                break;
            }
        }
    }
    (costs, rates)
}

// ------------------------------------------------------------------------- model time

/// `model_time_ledger`: the ledger the task names explicitly - **ABSENT rather than guessed** when missing.
#[derive(Debug, Clone, Default)]
pub struct ModelTimeLedger {
    pub present: bool,
    pub reason: String,
    pub checked: String,
    pub data: Value,
}

pub fn model_time_ledger(root: &Path) -> ModelTimeLedger {
    let path = root.join(MODEL_TIME_LEDGER);
    let missing = ModelTimeLedger {
        present: false,
        reason: ABSENT.to_string(),
        checked: MODEL_TIME_LEDGER.to_string(),
        data: Value::Null,
    };
    if !path.exists() {
        return missing;
    }
    match std::fs::read_to_string(&path) {
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(data) => ModelTimeLedger {
                present: true,
                reason: String::new(),
                checked: MODEL_TIME_LEDGER.to_string(),
                data,
            },
            Err(_) => ModelTimeLedger {
                reason: "UNREADABLE (JSONDecodeError)".to_string(),
                ..missing
            },
        },
        Err(_) => ModelTimeLedger {
            reason: "UNREADABLE (OSError)".to_string(),
            ..missing
        },
    }
}

// ------------------------------------------------------------------------------- collect

/// The receipts section: what the ledger shards hold, split into costs and rates.
#[derive(Debug, Clone, Default)]
pub struct Receipts {
    pub files: Vec<String>,
    pub records: usize,
    pub cost_records: usize,
    pub rate_only_records: usize,
    pub usd_total: f64,
    pub by_month: Vec<(String, f64)>,
    pub by_milestone: Vec<(String, f64)>,
    pub entries: Vec<ReceiptCost>,
    pub rate_entries: Vec<ReceiptRate>,
}

/// Everything `budget show` prints, as data - **so the view and the rules cannot disagree.**
#[derive(Debug, Clone, Default)]
pub struct BudgetData {
    pub receipts: Receipts,
    pub model_time: ModelTimeLedger,
    pub resource: ResourceLedger,
    pub pod: PodSpend,
    pub disk: Value,
}

/// `_sum_by` over cost rows: `str(row.get(key) or "(unset)")`, per-step rounding, sorted keys.
///
/// A milestone of `""` therefore groups under `(unset)` with the missing ones, while a month never does -
/// `receipt_costs` has already turned an empty month into `?`.
fn sum_by_field(costs: &[ReceiptCost], by_milestone: bool) -> Vec<(String, f64)> {
    let mut out: Vec<(String, f64)> = Vec::new();
    for c in costs {
        let raw = if by_milestone {
            c.milestone.clone()
        } else {
            c.month.clone()
        };
        let key = if raw.is_empty() {
            "(unset)".to_string()
        } else {
            raw
        };
        accumulate(&mut out, &key, c.usd);
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// `collect`: read the ledger shards, then the four other sections.
pub fn collect(repo: &Path, ledger_dir: &Path) -> BudgetData {
    let mut recs: Vec<Value> = Vec::new();
    let mut files: Vec<String> = Vec::new();
    if ledger_dir.is_dir() {
        let mut shards: Vec<std::path::PathBuf> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(ledger_dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.extension().is_some_and(|x| x == "jsonl") {
                    shards.push(p);
                }
            }
        }
        shards.sort();
        for f in &shards {
            files.push(rel(f, repo));
            for mut rec in read_jsonl_tolerant(f) {
                if !rec.is_object() {
                    continue;
                }
                // `setdefault("_file", f.name)`: a record that names its own file keeps it.
                if rec.get("_file").is_none() {
                    let name = f
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if let Some(map) = rec.as_object_mut() {
                        map.insert("_file".to_string(), Value::String(name));
                    }
                }
                recs.push(rec);
            }
        }
    }
    let (costs, rates) = receipt_costs(&recs);
    let usd_total: f64 = costs.iter().map(|c| c.usd).sum();
    let receipts = Receipts {
        files,
        records: recs.len(),
        cost_records: costs.len(),
        rate_only_records: rates.len(),
        usd_total: round_to(usd_total, 4),
        by_month: sum_by_field(&costs, false),
        by_milestone: sum_by_field(&costs, true),
        entries: costs,
        rate_entries: rates,
    };
    BudgetData {
        receipts,
        model_time: model_time_ledger(repo),
        resource: resource_ledger(&repo.join("docs").join("labs")),
        pod: pod_ledger_spend(&pod_state_dir()),
        disk: disk_free(repo),
    }
}

// ---------------------------------------------------------------------------------- show

/// Python's `str()` for a float: `12.0` stays `12.0`.
///
/// Rust's `Display` prints an integral float as `12`, while Python prints `12.0` - and the hours in this
/// output are floats from `round(x / 3600, 2)`, so the difference is visible on every whole hour.
pub fn py_float(v: f64) -> String {
    let s = format!("{v}");
    if s.contains('.') || s.contains('e') || s.contains("inf") || s.contains("NaN") {
        s
    } else {
        format!("{s}.0")
    }
}

/// `budget show`, as Python prints it.
pub fn show_text(data: &BudgetData) -> String {
    let mut out = String::new();
    let mut emit = |line: String| {
        out.push_str(&line);
        out.push('\n');
    };
    let rec = &data.receipts;
    emit("cumulative cost".to_string());
    if rec.files.is_empty() {
        emit(format!(
            "  receipts ledger: {ABSENT} (docs/harness/ledger/*.jsonl)"
        ));
    } else {
        emit(format!(
            "  receipts: {} record(s) in {} file(s); {} carry a cost field",
            rec.records,
            rec.files.len(),
            rec.cost_records
        ));
        if rec.cost_records > 0 {
            emit(format!(
                "  receipts cost fields total: ${:.2}",
                rec.usd_total
            ));
            emit(format!("    by month: {}", fmt_map(&rec.by_month)));
            emit(format!("    by milestone: {}", fmt_map(&rec.by_milestone)));
        } else {
            emit(format!(
                "  receipts cost fields: none recorded (by month: {ABSENT}; by milestone: {ABSENT})"
            ));
        }
        if rec.rate_only_records > 0 {
            emit(format!(
                "  note: {} receipt(s) carry a $/h rate, not a charge - not summed",
                rec.rate_only_records
            ));
        }
    }
    let mt = &data.model_time;
    if mt.present {
        emit(format!(
            "  tracked model-time ledger: {} -> present",
            mt.checked
        ));
        if mt.data.is_object() {
            for key in ["cumulative_ms", "limit_ms", "updated_utc"] {
                if let Some(v) = mt.data.get(key) {
                    emit(format!("    {key}: {}", py_str_of(v)));
                }
            }
        }
    } else {
        emit(format!(
            "  tracked model-time ledger: {} at {} (the path the harness was told to check; no number is invented)",
            mt.reason, mt.checked
        ));
    }
    let rl = &data.resource;
    if rl.present {
        let chain = &rl.chain;
        emit(format!(
            "  repo cumulative ledger: {} resource-ledger.json file(s) under {}",
            rl.count, rl.checked
        ));
        emit(format!(
            "    newest head of the pointer chain: {} ({} link(s) walked)",
            chain.head, chain.links
        ));
        if let (Some(hours), Some(seconds)) = (
            chain.cumulative_projected_hours,
            chain.cumulative_projected_seconds,
        ) {
            emit(format!(
                "    head projection: {} h ({:.0} s, the newest ledger's own figure)",
                py_float(hours),
                seconds
            ));
        }
        if chain.links == 1 {
            emit(
                "      the walk stopped at the head: it names no previous_ledger (a new accounting boundary), so the chain does not reach back to the first ledger"
                    .to_string(),
            );
        }
        if let (Some(seconds), Some(hours)) = (rl.latest_seconds, rl.latest_hours) {
            emit(format!(
                "    latest stated lab cumulative: {} h ({:.0} s) in {} [{}]",
                py_float(hours),
                seconds,
                rl.latest_path,
                rl.latest_field
            ));
        }
        for (path, usd) in &rl.gpu_cost_records {
            emit(format!(
                "    stated GPU cost: ${:.2} in {path}",
                num_of(usd).unwrap_or(0.0)
            ));
        }
    } else {
        emit(format!(
            "  repo cumulative ledger: {ABSENT} under {}",
            rl.checked
        ));
    }
    let pl = &data.pod;
    if pl.present {
        emit(format!(
            "  pod spend (actual, local): ${:.2} over {} closed pod(s)",
            pl.usd_total.unwrap_or(0.0),
            pl.closed_events
        ));
        emit(format!("    by month: {}", fmt_map(&pl.by_month)));
        emit(format!("    by lab: {}", fmt_map(&pl.by_lab)));
        emit(format!("    source: {} - {}", pl.checked, pl.scope));
    } else {
        emit(format!("  pod spend: {} at {}", pl.reason, pl.checked));
    }
    let d = &data.disk;
    if d.get("ok").and_then(Value::as_bool) != Some(true) {
        emit(format!(
            "  disk: UNAVAILABLE ({})",
            d.get("error").map(py_str_of).unwrap_or_default()
        ));
    } else {
        let low = d
            .get("band_gb")
            .and_then(|b| b.get(0))
            .map(py_str_of)
            .unwrap_or_default();
        let high = d
            .get("band_gb")
            .and_then(|b| b.get(1))
            .map(py_str_of)
            .unwrap_or_default();
        let below = d.get("below_floor").and_then(Value::as_bool) == Some(true);
        let in_band = d.get("in_band").and_then(Value::as_bool) == Some(true);
        let state = if in_band {
            "in band"
        } else if below {
            "BELOW FLOOR"
        } else {
            "above band"
        };
        emit(format!(
            "  disk free: {:.2} GB {state} (owner band {low}-{high} GB; retained-evidence margin {} MiB)",
            d.get("free_gb").and_then(Value::as_f64).unwrap_or(0.0),
            d.get("margin_mib").map(py_str_of).unwrap_or_default()
        ));
        if below {
            emit(format!("    clear it: {HYGIENE_COMMAND}"));
        }
    }
    emit(format!(
        "  owner caps: <= {POD_CAP} running pods, <= ${RATE_CAP_USD_PER_H:.2}/h (all labs, all sessions)"
    ));
    out
}

/// A `[(key, value)]` map as a JSON object, in the order the Rust side already sorted it.
fn map_ordered(pairs: &[(String, f64)]) -> Ordered {
    Ordered::Obj(
        pairs
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    serde_json::Number::from_f64(*v)
                        .map(Ordered::Num)
                        .unwrap_or(Ordered::Null),
                )
            })
            .collect(),
    )
}

fn ord(v: &Value) -> Ordered {
    match v {
        Value::Null => Ordered::Null,
        Value::Bool(b) => Ordered::Bool(*b),
        Value::Number(n) => Ordered::Num(n.clone()),
        Value::String(s) => Ordered::Str(s.clone()),
        Value::Array(items) => Ordered::Arr(items.iter().map(ord).collect()),
        Value::Object(map) => Ordered::Obj(map.iter().map(|(k, v)| (k.clone(), ord(v))).collect()),
    }
}

fn s(v: &str) -> Ordered {
    Ordered::Str(v.to_string())
}

fn num(v: f64) -> Ordered {
    serde_json::Number::from_f64(v)
        .map(Ordered::Num)
        .unwrap_or(Ordered::Null)
}

fn i(v: i64) -> Ordered {
    Ordered::Num(serde_json::Number::from(v))
}

/// `budget show --json`.
///
/// **The key order is Python's**, so this is built from `Ordered` rather than `serde_json::Value` - a
/// `BTreeMap` would sort every section and the comparison against Python would fail on the first line.
/// `ensure_ascii=False` is the flag this call passes, unlike most of the harness's `--json` output.
///
/// Two shape details matter: **the `present` and absent forms of `pod_ledger` carry DIFFERENT keys** (the
/// absent one has `reason` and no `records`/`method`/`scope`), and **`band_gb` prints as integers** `[30,
/// 70]`, because Python's band is a tuple of ints.
pub fn show_json(data: &BudgetData) -> String {
    let rec = &data.receipts;
    let receipts = Ordered::Obj(vec![
        (
            "files".to_string(),
            Ordered::Arr(rec.files.iter().map(|f| s(f)).collect()),
        ),
        ("records".to_string(), i(rec.records as i64)),
        ("cost_records".to_string(), i(rec.cost_records as i64)),
        (
            "rate_only_records".to_string(),
            i(rec.rate_only_records as i64),
        ),
        // Python's `round(sum(...), 4)` is the **int** 0 when there are no cost records (`sum([])` is
        // int 0) and a float otherwise, and the JSON shows the difference: `0` versus `0.0`.
        (
            "usd_total".to_string(),
            if rec.cost_records == 0 {
                i(0)
            } else {
                num(rec.usd_total)
            },
        ),
        ("by_month".to_string(), map_ordered(&rec.by_month)),
        ("by_milestone".to_string(), map_ordered(&rec.by_milestone)),
        (
            "entries".to_string(),
            Ordered::Arr(
                rec.entries
                    .iter()
                    .map(|c| {
                        Ordered::Obj(vec![
                            ("ts".to_string(), ord(&c.ts)),
                            ("month".to_string(), s(&c.month)),
                            ("kind".to_string(), ord(&c.kind)),
                            ("milestone".to_string(), s(&c.milestone)),
                            ("field".to_string(), s(&c.field)),
                            ("usd".to_string(), num(c.usd)),
                            ("file".to_string(), ord(&c.file)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "rate_entries".to_string(),
            Ordered::Arr(
                rec.rate_entries
                    .iter()
                    .map(|r| {
                        Ordered::Obj(vec![
                            ("ts".to_string(), ord(&r.ts)),
                            ("month".to_string(), s(&r.month)),
                            ("milestone".to_string(), s(&r.milestone)),
                            ("field".to_string(), s(&r.field)),
                            ("usd_per_h".to_string(), num(r.usd_per_h)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "note".to_string(),
            s("receipts record orchestration; a cost field here is an estimate recorded by a session, not a charge"),
        ),
    ]);
    let mt = &data.model_time;
    let model_time = if mt.present {
        Ordered::Obj(vec![
            ("present".to_string(), Ordered::Bool(true)),
            ("checked".to_string(), s(&mt.checked)),
            ("data".to_string(), ord(&mt.data)),
        ])
    } else {
        Ordered::Obj(vec![
            ("present".to_string(), Ordered::Bool(false)),
            ("reason".to_string(), s(&mt.reason)),
            ("checked".to_string(), s(&mt.checked)),
        ])
    };
    let rl = &data.resource;
    let chain = &rl.chain;
    let latest = Ordered::Obj({
        let mut pairs = vec![
            (
                "seconds".to_string(),
                rl.latest_seconds.map(num).unwrap_or(Ordered::Null),
            ),
            ("path".to_string(), s(&rl.latest_path)),
            ("field".to_string(), s(&rl.latest_field)),
        ];
        if let Some(h) = rl.latest_hours {
            pairs.push(("hours".to_string(), num(h)));
        }
        pairs
    });
    let resource = Ordered::Obj(vec![
        ("present".to_string(), Ordered::Bool(rl.present)),
        ("reason".to_string(), s(&rl.reason)),
        ("checked".to_string(), s(&rl.checked)),
        ("count".to_string(), i(rl.count as i64)),
        (
            "chain".to_string(),
            Ordered::Obj(vec![
                ("head".to_string(), s(&chain.head)),
                ("links".to_string(), i(chain.links as i64)),
                ("files".to_string(), i(chain.files as i64)),
                (
                    "increment_seconds".to_string(),
                    num(chain.increment_seconds),
                ),
                ("increment_hours".to_string(), num(chain.increment_hours)),
                (
                    "cumulative_projected_seconds".to_string(),
                    chain
                        .cumulative_projected_seconds
                        .map(num)
                        .unwrap_or(Ordered::Null),
                ),
                (
                    "cumulative_projected_hours".to_string(),
                    chain
                        .cumulative_projected_hours
                        .map(num)
                        .unwrap_or(Ordered::Null),
                ),
                ("note".to_string(), s(&chain.note)),
            ]),
        ),
        ("latest_lab_cumulative_seconds".to_string(), latest),
        (
            "gpu_cost_records".to_string(),
            Ordered::Arr(
                rl.gpu_cost_records
                    .iter()
                    .map(|(path, usd)| {
                        Ordered::Obj(vec![
                            ("path".to_string(), s(path)),
                            ("usd".to_string(), ord(usd)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ]);
    let pl = &data.pod;
    let pod = if pl.present {
        Ordered::Obj(vec![
            ("present".to_string(), Ordered::Bool(true)),
            ("checked".to_string(), s(&pl.checked)),
            ("records".to_string(), i(pl.records as i64)),
            ("closed_events".to_string(), i(pl.closed_events as i64)),
            (
                "usd_total".to_string(),
                pl.usd_total.map(num).unwrap_or(Ordered::Null),
            ),
            ("by_month".to_string(), map_ordered(&pl.by_month)),
            ("by_lab".to_string(), map_ordered(&pl.by_lab)),
            ("pods".to_string(), map_ordered(&pl.pods)),
            ("method".to_string(), s(&pl.method)),
            ("scope".to_string(), s(&pl.scope)),
        ])
    } else {
        Ordered::Obj(vec![
            ("present".to_string(), Ordered::Bool(false)),
            ("reason".to_string(), s(&pl.reason)),
            ("checked".to_string(), s(&pl.checked)),
            ("usd_total".to_string(), Ordered::Null),
        ])
    };
    let d = &data.disk;
    let disk = if d.get("ok").and_then(Value::as_bool) == Some(true) {
        Ordered::Obj(vec![
            ("ok".to_string(), Ordered::Bool(true)),
            (
                "path".to_string(),
                s(&d.get("path").map(py_str_of).unwrap_or_default()),
            ),
            (
                "free_bytes".to_string(),
                i(d.get("free_bytes").and_then(Value::as_i64).unwrap_or(0)),
            ),
            (
                "total_bytes".to_string(),
                i(d.get("total_bytes").and_then(Value::as_i64).unwrap_or(0)),
            ),
            (
                "free_gb".to_string(),
                num(d.get("free_gb").and_then(Value::as_f64).unwrap_or(0.0)),
            ),
            // The owner's band is a tuple of INTEGERS in Python, so it prints [30, 70], not [30.0, 70.0].
            (
                "band_gb".to_string(),
                Ordered::Arr(vec![i(DISK_BAND_GB.0), i(DISK_BAND_GB.1)]),
            ),
            (
                "in_band".to_string(),
                Ordered::Bool(d.get("in_band").and_then(Value::as_bool) == Some(true)),
            ),
            (
                "below_floor".to_string(),
                Ordered::Bool(d.get("below_floor").and_then(Value::as_bool) == Some(true)),
            ),
            (
                "above_band".to_string(),
                Ordered::Bool(d.get("above_band").and_then(Value::as_bool) == Some(true)),
            ),
            (
                "margin_mib".to_string(),
                i(d.get("margin_mib").and_then(Value::as_i64).unwrap_or(0)),
            ),
            (
                "hygiene".to_string(),
                s(&d.get("hygiene").map(py_str_of).unwrap_or_default()),
            ),
        ])
    } else {
        Ordered::Obj(vec![
            ("ok".to_string(), Ordered::Bool(false)),
            (
                "error".to_string(),
                s(&d.get("error").map(py_str_of).unwrap_or_default()),
            ),
            (
                "path".to_string(),
                s(&d.get("path").map(py_str_of).unwrap_or_default()),
            ),
        ])
    };
    let caps = Ordered::Obj(vec![
        ("pods".to_string(), i(POD_CAP)),
        ("usd_per_h".to_string(), num(RATE_CAP_USD_PER_H)),
    ]);
    let doc = Ordered::Obj(vec![
        ("receipts".to_string(), receipts),
        ("model_time_ledger".to_string(), model_time),
        ("resource_ledger".to_string(), resource),
        ("pod_ledger".to_string(), pod),
        ("disk".to_string(), disk),
        ("caps".to_string(), caps),
    ]);
    // `ensure_ascii=False` here, which is the flag this call passes.
    doc.to_pretty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// **A bool is not a number.** Python's `isinstance(True, int)` is True, so `_num` refuses bools before
    /// the numeric branch; without that, a JSON `true` in a cost field is one dollar.
    #[test]
    fn a_bool_is_never_money() {
        assert_eq!(num_of(&json!(true)), None);
        assert_eq!(num_of(&json!(false)), None);
        assert_eq!(num_of(&json!(2)), Some(2.0));
        // A number out of a string, as the regex does: commas first, then the first number.
        assert_eq!(num_of(&json!("$1,234.50")), Some(1234.5));
        assert_eq!(num_of(&json!("TBD")), None);
    }

    /// **Python rounds the decimal, ties to even.** `(x * 10^p).round_ties_even() / 10^p` multiplies first
    /// and was one tie off on this ledger's pod total (`371.9625` against Python's `371.9626`).
    #[test]
    fn rounding_is_pythons_decimal_rounding() {
        assert_eq!(round_to(371.96255, 4), 371.9626);
        assert_eq!(round_to(0.5, 0), 0.0);
        assert_eq!(round_to(1.5, 0), 2.0);
        assert_eq!(round_to(2.5, 0), 2.0);
        assert_eq!(round_to(1.23456, 4), 1.2346);
    }

    /// The tolerant reader: a corrupt line costs that line only, and only objects survive.
    #[test]
    fn a_corrupt_line_costs_that_line_only() {
        let dir = std::env::temp_dir().join(format!("harness-budget-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch");
        let p = dir.join("l.jsonl");
        std::fs::write(&p, "{\"a\": 1}\n{not json\n\n[1, 2]\n{\"b\": 2}\n").expect("write");
        let recs = read_jsonl_tolerant(&p);
        assert_eq!(
            recs.len(),
            2,
            "the two objects, not the array or the broken line"
        );
        assert_eq!(recs[0].get("a"), Some(&json!(1)));
        assert!(read_jsonl_tolerant(&dir.join("missing.jsonl")).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `str(row.get(key) or "(unset)")` - **a falsy value groups with the missing ones**, so a zero does
    /// not get a group of its own.
    #[test]
    fn a_falsy_group_key_is_unset() {
        assert_eq!(group_key(&json!({"lab": ""}), "lab"), "(unset)");
        assert_eq!(group_key(&json!({"lab": 0}), "lab"), "(unset)");
        assert_eq!(group_key(&json!({}), "lab"), "(unset)");
        assert_eq!(group_key(&json!({"lab": "kimi"}), "lab"), "kimi");
    }

    /// **A rate is not a charge**: `usd_per_h` is reported separately, never summed as spend.
    #[test]
    fn a_rate_is_split_out_and_never_a_cost() {
        let recs = vec![
            json!({"ts": "2026-10-14T09:00:00Z", "cost_usd": 1.5, "milestone": "m2"}),
            json!({"ts": "2026-10-14T10:00:00Z", "usd_per_h": 2.5}),
            json!({"ts": "2026-10-14T11:00:00Z"}),
        ];
        let (costs, rates) = receipt_costs(&recs);
        assert_eq!(costs.len(), 1);
        assert_eq!(costs[0].usd, 1.5);
        assert_eq!(costs[0].field, "cost_usd");
        assert_eq!(costs[0].month, "2026-10");
        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].usd_per_h, 2.5);
        assert_eq!(rates[0].field, "usd_per_h");
    }

    /// A missing timestamp is `?`, **not the string `None`** - which is what `str(None)` would produce if
    /// the field defaulted to null instead of `""`.
    #[test]
    fn a_missing_month_is_a_question_mark() {
        let (costs, _) = receipt_costs(&[json!({"usd": 1})]);
        assert_eq!(costs[0].month, "?");
    }

    /// `_chain`: a pointer matches on a TAIL, the head is chosen by STRING comparison, and the cumulative
    /// comes from the head only - the walked increments are not a new total.
    #[test]
    fn the_chain_walks_tails_and_takes_the_head_cumulative() {
        let entry = |path: &str, prev: &str, cum: Option<i64>, updated: &str| LedgerEntry {
            path: path.to_string(),
            previous_ledger: prev.to_string(),
            cumulative_projected_seconds: cum.map(|c| json!(c)).unwrap_or(Value::Null),
            updated_utc: updated.to_string(),
            increment_seconds: json!(3600),
            ..LedgerEntry::default()
        };
        let entries = vec![
            entry(
                "a/resource-ledger.json",
                "",
                Some(3600),
                "2026-10-01T00:00:00Z",
            ),
            // A pointer written as an ABSOLUTE path: the tail match is what resolves it.
            entry(
                "b/resource-ledger.json",
                "/workspace/uor-r4/a/resource-ledger.json",
                Some(7200),
                "2026-10-02T00:00:00Z",
            ),
        ];
        let c = chain(&entries);
        assert_eq!(c.head, "b/resource-ledger.json", "the newest updated_utc");
        assert_eq!(c.links, 2);
        assert_eq!(c.files, 2);
        assert_eq!(c.increment_seconds, 7200.0);
        assert_eq!(c.increment_hours, 2.0);
        // The head's own figure, NOT the sum of the walk.
        assert_eq!(c.cumulative_projected_seconds, Some(7200.0));
    }

    #[test]
    fn an_empty_chain_reports_absence_rather_than_zero_hours() {
        let c = chain(&[]);
        assert_eq!(c.head, "");
        assert_eq!(c.links, 0);
        assert_eq!(c.cumulative_projected_seconds, None);
        assert_eq!(c.note, "no resource-ledger.json found");
    }

    /// The band is printed from INTEGERS, which is how Python's tuple of ints reads.
    #[test]
    fn the_disk_band_is_integers() {
        assert_eq!(DISK_BAND_GB, (30, 70));
        let d = disk_free(Path::new("."));
        assert_eq!(d.get("ok").and_then(Value::as_bool), Some(true), "{d}");
        assert_eq!(d.get("band_gb"), Some(&json!([30, 70])));
        assert_eq!(d.get("margin_mib"), Some(&json!(128)));
        assert_eq!(d.get("hygiene"), Some(&json!(HYGIENE_COMMAND)));
    }

    /// `_parse_ts`: `Z` and an offset are both handled, and a naive timestamp is assumed UTC, as Python
    /// does - the durations here were computed with that assumption.
    #[test]
    fn timestamps_parse_like_fromisoformat() {
        assert_eq!(parse_ts("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_ts("1970-01-01T00:00:00+00:00"), Some(0));
        assert_eq!(parse_ts("1970-01-01T01:00:00+01:00"), Some(0));
        assert_eq!(parse_ts("1970-01-01T00:00:00"), Some(0), "naive is UTC");
        assert_eq!(parse_ts("1970-01-01T00:00:01Z"), Some(1));
        assert_eq!(parse_ts(""), None);
        assert_eq!(parse_ts("not a time"), None);
    }
}
