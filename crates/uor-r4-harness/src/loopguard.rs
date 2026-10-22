//! `uor-harness loop guard`, `loop install`, `loop onboard` and `loop copyright` (loop PR 9).
//!
//! # Why these four live together
//!
//! Each of the five clients (`claude`, `codex`, `kimi`, `antigravity`, `deepseek`) wires hooks,
//! user-level skills and rules through its own configuration layout, and each reports subagents
//! differently in its hook payloads (probe P-1 on `#2028`):
//!
//! - **Claude Code (`claude`)**: fires `SessionStart`, `PreToolUse`, `SubagentStart`, `SubagentStop`
//!   and `Stop`. A subagent's `PreToolUse` carries `agent_id`, so the guard scopes subagents
//!   against `local/<lab>/<topic>/orders/<n>.md`.
//! - **Antigravity (`antigravity`)**: fires `SessionStart`, `PreInvocation`, `PreToolUse`,
//!   `PostToolUse`, `PostInvocation` and `Stop` in the parent and in subagents. A subagent's
//!   `PreToolUse` carries its own `conversationId`, so the guard records the root `conversationId`
//!   at the first `SessionStart` and identifies subagents by a differing `conversationId`.
//! - **Kimi Code (`kimi`)**: fires `SessionStart`, `TurnStarted`, `PreToolUse`, `PostToolUse`,
//!   `SubagentStart`, `SubagentStop` and `Stop`. Subagent tool calls carry the shared `session_id`
//!   with no agent id, so the guard tracks subagent depth between `SubagentStart` and
//!   `SubagentStop` per `session_id`.
//! - **Codex (`codex`)** and **DeepSeek (`deepseek`)**: evaluate the same path, scope and command
//!   guard on their hook or stream payloads, alongside Codex's rules file and DeepSeek's headless
//!   overlay.
//!
//! `loop guard` is the single hook entry point and exits `0` at once outside the UOR-R4 repository
//! and its worktrees so it never touches other projects. Inside the repository it refuses writes to
//! owner-only paths (`docs/harness/owner-paths.json`), the shared primary checkout (R10), another
//! lab's worktree or `local/` folder, rendered views outside `kimi/render`, existing `archive/`
//! files, and paths outside the active ORDER or subagent suborder scope, and it refuses release or
//! cap-override commands (D56) and forbidden git or process commands.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// The five registered client names, in canonical sorted order.
pub const CLIENTS: [&str; 5] = ["antigravity", "claude", "codex", "deepseek", "kimi"];

/// The six Loop Kit skills installed for every client (`start-here` is excluded so it can never
/// start a second setup).
pub const ALLOWED_SKILLS: [&str; 6] = [
    "gates",
    "holding-the-goal",
    "scope-guard",
    "session-close",
    "solution-integrity",
    "working-discipline",
];

/// The excluded first-run interview skill.
pub const EXCLUDED_SKILL: &str = "start-here";

const USAGE: &str = "\
usage: uor-harness loop guard --client <C> [--lab <L>] --event <E> [--input <path>]
       uor-harness loop install (--client <C> | --all) [--home <dir>] [--zip <path>] [--apply] [--replace] [--dry-run] [--json]
       uor-harness loop onboard (--client <C> | --check-receipts) [--home <dir>] [--write-receipt] [--json]
       uor-harness loop copyright [--zip <path>] [--json]";

/// Resolved UOR-R4 roots for a directory inside the repository or one of its worktrees.
#[derive(Debug, Clone)]
pub struct UorRoots {
    /// The specific checkout or linked worktree containing the directory.
    pub worktree_root: PathBuf,
    /// The primary repository root (which holds `.worktrees/` and shared `local/`), or
    /// `worktree_root` when standalone.
    pub primary_root: PathBuf,
}

/// Whether `dir` is the root of a UOR-R4 checkout or worktree.
fn is_uor_root(dir: &Path) -> bool {
    dir.join("CHARTER.md").is_file() && dir.join("docs/harness/labs.json").is_file()
}

/// Find the UOR-R4 worktree and primary roots for `dir`, or `None` when `dir` is outside the
/// repository and its worktrees.
pub fn find_uor_roots(dir: &Path) -> Option<UorRoots> {
    let canon = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    for anc in canon.ancestors() {
        if is_uor_root(anc) {
            let worktree_root = anc.to_path_buf();
            // If this worktree lives at `<primary>/.worktrees/<name>`, `<primary>` is two levels up.
            let primary_root = worktree_root
                .parent()
                .filter(|p| p.file_name().and_then(|n| n.to_str()) == Some(".worktrees"))
                .and_then(Path::parent)
                .filter(|p| is_uor_root(p))
                .map(Path::to_path_buf)
                .unwrap_or_else(|| worktree_root.clone());
            return Some(UorRoots {
                worktree_root,
                primary_root,
            });
        }
    }
    None
}

/// Whether `dir` lies inside the UOR-R4 repository or one of its worktrees.
pub fn in_uor_repo(dir: &Path) -> bool {
    find_uor_roots(dir).is_some()
}

/// Outcome of evaluating `loop guard`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardOutcome {
    /// The hook fired outside the UOR-R4 repository and its worktrees; exit 0 immediately.
    OutsideRepo,
    /// The event or tool call is permitted.
    Allow,
    /// The tool call or invocation is refused with a human-readable reason.
    Refused(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SubagentContext {
    TopLevel,
    Subagent(Option<String>),
}

/// Check that `client` is one of the five supported clients.
pub fn check_client(client: &str) -> Result<String, String> {
    let slug = crate::state::slug(client);
    if CLIENTS.contains(&slug.as_str()) {
        Ok(slug)
    } else {
        Err(format!(
            "unknown client {client:?}; expected one of {}",
            CLIENTS.join(", ")
        ))
    }
}

/// The lane assigned to each lab on `docs/loop/README.md`.
pub fn client_lane(client: &str) -> &'static str {
    match client {
        "claude" => "loop architecture, core gates and setup reviews (D52, D59)",
        "codex" => "item validator, gates runner, policy check and review records (PRs 2, 4, 5, 6)",
        "kimi" => "PM, rendered views, notes migration and render refresh (PRs 3, 8, 10)",
        "antigravity" => "client adapters: loop guard, install, onboard and copyright (PR 9)",
        "deepseek" => {
            "claim, suborder, ask, session loop, queue and watch (PRs 7a, 7b, 7c, 11, 12)"
        }
        _ => "general",
    }
}

/// The user-level skills directory for `client` under `home`.
pub fn client_skills_dir(home: &Path, client: &str) -> PathBuf {
    match client {
        "claude" => home.join(".claude").join("skills"),
        "codex" => home.join(".codex").join("skills"),
        "kimi" => home.join(".agents").join("skills"),
        "antigravity" => home.join(".gemini").join("config").join("skills"),
        "deepseek" => home.join(".dsh").join("skills"),
        _ => home.join(format!(".{client}")).join("skills"),
    }
}

/// Relative display path (under `~`) for the client's skills directory.
pub fn client_skills_rel(client: &str) -> &'static str {
    match client {
        "claude" => "~/.claude/skills",
        "codex" => "~/.codex/skills",
        "kimi" => "~/.agents/skills",
        "antigravity" => "~/.gemini/config/skills",
        "deepseek" => "~/.dsh/skills",
        _ => "~/.skills",
    }
}

/// Guard state directory for `lab` inside `roots`.
fn guard_state_dir(roots: &UorRoots, lab: &str) -> PathBuf {
    roots.worktree_root.join("local").join(lab).join("guard")
}

/// Write `bytes` atomically to `path` via a sibling temporary file and rename.
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    let tmp = path.with_extension(format!(
        "tmp.{}.{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::write(&tmp, bytes).map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("cannot rename {} to {}: {e}", tmp.display(), path.display())
    })
}

/// Execute `f` while holding an exclusive advisory lockfile in `gdir`.
///
/// Fails closed (`Err`) if the lock cannot be acquired within the retry budget, and renames stale
/// locks (> 5s old) to a unique path before unlinking so concurrent waiters cannot delete a newly
/// acquired peer lock.
fn with_guard_lock<T>(gdir: &Path, f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    fs::create_dir_all(gdir).map_err(|e| format!("cannot create {}: {e}", gdir.display()))?;
    let lock_path = gdir.join(".lock");
    let mut acquired = false;
    for attempt in 0..100 {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(_) => {
                acquired = true;
                break;
            }
            Err(_) => {
                if attempt == 50 {
                    // Break stale lock older than 5 seconds by renaming to a unique path first.
                    if let Ok(meta) = fs::metadata(&lock_path) {
                        if let Ok(modified) = meta.modified() {
                            if modified.elapsed().map(|d| d.as_secs() >= 5).unwrap_or(true) {
                                let stale_path = gdir.join(format!(
                                    ".lock.stale.{}.{}",
                                    std::process::id(),
                                    std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .map(|d| d.as_nanos())
                                        .unwrap_or(0)
                                ));
                                if fs::rename(&lock_path, &stale_path).is_ok() {
                                    let _ = fs::remove_file(&stale_path);
                                    continue;
                                }
                            }
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }
    }
    if !acquired {
        return Err(format!(
            "cannot acquire guard lock {}: lock contention timeout (fail-closed)",
            lock_path.display()
        ));
    }
    let res = f();
    let _ = fs::remove_file(&lock_path);
    res
}

/// Persisted Antigravity session state so stale `root-conversation.id` files from killed sessions
/// are replaced on a new top-level `SessionStart`, while child subagent `SessionStart` events keep
/// the parent root.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct AntigravitySessionState {
    root_cid: String,
    pending_subagents: u32,
    root_tool_calls: u32,
    #[serde(default)]
    subagent_cids: Vec<String>,
}

/// Sanitize an identifier for use in a file name.
fn safe_id(raw: &str) -> String {
    let s: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if s.is_empty() {
        "default".to_string()
    } else {
        s
    }
}

/// Extract the working directory named by the hook payload, falling back to `fallback`.
fn payload_cwd(payload: &Value, fallback: &Path) -> PathBuf {
    if let Some(cwd) = payload
        .get("cwd")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
    {
        return PathBuf::from(cwd);
    }
    if let Some(first) = payload
        .get("workspacePaths")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
    {
        return PathBuf::from(first);
    }
    fallback.to_path_buf()
}

/// Record session/subagent lifecycle transitions and return whether the current event is inside a
/// subagent.
fn update_and_detect_subagent(
    roots: &UorRoots,
    client: &str,
    lab: &str,
    event: &str,
    payload: &Value,
) -> Result<SubagentContext, String> {
    let gdir = guard_state_dir(roots, lab);
    match client {
        "antigravity" => with_guard_lock(&gdir, || {
            let cid = payload
                .get("conversationId")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty());
            let is_sub_flag = payload.get("isSubagent").and_then(Value::as_bool) == Some(true);
            let state_file = gdir.join("antigravity-session.json");
            let root_file = gdir.join("root-conversation.id");

            let mut state: AntigravitySessionState = fs::read_to_string(&state_file)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_else(|| {
                    let legacy_root = fs::read_to_string(&root_file)
                        .ok()
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();
                    AntigravitySessionState {
                        root_cid: legacy_root,
                        pending_subagents: 0,
                        root_tool_calls: 0,
                        subagent_cids: Vec::new(),
                    }
                });

            match event {
                "SessionStart" => {
                    if let Some(c) = cid {
                        if is_sub_flag {
                            state.pending_subagents = state.pending_subagents.saturating_sub(1);
                            if !state.subagent_cids.iter().any(|x| x == c) {
                                state.subagent_cids.push(c.to_string());
                            }
                        } else if state.root_cid.is_empty()
                            || state.root_cid == c
                            || state.pending_subagents == 0
                        {
                            // Either no prior root exists, or no subagent spawn is pending from the
                            // previous root (even if it exited or crashed with 0 tool calls), so
                            // this SessionStart begins a fresh top-level session.
                            state.root_cid = c.to_string();
                            state.pending_subagents = 0;
                            state.root_tool_calls = 0;
                            state.subagent_cids.clear();
                        } else {
                            // A subagent SessionStart spawned by the active root session.
                            state.pending_subagents = state.pending_subagents.saturating_sub(1);
                            if !state.subagent_cids.iter().any(|x| x == c) {
                                state.subagent_cids.push(c.to_string());
                            }
                        }
                        let serialized = serde_json::to_vec_pretty(&state)
                            .map_err(|e| format!("cannot serialize state: {e}"))?;
                        atomic_write(&state_file, &serialized)?;
                        atomic_write(&root_file, format!("{}\n", state.root_cid).as_bytes())?;
                    }
                    if is_sub_flag
                        || cid.is_some_and(|c| !state.root_cid.is_empty() && state.root_cid != c)
                    {
                        return Ok(SubagentContext::Subagent(cid.map(String::from)));
                    }
                    Ok(SubagentContext::TopLevel)
                }
                "Stop" => {
                    if let Some(c) = cid {
                        if state.root_cid == c {
                            let _ = fs::remove_file(&state_file);
                            let _ = fs::remove_file(&root_file);
                            return Ok(SubagentContext::TopLevel);
                        }
                        state.subagent_cids.retain(|x| x != c);
                        if let Ok(serialized) = serde_json::to_vec_pretty(&state) {
                            let _ = atomic_write(&state_file, &serialized);
                        }
                        return Ok(SubagentContext::Subagent(Some(c.to_string())));
                    }
                    Ok(SubagentContext::TopLevel)
                }
                _ => {
                    if is_sub_flag {
                        return Ok(SubagentContext::Subagent(cid.map(String::from)));
                    }
                    // Fail closed if no root session has been established via SessionStart.
                    if state.root_cid.is_empty() {
                        return Ok(SubagentContext::Subagent(cid.map(String::from)));
                    }
                    if let Some(c) = cid {
                        if c != state.root_cid {
                            return Ok(SubagentContext::Subagent(Some(c.to_string())));
                        }
                        if event == "PreToolUse" {
                            let (tname, _) = extract_tool(payload);
                            if tname == "invoke_subagent" {
                                state.pending_subagents += 1;
                            } else {
                                state.root_tool_calls += 1;
                            }
                            if let Ok(serialized) = serde_json::to_vec_pretty(&state) {
                                let _ = atomic_write(&state_file, &serialized);
                            }
                        }
                        return Ok(SubagentContext::TopLevel);
                    }
                    Ok(SubagentContext::Subagent(None))
                }
            }
        }),
        "kimi" => with_guard_lock(&gdir, || {
            let sid = safe_id(
                payload
                    .get("session_id")
                    .and_then(Value::as_str)
                    .unwrap_or("default"),
            );
            let depth_file = gdir.join(format!("kimi-{sid}.depth"));
            let cur_depth: u32 = fs::read_to_string(&depth_file)
                .ok()
                .and_then(|s| s.trim().parse::<u32>().ok())
                .unwrap_or(0);
            match event {
                "SessionStart" => {
                    atomic_write(&depth_file, b"0\n")?;
                    Ok(SubagentContext::TopLevel)
                }
                "SubagentStart" => {
                    let next = cur_depth + 1;
                    atomic_write(&depth_file, format!("{next}\n").as_bytes())?;
                    Ok(SubagentContext::Subagent(Some(format!("depth-{next}"))))
                }
                "SubagentStop" => {
                    let next = cur_depth.saturating_sub(1);
                    atomic_write(&depth_file, format!("{next}\n").as_bytes())?;
                    Ok(if next > 0 {
                        SubagentContext::Subagent(Some(format!("depth-{next}")))
                    } else {
                        SubagentContext::TopLevel
                    })
                }
                "Stop" => {
                    let _ = fs::remove_file(&depth_file);
                    Ok(SubagentContext::TopLevel)
                }
                _ => {
                    let origin_sub = payload
                        .get("origin_name")
                        .and_then(Value::as_str)
                        .map(|s| s.eq_ignore_ascii_case("subagent"))
                        .unwrap_or(false);
                    if cur_depth > 0 {
                        Ok(SubagentContext::Subagent(Some(format!(
                            "depth-{cur_depth}"
                        ))))
                    } else if origin_sub {
                        Ok(SubagentContext::Subagent(Some("depth-1".into())))
                    } else {
                        Ok(SubagentContext::TopLevel)
                    }
                }
            }
        }),
        "claude" | "codex" | "deepseek" => with_guard_lock(&gdir, || {
            let aid = payload
                .get("agent_id")
                .or_else(|| payload.get("subagent_id"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from);
            let bindings_file = gdir.join("subagent-bindings.json");
            let depth_file = gdir.join(format!("{client}-unnamed.depth"));
            let cur_depth: u32 = fs::read_to_string(&depth_file)
                .ok()
                .and_then(|s| s.trim().parse::<u32>().ok())
                .unwrap_or(0);
            match event {
                "SessionStart" if aid.is_none() => {
                    let _ = atomic_write(&depth_file, b"0\n");
                }
                "SubagentStart" => {
                    if let Some(ref agent_id) = aid {
                        let mut bindings: BTreeMap<String, String> =
                            fs::read_to_string(&bindings_file)
                                .ok()
                                .and_then(|s| serde_json::from_str(&s).ok())
                                .unwrap_or_default();
                        if !bindings.contains_key(agent_id) {
                            if let Some(sub_file) =
                                claim_next_suborder_for_agent(roots, lab, agent_id, &bindings)
                            {
                                bindings.insert(agent_id.clone(), sub_file);
                                if let Ok(bytes) = serde_json::to_vec_pretty(&bindings) {
                                    let _ = atomic_write(&bindings_file, &bytes);
                                }
                            }
                        }
                    } else {
                        let next = cur_depth + 1;
                        let _ = atomic_write(&depth_file, format!("{next}\n").as_bytes());
                        return Ok(SubagentContext::Subagent(None));
                    }
                }
                "SubagentStop" => {
                    if let Some(ref agent_id) = aid {
                        let mut bindings: BTreeMap<String, String> =
                            fs::read_to_string(&bindings_file)
                                .ok()
                                .and_then(|s| serde_json::from_str(&s).ok())
                                .unwrap_or_default();
                        if bindings.remove(agent_id).is_some() {
                            if let Ok(bytes) = serde_json::to_vec_pretty(&bindings) {
                                let _ = atomic_write(&bindings_file, &bytes);
                            }
                        }
                    } else {
                        let next = cur_depth.saturating_sub(1);
                        let _ = atomic_write(&depth_file, format!("{next}\n").as_bytes());
                    }
                }
                _ => {}
            }
            if let Some(agent_id) = aid {
                return Ok(SubagentContext::Subagent(Some(agent_id)));
            }
            if cur_depth > 0
                || payload.get("is_subagent").and_then(Value::as_bool) == Some(true)
                || payload.get("origin_name").and_then(Value::as_str) == Some("subagent")
            {
                return Ok(SubagentContext::Subagent(None));
            }
            Ok(SubagentContext::TopLevel)
        }),
        _ => Ok(SubagentContext::TopLevel),
    }
}

/// Extract `(tool_name, tool_args)` from a hook payload.
fn extract_tool(payload: &Value) -> (String, Value) {
    let name = payload
        .get("tool_name")
        .and_then(Value::as_str)
        .or_else(|| {
            payload
                .get("toolCall")
                .and_then(|tc| tc.get("name"))
                .and_then(Value::as_str)
        })
        .or_else(|| payload.get("name").and_then(Value::as_str))
        .unwrap_or("")
        .to_string();
    let args = payload
        .get("tool_input")
        .or_else(|| payload.get("toolCall").and_then(|tc| tc.get("args")))
        .or_else(|| payload.get("input"))
        .or_else(|| payload.get("arguments"))
        .cloned()
        .unwrap_or(Value::Null);
    (name, args)
}

/// Whether `tool_name` is a shell/command execution tool.
fn is_shell_tool(name: &str) -> bool {
    matches!(
        name,
        "Bash"
            | "bash"
            | "run_command"
            | "shell"
            | "Shell"
            | "exec"
            | " exec_command"
            | "exec_command"
            | "terminal"
            | "execute_command"
    )
}

/// Whether `tool_name` is a read-only tool.
fn is_readonly_tool(name: &str) -> bool {
    matches!(
        name,
        "Read"
            | "read_file"
            | "view_file"
            | "Glob"
            | "Grep"
            | "LS"
            | "list_dir"
            | "find_by_name"
            | "grep_search"
    )
}

/// Whether `tool_name` is a file write/edit/patch/delete tool.
fn is_write_tool(name: &str) -> bool {
    matches!(
        name,
        "Write"
            | "Edit"
            | "MultiEdit"
            | "NotebookEdit"
            | "write_file"
            | "edit_file"
            | "create_file"
            | "replace_in_file"
            | "apply_patch"
            | "delete_file"
            | "str_replace_editor"
            | "patch_file"
    )
}

/// Extract the command string from tool arguments, if present.
fn extract_command(args: &Value) -> Option<String> {
    for key in ["command", "CommandLine", "cmd", "script"] {
        if let Some(s) = args.get(key).and_then(Value::as_str) {
            return Some(s.to_string());
        }
    }
    if let Some(arr) = args.get("cmd").and_then(Value::as_array) {
        let joined: Vec<&str> = arr.iter().filter_map(Value::as_str).collect();
        if !joined.is_empty() {
            return Some(joined.join(" "));
        }
    }
    None
}

/// Extract target file paths from a tool's arguments.
fn extract_target_paths(tool_name: &str, args: &Value) -> Vec<String> {
    let mut out = Vec::new();
    for key in [
        "file_path",
        "Path",
        "path",
        "target_file",
        "TargetFile",
        "notebook_path",
        "destination",
    ] {
        if let Some(s) = args
            .get(key)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
        {
            out.push(s.to_string());
        }
    }
    if let Some(edits) = args.get("edits").and_then(Value::as_array) {
        for e in edits {
            for key in ["file_path", "Path", "path", "target_file"] {
                if let Some(s) = e.get(key).and_then(Value::as_str).filter(|s| !s.is_empty()) {
                    out.push(s.to_string());
                }
            }
        }
    }
    if (tool_name == "apply_patch" || args.get("patch").is_some()) && out.is_empty() {
        if let Some(patch) = args
            .get("patch")
            .or_else(|| args.get("input"))
            .and_then(Value::as_str)
        {
            for line in patch.lines() {
                for prefix in [
                    "*** Update File: ",
                    "*** Add File: ",
                    "*** Delete File: ",
                    "+++ b/",
                ] {
                    if let Some(rest) = line.strip_prefix(prefix) {
                        let p = rest.trim();
                        if !p.is_empty() && p != "/dev/null" {
                            out.push(p.to_string());
                        }
                    }
                }
            }
        }
    }
    out
}

/// Whether `path` names a sensitive credential or secret file (CHARTER §3.13).
fn is_credential_path(path: &str) -> bool {
    let stripped = path
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .strip_prefix("of=")
        .or_else(|| path.strip_prefix("--file="))
        .or_else(|| path.strip_prefix("--config="))
        .or_else(|| path.strip_prefix("file://"))
        .unwrap_or(path);
    let mut lower = stripped.replace('\\', "/").to_lowercase();
    while lower.contains("//") {
        lower = lower.replace("//", "/");
    }
    while lower.contains("/./") {
        lower = lower.replace("/./", "/");
    }
    let lower = lower.trim_end_matches('/');
    // Public SSH keys (`*.pub`) are not secret credentials.
    if lower.ends_with(".pub") {
        return false;
    }
    let base = lower.rsplit('/').next().unwrap_or(lower);
    let is_env_secret = base == ".env"
        || (base.starts_with(".env.")
            && !matches!(base, ".env.example" | ".env.sample" | ".env.template"));
    is_env_secret
        || matches!(
            base,
            ".netrc"
                | ".git-credentials"
                | ".npmrc"
                | ".pypirc"
                | ".vast_api_key"
                | "id_rsa"
                | "id_ed25519"
                | "id_ecdsa"
                | "id_dsa"
                | "credentials.json"
                | "auth.json"
                | "client_secret.json"
                | "service_account.json"
                | ".aws"
                | ".runpod"
                | ".huggingface"
        )
        || lower.contains("/.ssh/id_")
        || lower.contains("/.aws/")
        || lower.ends_with("/.aws")
        || lower.contains("/.config/gh/hosts.yml")
        || lower.contains("/.runpod/")
        || lower.ends_with("/.runpod")
        || lower.contains("/.cargo/credentials")
        || lower.contains("/.huggingface/token")
        || lower.contains("/huggingface/token")
        || lower.contains("/.docker/config.json")
        || (lower.starts_with("/proc/") && lower.ends_with("/environ"))
}

/// One tokenized shell command segment extracted from a command line.
#[derive(Debug, Default)]
struct ShellSegment {
    tokens: Vec<String>,
    redirects: Vec<String>,
    subshells: Vec<String>,
}

/// Tokenize a shell command string into pipeline/list segments, capturing output redirections and
/// nested `$(...)` or `` `...` `` command substitutions.
fn tokenize_shell_segments(cmd: &str) -> Vec<ShellSegment> {
    let mut segments = Vec::new();
    let mut cur_seg = ShellSegment::default();
    let mut cur_tok = String::new();
    let mut in_token = false;
    let mut chars = cmd.chars().peekable();

    let flush_tok = |tok: &mut String, in_tok: &mut bool, seg: &mut ShellSegment| {
        if *in_tok {
            seg.tokens.push(std::mem::take(tok));
            *in_tok = false;
        }
    };
    let flush_seg = |tok: &mut String,
                     in_tok: &mut bool,
                     seg: &mut ShellSegment,
                     out: &mut Vec<ShellSegment>| {
        if *in_tok {
            seg.tokens.push(std::mem::take(tok));
            *in_tok = false;
        }
        if !seg.tokens.is_empty() || !seg.redirects.is_empty() || !seg.subshells.is_empty() {
            out.push(std::mem::take(seg));
        }
    };

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(next) = chars.next() {
                    if next != '\n' {
                        cur_tok.push(next);
                        in_token = true;
                    }
                }
            }
            '\'' => {
                in_token = true;
                for sc in chars.by_ref() {
                    if sc == '\'' {
                        break;
                    }
                    cur_tok.push(sc);
                }
            }
            '"' => {
                in_token = true;
                while let Some(dc) = chars.next() {
                    if dc == '"' {
                        break;
                    }
                    if dc == '\\' {
                        if let Some(esc) = chars.next() {
                            cur_tok.push(esc);
                        }
                        continue;
                    }
                    if dc == '$' && chars.peek() == Some(&'(') {
                        chars.next();
                        let sub = read_balanced_parens(&mut chars);
                        cur_seg.subshells.push(sub);
                        continue;
                    }
                    if dc == '`' {
                        let sub = read_backticks(&mut chars);
                        cur_seg.subshells.push(sub);
                        continue;
                    }
                    cur_tok.push(dc);
                }
            }
            '$' if chars.peek() == Some(&'(') => {
                chars.next();
                let sub = read_balanced_parens(&mut chars);
                cur_seg.subshells.push(sub);
                in_token = true;
            }
            '`' => {
                let sub = read_backticks(&mut chars);
                cur_seg.subshells.push(sub);
                in_token = true;
            }
            ';' | '\n' | '(' | ')' | '{' | '}' => {
                flush_seg(&mut cur_tok, &mut in_token, &mut cur_seg, &mut segments);
            }
            '#' if !in_token => {
                for hc in chars.by_ref() {
                    if hc == '\n' {
                        break;
                    }
                }
                flush_seg(&mut cur_tok, &mut in_token, &mut cur_seg, &mut segments);
            }
            '|' => {
                if chars.peek() == Some(&'|') {
                    chars.next();
                }
                flush_seg(&mut cur_tok, &mut in_token, &mut cur_seg, &mut segments);
            }
            '&' => {
                if chars.peek() == Some(&'>') {
                    chars.next();
                    if chars.peek() == Some(&'>') {
                        chars.next();
                    }
                    flush_tok(&mut cur_tok, &mut in_token, &mut cur_seg);
                    if let Some(target) = read_next_shell_word(&mut chars, &mut cur_seg.subshells) {
                        cur_seg.redirects.push(target);
                    }
                } else {
                    if chars.peek() == Some(&'&') {
                        chars.next();
                    }
                    flush_seg(&mut cur_tok, &mut in_token, &mut cur_seg, &mut segments);
                }
            }
            '>' => {
                // If cur_tok is purely a file descriptor like "1" or "2", drop it before the redirect.
                if in_token && (cur_tok == "1" || cur_tok == "2") {
                    cur_tok.clear();
                    in_token = false;
                } else {
                    flush_tok(&mut cur_tok, &mut in_token, &mut cur_seg);
                }
                if chars.peek() == Some(&'>') {
                    chars.next();
                }
                if let Some(target) = read_next_shell_word(&mut chars, &mut cur_seg.subshells) {
                    cur_seg.redirects.push(target);
                }
            }
            c if c.is_whitespace() => {
                flush_tok(&mut cur_tok, &mut in_token, &mut cur_seg);
            }
            other => {
                cur_tok.push(other);
                in_token = true;
            }
        }
    }
    flush_seg(&mut cur_tok, &mut in_token, &mut cur_seg, &mut segments);
    segments
}

