//! `loop drive`: the supervisor for the five-lab loop (loop PR 10).
//!
//! # What it does
//!
//! One invocation runs one or more **fresh sessions**. Before each session the loop's own
//! [`loopnext::run`] decides go, wait (sleep `--poll` seconds and ask again) or stop: the kill switch (`local/loop/STOP` or the `loop:pause`
//! label on #2028), held work, an empty queue and the rest of the loop page's stops are **not
//! re-implemented here**. The headless client is launched from the caller's explicit argv with one
//! `{prompt}` element, in its own process group, and only that recorded pid/process group is ever
//! signalled. The record (pid, pgid, client, model, prompt hash, start time and the process start
//! identity) lives under `local/loop/drive/<lab>/`; the supervisor waits on the tracked child, so a
//! client's after-stop summary process is never mistaken for the lab session.
//!
//! # The soft 2-hour check-in (D53)
//!
//! At [`CHECK_IN_SECONDS`] the supervisor **checks the live session**: commits, gate-run activity and
//! working-tree changes are real progress (continue with a note); nothing is stalled (close it and
//! card it); a second consecutive stalled session is spiraled. A leased GPU run is exempt: it is
//! never closed while a lease covers it, and [`watch_lease_until_renew_fails`] renews it from a
//! detached watcher that outlives the session.
//!
//! # Why the process layer is thin
//!
//! The planner ([`classify`], [`check_in`], [`compose_prompt`], [`launch_argv`]) is pure and tested
//! with a fake clock; the process actions are [`start_session`], [`session_alive`],
//! [`terminate_session`] and [`run_sessions`], so the risky part is small and inspectable. Every
//! signal is gated on an identity match with the recorded process, never on a pattern.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// The soft task limit of D53: two hours on a session before the check-in runs.
pub const CHECK_IN_SECONDS: u64 = 7_200;
/// Where session records live, relative to the repository root the supervisor was given.
pub const DRIVE_DIR: &str = "local/loop/drive";
/// How many sessions one invocation may start by default.
pub const DEFAULT_MAX_SESSIONS: usize = 1;
/// The `GIT_*` variables a git child must not inherit.
/// Remove the `GIT_*` variables that **locate a repository or override config** (including
/// `GIT_CONFIG_PARAMETERS`), so a child cannot inherit another repository; the transport/auth
/// variables such as `GIT_SSH_COMMAND` and `GIT_ASKPASS` stay, or a push that needs them breaks.
const GIT_STRIP: [&str; 8] = [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_COMMON_DIR",
    "GIT_NAMESPACE",
    "GIT_PREFIX",
];

fn strip_git_env(command: &mut Command) {
    for name in GIT_STRIP {
        command.env_remove(name);
    }
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_CONFIG") {
            command.env_remove(key);
        }
    }
}

/// Why a drive command refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriveError {
    /// The launch argv does not carry exactly one `{prompt}` placeholder, or one in argv[0].
    PromptPlaceholder(usize),
    /// The client or model flag is missing.
    MissingFlag(&'static str),
    /// The client name is not a registered adapter under `docs/harness/onboarding/`.
    UnknownClient(String),
    /// The session could not be launched or its record written.
    Io(String),
    /// A safety guard refused an action.
    Refused(String),
}

impl fmt::Display for DriveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PromptPlaceholder(found) => write!(
                formatter,
                "the launch command must contain exactly one {{prompt}} placeholder, found {found}"
            ),
            Self::MissingFlag(flag) => write!(formatter, "loop drive needs {flag}"),
            Self::UnknownClient(client) => write!(
                formatter,
                "no client adapter docs/harness/onboarding/{client}.json"
            ),
            Self::Io(message) | Self::Refused(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for DriveError {}

type Result<T> = std::result::Result<T, DriveError>;

/// One observation of the repository a check-in classifies progress from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    /// `git rev-parse HEAD`.
    pub head: String,
    /// `git rev-list --count HEAD`.
    pub commits: u64,
    /// Lines in the lab's gate ledger; gate runs are activity even without a commit.
    pub gate_runs: u64,
    /// `git status --porcelain` entries: edits in progress are activity too.
    pub changed_paths: usize,
}

impl Observation {
    /// The observation from `root` and `lab`; read-only, no network.
    pub fn take(root: &Path, lab: &str) -> Observation {
        let git = |args: &[&str]| -> String {
            let mut command = Command::new("git");
            command.args(args).current_dir(root);
            strip_git_env(&mut command);
            command
                .output()
                .ok()
                .filter(|out| out.status.success())
                .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
                .unwrap_or_default()
        };
        let head = git(&["rev-parse", "HEAD"]);
        let commits = git(&["rev-list", "--count", "HEAD"]).parse().unwrap_or(0);
        let changed = git(&["status", "--porcelain"]);
        // The supervisor's own drive directory (session logs, records, cards) is ignored for
        // progress: it changes on every poll and says nothing about the work.
        let changed_paths = changed
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter(|line| !line.contains("local/loop/drive/"))
            .count();
        let ledger = root
            .join("local")
            .join(crate::state::slug(lab))
            .join("gates.jsonl");
        let gate_runs = std::fs::read_to_string(ledger)
            .map(|text| text.lines().count() as u64)
            .unwrap_or(0);
        Observation {
            head,
            commits,
            gate_runs,
            changed_paths,
        }
    }
}
/// What the repository says about progress between two observations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Progress {
    /// Real progress: commits, gate runs or working-tree edits.
    Progressing(&'static str),
    /// No movement at all.
    Stalled,
}

/// Classify progress. The classification is deliberately conservative: **any** of a new commit,
/// another gate run or a changed working tree counts as progress, because D53 says real progress
/// continues and only a session with nothing to show is stalled.
pub fn classify(previous: &Observation, current: &Observation) -> Progress {
    if !current.head.is_empty() && current.head != previous.head {
        return Progress::Progressing("new HEAD");
    }
    if current.commits > previous.commits {
        return Progress::Progressing("new commits");
    }
    if current.gate_runs > previous.gate_runs {
        return Progress::Progressing("new gate runs");
    }
    if current.changed_paths != previous.changed_paths {
        return Progress::Progressing("working tree changed");
    }
    Progress::Stalled
}

/// What the 2-hour check-in decides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckIn {
    /// Under two hours, or real progress: continue, with a note on the owning issue.
    Continue {
        /// The reason printed or posted.
        note: String,
    },
    /// Two hours with no progress: close the session with WIP state and a card.
    Close {
        /// The reason printed or carded.
        reason: String,
    },
}

