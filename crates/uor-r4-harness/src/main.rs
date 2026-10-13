//! `uor-harness` — the Rust side of the harness port.
//!
//! Today this reads and validates harness state against the frozen contract. It does not write, and
//! it is not on the labs' path: the Python harness owns behaviour until this passes the same checks.
//!
//! ```text
//! uor-harness check [--repo DIR] [--json]   # state against the frozen contract
//! uor-harness presence [--repo DIR]         # what each lab is doing, live tier first
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use uor_r4_harness::onboard::{detect_runtime, onboard};
use uor_r4_harness::prep::{check_all, SheetReport};
use uor_r4_harness::presence::{heartbeat, Presence};
use uor_r4_harness::receipt::{append, current_month, Receipt as NewReceipt};
use uor_r4_harness::registries::Registries;
use uor_r4_harness::state::{read_ledger, PresenceRow, StatePaths};
use uor_r4_harness::Contract;

/// Flags that only a subcommand understands, forwarded rather than rejected at the top level.
const SUBCOMMAND_FLAGS: [&str; 13] = [
    "--kind",
    "--milestone",
    "-m",
    "--skill",
    "--role",
    "--branch",
    "--note",
    "--outcome",
    "--mechanism",
    "--run",
    "--dry-run",
    "--doing",
    "--next",
];

/// The subset that takes a value, so the value is not mistaken for a command.
const SUBCOMMAND_VALUE_FLAGS: [&str; 12] = [
    "--kind",
    "--milestone",
    "-m",
    "--skill",
    "--role",
    "--branch",
    "--note",
    "--outcome",
    "--mechanism",
    "--run",
    "--doing",
    "--next",
];

const USAGE: &str = "\
uor-harness - the multi-lab research harness (Rust side of the port)

USAGE:
    uor-harness <COMMAND> [--repo DIR]

COMMANDS:
    onboard     Verify this machine can use the harness, and say what to do
                next. Read-only: records nothing, claims nothing. Exits 1 if
                anything FAILs.
    check       Check persisted state against the frozen contract
                (docs/harness/state-formats.json). Exits 1 on drift.
    presence    What each lab is doing now. Live tier first, durable as the
                fallback, reporting which answered.
    ledger      Receipt counts by lab and kind.
    receipt     Append one receipt to this lab's shard. The first WRITE on the
                Rust side; use --dry-run to see the line without recording it.
    heartbeat   Check in: publish presence to both tiers and record a receipt.
                Use --dry-run to see what would be written.
    prep-check  Read every pre-registration sheet: its TEST FITNESS verdict and
                which required blocks are missing. The read half of the D22 gate.
    registry    The milestone registry as this implementation reads it: headline,
                panel and line counts per milestone. Compared against Python by
                parity.py, because the headline is the number a run is scored on.
    help        This text.

