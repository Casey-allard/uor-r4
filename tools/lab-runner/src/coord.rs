//! Cooperative GitHub coordination. A single-parent non-forced ref update is
//! the ownership transaction; issue comments are only human-readable views.
use crate::{delivery, invalid, jobs, process, spec::JobSpec, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const BRANCH: &str = "codex/lab-state";
const REMOTE_REF: &str = "refs/remotes/origin/codex/lab-state";
const TTL: u64 = 1200;
const REASON_MAX: usize = 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lab {
    pub lab: String,
    pub adapter: String,
    pub capabilities: Vec<String>,
    pub available: bool,
    pub heartbeat: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    pub session: String,
    pub epoch: u64,
    pub work_card: String,
    pub policy_sha: String,
    pub base_sha: String,
    pub paths: Vec<String>,
    pub expires: u64,
    pub phase: String,
    pub checkpoint: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attempt {
    pub issue: u64,
    pub session: String,
    pub task_epoch: u64,
    pub host: String,
    pub phase: String,
    pub receipt: Option<ExitEvidence>,
    /// Legacy unbound reservations remain reserved but cannot admit work.
    #[serde(default)]
    pub spec: Option<JobSpec>,
    #[serde(default)]
    pub spec_sha256: Option<String>,
    #[serde(default)]
    pub runner_root: Option<PathBuf>,
}

/// A locally verified durable DONE receipt, never a free-form assertion that
/// a remote or unobservable worker stopped.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExitEvidence {
    pub host: String,
    pub job_id: String,
    pub attempt_id: String,
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Debug, Deserialize)]
struct ConfirmedExit {
    schema: String,
    id: String,
    attempt_id: String,
    host: String,
    spec_sha256: String,
    coordination: crate::spec::CoordinationClaim,
    process_state: String,
    outcome: String,
    exit_status: Option<i64>,
    elapsed_ms: u64,
}

/// How a durable DONE receipt is bound to a reservation.
#[derive(Clone, Copy, PartialEq, Eq)]
enum EvidenceMode {
    /// Existing finalization: a live claim owns the task generation, and the
    /// receipt must bind the exact reserved spec digest.
    Finalize,
    /// Orphan close-out (`Action::Abandon`): no live claim exists and the
    /// durable spec's digest has drifted away from the reserved digest, so the
    /// durable job is bound by identity instead. The reservation must also
    /// prove that no process ever started.
    Abandon,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub schema: String,
    pub repository: String,
    pub policy_sha: String,
    pub sequence: u64,
    pub observed_server_time: u64,
    pub labs: BTreeMap<String, Lab>,
    pub tasks: BTreeMap<u64, Claim>,
    pub attempts: BTreeMap<String, Attempt>,
}

/// Exact delivered evidence plus a separate full-issue acceptance review.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionEvidence {
    pub path: PathBuf,
    pub sha256: String,
    pub merge_sha: String,
    pub acceptance: delivery::FileEvidence,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompletionAcceptance {
    schema: String,
    task_issue: u64,
    work_card: String,
    head_sha: String,
    merge_sha: String,
    reviewer_session: String,
    decision: String,
    full_scope: bool,
    acceptance_criteria: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Event {
    pub event_id: String,
    #[serde(flatten)]
    pub action: Action,
}

// Flatten and deny_unknown_fields cannot be combined in a derived deserializer.
// Parse the envelope explicitly, then require every remaining action field to be known.
impl<'de> Deserialize<'de> for Event {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let serde_json::Value::Object(mut fields) = serde_json::Value::deserialize(deserializer)?
        else {
            return Err(serde::de::Error::custom(
                "coordination event must be an object",
            ));
        };
        let event_id = match fields.remove("event_id") {
            Some(serde_json::Value::String(value)) => value,
            _ => return Err(serde::de::Error::custom("event_id must be a string")),
        };
        let action = serde_json::from_value(serde_json::Value::Object(fields))
            .map_err(serde::de::Error::custom)?;
        Ok(Self { event_id, action })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    AdoptPolicy {
        previous_policy_sha: String,
        policy_sha: String,
        decision_receipt: delivery::FileEvidence,
    },
    Register {
        session: String,
        lab: String,
        adapter: String,
        capabilities: Vec<String>,
    },
    Heartbeat {
        session: String,
        available: bool,
    },
    Claim {
        issue: u64,
        session: String,
        work_card: String,
        policy_sha: String,
        base_sha: String,
        paths: Vec<String>,
        dependencies: Vec<u64>,
        recovery_receipt: Option<String>,
    },
    Checkpoint {
        issue: u64,
        session: String,
        epoch: u64,
        packet: serde_json::Value,
    },
    Release {
        issue: u64,
        session: String,
        epoch: u64,
    },
    Reserve {
        issue: u64,
        session: String,
        epoch: u64,
        attempt: String,
        host: String,
        runner_root: PathBuf,
        spec: JobSpec,
    },
    FinishAttempt {
        issue: u64,
        session: String,
        epoch: u64,
        attempt: String,
        receipt: ExitEvidence,
    },
    /// Close a reservation that provably never produced compute. This is the
    /// only finalization path that does not require a live claim, so an orphaned
    /// reservation (for example a job cancelled in the queue after its
    /// workspace volume was lost) cannot deadlock policy adoption. It is
    /// deliberately narrower than `FinishAttempt`: the reserving session must
    /// still own the reservation, and the receipt must positively prove that no
    /// process ever started.
    Abandon {
        attempt: String,
        session: String,
        reason: String,
        receipt: ExitEvidence,
    },
    Complete {
        issue: u64,
        session: String,
        epoch: u64,
        delivery_receipt: CompletionEvidence,
    },
}

fn nonempty(value: &str) -> Result<()> {
    if value.trim().is_empty() || value.contains('\0') {
        return Err(invalid("empty or NUL identity"));
    }
    Ok(())
}

fn bounded_reason(value: &str) -> Result<()> {
    nonempty(value)?;
    if value.len() > REASON_MAX {
        return Err(invalid("reason must be non-empty and bounded"));
    }
    Ok(())
}

pub fn work_card_digest(value: &str) -> Result<()> {
    sha256(value.strip_prefix("sha256:").ok_or_else(|| {
        invalid("work_card must be a sha256 digest of retained immutable content")
    })?)
}

pub fn validate_policy_comparison(value: &serde_json::Value) -> Result<()> {
    if !matches!(value["status"].as_str(), Some("ahead" | "identical")) {
        return Err(invalid(
            "policy revision is not retained in protected main ancestry",
        ));
    }
    let files = value["files"]
        .as_array()
        .ok_or_else(|| invalid("policy comparison unavailable"))?;
    if files.len() >= 300
        || files.iter().any(|f| {
            [f["filename"].as_str(), f["previous_filename"].as_str()]
                .into_iter()
                .flatten()
                .any(|p| {
                    p == "AGENTS.md"
                        || p == "docs/integration/DECISIONS.md"
                        || p.starts_with("docs/integration/agent-execution-policy.")
                        || (p == "docs/labs/protocol.md" || p == "docs/labs/operations.md")
                })
        })
    {
        return Err(invalid("shared policy changed or comparison incomplete; hold admissions and perform reviewed policy migration"));
    }
    Ok(())
}

pub fn require_current_policy(repository: &str, policy_sha: &str) -> Result<()> {
    sha(policy_sha)?;
    validate_policy_comparison(&read_github_json(&format!(
        "repos/{repository}/compare/{policy_sha}...main"
    ))?)
}
fn safe_id(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
    {
        return Err(invalid("invalid event/attempt identity"));
    }
    Ok(())
}
fn sha(value: &str) -> Result<()> {
    if value.len() != 40 || !value.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(invalid("expected full source/policy SHA"));
    }
    Ok(())
}
fn sha256(value: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    {
        return Err(invalid("expected lowercase SHA-256"));
    }
    Ok(())
}
fn expiry(now: u64) -> Result<u64> {
    now.checked_add(TTL)
        .ok_or_else(|| invalid("lease time overflow"))
}
fn spec_digest(spec: &JobSpec) -> Result<String> {
    Ok(digest(&serde_json::to_vec(spec)?))
}

fn spec_binding(
    spec: &JobSpec,
    issue: u64,
    session: &str,
    epoch: u64,
    work_card: &str,
    attempt: &str,
) -> Result<()> {
    let claim = spec
        .coordination
        .as_ref()
        .ok_or_else(|| invalid("reserved spec lacks coordination identity"))?;
    if spec.id != attempt
        || claim.attempt_id != attempt
        || claim.issue != issue
        || claim.session != session
        || claim.epoch != epoch
        || claim.work_card != work_card
    {
        return Err(invalid(
            "reserved spec differs from task/session/epoch/work-card/attempt",
        ));
    }
    Ok(())
}

fn receipt_binding(attempt_id: &str, attempt: &Attempt, receipt: &ExitEvidence) -> Result<()> {
    safe_id(&receipt.job_id)?;
    sha256(&receipt.sha256)?;
    if receipt.host != attempt.host
        || receipt.attempt_id != attempt_id
        || receipt.job_id != attempt_id
    {
        return Err(invalid("exit evidence host/job/attempt mismatch"));
    }
    let root = attempt
        .runner_root
        .as_ref()
        .ok_or_else(|| invalid("legacy unbound attempt needs explicit reconciliation"))?;
    if !root.is_absolute()
        || !["exit.json", "reconciliation.json"]
            .iter()
            .any(|name| receipt.path == jobs::done_dir(root).join(attempt_id).join(name))
    {
        return Err(invalid(
            "exit evidence must name its reserved runner DONE receipt",
        ));
    }
    Ok(())
}
fn owned<'a>(
    state: &'a mut State,
    issue: u64,
    session: &str,
    epoch: u64,
    now: u64,
) -> Result<&'a mut Claim> {
    let claim = state
        .tasks
        .get_mut(&issue)
        .ok_or_else(|| invalid("task is unclaimed"))?;
    if claim.policy_sha != state.policy_sha
        || claim.session != session
        || claim.epoch != epoch
        || claim.expires <= now
        || claim.phase != "claimed"
    {
        return Err(invalid("stale or expired task generation"));
    }
    Ok(claim)
}

