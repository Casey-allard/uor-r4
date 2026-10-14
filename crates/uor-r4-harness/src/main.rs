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

use uor_r4_harness::briefing;
use uor_r4_harness::githelp;
use uor_r4_harness::mistakes;
use uor_r4_harness::multilab;
use uor_r4_harness::notes;
use uor_r4_harness::onboard::{detect_runtime, onboard};
use uor_r4_harness::prep;
use uor_r4_harness::prep::{check_all, SheetReport};
use uor_r4_harness::presence;
use uor_r4_harness::presence::{heartbeat, Presence};
use uor_r4_harness::receipt::{
    append, append_live, current_month, sync_live_to_committed, Receipt as NewReceipt,
};
use uor_r4_harness::registries::Registries;
use uor_r4_harness::shards;
use uor_r4_harness::skills;
use uor_r4_harness::state::{read_ledger, PresenceRow, StatePaths};
use uor_r4_harness::status;
use uor_r4_harness::team;
use uor_r4_harness::Contract;

/// Flags that only a subcommand understands, forwarded rather than rejected at the top level.
/// Flags a PORTED command shares with its Python original, forwarded rather than interpreted.
///
/// `check` is delegated (57 rules), so `check --only <rule>` must reach Python intact. Without this
/// the flag was forwarded alone and its value became the next command word.
const PYTHON_PASSTHROUGH_FLAGS: [&str; 1] = ["--only"];
const PYTHON_PASSTHROUGH_VALUE_FLAGS: [&str; 1] = ["--only"];

/// Commands this binary implements. Anything else is delegated to the Python harness verbatim.
///
/// Declared here rather than inferred from the dispatch below, because the decision must be made
/// BEFORE flag parsing: an unported command's flags are unknown to this binary, and rejecting one
/// meant delegation only worked for commands whose flags happened to be in the lists - `check --only`
/// failed with "unknown option --only" before it could delegate. Which defeats the point.
use uor_r4_harness::ruleinventory::PORTED_COMMANDS;

const SUBCOMMAND_FLAGS: [&str; 33] = [
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
    "--line",
    "--next",
    "--date",
    "--mechanism",
    "--bar",
    "--force",
    "--base",
    "--split",
    "--steps",
    "--stop",
    "--cost",
    "--seeds",
    "--rows",
    "--cached",
    "--mark-seen",
    "--no-notes",
    "--update-baseline",
    "--list",
    "--quiet",
    "--apply",
    // `check --staged` selects the staged rule SET. It was missing from this list, so the top-level
    // parser rejected it as an unknown option before `check` could see it.
    "--staged",
];

/// The subset that takes a value, so the value is not mistaken for a command.
const SUBCOMMAND_VALUE_FLAGS: [&str; 50] = [
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
    "--date",
    "--mechanism",
    "--bar",
    "--base",
    "--split",
    "--steps",
    "--stop",
    "--cost",
    "--seeds",
    "--rows",
    "--at",
    "--count",
    "--headline",
    "--line",
    "-f",
    "--files",
    "--actor",
    "--verdict",
    "--reviewer",
    "--sheet",
    "--body-file",
    "--body",
    "--action",
    "--task",
    "--state",
    "--scope",
    // `mistakes capture`. The four `nargs="*"` flags take several values; only the first is
    // consumed here and `flag_all_values` collects the rest from the argument list.
    "--id",
    "--title",
    "--severity",
    "--class",
    "--trigger",
    "--paths",
    "--milestones",
    "--guard",
    "--evidence",
    "--symptom",
    "--lesson",
    // `heartbeat --session`: which session of a lab is checking in.
    "--session",
];

/// Per-command help, for the commands whose flags are not guessable from the global usage.
///
/// **`heartbeat --help` used to print the global usage**, which lists the command and none of its
/// flags: a lab had to read `main.rs` to discover `--line`/`--doing`/`--count`/`--next`/`--session`,
/// on the one command every lab runs every few minutes. Commands not listed here still print the
/// global usage, which is honest about how much is written down.
fn command_help(command: &str) -> Option<&'static str> {
    match command {
        "heartbeat" => Some(HEARTBEAT_HELP),
        _ => None,
    }
}

const HEARTBEAT_HELP: &str = "\
uor-harness heartbeat - the required check-in (at least every 5 minutes)

USAGE:
    uor-harness heartbeat -m <milestone> [flags]

FLAGS:
    -m, --milestone M   the milestone this work belongs to (m1..m8)
        --line L        the session contract's line (e.g. \"no model line - tooling\")
        --doing D       the older spelling of --line; both are accepted
        --count C       the D21 count, e.g. 0/3
        --headline H    the D21 headline, e.g. \"37/232 -> 37/232\"
        --next N        what happens next
        --session S     which session of this lab is checking in
        --lab L         the lab identity (default: $UOR_LAB)
        --dry-run       print the record instead of publishing it
        --no-notes      do not read the unread-notes cache

WHAT IT WRITES:
    The live presence tier, immediately visible to every lab on this machine, and one
    heartbeat receipt in the live ledger shard. Both come from ONE record, so they cannot
    disagree about what this lab is doing.

    A flag that is ABSENT carries the previous check-in's value forward; a flag passed with
    an empty value clears the field. `--line` and `--next` are not required, and leaving them
    out no longer blanks them.

    The committed presence tier is NOT written here: R10 keeps a check-in from dirtying a
    checkout. `uor-harness multilab sync-live` copies the live tier into it deliberately.

See docs/harness/README.md and docs/labs/session-goal.md.
";

const USAGE: &str = "\
uor-harness - the multi-lab research harness (Rust side of the port)

USAGE:
    uor-harness <COMMAND> [--repo DIR]