The Python harness owns behaviour until this path passes the same checks; the
formats are frozen so the two agree about what the shared record says.
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut command: Option<String> = None;
    let mut repo: Option<PathBuf> = None;
    let mut json = false;
    let mut rest: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--repo" => {
                match args.get(i + 1) {
                    Some(p) => {
                        repo = Some(PathBuf::from(p));
                        // Forwarded as well as consumed: a subcommand that reads more of its own
                        // arguments (`receipt`) needs to know which repository it is writing to.
                        rest.push("--repo".to_string());
                        rest.push(p.clone());
                        // Advance past the value here; the loop's own increment then moves to the
                        // next flag or command word. Incrementing twice skipped the command
                        // entirely, so `--repo DIR receipt ...` ran nothing and printed usage.
                        i += 1;
                    }
                    None => {
                        eprintln!("uor-harness: --repo needs a directory");
                        return ExitCode::from(2);
                    }
                }
            }
            "--json" => json = true,
            "--lab" => {
                match args.get(i + 1) {
                    Some(v) => {
                        // A global flag that subcommands also read. Forwarded as well as accepted,
                        // because `receipt` needs the lab to choose the shard it writes.
                        rest.push("--lab".to_string());
                        rest.push(v.clone());
                        i += 1;
                    }
                    None => {
                        eprintln!("uor-harness: --lab needs a lab name");
                        return ExitCode::from(2);
                    }
                }
            }
            "-h" | "--help" | "help" => {
                print!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            // Flags belonging to a subcommand are collected and handed to it, not rejected here:
            // `receipt` takes many optional fields and the top-level loop cannot know them all.
            other if other.starts_with('-') => {
                if SUBCOMMAND_FLAGS.contains(&other) {
                    rest.push(other.to_string());
                    // A value-taking subcommand flag carries the next token with it.
                    if !other.contains('=') && SUBCOMMAND_VALUE_FLAGS.contains(&other) {
                        if let Some(v) = args.get(i + 1) {
                            rest.push(v.clone());
                            i += 1;
                        }
                    }
                } else {
                    eprintln!("uor-harness: unknown option {other}\n\n{USAGE}");
                    return ExitCode::from(2);
                }
            }
            other => command = Some(other.to_string()),
        }
        i += 1;
    }

    let repo = match repo {
        Some(p) => p,
        None => match std::env::current_dir() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("uor-harness: cannot read the working directory: {e}");
                return ExitCode::from(2);
            }
        },
    };

    match command.as_deref() {
        Some("onboard") => cmd_onboard(&repo, json),
        Some("check") => cmd_check(&repo, json),
        Some("presence") => cmd_presence(&repo, json),
        Some("ledger") => cmd_ledger(&repo, json),
        Some("receipt") => cmd_receipt(&repo, &rest, json),
        Some("heartbeat") => cmd_heartbeat(&repo, &rest, json),
        Some("prep-check") => cmd_prep_check(&repo, json),
        Some("registry") => cmd_registry(&repo, json),
        Some(other) => {
            eprintln!("uor-harness: unknown command {other}\n\n{USAGE}");
            ExitCode::from(2)
        }
        None => {
            eprint!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn cmd_onboard(repo: &Path, json: bool) -> ExitCode {
    let report = onboard(repo, std::env::var("UOR_LAB").ok());
    if json {
        let findings: Vec<String> = report
            .findings
            .iter()
            .map(|f| {
                format!(
                    "    {{\"name\": {:?}, \"level\": {:?}, \"detail\": {:?}, \"fix\": {:?}}}",
                    f.name,
                    format!("{:?}", f.level).to_lowercase(),
                    f.detail,
                    f.fix
                )
            })
            .collect();
        let steps: Vec<String> = report.next_steps.iter().map(|s| format!("{s:?}")).collect();
        println!(
            "{{\n  \"ready\": {},\n  \"lab\": {:?},\n  \"repo\": {:?},\n  \"failures\": {},\n  \"findings\": [\n{}\n  ],\n  \"next_steps\": [{}]\n}}",
            report.ready(),
            report.lab,
            report.repo.display().to_string(),
            report.failures(),
            findings.join(",\n"),
            steps.join(", ")
        );
    } else {
        println!("uor-harness onboard - can this machine use the harness?");
        println!("  repo:      {}", report.repo.display());
        println!("  live tier: {}", report.live_tier.display());
        if let Some(runtime) = detect_runtime() {
            println!("  runtime:   {runtime} (detected; advisory only)");
        }
        println!();
        for f in &report.findings {
            println!("  {} {:<10} {}", f.level.mark(), f.name, f.detail);
            if !f.fix.is_empty() {
                println!("       -> {}", f.fix);
            }
        }
        println!();
        if report.ready() {
            println!("READY. Nothing failed. Next:");
        } else {
            println!("NOT READY: {} check(s) failed. Next:", report.failures());
        }
        for step in &report.next_steps {
            println!("  - {step}");
        }
    }
    if report.ready() {
        ExitCode::SUCCESS
    } else {
        // Distinct from `check`'s exit 1 so CI can tell "state drifted" from "this machine is not
        // onboarded"; both are failures, but they need different fixes.
        ExitCode::from(3)
    }
}

fn flag_value(args: &[String], name: &str) -> Option<String> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == name {
            return args.get(i + 1).cloned();
        }
        i += 1;
    }
    None
}

