//! Coordinator-enforced protected delivery. Receipts are procedural evidence,
//! not cryptographic proof of independent reviewers sharing one GitHub account.
//! No status is manufactured. Server enforcement remains a separate admin step.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{invalid, ledger::sha256, Result};

pub const SCHEMA: &str = "uor-r4.delivery-receipt/1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileEvidence {
    pub path: PathBuf,
    pub sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckReceipt {
    pub name: String,
    pub head_sha: String,
    pub base_sha: String,
    pub argv: Vec<String>,
    pub exit_code: i32,
    pub outcome: String,
    pub log: FileEvidence,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewReceipt {
    pub review_id: String,
    pub reviewer_session: String,
    pub launched_by: String,
    pub head_sha: String,
    pub base_sha: String,
    pub decision: String,
    pub unresolved_required_fixes: u32,
    pub evidence: FileEvidence,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CouncilVote {
    pub reviewer_session: String,
    pub decision: String,
    pub reasons: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CouncilReceipt {
    pub decision_id: String,
    pub head_sha: String,
    pub base_sha: String,
    pub votes: Vec<CouncilVote>,
    pub evidence: FileEvidence,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeliveryReceipt {
    pub schema: String,
    pub repository: String,
    pub pull_request: u64,
    pub task_issue: u64,
    pub author_session: String,
    pub claim_session: String,
    pub claim_epoch: u64,
    pub work_card: String,
    pub head_sha: String,
    pub base_sha: String,
    /// docs, code, or model; derives minimum real local check names.
    pub change_kind: String,
    /// A=lane work, B=measured results, C=shared interfaces/policy.
    pub class: String,
    pub checks: Vec<CheckReceipt>,
    pub reviews: Vec<ReviewReceipt>,
    /// B and model changes need independently reread, artifact-bound evidence.
    pub result_evidence: Option<FileEvidence>,
    pub result_reader_session: Option<String>,
    pub council: Option<CouncilReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryDecision {
    pub eligible: bool,
    pub repository: String,
    pub pull_request: u64,
    pub head_sha: String,
    pub base_sha: String,
    pub gate: String,
    pub limitations: Vec<String>,
}

fn nonempty(value: &str, name: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(invalid(format!("delivery {name} must be nonempty")));
    }
    Ok(())
}
fn sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
fn repository(value: &str) -> bool {
    let parts: Vec<_> = value.split('/').collect();
    parts.len() == 2
        && parts.iter().all(|p| {
            !p.is_empty()
                && !p.starts_with('-')
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        })
}
fn verify_file(evidence: &FileEvidence) -> Result<()> {
    if !evidence.path.is_absolute() {
        return Err(invalid("delivery evidence path must be absolute"));
    }
    let metadata = fs::symlink_metadata(&evidence.path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(invalid(
            "delivery evidence must be a regular file, not a link",
        ));
    }
    if metadata.len() == 0 || metadata.len() > 64 * 1024 * 1024 {
        return Err(invalid(
            "delivery evidence must be nonempty and at most 64 MiB",
        ));
    }
    if sha256(&fs::read(&evidence.path)?) != evidence.sha256 {
        return Err(invalid("delivery evidence digest mismatch"));
    }
    Ok(())
}
fn bound(head: &str, base: &str, receipt: &DeliveryReceipt) -> Result<()> {
    if head != receipt.head_sha || base != receipt.base_sha {
        return Err(invalid("stale receipt: head or base differs"));
    }
    Ok(())
}

/// Validate local evidence without GitHub changes. Check names describe the
/// declared validation scope; source binding and nonzero/unknown outcomes fail.
pub fn validate(receipt: &DeliveryReceipt) -> Result<DeliveryDecision> {
    if receipt.schema != SCHEMA
        || !repository(&receipt.repository)
        || receipt.pull_request == 0
        || receipt.task_issue == 0
    {
        return Err(invalid(
            "invalid delivery schema, repository or PR identity",
        ));
    }
    if !sha(&receipt.head_sha) || !sha(&receipt.base_sha) {
        return Err(invalid("delivery needs full lowercase commit SHAs"));
    }
    nonempty(&receipt.author_session, "author_session")?;
    nonempty(&receipt.claim_session, "claim_session")?;
    nonempty(&receipt.work_card, "work_card")?;
    crate::coord::work_card_digest(&receipt.work_card)?;
    if receipt.claim_epoch == 0 {
        return Err(invalid("delivery needs a nonzero task generation"));
    }
    if !["A", "B", "C"].contains(&receipt.class.as_str()) {
        return Err(invalid("delivery class must be A, B or C"));
    }
    let required: &[&str] = match receipt.change_kind.as_str() {
        "docs" => &["diff-check", "claim-wording"],
        "code" => &["diff-check", "format", "compile", "focused-tests"],
        "model" => &[
            "diff-check",
            "format",
            "compile",
            "focused-tests",
            "loaded-behavior",
        ],
        _ => return Err(invalid("change_kind must be docs, code or model")),
    };
    let mut checks = BTreeSet::new();
    for check in &receipt.checks {
        bound(&check.head_sha, &check.base_sha, receipt)?;
        if !checks.insert(check.name.as_str()) {
            return Err(invalid("duplicate validation check name"));
        }
        if check.exit_code != 0
            || check.outcome != "PASS"
            || check.argv.is_empty()
            || check.argv[0].trim().is_empty()
        {
            return Err(invalid(
                "required local validation is failed, unavailable, unrun or lacks its command",
            ));
        }
        verify_file(&check.log)?;
    }
    for name in required {
        if !checks.contains(name) {
            return Err(invalid(format!("missing required local check: {name}")));
        }
    }
    let mut review_ids = BTreeSet::new();
    let mut reviewers = BTreeSet::new();
    for review in &receipt.reviews {
        bound(&review.head_sha, &review.base_sha, receipt)?;
        nonempty(&review.review_id, "review_id")?;
        nonempty(&review.reviewer_session, "reviewer_session")?;
        nonempty(&review.launched_by, "launched_by")?;
        if !review_ids.insert(review.review_id.as_str()) {
            return Err(invalid("duplicate review identity"));
        }
        if review.reviewer_session == receipt.author_session {
            return Err(invalid(
                "author session cannot provide independent approval",
            ));
        }
        if review.decision != "APPROVE" || review.unresolved_required_fixes != 0 {
            return Err(invalid("review has not approved all required fixes"));
        }
        verify_file(&review.evidence)?;
        reviewers.insert(review.reviewer_session.as_str());
    }
    if reviewers.is_empty() {
        return Err(invalid("an independent technical review is required"));
    }
    if receipt.class == "B" || receipt.change_kind == "model" || receipt.result_evidence.is_some() {
        let evidence = receipt
            .result_evidence
            .as_ref()
            .ok_or_else(|| invalid("result requires artifact-bound evidence"))?;
        verify_file(evidence)?;
        let reader = receipt
            .result_reader_session
            .as_deref()
            .ok_or_else(|| invalid("result lacks an independent evidence reader"))?;
        if reader == receipt.author_session || !reviewers.contains(reader) {
            return Err(invalid(
                "result reader must be a recorded non-author reviewer",
            ));
        }
    }
    if receipt.class == "C" {
        let council = receipt
            .council
            .as_ref()
            .ok_or_else(|| invalid("class C requires a recorded council decision"))?;
        bound(&council.head_sha, &council.base_sha, receipt)?;
        nonempty(&council.decision_id, "council decision_id")?;
        verify_file(&council.evidence)?;
        if council.votes.len() != 3 {
            return Err(invalid("council needs exactly three decision seats"));
        }
        let mut voters = BTreeSet::new();
        let mut approvals = 0;
        let mut nonauthors = 0;
        for vote in &council.votes {
            nonempty(&vote.reviewer_session, "council reviewer_session")?;
            nonempty(&vote.reasons, "council reasons")?;
            if !voters.insert(vote.reviewer_session.as_str()) {
                return Err(invalid("council seats must be distinct reviewer passes"));
            }
            if vote.reviewer_session != receipt.author_session {
                nonauthors += 1;
            }
            match vote.decision.as_str() {
                "APPROVE" => approvals += 1,
                "REJECT" => {}
                _ => return Err(invalid("council vote must approve or reject")),
            }
        }
        if approvals < 2 || nonauthors < 2 {
            return Err(invalid(
                "council requires 2/3 approval and two non-author seats",
            ));
        }
    }
    Ok(DeliveryDecision { eligible:true, repository:receipt.repository.clone(),pull_request:receipt.pull_request,head_sha:receipt.head_sha.clone(),base_sha:receipt.base_sha.clone(),gate:"coordinator-enforced; server requirement not assumed".into(),limitations:vec!["Shared-account reviewer independence is procedural, not cryptographic attestation.".into(),"Queue integration or a later base change can require fresh checks; PR checks alone do not establish combined behavior.".into()] })
}

fn gh(args: &[String]) -> Result<Vec<u8>> {
    let output = Command::new("gh").args(args).output()?;
    if !output.status.success() {
        return Err(invalid(format!(
            "gh failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(output.stdout)
}
fn gh_json(args: &[String]) -> Result<Value> {
    Ok(serde_json::from_slice(&gh(args)?)?)
}

/// A source PR cannot bypass compilation by calling itself documentation.
pub fn validate_scope(receipt: &DeliveryReceipt, files: &[String]) -> Result<()> {
    if files.is_empty() {
        return Err(invalid("PR changed-file inventory is empty"));
    }
    let docs_only = files
        .iter()
        .all(|p| p.ends_with(".md") || (p.starts_with("docs/") && p.ends_with(".json")));
    if !docs_only && receipt.change_kind == "docs" {
        return Err(invalid(
            "source changes cannot use documentation-only validation",
        ));
    }
    let model = files.iter().any(|p| {
        p.ends_with(".rs")
            && ([
                "crates/uor-r4-core/src/native_geometric/",
                "crates/uor-r4-stack/src/",
                "crates/uor-r4-training/src/",
                "crates/uor-r4-integer/src/",
                "crates/uor-r4-api/src/",
            ]
            .iter()
            .any(|prefix| p.starts_with(prefix))
                || [
                    "src/native_geometric_cli.rs",
                    "src/service.rs",
                    "src/native_wasm.rs",
                ]
                .contains(&p.as_str()))
    });
    if model && receipt.change_kind != "model" {
        return Err(invalid(
            "model/serving source changes need loaded-behavior validation",
        ));
    }
    let shared_policy = files.iter().any(|p| {
        p == "AGENTS.md"
            || p == "docs/integration/DECISIONS.md"
            || p.starts_with("docs/integration/agent-execution-policy")
            || p == "docs/labs/protocol.md"
            || p == "docs/labs/operations.md"
            || p.starts_with("tools/lab-runner/src/")
            || p.starts_with(".github/workflows/")
    });
    if shared_policy && receipt.class != "C" {
        return Err(invalid(
            "shared policy/workflow changes require council class C",
        ));
    }
    Ok(())
}

/// The delivery steward must own the current task generation. A source review
/// does not authorize an expired session to release a successor's work.
pub fn validate_claim(receipt: &DeliveryReceipt, state: &Value, now: u64) -> Result<()> {
    if state["schema"] != "uor-r4.lab-state/1" || state["repository"] != receipt.repository {
        return Err(invalid("delivery coordination identity is unavailable"));
    }
    let task = &state["tasks"][receipt.task_issue.to_string()];
    if task["session"] != receipt.claim_session
        || task["epoch"].as_u64() != Some(receipt.claim_epoch)
        || task["work_card"] != receipt.work_card
        || task["policy_sha"].as_str().is_none()
        || task["policy_sha"] != state["policy_sha"]
        || task["phase"] != "claimed"
        || task["expires"]
            .as_u64()
            .is_none_or(|expires| expires <= now)
    {
        return Err(invalid(
            "delivery task generation expired, changed or is unclaimed",
        ));
    }
    let lab = &state["labs"][&receipt.claim_session];
    if lab["available"] != true
        || lab["heartbeat"]
            .as_u64()
            .is_none_or(|t| t > now || now.saturating_sub(t) >= 1200)
    {
        return Err(invalid(
            "delivery steward is unavailable or has a stale heartbeat",
        ));
    }
    Ok(())
}

fn check_remote_claim(receipt: &DeliveryReceipt) -> Result<()> {
    let reference = gh_json(&[
        "api".into(),
        format!("repos/{}/git/ref/heads/codex/lab-state", receipt.repository),
    ])?;
    let head = reference["object"]["sha"]
        .as_str()
        .filter(|s| sha(s))
        .ok_or_else(|| invalid("coordination ref lacks a full SHA"))?;
    let state = gh_json(&[
        "api".into(),
        "-H".into(),
        "Accept: application/vnd.github.raw+json".into(),
        format!(
            "repos/{}/contents/state.json?ref={head}",
            receipt.repository
        ),
    ])?;
    let now = crate::coord::github_time(&receipt.repository)?;
    crate::coord::require_current_policy(
        &receipt.repository,
        state["policy_sha"]
            .as_str()
            .ok_or_else(|| invalid("missing operational policy revision"))?,
    )?;
    validate_claim(receipt, &state, now)
}

/// Pure live-state validator for deterministic tests. required_contexts must
/// come from effective main branch rules, never from the author's receipt.
pub fn validate_live(
    receipt: &DeliveryReceipt,
    pr: &Value,
    required_contexts: &[String],
    merge_queue_required: bool,
) -> Result<()> {
    if pr["headRefOid"] != receipt.head_sha || pr["baseRefOid"] != receipt.base_sha {
        return Err(invalid("live PR moved since review or tests"));
    }
    if pr["baseRefName"] != "main"
        || pr["state"] != "OPEN"
        || pr["isDraft"] != false
        || pr["mergeStateStatus"] != "CLEAN"
    {
        return Err(invalid("PR must be open, ready, CLEAN and targeting main"));
    }
    if !merge_queue_required || required_contexts.is_empty() {
        return Err(invalid(
            "effective main rules do not prove required checks and protected merge queue",
        ));
    }
    let checks = pr["statusCheckRollup"]
        .as_array()
        .ok_or_else(|| invalid("PR status checks unavailable"))?;
    let mut statuses = BTreeMap::new();
    for check in checks {
        let name = check["name"].as_str().or_else(|| check["context"].as_str());
        if let Some(name) = name {
            let passed = check["conclusion"] == "SUCCESS" || check["state"] == "SUCCESS";
            // Any duplicate incomplete/failing context prevents an ambiguous success.
            statuses
                .entry(name.to_string())
                .and_modify(|old: &mut bool| *old &= passed)
                .or_insert(passed);
        }
    }
    for name in required_contexts {
        if statuses.get(name) != Some(&true) {
            return Err(invalid(format!(
                "required GitHub context is not successful: {name}"
            )));
        }
    }
    Ok(())
}

pub fn read_receipt(path: &Path) -> Result<DeliveryReceipt> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
/// Refresh current PR and effective rules; this operation never posts a status.
pub fn check(path: &Path) -> Result<DeliveryDecision> {
    let receipt = read_receipt(path)?;
    let decision = validate(&receipt)?;
    let pages = gh_json(&[
        "api".into(),
        format!(
            "repos/{}/pulls/{}/files?per_page=100",
            receipt.repository, receipt.pull_request
        ),
        "--paginate".into(),
        "--slurp".into(),
    ])?;
    let mut files = Vec::new();
    for page in pages
        .as_array()
        .ok_or_else(|| invalid("changed-file pages unavailable"))?
    {
        for file in page
            .as_array()
            .ok_or_else(|| invalid("changed-file page malformed"))?
        {
            files.push(
                file["filename"]
                    .as_str()
                    .ok_or_else(|| invalid("changed file lacks name"))?
                    .to_string(),
            );
        }
    }
    validate_scope(&receipt, &files)?;
    let (owner, name) = receipt
        .repository
        .split_once('/')
        .ok_or_else(|| invalid("repository identity malformed"))?;
    let query=format!("query {{ repository(owner: {}, name: {}) {{ issue(number: {}) {{ state blockedBy(first:100) {{ nodes {{ number state }} pageInfo {{ hasNextPage }} }} }} }} }}",serde_json::to_string(owner)?,serde_json::to_string(name)?,receipt.task_issue);
    let task = gh_json(&[
        "api".into(),
        "graphql".into(),
        "-f".into(),
        format!("query={query}"),
    ])?;
    let issue = &task["data"]["repository"]["issue"];
    if task.get("errors").is_some()
        || issue["state"] != "OPEN"
        || issue["blockedBy"]["pageInfo"]["hasNextPage"] != false
    {
        return Err(invalid(
            "owning task or complete dependency state unavailable",
        ));
    }
    let blockers = issue["blockedBy"]["nodes"]
        .as_array()
        .ok_or_else(|| invalid("task dependencies unavailable"))?;
    if blockers.iter().any(|v| v["state"] != "CLOSED") {
        return Err(invalid("owning task still has an open native blocker"));
    }
    let rules = gh_json(&[
        "api".into(),
        format!("repos/{}/rules/branches/main", receipt.repository),
    ])?;
    let rules = rules
        .as_array()
        .ok_or_else(|| invalid("effective branch rules are unavailable"))?;
    let merge_queue = rules.iter().any(|v| v["type"] == "merge_queue");
    let mut required = BTreeSet::new();
    for rule in rules {
        if rule["type"] == "required_status_checks" {
            for item in rule["parameters"]["required_status_checks"]
                .as_array()
                .ok_or_else(|| invalid("required status rules are malformed"))?
            {
                required.insert(
                    item["context"]
                        .as_str()
                        .ok_or_else(|| invalid("required status lacks context"))?
                        .to_string(),
                );
            }
        }
    }
    let pr = gh_json(&[
        "pr".into(),
        "view".into(),
        receipt.pull_request.to_string(),
        "--repo".into(),
        receipt.repository.clone(),
        "--json".into(),
        "headRefOid,baseRefOid,baseRefName,state,isDraft,mergeStateStatus,statusCheckRollup".into(),
    ])?;
    validate_live(
        &receipt,
        &pr,
        &required.into_iter().collect::<Vec<_>>(),
        merge_queue,
    )?;
    check_remote_claim(&receipt)?;
    Ok(decision)
}

/// Explicitly requested delivery only: revalidate immediately before adding the
/// exact approved head to the existing queue. No --admin, --auto or direct push.
pub fn enqueue(path: &Path) -> Result<DeliveryDecision> {
    let decision = check(path)?;
    gh(&[
        "pr".into(),
        "merge".into(),
        decision.pull_request.to_string(),
        "--repo".into(),
        decision.repository.clone(),
        "--match-head-commit".into(),
        decision.head_sha.clone(),
    ])?;
    Ok(decision)
}