COMMANDS:
    onboard     Verify this machine can use the harness, and say what to do
                next. Read-only: records nothing, claims nothing. Exits 1 if
                anything FAILs.
    check       RUNS THE FULL RULE SET (delegated to Python until every rule
                is ported). 57 rules; exits 1 on any error.
    check-formats  Check only the state formats against the frozen contract
                (docs/harness/state-formats.json). One rule of the 57 - use
                `check` unless you specifically want this one.
    presence    What each lab is doing now. Live tier first, durable as the
                fallback, reporting which answered.
    ledger      Receipt counts by lab and kind.
    receipt     Append one receipt to this lab's shard. The first WRITE on the
                Rust side; use --dry-run to see the line without recording it.
    heartbeat   Check in: publish presence and append a receipt, both to the LIVE
                tier, so a check-in never dirties a working checkout (R10).
    git         Branch hygiene: clean-merged lists local branches already
                merged into origin/main, --apply deletes the proven ones.
    multilab    The shared board: presence, claims, touch. `touch` refreshes
                liveness for hooks WITHOUT changing what the lab is doing.
    mistakes    Lessons from what went wrong here before: list, find -f <files>.
                Run `find` before editing - a lesson that does not surface at
                the moment someone repeats it is a lesson nobody reads.
    skill       Skills: list, show, match, use. `use` records a receipt line
                saying a skill was actually loaded and deployed.
    team        Branch claims: claim, release, claims, roster.
                `team claim --branch B --lab L` before you start work.
    version     The commit this binary was built from, and whether it is
                behind origin/main - answers why a flag was rejected.
    notes       Unread notes: comments on the milestone issues this lab has
                not seen. Refreshes from GitHub by default; --cached skips the
                network. A marker per lab lives in the live tier (R10-clean).
    ledger-sync Append this lab's live receipts into the committed shard. Run
                before opening a PR so the record reaches review. Append-only.
                Use --dry-run to see what would be written.
    prep        The D22 gate: `new` writes a pre-registration sheet, `fit`
                records a TEST FITNESS review, `list` shows the sheets. Get a
                sheet to FIT on the milestone before any compute.
    prep-check  Read every pre-registration sheet: its TEST FITNESS verdict and
                which required blocks are missing. The read half of the D22 gate.
    registry    The milestone registry as this implementation reads it: headline,
                panel and line counts per milestone. Compared against Python by
                parity.py, because the headline is the number a run is scored on.
    prep-render Render a pre-registration sheet to stdout without writing it. The
                WRITE half of the D22 gate, checked byte-for-byte against Python.
    brief       The context pack for a task. Section 1 (state) and the owner rules
                are ported; the rest is listed as unported rather than omitted.
    shard-check Refuse a ledger shard that lost lines. A shard may grow and
                may never shrink; exits 1 on a lost receipt. Compares the
                working tree against --base (default origin/main).
    status      What is true now. Sections 1 (milestones) and 2 (receipts) are
                ported; section 3 (delivery) says so rather than rendering blank.
    ratchet     d23|d25   (or the banned word each one counts) [--update-baseline] [--list] [--quiet]
                The D23/D25 ratchets (Rust port of the scripts/check_no_*.py pair,
                same baselines). Exits 1 when a count grew; --update-baseline
                may only lower counts.
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
                // **`heartbeat --help` printed the global usage**, so a lab could not learn that
                // `--line`, `--doing`, `--count`, `--next` or `--session` exist and had to read
                // this file to find out (claude/review1, #2028 6163960613 item 3). A command word
                // seen BEFORE the help flag selects its own flag list.
                if let Some(cmd) = command.as_deref() {
                    if let Some(block) = command_help(cmd) {
                        print!("{block}");
                        return ExitCode::SUCCESS;
                    }
                }
                print!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            // Flags belonging to a subcommand are collected and handed to it, not rejected here:
            // `receipt` takes many optional fields and the top-level loop cannot know them all.
            other if other.starts_with('-') => {
                // A flag is known if it takes a value OR is a bare switch. Checking only the switch
                // list silently rejected every value-only flag: `--at` was added to the value list
                // and still failed with "unknown option", because the two lists were consulted as if
                // the first were the complete set.
                let takes_value = SUBCOMMAND_VALUE_FLAGS.contains(&other);
                if SUBCOMMAND_FLAGS.contains(&other) || takes_value {
                    rest.push(other.to_string());
                    // A value-taking subcommand flag carries the next token with it.
                    if !other.contains('=') && takes_value {
                        if let Some(v) = args.get(i + 1) {
                            rest.push(v.clone());
                            i += 1;
                        }
                    }
                } else if command
                    .as_deref()
                    .is_none_or(|c| !PORTED_COMMANDS.contains(&c))
                    || PYTHON_PASSTHROUGH_FLAGS.contains(&other)
                {
                    // An unported command's flags are the PYTHON parser's business, and so are the
                    // few flags a ported command shares with its Python original. Forward the token
                    // AND ITS VALUE: forwarding only the flag left the value to be read as the next
                    // command word, so `check --only harness.state_formats` delegated the command
                    // `harness.state_formats`.
                    rest.push(other.to_string());
                    if PYTHON_PASSTHROUGH_VALUE_FLAGS.contains(&other) {
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
            // The FIRST non-flag word is the command; later ones are its arguments.
            //
            // This used to overwrite unconditionally, so a subcommand group lost its head:
            // `uor-harness multilab touch` set the command to "multilab" and then to "touch", and
            // delegated a bare `touch` - which Python rejects as `invalid choice: 'touch'`. The group
            // word is part of the invocation and has to survive to the delegate.
            other => match command {
                Some(_) => rest.push(other.to_string()),
                None => command = Some(other.to_string()),
            },
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
        // NOT `check`. The Rust checker validates state formats - ONE rule of the 57 the Python
        // `check` runs. Exposing it as `check` would mean a lab runs `uor-harness check`, sees
        // "state formats ok", and believes it has validated its work while 56 rules never ran.
        // That is the worst possible outcome for the command labs trust most, so the full check
        // stays delegated and the Rust one is explicit: `check-formats`.
        // **`check` and `check-formats` are separate commands**, not one with a flag.
        //
        // The earlier note here said exposing the format-contract-only checker as `check` "would
        // mean a lab runs `uor-harness check`, sees 'state formats ok', and believes it has
        // validated its work while 56 rules never ran." **That was a correct prediction** - it is
        // what I found by verifying. `cmd_check_rules` is what answers it.
        //
        // **One boolean through one function was tried and reverted**: it made `check-formats` load
        // the registries on a bare repository and return 2, which `parity`'s controlled one-row
        // cases caught as `said ok=false, expected True`. **Two commands with two bodies cannot
        // drift that way; a flag can.**
        Some("check-formats") => cmd_check(&repo, json),
        // **`check`'s own flags are parsed, not passed through as a filter.** The argument list
        // used to BE the `--only` prefix set, so `--repo <path>` - which the top-level parser
        // forwards here on purpose - narrowed the run to nothing and printed `0 rules, 0 failed`.
        Some("check") => match uor_r4_harness::checkdispatch::parse_args(&rest) {
            Ok(parsed) => cmd_check_rules(&repo, &parsed, json),
            // argparse exits 2 on an argument it does not know, and so does this: an unknown
            // token must never be read as a rule prefix that happens to match nothing.
            Err(e) => {
                eprintln!("uor-harness: {e}");
                ExitCode::from(2)
            }
        },
        Some("presence") => cmd_presence(&repo, json),
        Some("ledger") => cmd_ledger(&repo, json),
        Some("receipt") => cmd_receipt(&repo, &rest, json),
        Some("heartbeat") => cmd_heartbeat(&repo, &rest, json),
        Some("ledger-sync") => cmd_ledger_sync(&repo, &rest, json),
        Some("notes") => cmd_notes(&repo, &rest, json),
        Some("version") => cmd_version(&repo),
        Some("team") => cmd_team(&repo, &rest, json),
        Some("skill") => cmd_skill(&repo, &rest, json),
        Some("mistakes") => cmd_mistakes(&repo, &rest, json),
        Some("multilab") => cmd_multilab(&repo, &rest, json),
        Some("git") => cmd_git(&repo, &rest, json),
        Some("prep") => cmd_prep(&repo, &rest, json),
        Some("prep-check") => cmd_prep_check(&repo, json),
        Some("registry") => cmd_registry(&repo, json),
        Some("prep-render") => cmd_prep_render(&repo, &rest),
        Some("brief") => cmd_brief(&repo, &rest),
        Some("shard-check") => cmd_shard_check(&repo, &rest, json),
        Some("status") => cmd_status(&repo, &rest),
        Some("ratchet") => cmd_ratchet(&repo, &rest),
        Some("lint") => cmd_lint(&repo, &rest, json),
        Some("check-rules") => {
            // The port's coverage record. `parity.py` compares this against Python's registered
            // rules, because `check`'s rule set changes while the port is in progress and a module
            // that matched once can fall behind silently.
            if json {
                print!("{}", uor_r4_harness::ruleinventory::to_json());
            } else {
                println!(
                    "check rules implemented in Rust ({} total):",
                    uor_r4_harness::ruleinventory::all_ids().len()
                );
                for (m, n) in uor_r4_harness::ruleinventory::counts() {
                    println!("  {m:<12} {n}");
                }
            }
            ExitCode::SUCCESS
        }
        // A command this binary has not ported yet runs the Python implementation BENEATH this
        // entry point, and says so on every call.
        //
        // This is the half of the port that was missing: `cli.py` was still the front door, so four
        // labs typed `python3` forty times a day and could not tell whether they were using the Rust
        // harness or not. Now `uor-harness` is the interface and Python is an implementation detail
        // that announces itself - the same honesty `brief` uses for its unported sections.
        //
        // The notice goes to stderr and is not suppressible, deliberately: a lab should know when it
        // is still on the old path, and a silent delegation would make the remaining work invisible
        // again - which is exactly how it stayed unfinished.
        Some(other) => delegate_to_python(&repo, other, &rest),

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

/// The first bare word in `args` that is not the value of a value-taking flag.
///
/// Shared by every grouped command (`team`, `multilab`), because each hit the same two bugs: taking
/// `args[0]` picked up a forwarded `--repo`, and taking "the first word without a dash" picked up the
/// repo PATH. The value list is the only thing that can tell a flag's argument from a subcommand.
fn action_word(args: &[String]) -> &str {
    let mut skip_next = false;
    for a in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if a.starts_with('-') {
            // A value-taking flag consumes the next token even when written `--flag value`.
            //
            // GLOBAL value-taking flags are consumed by the top-level parser and pushed into `rest`
            // for the subcommand's benefit, so they never had to appear in SUBCOMMAND_VALUE_FLAGS -
            // and their VALUES then became candidate actions. `--repo <dir>` produced
            // `unknown action "<path>"` and `--lab <name>` produced `unknown action "deepseek"`.
            // Every global value-taking flag belongs in this list, and a new one added to the parser
            // must be added here too.
            const GLOBAL_VALUE_FLAGS: [&str; 2] = ["--repo", "--lab"];
            let takes_value = GLOBAL_VALUE_FLAGS.contains(&a.as_str())
                || (SUBCOMMAND_VALUE_FLAGS.contains(&a.as_str()) && !a.contains('='));
            skip_next = takes_value && !a.contains('=');
            continue;
        }
        return a;
    }
    ""
}

/// `mistakes <action>` - lessons from what went wrong here before (D30 port).
///
/// `list` and `find` are Rust. `show` and `capture` still delegate by name: `capture` writes a
/// **lesson file plus a guard**, which is a bigger surface than reading one and belongs in its own
/// unit rather than bolted on here.
fn cmd_mistakes(repo: &Path, args: &[String], json: bool) -> ExitCode {
    let paths = StatePaths::from_env(repo);
    let action = action_word(args);
    let ms = mistakes::load(&paths);

    match action {
        "list" => {
            if json {
                let rows: Vec<String> = ms
                    .iter()
                    .map(|m| {
                        format!(
                            "    {{\"id\": {:?}, \"title\": {:?}, \"severity\": {:?}, \"status\": {:?}, \"guard\": {:?}}}",
                            m.id, m.title, m.severity, m.status, m.guard
                        )
                    })
                    .collect();
                println!("{{\n  \"mistakes\": [\n{}\n  ]\n}}", rows.join(",\n"));
            } else {
                println!("{} lessons in docs/harness/mistakes/\n", ms.len());
                for m in &ms {
                    println!("  {:<12} {:<52} {}", m.severity, m.id, m.title);
                }
            }
            ExitCode::SUCCESS
        }
        "find" => {
            let files = flag_values(args, "--files", "-f");
            let query = flag_value(args, "--query").unwrap_or_default();
            let milestone = flag_value(args, "--milestone").unwrap_or_default();
            let milestones: Vec<String> = if milestone.is_empty() {
                Vec::new()
            } else {
                vec![milestone]
            };
            let hits = mistakes::match_lessons(&ms, &files, &query, &milestones);

            if json {
                let rows: Vec<String> = hits
                    .iter()
                    .map(|h| {
                        format!(
                            "    {{\"id\": {:?}, \"title\": {:?}, \"severity\": {:?}, \"class\": {:?}, \"guard\": {:?}, \"why\": {:?}, \"path\": {:?}}}",
                            h.mistake.id, h.mistake.title, h.mistake.severity, h.mistake.class,
                            h.mistake.guard, h.why, h.mistake.path.display().to_string()
                        )
                    })
                    .collect();
                println!("{{\n  \"matches\": [\n{}\n  ]\n}}", rows.join(",\n"));
            } else if hits.is_empty() {
                println!("no lesson applies to that");
            } else {
                println!("{} lesson(s) apply:\n", hits.len());
                for h in &hits {
                    // The `why` names WHICH rule fired. A lesson surfaced for the wrong reason
                    // teaches the wrong thing, so it is printed rather than inferred.
                    println!(
                        "  [{}] {} - {}",
                        h.mistake.severity, h.mistake.id, h.mistake.title
                    );
                    println!("      why: {}", h.why);
                    if !h.mistake.guard.is_empty() {
                        println!("      guard: {}", h.mistake.guard);
                    }
                }
            }
            ExitCode::SUCCESS
        }
        // `show` reads one lesson body; `capture` writes one. **`capture` is the writer, and the
        // reason this pair stayed delegated: it must never write a file that
        // `mistakes.registry_valid` would reject**, so the draft is validated before the write and
        // the written file is read back and removed if it somehow fails.
        "show" => {
            let key = action_word_after(args, &["show"]);
            let lessons = mistakes::load(&paths);
            let found = if key.is_empty() {
                None
            } else {
                mistakes::find_mistake(&lessons, &key)
            };
            // `args.id` is `None` when absent, and the message renders it - so a missing id says
            // `unknown mistake None` rather than naming an argument, exactly as Python does.
            let Some(m) = found else {
                let shown = if key.is_empty() {
                    "None".to_string()
                } else {
                    uor_r4_harness::prgates::py_repr_public(&key)
                };
                eprintln!("unknown mistake {shown}");
                return ExitCode::from(2);
            };
            match std::fs::read_to_string(&m.path) {
                Ok(text) => {
                    // Python writes through `emit`, which adds the newline only when it is missing.
                    if text.ends_with('\n') {
                        print!("{text}");
                    } else {
                        println!("{text}");
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("uor-harness mistakes show: {e}");
                    ExitCode::from(1)
                }
            }
        }
        "capture" => {
            let id = flag_value(args, "--id")
                .unwrap_or_default()
                .trim()
                .to_string();
            let title = flag_value(args, "--title")
                .unwrap_or_default()
                .trim()
                .to_string();
            // argparse also requires both, so this is reachable only through the Rust parser,
            // which is lenient; the refusal and the exit code are Python's either way.
            if id.is_empty() || title.is_empty() {
                eprintln!("mistakes capture needs --id and --title");
                return ExitCode::from(2);
            }
            if !mistakes::kebab_case_id(&id) {
                eprintln!(
                    "--id {} must be kebab-case (a-z, 0-9, '-')",
                    uor_r4_harness::prgates::py_repr_public(&id)
                );
                return ExitCode::from(2);
            }
            let severity = flag_value(args, "--severity").unwrap_or_else(|| "warn".to_string());
            let class = flag_value(args, "--class").unwrap_or_else(|| "process".to_string());
            if !mistakes::SEVERITIES.contains(&severity.as_str()) {
                eprintln!(
                    "--severity must be one of {}",
                    mistakes::py_tuple(&mistakes::SEVERITIES)
                );
                return ExitCode::from(2);
            }
            if !mistakes::CLASSES.contains(&class.as_str()) {
                eprintln!(
                    "--class must be one of {}",
                    mistakes::py_tuple(&mistakes::CLASSES)
                );
                return ExitCode::from(2);
            }
            let target = mistakes::dir(&paths).join(format!("{id}.md"));
            if target.exists() && !args.iter().any(|a| a == "--force") {
                eprintln!("{} exists; pass --force to overwrite it", target.display());
                return ExitCode::from(1);
            }
            let draft = mistakes::LessonDraft {
                id: id.clone(),
                title,
                severity: severity.clone(),
                class: class.clone(),
                trigger: flag_all_values(args, "--trigger"),
                paths: flag_all_values(args, "--paths"),
                milestones: flag_all_values(args, "--milestones"),
                mechanism: flag_value(args, "--mechanism").unwrap_or_default(),
                guard: flag_value(args, "--guard").unwrap_or_default(),
                evidence: flag_all_values(args, "--evidence"),
                symptom: flag_value(args, "--symptom").unwrap_or_default(),
                lesson: flag_value(args, "--lesson").unwrap_or_default(),
                ..Default::default()
            };
            let text = uor_r4_harness::prgates::redact_public(&mistakes::build_lesson_text(&draft));
            let (problems, warnings) = mistakes::draft_problems(&paths, &target, &text);
            if !problems.is_empty() {
                eprintln!(
                    "refusing to write {}: the draft is not a valid lesson",
                    target.display()
                );
                for problem in &problems {
                    eprintln!("  {problem}");
                }
                return ExitCode::from(1);
            }
            for warning in &warnings {
                eprintln!("note: {warning}");
            }
            let existed = target.exists();
            if let Some(parent) = target.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(e) = std::fs::write(&target, &text) {
                eprintln!("uor-harness mistakes capture: {e}");
                return ExitCode::from(1);
            }
            // Read back what was written: a capture must not leave behind a file the registry
            // rejects, and if it somehow did, the file is removed rather than left as evidence.
            let check = mistakes::parse(&id, &text, &target);
            if let Some(first) = check.errors.first() {
                let _ = std::fs::remove_file(&target);
                eprintln!("wrote and removed {}: {first}", target.display());
                return ExitCode::from(1);
            }
            println!(
                "{} {}",
                if existed { "overwrote" } else { "wrote" },
                target.display()
            );
            // **Three more lines, and they are the command's own guidance.** They were missed on
            // the first pass because the function was read only as far as the write; the
            // comparison of stdout is what showed the difference.
            println!(
                "  severity {severity}, class {class}, guard {}",
                check.guard
            );
            println!("  read it back: python3 scripts/harness/cli.py mistakes show {id}");
            println!("  check: python3 scripts/harness/cli.py check --only mistakes");
            ExitCode::SUCCESS
        }
        "" => {
            eprintln!("uor-harness mistakes: which action? list, find, show, capture");
            ExitCode::from(2)
        }
        other => {
            eprintln!("uor-harness mistakes: unknown action {other:?}; try list, find");
            ExitCode::from(2)
        }
    }
}

/// `skill <action>` - the skill registry (D30 port of the Python `skill`).
///
/// `use` is the one labs invoke every session: it validates the id and writes **one receipt line**
/// saying the skill was loaded. **An unknown id refuses rather than recording**, because a receipt
/// naming a skill that does not exist is worse than no receipt — the record would claim a skill was
/// used.
fn cmd_skill(repo: &Path, args: &[String], json: bool) -> ExitCode {
    let paths = StatePaths::from_env(repo);
    let action = action_word(args);
    let skills = skills::load(&paths);
    let id = action_word_after(args, &["show", "use", "match"]);

    match action {
        "list" => {
            if json {
                let rows: Vec<String> = skills
                    .iter()
                    .map(|s| {
                        format!(
                            "    {{\"id\": {:?}, \"description\": {:?}, \"milestones\": {:?}, \"version\": {:?}}}",
                            s.id, s.description, s.milestones, s.version
                        )
                    })
                    .collect();
                println!("{{\n  \"skills\": [\n{}\n  ]\n}}", rows.join(",\n"));
            } else {
                println!("{} skills in docs/harness/skills/\n", skills.len());
                for s in &skills {
                    println!("  {:<26} v{:<3} {}", s.id, s.version, s.description);
                }
            }
            ExitCode::SUCCESS
        }
        "show" => match skills::find(&skills, &id) {
            Some(s) => {
                if json {
                    let body = serde_json::json!({
                        "id": s.id, "name": s.name, "description": s.description,
                        "when": s.when, "applies_to": s.applies_to, "requires": s.requires,
                        "outputs": s.outputs, "forbidden": s.forbidden,
                        "milestones": s.milestones, "path": s.path.display().to_string(),
                    });
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&body).unwrap_or_default()
                    );
                } else {
                    println!("{}  ({})", s.id, s.path.display());
                    println!("  {}", s.description);
                    if !s.when.is_empty() {
                        println!("  when: {}", s.when.join(", "));
                    }
                    if !s.applies_to.is_empty() {
                        println!("  applies to: {}", s.applies_to.join(", "));
                    }
                    if !s.forbidden.is_empty() {
                        println!("  forbidden: {}", s.forbidden.join(", "));
                    }
                }
                ExitCode::SUCCESS
            }
            None => {
                eprintln!("unknown skill {id:?}");
                ExitCode::from(2)
            }
        },
        "match" => {
            let files = flag_values(args, "--files", "-f");
            let hits = skills::matching(&skills, &files);
            if json {
                let rows: Vec<String> = hits
                    .iter()
                    .map(|s| format!("    {{\"id\": {:?}}}", s.id))
                    .collect();
                println!("{{\n  \"matches\": [\n{}\n  ]\n}}", rows.join(",\n"));
            } else if hits.is_empty() {
                println!("no skill applies to those files");
            } else {
                for s in &hits {
                    println!("  {:<26} {}", s.id, s.description);
                }
            }
            ExitCode::SUCCESS
        }
        "use" => {
            // Validate BEFORE writing. A receipt for a skill that does not exist would record that
            // something was loaded when nothing was.
            let Some(skill) = skills::find(&skills, &id) else {
                eprintln!("unknown skill {id:?}");
                eprintln!("  list them with: uor-harness skill list");
                return ExitCode::from(2);
            };
            let lab = flag_value(args, "--lab")
                .or_else(|| std::env::var("UOR_LAB").ok())
                .unwrap_or_default();
            if lab.trim().is_empty() {
                // The Python harness warns loudly rather than refusing, and the receipt is still
                // written - an unattributed record is worse than an absent one, but refusing here
                // would lose the record entirely.
                eprintln!(
                    "uor-harness: no lab identity. Pass --lab <lab> or export UOR_LAB; the receipt \
                     is recorded but attributed to nobody, and no other lab can see who did the work."
                );
            }
            let mut r = NewReceipt::new("skill", lab.clone());
            r = r.with("skill", skill.id.clone());
            for key in ["milestone", "actor", "outcome", "note"] {
                if let Some(v) = flag_value(args, &format!("--{key}")) {
                    r = r.with(key, v);
                } else if key == "outcome" {
                    r = r.with("outcome", "started");
                }
            }
            if let Err(e) = r.validate() {
                eprintln!("uor-harness: {e}");
                return ExitCode::from(2);
            }
            match append(&paths, &r, &current_month()) {
                Ok(shard) => {
                    if json {
                        println!(
                            "{{\"skill\": {:?}, \"lab\": {:?}, \"shard\": {:?}}}",
                            skill.id,
                            lab,
                            shard.display().to_string()
                        );
                    } else {
                        println!("skill used: {} ({})", skill.id, shard.display());
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("uor-harness: {e}");
                    ExitCode::from(1)
                }
            }
        }
        "" => {
            eprintln!("uor-harness skill: which action? list, show, match, use");
            ExitCode::from(2)
        }
        other => {
            eprintln!("uor-harness skill: unknown action {other:?}; try list, show, match, use");
            ExitCode::from(2)
        }
    }
}

/// The first bare word AFTER `markers` in `args` - the positional argument of a grouped command.
///
/// `action_word` returns the action; this finds what follows it. Both step over value-taking flags,
/// because a repo path or lab name is a bare word too and was mistaken for the action three times.
fn action_word_after(args: &[String], markers: &[&str]) -> String {
    let mut seen = false;
    let mut skip_next = false;
    for a in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if a.starts_with('-') {
            const GLOBAL_VALUE_FLAGS: [&str; 2] = ["--repo", "--lab"];
            skip_next = GLOBAL_VALUE_FLAGS.contains(&a.as_str())
                || (SUBCOMMAND_VALUE_FLAGS.contains(&a.as_str()) && !a.contains('='));
            continue;
        }
        if !seen {
            if markers.contains(&a.as_str()) {
                seen = true;
            }
            continue;
        }
        return a.clone();
    }
    String::new()
}

/// Every value of a flag that takes several (`--scope a b c`), stopping at the next flag.
///
/// `flag_values` cannot be used for this: it pairs a flag with exactly one value, and Python's
/// `--scope` is `nargs="*"`. The top-level parser consumes the first value and the rest arrive as
/// bare words, so they have to be collected here rather than lost.
fn flag_all_values(args: &[String], name: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == name {
            let mut j = i + 1;
            while let Some(v) = args.get(j) {
                if v.starts_with('-') {
                    break;
                }
                out.push(v.clone());
                j += 1;
            }
            i = j;
            continue;
        }
        i += 1;
    }
    out
}