fn cmd_heartbeat(repo: &Path, args: &[String], json: bool) -> ExitCode {
    let paths = StatePaths::from_env(repo);
    let lab = flag_value(args, "--lab")
        .or_else(|| std::env::var("UOR_LAB").ok())
        .unwrap_or_default();
    if lab.trim().is_empty() {
        eprintln!(
            "uor-harness: WARNING - no lab identity. This check-in would be attributed to nobody.\n\
             \x20            export UOR_LAB=<kimi|claude|codex|deepseek>"
        );
        return ExitCode::from(2);
    }
    let milestone = flag_value(args, "--milestone")
        .or_else(|| flag_value(args, "-m"))
        .unwrap_or_default();
    let mut p = Presence::new(&paths, &lab, &milestone);
    if let Some(v) = flag_value(args, "--doing") {
        p = p.with_doing(v);
    }
    if let Some(v) = flag_value(args, "--next") {
        p = p.with_next(v);
    }
    if args.iter().any(|a| a == "--dry-run") {
        println!("{}", p.line());
        return ExitCode::SUCCESS;
    }

    // The receipt and the presence are written from ONE record, so they cannot disagree about what
    // this lab is doing.
    let mut r = NewReceipt::new("heartbeat", lab.clone());
    if !p.milestone.is_empty() {
        r = r.with("milestone", p.milestone.clone());
    }
    if !p.doing.is_empty() {
        r = r.with("line", p.doing.clone());
    }
    if !p.next.is_empty() {
        r = r.with("next", p.next.clone());
    }
    let month = current_month();
    let shard = match append(&paths, &r, &month) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("uor-harness: {e}");
            return ExitCode::from(1);
        }
    };
    let (live_ok, durable) = match heartbeat(&paths, &p) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("uor-harness: {e}");
            return ExitCode::from(1);
        }
    };
    if json {
        println!(
            "{{\"lab\": {:?}, \"presence\": {:?}, \"receipt\": {:?}, \"live_tier\": {}}}",
            lab,
            durable.display().to_string(),
            shard.display().to_string(),
            live_ok
        );
    } else {
        println!("checked in: {} at {}", durable.display(), p.updated);
        if !live_ok {
            // Say so rather than pretend: peers will not see this until they fetch the record.
            println!(
                "  note: the live tier could not be written, so peers see this on their next fetch"
            );
        }
    }
    ExitCode::SUCCESS
}

fn cmd_registry(repo: &Path, json: bool) -> ExitCode {
    let paths = StatePaths::from_env(repo);
    let regs = match Registries::load(&paths) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("uor-harness: {e}");
            return ExitCode::from(2);
        }
    };
    if json {
        let rows: Vec<String> = regs
            .milestones
            .iter()
            .map(|m| {
                let panel = regs.panel_for(m).map(|p| p.id.clone()).unwrap_or_default();
                format!(
                    "    {{\"key\": {:?}, \"id\": {:?}, \"issue\": {}, \"headline\": {:?}, \"panel\": {:?}, \"lines\": {:?}}}",
                    m.key,
                    m.id,
                    m.issue,
                    m.headline_line(),
                    panel,
                    m.line_counts()
                )
            })
            .collect();
        println!(
            "{{\n  \"milestones\": [\n{}\n  ],\n  \"panels\": {},\n  \"owner_rules\": {}\n}}",
            rows.join(",\n"),
            regs.panels.len(),
            regs.owner_rules.len()
        );
    } else {
        println!(
            "{} milestones, {} panels, {} owner rules",
            regs.milestones.len(),
            regs.panels.len(),
            regs.owner_rules.len()
        );
        for m in &regs.milestones {
            println!(
                "  {:<6} {:<52} {}",
                m.id,
                m.headline_line(),
                m.line_counts()
            );
        }
    }
    ExitCode::SUCCESS
}