impl State {
    pub fn new(repository: String, policy_sha: String) -> Result<Self> {
        let components: Vec<_> = repository.split('/').collect();
        if components.len() != 2
            || components.iter().any(|s| {
                s.is_empty()
                    || !s
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.')
            })
        {
            return Err(invalid("repository must be owner/name"));
        }
        sha(&policy_sha)?;
        Ok(Self {
            schema: "uor-r4.lab-state/1".into(),
            repository,
            policy_sha,
            sequence: 0,
            observed_server_time: 0,
            labs: BTreeMap::new(),
            tasks: BTreeMap::new(),
            attempts: BTreeMap::new(),
        })
    }
    pub fn transition(&mut self, event: &Event, now: u64) -> Result<()> {
        safe_id(&event.event_id)?;
        if self.schema != "uor-r4.lab-state/1" || now < self.observed_server_time {
            return Err(invalid(
                "unsupported state or server clock regression; reconcile",
            ));
        }
        let mut next = self.clone();
        next.apply(&event.action, now)?;
        next.sequence = next
            .sequence
            .checked_add(1)
            .ok_or_else(|| invalid("sequence overflow"))?;
        next.observed_server_time = now;
        *self = next;
        Ok(())
    }
    fn apply(&mut self, action: &Action, now: u64) -> Result<()> {
        match action {
            Action::AdoptPolicy {
                previous_policy_sha,
                policy_sha,
                ..
            } => {
                sha(policy_sha)?;
                if previous_policy_sha != &self.policy_sha || previous_policy_sha == policy_sha {
                    return Err(invalid(
                        "policy adoption must replace the exact prior revision",
                    ));
                }
                if self
                    .tasks
                    .values()
                    .any(|c| c.phase == "claimed" && c.expires > now)
                    || self.attempts.values().any(|a| a.phase != "finalized")
                {
                    return Err(invalid("drain/release live claims and reconcile all attempts before policy adoption"));
                }
                self.policy_sha = policy_sha.clone();
            }
            Action::Register {
                session,
                lab,
                adapter,
                capabilities,
            } => {
                safe_id(session)?;
                nonempty(lab)?;
                nonempty(adapter)?;
                if self.labs.contains_key(session) {
                    return Err(invalid("session already registered"));
                }
                self.labs.insert(
                    session.clone(),
                    Lab {
                        lab: lab.clone(),
                        adapter: adapter.clone(),
                        capabilities: capabilities.clone(),
                        available: true,
                        heartbeat: now,
                    },
                );
            }
            Action::Heartbeat { session, available } => {
                let lab = self
                    .labs
                    .get_mut(session)
                    .ok_or_else(|| invalid("unknown session"))?;
                lab.heartbeat = now;
                lab.available = *available;
                // An expired lease cannot be revived by a delayed old session.
                if *available {
                    let expires = expiry(now)?;
                    for claim in self.tasks.values_mut().filter(|c| {
                        c.session == *session && c.phase == "claimed" && c.expires > now
                    }) {
                        claim.expires = expires;
                    }
                }
            }
            Action::Claim {
                issue,
                session,
                work_card,
                policy_sha,
                base_sha,
                paths,
                dependencies,
                recovery_receipt,
            } => {
                work_card_digest(work_card)?;
                if policy_sha != &self.policy_sha {
                    return Err(invalid(
                        "claim policy revision differs from shared authority",
                    ));
                }
                sha(base_sha)?;
                let lab = self
                    .labs
                    .get(session)
                    .ok_or_else(|| invalid("register the session first"))?;
                if !lab.available || expiry(lab.heartbeat)? <= now {
                    return Err(invalid("lab is unavailable"));
                }
                for dep in dependencies {
                    if !self.tasks.get(dep).is_some_and(|c| c.phase == "complete") {
                        return Err(invalid(format!("dependency {dep} incomplete")));
                    }
                }
                let epoch =
                    match self.tasks.get(issue) {
                        Some(old) if old.phase == "complete" => {
                            return Err(invalid("completed task needs an explicit successor issue"))
                        }
                        Some(old) if old.phase == "claimed" && old.expires > now => {
                            return Err(invalid("task already leased"))
                        }
                        Some(old) => {
                            nonempty(recovery_receipt.as_deref().ok_or_else(|| {
                                invalid("reacquisition needs a recovery receipt")
                            })?)?;
                            old.epoch
                                .checked_add(1)
                                .ok_or_else(|| invalid("epoch overflow"))?
                        }
                        None => 1,
                    };
                for path in paths {
                    if path.is_empty()
                        || path.starts_with('/')
                        || path.contains('\\')
                        || path
                            .split('/')
                            .any(|s| s.is_empty() || s == "." || s == "..")
                    {
                        return Err(invalid("ownership paths must be canonical repository-relative paths without empty, dot or parent components"));
                    }
                    for (other_issue, claim) in &self.tasks {
                        if other_issue != issue
                            && claim.phase == "claimed"
                            && claim.expires > now
                            && claim.paths.iter().any(|p| {
                                p == path
                                    || p.starts_with(&format!("{path}/"))
                                    || path.starts_with(&format!("{p}/"))
                            })
                        {
                            return Err(invalid(format!(
                                "source scope overlaps issue {other_issue}"
                            )));
                        }
                    }
                }
                let checkpoint = self.tasks.get(issue).and_then(|c| c.checkpoint.clone());
                self.tasks.insert(
                    *issue,
                    Claim {
                        session: session.clone(),
                        epoch,
                        work_card: work_card.clone(),
                        policy_sha: policy_sha.clone(),
                        base_sha: base_sha.clone(),
                        paths: paths.clone(),
                        expires: expiry(now)?,
                        phase: "claimed".into(),
                        checkpoint,
                    },
                );
            }
            Action::Checkpoint {
                issue,
                session,
                epoch,
                packet,
            } => {
                if !packet.is_object()
                    || packet
                        .get("next_action")
                        .and_then(|v| v.as_str())
                        .is_none_or(str::is_empty)
                {
                    return Err(invalid("handoff needs an object with next_action"));
                }
                owned(self, *issue, session, *epoch, now)?.checkpoint = Some(packet.clone());
            }
            Action::Release {
                issue,
                session,
                epoch,
            } => {
                let c = owned(self, *issue, session, *epoch, now)?;
                c.phase = "released".into();
                c.expires = now;
            }
            Action::Reserve {
                issue,
                session,
                epoch,
                attempt,
                host,
                runner_root,
                spec,
            } => {
                let work_card = owned(self, *issue, session, *epoch, now)?.work_card.clone();
                safe_id(attempt)?;
                nonempty(host)?;
                if !runner_root.is_absolute()
                    || runner_root
                        .components()
                        .any(|part| matches!(part, std::path::Component::ParentDir))
                {
                    return Err(invalid(
                        "runner root must be absolute without parent traversal",
                    ));
                }
                spec_binding(spec, *issue, session, *epoch, &work_card, attempt)?;
                if self.attempts.contains_key(attempt) {
                    return Err(invalid("attempt identity already exists"));
                }
                if self
                    .attempts
                    .values()
                    .any(|a| a.issue == *issue && a.phase != "finalized")
                {
                    return Err(invalid(
                        "prior attempt still reserved; adopt or reconcile it",
                    ));
                }
                let outstanding: Vec<_> = self
                    .attempts
                    .values()
                    .filter(|a| a.host == *host && a.phase != "finalized")
                    .collect();
                let mut reserved_specs = Vec::with_capacity(outstanding.len());
                for other in outstanding {
                    let other_spec = other.spec.as_ref().ok_or_else(|| {
                        invalid("legacy host reservation lacks an exact spec; reconcile it")
                    })?;
                    if Some(spec_digest(other_spec)?) != other.spec_sha256
                        || other.phase != "reserved"
                        || other.receipt.is_some()
                    {
                        return Err(invalid("unverified host reservation; reconcile it"));
                    }
                    if other.runner_root.as_ref() != Some(runner_root) {
                        return Err(invalid(
                            "concurrent host reservations require the same canonical runner root",
                        ));
                    }
                    other_spec.validate()?;
                    reserved_specs.push(other_spec.clone());
                }
                // Account every unresolved reservation, independent of lane or
                // job count. Local admission still requires all other reserved
                // attempts in the exact running set; this does not relax
                // validate_specs' omission fence for unknown or queued work.
                crate::admission::admit(&crate::admission::Load::of(&reserved_specs), spec)
                    .map_err(invalid)?;
                self.attempts.insert(
                    attempt.clone(),
                    Attempt {
                        issue: *issue,
                        session: session.clone(),
                        task_epoch: *epoch,
                        host: host.clone(),
                        phase: "reserved".into(),
                        receipt: None,
                        spec: Some(spec.clone()),
                        spec_sha256: Some(spec_digest(spec)?),
                        runner_root: Some(runner_root.clone()),
                    },
                );
            }
            Action::FinishAttempt {
                issue,
                session,
                epoch,
                attempt,
                receipt,
            } => {
                owned(self, *issue, session, *epoch, now)?;
                let a = self
                    .attempts
                    .get_mut(attempt)
                    .ok_or_else(|| invalid("unknown attempt"))?;
                if a.issue != *issue || a.phase != "reserved" {
                    return Err(invalid("attempt not reserved or wrong task"));
                }
                receipt_binding(attempt, a, receipt)?;
                // apply() verifies actual local DONE bytes and confirmed stop before
                // this pure state transition. New stewardship can reconcile old
                // execution, but cannot admit the stale task generation.
                a.phase = "finalized".into();
                a.receipt = Some(receipt.clone());
            }
            Action::Abandon {
                attempt,
                session,
                reason,
                receipt,
            } => {
                safe_id(attempt)?;
                bounded_reason(reason)?;
                let a = self
                    .attempts
                    .get_mut(attempt)
                    .ok_or_else(|| invalid("unknown attempt"))?;
                if a.phase != "reserved" {
                    return Err(invalid("attempt is not an open reservation"));
                }
                if a.session != *session {
                    return Err(invalid(
                        "only the reserving session may abandon its own reservation",
                    ));
                }
                receipt_binding(attempt, a, receipt)?;
                // Deliberately no owned() call: the task generation is released
                // or expired, and requiring a live claim here would recreate the
                // cycle this action exists to break (a claim needs the current
                // policy, policy adoption needs every attempt finalized). The
                // apply-path preflight verifies local never-started evidence
                // before this transition is ever committed; the reason stays
                // auditable in the durable event log.
                a.phase = "finalized".into();
                a.receipt = Some(receipt.clone());
            }
            Action::Complete {
                issue,
                session,
                epoch,
                delivery_receipt,
            } => {
                completion_shape(delivery_receipt)?;
                if self
                    .attempts
                    .values()
                    .any(|a| a.issue == *issue && a.phase != "finalized")
                {
                    return Err(invalid("task still has an unresolved attempt"));
                }
                let c = owned(self, *issue, session, *epoch, now)?;
                c.phase = "complete".into();
                c.checkpoint = Some(serde_json::json!({"delivery_receipt":delivery_receipt}));
            }
        }
        Ok(())
    }
}