fn read_balanced_parens(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut depth = 1usize;
    let mut out = String::new();
    for c in chars.by_ref() {
        if c == '(' {
            depth += 1;
        } else if c == ')' {
            depth -= 1;
            if depth == 0 {
                break;
            }
        }
        out.push(c);
    }
    out
}

fn read_backticks(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut out = String::new();
    while let Some(c) = chars.next() {
        if c == '`' {
            break;
        }
        if c == '\\' {
            if let Some(esc) = chars.next() {
                out.push(esc);
            }
            continue;
        }
        out.push(c);
    }
    out
}

fn read_next_shell_word(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    subshells: &mut Vec<String>,
) -> Option<String> {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() && c != '\n' {
            chars.next();
        } else {
            break;
        }
    }
    let mut word = String::new();
    let mut any = false;
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() || matches!(c, ';' | '|' | '&' | '(' | ')' | '>') {
            break;
        }
        let c = chars.next().unwrap_or_default();
        any = true;
        match c {
            '\\' => {
                if let Some(esc) = chars.next() {
                    word.push(esc);
                }
            }
            '\'' => {
                for sc in chars.by_ref() {
                    if sc == '\'' {
                        break;
                    }
                    word.push(sc);
                }
            }
            '"' => {
                while let Some(dc) = chars.next() {
                    if dc == '"' {
                        break;
                    }
                    if dc == '\\' {
                        if let Some(esc) = chars.next() {
                            word.push(esc);
                        }
                        continue;
                    }
                    if dc == '$' && chars.peek() == Some(&'(') {
                        chars.next();
                        subshells.push(read_balanced_parens(chars));
                        continue;
                    }
                    word.push(dc);
                }
            }
            other => word.push(other),
        }
    }
    if any {
        Some(word)
    } else {
        None
    }
}

/// Whether `tok` contains a shell variable expansion of a secret/token/key variable.
fn contains_secret_var_expansion(tok: &str) -> bool {
    let bytes = tok.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            let start = if i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                i + 2
            } else {
                i + 1
            };
            let mut end = start;
            while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            if end > start {
                let var = tok[start..end].to_ascii_uppercase();
                if var == "OWNER_RELEASE" || var.starts_with("UOR_POD_MAX_") {
                    return true;
                }
                if (var.contains("KEY")
                    || var.contains("TOKEN")
                    || var.contains("SECRET")
                    || var.contains("PASSWORD")
                    || var.contains("CREDENTIAL")
                    || var == "PAT"
                    || var.ends_with("_PAT"))
                    && !matches!(var.as_str(), "MONKEY" | "KEYBOARD")
                {
                    return true;
                }
            }
            i = end;
        } else {
            i += 1;
        }
    }
    false
}

/// Check a shell command against D56, forbidden git/process/credential rules, and subagent limits.
fn check_command(
    roots: &UorRoots,
    lab: &str,
    subagent: &SubagentContext,
    cwd: &Path,
    cmd: &str,
) -> Result<(), String> {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    if trimmed.contains("I release sheet") || trimmed.contains("OWNER_RELEASE") {
        return Err(
            "D56 / CHARTER §5.1: the guard never runs an owner release command for a lab; stage it for Casey"
                .into(),
        );
    }

    let segments = tokenize_shell_segments(trimmed);
    let mut effective_cwd = cwd.to_path_buf();
    for seg in &segments {
        for sub in &seg.subshells {
            check_command(roots, lab, subagent, &effective_cwd, sub)?;
        }
        for redir in &seg.redirects {
            let target = redir.trim_matches('"').trim_matches('\'');
            if !target.is_empty() && !target.starts_with("/dev/") && !target.starts_with('&') {
                check_write_path(roots, lab, subagent, &effective_cwd, target)?;
            }
        }
        if !seg.tokens.is_empty() {
            check_command_segment(roots, lab, subagent, &mut effective_cwd, &seg.tokens)?;
        }
    }
    Ok(())
}