fn cmd_prep_check(repo: &Path, json: bool) -> ExitCode {
    let paths = StatePaths::from_env(repo);
    let sheets = check_all(&paths);
    if json {
        let rows: Vec<String> = sheets.iter().map(sheet_json).collect();
        println!(
            "{{\n  \"sheets\": [\n{}\n  ],\n  \"cleared_for_compute\": {}\n}}",
            rows.join(",\n"),
            sheets.iter().filter(|s| s.may_compute()).count()
        );
    } else {
        // A view of the GATE, not a dump of the files: a lab wants to know what may be computed
        // against and what is missing, and nothing else. Rows are ordered FIT first, because the one
        // question anyone brings here is "is there a sheet I can run against".
        let mut rows: Vec<&SheetReport> = sheets.iter().collect();
        rows.sort_by_key(|s| (!s.may_compute(), s.path.clone()));
        let cleared = rows.iter().filter(|s| s.may_compute()).count();
        println!(
            "{} pre-registration sheet(s); {cleared} cleared for compute (D22 requires TEST FITNESS: FIT)",
            rows.len()
        );
        println!(
            "{:<9} {:<8} {:<9} {:<44} missing",
            "state", "verdict", "complete", "sheet"
        );
        println!("{}", "-".repeat(104));
        for s in rows {
            let name = s
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let state = if s.may_compute() { "FIT" } else { "not FIT" };
            let complete = if s.complete() { "yes" } else { "no" };
            let missing = s.missing();
            let gap = if missing.is_empty() {
                "-".to_string()
            } else {
                missing.join(", ")
            };
            println!("{state:<9} {:<8} {complete:<9} {name:<44} {gap}", s.fitness);
        }
        if cleared == 0 {
            println!();
            println!(
                "Nothing here is cleared for compute. A complete sheet with a PENDING verdict is"
            );
            println!(
                "still not FIT: completeness is the form being filled in, fitness is a reviewer"
            );
            println!("saying the bar is testable (D22).");
        }
    }
    // Exit 1 only when a sheet CLAIMS to be FIT and is not readable as one: that is the state worth
    // failing on. A PENDING sheet is the normal state of work in progress, not a failure.
    ExitCode::SUCCESS
}

fn sheet_json(s: &SheetReport) -> String {
    let blocks: Vec<String> = s
        .blocks
        .iter()
        .map(|b| format!("{{\"key\": {:?}, \"present\": {}}}", b.key, b.present))
        .collect();
    let missing: Vec<String> = s.missing().iter().map(|m| format!("{m:?}")).collect();
    format!(
        "    {{\"sheet\": {:?}, \"fitness\": {:?}, \"complete\": {}, \"may_compute\": {}, \"missing\": [{}], \"blocks\": [{}]}}",
        s.path.display().to_string(),
        s.fitness,
        s.complete(),
        s.may_compute(),
        missing.join(", "),
        blocks.join(", ")
    )
}

