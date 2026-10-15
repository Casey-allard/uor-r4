# Harness write path: R10 enforced on the tool, the `unknown` fallback, the tracker watch, and the claims split brain

Date: 2026-10-15. Lab: deepseek. Line: no model line - tooling. Task source: Kimi PM,
[#2028 comment 6193334352](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6193334352)
(five-item harness queue). No model compute, no GPU, no pod, no headline change.

## What was wrong

**1. The tool that enforces R10 was breaking R10.** R10 keeps the owner checkout (`~/uor-r4`) on clean
`origin/main`, and the harness writes were the thing dirtying it. `receipt`, `skill use` and `prep fit`
called `receipt::append` and appended straight into `docs/harness/ledger/<lab>-<month>.jsonl` of whatever
checkout the working directory named; `team claim`/`release` wrote `docs/harness/claims.json` there;
`ledger-sync`, `multilab sync-live`, `prep new` and `mistakes capture` the same. A lab whose shell sat in
the owner checkout therefore dirtied it on a coordination write, and another lab hand-moved the entries out
before the checkout could be reset. Heartbeat was already live-tier only (R10-clean) - the receipts were not.

**2. An unset lab name wrote nameless files.** Python `slug()` ends `return s or "unknown"`; the Rust port
returned the empty string, so an unattributed record landed in `.json` / `-2026-10.jsonl` - files no reader
looks for, holding a record that looks attributed. `receipt::shard_path` had patched around it for the
ledger alone; presence, the note marker and the note cache had not. The CLI also *refused* an unnamed
receipt (exit 2), which contradicts the documented contract ("records it, warns loudly") and loses the
record rather than surfacing the gap.

**3. The check-in watched every milestone and not the tracker.** `notes::watched_issues` returned the
milestone registry only, so #2028 - the roadmap issue, where the PM posts lab assignments and where this
very queue arrived - was never in the unread report. Measured: the heartbeat printed eight milestones and
silently omitted the issue that had just tasked the lab.

**4. The shared claims tier was a stale working copy.** `load_claims` read the "shared" tier from the
primary worktree's `docs/harness/claims.json` - the one checkout R10 keeps frozen, and therefore the one
most likely to be behind. Measured from the owner checkout: `multilab claims` listed 5 branches including
`codex/d19-object-store-impl`, which neither the live tier nor `origin/main` named any more; the command was
reporting the checkout's staleness as if it were a claim.

**5. Two installed copies, one of them a day stale.** `~/.local/share/uor-r4/bin/uor-harness` is a symlink
into the owner checkout's release build, built `2026-10-14T16:59:30Z`; `~/.cargo/bin/uor-harness` (what
`which uor-harness` finds) was built `2026-10-15T13:38:00Z`, after every harness commit on `origin/main`.

**6. `ledger-sync --dry-run` wrote.** Found while delivering this change: the flag is documented on the
command ("Use --dry-run to see what would be written") and `cmd_ledger_sync` never read it, so the dry run
appended 101 receipts to the branch's committed shard. A documented read-only flag that writes is worse
than a missing flag: it teaches a lab the wrong thing about which commands touch a checkout.

**7. The linter read its phrase table from the worktree it was BUILT in.** Reported by Codex on #2508
([6194024391](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6194024391)): `uor-harness lint
check` reported a complete PR body as missing its pre-registration, and the same source reinstalled from a
retained checkout reported `OK - 0 error, 0 warn`. `prbody.rs` read
`env!("CARGO_MANIFEST_DIR")/rules/prlint-phrases.json` at run time and fell back to `Value::Null`, so every
phrase became empty and the preregistration markers disappeared. **Labs install from a worktree and routine
cleanup deletes it**, so the defect re-arms on every merge train. Measured: the copy on PATH is built in
`.worktrees/codex-harness-main` and carries that path; the directory exists today and will not after cleanup.

## What changed