/// The check-in decision. Under [`CHECK_IN_SECONDS`] it always continues; past it, a progressing
/// session continues with a note (D53) and a stalled one closes. `stalled_repeats` counts the
/// **previous consecutive stalls, including this one**: 1 is the first stall, 2 or more is spiraled.
pub fn check_in(elapsed_seconds: u64, progress: &Progress, stalled_repeats: u32) -> CheckIn {
    if elapsed_seconds < CHECK_IN_SECONDS {
        return CheckIn::Continue {
            note: format!("session under the 2-hour soft check-in ({elapsed_seconds}s)"),
        };
    }
    match progress {
        Progress::Progressing(why) => CheckIn::Continue {
            note: format!("2-hour check-in: real progress ({why}); continuing"),
        },
        Progress::Stalled if stalled_repeats >= 2 => CheckIn::Close {
            reason: format!(
                "spiraled: two consecutive sessions with no commits, gate runs or edits ({elapsed_seconds}s)"
            ),
        },
        Progress::Stalled => CheckIn::Close {
            reason: format!(
                "stalled: no commits, gate runs or edits for {elapsed_seconds}s of the 2-hour task"
            ),
        },
    }
}

/// The argv to launch: the caller's command with its single `{prompt}` element replaced by the
/// composed prompt. The placeholder may not be the program itself, and a placeholder inside another
/// argument is left alone.
pub fn launch_argv(launch: &[String], prompt: &str) -> Result<Vec<String>> {
    let mut found = 0usize;
    let mut argv = Vec::with_capacity(launch.len());
    for (index, argument) in launch.iter().enumerate() {
        if argument == "{prompt}" {
            if index == 0 {
                return Err(DriveError::PromptPlaceholder(0));
            }
            found += 1;
            argv.push(prompt.to_string());
        } else {
            argv.push(argument.clone());
        }
    }
    if found != 1 {
        return Err(DriveError::PromptPlaceholder(found));
    }
    Ok(argv)
}

/// The session prompt: the caller's prompt, or a default composed from the `loop next` decision.
pub fn compose_prompt(caller_prompt: Option<&str>, decision: &str) -> String {
    if let Some(prompt) = caller_prompt {
        if !prompt.trim().is_empty() {
            return prompt.to_string();
        }
    }
    format!(
        "You are one fresh session of the UOR-R4 five-lab loop. `loop next` decided: {decision}.\n\nRun one unit of work per the loop cycle: read local/claude/onboarding/run-<lab>.txt, the bench inbox (main checkout local/bench/INBOX.md) and your PICKUP; claim, work inside the ORDER's scope, run gates, write the REPORT, push and request the exact-head review; then run `uor-harness loop next` again and take what it offers. End cleanly only when `loop next` prints WAIT or STOP."
    )
}

/// One running (or finished) session, as recorded on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRecord {
    /// The client adapter name.
    pub client: String,
    /// The pinned model id the supervisor recorded from its own launch command.
    pub model: String,
    /// The child pid.
    pub pid: u32,
    /// The child process group, which is what the supervisor signals.
    pub pgid: u32,
    /// Unix start time.
    pub started_unix: u64,
    /// `ps -o lstart=` of the child at launch: the identity every signal is checked against.
    pub started_lstart: String,
    /// The per-session nonce placed in the child's environment, for `ps -E` inspection.
    pub nonce: String,
    /// SHA-256 of the prompt, so a session can be tied to what it was asked.
    pub prompt_sha256: String,
}

/// The session record file for a lab.
pub fn record_path(root: &Path, lab: &str) -> PathBuf {
    root.join(DRIVE_DIR)
        .join(crate::state::slug(lab))
        .join("last-session.json")
}

/// Write the record atomically (temporary file then rename), so a supervisor restart never reads a
/// half-written session.
pub fn write_record(root: &Path, lab: &str, record: &SessionRecord) -> Result<()> {
    let path = record_path(root, lab);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            DriveError::Io(format!("cannot create {}: {error}", parent.display()))
        })?;
    }
    let text = serde_json::json!({
        "client": record.client,
        "model": record.model,
        "pid": record.pid,
        "pgid": record.pgid,
        "started_unix": record.started_unix,
        "started_lstart": record.started_lstart,
        "nonce": record.nonce,
        "prompt_sha256": record.prompt_sha256,
    })
    .to_string();
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, format!("{text}\n"))
        .map_err(|error| DriveError::Io(format!("cannot write {}: {error}", tmp.display())))?;
    std::fs::rename(&tmp, &path)
        .map_err(|error| DriveError::Io(format!("cannot record {}: {error}", path.display())))
}

/// Read the last session record, if any.
pub fn read_record(root: &Path, lab: &str) -> Option<SessionRecord> {
    let text = std::fs::read_to_string(record_path(root, lab)).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    Some(SessionRecord {
        client: value.get("client")?.as_str()?.to_string(),
        model: value.get("model")?.as_str()?.to_string(),
        pid: value.get("pid")?.as_u64()? as u32,
        pgid: value.get("pgid")?.as_u64()? as u32,
        started_unix: value.get("started_unix")?.as_u64()?,
        started_lstart: value
            .get("started_lstart")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        nonce: value
            .get("nonce")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        prompt_sha256: value.get("prompt_sha256")?.as_str()?.to_string(),
    })
}

/// The directory, under a lab's local state, that holds GPU collection records.
pub const GPU_COLLECTION_DIR: &str = "gpu-collections";

/// One GPU collection: a leased GPU run whose artifacts must be collected before the pod is down.
///
/// The lease watcher writes it when it starts renewing and updates it when the lease ends, so
/// `loop next` can offer the collection as work instead of guessing from a pod list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCollection {
    /// Stable id: `<lab>-<renew program slug>-<started unix>`.
    pub id: String,
    /// The lab that owns the run.
    pub lab: String,
    /// The command the watcher ran to renew the lease.
    pub renew: Vec<String>,
    /// Unix start time.
    pub started_unix: u64,
    /// Unix time of the most recent successful lease renewal a live watcher wrote, if any.
    pub last_renewed_unix: Option<u64>,
    /// Unix end time, once the watcher stopped.
    pub ended_unix: Option<u64>,
    /// `running`, `finished` (the lease ended) or `stopped` (a test/limit bound).
    pub status: String,
    /// True once a session has recorded where the artifacts went.
    pub collected: bool,
    /// Paths, relative to the repo or absolute, named when the collection was recorded.
    pub artifacts: Vec<String>,
    /// The watcher's notes, in order.
    pub notes: Vec<String>,
}

/// The GPU collection directory for one lab, under the gitignored `local/` tree.
pub fn collection_dir(root: &Path, lab: &str) -> PathBuf {
    root.join("local")
        .join(crate::state::slug(lab))
        .join(GPU_COLLECTION_DIR)
}

/// A filesystem-safe id for a new collection, deterministic given the renew program and start time.
pub fn collection_id(lab: &str, renew: &[String], started_unix: u64) -> String {
    let program = renew
        .first()
        .map(|argument| {
            Path::new(argument)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| argument.clone())
        })
        .unwrap_or_else(|| "run".to_string());
    let mut slug = String::new();
    for character in program.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        }
    }
    if slug.is_empty() {
        slug = "run".to_string();
    }
    format!("{}-{}-{}", crate::state::slug(lab), slug, started_unix)
}