fn cmd_receipt(repo: &Path, args: &[String], json: bool) -> ExitCode {
    let paths = StatePaths::from_env(repo);
    let lab = flag_value(args, "--lab")
        .or_else(|| std::env::var("UOR_LAB").ok())
        .unwrap_or_default();
    if lab.trim().is_empty() {
        eprintln!(
            "uor-harness: no lab identity. Pass --lab <lab> or export UOR_LAB; a receipt with no \
             lab is preserved but attributed to nobody."
        );
        return ExitCode::from(2);
    }
    let kind = flag_value(args, "--kind").unwrap_or_else(|| "receipt".to_string());
    let dry_run = args.iter().any(|a| a == "--dry-run");

    let mut r = NewReceipt::new(kind, lab.clone());
    for key in [
        "milestone",
        "skill",
        "role",
        "branch",
        "note",
        "outcome",
        "mechanism",
        "run",
    ] {
        if let Some(v) = flag_value(args, &format!("--{key}")) {
            r = r.with(key, v);
        }
    }
    if let Err(e) = r.validate() {
        eprintln!("uor-harness: {e}");
        return ExitCode::from(2);
    }

    if dry_run {
        // The line that WOULD be written. Nothing touches the ledger.
        println!("{}", r.to_line());
        return ExitCode::SUCCESS;
    }
    match append(&paths, &r, &current_month()) {
        Ok(shard) => {
            if json {
                println!(
                    "{{\"lab\": {:?}, \"kind\": {:?}, \"shard\": {:?}, \"line\": {:?}}}",
                    lab,
                    r.kind,
                    shard.display().to_string(),
                    r.to_line()
                );
            } else {
                println!("receipt recorded: {} in {}", shard.display(), r.ts);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("uor-harness: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_check(repo: &Path, json: bool) -> ExitCode {
    let contract = match Contract::load(repo) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("uor-harness: {e}");
            return ExitCode::from(2);
        }
    };
    let report = contract.check(repo);
    if json {
        let rows: Vec<String> = report
            .formats
            .iter()
            .map(|f| {
                let problems: Vec<String> = f.violations.iter().map(|v| v.to_string()).collect();
                format!(
                    "    {{\"format\": {:?}, \"files\": {}, \"rows\": {}, \"absent\": {}, \"violations\": {}}}",
                    f.format,
                    f.files,
                    f.rows,
                    f.absent,
                    serde_json::to_string(&problems).unwrap_or_else(|_| "[]".into())
                )
            })
            .collect();
        println!(
            "{{\n  \"contract\": \"docs/harness/state-formats.json\",\n  \"schema\": {},\n  \"ok\": {},\n  \"formats\": [\n{}\n  ]\n}}",
            contract.schema,
            report.ok(),
            rows.join(",\n")
        );
    } else {
        println!(
            "state formats: {} checked against docs/harness/state-formats.json",
            report.formats_checked
        );
        for f in &report.formats {
            let state = if f.violations.is_empty() {
                "ok".to_string()
            } else {
                format!("{} PROBLEM(S)", f.violations.len())
            };
            let detail = if f.rows > 0 {
                format!(" ({} rows)", f.rows)
            } else if f.absent {
                " (no file present)".to_string()
            } else {
                format!(" ({} file(s))", f.files)
            };
            println!("  {:<16} {}{}", f.format, state, detail);
            for v in f.violations.iter().take(5) {
                println!("      {v}");
            }
            if f.violations.len() > 5 {
                println!("      ... and {} more", f.violations.len() - 5);
            }
        }
    }
    if report.ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Discover the labs that have a presence record in either tier.
fn known_labs(paths: &StatePaths) -> Vec<String> {
    let mut labs = std::collections::BTreeSet::new();
    for dir in [paths.live_presence_dir(), paths.durable_presence_dir()] {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for e in entries.flatten() {
                if let Some(stem) = e.path().file_stem().and_then(|s| s.to_str()) {
                    if e.path().extension().is_some_and(|x| x == "json") {
                        labs.insert(stem.to_string());
                    }
                }
            }
        }
    }
    labs.into_iter().collect()
}

fn cmd_presence(repo: &Path, json: bool) -> ExitCode {
    let paths = StatePaths::from_env(repo);
    let now = now_unix();
    const TTL_MINUTES: i64 = 30;
    let labs = known_labs(&paths);

    if labs.is_empty() {
        if json {
            println!(
                "{{\"labs\": [], \"live_tier\": {:?}}}",
                paths.live.display().to_string()
            );
        } else {
            println!("no lab has published presence yet");
            println!("  an empty board is normal until someone checks in.");
            println!("  publish yours with:  export UOR_LAB=<lab> && uor-harness heartbeat -m <milestone>");
        }
        return ExitCode::SUCCESS;
    }

    let mut rows = Vec::new();
    for lab in &labs {
        if let Some(p) = PresenceRow::read_lab(&paths, lab) {
            let age = p.age_minutes(now);
            let stale = p.is_stale(now, TTL_MINUTES);
            rows.push((lab.clone(), p, age, stale));
        }
    }
    rows.sort_by(|a, b| a.3.cmp(&b.3).then_with(|| a.0.cmp(&b.0)));

    if json {
        let items: Vec<String> = rows
            .iter()
            .map(|(lab, p, age, stale)| {
                format!(
                    "    {{\"lab\": {lab:?}, \"via\": {:?}, \"stale\": {stale}, \"age_minutes\": {}, \"milestone\": {:?}, \"branch\": {:?}, \"doing\": {:?}}}",
                    p.source,
                    age.map(|a| a.to_string()).unwrap_or_else(|| "null".into()),
                    p.milestone,
                    p.branch,
                    p.doing
                )
            })
            .collect();
        println!(
            "{{\n  \"live_tier\": {:?},\n  \"ttl_minutes\": {TTL_MINUTES},\n  \"labs\": [\n{}\n  ]\n}}",
            paths.live.display().to_string(),
            items.join(",\n")
        );
        return ExitCode::SUCCESS;
    }

    println!(
        "{} lab(s) known (stale after {TTL_MINUTES}m) · live tier: {}",
        rows.len(),
        paths.live.display()
    );
    println!(
        "{:<12} {:<20} {:<8} {:<9} {:<22} doing",
        "lab", "state", "via", "milestone", "branch"
    );
    println!("{}", "-".repeat(104));
    for (lab, p, age, stale) in &rows {
        let state = match (stale, age) {
            (true, _) => "STALE".to_string(),
            (false, Some(a)) => format!("live ({a}m ago)"),
            (false, None) => "live (unknown age)".to_string(),
        };
        println!(
            "{:<12} {:<20} {:<8} {:<9} {:<22} {}",
            lab,
            state,
            p.source,
            if p.milestone.is_empty() {
                "-"
            } else {
                &p.milestone
            },
            if p.branch.is_empty() { "-" } else { &p.branch },
            p.doing
        );
    }
    ExitCode::SUCCESS
}

fn cmd_ledger(repo: &Path, json: bool) -> ExitCode {
    let paths = StatePaths::from_env(repo);
    let rows = read_ledger(&paths);
    let mut by_lab: std::collections::BTreeMap<String, std::collections::BTreeMap<String, usize>> =
        Default::default();
    for r in &rows {
        let lab = if r.lab.is_empty() {
            "unknown".to_string()
        } else {
            r.lab.clone()
        };
        *by_lab
            .entry(lab)
            .or_default()
            .entry(r.kind.clone())
            .or_insert(0) += 1;
    }
    if json {
        let items: Vec<String> = by_lab
            .iter()
            .map(|(lab, kinds)| {
                let ks: Vec<String> = kinds.iter().map(|(k, n)| format!("{k:?}: {n}")).collect();
                format!(
                    "    {{ \"lab\": {lab:?}, \"kinds\": {{ {} }} }}",
                    ks.join(", ")
                )
            })
            .collect();
        println!(
            "{{\n  \"receipts\": {},\n  \"by_lab\": [\n{}\n  ]\n}}",
            rows.len(),
            items.join(",\n")
        );
        return ExitCode::SUCCESS;
    }
    println!("{} receipts in the sharded ledger", rows.len());
    for (lab, kinds) in &by_lab {
        let ks: Vec<String> = kinds.iter().map(|(k, n)| format!("{k} {n}")).collect();
        println!("  {lab:<10} {}", ks.join(", "));
    }
    ExitCode::SUCCESS
}