/// Check a single tokenized command segment after unwrapping assignments and command wrappers.
fn check_command_segment(
    roots: &UorRoots,
    lab: &str,
    subagent: &SubagentContext,
    cwd: &mut PathBuf,
    tokens: &[String],
) -> Result<(), String> {
    // 1. Scan all tokens for universal flags (`--no-verify` / `--no-verif*`), secret expansions,
    //    and credential paths (both raw and resolved against `cwd`).
    for tok in tokens {
        if tok == "--no-verify" || tok.starts_with("--no-verif") {
            return Err("hook bypass (--no-verify) is forbidden".into());
        }
        if contains_secret_var_expansion(tok) {
            return Err(
                "CHARTER §3.13 / D56: expanding secret, token or owner-override environment variables is forbidden"
                    .into(),
            );
        }
        if is_credential_path(tok) {
            return Err(format!(
                "CHARTER §3.13: reading, writing or referencing credential path {tok} is forbidden"
            ));
        }
        if !tok.starts_with('-') && !tok.contains('=') {
            let resolved_tok = resolve_target_path(cwd, tok);
            if is_credential_path(&resolved_tok.to_string_lossy()) {
                return Err(format!(
                    "CHARTER §3.13: reading, writing or referencing credential path {tok} is forbidden"
                ));
            }
        }
    }

    // 2. Strip leading shell keywords, `KEY=VALUE` assignments, and command wrappers (`env`,
    //    `command`, `nohup`, `time`, `nice`, `sudo`, `timeout`, `stdbuf`, `xargs`, `exec`, `eval`).
    let mut idx = 0usize;
    while idx < tokens.len() {
        let tok = tokens[idx].as_str();
        if let Some((k, v)) = split_env_assignment(tok) {
            check_env_assignment(k, v)?;
            idx += 1;
            continue;
        }
        if matches!(
            tok,
            "do" | "then"
                | "else"
                | "elif"
                | "if"
                | "while"
                | "until"
                | "!"
                | "{"
                | "}"
                | "done"
                | "fi"
                | "esac"
                | "coproc"
        ) {
            idx += 1;
            continue;
        }
        if matches!(tok, "for" | "select" | "case") {
            // `for i in ...` or `case x in ...`: the body after `;` or `\n` is in the next segment.
            return Ok(());
        }
        let base = executable_basename(tok);
        match base {
            "eval" => {
                let rest = tokens[idx + 1..].join(" ");
                return check_command(roots, lab, subagent, cwd, &rest);
            }
            "exec" => {
                idx += 1;
                while idx < tokens.len() && tokens[idx].starts_with('-') {
                    let f = tokens[idx].as_str();
                    if f == "--" {
                        idx += 1;
                        break;
                    }
                    idx += 1;
                    if f == "-a" && idx < tokens.len() {
                        idx += 1;
                    }
                }
            }
            "env" => {
                idx += 1;
                while idx < tokens.len() {
                    let a = tokens[idx].as_str();
                    if a == "-u" || a == "--unset" {
                        let name = tokens.get(idx + 1).map(String::as_str).unwrap_or("");
                        if name == "UOR_LAB" {
                            return Err(
                                "D56 / CHARTER §5.1: a lab session may never unset UOR_LAB".into(),
                            );
                        }
                        idx += 2;
                    } else if let Some(rest) = a.strip_prefix("--unset=") {
                        if rest == "UOR_LAB" {
                            return Err(
                                "D56 / CHARTER §5.1: a lab session may never unset UOR_LAB".into(),
                            );
                        }
                        idx += 1;
                    } else if let Some(rest) = a.strip_prefix("-u") {
                        if rest == "UOR_LAB" {
                            return Err(
                                "D56 / CHARTER §5.1: a lab session may never unset UOR_LAB".into(),
                            );
                        }
                        idx += 1;
                    } else if a == "-i" || a == "--ignore-environment" || a == "-" {
                        return Err(
                            "D56 / CHARTER §5.1: clearing the environment (-i) in a lab session is forbidden"
                                .into(),
                        );
                    } else if a == "-C" || a == "--chdir" {
                        if let Some(dir) = tokens.get(idx + 1) {
                            *cwd = resolve_target_path(cwd, dir);
                        }
                        idx += 2;
                    } else if let Some(dir) =
                        a.strip_prefix("--chdir=").or_else(|| a.strip_prefix("-C"))
                    {
                        if !dir.is_empty() {
                            *cwd = resolve_target_path(cwd, dir);
                        }
                        idx += 1;
                    } else if a == "-S" || a == "--split-string" {
                        if let Some(script) = tokens.get(idx + 1) {
                            check_command(roots, lab, subagent, cwd, script)?;
                        }
                        idx += 2;
                    } else if let Some(script) = a
                        .strip_prefix("--split-string=")
                        .or_else(|| a.strip_prefix("-S"))
                    {
                        if !script.is_empty() {
                            check_command(roots, lab, subagent, cwd, script)?;
                        }
                        idx += 1;
                    } else if a == "-0" || a == "--null" || a == "-v" || a == "--debug" {
                        idx += 1;
                    } else if let Some((k, v)) = split_env_assignment(a) {
                        check_env_assignment(k, v)?;
                        idx += 1;
                    } else {
                        break;
                    }
                }
                if idx >= tokens.len() {
                    return Err(
                        "CHARTER §3.13: dumping environment variables via bare env is forbidden"
                            .into(),
                    );
                }
            }
            "command" | "builtin" => {
                idx += 1;
                while idx < tokens.len() && tokens[idx].starts_with('-') {
                    let f = tokens[idx].as_str();
                    if f == "-v" || f == "-V" {
                        // `command -v foo` only prints the path of `foo` and does not execute it.
                        return Ok(());
                    }
                    idx += 1;
                }
            }
            "nohup" | "time" => {
                idx += 1;
                while idx < tokens.len() && tokens[idx].starts_with('-') {
                    idx += 1;
                }
            }
            "nice" => {
                idx += 1;
                while idx < tokens.len() && tokens[idx].starts_with('-') {
                    let f = tokens[idx].as_str();
                    idx += 1;
                    if (f == "-n" || f == "--adjustment") && idx < tokens.len() {
                        idx += 1;
                    }
                }
            }
            "timeout" => {
                idx += 1;
                while idx < tokens.len() && tokens[idx].starts_with('-') {
                    let f = tokens[idx].as_str();
                    idx += 1;
                    if matches!(f, "-k" | "-s" | "--kill-after" | "--signal") && idx < tokens.len()
                    {
                        idx += 1;
                    }
                }
                if idx < tokens.len() {
                    idx += 1; // duration argument
                }
            }
            "stdbuf" => {
                idx += 1;
                while idx < tokens.len() && tokens[idx].starts_with('-') {
                    let f = tokens[idx].as_str();
                    idx += 1;
                    if matches!(f, "-i" | "-o" | "-e") && idx < tokens.len() {
                        idx += 1;
                    }
                }
            }
            "sudo" => {
                idx += 1;
                while idx < tokens.len() && tokens[idx].starts_with('-') {
                    let f = tokens[idx].as_str();
                    idx += 1;
                    if matches!(
                        f,
                        "-u" | "-g" | "-C" | "-D" | "-h" | "-p" | "-r" | "-t" | "-U"
                    ) && idx < tokens.len()
                    {
                        idx += 1;
                    }
                }
            }
            "xargs" => {
                idx += 1;
                while idx < tokens.len() && tokens[idx].starts_with('-') {
                    let f = tokens[idx].as_str();
                    idx += 1;
                    if matches!(
                        f,
                        "-I" | "-J"
                            | "-L"
                            | "-n"
                            | "-P"
                            | "-s"
                            | "-d"
                            | "-E"
                            | "-a"
                            | "--max-args"
                            | "--max-procs"
                            | "--max-chars"
                            | "--max-lines"
                            | "--replace"
                            | "--eof"
                            | "--arg-file"
                            | "--delimiter"
                            | "--process-slot-var"
                    ) && idx < tokens.len()
                    {
                        idx += 1;
                    }
                }
            }
            _ => break,
        }
    }

    if idx >= tokens.len() {
        return Ok(());
    }

    let prog = executable_basename(&tokens[idx]);
    let args = &tokens[idx + 1..];

    match prog {
        "cd" => {
            if let Some(dir) = args.iter().find(|a| !a.starts_with('-')) {
                let next_cwd = resolve_target_path(cwd, dir);
                if is_credential_path(&next_cwd.to_string_lossy()) {
                    return Err(format!(
                        "CHARTER §3.13: entering credential directory {dir} is forbidden"
                    ));
                }
                *cwd = next_cwd;
            }
        }
        "sh" | "bash" | "zsh" | "dash" | "ksh" => {
            for (i, a) in args.iter().enumerate() {
                let is_c = a == "-c"
                    || (a.starts_with('-')
                        && !a.starts_with("--")
                        && a[1..].chars().all(|ch| ch.is_ascii_alphabetic())
                        && a[1..].contains('c'));
                if is_c {
                    if let Some(script) = args.get(i + 1) {
                        check_command(roots, lab, subagent, cwd, script)?;
                    }
                    break;
                }
            }
        }
        "unset" => {
            if args.iter().any(|a| a == "UOR_LAB") {
                return Err(
                    "D56 / CHARTER §5.1: a lab session may never unset or clear UOR_LAB".into(),
                );
            }
        }
        "pkill" | "killall" => {
            return Err("pkill and killall are forbidden; stop only your own started pids".into());
        }
        "ps" => {
            for a in args {
                if a == "eww"
                    || a == "auxe"
                    || a == "auxwwwe"
                    || a == "-e"
                    || (!a.starts_with('-')
                        && a.contains('e')
                        && (a.contains('w')
                            || a.contains('a')
                            || a.contains('u')
                            || a.contains('x')))
                {
                    return Err(
                        "CHARTER §3.13: dumping process environment variables via ps eww is forbidden"
                            .into(),
                    );
                }
            }
        }
        "printenv" => {
            if args.is_empty() {
                return Err(
                    "CHARTER §3.13: dumping all environment variables via printenv is forbidden"
                        .into(),
                );
            }
            for a in args {
                if !a.starts_with('-')
                    && !matches!(
                        a.as_str(),
                        "UOR_LAB"
                            | "PATH"
                            | "HOME"
                            | "PWD"
                            | "USER"
                            | "SHELL"
                            | "TERM"
                            | "CARGO_TARGET_DIR"
                            | "RUST_BACKTRACE"
                            | "RUST_LOG"
                    )
                {
                    return Err(format!(
                        "CHARTER §3.13: reading environment variable {a} via printenv is forbidden"
                    ));
                }
            }
        }
        "export" | "declare" | "typeset" => {
            let has_assignment = args.iter().any(|a| a.contains('='));
            if args.is_empty()
                || (!has_assignment
                    && args
                        .iter()
                        .any(|a| matches!(a.as_str(), "-p" | "-x" | "-xp" | "-px" | "--")))
            {
                return Err(
                    "CHARTER §3.13: dumping environment variables via export/declare is forbidden"
                        .into(),
                );
            }
            for a in args {
                if let Some((k, v)) = split_env_assignment(a) {
                    check_env_assignment(k, v)?;
                }
            }
        }
        "set" => {
            if args.is_empty() {
                return Err(
                    "CHARTER §3.13: dumping shell variables via bare set is forbidden".into(),
                );
            }
        }
        "git" => check_git_command(roots, lab, subagent, cwd, args)?,
        "gh" => check_gh_command(subagent, args)?,
        "uor-harness" | "uor-r4-harness" => check_uor_harness_command(subagent, args)?,
        "cargo" => {
            if args.iter().any(|a| a == "run" || a == "r") {
                if let Some(dd) = args.iter().position(|a| a == "--") {
                    check_uor_harness_command(subagent, &args[dd + 1..])?;
                }
            }
        }
        "uor-pod" => check_uor_pod_command(args)?,
        _ => {}
    }

    check_shell_file_mods(roots, lab, subagent, cwd, prog, args)?;
    Ok(())
}

fn executable_basename(raw: &str) -> &str {
    raw.rsplit('/').next().unwrap_or(raw)
}

fn split_env_assignment(tok: &str) -> Option<(&str, &str)> {
    let eq = tok.find('=')?;
    if eq == 0 {
        return None;
    }
    let key = &tok[..eq];
    if !key
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return None;
    }
    Some((key, &tok[eq + 1..]))
}

fn check_env_assignment(key: &str, val: &str) -> Result<(), String> {
    let trimmed = val.trim().trim_matches('"').trim_matches('\'');
    if key == "UOR_LAB" && trimmed.is_empty() {
        return Err(
            "D56 / CHARTER §5.1: a lab session may never clear UOR_LAB to run an owner command"
                .into(),
        );
    }
    if key == "OWNER_RELEASE" {
        return Err(
            "D56 / CHARTER §5.1: the guard never runs an owner release command for a lab".into(),
        );
    }
    if key.starts_with("UOR_POD_MAX_") {
        return Err(
            "CHARTER §5.2 / D56: labs cannot override GPU pod or spend caps; stage the command for Casey"
                .into(),
        );
    }
    let key_upper = key.to_ascii_uppercase();
    if matches!(
        key_upper.as_str(),
        "GIT_CONFIG_GLOBAL" | "GIT_CONFIG_SYSTEM" | "GIT_CONFIG_NOSYSTEM"
    ) {
        return Err("hook bypass (GIT_CONFIG_GLOBAL/SYSTEM override) is forbidden".into());
    }
    if key_upper == "GIT_CONFIG_PARAMETERS" || key_upper.starts_with("GIT_CONFIG_") {
        let lower_val = trimmed.to_ascii_lowercase();
        if is_forbidden_git_config_kv(&lower_val) {
            return Err("hook bypass (git config override via environment) is forbidden".into());
        }
    }
    Ok(())
}

fn is_forbidden_git_config_kv(lower_kv: &str) -> bool {
    let key_part = lower_kv
        .split('=')
        .next()
        .unwrap_or(lower_kv)
        .trim()
        .trim_matches('\'')
        .trim_matches('"');
    lower_kv.contains("core.hookspath")
        || lower_kv.contains("include.path")
        || lower_kv.contains("includeif.")
        || key_part == "push.default"
        || (key_part.starts_with("remote.") && key_part.ends_with(".push"))
        || (lower_kv.contains("remote.") && lower_kv.contains(".push"))
}