fn git(repo: &Path, args: &[&str], input: Option<&[u8]>, index: Option<&Path>) -> Result<String> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repo)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(path) = index {
        command.env("GIT_INDEX_FILE", path);
    }
    let mut child = command.spawn()?;
    if let Some(bytes) = input {
        child
            .stdin
            .take()
            .ok_or_else(|| invalid("git stdin unavailable"))?
            .write_all(bytes)?;
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(invalid(format!(
            "git {}: {}",
            args.first().unwrap_or(&""),
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().into())
}
fn fetch(repo: &Path) -> Result<String> {
    if git(repo, &["config", "--get", "uor.coordination"], None, None)? != "1" {
        return Err(invalid("not a dedicated coordination store"));
    }
    git(
        repo,
        &[
            "fetch",
            "--quiet",
            "origin",
            "refs/heads/codex/lab-state:refs/remotes/origin/codex/lab-state",
        ],
        None,
        None,
    )?;
    git(repo, &["rev-parse", REMOTE_REF], None, None)
}
fn state_at(repo: &Path, head: &str) -> Result<State> {
    Ok(serde_json::from_str(&git(
        repo,
        &["show", &format!("{head}:state.json")],
        None,
        None,
    )?)?)
}
fn commit(
    repo: &Path,
    parent: Option<&str>,
    state: &State,
    event: Option<&Event>,
    now: u64,
) -> Result<String> {
    let index = repo.join(format!(
        "uor-index-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let result = (|| {
        git(
            repo,
            &["read-tree", parent.unwrap_or("--empty")],
            None,
            Some(&index),
        )?;
        let blob = git(
            repo,
            &["hash-object", "-w", "--stdin"],
            Some(&serde_json::to_vec_pretty(state)?),
            None,
        )?;
        git(
            repo,
            &[
                "update-index",
                "--add",
                "--cacheinfo",
                &format!("100644,{blob},state.json"),
            ],
            None,
            Some(&index),
        )?;
        if let Some(e) = event {
            let bytes = serde_json::to_vec_pretty(
                &serde_json::json!({"event":e,"observed_server_time":now,"previous_head":parent}),
            )?;
            let blob = git(repo, &["hash-object", "-w", "--stdin"], Some(&bytes), None)?;
            git(
                repo,
                &[
                    "update-index",
                    "--add",
                    "--cacheinfo",
                    &format!("100644,{blob},events/{}.json", e.event_id),
                ],
                None,
                Some(&index),
            )?;
        }
        let tree = git(repo, &["write-tree"], None, Some(&index))?;
        let mut args = vec!["commit-tree", tree.as_str()];
        if let Some(p) = parent {
            args.extend(["-p", p]);
        }
        let message = format!(
            "lab-state: {}\n",
            event.map(|e| e.event_id.as_str()).unwrap_or("genesis")
        );
        git(repo, &args, Some(message.as_bytes()), None)
    })();
    let _ = fs::remove_file(index);
    result
}

pub fn initialize(repo: &Path, remote: &str, repository: &str, policy: &str) -> Result<String> {
    let state = State::new(repository.into(), policy.into())?;
    if repo.exists() {
        return Err(invalid(
            "coordination directory already exists; use status/apply",
        ));
    }
    fs::create_dir_all(repo)?;
    git(repo, &["init", "--bare", "--quiet"], None, None)?;
    git(repo, &["config", "uor.coordination", "1"], None, None)?;
    git(repo, &["remote", "add", "origin", remote], None, None)?;
    let existing = git(
        repo,
        &["ls-remote", "origin", "refs/heads/codex/lab-state"],
        None,
        None,
    )?;
    if !existing.is_empty() {
        let head = fetch(repo)?;
        let observed = state_at(repo, &head)?;
        if observed.repository != repository || observed.policy_sha != policy {
            return Err(invalid(
                "existing operational repository/policy differs; inspect it before joining",
            ));
        }
        return Ok(head);
    }
    let head = commit(repo, None, &state, None, 0)?;
    git(
        repo,
        &["push", "origin", &format!("{head}:refs/heads/{BRANCH}")],
        None,
        None,
    )?;
    fetch(repo)
}

pub fn status(repo: &Path) -> Result<State> {
    let head = fetch(repo)?;
    state_at(repo, &head)
}

// RFC 7231 Date is obtained from the actual GitHub response, never from an agent clock.
fn parse_http_date(value: &str) -> Result<u64> {
    let p: Vec<_> = value.split_whitespace().collect();
    if p.len() != 6 || p[5] != "GMT" {
        return Err(invalid("unrecognized GitHub Date"));
    }
    let day = p[1]
        .parse::<i64>()
        .map_err(|_| invalid("invalid date day"))?;
    let month = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ]
    .iter()
    .position(|s| *s == p[2])
    .ok_or_else(|| invalid("invalid month"))? as i64
        + 1;
    let year = p[3].parse::<i64>().map_err(|_| invalid("invalid year"))?;
    let time: Vec<_> = p[4]
        .split(':')
        .map(str::parse::<i64>)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_| invalid("invalid time"))?;
    if time.len() != 3 {
        return Err(invalid("invalid time"));
    }
    let y = year - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let days = era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719468;
    u64::try_from(days * 86400 + time[0] * 3600 + time[1] * 60 + time[2])
        .map_err(|_| invalid("invalid epoch"))
}
pub fn github_time(repository: &str) -> Result<u64> {
    let out = Command::new("gh")
        .args([
            "api",
            "--include",
            "-H",
            "Cache-Control: no-cache",
            &format!("repos/{repository}"),
        ])
        .output()?;
    if !out.status.success() {
        return Err(invalid(
            "GitHub unavailable; no new ownership or admissions",
        ));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let date = text
        .lines()
        .find_map(|line| {
            line.split_once(':')
                .filter(|(key, _)| key.eq_ignore_ascii_case("date"))
                .map(|(_, v)| v.trim())
        })
        .ok_or_else(|| invalid("GitHub response lacks Date"))?;
    parse_http_date(date)
}

/// Native GitHub dependencies are authoritative even if a caller omits the
/// optional local dependency IDs from a claim. Truncated or unsupported API
/// responses deny new work rather than pretending that the issue is unblocked.
fn validate_native_blockers(value: &serde_json::Value) -> Result<()> {
    let issue = &value["data"]["repository"]["issue"];
    if value.get("errors").is_some()
        || issue["state"] != "OPEN"
        || issue["blockedBy"]["pageInfo"]["hasNextPage"] != false
    {
        return Err(invalid(
            "owning issue or complete native blocker state unavailable",
        ));
    }
    let blockers = issue["blockedBy"]["nodes"]
        .as_array()
        .ok_or_else(|| invalid("native blocked_by state unavailable"))?;
    if blockers
        .iter()
        .any(|row| row["number"].as_u64().is_none() || row["state"] != "CLOSED")
    {
        return Err(invalid(
            "owning issue has an open or unverified native blocker",
        ));
    }
    Ok(())
}

fn require_unblocked(repository: &str, issue: u64) -> Result<()> {
    if issue == 0 {
        return Err(invalid("issue must be positive"));
    }
    let (owner, name) = repository
        .split_once('/')
        .ok_or_else(|| invalid("repository identity malformed"))?;
    let query = format!("query {{ repository(owner: {}, name: {}) {{ issue(number: {}) {{ state blockedBy(first:100) {{ nodes {{ number state }} pageInfo {{ hasNextPage }} }} }} }} }}", serde_json::to_string(owner)?, serde_json::to_string(name)?, issue);
    let response = Command::new("gh")
        .args(["api", "graphql", "-f", &format!("query={query}")])
        .output()?;
    if !response.status.success() {
        return Err(invalid(
            "GitHub blocked_by observation unavailable; no new claim or admission",
        ));
    }
    validate_native_blockers(&serde_json::from_slice(&response.stdout)?)
}

fn validate_exit_identity(
    exit: &ConfirmedExit,
    attempt_id: &str,
    attempt: &Attempt,
    mode: EvidenceMode,
    durable: Option<&JobSpec>,
) -> Result<()> {
    match mode {
        EvidenceMode::Finalize => {
            let spec = attempt.spec.as_ref().ok_or_else(|| {
                invalid("unbound historical attempt cannot be finalized automatically")
            })?;
            let expected = attempt
                .spec_sha256
                .as_deref()
                .ok_or_else(|| invalid("attempt lacks spec digest"))?;
            let claim = spec
                .coordination
                .as_ref()
                .ok_or_else(|| invalid("reserved spec lacks claim"))?;
            if exit.schema != jobs::EXIT_SCHEMA
                || exit.id != spec.id
                || exit.attempt_id != attempt_id
                || exit.host != attempt.host
                || exit.spec_sha256 != expected
                || spec_digest(spec)? != expected
                || exit.coordination.issue != attempt.issue
                || exit.coordination.session != attempt.session
                || exit.coordination.epoch != attempt.task_epoch
                || exit.coordination.work_card != claim.work_card
                || exit.coordination.attempt_id != attempt_id
            {
                return Err(invalid(
                    "exit receipt differs from reserved execution identity",
                ));
            }
        }
        EvidenceMode::Abandon => {
            // The reserved digest is deliberately not re-bound here. Digest and
            // work-card drift between the reservation and the durable queued
            // spec is exactly the orphan condition `Abandon` exists to close, so
            // the durable file is bound by job identity instead: it is this
            // attempt, and the receipt must bind its exact durable bytes (its
            // spec digest and its full coordination claim). This admits no
            // compute: the caller separately requires never-started evidence.
            let durable = durable.ok_or_else(|| invalid("abandon lacks durable job identity"))?;
            let claim = durable
                .coordination
                .as_ref()
                .ok_or_else(|| invalid("durable spec lacks coordination identity"))?;
            if exit.schema != jobs::EXIT_SCHEMA
                || exit.id != durable.id
                || durable.id != attempt_id
                || exit.attempt_id != attempt_id
                || exit.host != attempt.host
                || exit.spec_sha256 != spec_digest(durable)?
                || claim.attempt_id != attempt_id
                || exit.coordination.issue != attempt.issue
                || exit.coordination.session != attempt.session
                || exit.coordination.epoch != attempt.task_epoch
                || exit.coordination.attempt_id != attempt_id
                || serde_json::to_value(&exit.coordination)? != serde_json::to_value(claim)?
            {
                return Err(invalid(
                    "abandon receipt does not bind the durable job identity",
                ));
            }
        }
    }
    Ok(())
}

fn validate_exit_contents(
    exit: &ConfirmedExit,
    attempt_id: &str,
    attempt: &Attempt,
    mode: EvidenceMode,
    durable: Option<&JobSpec>,
) -> Result<()> {
    validate_exit_identity(exit, attempt_id, attempt, mode, durable)?;
    let never_started = exit.process_state == "never_started"
        && exit.outcome == "cancelled"
        && exit.exit_status.is_none()
        && exit.elapsed_ms == 0;
    if exit.process_state != "confirmed_stopped" && !never_started {
        return Err(invalid("worker stop is unknown; retain its reservation"));
    }
    let terminal = match exit.outcome.as_str() {
        "completed" => exit.exit_status == Some(0),
        "failed" => exit.exit_status.is_some(),
        "wall_killed" | "criterion_killed" | "resource_killed" | "cancelled" | "interrupted" => {
            true
        }
        _ => false,
    };
    if !terminal {
        return Err(invalid(
            "unknown or inconsistent execution outcome; retain reservation",
        ));
    }
    // Deserialization requires the measured elapsed field; zero is valid for a
    // very short process, unlike an omitted/defaulted duration.
    let _measured_elapsed_ms = exit.elapsed_ms;
    Ok(())
}

/// An abandon may never close a reservation that may have produced compute. The
/// stopped-reconciliation mechanisms exist for those cases and stay available to
/// `FinishAttempt`; here the receipt must positively say `never_started`.
fn require_never_started(mode: EvidenceMode, exit: &ConfirmedExit) -> Result<()> {
    if mode == EvidenceMode::Abandon && exit.process_state != "never_started" {
        return Err(invalid(
            "abandon requires never-started evidence; a reservation that may have produced compute needs reviewed reconciliation",
        ));
    }
    Ok(())
}

fn verify_exit_evidence(
    attempt_id: &str,
    attempt: &Attempt,
    evidence: &ExitEvidence,
    mode: EvidenceMode,
) -> Result<()> {
    receipt_binding(attempt_id, attempt, evidence)?;
    if evidence.host != process::host_id()? {
        return Err(invalid(
            "exit host is not reachable as this local host; retain reservation",
        ));
    }
    let root = attempt
        .runner_root
        .as_ref()
        .ok_or_else(|| invalid("missing runner root"))?;
    let canonical_root = fs::canonicalize(root)?;
    let _job_guard = jobs::job_lock(&canonical_root, attempt_id)?;
    let name = evidence
        .path
        .file_name()
        .ok_or_else(|| invalid("receipt filename missing"))?;
    if canonical_root.starts_with("/Volumes")
        || fs::canonicalize(&evidence.path)?
            != jobs::done_dir(&canonical_root).join(attempt_id).join(name)
    {
        return Err(invalid(
            "exit receipt escapes its internal durable runner root",
        ));
    }
    let metadata = fs::symlink_metadata(&evidence.path)?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() == 0
        || metadata.len() > 1024 * 1024
    {
        return Err(invalid(
            "exit evidence must be a regular bounded nonempty file",
        ));
    }
    let bytes = fs::read(&evidence.path)?;
    if digest(&bytes) != evidence.sha256 {
        return Err(invalid("exit evidence SHA-256 mismatch"));
    }
    let dir = evidence
        .path
        .parent()
        .ok_or_else(|| invalid("exit path has no directory"))?;
    let saved = jobs::read_spec(dir)?;
    let durable = match mode {
        EvidenceMode::Finalize => {
            if Some(spec_digest(&saved)?) != attempt.spec_sha256 {
                return Err(invalid("durable job spec differs from reserved spec"));
            }
            None
        }
        EvidenceMode::Abandon => {
            // A digest mismatch between the reservation and the durable queued
            // spec is precisely the orphan condition this mode closes, so bind
            // the durable job identity (its id and coordinated attempt) instead
            // of the reserved digest. The receipt must still bind these exact
            // durable bytes; validate_exit_identity checks that below.
            if saved.id != attempt_id
                || saved
                    .coordination
                    .as_ref()
                    .map(|claim| claim.attempt_id.as_str())
                    != Some(attempt_id)
            {
                return Err(invalid(
                    "durable job spec identity differs from the abandoned attempt",
                ));
            }
            Some(&saved)
        }
    };
    let exit: ConfirmedExit;
    if name == "reconciliation.json" {
        let proof: serde_json::Value = serde_json::from_slice(&bytes)?;
        let original_path = dir.join("exit.json");
        let original_meta = fs::symlink_metadata(&original_path)?;
        if !original_meta.file_type().is_file()
            || original_meta.len() == 0
            || original_meta.len() > 1024 * 1024
        {
            return Err(invalid(
                "original UNKNOWN receipt must be a bounded regular file",
            ));
        }
        let original = fs::read(original_path)?;
        exit = serde_json::from_slice(&original)?;
        require_never_started(mode, &exit)?;
        validate_exit_identity(&exit, attempt_id, attempt, mode, durable)?;
        if proof["schema"] != "uor-r4.stopped-reconciliation/1"
            || proof["id"] != attempt_id
            || proof["attempt_id"] != attempt_id
            || proof["host"] != attempt.host
            || proof["spec_sha256"].as_str() != attempt.spec_sha256.as_deref()
            || proof["coordination"] != serde_json::to_value(&exit.coordination)?
            || proof["original_exit_sha256"] != digest(&original)
            || proof["scientific_outcome"] != "unknown"
            || proof["outcome"] != "interrupted"
            || exit.outcome != "unknown"
            || proof["additional_charged_ms"].as_u64().is_none()
        {
            return Err(invalid(
                "stopped reconciliation does not bind the preserved UNKNOWN result",
            ));
        }
        let local_attempt = jobs::read_attempt(dir)?;
        if local_attempt.attempt_id != attempt_id {
            return Err(invalid("durable attempt identity differs"));
        }
        match proof["evidence_kind"].as_str() {
            Some("pre_spawn_failure") => {
                // The immutable producer marker is positive control-flow
                // evidence. Missing process/gate files alone prove nothing.
                let marker =
                    jobs::validate_pre_spawn_failure(&canonical_root, dir, &saved, &local_attempt)?;
                let marker_hash = digest(&fs::read(dir.join("preflight-failure.json"))?);
                let original_value: serde_json::Value = serde_json::from_slice(&original)?;
                if proof["process_state"] != "confirmed_not_started"
                    || proof["additional_charged_ms"].as_u64() != Some(0)
                    || proof["measurement"] != "measured"
                    || original_value["measurement"] != "measured"
                    || exit.elapsed_ms < marker.elapsed_ms
                    || !proof["process_identity_sha256"].is_null()
                    || proof["pre_spawn_failure_sha256"] != marker_hash
                    || jobs::queue_dir(&canonical_root).join(attempt_id).exists()
                    || jobs::running_dir(&canonical_root).join(attempt_id).exists()
                {
                    return Err(invalid("pre-spawn reconciliation conflicts with positive proof, cost or launch state"));
                }
                return Ok(());
            }
            Some("owned_process") => {
                if proof["process_identity_sha256"] != digest(&fs::read(dir.join("process.json"))?)
                    || proof["process_state"] != "confirmed_stopped"
                    || !proof["pre_spawn_failure_sha256"].is_null()
                {
                    return Err(invalid(
                        "owned-process reconciliation lacks its exact process identity",
                    ));
                }
                // Continue to revalidate the token and absence of live members.
            }
            _ => {
                return Err(invalid(
                    "reconciliation lacks a supported positive evidence kind",
                ))
            }
        }
    } else {
        exit = serde_json::from_slice(&bytes)?;
        require_never_started(mode, &exit)?;
        validate_exit_contents(&exit, attempt_id, attempt, mode, durable)?;
    }
    if exit.process_state == "never_started" {
        // A queued cancellation can release its reservation only while the
        // same launch lock is held and no durable launch intent ever existed.
        if jobs::queue_dir(&canonical_root).join(attempt_id).exists()
            || jobs::running_dir(&canonical_root).join(attempt_id).exists()
            || jobs::consumed_attempt_path(attempt_id)?.exists()
            || dir.join("attempt.json").exists()
            || dir.join("process.json").exists()
            || !dir.join("cancel.json").is_file()
        {
            return Err(invalid(
                "never-started evidence conflicts with launch state; retain reservation",
            ));
        }
        return Ok(());
    }
    let local_attempt = jobs::read_attempt(dir)?;
    if local_attempt.attempt_id != attempt_id {
        return Err(invalid("durable attempt identity differs"));
    }
    let identity = jobs::read_identity(dir)?;
    if identity.token != local_attempt.process_token
        || !process::owned_members(&identity)?.is_empty()
    {
        return Err(invalid(
            "worker is live or its stopped identity is unverified; retain reservation",
        ));
    }
    Ok(())
}

fn completion_shape(evidence: &CompletionEvidence) -> Result<()> {
    sha(&evidence.merge_sha)?;
    for file in [
        delivery::FileEvidence {
            path: evidence.path.clone(),
            sha256: evidence.sha256.clone(),
        },
        evidence.acceptance.clone(),
    ] {
        if !file.path.is_absolute()
            || file.sha256.len() != 64
            || !file
                .sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err(invalid(
                "completion needs absolute evidence paths and SHA-256 identities",
            ));
        }
    }
    Ok(())
}