/// The record path for one collection id.
pub fn collection_path(root: &Path, lab: &str, id: &str) -> PathBuf {
    collection_dir(root, lab).join(format!("{}.json", crate::state::slug(id)))
}

fn collection_value(collection: &GpuCollection) -> serde_json::Value {
    serde_json::json!({
        "id": collection.id,
        "lab": collection.lab,
        "renew": collection.renew,
        "started_unix": collection.started_unix,
        "last_renewed_unix": collection.last_renewed_unix,
        "ended_unix": collection.ended_unix,
        "status": collection.status,
        "collected": collection.collected,
        "artifacts": collection.artifacts,
        "notes": collection.notes,
    })
}

fn write_collection_at(path: &Path, collection: &GpuCollection) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            DriveError::Io(format!("cannot create {}: {error}", parent.display()))
        })?;
    }
    let text = collection_value(collection).to_string();
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, format!("{text}\n")).map_err(|error| {
        DriveError::Io(format!("cannot write {}: {error}", temporary.display()))
    })?;
    std::fs::rename(&temporary, path)
        .map_err(|error| DriveError::Io(format!("cannot record {}: {error}", path.display())))
}

fn collection_from_value(value: &serde_json::Value) -> Option<GpuCollection> {
    let strings = |key: &str| -> Vec<String> {
        value
            .get(key)
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };
    Some(GpuCollection {
        id: value.get("id")?.as_str()?.to_string(),
        lab: value.get("lab")?.as_str()?.to_string(),
        renew: strings("renew"),
        started_unix: value.get("started_unix")?.as_u64()?,
        last_renewed_unix: value.get("last_renewed_unix").and_then(|v| v.as_u64()),
        ended_unix: value.get("ended_unix").and_then(|v| v.as_u64()),
        status: value.get("status")?.as_str()?.to_string(),
        collected: value.get("collected")?.as_bool()?,
        artifacts: strings("artifacts"),
        notes: strings("notes"),
    })
}

/// Write one collection record atomically and return its path.
pub fn write_collection(root: &Path, lab: &str, collection: &GpuCollection) -> Result<PathBuf> {
    let path = collection_path(root, lab, &collection.id);
    write_collection_at(&path, collection)?;
    Ok(path)
}

/// Read every collection record for a lab, oldest id first; malformed files are skipped.
pub fn read_collections(root: &Path, lab: &str) -> Result<Vec<GpuCollection>> {
    let directory = collection_dir(root, lab);
    let entries = match std::fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(DriveError::Io(format!(
                "cannot read {}: {error}",
                directory.display()
            )))
        }
    };
    let mut collections = Vec::new();
    for entry in entries {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        if let Some(collection) = collection_from_value(&value) {
            collections.push(collection);
        }
    }
    collections.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(collections)
}

/// Mark a collection collected and record where its artifacts were placed.
pub fn collect_collection(
    root: &Path,
    lab: &str,
    id: &str,
    artifacts: &[String],
) -> Result<PathBuf> {
    let path = collection_path(root, lab, id);
    let text = std::fs::read_to_string(&path).map_err(|error| {
        DriveError::Io(format!("no collection {id} at {}: {error}", path.display()))
    })?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|error| DriveError::Io(format!("{} is not JSON: {error}", path.display())))?;
    let mut collection = collection_from_value(&value)
        .ok_or_else(|| DriveError::Io(format!("{} is not a collection record", path.display())))?;
    collection.collected = true;
    for artifact in artifacts {
        if !collection.artifacts.contains(artifact) {
            collection.artifacts.push(artifact.clone());
        }
    }
    write_collection_at(&path, &collection)?;
    Ok(path)
}

/// Watch a leased GPU run and keep a collection record: written `running` before the first
/// renewal, updated when the lease ends (`finished`) or a bound stops the watcher (`stopped`).
pub fn watch_lease_collect(
    root: &Path,
    lab: &str,
    renew: &[String],
    interval_seconds: u64,
    ticks: Option<usize>,
) -> Result<(PathBuf, Vec<String>)> {
    let started_unix = unix_now();
    let mut id = collection_id(lab, renew, started_unix);
    let mut attempt = 2u32;
    while collection_path(root, lab, &id).exists() {
        id = format!("{}-{}", collection_id(lab, renew, started_unix), attempt);
        attempt += 1;
    }
    let mut collection = GpuCollection {
        id,
        lab: lab.to_string(),
        renew: renew.to_vec(),
        started_unix,
        last_renewed_unix: None,
        ended_unix: None,
        status: "running".to_string(),
        collected: false,
        artifacts: Vec::new(),
        notes: Vec::new(),
    };
    let path = write_collection(root, lab, &collection)?;
    let outcome = watch_lease_until_renew_fails_with(renew, interval_seconds, ticks, |_| {
        collection.last_renewed_unix = Some(unix_now());
        write_collection_at(&path, &collection)
    });
    collection.ended_unix = Some(unix_now());
    match outcome {
        Ok(notes) => {
            collection.notes = notes.clone();
            collection.status = if notes.iter().any(|note| note.contains("lease ended")) {
                "finished".to_string()
            } else {
                "stopped".to_string()
            };
            write_collection_at(&path, &collection)?;
            Ok((path, notes))
        }
        Err(error) => {
            collection.status = "stopped".to_string();
            collection.notes = vec![format!("watcher error: {error}")];
            write_collection_at(&path, &collection)?;
            Err(error)
        }
    }
}
/// `ps -o <field> -p <pid>`, trimmed; `None` when the process is gone or `ps` fails.
fn ps_field(pid: u32, field: &str) -> Option<String> {
    let out = Command::new("ps")
        .args(["-o", field, "-p", &pid.to_string()])
        // The identity string must not depend on the caller's locale or time zone: a watcher
        // started under another environment must read the same start time.
        .env("LC_ALL", "C")
        .env("TZ", "UTC")
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

/// The supervisor's own process group, for the self-signal guard.
fn own_pgid() -> Option<u32> {
    ps_field(std::process::id(), "pgid=").and_then(|value| value.parse().ok())
}

/// Is the recorded process still the one that was started? Pid plus its `lstart` identity, never a
/// pattern: a reused pid has another start time, and a leftover child of the client is another pid.
pub fn session_alive(record: &SessionRecord) -> bool {
    if record.pid <= 1 || record.started_lstart.is_empty() {
        return false;
    }
    ps_field(record.pid, "lstart=").is_some_and(|current| current == record.started_lstart)
}
/// Start one session: the argv runs in its own process group with a per-session nonce in its
/// environment, stdout/stderr go to `log`, and the record is written before the caller waits. If the
/// record cannot be written the child is killed by its own group and the error is returned, so no
/// untracked session is ever left running.
pub fn start_session(
    root: &Path,
    lab: &str,
    client: &str,
    model: &str,
    argv: &[String],
    prompt_sha256: &str,
    log: &Path,
) -> Result<(Child, SessionRecord)> {
    let (program, arguments) = argv
        .split_first()
        .ok_or(DriveError::MissingFlag("a launch command after --"))?;
    if let Some(parent) = log.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            DriveError::Io(format!("cannot create {}: {error}", parent.display()))
        })?;
    }
    let out = std::fs::File::create(log)
        .map_err(|error| DriveError::Io(format!("cannot create {}: {error}", log.display())))?;
    let err = out
        .try_clone()
        .map_err(|error| DriveError::Io(format!("cannot clone the log handle: {error}")))?;
    let nonce = format!(
        "{}-{}",
        unix_now(),
        &prompt_sha256[..16.min(prompt_sha256.len())]
    );
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let mut command = Command::new(program);
        command
            .args(arguments)
            .current_dir(root)
            .stdin(Stdio::null())
            .stdout(Stdio::from(out))
            .stderr(Stdio::from(err))
            .env("UOR_LOOP_DRIVE_NONCE", &nonce)
            .env("UOR_LOOP_DRIVE_LAB", lab);
        strip_git_env(&mut command);
        command.process_group(0);
        let mut child = command
            .spawn()
            .map_err(|error| DriveError::Io(format!("cannot launch {program}: {error}")))?;
        let pid = child.id();
        let started_lstart = ps_field(pid, "lstart=").unwrap_or_default();
        if started_lstart.is_empty() {
            // Without an identity no later signal can be checked, and an unbounded wait could hang.
            let _ = signal_group(pid, "KILL");
            let _ = child.wait();
            return Err(DriveError::Refused(format!(
                "cannot read the start identity of pid {pid}; refusing to supervise it"
            )));
        }
        let record = SessionRecord {
            client: client.to_string(),
            model: model.to_string(),
            pid,
            pgid: pid,
            started_unix: unix_now(),
            started_lstart,
            nonce,
            prompt_sha256: prompt_sha256.to_string(),
        };
        if let Err(error) = write_record(root, lab, &record) {
            let _ = signal_group(pid, "KILL");
            let _ = child.wait();
            return Err(error);
        }
        Ok((child, record))
    }
    #[cfg(not(unix))]
    {
        let _ = (
            program,
            arguments,
            out,
            err,
            nonce,
            client,
            model,
            prompt_sha256,
            log,
        );
        Err(DriveError::Io(
            "loop drive signals process groups and is unix-only".to_string(),
        ))
    }
}