| file | change |
|---|---|
| `crates/uor-r4-harness/src/state.rs` | the shared checkout is identified by shape **plus `git config uor.sharedCheckout true`**, which survives the cleanup that removes every linked worktree; `git_out` clears `GIT_DIR`/`GIT_WORK_TREE`; `slug()` gains Python's `unknown` fallback; new `slug_stem()` keeps "did a lab name itself" a separate question; new `shared_primary_checkout()` and `refuse_shared_checkout()` identify the shared checkout by the shape of the clone |
| `crates/uor-r4-harness/src/receipt.rs` | committed writes are refused in the shared checkout; `Receipt::validate` no longer refuses an unnamed lab; `unattributed()` + `UNATTRIBUTED_WARNING` (Python's wording, once per process); shard names use `slug()` directly |
| `crates/uor-r4-harness/src/team.rs` | `save_claims` refuses the shared checkout; the shared tier is read from `origin/main` through `git show`, not from a working copy; the shared checkout no longer supplies the caller's own tier |
| `crates/uor-r4-harness/src/presence.rs` | `publish_durable` refuses the shared checkout; `Presence` keeps the raw lab so an unnamed check-in is still refused while the record and paths carry the slug |
| `crates/uor-r4-harness/src/multilab.rs` | `migrate_ledger` and `sync_live_to_durable` refuse the shared checkout — the sync refuses **before** it reads the live tier, because the per-row guard only fired when there was a row to publish |
| `crates/uor-r4-harness/src/ratchet.rs`, `githelp.rs` | `ratchet --update-baseline` and `git install-hooks` refuse the shared checkout; `state::refuse_shared_checkout_at` is the path-based entry point for writers that hold a repository, not the harness state |
| `crates/uor-r4-harness/src/receipt.rs` (2) | `record` (`team dispatch`) writes the LIVE shard like the other coordination receipts, so the live-first rule the docs state is true of every receipt |
| `crates/uor-r4-harness/tests/committed_writes_refuse_the_shared_checkout.rs` | **the entry-point pin**: a real shared-layout clone, the real binary, every committed-write command asserted to refuse with R10 and to leave the checkout unchanged, the same commands asserted NOT to be refused in a lab worktree, the dry-run dispatch pinned, the marker case, and the hook-environment case |
| `crates/uor-r4-harness/src/main.rs` | `receipt`, `skill use` and `prep fit` write the LIVE shard; `cmd_receipt` warns instead of refusing an unnamed lab; one warning text for all writers; `ledger-sync --dry-run` is honoured (the write now sits behind `plan_sync`) |
| `crates/uor-r4-harness/src/receipt.rs` | `plan_sync` is the read half of the sync: what would be appended, and the lines, writing nothing |
| `crates/uor-r4-harness/src/main.rs` (2) | `prep new` (the D22 sheet) and `mistakes capture` (the lesson) refuse the shared checkout too - both are record files that ride the delivery PR |
| `crates/uor-r4-harness/src/notes.rs`, `registries.rs` | `watched_issues` puts the tracker first, then the milestones; `Registries::tracker_issue()` reads it from the registry |
| `crates/uor-r4-harness/src/prbody.rs`, `main.rs` | the phrase table is **compiled into the binary** (`include_str!`); a checkout copy is preferred when it parses, and a missing or malformed one is skipped rather than accepted, so the table can never come back empty. `main` points the parser at the `--repo` registry once per process, so editing the registry still takes effect without a rebuild |
| `scripts/harness/parity.py` | the Rust-writes/Python-reads probe now follows the two-step path: it points `UOR_HARNESS_LIVE` at a scratch directory, asserts `receipt` wrote nothing into the checkout, runs `ledger-sync`, then reads the committed shard through Python. Without this the probe asserted the old contract and CI failed on it (run `38747932047`), which is the parity job doing its job |
| `AGENTS.md`, `CONTRIBUTING.md`, `docs/labs/session-goal.md` | the same correction as the README: a receipt is live-tier first and reaches the committed shard through `ledger-sync` |
| `docs/harness/README.md`, `pm-rules.md`, `BOOTSTRAP.md` | the write-path table, the enforced-R10 refusal, the tracker in the check-in, the two installed copies; the presence TTL in the README corrected from 30 to 5 minutes (the code has used 5 since 13 October) |

## Verification

Exact head: this branch. `CARGO_TARGET_DIR=local/harness-target cargo test -p uor-r4-harness --offline`:
**533 passed, 0 failed, 6 ignored**, plus **5 entry-point integration tests**. New tests cover the guard (shared checkout found, linked worktree
writable, subdirectory still the shared checkout, single-checkout clone never refused), the refusal message,
`slug`/`slug_stem`, the unnamed receipt landing in `unknown-<month>.jsonl`, the tracker in `watched_issues`,
the shared tier reading `origin/main` rather than a stale working copy, and a claim in a lab's own checkout
staying visible to it before the PR.

End-to-end with the built binary, run **from the owner checkout** (the failing case):

```
$ uor-harness receipt --kind harness_test --note "R10 e2e probe"      # exit 0
receipt recorded: ~/.local/share/uor-r4/harness/ledger/deepseek-2026-10.jsonl in 2026-10-15T13:51:08Z
  live tier - `uor-harness ledger-sync` carries it into the committed shard from your own worktree
$ uor-harness ledger-sync                                            # exit 1, refused
uor-harness: refusing to write the committed ledger shard into the shared checkout /Users/casey.allard/uor-r4 (R10): ...
$ uor-harness team claim --branch kimi/probe-r10 --lab kimi           # exit 2, refused
team claim: refusing to write the claims store into the shared checkout /Users/casey.allard/uor-r4 (R10): ...
$ git -C ~/uor-r4 status --porcelain -- docs/harness                  # empty: the checkout stayed clean
```

`ledger-sync --dry-run`, same binary, on a shard holding one unsynced live receipt: "would append 1
receipt(s) ... nothing written", the committed file's sha256 **unchanged** (`046ca5c3…` before and after),
and the real sync then appended exactly that one line. Before the fix the same dry run appended 101 lines.

Unset lab, same binary: `env -u UOR_LAB uor-harness receipt --kind harness_test_unattributed` warns in
Python's words and records into `unknown-2026-10.jsonl` with `"lab": ""`. Tracker watch, same binary:
`uor-harness notes --lab deepseek` and the check-in now report `#2028` first
(`272 unread note(s) on #2028 (latest: 6193619622, ...)`). Claims tiers, measured: from the owner checkout the
board reports the live mirror plus `origin/main`'s committed store; the stale `codex/d19-object-store-impl`
entry from that checkout's working copy no longer appears.

`uor-harness check` at the head: **0 errors**; the 10 warnings are pre-existing or other labs' (a kimi
claim already in main, kimi presence stale, one historic unattributed row, three PENDING prep sheets,
unfrozen m5-m8 panels). CI on the head: the `harness` workflow (harness rules and tests, plus Rust/Python
parity - which caught the old two-step assumption as a build failure first, run `38747932047`) and the `ci`
workflow are **green**.