/// Inspect a `git` invocation after unwrapping all global `git` flags (`-C`, `-c`, `--git-dir`,
/// `--work-tree`, `--config-env`, etc.).
fn check_git_command(
    roots: &UorRoots,
    lab: &str,
    subagent: &SubagentContext,
    cwd: &Path,
    args: &[String],
) -> Result<(), String> {
    let mut i = 0usize;
    let mut effective_cwd = cwd.to_path_buf();
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--" {
            i += 1;
            break;
        }
        if a == "-C" {
            if let Some(dir) = args.get(i + 1) {
                effective_cwd = resolve_target_path(&effective_cwd, dir);
            }
            i += 2;
            continue;
        }
        if let Some(dir) = a.strip_prefix("-C") {
            if !dir.is_empty() {
                effective_cwd = resolve_target_path(&effective_cwd, dir);
                i += 1;
                continue;
            }
        }
        if a == "-c" || a == "--config-env" {
            let kv = args.get(i + 1).map(String::as_str).unwrap_or("");
            if is_forbidden_git_config_kv(&kv.to_ascii_lowercase()) {
                return Err("hook/push config bypass (git -c ...) is forbidden".into());
            }
            i += 2;
            continue;
        }
        if let Some(kv) = a
            .strip_prefix("-c")
            .or_else(|| a.strip_prefix("--config-env="))
        {
            if is_forbidden_git_config_kv(&kv.to_ascii_lowercase()) {
                return Err("hook/push config bypass (git -c ...) is forbidden".into());
            }
            i += 1;
            continue;
        }
        if matches!(
            a,
            "--git-dir" | "--work-tree" | "--namespace" | "--super-prefix" | "--exec-path"
        ) {
            i += 2;
            continue;
        }
        if a.starts_with("--git-dir=")
            || a.starts_with("--work-tree=")
            || a.starts_with("--namespace=")
            || a.starts_with("--super-prefix=")
            || a.starts_with("--exec-path=")
            || matches!(
                a,
                "--bare"
                    | "--no-pager"
                    | "-p"
                    | "--paginate"
                    | "--no-replace-objects"
                    | "--literal-pathspecs"
                    | "--glob-pathspecs"
                    | "--noglob-pathspecs"
                    | "--icase-pathspecs"
                    | "--no-optional-locks"
            )
        {
            i += 1;
            continue;
        }
        if a.starts_with('-') {
            i += 1;
            continue;
        }
        break;
    }

    let Some(subcmd) = args.get(i).map(String::as_str) else {
        return Ok(());
    };
    let sub_args = &args[i + 1..];

    match subcmd {
        "stash" => {
            return Err(
                "git stash is forbidden in the five-lab loop; commit WIP in your worktree".into(),
            );
        }
        "config" => {
            let mut readonly = false;
            let mut pos = Vec::new();
            let mut j = 0usize;
            while j < sub_args.len() {
                let a = sub_args[j].as_str();
                if a == "--" {
                    pos.extend(sub_args[j + 1..].iter().map(String::as_str));
                    break;
                }
                if matches!(
                    a,
                    "--get" | "--get-all" | "--get-regexp" | "--get-urlmatch" | "--list" | "-l"
                ) {
                    readonly = true;
                    j += 1;
                    continue;
                }
                if matches!(
                    a,
                    "-f" | "--file" | "--blob" | "--type" | "-t" | "--default"
                ) {
                    j += 2;
                    continue;
                }
                if a.starts_with('-') {
                    j += 1;
                    continue;
                }
                pos.push(a);
                j += 1;
            }
            if !readonly {
                if let Some(&first) = pos.first() {
                    if matches!(first, "list" | "get") {
                        readonly = true;
                    }
                }
            }
            if !readonly {
                for p in &pos {
                    let lower = p.to_ascii_lowercase();
                    if is_forbidden_git_config_kv(&lower) {
                        return Err(format!(
                            "git config write to {p} is forbidden (hook/include/push override)"
                        ));
                    }
                }
            }
        }
        "commit" => {
            let mut j = 0usize;
            while j < sub_args.len() {
                let a = sub_args[j].as_str();
                if a == "--" {
                    break;
                }
                if a == "--no-verify" || a.starts_with("--no-verif") {
                    return Err("hook bypass (--no-verify / git commit -n) is forbidden".into());
                }
                if a.starts_with('-') && !a.starts_with("--") {
                    let cluster = &a[1..];
                    for (ci, ch) in cluster.char_indices() {
                        if ch == 'n' {
                            return Err(
                                "hook bypass (--no-verify / git commit -n) is forbidden".into()
                            );
                        }
                        // `-m`, `-F`, `-c`, `-C`, `-u`, `--author`, etc. consume the rest of the
                        // cluster (or the next token if at the end of the cluster) as their value.
                        if matches!(ch, 'm' | 'F' | 'c' | 'C' | 'u' | 't') {
                            if ci + ch.len_utf8() == cluster.len() {
                                j += 1;
                            }
                            break;
                        }
                    }
                } else if matches!(
                    a,
                    "--message"
                        | "--file"
                        | "--reuse-message"
                        | "--reedit-message"
                        | "--fixup"
                        | "--squash"
                        | "--author"
                        | "--date"
                        | "--cleanup"
                        | "--template"
                ) {
                    j += 1;
                }
                j += 1;
            }
        }
        "push" => {
            if matches!(subagent, SubagentContext::Subagent(_)) {
                return Err(
                    "subagent stop rule: subagents may never claim, open, review or merge a PR, push, or post as the lab"
                        .into(),
                );
            }
            let mut positionals = Vec::new();
            let mut j = 0usize;
            let mut delete_flag = false;
            while j < sub_args.len() {
                let a = sub_args[j].as_str();
                if a == "--" {
                    positionals.extend(sub_args[j + 1..].iter().map(String::as_str));
                    break;
                }
                if a == "--force-with-lease" || a.starts_with("--force-with-lease=") {
                    j += 1;
                    continue;
                }
                if a == "--force" || a == "--force-if-includes" {
                    return Err("git push --force is forbidden".into());
                }
                if a == "--mirror" || a == "--all" || a == "--branches" {
                    return Err(format!(
                        "git push {a} is forbidden (would push to or overwrite main)"
                    ));
                }
                if a == "--no-verify" || a.starts_with("--no-verif") {
                    return Err("hook bypass (git push --no-verify) is forbidden".into());
                }
                if a == "--delete" || a == "-d" {
                    delete_flag = true;
                    j += 1;
                    continue;
                }
                if matches!(
                    a,
                    "--repo" | "--receive-pack" | "--exec" | "-o" | "--push-option"
                ) {
                    j += 2;
                    continue;
                }
                if a.starts_with('-') && !a.starts_with("--") {
                    if a[1..].contains('f') {
                        return Err("git push --force (-f) is forbidden".into());
                    }
                    if a[1..].contains('d') {
                        delete_flag = true;
                    }
                    j += 1;
                    continue;
                }
                if a.starts_with("--") {
                    j += 1;
                    continue;
                }
                positionals.push(a);
                j += 1;
            }
            for (pi, pos) in positionals.iter().copied().enumerate() {
                if pos.starts_with('+') {
                    return Err("git push +refspec (force push) is forbidden".into());
                }
                // Positional 0 is the remote name unless it contains `:` (a refspec).
                if pi >= 1 || pos.contains(':') {
                    let dst = pos
                        .rsplit(':')
                        .next()
                        .unwrap_or(pos)
                        .trim_start_matches('+');
                    if dst == "main"
                        || dst == "refs/heads/main"
                        || dst == "heads/main"
                        || dst.ends_with("/main")
                        || (delete_flag && dst == "main")
                    {
                        return Err("direct push to main is forbidden; deliver through a PR".into());
                    }
                }
            }
        }
        "rm" | "mv" | "restore" => {
            for a in sub_args {
                if a == "--" || a.starts_with('-') {
                    continue;
                }
                check_write_path(roots, lab, subagent, &effective_cwd, a)?;
            }
        }
        "checkout" => {
            if let Some(dd) = sub_args.iter().position(|a| a == "--") {
                for a in &sub_args[dd + 1..] {
                    check_write_path(roots, lab, subagent, &effective_cwd, a)?;
                }
            } else {
                let mut pos = Vec::new();
                let mut j = 0usize;
                while j < sub_args.len() {
                    let a = sub_args[j].as_str();
                    if matches!(
                        a,
                        "-b" | "-B" | "--orphan" | "--conflict" | "--pathspec-from-file"
                    ) {
                        j += 2;
                        continue;
                    }
                    if a.starts_with('-') {
                        j += 1;
                        continue;
                    }
                    pos.push(a);
                    j += 1;
                }
                // `git checkout <tree-ish> <path...>` or `git checkout <existing-file>`
                for (pi, p) in pos.iter().copied().enumerate() {
                    let resolved = resolve_target_path(&effective_cwd, p);
                    if pi >= 1 || resolved.is_file() {
                        check_write_path(roots, lab, subagent, &effective_cwd, p)?;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Inspect a `gh` CLI invocation for token dumps, unguarded merges, and subagent restrictions.
fn check_gh_command(subagent: &SubagentContext, args: &[String]) -> Result<(), String> {
    let mut pos = Vec::new();
    let mut i = 0usize;
    while i < args.len() {
        let a = args[i].as_str();
        if matches!(a, "-R" | "--repo" | "--hostname") {
            i += 2;
            continue;
        }
        if a.starts_with("--repo=") || a.starts_with("--hostname=") {
            i += 1;
            continue;
        }
        pos.push(a);
        i += 1;
    }
    let s1 = pos.first().copied().unwrap_or("");
    let s2 = pos.get(1).copied().unwrap_or("");

    if s1 == "auth" && (s2 == "token" || pos.iter().any(|a| *a == "--show-token" || *a == "-t")) {
        return Err("CHARTER §3.13: reading or printing credentials or tokens is forbidden".into());
    }
    if s1 == "api"
        && pos.iter().any(|a| {
            let lower = a.to_ascii_lowercase();
            lower.contains("/merge")
                || lower.ends_with("merge")
                || lower.contains("mergepullrequest")
                || lower.contains("enqueuepullrequest")
                || lower.contains("enablepullrequestautomerge")
        })
    {
        return Err(
            "merging via gh api is forbidden; use gh pr merge --match-head-commit <full-sha>"
                .into(),
        );
    }
    if s1 == "pr" && s2 == "merge" {
        if matches!(subagent, SubagentContext::Subagent(_)) {
            return Err(
                "subagent stop rule: subagents may never claim, open, review or merge a PR, push, or post as the lab"
                    .into(),
            );
        }
        if pos.iter().any(|a| *a == "--admin" || *a == "--auto") {
            return Err("gh pr merge --admin / --auto is forbidden".into());
        }
        let mut matched_sha = None;
        for (idx, a) in pos.iter().copied().enumerate() {
            if a == "--match-head-commit" {
                matched_sha = pos.get(idx + 1).copied();
            } else if let Some(rest) = a.strip_prefix("--match-head-commit=") {
                matched_sha = Some(rest);
            }
        }
        let valid_sha =
            matched_sha.is_some_and(|s| s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit()));
        if !valid_sha {
            return Err("gh pr merge requires --match-head-commit <full-40-char-sha>".into());
        }
    }
    if matches!(subagent, SubagentContext::Subagent(_))
        && ((s1 == "pr"
            && matches!(
                s2,
                "create" | "merge" | "review" | "comment" | "close" | "edit" | "ready"
            ))
            || (s1 == "issue" && matches!(s2, "create" | "comment" | "close" | "edit" | "reopen")))
    {
        return Err(
            "subagent stop rule: subagents may never claim, open, review or merge a PR, push, or post as the lab"
                .into(),
        );
    }
    Ok(())
}

/// Inspect a `uor-harness` CLI invocation for owner-only commands and subagent restrictions.
fn check_uor_harness_command(subagent: &SubagentContext, args: &[String]) -> Result<(), String> {
    let mut i = 0usize;
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--" {
            i += 1;
            break;
        }
        if a == "--repo" {
            i += 2;
            continue;
        }
        if a.starts_with("--repo=")
            || matches!(
                a,
                "--json" | "-v" | "--verbose" | "-q" | "--quiet" | "--offline" | "--locked"
            )
        {
            i += 1;
            continue;
        }
        if a.starts_with('-') {
            i += 1;
            continue;
        }
        break;
    }
    let rem = if i <= args.len() { &args[i..] } else { &[] };
    let s1 = rem.first().map(String::as_str).unwrap_or("");
    let s2 = rem.get(1).map(String::as_str).unwrap_or("");

    if s1 == "release"
        || (s1 == "team"
            && s2 == "release"
            && rem
                .iter()
                .any(|a| a == "--owner" || a.starts_with("--owner=")))
    {
        return Err(
            "D56 / CHARTER §5.1: the guard never runs an owner release command for a lab; stage it for Casey"
                .into(),
        );
    }
    if s1 == "git" && s2 == "install-hooks" {
        return Err(
            "owner decision (#2028): uor-harness git install-hooks may be run only by the owner from Terminal"
                .into(),
        );
    }
    if s1 == "loop"
        && s2 == "install"
        && rem
            .iter()
            .any(|a| a == "--apply" || a.starts_with("--apply="))
    {
        return Err(
            "owner decision (#2028): uor-harness loop install --apply may be run only by the owner from Terminal"
                .into(),
        );
    }
    if matches!(subagent, SubagentContext::Subagent(_))
        && ((s1 == "loop" && matches!(s2, "claim" | "review" | "close" | "queue"))
            || (s1 == "team" && s2 == "claim"))
    {
        return Err(
            "subagent stop rule: subagents may never claim, open, review or merge a PR, push, or post as the lab"
                .into(),
        );
    }
    Ok(())
}

/// Inspect a `uor-pod` CLI invocation for forbidden cap overrides (D56 / CHARTER §5.2).
fn check_uor_pod_command(args: &[String]) -> Result<(), String> {
    for a in args {
        if a == "--max-pods"
            || a.starts_with("--max-pods=")
            || a == "--max-hourly-usd"
            || a.starts_with("--max-hourly-usd=")
            || a == "--override-cap"
            || a.starts_with("--override-cap=")
        {
            return Err(
                "CHARTER §5.2 / D56: labs cannot override GPU pod or spend caps; stage the command for Casey"
                    .into(),
            );
        }
    }
    Ok(())
}

/// Check file-modifying shell commands (`tee`, `rm`, `cp`, `mv`, `sed -i`, `dd of=`, etc.) against
/// `check_write_path`.
fn check_shell_file_mods(
    roots: &UorRoots,
    lab: &str,
    subagent: &SubagentContext,
    cwd: &Path,
    prog: &str,
    args: &[String],
) -> Result<(), String> {
    match prog {
        "tee" => {
            for a in args {
                if !a.starts_with('-') && !a.starts_with("/dev/") {
                    check_write_path(roots, lab, subagent, cwd, a)?;
                }
            }
        }
        "rm" | "unlink" | "rmdir" | "touch" | "mkdir" => {
            for a in args {
                if a == "--" || a.starts_with('-') {
                    continue;
                }
                check_write_path(roots, lab, subagent, cwd, a)?;
            }
        }
        "truncate" => {
            let mut i = 0usize;
            while i < args.len() {
                let a = args[i].as_str();
                if matches!(a, "-s" | "--size" | "-r" | "--reference") {
                    i += 2;
                    continue;
                }
                if !a.starts_with('-') {
                    check_write_path(roots, lab, subagent, cwd, a)?;
                }
                i += 1;
            }
        }
        "chmod" | "chown" => {
            let mut pos = Vec::new();
            let mut i = 0usize;
            while i < args.len() {
                let a = args[i].as_str();
                if a == "--" {
                    pos.extend(args[i + 1..].iter().map(String::as_str));
                    break;
                }
                if a.starts_with('-') {
                    i += 1;
                    continue;
                }
                pos.push(a);
                i += 1;
            }
            // First positional is mode/owner; remaining positionals are target paths.
            for p in pos.into_iter().skip(1) {
                check_write_path(roots, lab, subagent, cwd, p)?;
            }
        }
        "cp" | "mv" | "install" | "ln" => {
            let mut pos = Vec::new();
            let mut target_dir = None;
            let mut i = 0usize;
            while i < args.len() {
                let a = args[i].as_str();
                if a == "--" {
                    pos.extend(args[i + 1..].iter().map(String::as_str));
                    break;
                }
                if a == "-t" || a == "--target-directory" {
                    target_dir = args.get(i + 1).map(String::as_str);
                    i += 2;
                    continue;
                }
                if let Some(td) = a.strip_prefix("--target-directory=") {
                    target_dir = Some(td);
                    i += 1;
                    continue;
                }
                if matches!(a, "-S" | "--suffix" | "-m" | "--mode" | "-o" | "-g") {
                    i += 2;
                    continue;
                }
                if a.starts_with('-') {
                    i += 1;
                    continue;
                }
                pos.push(a);
                i += 1;
            }
            if let Some(td) = target_dir {
                check_write_path(roots, lab, subagent, cwd, td)?;
                if prog == "mv" {
                    for src in &pos {
                        check_write_path(roots, lab, subagent, cwd, src)?;
                    }
                }
            } else if pos.len() >= 2 {
                if prog == "mv" {
                    for p in &pos {
                        check_write_path(roots, lab, subagent, cwd, p)?;
                    }
                } else if let Some(dst) = pos.last() {
                    check_write_path(roots, lab, subagent, cwd, dst)?;
                }
            }
        }
        "sed" | "perl" | "ruby" => {
            let mut in_place = false;
            let mut has_explicit_expr = false;
            let mut pos = Vec::new();
            let mut i = 0usize;
            while i < args.len() {
                let a = args[i].as_str();
                if a == "--" {
                    pos.extend(args[i + 1..].iter().map(String::as_str));
                    break;
                }
                if a == "-i"
                    || a.starts_with("-i")
                    || a == "--in-place"
                    || a.starts_with("--in-place=")
                {
                    in_place = true;
                    // On macOS, `sed -i '' 's/a/b/' file` passes an empty backup suffix as the
                    // next token.
                    if a == "-i" && args.get(i + 1).is_some_and(|n| n.is_empty()) {
                        i += 2;
                        continue;
                    }
                    i += 1;
                    continue;
                }
                if matches!(a, "-e" | "--expression" | "-f" | "--file") {
                    has_explicit_expr = true;
                    i += 2;
                    continue;
                }
                if a.starts_with("-e") || a.starts_with("-f") {
                    has_explicit_expr = true;
                    i += 1;
                    continue;
                }
                if a.starts_with('-') {
                    i += 1;
                    continue;
                }
                pos.push(a);
                i += 1;
            }
            if in_place {
                let skip = if has_explicit_expr { 0 } else { 1 };
                for p in pos.into_iter().skip(skip) {
                    check_write_path(roots, lab, subagent, cwd, p)?;
                }
            }
        }
        "dd" => {
            for a in args {
                if let Some(of_path) = a.strip_prefix("of=") {
                    if !of_path.starts_with("/dev/") {
                        check_write_path(roots, lab, subagent, cwd, of_path)?;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Normalize a path lexically (resolving `.` and `..` without requiring the target file to exist).
fn normalize_lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Resolve `raw_path` against `cwd`, expanding symlinks on each existing prefix component before
/// applying `..` so a symlink inside `local/<lab>/` pointing to `../../docs` followed by
/// `../CHARTER.md` resolves to `<repo>/CHARTER.md` and is refused.
fn resolve_target_path(cwd: &Path, raw_path: &str) -> PathBuf {
    let base_cwd = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
    let expanded = if let Some(rest) = raw_path.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
            home.join(rest)
        } else {
            PathBuf::from(raw_path)
        }
    } else if raw_path == "~" {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(raw_path))
    } else if Path::new(raw_path).is_absolute() {
        PathBuf::from(raw_path)
    } else {
        base_cwd.join(raw_path)
    };
    resolve_components_with_symlinks(&expanded, 0)
}

fn resolve_components_with_symlinks(path: &Path, depth: u32) -> PathBuf {
    if depth > 16 {
        return normalize_lexical(path);
    }
    let mut acc = PathBuf::new();
    for comp in path.components() {
        match comp {
            std::path::Component::Prefix(p) => acc.push(p.as_os_str()),
            std::path::Component::RootDir => acc.push(comp.as_os_str()),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if let Ok(canon) = acc.canonicalize() {
                    acc = canon;
                }
                acc.pop();
                if let Ok(canon) = acc.canonicalize() {
                    acc = canon;
                }
            }
            std::path::Component::Normal(name) => {
                acc.push(name);
                if let Ok(canon) = acc.canonicalize() {
                    acc = canon;
                } else if let Ok(target) = fs::read_link(&acc) {
                    acc.pop();
                    let next = if target.is_absolute() {
                        target
                    } else {
                        acc.join(target)
                    };
                    acc = resolve_components_with_symlinks(&next, depth + 1);
                }
            }
        }
    }
    acc
}

/// Load the owner-only path globs from `docs/harness/owner-paths.json`.
fn load_owner_globs(root: &Path) -> Result<Vec<String>, String> {
    let path = root.join("docs/harness/owner-paths.json");
    let text =
        fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let val: Value =
        serde_json::from_str(&text).map_err(|e| format!("malformed {}: {e}", path.display()))?;
    let arr = val
        .get("paths")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{} has no paths array", path.display()))?;
    Ok(arr
        .iter()
        .filter_map(|p| p.get("path").and_then(Value::as_str).map(String::from))
        .collect())
}

/// Find the active ORDER (`(order_rel_path, topic, order_text)`) in `worktree_root` for `lab`.
///
/// Resolves only from:
/// 1. `local/<lab>/guard/active-order` if present;
/// 2. An ORDER added or modified on the worktree's branch relative to `origin/main` (when
///    `origin/main` exists in git), so closed historical task folders on `origin/main` (such as
///    `claude-20`) are never mistaken for an active ORDER;
/// 3. In standalone test fixtures without `origin/main`, an open `docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md`
///    that has no closed `REPORT.md`.
fn find_active_order(roots: &UorRoots, lab: &str) -> Option<(String, String, String)> {
    // 1. Explicit marker file `local/<lab>/guard/active-order` if present.
    let marker = guard_state_dir(roots, lab).join("active-order");
    if let Ok(rel) = fs::read_to_string(&marker) {
        let rel = rel.trim().replace('\\', "/");
        if !rel.is_empty() {
            let full = roots.worktree_root.join(&rel);
            if let Ok(text) = fs::read_to_string(&full) {
                let topic = full
                    .parent()
                    .and_then(Path::file_name)
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                return Some((rel, topic, text));
            }
        }
    }

    let prefix = format!("{lab}-");

    // 2. If `origin/main` exists in this git repository, only ORDER files added/modified since
    //    `origin/main` (or untracked/modified in the working tree) can be active.
    let has_origin_main = Command::new("git")
        .arg("-C")
        .arg(&roots.worktree_root)
        .args(["rev-parse", "--verify", "-q", "origin/main"])
        .output()
        .ok()
        .is_some_and(|o| o.status.success());

    if has_origin_main {
        let mut candidate_rels = HashSet::new();
        if let Ok(diff_out) = Command::new("git")
            .arg("-C")
            .arg(&roots.worktree_root)
            .args([
                "diff",
                "--name-only",
                "origin/main...HEAD",
                "--",
                "docs/loop/tasks/",
            ])
            .output()
        {
            if diff_out.status.success() {
                for line in String::from_utf8_lossy(&diff_out.stdout).lines() {
                    let p = line.trim();
                    if p.ends_with("/ORDER.md") {
                        candidate_rels.insert(p.to_string());
                    }
                }
            }
        }
        if let Ok(st_out) = Command::new("git")
            .arg("-C")
            .arg(&roots.worktree_root)
            .args(["status", "--porcelain", "--", "docs/loop/tasks/"])
            .output()
        {
            if st_out.status.success() {
                for line in String::from_utf8_lossy(&st_out.stdout).lines() {
                    if line.len() > 3 {
                        let p = line[3..].trim();
                        if p.ends_with("/ORDER.md") {
                            candidate_rels.insert(p.to_string());
                        } else if p.ends_with('/') {
                            // Untracked directory under `docs/loop/tasks/...`: walk for ORDER.md.
                            let dir = roots.worktree_root.join(p);
                            collect_untracked_orders(
                                &roots.worktree_root,
                                &dir,
                                &mut candidate_rels,
                            );
                        }
                    }
                }
            }
        }

        let mut best: Option<(bool, u64, String, String, String)> = None;
        for rel in candidate_rels {
            let full = roots.worktree_root.join(&rel);
            let Ok(text) = fs::read_to_string(&full) else {
                continue;
            };
            let Some(parent) = full.parent() else {
                continue;
            };
            let is_open = !parent.join("REPORT.md").exists();
            let Some(topic) = parent
                .file_name()
                .and_then(|n| n.to_str())
                .map(String::from)
            else {
                continue;
            };
            let Some(num_str) = topic.strip_prefix(&prefix) else {
                continue;
            };
            let Ok(num) = num_str.parse::<u64>() else {
                continue;
            };
            if best
                .as_ref()
                .is_none_or(|(b_open, b_num, _, _, _)| (is_open, num) >= (*b_open, *b_num))
            {
                best = Some((is_open, num, rel, topic, text));
            }
        }
        return best.map(|(_, _, rel, topic, text)| (rel, topic, text));
    }

    // 3. Standalone test fixture without `origin/main`: scan `docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md`
    //    and skip closed folders that already have `REPORT.md`.
    let tasks_dir = roots.worktree_root.join("docs/loop/tasks");
    let entries = fs::read_dir(&tasks_dir).ok()?;
    let mut best: Option<(u64, String, String, String)> = None;
    for folder_entry in entries.flatten() {
        if !folder_entry.path().is_dir() {
            continue;
        }
        let folder_name = folder_entry.file_name().to_string_lossy().to_string();
        let Ok(sub_entries) = fs::read_dir(folder_entry.path()) else {
            continue;
        };
        for sub in sub_entries.flatten() {
            let topic = sub.file_name().to_string_lossy().to_string();
            let Some(num_str) = topic.strip_prefix(&prefix) else {
                continue;
            };
            let Ok(num) = num_str.parse::<u64>() else {
                continue;
            };
            if sub.path().join("REPORT.md").exists() {
                continue;
            }
            let order_file = sub.path().join("ORDER.md");
            let Ok(text) = fs::read_to_string(&order_file) else {
                continue;
            };
            let rel = format!("docs/loop/tasks/{folder_name}/{topic}/ORDER.md");
            if best.as_ref().is_none_or(|(b_num, _, _, _)| num >= *b_num) {
                best = Some((num, rel, topic, text));
            }
        }
    }
    best.map(|(_, rel, topic, text)| (rel, topic, text))
}

fn collect_untracked_orders(root: &Path, dir: &Path, out: &mut HashSet<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_untracked_orders(root, &p, out);
        } else if p.file_name().and_then(|n| n.to_str()) == Some("ORDER.md") {
            if let Ok(rel) = p.strip_prefix(root) {
                out.insert(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
}

/// Extract backticked globs from a markdown section such as `## Scope` or `## Files in scope`.
fn parse_section_globs(markdown: &str, heading_prefix: &str) -> Vec<String> {
    let mut in_sec = false;
    let mut globs = Vec::new();
    let tick_re = regex::Regex::new(r"`([^`]+)`").expect("valid regex");
    for line in markdown.lines() {
        let trimmed = line.trim();
        if let Some(h) = trimmed.strip_prefix("## ") {
            in_sec = h
                .trim()
                .to_lowercase()
                .starts_with(&heading_prefix.to_lowercase());
            continue;
        }
        if in_sec {
            for cap in tick_re.captures_iter(trimmed) {
                let candidate = cap[1].trim();
                // Skip prose citations or flags; keep file paths and globs.
                if !candidate.is_empty()
                    && !candidate.starts_with('-')
                    && !candidate.contains(' ')
                    && (candidate.contains('/')
                        || candidate.contains('.')
                        || candidate.contains('*'))
                {
                    globs.push(candidate.to_string());
                }
            }
        }
    }
    globs
}

/// Claim the next unbound suborder file (`<agent_id>.md` or lowest unclaimed `<n>.md`) for a
/// subagent starting in `local/<lab>/<topic>/orders/`.
fn claim_next_suborder_for_agent(
    roots: &UorRoots,
    lab: &str,
    agent_id: &str,
    existing_bindings: &BTreeMap<String, String>,
) -> Option<String> {
    let (_, topic, _) = find_active_order(roots, lab)?;
    for base in [&roots.worktree_root, &roots.primary_root] {
        let orders_dir = base.join("local").join(lab).join(&topic).join("orders");
        if !orders_dir.is_dir() {
            continue;
        }
        let direct_name = format!("{agent_id}.md");
        if orders_dir.join(&direct_name).is_file() {
            return Some(direct_name);
        }
        let claimed: HashSet<&str> = existing_bindings.values().map(String::as_str).collect();
        let mut numbered: Vec<(u64, String)> = Vec::new();
        if let Ok(entries) = fs::read_dir(&orders_dir) {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                let Some(stem) = name.strip_suffix(".md") else {
                    continue;
                };
                if let Ok(n) = stem.parse::<u64>() {
                    if !claimed.contains(name.as_str()) {
                        numbered.push((n, name));
                    }
                }
            }
        }
        numbered.sort_by_key(|(n, _)| *n);
        if let Some((_, name)) = numbered.into_iter().next() {
            return Some(name);
        }
    }
    None
}

/// Find the suborder scope globs for a subagent from `local/<lab>/<topic>/orders/`.
///
/// Subagents are never granted write access by falling back to an unbound highest-numbered
/// `<n>.md`: an `agent_id` must match `orders/<agent_id>.md`, be bound in
/// `local/<lab>/guard/subagent-bindings.json` at `SubagentStart`, or (for Antigravity/Kimi) match
/// the active single subagent slot.
fn find_suborder_globs(
    roots: &UorRoots,
    lab: &str,
    topic: &str,
    agent_id: Option<&str>,
) -> Option<(PathBuf, Vec<String>)> {
    let gdir = guard_state_dir(roots, lab);
    for base in [&roots.worktree_root, &roots.primary_root] {
        let orders_dir = base.join("local").join(lab).join(topic).join("orders");
        if !orders_dir.is_dir() {
            continue;
        }
        if let Some(aid) = agent_id {
            // 1. Direct `<agent_id>.md` file.
            let direct = orders_dir.join(format!("{aid}.md"));
            if let Ok(text) = fs::read_to_string(&direct) {
                let globs = parse_section_globs(&text, "Scope");
                if !globs.is_empty() {
                    return Some((direct, globs));
                }
            }
            // 2. Explicit binding in `local/<lab>/guard/subagent-bindings.json` (recorded at
            //    `SubagentStart`).
            let bindings_file = gdir.join("subagent-bindings.json");
            if let Ok(btext) = fs::read_to_string(&bindings_file) {
                if let Ok(bindings) = serde_json::from_str::<BTreeMap<String, String>>(&btext) {
                    if let Some(bound_file) = bindings.get(aid) {
                        let bound_path = orders_dir.join(bound_file);
                        if let Ok(text) = fs::read_to_string(&bound_path) {
                            let globs = parse_section_globs(&text, "Scope");
                            if !globs.is_empty() {
                                return Some((bound_path, globs));
                            }
                        }
                    }
                    // If `subagent-bindings.json` exists and `aid` is not bound in it, refuse
                    // writes (this subagent has no suborder binding).
                    return None;
                }
            }
            // 3. Antigravity subagent conversation slot (`antigravity-session.json` records
            //    `subagent_cids` in spawn order).
            let agy_file = gdir.join("antigravity-session.json");
            if let Ok(atext) = fs::read_to_string(&agy_file) {
                if let Ok(astate) = serde_json::from_str::<AntigravitySessionState>(&atext) {
                    if let Some(pos) = astate.subagent_cids.iter().position(|x| x == aid) {
                        let slot_path = orders_dir.join(format!("{}.md", pos + 1));
                        if let Ok(text) = fs::read_to_string(&slot_path) {
                            let globs = parse_section_globs(&text, "Scope");
                            if !globs.is_empty() {
                                return Some((slot_path, globs));
                            }
                        }
                    }
                    return None;
                }
            }
            // 4. Kimi depth-1 slot (`depth-1` -> `1.md`; concurrent depth > 1 has no per-agent id
            //    in Kimi and is refused).
            if let Some(d_str) = aid.strip_prefix("depth-") {
                if d_str == "1" {
                    let slot1 = orders_dir.join("1.md");
                    if let Ok(text) = fs::read_to_string(&slot1) {
                        let globs = parse_section_globs(&text, "Scope");
                        if !globs.is_empty() {
                            return Some((slot1, globs));
                        }
                    }
                }
                return None;
            }
        }
    }
    None
}

/// Current git branch of `repo`, or empty string.
fn current_branch(repo: &Path) -> String {
    Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["branch", "--show-current"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// Check whether a write/edit to `raw_path` is permitted inside `roots` for `lab`.
fn check_write_path(
    roots: &UorRoots,
    lab: &str,
    subagent: &SubagentContext,
    cwd: &Path,
    raw_path: &str,
) -> Result<(), String> {
    if is_credential_path(raw_path) {
        return Err(format!(
            "CHARTER §3.13: writing or reading credential path {raw_path} is forbidden"
        ));
    }
    let resolved = resolve_target_path(cwd, raw_path);

    // 1. Check if the path is inside `.worktrees/<wt_name>/...` of `primary_root`.
    let worktrees_dir = roots.primary_root.join(".worktrees");
    if let Ok(wt_rel) = resolved.strip_prefix(&worktrees_dir) {
        let mut comps = wt_rel.components();
        let Some(wt_comp) = comps.next() else {
            return Err("cannot write directly to .worktrees/".into());
        };
        let wt_name = wt_comp.as_os_str().to_string_lossy();
        let own_prefix = format!("{lab}-");
        if wt_name != lab && !wt_name.starts_with(&own_prefix) {
            return Err(format!(
                "cross-worktree write refused: .worktrees/{wt_name} belongs to another lab (lab is {lab})"
            ));
        }
        let rest: PathBuf = comps.collect();
        let rel_str = rest.to_string_lossy().replace('\\', "/");
        let wt_root = UorRoots {
            worktree_root: worktrees_dir.join(wt_comp.as_os_str()),
            primary_root: roots.primary_root.clone(),
        };
        return check_repo_rel_write(&wt_root, lab, subagent, &rel_str);
    }

    // 2. Check if the path is inside `roots.worktree_root`.
    if let Ok(rel) = resolved.strip_prefix(&roots.worktree_root) {
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        // If `worktree_root` IS the shared primary checkout, only `local/<lab>/**` is writable.
        if roots.worktree_root == roots.primary_root
            && crate::state::shared_primary_checkout(&roots.primary_root).is_some()
            && !rel_str.starts_with(&format!("local/{lab}/"))
            && rel_str != format!("local/{lab}")
        {
            return Err(format!(
                "R10: writing to {rel_str} in the shared primary checkout is refused; work in .worktrees/{lab}-<topic>/ or local/{lab}/"
            ));
        }
        return check_repo_rel_write(roots, lab, subagent, &rel_str);
    }

    // 3. Check if the path is inside `roots.primary_root` (when `worktree_root` is a linked worktree).
    if let Ok(rel) = resolved.strip_prefix(&roots.primary_root) {
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if rel_str.starts_with(&format!("local/{lab}/")) || rel_str == format!("local/{lab}") {
            return Ok(());
        }
        return Err(format!(
            "R10: writing to {rel_str} in the shared primary checkout from a worktree is refused"
        ));
    }

    // 4. Outside the repository and its worktrees while running a session inside the repo.
    Err(format!(
        "path {} is outside the worktree {}",
        resolved.display(),
        roots.worktree_root.display()
    ))
}

/// Check a repository-relative path `rel` against `local/`, owner-only paths, rendered views,
/// archive immutability, subagent suborder scope, and active ORDER scope.
fn check_repo_rel_write(
    roots: &UorRoots,
    lab: &str,
    subagent: &SubagentContext,
    rel: &str,
) -> Result<(), String> {
    let rel = rel.trim_start_matches("./");
    if rel.is_empty() {
        return Err("cannot overwrite worktree root".into());
    }

    // 1. `local/` partitioning: a lab may write only under `local/<lab>/` (or `local/*-harness-target/`).
    if let Some(after_local) = rel.strip_prefix("local/") {
        let seg = after_local.split('/').next().unwrap_or("");
        if seg == lab || seg.ends_with("-harness-target") || seg == "harness-target" {
            return Ok(());
        }
        if seg == "loop" {
            return Err(
                "local/loop/ is reserved for owner install backups and the kill switch".into(),
            );
        }
        return Err(format!(
            "local/{seg}/ belongs to another lab; lab {lab} may write only under local/{lab}/"
        ));
    }

    // 2. Owner-only paths (`docs/harness/owner-paths.json`).
    let owner_globs = load_owner_globs(&roots.worktree_root)?;
    for g in &owner_globs {
        if crate::loopprcheck::glob_match(g, rel) {
            return Err(format!(
                "{rel} is an owner-only path ({g} in docs/harness/owner-paths.json)"
            ));
        }
    }

    // 3. Rendered views lock (`STATE.md`, `HANDOFF.md`, `docs/harness/LESSONS.md`).
    if matches!(rel, "STATE.md" | "HANDOFF.md" | "docs/harness/LESSONS.md") {
        let branch = current_branch(&roots.worktree_root);
        if branch != "kimi/render" {
            return Err(format!(
                "{rel} is a rendered view and may change only on branch kimi/render"
            ));
        }
    }

    // 4. Archive immutability (CHARTER §3.12): existing files in `archive/` are never modified.
    if rel.starts_with("archive/") && roots.worktree_root.join(rel).exists() {
        return Err(format!(
            "CHARTER §3.12: existing archive file {rel} is byte-identical and immutable"
        ));
    }

    // 5. Active ORDER and subagent suborder scope.
    let active = find_active_order(roots, lab);
    if let SubagentContext::Subagent(agent_id) = subagent {
        let Some((_, ref topic, _)) = active else {
            return Err(format!(
                "subagent write to {rel} refused: no active ORDER found for lab {lab}"
            ));
        };
        let Some((sub_path, sub_globs)) =
            find_suborder_globs(roots, lab, topic, agent_id.as_deref())
        else {
            return Err(format!(
                "subagent write to {rel} refused: no suborder in local/{lab}/{topic}/orders/ (subagents without a suborder are read-only)"
            ));
        };
        if !sub_globs
            .iter()
            .any(|g| crate::loopprcheck::glob_match(g, rel))
        {
            return Err(format!(
                "subagent write to {rel} is outside suborder {} scope [{}]",
                sub_path.display(),
                sub_globs.join(", ")
            ));
        }
    }

    if let Some((order_rel, _, order_text)) = active {
        let order_folder = order_rel.trim_end_matches("ORDER.md");
        let item_id = crate::loopclaim::order_item(&order_text);
        let item_rel = item_id
            .as_ref()
            .map(|id| format!("docs/roadmap/items/{id}.md"));
        let always_allowed = rel.starts_with(order_folder)
            || item_rel.as_deref() == Some(rel)
            || rel.starts_with("docs/harness/mistakes/")
            || rel == "docs/start/FILEMAP.md"
            || crate::loopclaim::SHARED_FILES.contains(&rel);
        if !always_allowed {
            // Collect scope globs from the item's `scope:` line (if present) and the ORDER's
            // `## Files in scope` / `## Scope` section.
            let mut scope_globs = item_id
                .as_deref()
                .and_then(|id| crate::loopclaim::read_item(&roots.worktree_root, id).ok())
                .and_then(|item| item.scope)
                .unwrap_or_default();
            if scope_globs.is_empty() {
                scope_globs = parse_section_globs(&order_text, "Files in scope");
                if scope_globs.is_empty() {
                    scope_globs = parse_section_globs(&order_text, "Scope");
                }
            }
            if !scope_globs.is_empty()
                && !scope_globs
                    .iter()
                    .any(|g| crate::loopprcheck::glob_match(g, rel))
            {
                return Err(format!(
                    "{rel} is outside active ORDER {order_rel} scope [{}]",
                    scope_globs.join(", ")
                ));
            }
        }
    }

    Ok(())
}

/// Evaluate `loop guard` for `client`, `lab`, `event` and `payload`, with `fallback_cwd` used when
/// the payload does not carry a directory.
pub fn evaluate_guard(
    fallback_cwd: &Path,
    client: &str,
    lab: &str,
    event: &str,
    payload: &Value,
) -> GuardOutcome {
    let cwd = payload_cwd(payload, fallback_cwd);
    let Some(roots) = find_uor_roots(&cwd) else {
        return GuardOutcome::OutsideRepo;
    };
    if let Err(e) = check_client(client) {
        return GuardOutcome::Refused(e);
    }
    let lab_slug = crate::state::slug(lab);
    if lab.trim().is_empty() {
        return GuardOutcome::Refused(
            "UOR_LAB / --lab is not set; every hook inside UOR-R4 requires a registered lab".into(),
        );
    }
    if let Err(e) = crate::labs::check_lab(&roots.worktree_root, &lab_slug) {
        return GuardOutcome::Refused(e);
    }

    let subagent = match update_and_detect_subagent(&roots, client, &lab_slug, event, payload) {
        Ok(s) => s,
        Err(e) => return GuardOutcome::Refused(e),
    };

    if event != "PreToolUse" {
        return GuardOutcome::Allow;
    }

    let (tool_name, tool_args) = extract_tool(payload);
    if is_shell_tool(&tool_name) || extract_command(&tool_args).is_some() {
        if let Some(cmd) = extract_command(&tool_args) {
            if let Err(reason) = check_command(&roots, &lab_slug, &subagent, &cwd, &cmd) {
                return GuardOutcome::Refused(reason);
            }
        }
    }

    let targets = extract_target_paths(&tool_name, &tool_args);
    if is_readonly_tool(&tool_name) {
        for t in &targets {
            if is_credential_path(t) {
                return GuardOutcome::Refused(format!(
                    "CHARTER §3.13: reading credential path {t} is forbidden"
                ));
            }
        }
        return GuardOutcome::Allow;
    }

    if is_write_tool(&tool_name) || !targets.is_empty() {
        for t in &targets {
            if let Err(reason) = check_write_path(&roots, &lab_slug, &subagent, &cwd, t) {
                return GuardOutcome::Refused(reason);
            }
        }
    }

    GuardOutcome::Allow
}

/// Build the client-specific canary payload attempting a denied write to `CHARTER.md` inside `repo`.
pub fn canary_payload_for_client(repo: &Path, client: &str) -> Value {
    let cwd = repo.display().to_string();
    let denied = repo.join("CHARTER.md").display().to_string();
    match client {
        "antigravity" => json!({
            "conversationId": "canary-root-session",
            "workspacePaths": [cwd],
            "toolCall": {
                "name": "write_file",
                "args": {
                    "Path": denied,
                    "Content": "canary write must be refused\n"
                }
            }
        }),
        "claude" => json!({
            "session_id": "canary-session",
            "cwd": cwd,
            "hook_event_name": "PreToolUse",
            "tool_name": "Write",
            "tool_input": {
                "file_path": denied,
                "content": "canary write must be refused\n"
            }
        }),
        "codex" => json!({
            "session_id": "canary-session",
            "cwd": cwd,
            "event": "PreToolUse",
            "tool_name": "write_file",
            "tool_input": {
                "file_path": denied,
                "content": "canary write must be refused\n"
            }
        }),
        "kimi" => json!({
            "session_id": "canary-session",
            "cwd": cwd,
            "hook_event_name": "PreToolUse",
            "tool_name": "WriteFile",
            "tool_input": {
                "path": denied,
                "content": "canary write must be refused\n"
            }
        }),
        "deepseek" => json!({
            "session_id": "canary-session",
            "cwd": cwd,
            "event": "PreToolUse",
            "tool_name": "Write",
            "tool_input": {
                "file_path": denied,
                "content": "canary write must be refused\n"
            }
        }),
        _ => json!({
            "cwd": cwd,
            "tool_name": "Write",
            "tool_input": { "file_path": denied }
        }),
    }
}

/// Run the canary write check for `client` in `repo`: returns `Ok(refusal_reason)` when the canary
/// write to a denied path is refused, or `Err(why)` if the canary write was allowed.
pub fn run_canary(repo: &Path, client: &str) -> Result<String, String> {
    let c = check_client(client)?;
    let payload = canary_payload_for_client(repo, &c);
    match evaluate_guard(repo, &c, &c, "PreToolUse", &payload) {
        GuardOutcome::Refused(reason) => Ok(reason),
        GuardOutcome::Allow => Err(format!(
            "canary write to CHARTER.md for client {c} was ALLOWED instead of refused"
        )),
        GuardOutcome::OutsideRepo => Err(format!(
            "canary write for client {c} resolved as outside-repo at {}",
            repo.display()
        )),
    }
}

// ---------------------------------------------------------------------------
// Loop Kit archive reader & skill YAML normalization (`loop install`, `loop copyright`)
// ---------------------------------------------------------------------------

/// Find the default Loop Kit zip path on the owner's laptop (`$UOR_LOOP_KIT_ZIP` or newest
/// `~/Downloads/LoopKit*.zip`).
pub fn default_kit_zip(home: Option<&Path>) -> Option<PathBuf> {
    if let Ok(env_p) = std::env::var("UOR_LOOP_KIT_ZIP") {
        let p = PathBuf::from(env_p.trim());
        if p.exists() {
            return Some(p);
        }
    }
    let mut candidates = Vec::new();
    if let Some(h) = home {
        candidates.push(h.to_path_buf());
    }
    if let Some(env_h) = std::env::var_os("HOME").map(PathBuf::from) {
        if !candidates.contains(&env_h) {
            candidates.push(env_h);
        }
    }
    for h in candidates {
        let downloads = h.join("Downloads");
        let Ok(entries) = fs::read_dir(&downloads) else {
            continue;
        };
        let mut matches: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                name.starts_with("LoopKit") && name.ends_with(".zip") && p.is_file()
            })
            .collect();
        matches.sort();
        if let Some(found) = matches.pop() {
            return Some(found);
        }
    }
    None
}

/// Read all text entries (`(entry_name, content)`) from a Loop Kit zip file or unpacked directory.
pub fn read_kit_entries(source: &Path) -> Result<Vec<(String, String)>, String> {
    if source.is_dir() {
        let mut out = Vec::new();
        collect_dir_texts(source, source, &mut out)?;
        out.sort_by(|a, b| a.0.cmp(&b.0));
        return Ok(out);
    }
    if !source.is_file() {
        return Err(format!(
            "Loop Kit source {} does not exist",
            source.display()
        ));
    }
    let list_out = Command::new("unzip")
        .arg("-Z1")
        .arg(source)
        .output()
        .map_err(|e| format!("cannot run unzip on {}: {e}", source.display()))?;
    if !list_out.status.success() {
        return Err(format!(
            "unzip -Z1 {} failed: {}",
            source.display(),
            String::from_utf8_lossy(&list_out.stderr).trim()
        ));
    }
    let names: Vec<String> = String::from_utf8_lossy(&list_out.stdout)
        .lines()
        .map(str::trim)
        .filter(|n| {
            !n.is_empty() && !n.ends_with('/') && (n.ends_with(".md") || n.ends_with(".txt"))
        })
        .map(String::from)
        .collect();
    let mut out = Vec::new();
    for name in names {
        let entry_out = Command::new("unzip")
            .arg("-p")
            .arg(source)
            .arg(&name)
            .output()
            .map_err(|e| format!("cannot read {name} from {}: {e}", source.display()))?;
        if !entry_out.status.success() {
            return Err(format!(
                "unzip -p {} {name} failed: {}",
                source.display(),
                String::from_utf8_lossy(&entry_out.stderr).trim()
            ));
        }
        let text = String::from_utf8_lossy(&entry_out.stdout).into_owned();
        out.push((name, text));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

fn collect_dir_texts(
    base: &Path,
    dir: &Path,
    out: &mut Vec<(String, String)>,
) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_dir_texts(base, &p, out)?;
        } else if p.is_file() {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.ends_with(".md") || name.ends_with(".txt") {
                let rel = p
                    .strip_prefix(base)
                    .unwrap_or(&p)
                    .to_string_lossy()
                    .replace('\\', "/");
                let text = fs::read_to_string(&p)
                    .map_err(|e| format!("cannot read {}: {e}", p.display()))?;
                out.push((rel, text));
            }
        }
    }
    Ok(())
}

/// Normalize a `SKILL.md` file's YAML frontmatter so an unquoted `description:` containing `: `
/// is quoted (preventing strict YAML parsers such as `dsh` from rejecting the skill).
pub fn normalize_skill_frontmatter(text: &str) -> String {
    let mut lines = text.lines();
    let Some(first) = lines.next() else {
        return text.to_string();
    };
    if first.trim() != "---" {
        return text.to_string();
    }
    let mut out = String::from("---\n");
    let mut in_fm = true;
    for line in lines {
        if in_fm && line.trim() == "---" {
            in_fm = false;
            out.push_str("---\n");
            continue;
        }
        if in_fm {
            if let Some(rest) = line.strip_prefix("description:") {
                let val = rest.trim();
                let already_quoted = (val.starts_with('"') && val.ends_with('"'))
                    || (val.starts_with('\'') && val.ends_with('\''));
                if !already_quoted && val.contains(": ") {
                    let escaped = val.replace('\\', "\\\\").replace('"', "\\\"");
                    out.push_str(&format!("description: \"{escaped}\"\n"));
                    continue;
                }
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Extract the six allowed skills (`skill_name -> normalized_SKILL.md`) from the Loop Kit entries,
/// explicitly skipping `start-here`.
pub fn load_allowed_kit_skills(source: &Path) -> Result<BTreeMap<String, String>, String> {
    let entries = read_kit_entries(source)?;
    let mut map = BTreeMap::new();
    for (name, content) in entries {
        for skill in ALLOWED_SKILLS {
            let suffix = format!("skills/{skill}/SKILL.md");
            if name == suffix || name.ends_with(&format!("/{suffix}")) {
                map.insert(skill.to_string(), normalize_skill_frontmatter(&content));
            }
        }
    }
    let missing: Vec<&str> = ALLOWED_SKILLS
        .iter()
        .copied()
        .filter(|s| !map.contains_key(*s))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "Loop Kit source {} is missing skill(s): {}",
            source.display(),
            missing.join(", ")
        ));
    }
    debug_assert!(!map.contains_key(EXCLUDED_SKILL));
    Ok(map)
}

// ---------------------------------------------------------------------------
// `loop install`
// ---------------------------------------------------------------------------

/// One planned file write for `loop install`.
#[derive(Debug, Clone)]
pub struct PlannedWrite {
    pub client: String,
    pub path: PathBuf,
    pub content: String,
    pub description: &'static str,
}

/// Build the planned writes for `client` under `home` and `repo/local`.
pub fn plan_client_install(
    repo: &Path,
    home: &Path,
    client: &str,
    skills: &BTreeMap<String, String>,
) -> Result<Vec<PlannedWrite>, String> {
    let c = check_client(client)?;
    let mut writes = Vec::new();

    // 1. Six allowed skills into the client's user-level skills directory.
    let sdir = client_skills_dir(home, &c);
    for (skill, text) in skills {
        if skill == EXCLUDED_SKILL {
            continue;
        }
        writes.push(PlannedWrite {
            client: c.clone(),
            path: sdir.join(skill).join("SKILL.md"),
            content: text.clone(),
            description: "Loop Kit skill (normalized YAML frontmatter)",
        });
    }

    // 2. Client-specific hook and rule adapter files.
    let guard_cmd = |ev: &str| {
        format!("$HOME/.cargo/bin/uor-harness loop guard --client {c} --lab {c} --event {ev}")
    };
    match c.as_str() {
        "claude" => {
            let settings_path = home.join(".claude").join("settings.json");
            let mut root_obj = read_existing_json_object(&settings_path)?;
            let mut hooks_obj = extract_existing_hooks_map(&root_obj, &settings_path)?;
            for ev in [
                "SessionStart",
                "PreToolUse",
                "SubagentStart",
                "SubagentStop",
                "Stop",
            ] {
                merge_event_hook(
                    &mut hooks_obj,
                    ev,
                    json!({
                        "matcher": ".*",
                        "hooks": [{
                            "type": "command",
                            "command": guard_cmd(ev)
                        }]
                    }),
                    &settings_path,
                )?;
            }
            root_obj.insert("hooks".to_string(), Value::Object(hooks_obj));
            let content =
                serde_json::to_string_pretty(&Value::Object(root_obj)).unwrap_or_default() + "\n";
            writes.push(PlannedWrite {
                client: c.clone(),
                path: settings_path,
                content,
                description: "Claude Code user-level loop guard hooks",
            });
            let claude_md = home.join(".claude").join("CLAUDE.md");
            let existing_claude_md = if claude_md.exists() {
                fs::read_to_string(&claude_md)
                    .map_err(|e| format!("cannot read existing {}: {e}", claude_md.display()))?
            } else {
                String::new()
            };
            let pointer = merge_claude_md(&existing_claude_md, &claude_md)?;
            writes.push(PlannedWrite {
                client: c.clone(),
                path: claude_md,
                content: pointer,
                description: "Claude Code user-level pointer to AGENTS.md and CHARTER.md §5.10",
            });
        }
        "codex" => {
            let hooks_path = home.join(".codex").join("hooks.json");
            let mut root_obj = read_existing_json_object(&hooks_path)?;
            root_obj
                .entry("schema".to_string())
                .or_insert_with(|| json!(1));
            let mut hooks_obj = extract_existing_hooks_map(&root_obj, &hooks_path)?;
            for ev in ["SessionStart", "PreToolUse", "Stop"] {
                merge_event_hook(
                    &mut hooks_obj,
                    ev,
                    json!({ "command": guard_cmd(ev) }),
                    &hooks_path,
                )?;
            }
            root_obj.insert("hooks".to_string(), Value::Object(hooks_obj));
            writes.push(PlannedWrite {
                client: c.clone(),
                path: hooks_path,
                content: serde_json::to_string_pretty(&Value::Object(root_obj)).unwrap_or_default()
                    + "\n",
                description: "Codex user-level loop guard hooks",
            });
            let rules_path = home.join(".codex").join("rules").join("uor-r4.rules");
            let rules_text = "\
# UOR-R4 Codex command rules (CHARTER.md §5.10)
prefix_rule(pattern=[\"git\", \"push\", \"--force\"], decision=\"forbidden\")
prefix_rule(pattern=[\"git\", \"push\", \"-f\"], decision=\"forbidden\")
prefix_rule(pattern=[\"git\", \"push\", \"origin\", \"main\"], decision=\"forbidden\")
prefix_rule(pattern=[\"git\", \"stash\"], decision=\"forbidden\")
prefix_rule(pattern=[\"git\", \"commit\", \"--no-verify\"], decision=\"forbidden\")
prefix_rule(pattern=[\"pkill\"], decision=\"forbidden\")
prefix_rule(pattern=[\"killall\"], decision=\"forbidden\")
";
            writes.push(PlannedWrite {
                client: c.clone(),
                path: rules_path,
                content: rules_text.to_string(),
                description: "Codex forbidden-command rules file",
            });
        }
        "kimi" => {
            let cfg_path = home.join(".kimi-code").join("config.toml");
            let existing = if cfg_path.exists() {
                fs::read_to_string(&cfg_path)
                    .map_err(|e| format!("cannot read existing {}: {e}", cfg_path.display()))?
            } else {
                String::new()
            };
            let cleaned = strip_kimi_hooks(&existing)?;
            let mut toml_out = cleaned.trim_end().to_string();
            if !toml_out.is_empty() {
                toml_out.push_str("\n\n");
            }
            toml_out.push_str("# uor-r4-loop-guard:start\n");
            for ev in [
                "SessionStart",
                "PreToolUse",
                "SubagentStart",
                "SubagentStop",
                "Stop",
            ] {
                toml_out.push_str(&format!(
                    "[[hooks]]\nevent = \"{ev}\"\nmatcher = \".*\"\ncommand = \"{}\"\ntimeout = 15\n\n",
                    guard_cmd(ev)
                ));
            }
            toml_out.push_str("# uor-r4-loop-guard:end\n");
            writes.push(PlannedWrite {
                client: c.clone(),
                path: cfg_path,
                content: toml_out,
                description: "Kimi Code loop guard hooks (retiring second loop hook)",
            });
            let pointer_start = "# UOR-R4 start pointer\n\nRead `AGENTS.md`, `CHARTER.md` §5.10 and `docs/loop/README.md` in the repository. Private rulebooks and the second Kimi loop are retired (D47).\n";
            let pointer_handoff = "# UOR-R4 handoff pointer\n\nSession handoff lives in `local/kimi/PICKUP.md` and the rendered `STATE.md` / `HANDOFF.md` views (`CHARTER.md` §5.10, `docs/loop/README.md`).\n";
            if pointer_start.len() > 1024 || pointer_handoff.len() > 1024 {
                return Err("Kimi pointer file exceeds 1024-byte cap".into());
            }
            writes.push(PlannedWrite {
                client: c.clone(),
                path: home.join(".kimi-code").join("UOR-R4-START.md"),
                content: pointer_start.to_string(),
                description: "Kimi <= 1 KB start pointer",
            });
            writes.push(PlannedWrite {
                client: c.clone(),
                path: home.join(".kimi-code").join("UOR-R4-HANDOFF.md"),
                content: pointer_handoff.to_string(),
                description: "Kimi <= 1 KB handoff pointer",
            });
        }
        "antigravity" => {
            let pdir = home
                .join(".gemini")
                .join("config")
                .join("plugins")
                .join("uor-r4-loop");
            let plugin_json = json!({
                "name": "uor-r4-loop",
                "version": "0.1.0",
                "description": "UOR-R4 five-lab loop guard plugin for Antigravity",
                "hooks": "./hooks.json"
            });
            let hooks_path = pdir.join("hooks.json");
            let mut root_obj = read_existing_json_object(&hooks_path)?;
            let mut hooks_obj = extract_existing_hooks_map(&root_obj, &hooks_path)?;
            for ev in ["SessionStart", "PreToolUse", "Stop"] {
                merge_event_hook(
                    &mut hooks_obj,
                    ev,
                    json!({
                        "matcher": ".*",
                        "hooks": [{
                            "type": "command",
                            "command": guard_cmd(ev),
                            "timeout": 15
                        }]
                    }),
                    &hooks_path,
                )?;
            }
            root_obj.insert("hooks".to_string(), Value::Object(hooks_obj));
            writes.push(PlannedWrite {
                client: c.clone(),
                path: pdir.join("plugin.json"),
                content: serde_json::to_string_pretty(&plugin_json).unwrap_or_default() + "\n",
                description: "Antigravity uor-r4-loop plugin manifest",
            });
            writes.push(PlannedWrite {
                client: c.clone(),
                path: hooks_path,
                content: serde_json::to_string_pretty(&Value::Object(root_obj)).unwrap_or_default()
                    + "\n",
                description: "Antigravity uor-r4-loop guard hooks",
            });
        }
        "deepseek" => {
            let hooks_path = home.join(".dsh").join("hooks.json");
            let mut root_obj = read_existing_json_object(&hooks_path)?;
            root_obj
                .entry("schema".to_string())
                .or_insert_with(|| json!(1));
            root_obj.insert(
                "note".to_string(),
                json!("DeepSeek dsh has no native hook runner (P-1); supervisor stream guard and policy gate enforce scope, and subagents without a suborder stay read-only."),
            );
            let mut hooks_obj = extract_existing_hooks_map(&root_obj, &hooks_path)?;
            for ev in ["SessionStart", "PreToolUse", "Stop"] {
                merge_event_hook(
                    &mut hooks_obj,
                    ev,
                    json!({ "command": guard_cmd(ev) }),
                    &hooks_path,
                )?;
            }
            root_obj.insert("hooks".to_string(), Value::Object(hooks_obj));
            writes.push(PlannedWrite {
                client: c.clone(),
                path: hooks_path,
                content: serde_json::to_string_pretty(&Value::Object(root_obj)).unwrap_or_default()
                    + "\n",
                description: "DeepSeek loop guard descriptor",
            });
            let overlay = "\
# UOR-R4 DeepSeek headless launch overlay (P-2, CHARTER.md §5.10)
provider: deepseek-account
model: deepseek-flash
guard_command: \"$HOME/.cargo/bin/uor-harness loop guard --client deepseek --lab deepseek --event PreToolUse\"
subagents_default: read-only
";
            writes.push(PlannedWrite {
                client: c.clone(),
                path: home
                    .join(".dsh")
                    .join("profiles")
                    .join("headless")
                    .join("cordis.patch.yml"),
                content: overlay.to_string(),
                description: "DeepSeek user-level headless overlay",
            });
            writes.push(PlannedWrite {
                client: c.clone(),
                path: repo
                    .join("local")
                    .join("deepseek")
                    .join("headless-overlay.yml"),
                content: overlay.to_string(),
                description: "DeepSeek repo-local headless overlay (gitignored local/)",
            });
        }
        _ => {}
    }

    Ok(writes)
}

/// Read an existing JSON object from `path`, failing closed if the file exists and cannot be
/// parsed as a top-level JSON object.
fn read_existing_json_object(path: &Path) -> Result<serde_json::Map<String, Value>, String> {
    if !path.exists() {
        return Ok(serde_json::Map::new());
    }
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("cannot read existing {}: {e}", path.display()))?;
    let val: Value = serde_json::from_str(&raw)
        .map_err(|e| format!("cannot parse existing {} as JSON: {e}", path.display()))?;
    val.as_object()
        .cloned()
        .ok_or_else(|| format!("existing {} is not a JSON object", path.display()))
}

/// Extract the existing `hooks` map from `root_obj`, failing closed if `hooks` is present and not
/// a JSON object.
fn extract_existing_hooks_map(
    root_obj: &serde_json::Map<String, Value>,
    path: &Path,
) -> Result<serde_json::Map<String, Value>, String> {
    match root_obj.get("hooks") {
        None | Some(Value::Null) => Ok(serde_json::Map::new()),
        Some(Value::Object(m)) => Ok(m.clone()),
        Some(_) => Err(format!(
            "existing `hooks` key in {} is not a JSON object",
            path.display()
        )),
    }
}

/// Merge a `uor-harness loop guard` hook entry into `hooks_obj[ev]`, preserving any existing
/// user hook entries on `ev` whose JSON does not invoke `uor-harness`.
fn merge_event_hook(
    hooks_obj: &mut serde_json::Map<String, Value>,
    ev: &str,
    new_entry: Value,
    path: &Path,
) -> Result<(), String> {
    let mut entries = match hooks_obj.get(ev) {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(arr)) => arr
            .iter()
            .filter(|elem| {
                !serde_json::to_string(elem)
                    .unwrap_or_default()
                    .contains("uor-harness")
            })
            .cloned()
            .collect(),
        Some(_) => {
            return Err(format!(
                "existing `hooks.{ev}` in {} is not a JSON array",
                path.display()
            ))
        }
    };
    entries.push(new_entry);
    hooks_obj.insert(ev.to_string(), Value::Array(entries));
    Ok(())
}

/// Remove existing `uor-harness` / retired UOR-R4 `[[hooks]]` blocks and any previous
/// `# uor-r4-loop-guard` block from Kimi's `config.toml`, keeping user `[[hooks]]` tables and all
/// other TOML tables intact, and failing on malformed TOML lines.
fn strip_kimi_hooks(toml_text: &str) -> Result<String, String> {
    let mut out: Vec<String> = Vec::new();
    let mut skip_guard_block = false;
    let mut in_hook_table = false;
    let mut hook_buf: Vec<String> = Vec::new();
    let mut bracket_depth: i32 = 0;

    let flush_hook_buf = |buf: &mut Vec<String>, out: &mut Vec<String>| {
        if buf.is_empty() {
            return;
        }
        let body = buf.join("\n");
        if !body.contains("uor-harness") && !body.contains("UOR-R4") {
            // Trim trailing blank lines inside preserved hook block before appending.
            while buf.last().is_some_and(|l| l.trim().is_empty()) {
                buf.pop();
            }
            out.append(buf);
            out.push(String::new());
        } else {
            buf.clear();
        }
    };

    for line in toml_text.lines() {
        let trimmed = line.trim();
        if trimmed == "# uor-r4-loop-guard:start" {
            if in_hook_table {
                flush_hook_buf(&mut hook_buf, &mut out);
                in_hook_table = false;
            }
            skip_guard_block = true;
            continue;
        }
        if trimmed == "# uor-r4-loop-guard:end" {
            skip_guard_block = false;
            continue;
        }
        if skip_guard_block {
            continue;
        }

        // Validate syntax of non-empty, non-comment lines outside multiline bracket arrays.
        if !trimmed.is_empty() && !trimmed.starts_with('#') && bracket_depth == 0 {
            let no_comment = trimmed.split('#').next().unwrap_or("").trim();
            let is_header = no_comment.starts_with('[') && no_comment.ends_with(']');
            let is_kv = no_comment.contains('=');
            if !is_header && !is_kv {
                return Err(format!(
                    "cannot parse existing ~/.kimi-code/config.toml (malformed line: {trimmed})"
                ));
            }
        }
        if !trimmed.starts_with('#') && (!trimmed.starts_with('[') || bracket_depth > 0) {
            for ch in trimmed.chars() {
                if ch == '[' {
                    bracket_depth += 1;
                } else if ch == ']' {
                    bracket_depth = (bracket_depth - 1).max(0);
                }
            }
        }

        if bracket_depth == 0 && trimmed == "[[hooks]]" {
            if in_hook_table {
                flush_hook_buf(&mut hook_buf, &mut out);
            }
            in_hook_table = true;
            hook_buf.push(line.to_string());
            continue;
        }
        if bracket_depth == 0 && in_hook_table && trimmed.starts_with('[') {
            flush_hook_buf(&mut hook_buf, &mut out);
            in_hook_table = false;
        }
        if in_hook_table {
            hook_buf.push(line.to_string());
        } else {
            out.push(line.to_string());
        }
    }
    if in_hook_table {
        flush_hook_buf(&mut hook_buf, &mut out);
    }
    Ok(out.join("\n"))
}

const CLAUDE_MD_START: &str = "<!-- uor-r4-loop-start -->";
const CLAUDE_MD_END: &str = "<!-- uor-r4-loop-end -->";
const CLAUDE_MD_BODY: &str = "# UOR-R4 pointer\n\nWhen working inside `~/uor-r4` or its worktrees, read `AGENTS.md` and `CHARTER.md` §5.10 first. Where any note disagrees with `CHARTER.md`, the charter wins.";

/// Merge the UOR-R4 pointer into `~/.claude/CLAUDE.md` between marker lines so existing user
/// instructions are preserved and re-runs replace only the marked block.
///
/// Counts `CLAUDE_MD_START` and `CLAUDE_MD_END` markers and accepts only `(0, 0)` or `(1, 1)` with
/// the start marker preceding the end marker; any other marker state returns `Err` so `loop install`
/// refuses with exit `2` and writes nothing.
fn merge_claude_md(existing: &str, path: &Path) -> Result<String, String> {
    let start_count = existing.matches(CLAUDE_MD_START).count();
    let end_count = existing.matches(CLAUDE_MD_END).count();
    let marked_block = format!("{CLAUDE_MD_START}\n{CLAUDE_MD_BODY}\n{CLAUDE_MD_END}");
    match (start_count, end_count) {
        (0, 0) => {
            if let Some(pos) = existing.find(CLAUDE_MD_BODY) {
                let before = &existing[..pos];
                let after = &existing[pos + CLAUDE_MD_BODY.len()..];
                let mut out = format!("{before}{marked_block}{after}");
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                return Ok(out);
            }
            let trimmed = existing.trim_end();
            if trimmed.is_empty() {
                Ok(format!("{marked_block}\n"))
            } else {
                Ok(format!("{trimmed}\n\n{marked_block}\n"))
            }
        }
        (1, 1) => {
            let s = existing.find(CLAUDE_MD_START).unwrap_or(0);
            let e = existing.find(CLAUDE_MD_END).unwrap_or(0);
            if s >= e {
                return Err(format!(
                    "REFUSED: malformed UOR-R4 marker state in {}: end marker `{CLAUDE_MD_END}` appears before start marker `{CLAUDE_MD_START}`",
                    path.display()
                ));
            }
            let before = &existing[..s];
            let after = &existing[e + CLAUDE_MD_END.len()..];
            let mut out = format!("{before}{marked_block}{after}");
            if !out.ends_with('\n') {
                out.push('\n');
            }
            Ok(out)
        }
        _ => Err(format!(
            "REFUSED: malformed UOR-R4 marker state in {} (found {start_count} start marker(s) `{CLAUDE_MD_START}` and {end_count} end marker(s) `{CLAUDE_MD_END}`; expected (0,0) or (1,1) with start before end)",
            path.display()
        )),
    }
}

/// Execute `loop install` for the selected clients.
pub fn run_install(
    repo: &Path,
    home: &Path,
    clients: &[String],
    zip_path: &Path,
    replace: bool,
    dry_run: bool,
) -> Result<Value, String> {
    let skills = load_allowed_kit_skills(zip_path)?;
    let mut all_writes = Vec::new();
    for c in clients {
        let mut w = plan_client_install(repo, home, c, &skills)?;
        all_writes.append(&mut w);
    }

    // Verify that no write inside `repo` lands outside `repo/local/`, checking both raw and
    // symlink-resolved paths so `--home <repo>/docs` is refused.
    let canon_repo = resolve_target_path(repo, ".");
    for w in &all_writes {
        let canon_w = resolve_target_path(repo, &w.path.to_string_lossy());
        for (r_base, w_target) in [(repo, &w.path), (canon_repo.as_path(), &canon_w)] {
            if let Ok(rel) = w_target.strip_prefix(r_base) {
                let rel_s = rel.to_string_lossy().replace('\\', "/");
                if !rel_s.starts_with("local/") {
                    return Err(format!(
                        "loop install refuses to write outside gitignored local/ inside the repository: {rel_s}"
                    ));
                }
            }
        }
    }

    // Check for differing existing files before writing anything.
    let mut conflicts = Vec::new();
    for w in &all_writes {
        if let Ok(existing) = fs::read_to_string(&w.path) {
            if existing != w.content {
                conflicts.push(w.path.display().to_string());
            }
        }
    }
    // Also check if `start-here` exists in any selected client's skills folder.
    let mut start_here_existing = Vec::new();
    for c in clients {
        let sh = client_skills_dir(home, c)
            .join(EXCLUDED_SKILL)
            .join("SKILL.md");
        if sh.exists() {
            start_here_existing.push((c.clone(), sh));
        }
    }
    for (_, sh) in &start_here_existing {
        conflicts.push(sh.display().to_string());
    }

    if !conflicts.is_empty() && !replace && !dry_run {
        return Err(format!(
            "REFUSED: {} existing file(s) differ from planned install (re-run with --apply --replace to back up to local/loop/install-backups/ and replace): {}",
            conflicts.len(),
            conflicts.join(", ")
        ));
    }

    static BACKUP_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let seq = BACKUP_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let epoch_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let run_stamp = format!("{epoch_ms}-{}-{seq}", std::process::id());
    let backup_root = repo
        .join("local")
        .join("loop")
        .join("install-backups")
        .join(run_stamp);
    let mut backups = Vec::new();
    if !dry_run {
        for (c, sh) in &start_here_existing {
            let bpath = backup_file_path(&backup_root, home, repo, c, sh);
            if let Some(p) = bpath.parent() {
                fs::create_dir_all(p).map_err(|e| format!("cannot create {}: {e}", p.display()))?;
            }
            fs::copy(sh, &bpath).map_err(|e| {
                format!(
                    "cannot back up {} to {}: {e}",
                    sh.display(),
                    bpath.display()
                )
            })?;
            fs::remove_file(sh)
                .map_err(|e| format!("cannot remove excluded skill {}: {e}", sh.display()))?;
            if let Some(dir) = sh.parent() {
                let _ = fs::remove_dir(dir);
            }
            backups.push(bpath.display().to_string());
        }

        for w in &all_writes {
            if let Ok(existing) = fs::read_to_string(&w.path) {
                if existing == w.content {
                    continue;
                }
                let bpath = backup_file_path(&backup_root, home, repo, &w.client, &w.path);
                if let Some(p) = bpath.parent() {
                    fs::create_dir_all(p)
                        .map_err(|e| format!("cannot create {}: {e}", p.display()))?;
                }
                fs::write(&bpath, existing.as_bytes())
                    .map_err(|e| format!("cannot write backup {}: {e}", bpath.display()))?;
                backups.push(bpath.display().to_string());
            }
            if let Some(parent) = w.path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
            }
            fs::write(&w.path, w.content.as_bytes())
                .map_err(|e| format!("cannot write {}: {e}", w.path.display()))?;
        }
    }

    let files: Vec<Value> = all_writes
        .iter()
        .map(|w| {
            json!({
                "client": w.client,
                "path": w.path.display().to_string(),
                "bytes": w.content.len(),
                "description": w.description,
            })
        })
        .collect();

    Ok(json!({
        "clients": clients,
        "applied": !dry_run,
        "dry_run": dry_run,
        "replace": replace,
        "skills_installed": ALLOWED_SKILLS,
        "start_here_excluded": true,
        "files": files,
        "conflicts": conflicts,
        "backups": backups,
    }))
}

fn backup_file_path(
    backup_root: &Path,
    home: &Path,
    repo: &Path,
    client: &str,
    target: &Path,
) -> PathBuf {
    let rel = target
        .strip_prefix(home)
        .or_else(|_| target.strip_prefix(repo))
        .unwrap_or(target);
    let clean: PathBuf = rel
        .components()
        .filter(|c| matches!(c, std::path::Component::Normal(_)))
        .collect();
    backup_root.join(client).join(clean)
}

// ---------------------------------------------------------------------------
// `loop onboard` and committed receipts (`docs/harness/onboarding/<client>.json`)
// ---------------------------------------------------------------------------

/// Verdict of a single onboarding check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnboardVerdict {
    #[serde(rename = "PASS")]
    Pass,
    #[serde(rename = "N/A")]
    NotApplicable,
    #[serde(rename = "FAIL")]
    Fail,
    #[serde(rename = "REFUSED")]
    Refused,
}

impl OnboardVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            OnboardVerdict::Pass => "PASS",
            OnboardVerdict::NotApplicable => "N/A",
            OnboardVerdict::Fail => "FAIL",
            OnboardVerdict::Refused => "REFUSED",
        }
    }
}

/// One row of `loop onboard --client <C>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardRow {
    pub check: String,
    pub verdict: OnboardVerdict,
    pub detail: String,
}

/// Full onboarding report for a client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientOnboardReport {
    pub schema: u32,
    pub client: String,
    pub lab: String,
    pub vendor: String,
    pub headless: String,
    pub skills_dir: String,
    pub skills_installed: Vec<String>,
    pub start_here_excluded: bool,
    pub hook_entry: String,
    pub canary_target: String,
    pub canary_outcome: String,
    pub checks: Vec<OnboardRow>,
    pub lane: String,
    pub pickup: String,
}

impl ClientOnboardReport {
    pub fn passed(&self) -> bool {
        self.checks.iter().all(|r| {
            matches!(
                r.verdict,
                OnboardVerdict::Pass | OnboardVerdict::NotApplicable
            )
        }) && self
            .checks
            .iter()
            .any(|r| r.check == "canary" && r.verdict == OnboardVerdict::Pass)
    }
}

/// Whether the client's `loop guard` hook configuration has already been applied under `home`.
fn client_hooks_applied(home: &Path, client: &str) -> bool {
    let config_path = match client {
        "claude" => home.join(".claude").join("settings.json"),
        "codex" => home.join(".codex").join("hooks.json"),
        "kimi" => home.join(".kimi-code").join("config.toml"),
        "antigravity" => home
            .join(".gemini")
            .join("config")
            .join("plugins")
            .join("uor-r4-loop")
            .join("hooks.json"),
        "deepseek" => home.join(".dsh").join("hooks.json"),
        _ => home.join("hooks.json"),
    };
    fs::read_to_string(&config_path)
        .map(|t| t.contains("loop guard"))
        .unwrap_or(false)
}

/// Check that the 6 allowed skills are present in `sdir` (or report `N/A` when `loop install --apply`
/// has not yet been run by the owner for that client) and that `start-here` is absent.
fn check_client_skills(_repo: &Path, home: &Path, client: &str) -> (OnboardRow, Vec<String>) {
    let sdir = client_skills_dir(home, client);
    let sh = sdir.join(EXCLUDED_SKILL).join("SKILL.md");
    if sh.exists() {
        return (
            OnboardRow {
                check: "skills".into(),
                verdict: OnboardVerdict::Refused,
                detail: format!(
                    "{EXCLUDED_SKILL} is installed at {} and must be removed so it cannot start a second setup",
                    sh.display()
                ),
            },
            Vec::new(),
        );
    }
    let mut found = Vec::new();
    let mut bad_yaml = Vec::new();
    for skill in ALLOWED_SKILLS {
        let p = sdir.join(skill).join("SKILL.md");
        if let Ok(text) = fs::read_to_string(&p) {
            if !text.trim().is_empty() {
                found.push(skill);
                for line in text.lines().take(12) {
                    if let Some(rest) = line.strip_prefix("description:") {
                        let v = rest.trim();
                        let quoted = (v.starts_with('"') && v.ends_with('"'))
                            || (v.starts_with('\'') && v.ends_with('\''));
                        if !quoted && v.contains(": ") {
                            bad_yaml.push(skill);
                        }
                    }
                }
            }
        }
    }
    let found_strings: Vec<String> = found.iter().map(|s| (*s).to_string()).collect();
    if found.len() == ALLOWED_SKILLS.len() {
        if !bad_yaml.is_empty() {
            if client_hooks_applied(home, client) {
                return (
                    OnboardRow {
                        check: "skills".into(),
                        verdict: OnboardVerdict::Fail,
                        detail: format!(
                            "unquoted YAML colon in description of {} under {}",
                            bad_yaml.join(", "),
                            sdir.display()
                        ),
                    },
                    found_strings,
                );
            }
            return (
                OnboardRow {
                    check: "skills".into(),
                    verdict: OnboardVerdict::NotApplicable,
                    detail: format!(
                        "N/A until owner runs uor-harness loop install --all --apply --replace (6 allowed skills present in {}; YAML frontmatter normalization staged for {})",
                        client_skills_rel(client),
                        bad_yaml.join(", ")
                    ),
                },
                found_strings,
            );
        }
        return (
            OnboardRow {
                check: "skills".into(),
                verdict: OnboardVerdict::Pass,
                detail: format!(
                    "6 allowed skills installed in {} (start-here excluded)",
                    client_skills_rel(client)
                ),
            },
            found_strings,
        );
    }
    if found.is_empty() && !client_hooks_applied(home, client) {
        return (
            OnboardRow {
                check: "skills".into(),
                verdict: OnboardVerdict::NotApplicable,
                detail: format!(
                    "N/A until owner runs uor-harness loop install --all --apply (target: {}; 6 allowed skills staged, start-here excluded)",
                    client_skills_rel(client)
                ),
            },
            found_strings,
        );
    }
    let missing: Vec<&str> = ALLOWED_SKILLS
        .iter()
        .copied()
        .filter(|s| !found.contains(s))
        .collect();
    (
        OnboardRow {
            check: "skills".into(),
            verdict: OnboardVerdict::Fail,
            detail: format!(
                "missing skill(s) in {}: {}",
                sdir.display(),
                missing.join(", ")
            ),
        },
        found_strings,
    )
}

/// Check hook/adapter wiring for `client`.
fn check_client_hooks(_repo: &Path, home: &Path, client: &str) -> OnboardRow {
    let hook_entry =
        format!("uor-harness loop guard --client {client} --lab {client} --event PreToolUse");
    let (config_path, target_rel) = match client {
        "claude" => (
            home.join(".claude").join("settings.json"),
            "~/.claude/settings.json",
        ),
        "codex" => (
            home.join(".codex").join("hooks.json"),
            "~/.codex/hooks.json + ~/.codex/rules/uor-r4.rules",
        ),
        "kimi" => (
            home.join(".kimi-code").join("config.toml"),
            "~/.kimi-code/config.toml",
        ),
        "antigravity" => (
            home.join(".gemini")
                .join("config")
                .join("plugins")
                .join("uor-r4-loop")
                .join("hooks.json"),
            "~/.gemini/config/plugins/uor-r4-loop/hooks.json",
        ),
        "deepseek" => (
            home.join(".dsh").join("hooks.json"),
            "~/.dsh/hooks.json + ~/.dsh/profiles/headless/cordis.patch.yml",
        ),
        _ => (home.join("hooks.json"), "~/hooks.json"),
    };
    if let Ok(text) = fs::read_to_string(&config_path) {
        if text.contains("loop guard") {
            if client == "kimi" && text.contains("Proceed|Continue) UOR-R4") {
                return OnboardRow {
                    check: "hooks".into(),
                    verdict: OnboardVerdict::Refused,
                    detail:
                        "Kimi's retired second loop hook is still present in ~/.kimi-code/config.toml"
                            .into(),
                };
            }
            let shown = config_path
                .strip_prefix(home)
                .map(|r| format!("~/{}", r.to_string_lossy().replace('\\', "/")))
                .unwrap_or_else(|_| config_path.display().to_string());
            let detail = if client == "deepseek" {
                format!(
                    "{hook_entry} wired in {shown} (P-1: dsh has no native hook runner; supervisor stream guard + policy gate enforce scope)"
                )
            } else {
                format!("{hook_entry} wired in {shown}")
            };
            return OnboardRow {
                check: "hooks".into(),
                verdict: OnboardVerdict::Pass,
                detail,
            };
        }
    }
    let p1_note = match client {
        "claude" => "P-1 pass: SessionStart, PreToolUse with subagent agent_id, SubagentStop, Stop",
        "antigravity" => "P-1 pass: SessionStart root conversationId, PreToolUse subagent conversationId, Stop",
        "kimi" => "P-1 pass: SessionStart, SubagentStart/SubagentStop depth tracking, PreToolUse, Stop",
        "deepseek" => "P-1: dsh has no native hook runner; headless overlay + supervisor stream guard + read-only subagent fallback",
        "codex" => "subagents read-only without suborder",
        _ => "loop guard staged",
    };
    OnboardRow {
        check: "hooks".into(),
        verdict: OnboardVerdict::NotApplicable,
        detail: format!(
            "N/A until owner runs uor-harness loop install --all --apply --replace (target: {target_rel}; {p1_note})"
        ),
    }
}

/// Path to the committed onboarding receipt `docs/harness/onboarding/<client>.json`.
pub fn onboarding_receipt_path(repo: &Path, client: &str) -> PathBuf {
    repo.join("docs")
        .join("harness")
        .join("onboarding")
        .join(format!("{client}.json"))
}

/// Run `loop onboard --client <C>` and return the structured report.
pub fn run_onboard(
    repo: &Path,
    home: &Path,
    client: &str,
    lab_env: &str,
) -> Result<ClientOnboardReport, String> {
    let c = check_client(client)?;
    let labs = crate::labs::load_labs(repo)?;
    let lab_info = labs
        .get(&c)
        .ok_or_else(|| format!("client {c} is not in docs/harness/labs.json"))?;

    let mut checks = Vec::new();

    // 1. rules reachable
    let charter_ok = fs::read_to_string(repo.join("CHARTER.md"))
        .map(|t| t.contains("5.10"))
        .unwrap_or(false);
    let agents_ok = repo.join("AGENTS.md").is_file();
    let loop_ok = repo.join("docs/loop/README.md").is_file();
    if charter_ok && agents_ok && loop_ok {
        checks.push(OnboardRow {
            check: "rules".into(),
            verdict: OnboardVerdict::Pass,
            detail: "CHARTER.md §5.10, AGENTS.md and docs/loop/README.md reachable".into(),
        });
    } else {
        checks.push(OnboardRow {
            check: "rules".into(),
            verdict: OnboardVerdict::Fail,
            detail: "missing CHARTER.md §5.10, AGENTS.md or docs/loop/README.md".into(),
        });
    }

    // 2. skills found
    let (skills_row, skills_installed) = check_client_skills(repo, home, &c);
    checks.push(skills_row);

    // 3. hooks on the current binary
    checks.push(check_client_hooks(repo, home, &c));

    // 4. in-process canary write to a denied path refused
    let (canary_verdict, canary_detail) = match run_canary(repo, &c) {
        Ok(reason) => (
            OnboardVerdict::Pass,
            format!(
                "in-process loop guard evaluation refused canary write to CHARTER.md ({reason})"
            ),
        ),
        Err(why) => (OnboardVerdict::Fail, why),
    };
    checks.push(OnboardRow {
        check: "canary".into(),
        verdict: canary_verdict,
        detail: canary_detail,
    });

    // 5. identity (UOR_LAB set and registered)
    let effective_lab = if lab_env.trim().is_empty() {
        String::new()
    } else {
        crate::state::slug(lab_env)
    };
    if effective_lab.is_empty() {
        checks.push(OnboardRow {
            check: "identity".into(),
            verdict: OnboardVerdict::Fail,
            detail: format!("UOR_LAB is not set (expected UOR_LAB={c})"),
        });
    } else if crate::labs::check_lab(repo, &effective_lab).is_err() {
        checks.push(OnboardRow {
            check: "identity".into(),
            verdict: OnboardVerdict::Refused,
            detail: format!("UOR_LAB={effective_lab} is not in docs/harness/labs.json"),
        });
    } else if effective_lab != c {
        checks.push(OnboardRow {
            check: "identity".into(),
            verdict: OnboardVerdict::Fail,
            detail: format!("UOR_LAB={effective_lab} does not match --client {c}"),
        });
    } else {
        checks.push(OnboardRow {
            check: "identity".into(),
            verdict: OnboardVerdict::Pass,
            detail: format!("UOR_LAB={c} registered in docs/harness/labs.json"),
        });
    }

    // 6. binary current
    let stale = crate::version::stale(repo);
    if let Some(line) = stale.line() {
        checks.push(OnboardRow {
            check: "binary".into(),
            verdict: OnboardVerdict::Fail,
            detail: line,
        });
    } else {
        checks.push(OnboardRow {
            check: "binary".into(),
            verdict: OnboardVerdict::Pass,
            detail: "uor-harness binary is current with origin/main".into(),
        });
    }

    let pickup_rel = format!("local/{c}/PICKUP.md");
    let pickup = if repo.join(&pickup_rel).is_file() {
        format!("{pickup_rel} (present)")
    } else {
        format!("{pickup_rel} (run `uor-harness loop next` then `uor-harness loop arrive`)")
    };

    Ok(ClientOnboardReport {
        schema: 1,
        client: c.clone(),
        lab: c.clone(),
        vendor: lab_info.vendor.clone(),
        headless: lab_info.headless.clone(),
        skills_dir: client_skills_rel(&c).to_string(),
        skills_installed,
        start_here_excluded: true,
        hook_entry: format!("uor-harness loop guard --client {c} --lab {c} --event <Event>"),
        canary_target: "CHARTER.md".to_string(),
        canary_outcome: if canary_verdict == OnboardVerdict::Pass {
            "REFUSED".to_string()
        } else {
            "ALLOWED".to_string()
        },
        checks,
        lane: client_lane(&c).to_string(),
        pickup,
    })
}

/// Write the committed onboarding receipt `docs/harness/onboarding/<client>.json`.
pub fn write_onboarding_receipt(
    repo: &Path,
    report: &ClientOnboardReport,
) -> Result<PathBuf, String> {
    if !report.passed() {
        return Err(format!(
            "cannot write onboarding receipt for {}: one or more checks did not PASS",
            report.client
        ));
    }
    let path = onboarding_receipt_path(repo, &report.client);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(report)
        .map_err(|e| format!("cannot serialize receipt: {e}"))?
        + "\n";
    fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(path)
}

/// Verify all five committed onboarding receipts in `docs/harness/onboarding/<client>.json` and
/// re-run each client's live canary write refusal check.
pub fn check_all_receipts(repo: &Path) -> Result<Vec<String>, String> {
    let labs = crate::labs::load_labs(repo)?;
    let expected_skills: Vec<String> = ALLOWED_SKILLS.iter().map(|s| (*s).to_string()).collect();
    let mut verified = Vec::new();
    for client in CLIENTS {
        let path = onboarding_receipt_path(repo, client);
        let text = fs::read_to_string(&path).map_err(|e| {
            format!(
                "missing onboarding receipt {} for {client}: {e}",
                path.display()
            )
        })?;
        let receipt: ClientOnboardReport = serde_json::from_str(&text)
            .map_err(|e| format!("malformed receipt {}: {e}", path.display()))?;
        if receipt.schema != 1 || receipt.client != client || receipt.lab != client {
            return Err(format!(
                "receipt {} has mismatched client/lab identity",
                path.display()
            ));
        }
        let Some(lab_info) = labs.get(client) else {
            return Err(format!("{client} is not in docs/harness/labs.json"));
        };
        if receipt.vendor != lab_info.vendor || receipt.headless != lab_info.headless {
            return Err(format!(
                "receipt {} vendor/headless disagrees with docs/harness/labs.json",
                path.display()
            ));
        }
        let skills_check = receipt.checks.iter().find(|r| r.check == "skills");
        let skills_na = skills_check.is_some_and(|r| r.verdict == OnboardVerdict::NotApplicable);
        let skills_ok = receipt.start_here_excluded
            && (receipt.skills_installed == expected_skills
                || (skills_na && receipt.skills_installed.is_empty()));
        if !skills_ok {
            return Err(format!(
                "receipt {} does not list the 6 allowed skills with start-here excluded",
                path.display()
            ));
        }
        if !receipt.passed() || receipt.canary_outcome != "REFUSED" {
            return Err(format!(
                "receipt {} does not record all PASS/N/A checks and a REFUSED canary",
                path.display()
            ));
        }
        // Re-run the live in-process canary write check for this client (the planted fault!).
        run_canary(repo, client)?;
        verified.push(client.to_string());
    }
    Ok(verified)
}

// ---------------------------------------------------------------------------
// `loop copyright`: local 8-word window comparison against the private Loop Kit
// ---------------------------------------------------------------------------

/// Number of consecutive words in a copyright comparison window.
pub const COPYRIGHT_WINDOW: usize = 8;

/// Tokenize text into `(lowercase_alphanumeric_word, 1_based_line_number)` pairs across the entire
/// file (including YAML frontmatter, which carries multi-sentence skill descriptions).
fn tokenize_words_with_lines(text: &str) -> Vec<(String, usize)> {
    let mut words = Vec::new();
    let mut cur = String::new();
    let mut line_no = 1usize;
    let mut word_start_line = 1usize;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            if cur.is_empty() {
                word_start_line = line_no;
            }
            cur.push(ch.to_ascii_lowercase());
        } else {
            if !cur.is_empty() {
                words.push((std::mem::take(&mut cur), word_start_line));
            }
            if ch == '\n' {
                line_no += 1;
            }
        }
    }
    if !cur.is_empty() {
        words.push((cur, word_start_line));
    }
    words
}

/// One 8-word overlap match found in a repository file (never stores or prints kit text).
#[derive(Debug, Clone, Serialize)]
pub struct CopyrightHit {
    pub repo_file: String,
    pub line: usize,
    pub kit_entry: String,
    pub word_offset: usize,
}

/// Compare tracked markdown and text files in `repo` against `kit_source` using 8-word windows.
///
/// Runs on the laptop only and never writes kit text or hashes to the repository.
pub fn check_copyright(repo: &Path, kit_source: &Path) -> Result<Vec<CopyrightHit>, String> {
    let kit_entries = read_kit_entries(kit_source)?;
    if kit_entries.is_empty() {
        return Err(format!(
            "Loop Kit source {} contained no readable markdown or text files",
            kit_source.display()
        ));
    }

    // Build an in-memory map from each 8-word window `Vec<String>` to the kit entry name.
    let mut kit_windows: HashMap<Vec<String>, String> = HashMap::new();
    for (entry_name, text) in &kit_entries {
        let words: Vec<String> = tokenize_words_with_lines(text)
            .into_iter()
            .map(|(w, _)| w)
            .collect();
        if words.len() < COPYRIGHT_WINDOW {
            continue;
        }
        for win in words.windows(COPYRIGHT_WINDOW) {
            kit_windows
                .entry(win.to_vec())
                .or_insert_with(|| entry_name.clone());
        }
    }

    let repo_files = list_repo_text_files(repo)?;
    let mut hits = Vec::new();
    for rel in repo_files {
        let full = repo.join(&rel);
        let Ok(text) = fs::read_to_string(&full) else {
            continue;
        };
        let words_with_lines = tokenize_words_with_lines(&text);
        if words_with_lines.len() < COPYRIGHT_WINDOW {
            continue;
        }
        let words: Vec<String> = words_with_lines.iter().map(|(w, _)| w.clone()).collect();
        let mut seen_entries = HashSet::new();
        for (idx, win) in words.windows(COPYRIGHT_WINDOW).enumerate() {
            if let Some(kit_entry) = kit_windows.get(win) {
                if seen_entries.insert(kit_entry.clone()) {
                    hits.push(CopyrightHit {
                        repo_file: rel.clone(),
                        line: words_with_lines[idx].1,
                        kit_entry: kit_entry.clone(),
                        word_offset: idx,
                    });
                }
            }
        }
    }
    Ok(hits)
}

/// List tracked `.md` and `.txt` files in `repo` (falling back to walking `docs/` and top-level
/// `.md` files when `repo` is not a git repository in unit tests).
fn list_repo_text_files(repo: &Path) -> Result<Vec<String>, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["ls-files", "-z"])
        .output();
    if let Ok(o) = out {
        if o.status.success() {
            let mut files: Vec<String> = o
                .stdout
                .split(|b| *b == 0)
                .filter(|s| !s.is_empty())
                .map(|s| String::from_utf8_lossy(s).into_owned())
                .filter(|p| {
                    !p.starts_with("archive/") && (p.ends_with(".md") || p.ends_with(".txt"))
                })
                .collect();
            files.sort();
            return Ok(files);
        }
    }
    let mut raw = Vec::new();
    collect_dir_texts(repo, repo, &mut raw)?;
    let mut files: Vec<String> = raw
        .into_iter()
        .map(|(rel, _)| rel)
        .filter(|p| !p.starts_with("archive/") && !p.starts_with("local/"))
        .collect();
    files.sort();
    Ok(files)
}

// ---------------------------------------------------------------------------
// CLI entry point (`uor-harness loop <guard|install|onboard|copyright>`)
// ---------------------------------------------------------------------------

fn parse_flags(
    args: &[String],
    values: &[&str],
    switches: &[&str],
) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if switches.contains(&a) {
            out.insert(a.to_string(), String::new());
        } else if values.contains(&a) {
            let v = args
                .get(i + 1)
                .ok_or_else(|| format!("{a} needs a value"))?;
            out.insert(a.to_string(), v.clone());
            i += 1;
        } else {
            return Err(format!("unexpected argument {a:?}"));
        }
        i += 1;
    }
    Ok(out)
}

/// Dispatch `loop guard`, `loop install`, `loop onboard` and `loop copyright`.
pub fn cli(root: &Path, lab: &str, action: &str, args: &[String]) -> u8 {
    match action {
        "guard" => cli_guard(root, lab, args),
        "install" => cli_install(root, lab, args),
        "onboard" => cli_onboard(root, lab, args),
        "copyright" => cli_copyright(root, args),
        _ => {
            eprintln!("{USAGE}");
            2
        }
    }
}

fn cli_guard(root: &Path, lab: &str, args: &[String]) -> u8 {
    let in_repo_fallback = find_uor_roots(root).is_some();
    let flags = match parse_flags(args, &["--client", "--event", "--input", "--lab"], &[]) {
        Ok(f) => f,
        Err(e) => {
            if !in_repo_fallback {
                return 0;
            }
            eprintln!("uor-harness loop guard: {e}\n{USAGE}");
            return 2;
        }
    };
    let Some(client) = flags.get("--client").map(String::as_str) else {
        if !in_repo_fallback {
            return 0;
        }
        eprintln!("uor-harness loop guard: missing --client\n{USAGE}");
        return 2;
    };
    let Some(event) = flags.get("--event").map(String::as_str) else {
        if !in_repo_fallback {
            return 0;
        }
        eprintln!("uor-harness loop guard: missing --event\n{USAGE}");
        return 2;
    };
    let effective_lab = flags
        .get("--lab")
        .map(String::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or(lab);

    let raw_payload = if let Some(input_path) = flags.get("--input") {
        match fs::read_to_string(input_path) {
            Ok(s) => s,
            Err(e) => {
                if !in_repo_fallback {
                    return 0;
                }
                eprintln!("uor-harness loop guard: cannot read {input_path}: {e}");
                return 2;
            }
        }
    } else if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        let mut buf = String::new();
        let _ = std::io::stdin().read_to_string(&mut buf);
        buf
    } else {
        String::new()
    };

    let payload: Value = if raw_payload.trim().is_empty() {
        json!({})
    } else {
        match serde_json::from_str(&raw_payload) {
            Ok(v) => v,
            Err(e) => {
                if !in_repo_fallback {
                    return 0;
                }
                eprintln!("uor-harness loop guard: invalid JSON payload: {e}");
                return 2;
            }
        }
    };

    match evaluate_guard(root, client, effective_lab, event, &payload) {
        GuardOutcome::OutsideRepo => 0,
        GuardOutcome::Allow => {
            if client == "antigravity" && event == "PreToolUse" {
                println!("{}", json!({ "decision": "allow" }));
            }
            0
        }
        GuardOutcome::Refused(reason) => {
            if client == "antigravity" && event == "PreToolUse" {
                println!("{}", json!({ "decision": "deny", "reason": reason }));
            }
            eprintln!("REFUSED: {reason}");
            2
        }
    }
}

fn cli_install(root: &Path, lab: &str, args: &[String]) -> u8 {
    let flags = match parse_flags(
        args,
        &["--client", "--home", "--zip"],
        &["--all", "--apply", "--replace", "--dry-run", "--json"],
    ) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("uor-harness loop install: {e}\n{USAGE}");
            return 2;
        }
    };
    if flags.contains_key("--apply") && flags.contains_key("--dry-run") {
        eprintln!("uor-harness loop install: cannot pass both --apply and --dry-run");
        return 2;
    }
    let all = flags.contains_key("--all");
    let client_opt = flags.get("--client").cloned();
    if all == client_opt.is_some() {
        eprintln!("uor-harness loop install: pass exactly one of --client <C> or --all");
        return 2;
    }
    let clients: Vec<String> = if all {
        CLIENTS.iter().map(|s| (*s).to_string()).collect()
    } else {
        match check_client(client_opt.as_deref().unwrap_or("")) {
            Ok(c) => vec![c],
            Err(e) => {
                eprintln!("uor-harness loop install: {e}");
                return 2;
            }
        }
    };

    let explicit_home = flags.get("--home").map(PathBuf::from);
    let real_home = std::env::var_os("HOME").map(PathBuf::from);
    let home = explicit_home.or(real_home);
    let Some(home) = home else {
        eprintln!("uor-harness loop install: HOME is not set and --home was not given");
        return 2;
    };

    let apply = flags.contains_key("--apply");
    let dry_run = !apply;
    let lab_env = std::env::var_os("UOR_LAB");
    let lab_present = !lab.is_empty() || lab_env.is_some();
    let active_lab = if !lab.trim().is_empty() {
        lab.trim().to_string()
    } else {
        lab_env
            .map(|v| v.to_string_lossy().into_owned())
            .unwrap_or_default()
    };
    // Owner decision (#2028): `loop install` is plan-only by default; `--apply` writes files and is
    // refused whenever `UOR_LAB` is present in the environment, regardless of `--home` or `HOME`.
    if apply && lab_present {
        eprintln!(
            "uor-harness loop install: REFUSED: owner decision (#2028): loop install --apply may be run only by the owner from Terminal with UOR_LAB unset (found UOR_LAB={active_lab:?})"
        );
        return 2;
    }

    let zip = flags
        .get("--zip")
        .map(PathBuf::from)
        .or_else(|| default_kit_zip(Some(&home)));
    let Some(zip) = zip else {
        eprintln!(
            "uor-harness loop install: REFUSED: Loop Kit zip not found (pass --zip <path> or place LoopKit*.zip in ~/Downloads)"
        );
        return 2;
    };

    let replace = flags.contains_key("--replace");
    let want_json = flags.contains_key("--json");

    match run_install(root, &home, &clients, &zip, replace, dry_run) {
        Ok(summary) => {
            if want_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&summary).unwrap_or_default()
                );
            } else {
                if let Some(files) = summary.get("files").and_then(Value::as_array) {
                    println!(
                        "loop install ({}): {} file(s) {}:",
                        if dry_run {
                            "plan-only; pass --apply to write"
                        } else {
                            "applied"
                        },
                        files.len(),
                        if dry_run { "planned" } else { "written" },
                    );
                    for f in files {
                        println!(
                            "  [{}] {} ({} B) — {}",
                            f["client"].as_str().unwrap_or(""),
                            f["path"].as_str().unwrap_or(""),
                            f["bytes"].as_u64().unwrap_or(0),
                            f["description"].as_str().unwrap_or("")
                        );
                    }
                }
                if let Some(backups) = summary.get("backups").and_then(Value::as_array) {
                    if !backups.is_empty() {
                        println!(
                            "backed up {} file(s) to local/loop/install-backups/:",
                            backups.len()
                        );
                        for b in backups {
                            println!("  {}", b.as_str().unwrap_or(""));
                        }
                    }
                }
            }
            0
        }
        Err(e) => {
            eprintln!("uor-harness loop install: {e}");
            2
        }
    }
}