fn read_bound_evidence(path: &Path, expected: &str) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if !path.is_absolute()
        || !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() == 0
        || metadata.len() > 8 * 1024 * 1024
    {
        return Err(invalid(
            "completion evidence must be a nonempty regular file at most 8 MiB",
        ));
    }
    let bytes = fs::read(path)?;
    if digest(&bytes) != expected {
        return Err(invalid("completion evidence digest changed"));
    }
    Ok(bytes)
}

fn validate_completion_binding(
    state: &State,
    issue: u64,
    session: &str,
    epoch: u64,
    evidence: &CompletionEvidence,
    receipt: &delivery::DeliveryReceipt,
    acceptance: &CompletionAcceptance,
) -> Result<()> {
    let claim = state
        .tasks
        .get(&issue)
        .ok_or_else(|| invalid("unknown completed task"))?;
    if receipt.repository != state.repository
        || receipt.task_issue != issue
        || receipt.claim_session != session
        || receipt.claim_epoch != epoch
        || receipt.work_card != claim.work_card
    {
        return Err(invalid(
            "completion delivery does not match this task generation and work card",
        ));
    }
    if acceptance.schema != "uor-r4.task-acceptance/1"
        || acceptance.task_issue != issue
        || acceptance.work_card != claim.work_card
        || acceptance.head_sha != receipt.head_sha
        || acceptance.merge_sha != evidence.merge_sha
        || acceptance.decision != "APPROVE"
        || !acceptance.full_scope
        || acceptance.acceptance_criteria.is_empty()
        || acceptance
            .acceptance_criteria
            .iter()
            .any(|c| c.trim().is_empty())
        || acceptance.reviewer_session == receipt.author_session
        || !receipt
            .reviews
            .iter()
            .any(|r| r.reviewer_session == acceptance.reviewer_session)
    {
        return Err(invalid(
            "completion requires independent explicit full-issue acceptance at the merged identity",
        ));
    }
    Ok(())
}

fn validate_merged_identity(
    evidence: &CompletionEvidence,
    receipt: &delivery::DeliveryReceipt,
    pr: &serde_json::Value,
    comparison: &serde_json::Value,
) -> Result<()> {
    if pr["number"].as_u64() != Some(receipt.pull_request)
        || pr["merged"] != true
        || pr["head"]["sha"] != receipt.head_sha
        || pr["merge_commit_sha"] != evidence.merge_sha
        || pr["base"]["ref"] != "main"
        || pr["base"]["repo"]["full_name"] != receipt.repository
        || !matches!(comparison["status"].as_str(), Some("ahead" | "identical"))
        || comparison["base_commit"]["sha"] != evidence.merge_sha
        || comparison["merge_base_commit"]["sha"] != evidence.merge_sha
    {
        return Err(invalid(
            "completion requires the exact merged PR retained in current main ancestry",
        ));
    }
    Ok(())
}