/// Every value given to a repeatable flag.
fn flag_values(args: &[String], long: &str, short: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == long || a == short {
            if let Some(v) = args.get(i + 1) {
                // A comma-separated list is accepted too, because that is how a lab writes several
                // files on one line.
                for part in v.split(',') {
                    let t = part.trim();
                    if !t.is_empty() {
                        out.push(t.to_string());
                    }
                }
            }
            i += 2;
            continue;
        }
        i += 1;
    }
    out
}

/// `prep <action>` - the D22 pre-training gate (D30 port of the Python `prep`).
///
/// `new` writes the sheet, `fit` records the review that clears compute, `list` shows what exists.
/// **This is the gate that decides whether a run may start**, so the port keeps the rule it must:
/// **a FIT verdict clears compute for the configuration in that sheet ONLY** (D22).
fn cmd_prep(repo: &Path, args: &[String], json: bool) -> ExitCode {
    let paths = StatePaths::from_env(repo);
    let action = action_word(args);
    let regs = match Registries::load(&paths) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("uor-harness: {e}");
            return ExitCode::from(2);
        }
    };
    let key = flag_value(args, "--milestone")
        .or_else(|| flag_value(args, "-m"))
        .unwrap_or_default();

    match action {
        "list" => {
            let sheets = prep::check_all(&paths);
            if json {
                let rows: Vec<String> = sheets
                    .iter()
                    .map(|r| {
                        format!(
                            "    {{\"path\": {:?}, \"fitness\": {:?}}}",
                            r.path.display().to_string(),
                            r.fitness
                        )
                    })
                    .collect();
                println!("{{\n  \"sheets\": [\n{}\n  ]\n}}", rows.join(",\n"));
            } else if sheets.is_empty() {
                println!("no pre-registration sheets yet");
                println!("  write one with: uor-harness prep new -m <milestone>");
            } else {
                println!("{} pre-registration sheet(s)\n", sheets.len());
                for r in &sheets {
                    let rel = r.path.strip_prefix(&paths.repo).unwrap_or(&r.path);
                    println!("  {:<22} {}", r.fitness, rel.display());
                }
            }
            ExitCode::SUCCESS
        }
        "fit" => {
            // The verdict names the state that clears compute, so an unknown one is refused rather
            // than recorded - a receipt saying "FIT-ish" would clear a run on a typo.
            let verdict = flag_value(args, "--verdict")
                .unwrap_or_default()
                .to_uppercase();
            if !["FIT", "NOT_FIT", "PENDING", "ABSENT"].contains(&verdict.as_str()) {
                eprintln!(
                    "uor-harness prep fit: verdict must be one of FIT, NOT_FIT, PENDING, ABSENT (got {:?})",
                    verdict
                );
                return ExitCode::from(2);
            }
            let Some(m) = regs.milestone(&key) else {
                eprintln!("uor-harness prep fit: no milestone {key:?}; try `uor-harness registry`");
                return ExitCode::from(2);
            };
            let reviewer = flag_value(args, "--reviewer").unwrap_or_default();
            let note = flag_value(args, "--note").unwrap_or_default();
            let sheet = flag_value(args, "--sheet").unwrap_or_default();
            let mechanism = flag_value(args, "--mechanism").unwrap_or_default();
            let body = prep::render_fit(&verdict, m, &sheet, &reviewer, &note, &mechanism);

            let lab = flag_value(args, "--lab")
                .or_else(|| std::env::var("UOR_LAB").ok())
                .unwrap_or_default();
            let mut r = NewReceipt::new("prep", lab.clone());
            r = r.with("milestone", m.id.clone());
            r = r.with("verdict", verdict.clone());
            if !sheet.is_empty() {
                r = r.with("sheet", sheet.clone());
            }
            if !reviewer.is_empty() {
                r = r.with("reviewer", reviewer.clone());
            }
            // The receipt is best-effort on purpose: **the sheet matters more than its receipt**, and
            // a broken ledger must not lose the review.
            let shard = append(&paths, &r, &current_month());
            if json {
                println!(
                    "{{\"verdict\": {:?}, \"milestone\": {:?}, \"receipt\": {:?}}}",
                    verdict,
                    m.id,
                    shard.map(|p| p.display().to_string()).unwrap_or_default()
                );
            } else {
                print!("{body}");
                match shard {
                    Ok(p) => println!("\nrecorded in {}", p.display()),
                    Err(e) => eprintln!("uor-harness: prep fit: receipt not recorded: {e}"),
                }
            }
            ExitCode::SUCCESS
        }
        "new" => {
            let Some(m) = regs.milestone(&key) else {
                eprintln!("uor-harness prep new: no milestone {key:?}; try `uor-harness registry`");
                return ExitCode::from(2);
            };
            let panel = regs.panel_for(m);
            let mech = flag_value(args, "--mechanism").unwrap_or_default();
            let sheet = prep::NewSheet {
                date: flag_value(args, "--date").unwrap_or_default(),
                mechanism: mech.clone(),
                run: flag_value(args, "--run").unwrap_or_default(),
                bar: flag_value(args, "--bar").unwrap_or_default(),
                base: flag_value(args, "--base").unwrap_or_default(),
                split: flag_value(args, "--split").unwrap_or_default(),
                steps: flag_value(args, "--steps").unwrap_or_default(),
                stop: flag_value(args, "--stop").unwrap_or_default(),
                cost: flag_value(args, "--cost").unwrap_or_default(),
                seeds: flag_value(args, "--seeds").and_then(|v| v.parse().ok()),
                rows: flag_value(args, "--rows").and_then(|v| v.parse().ok()),
            };
            // The rules come from the registry, so a sheet rendered by Rust and one rendered by Python
            // cite the same set - the drift parity caught when these were two constants.
            let rules: std::collections::HashMap<String, String> = regs
                .owner_rules
                .iter()
                .filter_map(|r| {
                    let id = r.get("id")?.as_str()?.to_string();
                    let text = r.get("rule")?.as_str()?.to_string();
                    Some((id, text))
                })
                .collect();
            let lookup = |id: &str| rules.get(id).cloned().unwrap_or_default();
            let body = prep::render_sheet(&sheet, m, panel, &lookup, &paths);
            let date = prep::today();
            let name = prep::sheet_name(&m.key, &mech, &date);
            let target = prep_dir_of(&paths).join(&name);

            // **A pre-registration sheet is the D22 gate's own record, and this overwrote one
            // silently.** Re-running `prep new` for a milestone that already has a sheet replaced the
            // bar, the panel and the cost pre-registered BEFORE the work - exactly the evidence D22
            // exists to preserve. Refusing is the safe default; replacing is explicit.
            if target.exists() && !args.iter().any(|a| a == "--force") {
                eprintln!(
                    "uor-harness: {} already exists and was NOT overwritten.\n\
                     It is the pre-registered bar, panel and cost for this milestone; replacing it\n\
                     discards the record that D22 scores the work against.\n\
                     Re-run with --force to replace it.",
                    target.display()
                );
                return ExitCode::from(1);
            }
            if let Some(parent) = target.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    eprintln!("uor-harness: cannot create {}: {e}", parent.display());
                    return ExitCode::from(1);
                }
            }
            if let Err(e) = std::fs::write(&target, &body) {
                eprintln!("uor-harness: cannot write {}: {e}", target.display());
                return ExitCode::from(1);
            }
            if json {
                println!(
                    "{{\"sheet\": {:?}, \"fitness\": {:?}}}",
                    target.display().to_string(),
                    prep::test_fitness(&body)
                );
            } else {
                println!(
                    "wrote {}",
                    target
                        .strip_prefix(&paths.repo)
                        .unwrap_or(&target)
                        .display()
                );
                print!("{body}");
            }
            ExitCode::SUCCESS
        }
        "" => {
            eprintln!("uor-harness prep: which action? new, fit, list");
            ExitCode::from(2)
        }
        other => {
            eprintln!("uor-harness prep: unknown action {other:?}; try new, fit, list");
            ExitCode::from(2)
        }
    }
}