/// `kill -<signal> -- -<group>`, the one place a signal is sent.
fn signal_group(pgid: u32, signal: &str) -> std::io::Result<std::process::ExitStatus> {
    if pgid <= 1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("refusing to signal process group {pgid}"),
        ));
    }
    Command::new("kill")
        .args([&format!("-{signal}"), "--", &format!("-{pgid}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
}

/// Terminate the recorded session: guards, then SIGTERM to its own process group, a short grace, then
/// SIGKILL to the same group. Nothing else is ever signalled - not another lab's worktree, not a
/// `pkill`-style pattern, not a pid that was not recorded, and nothing when the identity check fails.
pub fn terminate_session(record: &SessionRecord) -> Result<()> {
    if record.pgid <= 1 {
        return Err(DriveError::Refused(format!(
            "refusing to signal process group {}: not a session group",
            record.pgid
        )));
    }
    if record.pgid != record.pid {
        return Err(DriveError::Refused(format!(
            "refusing to signal group {}: the recorded pid {} is not its leader",
            record.pgid, record.pid
        )));
    }
    if own_pgid() == Some(record.pgid) {
        return Err(DriveError::Refused(format!(
            "refusing to signal group {}: it is the supervisor's own group",
            record.pgid
        )));
    }
    if !session_alive(record) {
        return Ok(());
    }
    let _ = signal_group(record.pgid, "TERM");
    for _ in 0..20 {
        if !session_alive(record) {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let _ = signal_group(record.pgid, "KILL");
    Ok(())
}

/// The session-bound watcher: while the tracked session lives, run the caller's renew command every
/// `interval_seconds` and note it; when the session ends, the watcher ends. It never signals the run.
/// `ticks` bounds the loop for tests; production passes `None`.
pub fn watch_until_exit(
    record: &SessionRecord,
    renew: &[String],
    interval_seconds: u64,
    ticks: Option<usize>,
) -> Result<Vec<String>> {
    if renew.is_empty() {
        return Err(DriveError::MissingFlag("a renew command after --"));
    }
    let mut notes = Vec::new();
    let mut rounds = 0usize;
    while session_alive(record) {
        if let Some(limit) = ticks {
            if rounds >= limit {
                notes.push(format!("watch stopped after {limit} tick(s) (test bound)"));
                break;
            }
        }
        let (program, arguments) = (&renew[0], &renew[1..]);
        let out = Command::new(program)
            .args(arguments)
            .output()
            .map_err(|error| DriveError::Io(format!("cannot run the renew command: {error}")))?;
        notes.push(format!(
            "renewed at {}: {} {}",
            unix_now(),
            out.status,
            String::from_utf8_lossy(&out.stdout).trim()
        ));
        rounds += 1;
        std::thread::sleep(std::time::Duration::from_secs(interval_seconds));
    }
    notes.push("session ended; watcher exits without cancelling anything".to_string());
    Ok(notes)
}

/// A leased GPU run's watcher: it **outlives the lab session**. Started detached, it renews the lease
/// every `interval_seconds` while the renew command succeeds and stops when that command reports the
/// lease is gone (a non-zero exit is the caller's lease-ended signal). It never signals the run and
/// never cancels a healthy one; `ticks` bounds the loop for tests.
pub fn watch_lease_until_renew_fails(
    renew: &[String],
    interval_seconds: u64,
    ticks: Option<usize>,
) -> Result<Vec<String>> {
    watch_lease_until_renew_fails_with(renew, interval_seconds, ticks, |_| Ok(()))
}

/// Like [`watch_lease_until_renew_fails`], but calls `on_renewed` after each successful renewal so a
/// caller can write a liveness heartbeat. The callback sees the notes so far and may fail the watch.
pub fn watch_lease_until_renew_fails_with(
    renew: &[String],
    interval_seconds: u64,
    ticks: Option<usize>,
    mut on_renewed: impl FnMut(&[String]) -> Result<()>,
) -> Result<Vec<String>> {
    if renew.is_empty() {
        return Err(DriveError::MissingFlag("a renew command after --"));
    }
    let mut notes = Vec::new();
    let mut rounds = 0usize;
    loop {
        if let Some(limit) = ticks {
            if rounds >= limit {
                notes.push(format!(
                    "lease watch stopped after {limit} tick(s) (test bound)"
                ));
                return Ok(notes);
            }
        }
        let (program, arguments) = (&renew[0], &renew[1..]);
        let out = Command::new(program)
            .args(arguments)
            .output()
            .map_err(|error| DriveError::Io(format!("cannot run the renew command: {error}")))?;
        if !out.status.success() {
            notes.push(format!(
                "lease ended at {}: renew exited {}; watcher stops without cancelling the run",
                unix_now(),
                out.status
            ));
            return Ok(notes);
        }
        notes.push(format!(
            "renewed at {}: {}",
            unix_now(),
            String::from_utf8_lossy(&out.stdout).trim()
        ));
        on_renewed(&notes)?;
        rounds += 1;
        std::thread::sleep(std::time::Duration::from_secs(interval_seconds));
    }
}
/// Reap a terminated child without an unbounded wait: after the signal, poll `try_wait` for a bound;
/// if it still runs, send KILL to its recorded group once more and poll a short bound, then refuse.
fn reap_child(child: &mut Child, record: &SessionRecord) -> Result<()> {
    for _ in 0..50 {
        if child
            .try_wait()
            .map_err(|error| DriveError::Io(format!("cannot poll pid {}: {error}", record.pid)))?
            .is_some()
        {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let _ = signal_group(record.pgid, "KILL");
    for _ in 0..10 {
        if child
            .try_wait()
            .map_err(|error| DriveError::Io(format!("cannot poll pid {}: {error}", record.pid)))?
            .is_some()
        {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Err(DriveError::Refused(format!(
        "pid {} did not exit after TERM and KILL; refusing to wait unbounded",
        record.pid
    )))
}

/// The inputs one `loop drive` invocation needs, separate from the hooks so the session loop can be
/// tested with a fake clock and a fake card sink.
pub struct DriveSpec<'a> {
    /// Repository root.
    pub root: &'a Path,
    /// Lab name.
    pub lab: &'a str,
    /// Client adapter name.
    pub client: &'a str,
    /// Pinned model id.
    pub model: &'a str,
    /// Caller launch argv, with one `{prompt}`.
    pub launch: &'a [String],
    /// Caller prompt override.
    pub prompt: Option<&'a str>,
    /// How many sessions this invocation may start.
    pub max_sessions: usize,
    /// The active ORDER, whose task folder bounds the WIP commit.
    pub order: Option<&'a Path>,
    /// A leased session is never closed by the check-in.
    pub exempt_lease: bool,
    /// Seconds to wait through `pause` on a WAIT before asking `loop next` again.
    pub poll: u64,
}

/// What `loop next` told the drive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Next {
    /// Start a session with this decision line.
    Go(String),
    /// Nothing to take yet: pause `poll` seconds and ask again; no session is started.
    Wait(String),
    /// A clean stop (the kill switch and the loop page's other stops).
    Stop,
}

/// The impure edges of a drive run, injectable for the fake-clock tests.
pub struct DriveHooks<'a> {
    /// Seconds since the epoch.
    pub now: &'a dyn Fn() -> u64,
    /// Wait between liveness polls.
    pub pause: &'a dyn Fn(u64),
    /// `loop next`: `Go` starts a session, `Wait` pauses and asks again, `Stop` is a clean stop,
    /// `Err` refuses.
    pub decision: &'a dyn Fn() -> std::result::Result<Next, String>,
    /// Post the PM card for a close; the CLI implementation posts on #2028.
    pub card: &'a dyn Fn(&str) -> std::result::Result<(), String>,
    /// Notes a person should see (a check-in note, a lease skip); printed by the CLI, recorded by tests.
    pub notice: &'a dyn Fn(&str),
}

/// Run the sessions. The check-in is evaluated **while the child is alive**: `try_wait` polls it, the
/// clock is injected, and only a session classified Stalled is terminated; a Progressing one keeps
/// running with a note. A leased session (`exempt_lease`) is never closed by the check-in. The
/// previous consecutive stall count is carried in so a second stall is reported as spiraled.
pub fn run_sessions(spec: &DriveSpec, hooks: &DriveHooks) -> Result<u8> {
    let mut last_code = 0u8;
    let mut stalls: u32 = 0;
    for session in 0..spec.max_sessions.max(1) {
        // WAIT is not a stop and does not use a session: pause and ask `loop next` again.
        let decision = loop {
            match (hooks.decision)() {
                Ok(Next::Go(decision)) => break decision,
                Ok(Next::Wait(reason)) => {
                    println!("WAIT {reason} — check again in {}s", spec.poll);
                    (hooks.pause)(spec.poll);
                }
                Ok(Next::Stop) => {
                    println!("STOP");
                    // A supervisor stop is exactly the case the PM must prompt the lab for; a
                    // clean session exit is the supervisor's restart path and stays uncarded.
                    let _ = (hooks.card)(
                        "STOP: loop next asked the supervisor to stop; the lab needs the next prompt",
                    );
                    return Ok(0);
                }
                Err(error) => {
                    return Err(DriveError::Io(format!(
                        "cannot decide with loop next: {error}"
                    )));
                }
            }
        };
        let prompt = compose_prompt(spec.prompt, &decision);
        let argv = launch_argv(spec.launch, &prompt)?;
        let mut before = Observation::take(spec.root, spec.lab);
        let directory = spec.root.join(DRIVE_DIR).join(crate::state::slug(spec.lab));
        let log = directory.join(format!("session-{}-{session}.log", (hooks.now)()));
        let digest = crate::attention_reference::sha256_hex(prompt.as_bytes());
        let (mut child, record) = start_session(
            spec.root,
            spec.lab,
            spec.client,
            spec.model,
            &argv,
            &digest,
            &log,
        )?;
        println!(
            "started session {session} pid {} pgid {} client {} model {} log {}",
            record.pid,
            record.pgid,
            spec.client,
            spec.model,
            log.display()
        );
        let mut next_check = record.started_unix + CHECK_IN_SECONDS;
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    // The session ended on its own: there is nothing left to close, and the
                    // check-in only closes a **live** stalled session. Reset the stall counter on
                    // real progress so the next session starts fresh.
                    let after = Observation::take(spec.root, spec.lab);
                    if matches!(classify(&before, &after), Progress::Progressing(_)) {
                        stalls = 0;
                    }
                    last_code = status.code().unwrap_or(1) as u8;
                    break;
                }
                Ok(None) => {
                    let now = (hooks.now)();
                    let elapsed = now.saturating_sub(record.started_unix);
                    if now >= next_check {
                        let after = Observation::take(spec.root, spec.lab);
                        let progress = classify(&before, &after);
                        let stall_count = if matches!(progress, Progress::Stalled) {
                            stalls + 1
                        } else {
                            0
                        };
                        match check_in(elapsed, &progress, stall_count) {
                            CheckIn::Continue { note } => {
                                (hooks.notice)(&note);
                                before = after;
                                next_check = next_check.saturating_add(CHECK_IN_SECONDS);
                                stalls = 0;
                            }
                            CheckIn::Close { reason } => {
                                if spec.exempt_lease {
                                    // A leased run is never closed, but the window must advance:
                                    // otherwise every poll re-observes and re-prints for hours.
                                    (hooks.notice)(&format!(
                                        "leased session; close skipped: {reason}"
                                    ));
                                    before = after;
                                    next_check = next_check.saturating_add(CHECK_IN_SECONDS);
                                } else {
                                    println!("CHECK-IN CLOSE: {reason}");
                                    terminate_session(&record)?;
                                    reap_child(&mut child, &record)?;
                                    close_actions(spec, &record, &reason, hooks, &mut stalls);
                                    last_code = 1;
                                    break;
                                }
                            }
                        }
                    }
                    (hooks.pause)(1);
                }
                Err(error) => {
                    let _ = terminate_session(&record);
                    return Err(DriveError::Io(format!(
                        "cannot poll pid {}: {error}",
                        record.pid
                    )));
                }
            }
        }
    }
    // Reaching an explicit multi-session limit is a supervisor stop, so the PM gets the prompt
    // card. The default single session (DEFAULT_MAX_SESSIONS) is a normal restart unit and is not
    // carded, which is what keeps the pinned progressing-client test at zero cards.
    if spec.max_sessions > 1 {
        let _ = (hooks.card)(&format!(
            "reached --max-sessions ({} sessions): the lab needs the next prompt",
            spec.max_sessions
        ));
    }
    Ok(last_code)
}

fn close_actions(
    spec: &DriveSpec,
    record: &SessionRecord,
    reason: &str,
    hooks: &DriveHooks,
    stalls: &mut u32,
) {
    *stalls += 1;
    if let Err(error) = close_stalled_session(spec.root, spec.lab, record, reason) {
        eprintln!("loop drive: cannot record the stalled session: {error}");
    }
    if let Err(error) = commit_and_push_wip(spec.root, spec.lab, spec.order, reason) {
        eprintln!("loop drive: cannot commit/push the WIP: {error}");
    }
    if let Err(error) = (hooks.card)(reason) {
        eprintln!("loop drive: cannot post the PM card: {error}");
    }
}
/// Commit and push the WIP of a closed session, scoped and legal: it refuses a branch that is not
/// `<lab>/*` or is `main`, stages **only the active ORDER task folder**, adds the `Order:` trailer and
/// strips `GIT_*` from the git child. A failure is returned, never silently swallowed; nothing is
/// ever force-pushed or deleted.
pub fn commit_and_push_wip(
    root: &Path,
    lab: &str,
    order: Option<&Path>,
    reason: &str,
) -> Result<()> {
    let git = |args: &[&str]| -> Result<std::process::Output> {
        let mut command = Command::new("git");
        command.args(args).current_dir(root);
        strip_git_env(&mut command);
        command
            .output()
            .map_err(|error| DriveError::Io(format!("cannot run git {:?}: {error}", args)))
    };
    let branch = git(&["branch", "--show-current"])?;
    let branch = String::from_utf8_lossy(&branch.stdout).trim().to_string();
    if branch == "main" || !branch.starts_with(&format!("{lab}/")) {
        return Err(DriveError::Refused(format!(
            "refusing to commit WIP on branch {branch:?}: not a {lab}/ branch"
        )));
    }
    let order = order.ok_or_else(|| {
        DriveError::Refused(
            "refusing to commit unscoped WIP: pass --order <the active ORDER>".to_string(),
        )
    })?;
    let order_rel = order.strip_prefix(root).map_err(|_| {
        DriveError::Refused(format!(
            "the active ORDER {} is not under the repository root",
            order.display()
        ))
    })?;
    let order_rel = order_rel.to_string_lossy().to_string();
    if !order_rel.starts_with("docs/loop/tasks/") || !order_rel.ends_with("/ORDER.md") {
        return Err(DriveError::Refused(format!(
            "refusing to commit WIP: {order_rel} is not a loop task ORDER"
        )));
    }
    let folder = order_rel
        .strip_suffix("/ORDER.md")
        .ok_or_else(|| DriveError::Refused("not an ORDER path".to_string()))?;
    let folder_name = folder.rsplit('/').next().unwrap_or_default();
    if !folder_name.starts_with(&format!("{lab}-")) {
        return Err(DriveError::Refused(format!(
            "refusing to commit WIP: {folder} is not a {lab} task folder"
        )));
    }
    let add = git(&["add", "--", folder])?;
    if !add.status.success() {
        return Err(DriveError::Io(format!(
            "git add {folder} refused: {}",
            String::from_utf8_lossy(&add.stderr).trim()
        )));
    }
    let staged = git(&["diff", "--cached", "--quiet"])?;
    if staged.status.success() {
        return Ok(());
    }
    let commit = git(&[
        "commit",
        "-m",
        &format!(
            "[{lab}] WIP: drive closed a session at the 2-hour check-in\n\n- reason: {reason}\n\nLab: {lab}\nOrder: {order_rel}"
        ),
    ])?;
    if !commit.status.success() {
        return Err(DriveError::Io(format!(
            "git commit refused: {}",
            String::from_utf8_lossy(&commit.stderr).trim()
        )));
    }
    let push = git(&["push", "origin", &branch])?;
    if !push.status.success() {
        return Err(DriveError::Io(format!(
            "git push origin {branch} refused: {}",
            String::from_utf8_lossy(&push.stderr).trim()
        )));
    }
    Ok(())
}

/// The body a check-in progress note posts on the work issue: the lab tag, the D53 reason and the
/// session target, and the explicit no-claim line.
pub fn note_body(lab: &str, target: Option<&str>, note: &str) -> String {
    let mut body = format!("[{lab}] drive check-in (soft 2-hour, D53): {note}");
    if let Some(target) = target {
        body.push_str(&format!("\n\n- session target: {target}"));
    }
    body.push_str("\n\nNo result is claimed; the session continues.");
    body
}

/// Post one check-in note on `issue` with the given `gh` program. The CLI passes `gh`; tests pass
/// a fake script, so the post pipeline (argv, body file, exit status) is asserted without a network.
pub fn post_note_with(
    root: &Path,
    gh: &Path,
    lab: &str,
    issue: u64,
    target: Option<&str>,
    note: &str,
) -> Result<()> {
    let directory = root.join(DRIVE_DIR).join(crate::state::slug(lab));
    std::fs::create_dir_all(&directory).map_err(|error| {
        DriveError::Io(format!("cannot create {}: {error}", directory.display()))
    })?;
    let path = {
        let mut path = directory.join(format!("note-{}.md", unix_now()));
        let mut suffix = 1;
        while path.exists() {
            path = directory.join(format!("note-{}-{suffix}.md", unix_now()));
            suffix += 1;
        }
        path
    };
    std::fs::write(&path, format!("{}\n", note_body(lab, target, note)))
        .map_err(|error| DriveError::Io(format!("cannot write {}: {error}", path.display())))?;
    let status = Command::new(gh)
        .args(["issue", "comment", &issue.to_string(), "--body-file"])
        .arg(&path)
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| DriveError::Io(format!("cannot run {}: {error}", gh.display())))?;
    if status.success() {
        Ok(())
    } else {
        Err(DriveError::Io(format!(
            "{} issue comment {issue} exited with {status}",
            gh.display()
        )))
    }
}

/// Post one check-in note through the real `gh`.
pub fn post_note(
    root: &Path,
    lab: &str,
    issue: u64,
    target: Option<&str>,
    note: &str,
) -> Result<()> {
    post_note_with(root, Path::new("gh"), lab, issue, target, note)
}

/// True when the model recorded for the session is actually carried by the launch command, so the
/// pin is checked rather than free text that only claims to be the model. Whole arguments only: an
/// argument equal to the model, `--model=<model>` or `-m=<model>`, or the value right after
/// `--model`/`-m`. A substring (`--model deepseek` against `deepseek-flash`, or `prompt` inside
/// `{prompt}`) never counts.
pub fn model_in_launch(model: &str, launch: &[String]) -> bool {
    if model.is_empty() {
        return false;
    }
    let mut after_flag = false;
    for arg in launch {
        if after_flag && arg == model {
            return true;
        }
        after_flag = arg == "--model" || arg == "-m";
        if arg == model {
            return true;
        }
        for prefix in ["--model=", "-m="] {
            if arg.strip_prefix(prefix) == Some(model) {
                return true;
            }
        }
    }
    false
}

/// The issue a check-in note posts on: the number of a `#<pr>` target, else the tracker #2028
/// (an item id, a command or no target are not an issue number).
pub fn note_issue(target: Option<&str>) -> u64 {
    target
        .and_then(|value| value.strip_prefix('#'))
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(2028)
}
/// The PM card for a closed session: what the PM reads to prompt the lab. Pure so the wording is
/// tested; [`post_pm_card`] sends it on #2028.
pub fn pm_card_text(lab: &str, reason: &str, head: &str, commits: u64) -> String {
    format!(
        "[{lab}] drive closed a session at the soft 2-hour check-in ({reason}).\n\n- git HEAD: {head}\n- commits: {commits}\nThe REPORT stub and PICKUP are next to this card under local/loop/drive/{lab}/close-*/. The lab needs a prompt."
    )
}

/// Post the PM card on #2028 through `gh`, with a cheap dedupe: if the card file already carries the
/// same head and reason, the post is skipped. Live-only: the tests test the wording and the stub
/// files, never the network.
pub fn post_pm_card(root: &Path, lab: &str, reason: &str) -> Result<()> {
    let observation = Observation::take(root, lab);
    let text = pm_card_text(lab, reason, &observation.head, observation.commits);
    let path = root
        .join(DRIVE_DIR)
        .join(crate::state::slug(lab))
        .join("pm-card.md");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            DriveError::Io(format!("cannot create {}: {error}", parent.display()))
        })?;
    }
    if let Ok(previous) = std::fs::read_to_string(&path) {
        if previous.contains(reason) && previous.contains(&observation.head) {
            return Ok(());
        }
    }
    std::fs::write(&path, format!("{text}\n"))
        .map_err(|error| DriveError::Io(format!("cannot write {}: {error}", path.display())))?;
    let status = Command::new("gh")
        .args(["issue", "comment", "2028", "--body-file"])
        .arg(&path)
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| DriveError::Io(format!("cannot run gh: {error}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(DriveError::Io(format!(
            "gh issue comment 2028 exited {status}"
        )))
    }
}