fn read_github_json(endpoint: &str) -> Result<serde_json::Value> {
    let output = Command::new("gh")
        .args(["api", "-H", "Cache-Control: no-cache", endpoint])
        .output()?;
    if !output.status.success() {
        return Err(invalid("GitHub completion observation unavailable"));
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn verify_completion(
    state: &State,
    issue: u64,
    session: &str,
    epoch: u64,
    evidence: &CompletionEvidence,
) -> Result<()> {
    completion_shape(evidence)?;
    let receipt: delivery::DeliveryReceipt =
        serde_json::from_slice(&read_bound_evidence(&evidence.path, &evidence.sha256)?)?;
    // Re-read every named check/review/result log; a path string is never delivery proof.
    delivery::validate(&receipt)?;
    let acceptance: CompletionAcceptance = serde_json::from_slice(&read_bound_evidence(
        &evidence.acceptance.path,
        &evidence.acceptance.sha256,
    )?)?;
    validate_completion_binding(
        state,
        issue,
        session,
        epoch,
        evidence,
        &receipt,
        &acceptance,
    )?;
    let pr = read_github_json(&format!(
        "repos/{}/pulls/{}",
        receipt.repository, receipt.pull_request
    ))?;
    let comparison = read_github_json(&format!(
        "repos/{}/compare/{}...main",
        receipt.repository, evidence.merge_sha
    ))?;
    validate_merged_identity(evidence, &receipt, &pr, &comparison)
}

fn preflight_event(state: &State, event: &Event) -> Result<()> {
    match &event.action {
        Action::AdoptPolicy {
            policy_sha,
            decision_receipt,
            ..
        } => {
            let receipt: delivery::DeliveryReceipt = serde_json::from_slice(&read_bound_evidence(
                &decision_receipt.path,
                &decision_receipt.sha256,
            )?)?;
            delivery::validate(&receipt)?;
            if receipt.class != "C" || receipt.repository != state.repository {
                return Err(invalid(
                    "policy adoption requires reviewed council delivery for this repository",
                ));
            }
            let pr = read_github_json(&format!(
                "repos/{}/pulls/{}",
                state.repository, receipt.pull_request
            ))?;
            if pr["merged"] != true
                || pr["head"]["sha"] != receipt.head_sha
                || pr["merge_commit_sha"] != *policy_sha
                || pr["base"]["ref"] != "main"
                || pr["base"]["repo"]["full_name"] != state.repository
            {
                return Err(invalid(
                    "new policy must bind the exact council-reviewed protected merge",
                ));
            }
            require_current_policy(&state.repository, policy_sha)
        }
        Action::Claim { issue, .. } => {
            require_current_policy(&state.repository, &state.policy_sha)?;
            require_unblocked(&state.repository, *issue)
        }
        Action::Reserve {
            issue,
            host,
            runner_root,
            spec,
            ..
        } => {
            require_current_policy(&state.repository, &state.policy_sha)?;
            require_unblocked(&state.repository, *issue)?;
            if host != &process::host_id()? {
                return Err(invalid("reserve only on the observed local host"));
            }
            let canonical_root = fs::canonicalize(runner_root)?;
            if !runner_root.is_absolute()
                || canonical_root.starts_with("/Volumes")
                || canonical_root != *runner_root
            {
                return Err(invalid(
                    "runner root must be a canonical verified internal directory",
                ));
            }
            spec.validate()
        }
        Action::FinishAttempt {
            attempt, receipt, ..
        } => {
            let reserved = state
                .attempts
                .get(attempt)
                .ok_or_else(|| invalid("unknown attempt"))?;
            verify_exit_evidence(attempt, reserved, receipt, EvidenceMode::Finalize)
        }
        Action::Abandon {
            attempt, receipt, ..
        } => {
            // No policy guard here: an orphaned reservation is what blocks
            // policy adoption, and adoption is what repairs a failing policy
            // comparison, so requiring the current policy would rebuild the
            // deadlock. The verifier still requires this host's own
            // never-started receipt for exactly this reservation.
            let reserved = state
                .attempts
                .get(attempt)
                .ok_or_else(|| invalid("unknown attempt"))?;
            verify_exit_evidence(attempt, reserved, receipt, EvidenceMode::Abandon)
        }
        Action::Complete {
            issue,
            session,
            epoch,
            delivery_receipt,
        } => verify_completion(state, *issue, session, *epoch, delivery_receipt),
        _ => Ok(()),
    }
}

fn replayed_event(repo: &Path, head: &str, event: &Event) -> Result<bool> {
    let path = format!("{head}:events/{}.json", event.event_id);
    // `cat-file -e` distinguishes absence; malformed existing content is an
    // error and must not be overwritten under the same event identity.
    if git(repo, &["cat-file", "-e", &path], None, None).is_err() {
        return Ok(false);
    }
    let value: serde_json::Value = serde_json::from_str(&git(repo, &["show", &path], None, None)?)?;
    if value.get("event") == Some(&serde_json::to_value(event)?) {
        return Ok(true);
    }
    Err(invalid("event ID already binds different contents"))
}

fn reconcile_push(repo: &Path, event: &Event, pushed: Result<String>) -> Result<String> {
    match pushed {
        Ok(head) => Ok(head),
        Err(error) => {
            let actual = fetch(repo)?;
            if replayed_event(repo, &actual, event)? {
                return Ok(actual);
            }
            Err(error)
        }
    }
}

fn publish(repo: &Path, next: &str, event: &Event) -> Result<String> {
    let pushed = git(
        repo,
        &["push", "origin", &format!("{next}:refs/heads/{BRANCH}")],
        None,
        None,
    )
    .map(|_| next.to_string());
    reconcile_push(repo, event, pushed)
}

pub fn apply(repo: &Path, event: &Event) -> Result<String> {
    safe_id(&event.event_id)?;
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(repo.join("coord.lock"))?;
    lock.lock()?;
    let head = fetch(repo)?;
    if replayed_event(repo, &head, event)? {
        return Ok(head);
    }
    let mut state = state_at(repo, &head)?;
    let now = github_time(&state.repository)?;
    preflight_event(&state, event)?;
    state.transition(event, now)?;
    let next = commit(repo, Some(&head), &state, Some(event), now)?;
    publish(repo, &next, event)
}

pub fn validate_admission(
    repo: &Path,
    issue: u64,
    session: &str,
    epoch: u64,
    work_card: &str,
) -> Result<()> {
    let mut state = status(repo)?;
    let now = github_time(&state.repository)?;
    let c = owned(&mut state, issue, session, epoch, now)?;
    if c.work_card != work_card {
        return Err(invalid("work-card version changed"));
    }
    require_unblocked(&state.repository, issue)?;
    Ok(())
}

fn validate_reserved_identity(
    state: &mut State,
    issue: u64,
    session: &str,
    epoch: u64,
    work_card: &str,
    attempt_id: &str,
    host: &str,
    now: u64,
) -> Result<()> {
    safe_id(attempt_id)?;
    let lab = state
        .labs
        .get(session)
        .ok_or_else(|| invalid("unknown lab session"))?;
    if !lab.available || expiry(lab.heartbeat)? <= now {
        return Err(invalid("lab unavailable; no new admission"));
    }
    if owned(state, issue, session, epoch, now)?.work_card != work_card {
        return Err(invalid("work-card version changed"));
    }
    let attempt = state
        .attempts
        .get(attempt_id)
        .ok_or_else(|| invalid("job has no reserved attempt"))?;
    if attempt.phase != "reserved"
        || attempt.issue != issue
        || attempt.session != session
        || attempt.task_epoch != epoch
        || attempt.host != host
        || attempt.receipt.is_some()
    {
        return Err(invalid(
            "reserved attempt host/task/session/generation is stale or finalized",
        ));
    }
    let spec = attempt.spec.as_ref().ok_or_else(|| {
        invalid("legacy reservation lacks an exact spec; reconcile before admission")
    })?;
    spec_binding(spec, issue, session, epoch, work_card, attempt_id)?;
    if Some(spec_digest(spec)?) != attempt.spec_sha256 {
        return Err(invalid("reserved spec digest mismatch"));
    }
    Ok(())
}

pub fn validate_job_admission(
    repo: &Path,
    issue: u64,
    session: &str,
    epoch: u64,
    work_card: &str,
    attempt_id: &str,
) -> Result<()> {
    let mut state = status(repo)?;
    let now = github_time(&state.repository)?;
    validate_reserved_identity(
        &mut state,
        issue,
        session,
        epoch,
        work_card,
        attempt_id,
        &process::host_id()?,
        now,
    )?;
    require_unblocked(&state.repository, issue)
}

pub(crate) fn validate_specs(
    state: &State,
    spec: &JobSpec,
    running: &[JobSpec],
    host: &str,
) -> Result<()> {
    let claim = spec
        .coordination
        .as_ref()
        .ok_or_else(|| invalid("missing coordinated spec"))?;
    let candidate = state
        .attempts
        .get(&claim.attempt_id)
        .ok_or_else(|| invalid("unreserved spec"))?;
    if Some(spec_digest(spec)?) != candidate.spec_sha256 {
        return Err(invalid("queued spec changed after reservation"));
    }
    // Resource bounds are locally enforced; cooperative reservations may have
    // come from another queue or an offline steward. Do not silently omit them
    // from local admission accounting. Hold until they are adopted here or
    // finalized with observed stop evidence.
    for (id, attempt) in &state.attempts {
        if attempt.host == host && attempt.phase != "finalized" && id != &claim.attempt_id {
            let local = running.iter().find(|job| {
                job.coordination
                    .as_ref()
                    .is_some_and(|c| &c.attempt_id == id)
            });
            let Some(local) = local else {
                return Err(invalid(format!("unresolved host reservation {id} absent from local running set; reconcile before admission")));
            };
            if Some(spec_digest(local)?) != attempt.spec_sha256 {
                return Err(invalid("running spec differs from host reservation"));
            }
        }
    }
    Ok(())
}

fn validate_runner_root(attempt: &Attempt, runner_root: &Path) -> Result<()> {
    let expected = attempt
        .runner_root
        .as_ref()
        .ok_or_else(|| invalid("missing reserved runner root"))?;
    if fs::canonicalize(expected)? != fs::canonicalize(runner_root)? {
        return Err(invalid(
            "active runner root differs from reserved runner root",
        ));
    }
    Ok(())
}

pub fn validate_reserved_spec(
    repo: &Path,
    runner_root: &Path,
    spec: &JobSpec,
    running: &[JobSpec],
) -> Result<()> {
    let mut state = status(repo)?;
    require_current_policy(&state.repository, &state.policy_sha)?;
    let now = github_time(&state.repository)?;
    let host = process::host_id()?;
    let claim = spec
        .coordination
        .as_ref()
        .ok_or_else(|| invalid("missing coordinated spec"))?;
    validate_reserved_identity(
        &mut state,
        claim.issue,
        &claim.session,
        claim.epoch,
        &claim.work_card,
        &claim.attempt_id,
        &host,
        now,
    )?;
    validate_specs(&state, spec, running, &host)?;
    validate_runner_root(
        state
            .attempts
            .get(&claim.attempt_id)
            .ok_or_else(|| invalid("missing attempt"))?,
        runner_root,
    )?;
    require_unblocked(&state.repository, claim.issue)
}
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(id: &str, action: Action) -> Event {
        Event {
            event_id: id.into(),
            action,
        }
    }
    fn register(s: &str) -> Event {
        event(
            s,
            Action::Register {
                session: s.into(),
                lab: s.into(),
                adapter: "manual".into(),
                capabilities: vec![],
            },
        )
    }
    fn claim(s: &str, recovery: Option<String>) -> Event {
        event(
            "claim",
            Action::Claim {
                issue: 1,
                session: s.into(),
                work_card:
                    "sha256:1111111111111111111111111111111111111111111111111111111111111111".into(),
                policy_sha: "b".repeat(40),
                base_sha: "a".repeat(40),
                paths: vec!["src/model".into()],
                dependencies: vec![],
                recovery_receipt: recovery,
            },
        )
    }
    fn state() -> State {
        State::new("UOR-Foundation/uor-r4".into(), "b".repeat(40)).unwrap()
    }
    fn spec(session: &str, epoch: u64, id: &str) -> JobSpec {
        JobSpec::parse(&serde_json::to_vec(&serde_json::json!({
            "id":id,"lab":session,"cwd":std::env::temp_dir(),"argv":["/bin/true"],
            "threads":1,"rss_gib":0.5,"wall_s":10,"kill_criterion":{"kind":"wall"},
            "storage":[{"volume":"internal","additional_bytes":1024,"checkpoint_bytes":1024}],
            "coordination":{"issue":1,"session":session,"epoch":epoch,"work_card":"sha256:1111111111111111111111111111111111111111111111111111111111111111","attempt_id":id}
        })).unwrap()).unwrap()
    }
    fn reserve(session: &str, epoch: u64, id: &str) -> Event {
        event(
            id,
            Action::Reserve {
                issue: 1,
                session: session.into(),
                epoch,
                attempt: id.into(),
                host: "mac".into(),
                runner_root: std::env::temp_dir().join("uor-coord-root"),
                spec: spec(session, epoch, id),
            },
        )
    }
    fn reserved_state() -> State {
        let mut state = state();
        state.transition(&register("one"), 1).unwrap();
        state.transition(&claim("one", None), 2).unwrap();
        state.transition(&reserve("one", 1, "job-one"), 3).unwrap();
        state
    }
    #[test]
    fn ownership_expires_and_old_epoch_cannot_revive() {
        let mut s = state();
        s.transition(&register("one"), 100).unwrap();
        s.transition(&claim("one", None), 101).unwrap();
        s.transition(&register("two"), 102).unwrap();
        assert!(s.transition(&claim("two", None), 103).is_err());
        s.transition(
            &event(
                "heartbeat",
                Action::Heartbeat {
                    session: "two".into(),
                    available: true,
                },
            ),
            1301,
        )
        .unwrap();
        assert!(s.transition(&claim("two", None), 1302).is_err());
        s.transition(&claim("two", Some("recovery:verified".into())), 1302)
            .unwrap();
        assert_eq!(s.tasks[&1].epoch, 2);
        assert!(s
            .transition(
                &event(
                    "release",
                    Action::Release {
                        issue: 1,
                        session: "one".into(),
                        epoch: 1
                    }
                ),
                1303
            )
            .is_err());
    }
    #[test]
    fn takeover_does_not_release_running_attempt() {
        let mut s = state();
        s.transition(&register("one"), 1).unwrap();
        s.transition(&claim("one", None), 2).unwrap();
        s.transition(&reserve("one", 1, "job-one"), 3).unwrap();
        s.transition(&register("two"), 1300).unwrap();
        s.transition(&claim("two", Some("receipt".into())), 1301)
            .unwrap();
        assert!(s.transition(&reserve("two", 2, "job-two"), 1302).is_err());
        assert_eq!(s.attempts["job-one"].phase, "reserved");
        assert!(validate_reserved_identity(
            &mut s,
            1,
            "two",
            2,
            "sha256:1111111111111111111111111111111111111111111111111111111111111111",
            "job-one",
            "mac",
            1303
        )
        .is_err());
        assert!(validate_reserved_identity(
            &mut s,
            1,
            "one",
            1,
            "sha256:1111111111111111111111111111111111111111111111111111111111111111",
            "job-one",
            "mac",
            1303
        )
        .is_err());
    }
    #[test]
    fn rejected_transition_has_no_partial_mutation() {
        let mut s = state();
        s.transition(&register("one"), 1).unwrap();
        let before = serde_json::to_vec(&s).unwrap();
        let mut c = claim("one", None);
        if let Action::Claim { paths, .. } = &mut c.action {
            paths.push("../escape".into());
        }
        assert!(s.transition(&c, 2).is_err());
        assert_eq!(before, serde_json::to_vec(&s).unwrap());
    }
    #[test]
    fn server_date_is_not_local_time() {
        assert_eq!(parse_http_date("Thu, 01 Jan 1970 00:00:00 GMT").unwrap(), 0);
        assert_eq!(
            parse_http_date("Tue, 29 Sep 2026 20:05:25 GMT").unwrap(),
            1790712325
        );
    }

    #[test]
    fn unavailable_heartbeat_never_extends_a_lease() {
        let mut s = reserved_state();
        let expiry = s.tasks[&1].expires;
        s.transition(
            &event(
                "offline",
                Action::Heartbeat {
                    session: "one".into(),
                    available: false,
                },
            ),
            100,
        )
        .unwrap();
        assert_eq!(s.tasks[&1].expires, expiry);
        assert!(validate_reserved_identity(
            &mut s,
            1,
            "one",
            1,
            "sha256:1111111111111111111111111111111111111111111111111111111111111111",
            "job-one",
            "mac",
            101
        )
        .is_err());
        assert_eq!(s.attempts["job-one"].phase, "reserved");
    }

    #[test]
    fn admission_fences_host_generation_spec_and_finalization() {
        let mut s = reserved_state();
        validate_reserved_identity(
            &mut s,
            1,
            "one",
            1,
            "sha256:1111111111111111111111111111111111111111111111111111111111111111",
            "job-one",
            "mac",
            10,
        )
        .unwrap();
        for (session, epoch, attempt, host) in [
            ("two", 1, "job-one", "mac"),
            ("one", 2, "job-one", "mac"),
            ("one", 1, "missing", "mac"),
            ("one", 1, "job-one", "other"),
        ] {
            assert!(validate_reserved_identity(
                &mut s,
                1,
                session,
                epoch,
                "sha256:1111111111111111111111111111111111111111111111111111111111111111",
                attempt,
                host,
                10
            )
            .is_err());
        }
        let original = s.attempts["job-one"].spec.clone().unwrap();
        validate_specs(&s, &original, &[], "mac").unwrap();
        let mut changed = original.clone();
        changed.rss_gib = 1.0;
        assert!(validate_specs(&s, &changed, &[], "mac").is_err());
        changed = original;
        changed.argv.push("unexpected".into());
        assert!(validate_specs(&s, &changed, &[], "mac").is_err());
        s.attempts.get_mut("job-one").unwrap().phase = "finalized".into();
        assert!(validate_reserved_identity(
            &mut s,
            1,
            "one",
            1,
            "sha256:1111111111111111111111111111111111111111111111111111111111111111",
            "job-one",
            "mac",
            10
        )
        .is_err());
    }

    #[test]
    fn unresolved_other_queue_reservation_prevents_undercounting() {
        let mut s = reserved_state();
        let current = s.attempts["job-one"].spec.clone().unwrap();
        let mut other = s.attempts["job-one"].clone();
        let mut other_spec = spec("one", 1, "job-other");
        other_spec.coordination.as_mut().unwrap().issue = 2;
        other.issue = 2;
        other.spec_sha256 = Some(spec_digest(&other_spec).unwrap());
        other.spec = Some(other_spec.clone());
        s.attempts.insert("job-other".into(), other);
        assert!(validate_specs(&s, &current, &[], "mac").is_err());
        validate_specs(&s, &current, &[other_spec], "mac").unwrap();
    }

    fn reserve_issue(s: &mut State, issue_number: u64, validation_lane: bool) -> Event {
        let mut second_claim = claim("one", None);
        if let Action::Claim { issue, paths, .. } = &mut second_claim.action {
            *issue = issue_number;
            *paths = vec![format!("src/issue-{issue_number}")];
        }
        s.transition(&second_claim, s.observed_server_time.max(4))
            .unwrap();
        let id = if issue_number == 2 {
            "job-two".to_string()
        } else {
            format!("job-{issue_number}")
        };
        let mut second = reserve("one", 1, &id);
        if let Action::Reserve { issue, spec, .. } = &mut second.action {
            *issue = issue_number;
            spec.coordination.as_mut().unwrap().issue = issue_number;
            spec.validation_lane = validation_lane;
        }
        second
    }

    fn reserve_second_issue(s: &mut State, validation_lane: bool) -> Event {
        reserve_issue(s, 2, validation_lane)
    }

    #[test]
    fn ordinary_and_validation_share_only_one_runner_and_keep_queue_fence() {
        let mut s = reserved_state();
        let second = reserve_second_issue(&mut s, true);
        s.transition(&second, 5).unwrap();
        let ordinary = s.attempts["job-one"].spec.clone().unwrap();
        let validation = s.attempts["job-two"].spec.clone().unwrap();
        // Both queued remains fail-closed; first start the ordinary job before
        // reserving/submitting validation, or reconcile one unstarted attempt.
        assert!(validate_specs(&s, &ordinary, &[], "mac").is_err());
        assert!(validate_specs(&s, &validation, &[], "mac").is_err());
        validate_specs(&s, &validation, &[ordinary], "mac").unwrap();
        validate_specs(
            &s,
            &s.attempts["job-one"].spec.clone().unwrap(),
            &[validation],
            "mac",
        )
        .unwrap();

        let mut validation_first = reserved_state();
        let first = validation_first.attempts.get_mut("job-one").unwrap();
        first.spec.as_mut().unwrap().validation_lane = true;
        first.spec_sha256 = Some(spec_digest(first.spec.as_ref().unwrap()).unwrap());
        let second = reserve_second_issue(&mut validation_first, false);
        validation_first.transition(&second, 5).unwrap();
    }

    #[test]
    fn multiple_reservations_of_either_lane_fit_actual_resource_limits() {
        for lane in [false, true] {
            let mut s = reserved_state();
            let first = s.attempts.get_mut("job-one").unwrap();
            first.spec.as_mut().unwrap().validation_lane = lane;
            first.spec_sha256 = Some(spec_digest(first.spec.as_ref().unwrap()).unwrap());
            for issue in 2..=8 {
                let next = reserve_issue(&mut s, issue, lane);
                s.transition(&next, 5).unwrap();
            }
            assert_eq!(s.attempts.len(), 8);
            let ninth = reserve_issue(&mut s, 9, lane);
            assert!(s.transition(&ninth, 5).is_err());
            assert_eq!(s.attempts.len(), 8);
        }
    }

    #[test]
    fn host_reservations_reject_different_roots_unbound_specs_and_exclusive_jobs() {
        for scenario in [
            "root",
            "legacy",
            "digest",
            "exclusive-first",
            "exclusive-new",
        ] {
            let mut s = reserved_state();
            let mut second = reserve_second_issue(&mut s, true);
            let first = s.attempts.get_mut("job-one").unwrap();
            match scenario {
                "root" => first.runner_root = Some(std::env::temp_dir().join("other-runner")),
                "legacy" => first.spec = None,
                "digest" => first.spec_sha256 = Some("0".repeat(64)),
                "exclusive-first" => {
                    first.spec.as_mut().unwrap().exclusive = true;
                    first.spec_sha256 = Some(spec_digest(first.spec.as_ref().unwrap()).unwrap());
                }
                "exclusive-new" => {
                    if let Action::Reserve { spec, .. } = &mut second.action {
                        spec.exclusive = true;
                    }
                }
                _ => unreachable!(),
            }
            assert!(s.transition(&second, 5).is_err(), "{scenario}");
        }
    }

    #[test]
    fn host_reservations_account_threads_and_memory_before_reserving() {
        for (threads, rss_gib, accepted) in [(6, 9.0, true), (7, 9.0, false), (6, 9.5, false)] {
            let mut s = reserved_state();
            let first = s.attempts.get_mut("job-one").unwrap();
            let ordinary = first.spec.as_mut().unwrap();
            ordinary.threads = threads;
            ordinary.rss_gib = rss_gib;
            first.spec_sha256 = Some(spec_digest(ordinary).unwrap());
            let mut second = reserve_second_issue(&mut s, true);
            if let Action::Reserve { spec, .. } = &mut second.action {
                spec.threads = 2;
                spec.rss_gib = 2.0;
            }
            assert_eq!(s.transition(&second, 5).is_ok(), accepted);
        }
    }

    #[test]
    fn additional_reservations_do_not_relax_per_issue_or_attempt_identity_fences() {
        let mut s = reserved_state();
        let mut same_issue = reserve("one", 1, "same-issue");
        if let Action::Reserve { spec, .. } = &mut same_issue.action {
            spec.validation_lane = true;
        }
        assert!(s.transition(&same_issue, 4).is_err());
        let second = reserve_second_issue(&mut s, true);
        s.transition(&second, 5).unwrap();
        assert!(s.transition(&second, 6).is_err());
        let mut third = reserve("one", 1, "job-three");
        let mut third_claim = claim("one", None);
        if let Action::Claim { issue, paths, .. } = &mut third_claim.action {
            *issue = 3;
            *paths = vec!["src/third".into()];
        }
        s.transition(&third_claim, 6).unwrap();
        if let Action::Reserve { issue, spec, .. } = &mut third.action {
            *issue = 3;
            spec.coordination.as_mut().unwrap().issue = 3;
            spec.validation_lane = true;
        }
        s.transition(&third, 7).unwrap();
        assert!(s.transition(&third, 8).is_err());
    }

    #[test]
    fn third_reservation_accounts_full_memory_union_and_all_identities() {
        for scenario in ["fits", "memory", "root", "legacy", "digest", "phase"] {
            let mut s = reserved_state();
            let first = s.attempts.get_mut("job-one").unwrap();
            first.spec.as_mut().unwrap().rss_gib = 5.0;
            first.spec_sha256 = Some(spec_digest(first.spec.as_ref().unwrap()).unwrap());
            let mut second = reserve_second_issue(&mut s, false);
            if let Action::Reserve { spec, .. } = &mut second.action {
                spec.rss_gib = 5.0;
            }
            s.transition(&second, 5).unwrap();
            let second = s.attempts.get_mut("job-two").unwrap();
            match scenario {
                "root" => second.runner_root = Some(std::env::temp_dir().join("foreign")),
                "legacy" => second.spec = None,
                "digest" => second.spec_sha256 = Some("0".repeat(64)),
                "phase" => second.phase = "unknown".into(),
                _ => {}
            }
            let mut third = reserve_issue(&mut s, 3, false);
            if let Action::Reserve { spec, .. } = &mut third.action {
                spec.rss_gib = if scenario == "memory" { 1.5 } else { 1.0 };
            }
            assert_eq!(
                s.transition(&third, 6).is_ok(),
                scenario == "fits",
                "{scenario}"
            );
        }
    }

    #[test]
    fn three_reservations_require_every_other_exact_local_running_spec() {
        let mut s = reserved_state();
        let second = reserve_second_issue(&mut s, false);
        s.transition(&second, 5).unwrap();
        let third = reserve_issue(&mut s, 3, false);
        s.transition(&third, 6).unwrap();
        let current = s.attempts["job-3"].spec.clone().unwrap();
        let first = s.attempts["job-one"].spec.clone().unwrap();
        let second = s.attempts["job-two"].spec.clone().unwrap();
        assert!(validate_specs(&s, &current, &[], "mac").is_err());
        assert!(validate_specs(&s, &current, &[first.clone()], "mac").is_err());
        assert!(validate_specs(&s, &current, &[second.clone()], "mac").is_err());
        validate_specs(&s, &current, &[first, second], "mac").unwrap();
    }

    #[test]
    fn initial_reservation_cannot_exceed_host_resource_limits() {
        for (threads, rss_gib) in [(9, 0.5), (1, 11.5)] {
            let mut s = state();
            s.transition(&register("one"), 1).unwrap();
            s.transition(&claim("one", None), 2).unwrap();
            let mut first = reserve("one", 1, "job-one");
            if let Action::Reserve { spec, .. } = &mut first.action {
                spec.threads = threads;
                spec.rss_gib = rss_gib;
            }
            assert!(s.transition(&first, 3).is_err());
            assert!(s.attempts.is_empty());
        }
    }

    #[test]
    fn actual_runner_root_must_match_reservation() {
        let root = std::env::temp_dir().join(format!("coord-root-binding-{}", std::process::id()));
        let a = root.join("a");
        let b = root.join("b");
        fs::create_dir_all(&a).unwrap();
        fs::create_dir_all(&b).unwrap();
        let mut s = reserved_state();
        let attempt = s.attempts.get_mut("job-one").unwrap();
        attempt.runner_root = Some(a.clone());
        validate_runner_root(attempt, &a).unwrap();
        assert!(validate_runner_root(attempt, &b).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn source_ownership_rejects_path_aliases() {
        for alias in [
            "crates/uor-r4-training/",
            "./crates/uor-r4-training",
            "crates//uor-r4-training",
            "crates/../crates/uor-r4-training",
            "crates\\uor-r4-training",
        ] {
            let mut state = state();
            let mut claim = claim("two", None);
            if let Action::Claim { paths, .. } = &mut claim.action {
                *paths = vec![alias.into()];
            }
            state.transition(&register("two"), 20).unwrap();
            assert!(state.transition(&claim, 21).is_err(), "accepted {alias}");
        }
    }

    #[test]
    fn policy_and_work_card_versions_fence_new_work() {
        for invalid in [
            "issue-1",
            "sha256:test",
            "",
            "https://github.com/o/r/issues/1",
        ] {
            assert!(work_card_digest(invalid).is_err());
        }
        let mut s = state();
        s.transition(&register("one"), 1).unwrap();
        let mut request = claim("one", None);
        if let Action::Claim { policy_sha, .. } = &mut request.action {
            *policy_sha = "c".repeat(40);
        }
        assert!(s.transition(&request, 2).is_err());
        validate_policy_comparison(
            &serde_json::json!({"status":"ahead","files":[{"filename":"src/routine.rs"}]}),
        )
        .unwrap();
        for value in [
            serde_json::json!({"status":"diverged","files":[]}),
            serde_json::json!({"status":"ahead","files":[{"filename":"docs/labs/protocol.md"}]}),
            serde_json::json!({"status":"ahead","files":[{"filename":"docs/labs/operations.md"}]}),
        ] {
            assert!(validate_policy_comparison(&value).is_err());
        }
        let adopt = event(
            "new-policy",
            Action::AdoptPolicy {
                previous_policy_sha: "b".repeat(40),
                policy_sha: "c".repeat(40),
                decision_receipt: delivery::FileEvidence {
                    path: "/not-executed".into(),
                    sha256: "d".repeat(64),
                },
            },
        );
        s.transition(&claim("one", None), 2).unwrap();
        assert!(s.transition(&adopt, 3).is_err());
        s.transition(
            &event(
                "release",
                Action::Release {
                    issue: 1,
                    session: "one".into(),
                    epoch: 1,
                },
            ),
            4,
        )
        .unwrap();
        s.transition(&adopt, 5).unwrap();
        assert_eq!(s.policy_sha, "c".repeat(40));
    }

    #[test]
    fn queued_cancel_requires_positive_never_started_evidence() {
        let s = reserved_state();
        let attempt = &s.attempts["job-one"];
        let mut exit: ConfirmedExit = serde_json::from_value(serde_json::json!({
            "schema":jobs::EXIT_SCHEMA, "id":"job-one", "attempt_id":"job-one", "host":"mac",
            "spec_sha256":attempt.spec_sha256, "coordination":attempt.spec.as_ref().unwrap().coordination,
            "process_state":"never_started", "outcome":"cancelled", "exit_status":null,"elapsed_ms":0
        })).unwrap();
        validate_exit_contents(&exit, "job-one", attempt, EvidenceMode::Finalize, None).unwrap();
        exit.elapsed_ms = 1;
        assert!(
            validate_exit_contents(&exit, "job-one", attempt, EvidenceMode::Finalize, None)
                .is_err()
        );
        exit.elapsed_ms = 0;
        exit.process_state = "unknown".into();
        assert!(
            validate_exit_contents(&exit, "job-one", attempt, EvidenceMode::Finalize, None)
                .is_err()
        );
    }

    #[test]
    fn cancelled_queue_receipt_can_release_only_without_launch_intent() {
        let id = format!(
            "cancel-bound-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let root = std::env::temp_dir().join(&id);
        fs::create_dir_all(&root).unwrap();
        let root = fs::canonicalize(&root).unwrap();
        let s = reserved_state();
        let mut attempt = s.attempts["job-one"].clone();
        let mut job = attempt.spec.clone().unwrap();
        job.id = id.clone();
        job.cwd = root.clone();
        job.coordination.as_mut().unwrap().attempt_id = id.clone();
        attempt.host = process::host_id().unwrap();
        attempt.runner_root = Some(root.clone());
        attempt.spec_sha256 = Some(spec_digest(&job).unwrap());
        attempt.spec = Some(job.clone());
        let input = root.join("input.json");
        fs::write(&input, serde_json::to_vec(&job).unwrap()).unwrap();
        jobs::submit(&root, &input).unwrap();
        jobs::cancel(&root, &id, &root.join("ledger")).unwrap();
        let path = jobs::done_dir(&root).join(&id).join("exit.json");
        let evidence = ExitEvidence {
            host: attempt.host.clone(),
            job_id: id.clone(),
            attempt_id: id.clone(),
            sha256: digest(&fs::read(&path).unwrap()),
            path: path.clone(),
        };
        verify_exit_evidence(&id, &attempt, &evidence, EvidenceMode::Finalize).unwrap();
        fs::write(path.parent().unwrap().join("attempt.json"), b"{}").unwrap();
        assert!(verify_exit_evidence(&id, &attempt, &evidence, EvidenceMode::Finalize).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    /// A reservation whose task generation is already released and expired, and
    /// whose durable queued spec drifted away from the reserved spec — the live
    /// orphan shape that `Action::Abandon` exists to close.
    struct OrphanFixture {
        root: PathBuf,
        id: String,
        state: State,
        attempt: Attempt,
        evidence: ExitEvidence,
    }

    impl OrphanFixture {
        fn new() -> Self {
            Self::with_drift(true)
        }

        /// The same reservation with the reserved spec bound to the durable
        /// queued spec exactly, as `FinishAttempt` requires.
        fn bound() -> Self {
            Self::with_drift(false)
        }

        fn with_drift(drift: bool) -> Self {
            let id = format!(
                "abandon-bound-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );
            let root = std::env::temp_dir().join(&id);
            fs::create_dir_all(&root).unwrap();
            let root = fs::canonicalize(&root).unwrap();
            // Release the claim so no live generation owns the reservation,
            // exactly as for a queued job cancelled long ago.
            let mut state = reserved_state();
            state
                .transition(
                    &event(
                        "release",
                        Action::Release {
                            issue: 1,
                            session: "one".into(),
                            epoch: 1,
                        },
                    ),
                    4,
                )
                .unwrap();
            let mut attempt = state.attempts.remove("job-one").unwrap();
            let mut job = attempt.spec.clone().unwrap();
            job.id = id.clone();
            job.cwd = root.clone();
            job.coordination.as_mut().unwrap().attempt_id = id.clone();
            attempt.host = process::host_id().unwrap();
            attempt.runner_root = Some(root.clone());
            let input = root.join("input.json");
            fs::write(&input, serde_json::to_vec(&job).unwrap()).unwrap();
            jobs::submit(&root, &input).unwrap();
            jobs::cancel(&root, &id, &root.join("ledger")).unwrap();
            if drift {
                // The reservation's work card no longer matches the durable
                // queued spec, so their digests differ too; `FinishAttempt`
                // refuses this orphan for exactly that reason.
                job.coordination.as_mut().unwrap().work_card = format!("sha256:{}", "2".repeat(64));
            }
            attempt.spec = Some(job.clone());
            attempt.spec_sha256 = Some(spec_digest(&job).unwrap());
            state.attempts.insert(id.clone(), attempt.clone());
            let path = jobs::done_dir(&root).join(&id).join("exit.json");
            let evidence = ExitEvidence {
                host: attempt.host.clone(),
                job_id: id.clone(),
                attempt_id: id.clone(),
                sha256: digest(&fs::read(&path).unwrap()),
                path,
            };
            Self {
                root,
                id,
                state,
                attempt,
                evidence,
            }
        }

        fn abandon_event(&self) -> Event {
            event(
                "abandon",
                Action::Abandon {
                    attempt: self.id.clone(),
                    session: "one".into(),
                    reason: "queued cancellation orphaned by workspace loss".into(),
                    receipt: self.evidence.clone(),
                },
            )
        }

        fn finish_event(&self) -> Event {
            event(
                "finish",
                Action::FinishAttempt {
                    issue: 1,
                    session: "one".into(),
                    epoch: 1,
                    attempt: self.id.clone(),
                    receipt: self.evidence.clone(),
                },
            )
        }
    }

    impl Drop for OrphanFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn abandon_finalizes_a_reserved_orphan_without_a_live_claim() {
        let mut f = OrphanFixture::new();
        assert_eq!(f.state.tasks[&1].phase, "released");
        // The existing path cannot close it: no live claim, and the durable
        // spec digest drifted from the reserved digest.
        let finish = f.finish_event();
        assert!(f.state.transition(&finish, 10).is_err());
        assert!(preflight_event(&f.state, &finish).is_err());
        // Abandon verifies the never-started receipt locally and finalizes the
        // reservation without any claim.
        let abandon = f.abandon_event();
        preflight_event(&f.state, &abandon).unwrap();
        f.state.transition(&abandon, 10).unwrap();
        assert_eq!(f.state.attempts[&f.id].phase, "finalized");
        assert!(f.state.attempts[&f.id].receipt.is_some());
        assert_eq!(f.state.tasks[&1].phase, "released");
        // The deadlock is broken: the finalized attempt no longer blocks policy
        // adoption, which is what restores claims.
        f.state
            .transition(
                &event(
                    "adopt",
                    Action::AdoptPolicy {
                        previous_policy_sha: "b".repeat(40),
                        policy_sha: "c".repeat(40),
                        decision_receipt: delivery::FileEvidence {
                            path: PathBuf::from("/tmp/unused-decision-receipt.json"),
                            sha256: "0".repeat(64),
                        },
                    },
                ),
                11,
            )
            .unwrap();
        assert_eq!(f.state.policy_sha, "c".repeat(40));
        // A finalized attempt cannot be abandoned twice.
        assert!(f.state.transition(&f.abandon_event(), 12).is_err());
    }

    #[test]
    fn abandon_rejects_stopped_or_unknown_process_evidence() {
        let f = OrphanFixture::bound();
        let original: serde_json::Value =
            serde_json::from_slice(&fs::read(&f.evidence.path).unwrap()).unwrap();
        for (state, outcome, status, elapsed) in [
            ("confirmed_stopped", "completed", Some(0), 10),
            ("unknown", "unknown", None, 0),
        ] {
            let mut exit = original.clone();
            exit["process_state"] = serde_json::json!(state);
            exit["outcome"] = serde_json::json!(outcome);
            exit["exit_status"] = serde_json::json!(status);
            exit["elapsed_ms"] = serde_json::json!(elapsed);
            let bytes = serde_json::to_vec(&exit).unwrap();
            fs::write(&f.evidence.path, &bytes).unwrap();
            let evidence = ExitEvidence {
                sha256: digest(&bytes),
                ..f.evidence.clone()
            };
            let error = verify_exit_evidence(&f.id, &f.attempt, &evidence, EvidenceMode::Abandon)
                .unwrap_err()
                .to_string();
            assert!(error.contains("never-started"), "{state}: {error}");
        }
        fs::write(&f.evidence.path, serde_json::to_vec(&original).unwrap()).unwrap();
        // A rejected abandon leaves the reservation open for review: the apply
        // path verifies before it transitions, so no state is consumed.
        let mut exit = original;
        exit["process_state"] = serde_json::json!("confirmed_stopped");
        exit["exit_status"] = serde_json::json!(null);
        let bytes = serde_json::to_vec(&exit).unwrap();
        fs::write(&f.evidence.path, &bytes).unwrap();
        let mut rejected = f.abandon_event();
        if let Action::Abandon { receipt, .. } = &mut rejected.action {
            receipt.sha256 = digest(&bytes);
        }
        let state = f.state.clone();
        assert!(preflight_event(&state, &rejected).is_err());
        assert_eq!(state.attempts[&f.id].phase, "reserved");
    }

    #[test]
    fn abandon_requires_the_reserving_session_and_an_open_reservation() {
        let mut f = OrphanFixture::new();
        let mut impostor = f.abandon_event();
        if let Action::Abandon { session, .. } = &mut impostor.action {
            *session = "two".into();
        }
        // Local evidence is valid; only the reservation's owner may close it.
        preflight_event(&f.state, &impostor).unwrap();
        assert!(f.state.transition(&impostor, 10).is_err());
        let mut unknown = f.abandon_event();
        if let Action::Abandon { attempt, .. } = &mut unknown.action {
            *attempt = "job-does-not-exist".into();
        }
        assert!(f.state.transition(&unknown, 10).is_err());
        let mut empty_reason = f.abandon_event();
        if let Action::Abandon { reason, .. } = &mut empty_reason.action {
            *reason = "  ".into();
        }
        assert!(f.state.transition(&empty_reason, 10).is_err());
        let mut huge_reason = f.abandon_event();
        if let Action::Abandon { reason, .. } = &mut huge_reason.action {
            *reason = "x".repeat(REASON_MAX + 1);
        }
        assert!(f.state.transition(&huge_reason, 10).is_err());
        assert_eq!(f.state.attempts[&f.id].phase, "reserved");
        f.state.transition(&f.abandon_event(), 10).unwrap();
        assert_eq!(f.state.attempts[&f.id].phase, "finalized");
    }

    #[test]
    fn abandon_rejects_any_launch_state_or_missing_cancel_marker() {
        let f = OrphanFixture::new();
        let dir = f.evidence.path.parent().unwrap().to_path_buf();
        let queued = jobs::queue_dir(&f.root).join(&f.id);
        fs::create_dir_all(&queued).unwrap();
        assert!(
            verify_exit_evidence(&f.id, &f.attempt, &f.evidence, EvidenceMode::Abandon).is_err()
        );
        fs::remove_dir(&queued).unwrap();
        let running = jobs::running_dir(&f.root).join(&f.id);
        fs::create_dir_all(&running).unwrap();
        assert!(
            verify_exit_evidence(&f.id, &f.attempt, &f.evidence, EvidenceMode::Abandon).is_err()
        );
        fs::remove_dir(&running).unwrap();
        fs::write(dir.join("attempt.json"), b"{}").unwrap();
        assert!(
            verify_exit_evidence(&f.id, &f.attempt, &f.evidence, EvidenceMode::Abandon).is_err()
        );
        fs::remove_file(dir.join("attempt.json")).unwrap();
        fs::write(dir.join("process.json"), b"{}").unwrap();
        assert!(
            verify_exit_evidence(&f.id, &f.attempt, &f.evidence, EvidenceMode::Abandon).is_err()
        );
        fs::remove_file(dir.join("process.json")).unwrap();
        verify_exit_evidence(&f.id, &f.attempt, &f.evidence, EvidenceMode::Abandon).unwrap();
        fs::remove_file(dir.join("cancel.json")).unwrap();
        assert!(
            verify_exit_evidence(&f.id, &f.attempt, &f.evidence, EvidenceMode::Abandon).is_err()
        );
        // The apply path refuses the whole action, so no transition is reached.
        assert!(preflight_event(&f.state, &f.abandon_event()).is_err());
        assert_eq!(f.state.attempts[&f.id].phase, "reserved");
    }

    #[test]
    fn abandon_event_round_trips_through_the_operator_json_shape() {
        // The operator submits `coord apply STORE EVENT_JSON`; this is the exact
        // envelope shape, parsed by the same derived deserializer.
        let value = serde_json::json!({
            "event_id": "orphan-abandon-1512",
            "op": "abandon",
            "attempt": "opencode-b0-flock-g1a2-20260930-g2",
            "session": "opencode-deepseek-20260930",
            "reason": "queued cancellation orphaned by lost workspace volume",
            "receipt": {
                "host": "host-284b64fb2f70567aee465dc4941430076137e580d19d9568b15726d14b157cec",
                "job_id": "opencode-b0-flock-g1a2-20260930-g2",
                "attempt_id": "opencode-b0-flock-g1a2-20260930-g2",
                "path": "/Users/casey.allard/.local/share/uor-r4/runner/done/opencode-b0-flock-g1a2-20260930-g2/exit.json",
                "sha256": "a282d22bf0a52951cc024a776fb4599c2e59e42f868d13f774bb2386bc84bddf"
            }
        });
        let decoded: Event = serde_json::from_value(value.clone()).unwrap();
        assert!(matches!(decoded.action, Action::Abandon { .. }));
        assert_eq!(value, serde_json::to_value(&decoded).unwrap());
        let mut wrong_op = value;
        wrong_op["op"] = serde_json::json!("finish_attempt");
        assert!(serde_json::from_value::<Event>(wrong_op).is_err());
    }

    #[test]
    fn finish_attempt_still_requires_a_live_unexpired_claim() {
        // A receipt that the unchanged Finalize verifier accepts exactly, but
        // whose task generation is released and expired: FinishAttempt must
        // still refuse it. Abandon is not allowed to weaken this.
        let mut f = OrphanFixture::bound();
        let finish = f.finish_event();
        preflight_event(&f.state, &finish).unwrap();
        let error = f.state.transition(&finish, 10).unwrap_err().to_string();
        assert!(
            error.contains("stale or expired task generation"),
            "{error}"
        );
        assert_eq!(f.state.attempts[&f.id].phase, "reserved");
        // Control: with a live unexpired claim the same receipt finalizes.
        let mut live = OrphanFixture::bound();
        let claim = live.state.tasks.get_mut(&1).unwrap();
        claim.phase = "claimed".into();
        claim.expires = 1_000;
        let finish = live.finish_event();
        preflight_event(&live.state, &finish).unwrap();
        live.state.transition(&finish, 10).unwrap();
        assert_eq!(live.state.attempts[&live.id].phase, "finalized");
    }

    #[test]
    fn pre_spawn_reconciliation_requires_positive_bound_proof_and_zero_extra_cost() {
        let id = format!(
            "pre-spawn-bound-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let root = std::env::temp_dir().join(&id);
        jobs::ensure_layout(&root).unwrap();
        let root = fs::canonicalize(root).unwrap();
        let ledger = root.join("ledger");
        fs::create_dir(&ledger).unwrap();
        crate::ledger::initialize_empty(&ledger, 100_000, "test").unwrap();
        let mut attempt = reserved_state().attempts["job-one"].clone();
        let mut job = attempt.spec.clone().unwrap();
        job.id = id.clone();
        job.cwd = root.clone();
        job.coordination.as_mut().unwrap().attempt_id = id.clone();
        attempt.host = process::host_id().unwrap();
        attempt.runner_root = Some(root.clone());
        attempt.spec_sha256 = Some(spec_digest(&job).unwrap());
        attempt.spec = Some(job.clone());
        let input = root.join("input.json");
        fs::write(&input, serde_json::to_vec(&job).unwrap()).unwrap();
        jobs::submit(&root, &input).unwrap();
        let running = jobs::running_dir(&root).join(&id);
        let guard = jobs::job_lock(&root, &id).unwrap();
        jobs::durable_rename(&jobs::queue_dir(&root).join(&id), &running).unwrap();
        let local = jobs::prepare_reserved_attempt(&running, &id, Some(&id)).unwrap();
        jobs::record_pre_spawn_failure(
            &root,
            &job,
            &local,
            12,
            "fixture hash mismatch before spawn".into(),
        )
        .unwrap();
        drop(guard);
        jobs::finalize(
            &root,
            &job,
            &jobs::Finalization {
                outcome: "unknown".into(),
                exit_status: None,
                reason: "preflight failed".into(),
                peak_rss_kib: 0,
                started_utc: local.started_utc,
                elapsed_ms: 12,
                measurement: crate::ledger::Measurement::Measured,
            },
            &ledger,
        )
        .unwrap();
        let path = jobs::reconcile_stopped(&root, &id, &ledger).unwrap();
        let dir = path.parent().unwrap();
        let original_exit = fs::read(dir.join("exit.json")).unwrap();
        let proof_bytes = fs::read(&path).unwrap();
        let mut evidence = ExitEvidence {
            host: attempt.host.clone(),
            job_id: id.clone(),
            attempt_id: id.clone(),
            path: path.clone(),
            sha256: digest(&proof_bytes),
        };
        verify_exit_evidence(&id, &attempt, &evidence, EvidenceMode::Finalize).unwrap();
        let proof: serde_json::Value = serde_json::from_slice(&proof_bytes).unwrap();
        for (field, value) in [
            ("evidence_kind", serde_json::json!(null)),
            ("evidence_kind", serde_json::json!("owned_process")),
            ("process_state", serde_json::json!("confirmed_stopped")),
            ("additional_charged_ms", serde_json::json!(1)),
            (
                "pre_spawn_failure_sha256",
                serde_json::json!("0".repeat(64)),
            ),
            ("original_exit_sha256", serde_json::json!("0".repeat(64))),
        ] {
            let mut bad = proof.clone();
            bad[field] = value;
            let bytes = serde_json::to_vec(&bad).unwrap();
            fs::write(&path, &bytes).unwrap();
            evidence.sha256 = digest(&bytes);
            assert!(
                verify_exit_evidence(&id, &attempt, &evidence, EvidenceMode::Finalize).is_err(),
                "accepted changed {field}"
            );
        }
        fs::write(&path, &proof_bytes).unwrap();
        evidence.sha256 = digest(&proof_bytes);
        fs::write(dir.join("launch.go"), b"contradictory launch state").unwrap();
        assert!(verify_exit_evidence(&id, &attempt, &evidence, EvidenceMode::Finalize).is_err());
        fs::remove_file(dir.join("launch.go")).unwrap();
        verify_exit_evidence(&id, &attempt, &evidence, EvidenceMode::Finalize).unwrap();
        fs::remove_file(dir.join("preflight-failure.json")).unwrap();
        assert!(verify_exit_evidence(&id, &attempt, &evidence, EvidenceMode::Finalize).is_err());
        assert_eq!(fs::read(dir.join("exit.json")).unwrap(), original_exit);
        assert_eq!(crate::ledger::rebuild(&ledger).unwrap().cumulative_ms, 12);
        fs::remove_dir_all(root).unwrap();
    }

    fn exit_for(attempt: &Attempt, id: &str) -> serde_json::Value {
        serde_json::json!({"schema":jobs::EXIT_SCHEMA,"id":id,"attempt_id":id,"host":attempt.host,
            "spec_sha256":attempt.spec_sha256,"coordination":attempt.spec.as_ref().unwrap().coordination,
            "process_state":"confirmed_stopped","outcome":"completed","exit_status":0,"elapsed_ms":10})
    }

    #[test]
    fn unknown_or_mismatched_exit_never_releases_reservation() {
        let s = reserved_state();
        let attempt = &s.attempts["job-one"];
        let base = exit_for(attempt, "job-one");
        validate_exit_contents(
            &serde_json::from_value(base.clone()).unwrap(),
            "job-one",
            attempt,
            EvidenceMode::Finalize,
            None,
        )
        .unwrap();
        for (field, value) in [
            ("process_state", serde_json::json!("unknown")),
            ("outcome", serde_json::json!("unknown")),
            ("host", serde_json::json!("remote")),
            ("attempt_id", serde_json::json!("wrong")),
            ("exit_status", serde_json::json!(null)),
            ("spec_sha256", serde_json::json!("0".repeat(64))),
        ] {
            let mut bad = base.clone();
            bad[field] = value;
            assert!(validate_exit_contents(
                &serde_json::from_value(bad).unwrap(),
                "job-one",
                attempt,
                EvidenceMode::Finalize,
                None
            )
            .is_err());
        }
        let mut bad = base;
        bad["coordination"]["epoch"] = serde_json::json!(2);
        assert!(validate_exit_contents(
            &serde_json::from_value(bad).unwrap(),
            "job-one",
            attempt,
            EvidenceMode::Finalize,
            None
        )
        .is_err());
        assert_eq!(attempt.phase, "reserved");
    }

    #[test]
    fn state_and_typed_events_round_trip_without_losing_resource_bindings() {
        let s = reserved_state();
        let bytes = serde_json::to_vec(&s).unwrap();
        let decoded: State = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(bytes, serde_json::to_vec(&decoded).unwrap());
        let e = reserve("one", 1, "roundtrip");
        let value = serde_json::to_value(&e).unwrap();
        let decoded: Event = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(value, serde_json::to_value(decoded).unwrap());
        let malformed = serde_json::json!({"event_id":"finish","op":"finish_attempt","issue":1,"session":"one","epoch":1,"attempt":"job-one","receipt":"unverified assertion"});
        assert!(serde_json::from_value::<Event>(malformed).is_err());
    }

    #[test]
    fn event_parser_rejects_unknown_fields_and_malformed_envelopes() {
        let valid = serde_json::json!({"event_id":"beat", "op":"heartbeat", "session":"one", "available":true});
        assert!(serde_json::from_value::<Event>(valid.clone()).is_ok());
        let mut extra = valid.clone();
        extra["unexpected"] = serde_json::json!(true);
        assert!(serde_json::from_value::<Event>(extra).is_err());
        let mut wrong_action = valid.clone();
        wrong_action["epoch"] = serde_json::json!(1);
        assert!(serde_json::from_value::<Event>(wrong_action).is_err());
        let mut missing = valid.clone();
        missing.as_object_mut().unwrap().remove("event_id");
        assert!(serde_json::from_value::<Event>(missing).is_err());
        let mut wrong_id = valid;
        wrong_id["event_id"] = serde_json::json!(1);
        assert!(serde_json::from_value::<Event>(wrong_id).is_err());
        assert!(serde_json::from_value::<Event>(serde_json::json!([])).is_err());
        let mut nested = serde_json::to_value(reserve("one", 1, "extra-receipt")).unwrap();
        nested["spec"]["unknown_field"] = serde_json::json!(true);
        assert!(serde_json::from_value::<Event>(nested).is_err());
    }

    #[test]
    fn exclusive_task_blocks_partner_until_finalized() {
        let mut state = reserved_state();
        let first = state.attempts.get_mut("job-one").unwrap();
        first.spec.as_mut().unwrap().exclusive = true;
        first.spec_sha256 = Some(spec_digest(first.spec.as_ref().unwrap()).unwrap());
        let mut other_claim = claim("one", None);
        if let Action::Claim { issue, paths, .. } = &mut other_claim.action {
            *issue = 2;
            *paths = vec!["src/other".into()];
        }
        state.transition(&other_claim, 4).unwrap();
        let mut other = reserve("one", 1, "job-two");
        if let Action::Reserve { issue, spec, .. } = &mut other.action {
            *issue = 2;
            spec.coordination.as_mut().unwrap().issue = 2;
        }
        let before = serde_json::to_value(&state).unwrap();
        assert!(state.transition(&other, 5).is_err());
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
        state.attempts.get_mut("job-one").unwrap().phase = "finalized".into();
        state.transition(&other, 6).unwrap();
    }

    fn completion_fixture() -> (
        State,
        CompletionEvidence,
        delivery::DeliveryReceipt,
        CompletionAcceptance,
    ) {
        let mut state = state();
        state.transition(&register("one"), 1).unwrap();
        state.transition(&claim("one", None), 2).unwrap();
        let file = delivery::FileEvidence {
            path: PathBuf::from("/missing/acceptance.json"),
            sha256: "c".repeat(64),
        };
        let evidence = CompletionEvidence {
            path: PathBuf::from("/missing/delivery.json"),
            sha256: "d".repeat(64),
            merge_sha: "e".repeat(40),
            acceptance: file.clone(),
        };
        let receipt = delivery::DeliveryReceipt {
            schema: delivery::SCHEMA.into(),
            repository: state.repository.clone(),
            pull_request: 99,
            task_issue: 1,
            author_session: "one".into(),
            claim_session: "one".into(),
            claim_epoch: 1,
            work_card: "sha256:1111111111111111111111111111111111111111111111111111111111111111"
                .into(),
            head_sha: "a".repeat(40),
            base_sha: "b".repeat(40),
            change_kind: "code".into(),
            class: "A".into(),
            checks: vec![],
            reviews: vec![delivery::ReviewReceipt {
                review_id: "review".into(),
                reviewer_session: "two".into(),
                launched_by: "one".into(),
                head_sha: "a".repeat(40),
                base_sha: "b".repeat(40),
                decision: "APPROVE".into(),
                unresolved_required_fixes: 0,
                evidence: file,
            }],
            result_evidence: None,
            result_reader_session: None,
            council: None,
        };
        let acceptance = CompletionAcceptance {
            schema: "uor-r4.task-acceptance/1".into(),
            task_issue: 1,
            work_card: "sha256:1111111111111111111111111111111111111111111111111111111111111111"
                .into(),
            head_sha: receipt.head_sha.clone(),
            merge_sha: evidence.merge_sha.clone(),
            reviewer_session: "two".into(),
            decision: "APPROVE".into(),
            full_scope: true,
            acceptance_criteria: vec![
                "All issue criteria verified against retained evidence.".into()
            ],
        };
        (state, evidence, receipt, acceptance)
    }

    #[test]
    fn complete_rejects_an_assertion_or_a_missing_receipt() {
        let arbitrary = serde_json::json!({"event_id":"complete", "op":"complete", "issue":1, "session":"one", "epoch":1, "delivery_receipt":"looks merged"});
        assert!(serde_json::from_value::<Event>(arbitrary).is_err());
        let (state, evidence, _, _) = completion_fixture();
        let event = event(
            "complete",
            Action::Complete {
                issue: 1,
                session: "one".into(),
                epoch: 1,
                delivery_receipt: evidence,
            },
        );
        assert!(preflight_event(&state, &event).is_err());
        assert_eq!(state.tasks[&1].phase, "claimed");
    }

    #[test]
    fn completion_requires_full_scope_independent_acceptance_and_current_generation() {
        let (state, evidence, receipt, mut acceptance) = completion_fixture();
        validate_completion_binding(&state, 1, "one", 1, &evidence, &receipt, &acceptance).unwrap();
        acceptance.full_scope = false;
        assert!(
            validate_completion_binding(&state, 1, "one", 1, &evidence, &receipt, &acceptance)
                .is_err()
        );
        acceptance.full_scope = true;
        acceptance.reviewer_session = "one".into();
        assert!(
            validate_completion_binding(&state, 1, "one", 1, &evidence, &receipt, &acceptance)
                .is_err()
        );
        acceptance.reviewer_session = "two".into();
        assert!(
            validate_completion_binding(&state, 1, "one", 2, &evidence, &receipt, &acceptance)
                .is_err()
        );
    }

    #[test]
    fn completion_requires_exact_merged_head_and_main_ancestry() {
        let (_, evidence, receipt, _) = completion_fixture();
        let pr = serde_json::json!({"number":99,"merged":true,"head":{"sha":receipt.head_sha},"merge_commit_sha":evidence.merge_sha,"base":{"ref":"main","repo":{"full_name":receipt.repository}}});
        let comparison = serde_json::json!({"status":"ahead","base_commit":{"sha":evidence.merge_sha},"merge_base_commit":{"sha":evidence.merge_sha}});
        validate_merged_identity(&evidence, &receipt, &pr, &comparison).unwrap();
        let mut unmerged = pr.clone();
        unmerged["merged"] = serde_json::json!(false);
        assert!(validate_merged_identity(&evidence, &receipt, &unmerged, &comparison).is_err());
        let mut stale = pr.clone();
        stale["head"]["sha"] = serde_json::json!("f".repeat(40));
        assert!(validate_merged_identity(&evidence, &receipt, &stale, &comparison).is_err());
        let mut rewritten = comparison;
        rewritten["merge_base_commit"]["sha"] = serde_json::json!("f".repeat(40));
        assert!(validate_merged_identity(&evidence, &receipt, &pr, &rewritten).is_err());
    }

    #[test]
    fn native_blockers_fail_closed_even_when_local_dependency_list_is_empty() {
        let base = serde_json::json!({"data":{"repository":{"issue":{"state":"OPEN","blockedBy":{"nodes":[],"pageInfo":{"hasNextPage":false}}}}}});
        validate_native_blockers(&base).unwrap();
        let mut blocked = base.clone();
        blocked["data"]["repository"]["issue"]["blockedBy"]["nodes"] =
            serde_json::json!([{"number":99,"state":"OPEN"}]);
        assert!(validate_native_blockers(&blocked).is_err());
        blocked["data"]["repository"]["issue"]["blockedBy"]["nodes"][0]["state"] =
            serde_json::json!("CLOSED");
        validate_native_blockers(&blocked).unwrap();
        blocked["data"]["repository"]["issue"]["blockedBy"]["pageInfo"]["hasNextPage"] =
            serde_json::json!(true);
        assert!(validate_native_blockers(&blocked).is_err());
        assert!(validate_native_blockers(
            &serde_json::json!({"errors":[{"message":"unsupported"}]})
        )
        .is_err());
        assert!(validate_native_blockers(&serde_json::json!({})).is_err());
    }

    struct GitFixture {
        root: PathBuf,
        one: PathBuf,
        two: PathBuf,
    }
    impl GitFixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "uor-coord-cas-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            let remote = root.join("remote.git");
            fs::create_dir_all(&remote).unwrap();
            git(&remote, &["init", "--bare", "--quiet"], None, None).unwrap();
            let one = root.join("one.git");
            let two = root.join("two.git");
            for store in [&one, &two] {
                fs::create_dir(store).unwrap();
                git(store, &["init", "--bare", "--quiet"], None, None).unwrap();
                git(
                    store,
                    &["config", "user.name", "Coordination Test"],
                    None,
                    None,
                )
                .unwrap();
                git(
                    store,
                    &["config", "user.email", "coordination-test@example.invalid"],
                    None,
                    None,
                )
                .unwrap();
                git(store, &["config", "uor.coordination", "1"], None, None).unwrap();
                git(
                    store,
                    &["remote", "add", "origin", remote.to_str().unwrap()],
                    None,
                    None,
                )
                .unwrap();
            }
            let mut initial = state();
            initial.transition(&register("one"), 1).unwrap();
            initial.transition(&register("two"), 2).unwrap();
            let head = commit(&one, None, &initial, None, 2).unwrap();
            git(
                &one,
                &["push", "origin", &format!("{head}:refs/heads/{BRANCH}")],
                None,
                None,
            )
            .unwrap();
            Self { root, one, two }
        }
    }
    impl Drop for GitFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn independent_git_contenders_cannot_both_commit_the_same_claim() {
        let fixture = GitFixture::new();
        let head_one = fetch(&fixture.one).unwrap();
        let head_two = fetch(&fixture.two).unwrap();
        assert_eq!(head_one, head_two);
        let mut first = claim("one", None);
        first.event_id = "claim-one".into();
        let mut second = claim("two", None);
        second.event_id = "claim-two".into();
        let mut state_one = state_at(&fixture.one, &head_one).unwrap();
        let mut state_two = state_at(&fixture.two, &head_two).unwrap();
        state_one.transition(&first, 10).unwrap();
        state_two.transition(&second, 10).unwrap();
        let next_one = commit(&fixture.one, Some(&head_one), &state_one, Some(&first), 10).unwrap();
        let next_two =
            commit(&fixture.two, Some(&head_two), &state_two, Some(&second), 10).unwrap();
        publish(&fixture.one, &next_one, &first).unwrap();
        assert!(publish(&fixture.two, &next_two, &second).is_err());
        let actual = status(&fixture.two).unwrap();
        assert_eq!(actual.tasks[&1].session, "one");
        assert_eq!(actual.sequence, state_one.sequence);
        let mut rebased = actual;
        assert!(rebased.transition(&second, 11).is_err());
    }

    #[test]
    fn lost_successful_push_reply_is_recovered_without_a_second_event() {
        let fixture = GitFixture::new();
        let head = fetch(&fixture.one).unwrap();
        let e = claim("one", None);
        let mut next_state = state_at(&fixture.one, &head).unwrap();
        next_state.transition(&e, 10).unwrap();
        let next = commit(&fixture.one, Some(&head), &next_state, Some(&e), 10).unwrap();
        git(
            &fixture.one,
            &["push", "origin", &format!("{next}:refs/heads/{BRANCH}")],
            None,
            None,
        )
        .unwrap();
        let recovered = reconcile_push(
            &fixture.one,
            &e,
            Err(invalid("simulated lost transport response")),
        )
        .unwrap();
        assert_eq!(recovered, next);
        assert!(replayed_event(&fixture.two, &fetch(&fixture.two).unwrap(), &e).unwrap());
        assert_eq!(status(&fixture.two).unwrap().sequence, next_state.sequence);
        let mut reused = e;
        reused.action = Action::Heartbeat {
            session: "one".into(),
            available: true,
        };
        assert!(replayed_event(&fixture.two, &next, &reused).is_err());
    }
}