fn prep_dir_of(paths: &StatePaths) -> std::path::PathBuf {
    uor_r4_harness::prep::prep_dir(paths)
}

/// `git <action>` - branch hygiene (D30 port of the Python `git`).
///
/// `clean-merged` is Rust. `install-hooks`, `status`, `switch` and `delete-merged` still delegate by
/// name; `switch` and `delete-merged` carry more guards than they look like they do, and porting them
/// without their tests would move the risk rather than the code.
fn cmd_git(repo: &Path, args: &[String], json: bool) -> ExitCode {
    let action = action_word(args);
    match action {
        "clean-merged" => {
            let apply = args.iter().any(|a| a == "--apply");
            let states = githelp::branch_states(repo);
            let removable: Vec<&githelp::BranchState> =
                states.iter().filter(|s| s.removable).collect();
            let held = states.iter().filter(|s| s.checked_out).count();
            let unmerged = states
                .iter()
                .filter(|s| s.state == githelp::MergeState::Unmerged)
                .count();

            if json {
                let rows: Vec<String> = states
                    .iter()
                    .map(|s| {
                        format!(
                            "    {{\"branch\": {:?}, \"state\": {:?}, \"checked_out\": {}, \"removable\": {}, \"reason\": {:?}}}",
                            s.branch,
                            match s.state {
                                githelp::MergeState::Merged => "merged",
                                githelp::MergeState::Unmerged => "unmerged",
                                githelp::MergeState::Unknown => "unknown",
                            },
                            s.checked_out, s.removable, s.reason
                        )
                    })
                    .collect();
                println!("{{\n  \"branches\": [\n{}\n  ]\n}}", rows.join(",\n"));
                return ExitCode::SUCCESS;
            }

            println!(
                "local branches: {} merged into origin/main and not checked out, {} in a worktree, {} unmerged",
                removable.len(),
                held,
                unmerged
            );
            if !removable.is_empty() {
                println!();
                println!("merged and safe to delete:");
                for s in &removable {
                    println!("  {}", s.branch);
                }
            }
            let kept: Vec<&githelp::BranchState> = states.iter().filter(|s| !s.removable).collect();
            if !kept.is_empty() {
                println!();
                println!("NOT deleted - kept, their owners decide:");
                for s in kept {
                    println!("  {:<44} {}", s.branch, s.reason);
                }
            }
            if !apply {
                if !removable.is_empty() {
                    println!();
                    println!("delete them with:  uor-harness git clean-merged --apply");
                }
                return ExitCode::SUCCESS;
            }
            // Re-checked inside `apply`, because a state computed before a worktree was added would
            // delete a branch that is now held.
            let (deleted, refused) = githelp::apply(repo, &states);
            println!();
            println!("deleted {} branch(es)", deleted.len());
            for (b, why) in refused {
                eprintln!("  refused {b}: {why}");
            }
            ExitCode::SUCCESS
        }
        // **`switch` is ported WITH ITS GUARD, which is the only reason it is worth having here.**
        // `git checkout -f` would discard an uncommitted claim, and a claim nobody has committed is
        // still a lab saying "I am working here" - so the guard is the command, not advice beside it.
        "switch" => {
            let target = args
                .iter()
                .find(|a| !a.starts_with('-') && *a != "switch")
                .cloned()
                .unwrap_or_default();
            if target.is_empty() {
                eprintln!("uor-harness git switch: which branch?");
                return ExitCode::from(2);
            }
            let create = args.iter().any(|a| a == "-b" || a == "--create");
            let paths = StatePaths::from_env(repo);
            let result = githelp::checkout_preserving_claims(&paths, &target, create);
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result).unwrap_or_default()
                );
            } else if result["switched"] == serde_json::json!(true) {
                println!(
                    "on {}: {}",
                    result["target"].as_str().unwrap_or(""),
                    result["note"].as_str().unwrap_or("")
                );
            } else {
                println!(
                    "could not switch to {}: {}",
                    result["target"].as_str().unwrap_or(""),
                    result["error"].as_str().unwrap_or("")
                );
            }
            if result["switched"] == serde_json::json!(true) {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        // **`delete-merged` is ported WITH ITS WHOLE GUARD CHAIN** — a live claim first and needing
        // no network, then a merged PR whose merge commit is on `origin/main`, then `git cherry` to
        // prove the branch introduces nothing upstream lacks. See `githelp::delete_merged_branch`.
        //
        // **`safe_to_delete` is the only flag this arm reads**, because the guard's own history
        // records what happens otherwise: an earlier version keyed on `merged`, a guard returned early
        // without clearing it, and **a REFUSED branch still printed "would delete".**
        "delete-merged" => {
            // **The branch is read from `--branch` FIRST, then positionally.** Python's parser
            // requires `--branch`; the port accepts either, **because a positional form is what the
            // sibling commands use (`git switch <target>`, `clean-merged`) and requiring the flag
            // would make this one command different for no reason a lab could see.** Accepting both
            // means a documented `--branch` invocation and a natural positional one behave
            // identically.
            let branch = args
                .iter()
                .position(|a| a == "--branch")
                .and_then(|i| args.get(i + 1).cloned())
                .or_else(|| {
                    args.iter()
                        .find(|a| !a.starts_with('-') && *a != "delete-merged" && *a != "git")
                        .cloned()
                })
                .unwrap_or_default();
            if branch.is_empty() {
                eprintln!("uor-harness git delete-merged: which branch?");
                return ExitCode::from(2);
            }
            let apply = args.iter().any(|a| a == "--apply");
            let paths = StatePaths::from_env(repo);
            let r = githelp::delete_merged_branch(&paths, &branch, apply);
            let safe = r["safe_to_delete"] == serde_json::json!(true);
            if json {
                println!("{}", serde_json::to_string_pretty(&r).unwrap_or_default());
            } else if safe {
                let deleted = r["deleted"] == serde_json::json!(true);
                let remote = r["remote_deleted"] == serde_json::json!(true);
                if deleted || remote {
                    // **Report each side separately.** A partial delete that reads as a success is how
                    // a lab ends up believing a branch is gone when it is not.
                    let local = if deleted {
                        "deleted".to_string()
                    } else {
                        format!("NOT deleted ({})", r["local_error"].as_str().unwrap_or("?"))
                    };
                    let mut rem = if remote {
                        "deleted".to_string()
                    } else {
                        "NOT deleted".to_string()
                    };
                    if let Some(e) = r["remote_error"].as_str() {
                        rem.push_str(&format!(" ({e})"));
                    }
                    println!("{branch}: local {local}; remote {rem}");
                } else {
                    println!(
                        "would delete {branch}: {}",
                        r["reason"].as_str().unwrap_or("")
                    );
                    println!("  run with --apply to delete it");
                }
            } else {
                println!(
                    "REFUSED to delete {branch}: {}",
                    r["reason"].as_str().unwrap_or("")
                );
                println!("  an unmerged remote branch is usually the only copy of that work");
            }
            if safe {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        // **`install-hooks` is ported because its refusals are the point**: it never clobbers a
        // foreign hook, and each refusal names what it found and how to proceed.
        "install-hooks" => {
            let paths = StatePaths::from_env(repo);
            let (code, out, err) = githelp::install_hooks(&paths);
            for l in &out {
                println!("{l}");
            }
            for l in &err {
                eprintln!("{l}");
            }
            if code == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(code as u8)
            }
        }
        // **`git status` is Rust**, and it is the whole thing: worktree facts, the PR, the delivery
        // state and the next command. **`merged-unverified` exits 1**, because that state means the PR
        // is merged and its commit is NOT on the origin/main we can see — **a lab that treats it as
        // success deletes a branch whose work it never found.**
        "status" => {
            let (data, state) = githelp::git_status(repo);
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&data).unwrap_or_default()
                );
            } else {
                let f = &data;
                println!(
                    "uor-harness git status - branch {} @ {}",
                    f["branch"].as_str().unwrap_or("?"),
                    f["head"].as_str().unwrap_or("?")
                );
                if f["changes_available"].as_bool() == Some(true) {
                    let c = &f["changes"];
                    let cleanliness = if f["clean"].as_bool() == Some(true) {
                        "clean".to_string()
                    } else {
                        format!(
                            "{} uncommitted change(s) ({} staged, {} unstaged, {} untracked)",
                            c["total"], c["staged"], c["unstaged"], c["untracked"]
                        )
                    };
                    println!(
                        "1. worktree {} - {cleanliness}",
                        f["worktree"].as_str().unwrap_or("")
                    );
                } else {
                    println!(
                        "1. worktree {} - UNAVAILABLE: {}",
                        f["worktree"].as_str().unwrap_or(""),
                        f["changes_reason"].as_str().unwrap_or("")
                    );
                }
                if f["origin_main"]["available"].as_bool() == Some(true) {
                    let commit = f["origin_main"]["commit"].as_str().unwrap_or("");
                    let short = &commit[..commit.len().min(8)];
                    let fetched = if f["fetch"]["ok"].as_bool() == Some(true) {
                        "just fetched".to_string()
                    } else {
                        format!(
                            "last fetch FAILED: {}",
                            f["fetch"]["reason"].as_str().unwrap_or("")
                        )
                    };
                    println!(
                        "2. origin/main {short} ({}) - {fetched}",
                        f["origin_main"]["subject"].as_str().unwrap_or("")
                    );
                } else {
                    println!(
                        "2. origin/main - UNAVAILABLE: {}",
                        f["origin_main"]["reason"]
                            .as_str()
                            .unwrap_or("cannot resolve origin/main")
                    );
                }
                let pr = &f["pr"];
                if pr["available"].as_bool() == Some(true) {
                    let draft = if pr["is_draft"].as_bool() == Some(true) {
                        " (draft)"
                    } else {
                        ""
                    };
                    let contained = match pr["contained_in_origin_main"].as_bool() {
                        Some(true) => "contained in origin/main",
                        Some(false) => "NOT contained in origin/main",
                        None => "containment unknown",
                    };
                    println!(
                        "3. PR #{} {}{draft}: {}",
                        pr["number"],
                        pr["state"].as_str().unwrap_or(""),
                        uor_r4_harness::prbody::truncate(pr["title"].as_str().unwrap_or(""), 80)
                    );
                    let sha = pr["checked_sha"].as_str().unwrap_or("");
                    let refname = if pr["checked_ref"].as_str().unwrap_or("").is_empty() {
                        "no merge ref".to_string()
                    } else {
                        pr["checked_ref"].as_str().unwrap_or("").to_string()
                    };
                    println!("   {refname} {} - {contained}", &sha[..sha.len().min(8)]);
                } else {
                    println!(
                        "3. PR - none for this branch ({})",
                        pr["reason"].as_str().unwrap_or("gh unavailable")
                    );
                }
                if f["upstream"].as_str().unwrap_or("").is_empty() {
                    println!("4. upstream - none (branch is not pushed)");
                } else {
                    println!(
                        "4. upstream {} - ahead {} / behind {} of origin/main",
                        f["upstream"].as_str().unwrap_or(""),
                        f["ahead"],
                        f["behind"]
                    );
                }
                println!("5. state: {state}");
                println!("6. next: {}", f["next_command"].as_str().unwrap_or(""));
                for step in f["next_steps"].as_array().into_iter().flatten() {
                    println!("   then: {}", step.as_str().unwrap_or(""));
                }
                for note in f["notes"].as_array().into_iter().flatten() {
                    println!("   note: {}", note.as_str().unwrap_or(""));
                }
            }
            if state == "merged-unverified" {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        "" => {
            eprintln!("uor-harness git: which action? install-hooks, status, clean-merged, switch, delete-merged");
            ExitCode::from(2)
        }
        other => {
            eprintln!("uor-harness git: unknown action {other:?}; try clean-merged");
            ExitCode::from(2)
        }
    }
}