/// The per-lab drive lock: one supervisor per lab, created O_EXCL and removed by its guard.
struct DriveLock {
    path: PathBuf,
}

impl DriveLock {
    fn acquire(root: &Path, lab: &str) -> Result<DriveLock> {
        let path = root
            .join(DRIVE_DIR)
            .join(crate::state::slug(lab))
            .join("drive.lock");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                DriveError::Io(format!("cannot create {}: {error}", parent.display()))
            })?;
        }
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                use std::io::Write;
                let _ = writeln!(file, "{}", std::process::id());
                Ok(DriveLock { path })
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(DriveError::Refused(format!(
                    "another loop drive holds {}; stop it before starting a second",
                    path.display()
                )))
            }
            Err(error) => Err(DriveError::Io(format!(
                "cannot create {}: {error}",
                path.display()
            ))),
        }
    }
}

impl Drop for DriveLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Seconds since the Unix epoch.
pub fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
/// Dispatch `loop drive`. Exit codes: 0 on a clean stop or a completed session, 1 when a session ran
/// and its client failed, 2 for usage and refusal errors.
pub fn cli(root: &Path, lab: &str, args: &[String]) -> u8 {
    let mut client: Option<String> = None;
    let mut model: Option<String> = None;
    let mut prompt: Option<String> = None;
    let mut prompt_file: Option<PathBuf> = None;
    let mut order: Option<PathBuf> = None;
    let mut max_sessions: Option<usize> = None;
    let mut watch = false;
    let mut lease = false;
    let mut exempt_lease = false;
    let mut pid: Option<u32> = None;
    let mut interval: u64 = 60;
    let mut poll: u64 = crate::loopnext::RECHECK_SECS;
    let mut launch: Vec<String> = Vec::new();
    let mut collect: Option<String> = None;
    let mut artifacts: Vec<String> = Vec::new();
    let mut index = 0;
    let mut after_separator = false;
    while index < args.len() {
        let argument = args[index].as_str();
        if after_separator {
            launch.push(argument.to_string());
            index += 1;
            continue;
        }
        match argument {
            "--" => after_separator = true,
            "--watch" => watch = true,
            "--lease" => lease = true,
            "--exempt-lease" => exempt_lease = true,
            "--client" | "--model" | "--prompt" | "--prompt-file" | "--max-sessions" | "--pid"
            | "--interval" | "--order" | "--poll" | "--collect" | "--artifact" => {
                let Some(value) = args.get(index + 1) else {
                    eprintln!("loop drive: {argument} needs a value");
                    return 2;
                };
                match argument {
                    "--client" => client = Some(value.clone()),
                    "--model" => model = Some(value.clone()),
                    "--prompt" => prompt = Some(value.clone()),
                    "--prompt-file" => prompt_file = Some(PathBuf::from(value)),
                    "--order" => order = Some(PathBuf::from(value)),
                    "--collect" => collect = Some(value.clone()),
                    "--artifact" => artifacts.push(value.clone()),
                    "--pid" => pid = value.parse().ok(),
                    "--interval" => interval = value.parse().unwrap_or(60),
                    "--poll" => match value.parse::<u64>() {
                        Ok(seconds) if seconds > 0 => poll = seconds,
                        _ => {
                            eprintln!(
                                "loop drive: --poll needs a whole number of seconds above 0, got {value:?}"
                            );
                            return 2;
                        }
                    },
                    _ => max_sessions = value.parse().ok(),
                }
                index += 2;
                continue;
            }
            other => {
                eprintln!(
                    "loop drive: unknown argument {other:?} (the client command goes after --)"
                );
                return 2;
            }
        }
        index += 1;
    }
    // A zero or tiny interval would spin the watcher; the CLI sets a floor, the library keeps the
    // value for tests that inject it.
    let interval = interval.max(1);
    if watch {
        if lease {
            return match watch_lease_collect(root, lab, &launch, interval, None) {
                Ok((path, notes)) => {
                    println!("collection: {}", path.display());
                    for note in notes {
                        println!("{note}");
                    }
                    0
                }
                Err(error) => {
                    eprintln!("loop drive --watch --lease: {error}");
                    2
                }
            };
        }
        let Some(pid) = pid else {
            eprintln!("loop drive --watch needs --pid <the tracked session pid>, or --lease");
            return 2;
        };
        let record = read_record(root, lab).filter(|record| record.pid == pid);
        let Some(record) = record else {
            eprintln!("loop drive --watch: no recorded session with pid {pid}");
            return 2;
        };
        return match watch_until_exit(&record, &launch, interval, None) {
            Ok(notes) => {
                for note in notes {
                    println!("{note}");
                }
                0
            }
            Err(error) => {
                eprintln!("loop drive --watch: {error}");
                2
            }
        };
    }
    if let Some(id) = &collect {
        return match collect_collection(root, lab, id, &artifacts) {
            Ok(path) => {
                println!("collected {id} ({})", path.display());
                0
            }
            Err(error) => {
                eprintln!("loop drive --collect: {error}");
                2
            }
        };
    }
    if !cfg!(unix) {
        eprintln!("loop drive signals process groups and is unix-only");
        return 2;
    }
    let Some(client) = client else {
        eprintln!("loop drive needs --client <name>");
        return 2;
    };
    let Some(model) = model else {
        eprintln!("loop drive needs --model <id>");
        return 2;
    };
    if !root
        .join("docs/harness/onboarding")
        .join(format!("{client}.json"))
        .is_file()
    {
        eprintln!("loop drive: no client adapter docs/harness/onboarding/{client}.json");
        return 2;
    }
    if launch.is_empty() {
        eprintln!("loop drive needs the client command after --, with one {{prompt}} placeholder");
        return 2;
    }
    if launch_argv(&launch, "prompt").is_err() {
        eprintln!("loop drive: the launch command must contain exactly one {{prompt}} placeholder");
        return 2;
    }
    if !model_in_launch(&model, &launch) {
        eprintln!(
            "loop drive: --model {model:?} is not carried by the launch command; put the pinned model in the --launch argv"
        );
        return 2;
    }
    if let Some(path) = &prompt_file {
        match std::fs::read_to_string(path) {
            Ok(text) => prompt = Some(text),
            Err(error) => {
                eprintln!("loop drive: cannot read {}: {error}", path.display());
                return 2;
            }
        }
    }
    let lock = match DriveLock::acquire(root, lab) {
        Ok(lock) => lock,
        Err(error) => {
            eprintln!("loop drive: {error}");
            return 2;
        }
    };
    let next_gh = crate::loopnext::RealNextGh {
        root: root.to_path_buf(),
    };
    let claim_gh = crate::loopclaim::RealGh {
        root: root.to_path_buf(),
    };
    // WAIT is not a stop: `run_sessions` pauses `--poll` seconds through its hook and asks again.
    // The last GO target rides along so a check-in note knows which issue to post on (#<pr> when the
    // decision names a PR; the tracker #2028 for an item id, a command or a stop).
    let last_target: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
    let decision = || -> std::result::Result<Next, String> {
        match crate::loopnext::run(root, lab, &next_gh, &claim_gh) {
            Ok((_, crate::loopnext::Decision::Wait(reason))) => {
                *last_target.borrow_mut() = None;
                Ok(Next::Wait(reason))
            }
            Ok((_, crate::loopnext::Decision::Stop(reason))) => {
                *last_target.borrow_mut() = None;
                println!("STOP {reason}");
                Ok(Next::Stop)
            }
            Ok((
                _,
                crate::loopnext::Decision::Go {
                    kind,
                    target,
                    reason,
                },
            )) => {
                *last_target.borrow_mut() = Some(target.clone());
                Ok(Next::Go(format!("GO {kind} {target} - {reason}")))
            }
            Err(error) => Err(error),
        }
    };
    let card = |reason: &str| post_pm_card(root, lab, reason).map_err(|error| error.to_string());
    let now = || unix_now();
    let pause = |seconds: u64| std::thread::sleep(std::time::Duration::from_secs(seconds));
    let notice = |note: &str| {
        println!("{note}");
        let target = last_target.borrow().clone();
        let issue = note_issue(target.as_deref());
        if let Err(error) = post_note(root, lab, issue, target.as_deref(), note) {
            eprintln!("loop drive: cannot post the check-in note: {error}");
        }
    };
    let spec = DriveSpec {
        root,
        lab,
        client: &client,
        model: &model,
        launch: &launch,
        prompt: prompt.as_deref(),
        max_sessions: max_sessions.unwrap_or(DEFAULT_MAX_SESSIONS),
        order: order.as_deref(),
        exempt_lease,
        poll,
    };
    let hooks = DriveHooks {
        now: &now,
        pause: &pause,
        decision: &decision,
        card: &card,
        notice: &notice,
    };
    let code = match run_sessions(&spec, &hooks) {
        Ok(0) => 0,
        Ok(_) => 1,
        Err(error) => {
            let _ = (card)(&format!("drive failed: {error}"));
            eprintln!("loop drive: {error}");
            2
        }
    };
    drop(lock);
    code
}

