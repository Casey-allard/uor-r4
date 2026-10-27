# ORDER: infra-loop-harness, claude-138 (take codex out of the lab registry)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 1 (the lab registry and its no-identity hint). Base: `9fdc7d4e9`.

## Goal

**Take codex out of the lab registry, at the owner's instruction (below).** Codex works independently on lexlean-r4 and is no longer part of the UOR-R4 loop. claude-137 (#3307) widened this item's scope for it, and recorded the receipt-gate decision (D68, #2904 owner log).

**The edits.**
1. **The registry.** `docs/harness/labs.json` lists four labs: antigravity, claude, deepseek and kimi. Its note says "four". This is the owner's edit. The guard refuses every lab write to an owner-only path, approval or not, so Casey makes it on this branch.
2. **The tests and hints that read the real registry** move to four labs, and the harness refuses `codex`.
   - `labs.rs` tests.
   - `receipt.rs`: the no-identity hints read `UOR_LAB=<antigravity|claude|deepseek|kimi>`.
   - `tests/lab_registry_cli.rs`, which holds the hints to that exact list.
   - The `onboard.rs` unknown-lab test.
3. **The receipt gate follows the registry (the D68 decision in claude-137).**
   - `check_all_receipts` walks the registry's labs, not `CLIENTS`. Each lab must be a known client with a valid receipt.
   - `CLIENTS` keeps codex's adapter.
   - `tests/loop_guard.rs`: the fixture registers every client it drives in its own copy of the registry, so codex's adapter keeps its coverage. The real-repo receipt test expects the registry's labs.
4. **Docs.**
   - `docs/harness/README.md`: `--check-receipts` covers every registered lab.
   - `docs/loop/README.md`: the title, the opening line and "## The five labs" table. The Codex row becomes a dated note; probe and history rows stay as history.
5. **The archive.** Codex's closed #3169 is kept byte-identically as `archive/history/branch-archive/codex-d63-finite-probe-scope-20261026.patch`, with a record file beside it. `INDEX.md` is immutable (CHARTER §3.12).
6. **The item's scope loses two globs.** claude-137 added `docs/harness/README.md` and `docs/loop/README.md` to `scope:`. Both are shared registration files (`loopclaim.rs`, `SHARED_FILES`), which any loop PR may edit. `loop claim` refuses an item whose scope lists one. Removing them changes nothing about what this unit may write. This is a D68 scope fix, and the fresh review covers it as a decision review.
7. **`loop drive --client codex`.** Confirm that it is refused once codex leaves the registry. Its receipt check only tests that `docs/harness/onboarding/codex.json` exists (`loopdrive.rs:1504`), and that file stays as history. If it is not refused, add the refusal in `loopdrive.rs`, with a test.

**Left for the owner:**
- `AGENTS.md`'s `UOR_LAB=<antigravity|claude|codex|deepseek|kimi>`, an owner path his instruction did not name;
- the item's title and "Why it matters" prose ("five-lab").

## Owner approval

Casey, in claude's chat, received at about 2026-10-26T21:34Z, naming `docs/harness/labs.json`:

> "Yes: close codex's PR #3169 with a note, and open a PR removing codex from docs/harness/labs.json, quoting this instruction in its ORDER."

It answered claude's question "Close codex's #3169 and take codex out of the lab registry?", after Casey wrote: "codex is not currently working with the rest of the groups, it is working independently on lexlean-r4 so it should not be included."

## Cancel probe (run first)

Cancel if codex is already out of the registry, or another open PR changes `docs/harness/labs.json`, the item file or the files below.

Result (2026-10-26T22:50Z, `origin/main` at `9fdc7d4e9`):
- `docs/harness/labs.json` still lists codex.
- The open PRs are #3306 (kimi, `d11-exporter-float-trunk`) and #3308 (antigravity, `m1-close-saved-model`). Neither touches these files.

**The claim was refused, and this ORDER is hand-written.** `loop claim infra-loop-harness --lab-topic drop-codex-registry` refused with: "scope glob `docs/harness/README.md` covers the shared file docs/harness/README.md; Casey (21-22 Oct, …) made the nine shared registration files open to every loop PR, so no item scope may list them".
- That glob is claude-137's own slip. It passed `xtask roadmap check` and two fresh reviews.
- Edit 6 removes both shared files.
- No other order holds the item.

**Not cancelled (instrument: the claim refusal is the lock reading a malformed scope; repaired here).**

## Files in scope

All inside the item's scope on `origin/main`, or always writable (the shared files, the item file, this task folder):
- `docs/harness/labs.json` (Casey's edit)
- `crates/uor-r4-harness/src/labs.rs`
- `crates/uor-r4-harness/src/receipt.rs`
- `crates/uor-r4-harness/tests/lab_registry_cli.rs`
- `crates/uor-r4-harness/src/onboard.rs`
- `crates/uor-r4-harness/src/loopguard.rs`
- `crates/uor-r4-harness/tests/loop_guard.rs`
- `crates/uor-r4-harness/src/loopdrive.rs` and `crates/uor-r4-harness/tests/loop_drive*.rs`, only if edit 7 needs them
- `archive/history/branch-archive/codex-d63-finite-probe-scope-20261026.*`
- `docs/harness/README.md` and `docs/loop/README.md` (shared files)
- `docs/roadmap/items/infra-loop-harness.md` (the `scope:` line)
- this task folder

## Done when

1. **The registry lists four labs,** and the harness refuses `codex` as an unregistered lab name (labs.rs test, CLI test, onboard test).
2. **The receipt gate walks the registry.** `uor-harness loop onboard --check-receipts` verifies four receipts on this branch. A test fails if `check_all_receipts` goes back to walking `CLIENTS`.
3. **Tests.** `cargo test -p uor-r4-harness --no-fail-fast` passes, apart from tests that fail the same way on `origin/main` from a lab worktree; the REPORT names those. `loop gates --full` is green.
4. **The archive patch reproduces #3169's tree** `8f0f86882ae254f21b548836ab85dd955f74e77c` from base `71d9247a87ba45588ea00a583d5321ae2a3d6082` in an isolated index, and its record file says so.
5. **Checks.**
   - `loop claim` accepts the item's scope again: `loop claim --dry-run`, or `loop next` no longer reports the shared-file refusal.
   - `xtask roadmap check` passes.
   - `loop pr-check --local` passes, including `owner-paths`.
6. **A REPORT** with `References #2904`.

## Stop and ask if

Stop and decide; never wait.
- **Casey has not made the registry edit** when everything else is ready. Post one "[claude, for Casey]" note on #2028 with the exact command, keep the PR a draft, and take other work. Only this item parks.
- **A test needs codex registered** for a reason other than the registry's lab list. Register codex in that test's own fixture copy, and never in the real file.