/// `multilab <action>` - the shared board (D30 port of the Python `multilab`).
///
/// `presence`, `claims` and `touch` are Rust. `sync-live` and `migrate-ledger` still delegate by
/// name, so each remaining action says so individually.
fn cmd_multilab(repo: &Path, args: &[String], json: bool) -> ExitCode {
    let paths = StatePaths::from_env(repo);
    let action = action_word(args);
    let lab = flag_value(args, "--lab")
        .or_else(|| std::env::var("UOR_LAB").ok())
        .unwrap_or_default();

    match action {
        "touch" => {
            if lab.trim().is_empty() {
                eprintln!("uor-harness multilab touch: {}", multilab::no_lab_message());
                return ExitCode::from(2);
            }
            match multilab::touch(&paths, &lab) {
                Ok(p) => {
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&p.to_json()).unwrap_or_default()
                        );
                    } else {
                        println!("{}: live, updated {}", p.lab, p.updated);
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("uor-harness multilab touch: {e}");
                    ExitCode::from(1)
                }
            }
        }
        "presence" => {
            let now = now_unix();
            let rows = multilab::board(&paths, now);
            if json {
                let mut items: Vec<String> = Vec::new();
                for e in &rows {
                    items.push(format!(
                        "    {{\"lab\": {:?}, \"updated\": {:?}, \"milestone\": {:?}, \"branch\": {:?}, \"worktree\": {:?}, \"doing\": {:?}, \"next\": {:?}, \"host\": {:?}, \"session\": {:?}, \"source\": {:?}, \"state\": {:?}}}",
                        e.row.lab, e.row.updated, e.row.milestone, e.row.branch, e.row.worktree,
                        e.row.doing, e.row.next, e.row.host, e.row.session, e.source, e.state()
                    ));
                }
                println!("[\n{}\n]", items.join(",\n"));
            } else if rows.is_empty() {
                println!("no lab has published presence yet");
                println!("  publish yours with:  export UOR_LAB=<lab> && uor-harness heartbeat -m <milestone>");
            } else {
                println!(
                    "{} lab(s) known (stale after {}m) · live tier: {}",
                    rows.len(),
                    uor_r4_harness::presence::PRESENCE_TTL_MINUTES,
                    paths.live.display()
                );
                println!(
                    "  {:<12} {:<20} {:<8} {:<9} {:<20} doing",
                    "lab", "state", "via", "milestone", "branch"
                );
                println!("  {}", multilab::separator());
                for e in &rows {
                    let branch = if e.row.branch.is_empty() {
                        "-"
                    } else {
                        &e.row.branch
                    };
                    println!(
                        "  {:<12} {:<20} {:<8} {:<9} {:<20} {}",
                        e.row.lab,
                        e.state(),
                        e.source,
                        e.row.milestone,
                        branch,
                        e.row.doing
                    );
                }
            }
            ExitCode::SUCCESS
        }
        "claims" => {
            let claims = multilab::claims(&paths);
            let cur = team::current_branch(repo);
            if json {
                let obj: serde_json::Map<String, serde_json::Value> = claims
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_json()))
                    .collect();
                let body = serde_json::json!({
                    "current_branch": cur,
                    "claims": serde_json::Value::Object(obj),
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&body).unwrap_or_default()
                );
            } else if claims.is_empty() {
                println!(
                    "no branch claims recorded · current branch {}",
                    if cur.is_empty() { "(detached)" } else { &cur }
                );
            } else {
                println!(
                    "branch claims ({}) · current branch {}",
                    claims.len(),
                    if cur.is_empty() { "(detached)" } else { &cur }
                );
                for (br, rec) in &claims {
                    let mark = if *br == cur { "  <- current" } else { "" };
                    println!(
                        "  {br:<44} lab={:<10} since {}{mark}",
                        rec.lab, rec.claimed_at
                    );
                }
            }
            ExitCode::SUCCESS
        }
        // Not ported yet, and each says so individually.
        "sync-live" | "migrate-ledger" => delegate_to_python(repo, "multilab", args),
        "" => {
            eprintln!("uor-harness multilab: which action? presence, claims, touch, sync-live, migrate-ledger");
            ExitCode::from(2)
        }
        other => {
            eprintln!(
                "uor-harness multilab: unknown action {other:?}; try presence, claims, touch"
            );
            ExitCode::from(2)
        }
    }
}