fn cli_onboard(root: &Path, lab: &str, args: &[String]) -> u8 {
    let flags = match parse_flags(
        args,
        &["--client", "--home"],
        &["--write-receipt", "--check-receipts", "--json"],
    ) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("uor-harness loop onboard: {e}\n{USAGE}");
            return 2;
        }
    };
    let want_json = flags.contains_key("--json");
    if flags.contains_key("--check-receipts") {
        match check_all_receipts(root) {
            Ok(verified) => {
                if want_json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "verdict": "PASS",
                            "clients": verified,
                        }))
                        .unwrap_or_default()
                    );
                } else {
                    println!(
                        "PASS onboarding receipts: {} verified and in-process canary writes refused ({})",
                        verified.len(),
                        verified.join(", ")
                    );
                }
                return 0;
            }
            Err(e) => {
                eprintln!("FAIL onboarding receipts: {e}");
                return 1;
            }
        }
    }

    let Some(client) = flags.get("--client").map(String::as_str) else {
        eprintln!("uor-harness loop onboard: pass --client <C> or --check-receipts\n{USAGE}");
        return 2;
    };
    let home = flags
        .get("--home")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));

    let effective_lab = if lab.trim().is_empty() {
        std::env::var("UOR_LAB").unwrap_or_default()
    } else {
        lab.to_string()
    };

    let report = match run_onboard(root, &home, client, &effective_lab) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("uor-harness loop onboard: {e}");
            return 2;
        }
    };

    if flags.contains_key("--write-receipt") {
        if let Err(e) = write_onboarding_receipt(root, &report) {
            eprintln!("uor-harness loop onboard: {e}");
            return 1;
        }
    }

    if want_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );
    } else {
        for row in &report.checks {
            println!(
                "{:<7} {:<10} {}",
                row.verdict.as_str(),
                row.check,
                row.detail
            );
        }
        println!("lane:   {}", report.lane);
        println!("pickup: {}", report.pickup);
    }

    if report.passed() {
        0
    } else if report
        .checks
        .iter()
        .any(|r| r.verdict == OnboardVerdict::Refused)
    {
        2
    } else {
        1
    }
}