## Review findings, fixed

The exact-head review (Kimi PM, dispatched reviewer) returned CHANGES REQUESTED on two narrow points; both are
fixed at the reviewed head:

- **D1 — the README row for `status` was false.** It claimed section 2 reads the committed shards *plus the
  live shard*; `status.rs` reads committed shards only (47 receipts for the day against 360 in the live
  tier). The row now says so, and the write-path section states plainly that a receipt reaches the `status`
  roll-up through `ledger-sync` and the merge, while live-tier readers see it immediately. The alternative
  the reviewer offered - overlaying the live tier in `read_all_shards` - was not taken: `status` is compared
  against Python's by the parity job, and Python reads committed shards, so an overlay would have to change
  both implementations to stay honest.
- **D2 — `ratchet --update-baseline` wrote a tracked file with no R10 guard**, which falsified "every
  remaining committed write refuses the shared checkout". It now refuses through a path-based entry point
  (`state::refuse_shared_checkout_at`), so a writer holding a repository rather than the harness state can be
  guarded too. Verified live: `ratchet d23 --update-baseline` from `~/uor-r4` exits 2 with the refusal and
  `scripts/` stays clean, while `ratchet d23 --list` still reads. `git install-hooks` carries the same guard
  (a lab running it from its own worktree still installs into the shared hooks directory, which is how every
  worktree is guarded).