/// `team <action>` - branch claims (D30 port of the Python `team`).
///
/// `claim` / `release` / `claims` are here; `roster` and `dispatch` still delegate, because they read
/// the roles registry and the port is landing in the order labs use it. **`team` is NOT in the
/// delegated set**, so the "not ported yet" notice stays off for the actions that are Rust and each
/// remaining action says so individually.
fn cmd_team(repo: &Path, args: &[String], json: bool) -> ExitCode {
    let paths = StatePaths::from_env(repo);
    // The action is the first bare word that is NOT the value of a preceding value-taking flag.
    //
    // Two wrong versions preceded this one: `args[0]` made `--repo <dir>` the action (the top-level
    // parser forwards both into `rest`), and "first word not starting with `-`" made the repo PATH the
    // action, because the path is a bare word too. The flag's value has to be stepped over, not
    // treated as a candidate.
    let action = action_word(args);
    let branch = flag_value(args, "--branch").unwrap_or_default();
    let lab = flag_value(args, "--lab")
        .or_else(|| std::env::var("UOR_LAB").ok())
        .unwrap_or_default();
    let note = flag_value(args, "--note").unwrap_or_default();

    match action {
        "claim" => match team::claim_branch(&paths, &branch, &lab, &note) {
            Ok(c) => {
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&c.to_json()).unwrap_or_default()
                    );
                } else {
                    println!("claimed {} for lab {} at {}", c.branch, c.lab, c.claimed_at);
                    println!(
                        "  head {} · {}",
                        c.head,
                        team::claims_file(&paths).display()
                    );
                }
                ExitCode::SUCCESS
            }
            Err(team::ClaimError::Usage(m)) => {
                eprintln!("team claim: {m}");
                ExitCode::from(2)
            }
            Err(team::ClaimError::Held(c)) => {
                eprintln!("team claim: {}", team::held_message(&c));
                eprintln!("  use a different branch, or release it with: uor-harness team release --branch <name>");
                ExitCode::from(1)
            }
            Err(team::ClaimError::NotHeld(b)) => {
                eprintln!("team claim: no claim on {b:?}");
                ExitCode::from(1)
            }
        },
        "release" => match team::release_branch(&paths, &branch, &lab) {
            Ok(c) => {
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&c.to_json()).unwrap_or_default()
                    );
                } else {
                    println!("released {} (was held by {})", c.branch, c.lab);
                }
                ExitCode::SUCCESS
            }
            Err(team::ClaimError::NotHeld(b)) => {
                eprintln!("team release: no claim on {b:?}");
                ExitCode::from(1)
            }
            Err(team::ClaimError::Held(c)) => {
                // Refusing to release someone else's claim is the point: a release that ignored the
                // owner would let any lab free any branch.
                eprintln!(
                    "team release: {} is held by lab {:?}, not {lab:?}",
                    c.branch, c.lab
                );
                ExitCode::from(1)
            }
            Err(team::ClaimError::Usage(m)) => {
                eprintln!("team release: {m}");
                ExitCode::from(2)
            }
        },
        "claims" => {
            let claims = team::load_claims(&paths);
            let cur = team::current_branch(repo);
            if json {
                let obj: serde_json::Map<String, serde_json::Value> = claims
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_json()))
                    .collect();
                let body = serde_json::json!({
                    "current_branch": cur,
                    "claims": serde_json::Value::Object(obj),
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&body).unwrap_or_default()
                );
            } else if claims.is_empty() {
                println!(
                    "no branch claims recorded · current branch {}",
                    if cur.is_empty() { "(detached)" } else { &cur }
                );
                println!("claim one with: uor-harness team claim --branch <name> --lab <lab>");
            } else {
                println!(
                    "branch claims ({}) · current branch {}",
                    claims.len(),
                    if cur.is_empty() { "(detached)" } else { &cur }
                );
                for (br, rec) in &claims {
                    let mark = if *br == cur { "  <- current" } else { "" };
                    println!(
                        "  {br:<44} lab={:<10} since {}{mark}",
                        rec.lab, rec.claimed_at
                    );
                }
            }
            ExitCode::SUCCESS
        }
        // `roster` reads the roles registry and prints it, so it is Rust.
        //
        // `dispatch` is NOT, and the reason is not the registry: it also WRITES a receipt through
        // the ledger shard writer, so porting it means porting `receipts.record` with its redaction,
        // its lab attribution and its unattributed warning. Wiring the read half while the write
        // half stays in Python would be the half-ported command this port keeps refusing to ship.
        "roster" => match uor_r4_harness::registries::load_roles(&paths) {
            Ok(roles) => {
                if json {
                    println!("{}", uor_r4_harness::registries::roster_json(&roles));
                } else {
                    print!("{}", uor_r4_harness::registries::render_roster(&roles));
                }
                ExitCode::SUCCESS
            }
            // Python raises on a registry it cannot read rather than printing an empty roster,
            // and `0 dispatchable roles` is precisely the wrong answer that reads like a right one.
            Err(e) => {
                eprintln!("uor-harness team roster: {e}");
                ExitCode::from(1)
            }
        },
        // dispatch records a real dispatch: it resolves the role, writes ONE receipt through
        // receipts.record and prints what landed. The write half is the point of the command, so
        // it is why this stayed delegated until record existed.
        "dispatch" => {
            let roles = match uor_r4_harness::registries::load_roles(&paths) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("uor-harness team dispatch: {e}");
                    return ExitCode::from(1);
                }
            };
            // role is positional and optional. Python prints None when it is absent, because the
            // message renders the parsed value with !r rather than naming a missing argument.
            let key = action_word_after(args, &["dispatch"]);
            let role = if key.is_empty() {
                None
            } else {
                uor_r4_harness::registries::find_role(&roles, &key)
            };
            let Some(role) = role else {
                let shown = if key.is_empty() {
                    "None".to_string()
                } else {
                    uor_r4_harness::prgates::py_repr_public(&key)
                };
                eprintln!("unknown role {shown}; try: uor-harness team roster");
                return ExitCode::from(2);
            };
            let milestone = flag_value(args, "--milestone")
                .or_else(|| flag_value(args, "-m"))
                .unwrap_or_default();
            let task = flag_value(args, "--task").unwrap_or_default();
            let scope = flag_all_values(args, "--scope");
            // Python's args.count or 1: a count of ZERO becomes one, because it tests truthiness.
            let count = match flag_value(args, "--count") {
                None => 1i64,
                Some(v) => match v.trim().parse::<i64>() {
                    Ok(0) => 1,
                    Ok(n) => n,
                    Err(_) => {
                        eprintln!("uor-harness: argument --count: invalid int value: {v:?}");
                        return ExitCode::from(2);
                    }
                },
            };
            let state = flag_value(args, "--state").unwrap_or_else(|| "dispatched".to_string());
            if !["dispatched", "running", "returned", "failed"].contains(&state.as_str()) {
                eprintln!(
                    "uor-harness: argument --state: invalid choice: {state:?} (choose from \
                     'dispatched', 'running', 'returned', 'failed')"
                );
                return ExitCode::from(2);
            }
            let route = role.route();
            let fields: Vec<(&str, serde_json::Value)> = vec![
                ("role", serde_json::Value::String(role.id.clone())),
                ("route", serde_json::Value::String(route.clone())),
                ("milestone", serde_json::Value::String(milestone)),
                ("task", serde_json::Value::String(task)),
                (
                    "scope",
                    serde_json::Value::Array(
                        scope.into_iter().map(serde_json::Value::String).collect(),
                    ),
                ),
                ("count", serde_json::Value::from(count)),
                ("state", serde_json::Value::String(state)),
            ];
            let recorded = match uor_r4_harness::receipt::record(
                &paths,
                "team",
                fields,
                std::env::var("UOR_LAB").ok().as_deref(),
            ) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("uor-harness: {e}");
                    return ExitCode::from(1);
                }
            };
            if recorded.unattributed {
                // Python warns and PRESERVES the receipt; refusing would lose the record rather than
                // surface the gap, which is the whole point of the warning.
                eprintln!(
                    "uor-harness: WARNING - no lab identity, so this record is attributed to nobody and no\n\
                     \x20            other lab can see who did it. Fix it for this session with:\n\
                     \x20            export UOR_LAB=<kimi|claude|codex|deepseek>"
                );
            }
            let ts = recorded.get_str("ts");
            if json {
                println!("{}", recorded.to_pretty());
            } else {
                println!(
                    "dispatched {} -> {route} (count {}) at {ts}",
                    role.id,
                    recorded
                        .get("count")
                        .and_then(serde_json::Value::as_i64)
                        .unwrap_or(count)
                );
                let task = recorded.get_str("task");
                if !task.is_empty() {
                    println!("  task: {task}");
                }
                if let Some(items) = recorded.get("scope").and_then(serde_json::Value::as_array) {
                    let rendered: Vec<&str> =
                        items.iter().filter_map(serde_json::Value::as_str).collect();
                    if !rendered.is_empty() {
                        println!("  write scope: {}", rendered.join(", "));
                    }
                }
                // Python prints the UNSHARDED month file here: the shard a line lands in is named
                // after the lab, while this is the path a reader opens.
                println!(
                    "  receipt: docs/harness/ledger/{}.jsonl",
                    &ts[..7.min(ts.len())]
                );
            }
            ExitCode::SUCCESS
        }
        "" => {
            eprintln!("uor-harness team: which action? claim, release, claims, roster, dispatch");
            ExitCode::from(2)
        }
        other => {
            eprintln!("uor-harness team: unknown action {other:?}; try claim, release, claims, roster, dispatch");
            ExitCode::from(2)
        }
    }
}

/// `version` - which commit this binary came from, and whether it is behind `origin/main`.
///
/// Exists because the alternative was a lab reading `unknown option --line` and having to guess that
/// its binary predated the flag. Asking directly is cheaper than diagnosing.
fn cmd_version(repo: &Path) -> ExitCode {
    let s = uor_r4_harness::version::stale(repo);
    println!(
        "uor-harness binary built at {}; this checkout is at {}",
        uor_r4_harness::version::BUILT_AT,
        uor_r4_harness::version::short(&uor_r4_harness::version::head(repo))
    );
    match &s {
        uor_r4_harness::version::Staleness::Behind { behind, main, .. } => {
            println!("  BEHIND origin/main ({main}) by {behind} commit(s) - rebuild before trusting a flag error");
        }
        uor_r4_harness::version::Staleness::Current { .. } => {
            println!("  current with origin/main")
        }
        uor_r4_harness::version::Staleness::Unknown { why } => println!("  cannot tell: {why}"),
    }
    ExitCode::SUCCESS
}

/// Run an unported command through the Python harness, announcing that it is unported.
///
/// The Rust binary is the entry point; Python is a fallback implementation. A command that is missing
/// from the Rust match above is forwarded verbatim, so every documented invocation works while the
/// port continues - and each one prints which path it took.
fn delegate_to_python(repo: &Path, command: &str, rest: &[String]) -> ExitCode {
    let script = repo.join("scripts").join("harness").join("cli.py");
    if !script.is_file() {
        eprintln!(
            "uor-harness: {command:?} is not ported yet, and {} is not present to run it.\n\
             \x20            This binary is the entry point; the remaining commands run through the \
             Python harness until they are ported.",
            script.display()
        );
        return ExitCode::from(2);
    }
    eprintln!(
        "uor-harness: note - {command:?} is not ported to Rust yet; running {} {command}",
        script.display()
    );
    // `--repo` is added here, so a copy forwarded into `rest` must be dropped rather than passed
    // twice: argparse rejects the duplicate and the delegation fails with "unrecognized arguments".
    let mut forwarded: Vec<&String> = Vec::new();
    let mut skip_next = false;
    for a in rest {
        if skip_next {
            skip_next = false;
            continue;
        }
        if a == "--repo" {
            skip_next = true;
            continue;
        }
        forwarded.push(a);
    }
    let status = std::process::Command::new("python3")
        .arg(&script)
        .arg("--repo")
        .arg(repo)
        .arg(command)
        .args(forwarded)
        .status();
    match status {
        Ok(s) => ExitCode::from(s.code().unwrap_or(1) as u8),
        Err(e) => {
            eprintln!("uor-harness: cannot run python3: {e}");
            ExitCode::from(2)
        }
    }
}

/// `notes [--lab L] [--cached|--mark-seen]` - what other labs said that this lab has not seen.
///
/// **Presence without awareness is half the plane.** The board says who is working; it does not say
/// that a CONTEST was posted twenty minutes ago and nobody has read it. That happened twice on
/// 13 October, and this is the durable fix: the check-in itself reports what you have missed.
///
/// Default refreshes from GitHub. `--cached` reads the cache and makes no network call (for a
/// check-in that must be instant); `--mark-seen` advances the marker without printing, for when the
/// notes have been read.
fn cmd_notes(repo: &Path, args: &[String], json: bool) -> ExitCode {
    let paths = StatePaths::from_env(repo);
    let lab = match flag_value(args, "--lab").or_else(|| std::env::var("UOR_LAB").ok()) {
        Some(l) if !l.trim().is_empty() => l,
        _ => {
            eprintln!(
                "uor-harness: which lab? pass --lab or set UOR_LAB - a marker with no owner would \
                 mark another lab's notes read."
            );
            return ExitCode::from(2);
        }
    };
    let regs = Registries::load(&paths).ok();
    let issues = notes::watched_issues(regs.as_ref());

    let mut fetched = 0usize;
    // Whether the read covered everything. A `--cached` call or a failed fetch does not know what it
    // missed, and the report says "at least" rather than presenting a floor as the truth.
    let mut exact = true;
    if !args.iter().any(|a| a == "--cached") {
        for issue in &issues {
            match fetch_issue_notes(*issue) {
                Some(found) => match notes::cache_append(&paths, &lab, &found) {
                    Ok(n) => fetched += n,
                    Err(e) => {
                        exact = false;
                        eprintln!("uor-harness: notes cache: {e}");
                    }
                },
                // A fetch failure is reported but NOT fatal: a lab with no network must still be able
                // to read what it already knows, and a check-in must never be blocked by GitHub.
                None => {
                    exact = false;
                    eprintln!("uor-harness: could not read #{issue} (no network or no gh)");
                }
            }
        }
    } else {
        exact = false;
    }

    let cached = notes::read_cache(&paths, &lab);
    let mut markers = notes::Markers::load(&paths, &lab);
    let unread = markers.unread(&cached);
    let items = notes::summarise(&unread);

    if args.iter().any(|a| a == "--mark-seen") {
        markers.advance(&cached);
        let saved = markers.save(&paths);
        if !json {
            println!(
                "notes: marked {} note(s) seen for {lab}{}",
                unread.len(),
                if saved {
                    ""
                } else {
                    " (marker not saved: live tier unwritable)"
                }
            );
        }
        return ExitCode::SUCCESS;
    }

    if json {
        let rows: Vec<String> = items
            .iter()
            .map(|u| {
                format!(
                    "    {{\"issue\": {}, \"count\": {}, \"latest_id\": {}, \"latest_author\": {:?}, \"latest_first_line\": {:?}}}",
                    u.issue, u.count, u.latest.id, u.latest.author, u.latest.first_line
                )
            })
            .collect();
        println!(
            "{{\n  \"lab\": {:?},\n  \"fetched\": {fetched},\n  \"cached\": {},\n  \"unread\": {},\n  \"notes\": [\n{}\n  ]\n}}",
            lab,
            cached.len(),
            unread.len(),
            rows.join(",\n")
        );
    } else {
        for line in notes::render(&items, exact) {
            println!("{line}");
        }
        if items.is_empty() {
            println!("notes: nothing unread");
        } else {
            println!();
            println!("Open them on the issue, then run: uor-harness notes --mark-seen");
        }
    }
    ExitCode::SUCCESS
}