fn cli_copyright(root: &Path, args: &[String]) -> u8 {
    let flags = match parse_flags(args, &["--zip"], &["--json"]) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("uor-harness loop copyright: {e}\n{USAGE}");
            return 2;
        }
    };
    let zip = flags
        .get("--zip")
        .map(PathBuf::from)
        .or_else(|| default_kit_zip(None));
    let Some(zip) = zip else {
        eprintln!(
            "REFUSED: loop copyright runs on the laptop only and requires the owner's private Loop Kit zip (--zip <path> or ~/Downloads/LoopKit*.zip)"
        );
        return 2;
    };
    let want_json = flags.contains_key("--json");
    match check_copyright(root, &zip) {
        Ok(hits) => {
            if want_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "verdict": if hits.is_empty() { "PASS" } else { "FAIL" },
                        "window_words": COPYRIGHT_WINDOW,
                        "matches": hits.len(),
                        "hits": hits,
                    }))
                    .unwrap_or_default()
                );
            } else if hits.is_empty() {
                println!(
                    "PASS copyright: 0 matching {COPYRIGHT_WINDOW}-word window(s) across repository text"
                );
            } else {
                eprintln!(
                    "FAIL copyright: {} matching {COPYRIGHT_WINDOW}-word window(s) found:",
                    hits.len()
                );
                for h in &hits {
                    eprintln!(
                        "  {}:{} (word offset {}) overlaps {}",
                        h.repo_file, h.line, h.word_offset, h.kit_entry
                    );
                }
            }
            if hits.is_empty() {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("REFUSED: {e}");
            2
        }
    }
}