/// Record a stalled session from git state: a REPORT stub and a PICKUP under the drive directory, so
/// the next session and the PM card can see what was happening. Nothing is deleted and no other file
/// outside the drive directory is touched.
pub fn close_stalled_session(
    root: &Path,
    lab: &str,
    record: &SessionRecord,
    reason: &str,
) -> Result<PathBuf> {
    let directory = root
        .join(DRIVE_DIR)
        .join(crate::state::slug(lab))
        .join(format!("close-{}", unix_now()));
    std::fs::create_dir_all(&directory).map_err(|error| {
        DriveError::Io(format!("cannot create {}: {error}", directory.display()))
    })?;
    let observation = Observation::take(root, lab);
    let stub = format!(
        "# REPORT stub: session closed by the soft 2-hour check-in\n\n- lab: {lab}\n- client: {}\n- model: {}\n- pid: {}\n- started_unix: {}\n- reason: {reason}\n- git HEAD: {}\n- commits: {}\n- changed paths: {}\n\nWritten from git state only; no result is claimed.\n",
        record.client,
        record.model,
        record.pid,
        record.started_unix,
        observation.head,
        observation.commits,
        observation.changed_paths
    );
    std::fs::write(directory.join("REPORT-stub.md"), stub)
        .map_err(|error| DriveError::Io(format!("cannot write the REPORT stub: {error}")))?;
    let pickup = format!(
        "# PICKUP (written by loop drive from git state)\n\n- closed: {reason}\n- HEAD: {}\n- commits: {}\n\nResume by reading the active ORDER and running `uor-harness loop next`.\n",
        observation.head, observation.commits
    );
    std::fs::write(directory.join("PICKUP.md"), pickup)
        .map_err(|error| DriveError::Io(format!("cannot write the PICKUP: {error}")))?;
    Ok(directory)
}