/// Fetch one issue's comments as `id<TAB>author<TAB>created_at<TAB>body-first-line`.
///
/// `gh` does the authentication and the JSON; this reads four tab-separated fields rather than take a
/// dependency on walking a dynamic tree. `None` means the call failed, which the caller reports and
/// survives - **a check-in must never be blocked by GitHub.**
fn fetch_issue_notes(issue: i64) -> Option<Vec<notes::Note>> {
    let out = std::process::Command::new("gh")
        .args([
            "api",
            // `--paginate`, because ONE PAGE IS NOT THE ISSUE. Without it this read only the first
            // 100 comments, so on #2029 (231 comments) it reported an old note as "latest" and missed
            // every recent one - the exact failure the command exists to prevent, in the command
            // itself. Claude reported it after #2029/#2030/#2032 all showed stale "latest" ids.
            "--paginate",
            &format!("repos/UOR-Foundation/uor-r4/issues/{issue}/comments?per_page=100"),
            "--jq",
            ".[] | [(.id|tostring), .user.login, .created_at, (.body|split(\"\\n\")|.[0])] | @tsv",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(notes::parse_tsv(
        issue,
        &String::from_utf8_lossy(&out.stdout),
    ))
}

/// `ledger sync [--lab L] [--month M]` - append live receipts into the committed shard.
///
/// The delivery step. Run it before opening a PR so the receipts recorded since the last delivery
/// reach the committed record that review sees.
fn cmd_ledger_sync(repo: &Path, args: &[String], json: bool) -> ExitCode {
    let paths = StatePaths::from_env(repo);
    let month = flag_value(args, "--month").unwrap_or_else(current_month);
    let lab = match flag_value(args, "--lab").or_else(|| std::env::var("UOR_LAB").ok()) {
        Some(l) if !l.trim().is_empty() => l,
        _ => {
            eprintln!(
                "uor-harness: which lab? pass --lab or set UOR_LAB. A sync with no lab would carry \
                 nobody's receipts and report success."
            );
            return ExitCode::from(2);
        }
    };
    match sync_live_to_committed(&paths, &lab, &month) {
        Ok(r) => {
            if json {
                println!(
                    "{{\"shard\": {:?}, \"added\": {}, \"already_committed\": {}, \"wrote\": {}}}",
                    r.shard, r.added, r.already_committed, r.wrote
                );
            } else if r.added == 0 {
                println!(
                    "ledger sync: nothing to add for {lab} in {month} ({} live receipt(s) already committed)",
                    r.already_committed
                );
            } else {
                println!(
                    "ledger sync: appended {} receipt(s) to {} ({} already committed)",
                    r.added, r.shard, r.already_committed
                );
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("uor-harness: {e}");
            ExitCode::from(1)
        }
    }
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
    // **A check-in must not ERASE what the lab already said.** `heartbeat -m m1` with no `--line`
    // and no `--next` used to rewrite presence with both fields empty, so a lab checking in with
    // the documented command read as live-and-idle on every board - including its own last intent,
    // which its previous file had held one moment earlier. Reported by claude/review1 on #2028
    // (6163960613) with the repro, and visible on the board as "live" with an empty `doing`.
    //
    // **Absent is not the same as empty.** A flag passed with an empty value is a lab clearing the
    // field deliberately and still works; only a flag that was never passed falls back.
    let previous = multilab::read_row(&paths, &lab);
    let prev_milestone = previous.as_ref().map(|r| r.milestone.clone());
    let prev_doing = previous.as_ref().map(|r| r.doing.clone());
    let prev_next = previous.as_ref().map(|r| r.next.clone());
    let prev_session = previous.as_ref().map(|r| r.session.clone());
    let milestone = presence::remembered(
        flag_value(args, "--milestone").or_else(|| flag_value(args, "-m")),
        prev_milestone.as_deref(),
    );
    let mut p = Presence::new(&paths, &lab, &milestone);
    // `--line` is the session contract's name (every PR declares its line and count); `--doing` is
    // the older spelling. Both are accepted so a lab following the contract does not hit an unknown
    // option on the command it must run every few minutes.
    p = p.with_doing(presence::remembered(
        flag_value(args, "--line").or_else(|| flag_value(args, "--doing")),
        prev_doing.as_deref(),
    ));
    p = p.with_next(presence::remembered(
        flag_value(args, "--next"),
        prev_next.as_deref(),
    ));
    // `--session` names WHICH session of this lab is checking in; a lab may run several. It was
    // never settable, so every presence file carried `"session":""` and two sessions of one lab
    // were indistinguishable on the board (claude/review1, #2028 6163960613 item 2). Carried
    // forward like the intent, for the same reason.
    p.session = presence::remembered(flag_value(args, "--session"), prev_session.as_deref());
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
    // The D21 count and headline are what the owner reads to see whether a line is moving. They were
    // dropped from the receipt, so a Rust check-in recorded less than the Python one it replaces.
    if let Some(v) = flag_value(args, "--count") {
        r = r.with("count", v);
    }
    if let Some(v) = flag_value(args, "--headline") {
        r = r.with("headline", v);
    }
    let month = current_month();
    // LIVE shard: a check-in every five minutes must never dirty a working checkout. `ledger sync`
    // carries these lines into the committed shard before a PR, so the record still reaches review.
    let shard = match append_live(&paths, &r, &month) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("uor-harness: {e}");
            return ExitCode::from(1);
        }
    };
    // A stale binary is worth one line at check-in: it is the moment a lab is already reading output,
    // and a build that predates a flag fails with "unknown option" rather than anything useful.
    if let Some(line) = uor_r4_harness::version::stale(repo).line() {
        eprintln!("uor-harness: {line}");
    }
    // Surface unread notes as part of the check-in: presence without awareness is half the plane.
    // Read from the CACHE, not the network - a check-in must stay instant, and `notes` refreshes.
    if !args.iter().any(|a| a == "--no-notes") {
        let cached = notes::read_cache(&paths, &lab);
        if !cached.is_empty() {
            let markers = notes::Markers::load(&paths, &lab);
            let items = notes::summarise(&markers.unread(&cached));
            // The heartbeat reads the CACHE, so it cannot know what a failed refresh missed: the
            // count it prints is a floor and says so.
            for line in notes::render(&items, false) {
                eprintln!("{line}");
            }
        }
    }
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

fn cmd_brief(repo: &Path, args: &[String]) -> ExitCode {
    let paths = StatePaths::from_env(repo);
    let regs = match Registries::load(&paths) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("uor-harness: {e}");
            return ExitCode::from(2);
        }
    };
    let milestone = flag_value(args, "--milestone").or_else(|| flag_value(args, "-m"));
    let query = flag_value(args, "--query")
        .or_else(|| flag_value(args, "-q"))
        .unwrap_or_default();
    let pack = if flag_value(args, "--section1-only").is_some() {
        briefing::assemble(&regs, milestone.as_deref(), &query)
    } else {
        briefing::assemble_full(&regs, &paths, milestone.as_deref(), &query)
    };
    print!("{}", pack.render());
    // The sections this port does not provide are STATED, not silently absent: a brief that omitted
    // the session contract would read as a brief with no session contract.
    println!();
    println!(
        "*Not ported to Rust yet:* the lessons that apply (7) and the skills (8), which need the \
         mistakes and skills registries. `scripts/harness/cli.py brief` remains authoritative for \
         those two."
    );
    ExitCode::SUCCESS
}

fn cmd_shard_check(repo: &Path, args: &[String], json: bool) -> ExitCode {
    let paths = StatePaths::from_env(repo);
    let base = flag_value(args, "--base").unwrap_or_else(|| "origin/main".to_string());
    let reports = match shards::check_ledger(&paths, &base) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("uor-harness: {e}");
            return ExitCode::from(2);
        }
    };
    if reports.is_empty() {
        println!("shard integrity: no ledger shards present under docs/harness/ledger/");
        return ExitCode::SUCCESS;
    }
    let bad = shards::violations(&reports);
    if json {
        let rows: Vec<String> = reports
            .iter()
            .map(|r| {
                format!(
                    "    {{\"shard\": {:?}, \"base_lines\": {}, \"head_lines\": {}, \"lost\": {}, \"added\": {}, \"ok\": {}}}",
                    r.path.display().to_string(),
                    r.base_lines,
                    r.head_lines,
                    r.lost_count(),
                    r.added,
                    r.ok()
                )
            })
            .collect();
        println!(
            "{{\n  \"base\": {:?},\n  \"shards\": [\n{}\n  ],\n  \"violations\": {}\n}}",
            base,
            rows.join(",\n"),
            bad.len()
        );
    } else {
        print!("{}", shards::render(&reports));
    }
    if bad.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn cmd_status(repo: &Path, args: &[String]) -> ExitCode {
    let paths = StatePaths::from_env(repo);
    // A caller may pin the timestamp so two implementations can be compared; otherwise it is now.
    let now = flag_value(args, "--at").unwrap_or_else(uor_r4_harness::receipt::now_rfc3339);
    // No `--lab`: section 2 aggregates EVERY lab's shard, so the view has no "whose receipts"
    // question to answer.
    let regs = Registries::load(&paths).ok();
    print!("{}", status::render(&paths, regs.as_ref(), &now));
    ExitCode::SUCCESS
}

fn cmd_prep_render(repo: &Path, args: &[String]) -> ExitCode {
    use uor_r4_harness::prep::{render_sheet, NewSheet};

    let paths = StatePaths::from_env(repo);
    let regs = match Registries::load(&paths) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("uor-harness: {e}");
            return ExitCode::from(2);
        }
    };
    let key = flag_value(args, "--milestone")
        .or_else(|| flag_value(args, "-m"))
        .unwrap_or_default();
    let Some(m) = regs.milestone(&key) else {
        eprintln!("uor-harness: no milestone {key:?}; try `uor-harness registry`");
        return ExitCode::from(2);
    };
    let panel = regs.panel_for(m);
    let sheet = NewSheet {
        date: flag_value(args, "--date").unwrap_or_default(),
        mechanism: flag_value(args, "--mechanism").unwrap_or_default(),
        run: flag_value(args, "--run").unwrap_or_default(),
        bar: flag_value(args, "--bar").unwrap_or_default(),
        base: flag_value(args, "--base").unwrap_or_default(),
        split: flag_value(args, "--split").unwrap_or_default(),
        steps: flag_value(args, "--steps").unwrap_or_default(),
        stop: flag_value(args, "--stop").unwrap_or_default(),
        cost: flag_value(args, "--cost").unwrap_or_default(),
        seeds: flag_value(args, "--seeds").and_then(|v| v.parse().ok()),
        rows: flag_value(args, "--rows").and_then(|v| v.parse().ok()),
    };
    // Prints, never writes. `prep new` writing files is a separate decision, and a renderer that
    // writes on its first outing is one nobody can inspect before it does.
    print!(
        "{}",
        render_sheet(&sheet, m, panel, &|id| regs.rule(id), &paths)
    );
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