A second exact-head review (Claude, on `e50865f04`) returned CHANGES with one medium finding and one test-quality
finding; both are fixed:

- **F1 — the guard switched off when the owner checkout had no linked worktrees.** The shape rule read any
  single checkout as the writer's own, and `uor-hygiene`/the post-merge cleanup remove worktrees, so `~/uor-r4`
  could reach that state and accept `team claim` again. Reproduced in review on a single-checkout scratch repo.
  Fixed with a marker that does not depend on the worktree count — `git config uor.sharedCheckout true`, set once
  in the shared checkout — checked alongside the shape. A linked worktree of a marked checkout stays writable, and
  an unmarked single-checkout clone is still a lab's own (both pinned).
- **F4 — the guards were not pinned.** Deleting the `ledger-sync` guard left all 533 unit tests green. There is now
  an entry-point integration test (`tests/committed_writes_refuse_the_shared_checkout.rs`) covering every
  committed-write command. **Every pin was verified by removing its guard and watching the test fail:**

  | entry point | guard removed → test |
  |---|---|
  | `team claim` | fails |
  | `ledger-sync` | fails |
  | `multilab sync-live` | fails |
  | `multilab migrate-ledger` | fails |
  | `prep new` | fails |
  | `mistakes capture` | fails |
  | `ratchet --update-baseline` | fails |
  | `git install-hooks` | fails |
  | the marker/shape identification itself | fails |
  | `receipt::append` (no CLI entry point; library API + defense in depth) | not covered — stated, not hidden |

- **F5 — a hook environment could make a lab worktree look shared.** A git hook exports `GIT_DIR`/`GIT_WORK_TREE`,
  which take precedence over `git -C`, so inside a hook `rev-parse --show-toplevel` reports the MAIN worktree and a
  lab's own write would have been refused for the wrong reason. `git_out` now clears both. Pinned by running
  `team claim` from a linked worktree with `GIT_DIR` set.

Non-blocking notes from the reviews, recorded rather than fixed: the unnamed-check-in exit code differs by path
(2 and 1); the claims view does not yet say which tier answered or how old `origin/main` is (F2); an unattributed
receipt's shard says `unknown` while its `"lab"` stays `""`, matching the retired Python (F3); `receipt::append`
and `write_line` remain the committed-write API with no CLI caller (the retired Python path still uses its own).
Two delivery-time observations worth keeping: `cargo` on `PATH` on this machine is **not** Rust's cargo
(`/opt/homebrew/bin/cargo` is a different tool), so anything invoking `cargo` by name — including `parity.py`'s
`build()` fallback — can silently do nothing; and a probe script that used `git checkout -- <file>` to restore a
guard also reverted two uncommitted edits, which is why the pin probe now runs against committed state only.

## What this does not fix

- **`scripts/harness/*.py` keeps the old behaviour** (`receipts.record` writes the committed shard, `multilab.claims_file` reads the working copy). The deployed path is the Rust binary - `check` is the only rule engine still delegated - so the Python side is a stale reference rather than a live writer, but a lab that calls `cli.py receipt` directly can still dirty a checkout.
- **The symlinked `bin/` copy is still a day stale**, and refreshing it requires a build in the owner checkout (R10). Instructions are published; the operational repoint is recorded as the next action.
- **A release is still visible as held until its PR merges** for a lab whose own store does not name the branch: the committed tier is the arbiter by design (a claim stamps two tiers with one timestamp, so "newest" cannot choose). The change removes the *stale working copy* from that path; it does not make release real-time.
- **The four-item queue's item 3 was already implemented** by #2497 (markers in the live tier, `--mark-seen`, heartbeat surfacing); what was missing was the tracker. No new mechanism was added there.

## Cost

Preparation, source edits, documentation and local test runs: laptop CPU only, no pod, no GPU. Two
`cargo test` runs of the harness crate (35 s each) plus incremental `cargo check`/`cargo build`; no new
storage class, no new dependency, no schema change. Ledger/receipts: one live receipt per command run;
the committed shard is unchanged by this PR except where the record is added by the normal delivery.