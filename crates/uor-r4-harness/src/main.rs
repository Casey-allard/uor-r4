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

use uor_r4_harness::state::{read_ledger, Presence, StatePaths};
use uor_r4_harness::Contract;

const USAGE: &str = "\
uor-harness - the multi-lab research harness (Rust side of the port)

USAGE:
    uor-harness <COMMAND> [--repo DIR]

COMMANDS:
    check       Check persisted state against the frozen contract
                (docs/harness/state-formats.json). Exits 1 on drift.
    presence    What each lab is doing now. Live tier first, durable as the
                fallback, reporting which answered.
    ledger      Receipt counts by lab and kind.
    help        This text.

The Python harness owns behaviour until this path passes the same checks; the
formats are frozen so the two agree about what the shared record says.
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut command: Option<String> = None;
    let mut repo: Option<PathBuf> = None;
    let mut json = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--repo" => {
                i += 1;
                match args.get(i) {
                    Some(p) => repo = Some(PathBuf::from(p)),
                    None => {
                        eprintln!("uor-harness: --repo needs a directory");
                        return ExitCode::from(2);
                    }
                }
            }
            "--json" => json = true,
            "-h" | "--help" | "help" => {
                print!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other if other.starts_with('-') => {
                eprintln!("uor-harness: unknown option {other}\n\n{USAGE}");
                return ExitCode::from(2);
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
        Some("check") => cmd_check(&repo, json),
        Some("presence") => cmd_presence(&repo, json),
        Some("ledger") => cmd_ledger(&repo, json),
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
        if let Some(p) = Presence::read_lab(&paths, lab) {
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