/// `check` - the frozen formats **and** every rule.
///
/// **Deliberately a separate function from `cmd_check`.** `cmd_check` is the format contract and
/// nothing else, proven by `parity`'s six controlled one-row cases; this one adds the rule set and
/// the single-object JSON that `parity` parses.
///
/// **The rules are computed once and printed into the same object as the formats**, because a second
/// object after the first is `Extra data` to any JSON reader - which is how the first attempt at this
/// was caught.
fn cmd_check_rules(
    repo: &Path,
    args: &uor_r4_harness::checkdispatch::CheckArgs,
    json: bool,
) -> ExitCode {
    let paths = StatePaths::from_env(repo);
    let regs = match Registries::load(&paths) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("uor-harness: {e}");
            return ExitCode::from(2);
        }
    };
    let now =
        uor_r4_harness::multilabrules::parse_claimed_at(&uor_r4_harness::receipt::now_rfc3339())
            .unwrap_or(0);
    // `--staged` swaps the rule SET, as Python's `scope=` does; it is not a filter over results.
    let scope = if args.staged {
        uor_r4_harness::checkdispatch::Scope::Staged
    } else {
        uor_r4_harness::checkdispatch::Scope::Repo
    };
    let findings = uor_r4_harness::checkdispatch::run_scoped(&paths, &regs, &args.only, scope, now);
    let (checked, failed, errors, warnings) = uor_r4_harness::checkdispatch::summarize(&findings);

    // The frozen formats, in the same shape `check-formats` prints.
    let contract_ok = match Contract::load(repo) {
        Ok(c) => c.check(repo).ok(),
        Err(e) => {
            eprintln!("uor-harness: {e}");
            false
        }
    };

    if json {
        let c = match Contract::load(repo) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("uor-harness: {e}");
                return ExitCode::from(2);
            }
        };
        let report = c.check(repo);
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
        print!(
            "{{\n  \"contract\": \"docs/harness/state-formats.json\",\n  \"schema\": {},\n  \"ok\": {},\n  \"formats\": [\n{}\n  ],",
            c.schema,
            report.ok(),
            rows.join(",\n")
        );
        print!(
            "\n  \"rules\": {{\n    \"checked\": {checked}, \"failed\": {failed}, \"errors\": {errors}, \"warnings\": {warnings},\n    \"findings\": ["
        );
        for (i, f) in findings.iter().enumerate() {
            let comma = if i + 1 == findings.len() { "" } else { "," };
            print!(
                "\n      {{\"rule\": {:?}, \"severity\": {:?}, \"ok\": {}, \"message\": {:?}, \"path\": {:?}}}{comma}",
                f.rule_id,
                f.severity,
                f.message == "ok",
                f.message,
                f.path
            );
        }
        println!("\n    ]\n  }}\n}}");
    } else {
        println!("{checked} rules, {failed} failed ({errors} error, {warnings} warn)");
        for f in findings.iter().filter(|f| f.message != "ok") {
            println!("  {} {} [{}]", f.severity, f.rule_id, f.path);
            println!("      {}", f.message);
            if !f.fix.is_empty() {
                println!("      fix: {}", f.fix);
            }
        }
    }
    // **`check` is one verdict: both halves decide it.**
    if contract_ok && errors == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
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
    // ONE source for the window. This was a local `const TTL_MINUTES: i64 = 30` while the presence
    // module and the heartbeat documented five, so this command told labs they had thirty minutes
    // and the check-in told them five - two answers to the same question, and the stale one was the
    // one a lab reads when deciding whether a peer is still working. Codex reported it after
    // onboarding. A test pins the two constants together.
    let ttl_minutes = uor_r4_harness::presence::PRESENCE_TTL_MINUTES;
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
            let stale = p.is_stale(now, ttl_minutes);
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
            "{{\n  \"live_tier\": {:?},\n  \"ttl_minutes\": {ttl_minutes},\n  \"labs\": [\n{}\n  ]\n}}",
            paths.live.display().to_string(),
            items.join(",\n")
        );
        return ExitCode::SUCCESS;
    }

    println!(
        "{} lab(s) known (stale after {ttl_minutes}m) · live tier: {}",
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

/// `ratchet d23|d25`: the D23/D25 interlocks, same scan and baselines as the Python scripts.
/// `lint check` / `lint format` — the D22/D23/D24 gate over a PR body.
///
/// **This is the command that makes the thirteen ported gates reachable.** Everything it needs is
/// now Rust: `check_pr` runs the gates, `format_pr_body` renders a compliant body, and the D23 scan
/// is the ratchet rather than the retired Python script.
///
/// **The two inputs `check_pr` takes as parameters are resolved HERE**, at the edge, so the gates
/// stay testable: `state_text` is read from the repo and `now` is the local date.
fn cmd_lint(repo: &Path, args: &[String], json: bool) -> ExitCode {
    use uor_r4_harness::prgates;
    use uor_r4_harness::state::StatePaths;

    // **An empty `--body ""` is not a body.** Python's `elif getattr(args, "body", None):` is a
    // FALSY check, so an empty string falls through to the "provide --body" error and exit 2. The
    // first version here treated the empty string as the body and linted it, which made the two
    // implementations disagree on the one input where "no body" and "an empty body" are the same
    // thing to a reader.
    let body = if let Some(file) = flag_value(args, "--body-file") {
        match std::fs::read_to_string(&file) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("uor-harness: cannot read {file}: {e}");
                return ExitCode::from(2);
            }
        }
    } else if let Some(inline) = flag_value(args, "--body").filter(|s| !s.is_empty()) {
        inline
    } else {
        // Python prints this bare, with no `uor-harness:` prefix - and a diff of the two
        // implementations is how this was found.
        eprintln!("provide --body or --body-file");
        return ExitCode::from(2);
    };

    let paths = StatePaths::resolve(repo, None, None);
    let regs = match uor_r4_harness::registries::Registries::load(&paths) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("uor-harness: {e}");
            return ExitCode::from(1);
        }
    };

    // `lint format` renders a compliant body and returns zero; it never gates.
    // **`format` is a SUBCOMMAND word, not a flag.** The dispatcher leaves it in `args` because
    // `lint` already claimed the command slot, so it arrives as `args[0]` - and the first version
    // only looked for `--action`, so `lint format` silently ran `check` and printed a summary where
    // Python printed a body. Caught by diffing the two commands rather than by reading either.
    let action = if args.iter().any(|a| a == "format") {
        "format".to_string()
    } else {
        "check".to_string()
    };
    if action == "format" {
        let f = uor_r4_harness::prbody::parse_pr_body(&body);
        // `print!`, not `println!`: `format_pr_body` already ends with a newline, exactly as Python's
        // does, and `println!` added a second - a blank line at the end of every rendered body, which
        // is invisible in a terminal and not invisible in a diff.
        print!("{}", prgates::format_pr_body(&f));
        return ExitCode::SUCCESS;
    }

    // The current-state entry is read here rather than inside the gate, so the gate is testable.
    let state_text = std::fs::read_to_string(repo.join("docs/integration/current-state.md")).ok();
    let now = uor_r4_harness::prep::today();
    let result = prgates::check_pr(&body, &paths, &regs, state_text.as_deref(), &now);

    if json {
        let problems: Vec<serde_json::Value> =
            result.problems.iter().map(|p| p.to_json()).collect();
        let doc = serde_json::json!({
            "ok": result.ok,
            "problems": problems,
            "summary": result.summary,
            "fields": result.fields,
        });
        println!("{}", serde_json::to_string_pretty(&doc).unwrap_or_default());
    } else {
        println!("{}", result.summary);
        for p in &result.problems {
            println!("  [{}] {}: {}", p.severity, p.rule, p.message);
            if !p.fix.is_empty() {
                println!("      fix: {}", p.fix);
            }
        }
    }
    if result.ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn cmd_ratchet(repo: &Path, args: &[String]) -> ExitCode {
    use uor_r4_harness::ratchet::{self, Kind, Update};
    let Some(kind) = args.iter().find_map(|a| Kind::parse(a)) else {
        eprintln!("uor-harness: ratchet needs `d23` or `d25`");
        return ExitCode::from(2);
    };
    let flag = |f: &str| args.iter().any(|a| a == f);
    let (what, rule, fix) = match kind {
        Kind::NormExp => (
            uor_r4_harness::ratchet::D23_WORD,
            "D23",
            "Replace it with a geometric mechanism,\nor get an owner decision; do not raise the baseline.",
        ),
        Kind::Matmul => (
            "dense matrix product call sites",
            "D25",
            "Replace it with a structured geometric\nproduct, or get an owner decision; do not raise the baseline.",
        ),
    };
    let current = match ratchet::scan(repo, kind) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("uor-harness: {e}");
            return ExitCode::from(2);
        }
    };
    let total = current.total();
    if flag("--list") && kind == Kind::Matmul {
        println!("dense matrix product call sites in code (D25):");
        for (path, sites) in &current.detail {
            for s in sites {
                println!("  {path}:{}: [{}] {}", s.line, s.kind, s.text);
            }
        }
        println!("  {total} sites in {} files", current.detail.len());
    }
    if flag("--update-baseline") {
        return match ratchet::update_baseline(repo, kind, &current.counts) {
            Ok(Update::Written {
                first,
                files,
                total,
            }) => {
                if first {
                    println!("first run: setting the ratchet at the current counts");
                }
                println!("baseline written: {files} files, {total} sites");
                ExitCode::SUCCESS
            }
            Ok(Update::Refused(raised)) => {
                eprintln!("REFUSED: --update-baseline may only lower counts. Raised:");
                for (p, o, n) in raised {
                    eprintln!("  {p}: {o} -> {n}");
                }
                eprintln!("Remove the new {what}, or justify it as an owner decision first.");
                ExitCode::from(1)
            }
            Err(e) => {
                eprintln!("uor-harness: {e}");
                ExitCode::from(2)
            }
        };
    }
    let baseline = match ratchet::load_baseline(repo, kind) {
        Ok(Some(b)) if !(kind == Kind::NormExp && b.is_empty()) => b,
        Ok(_) => {
            eprintln!("FAIL: no baseline. Run `uor-harness ratchet {} --update-baseline` once to set the ratchet.",
                if kind == Kind::NormExp { "d23" } else { "d25" });
            return ExitCode::from(1);
        }
        Err(e) => {
            eprintln!("uor-harness: {e}");
            return ExitCode::from(2);
        }
    };
    let grew = ratchet::growth(&current.counts, &baseline);
    let base_total: u64 = baseline.values().sum();
    if !grew.is_empty() {
        eprintln!("FAIL: {what} added to executable source ({rule} prohibits it):");
        for (p, b, n) in &grew {
            eprintln!("  {p}: {b} -> {n} (+{})", n - b);
        }
        eprintln!("\ntotal {base_total} -> {total}. {fix}");
        return ExitCode::from(1);
    }
    if !flag("--quiet") {
        println!(
            "OK: no {what} added. {total} remaining across {} files (baseline {base_total}).",
            current.counts.len()
        );
        let removed = ratchet::removed(&current.counts, &baseline);
        if removed > 0 {
            println!(
                "    {removed} removed since the baseline: run --update-baseline to lock it in."
            );
        }
        if kind == Kind::Matmul {
            println!(
                "    informational: {} tensor-core intrinsic sites (fragment and sync intrinsics) across {} files — not gated.",
                current.tensor_core, current.tensor_files
            );
        }
    }
    ExitCode::SUCCESS
}
