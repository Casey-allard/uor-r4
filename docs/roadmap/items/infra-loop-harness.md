---
id: infra-loop-harness
title: Build the loop harness (the loop subcommands of uor-harness)
milestone: infra
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: []
scope: [PLAN.md, docs/roadmap/items/*.md, crates/uor-r4-harness/src/loopreview.rs, crates/uor-r4-harness/src/loopnext.rs, crates/uor-r4-harness/src/loopreport.rs, crates/uor-r4-harness/src/loopclose.rs, crates/uor-r4-harness/src/loopclaim.rs, crates/uor-r4-harness/tests/loop_review*.rs, crates/uor-r4-harness/tests/loop_next*.rs, crates/uor-r4-harness/tests/loop_report*.rs, crates/uor-r4-harness/tests/loop_close*.rs, crates/uor-r4-harness/tests/loop_claim*.rs, crates/uor-r4-harness/tests/loop_repair_orders*.rs, crates/uor-r4-harness/tests/loop_shared_files*.rs, crates/uor-r4-harness/tests/loop_prcheck.rs, crates/uor-r4-harness/src/loopbench.rs, crates/uor-r4-harness/tests/loop_bench*.rs, docs/harness/d59.json, docs/harness/owner-paths.json, docs/integration/DECISIONS.md, AGENTS.md, CHARTER.md, docs/rules/changes.md, docs/mechanisms/README.md, docs/mechanisms/lut-serving.md, docs/mechanisms/hopf-observation.md, docs/mechanisms/spin-weights.md, docs/mechanisms/ternary-xor.md, crates/uor-r4-harness/src/loopdrive.rs, crates/uor-r4-harness/tests/loop_drive*.rs, crates/uor-r4-harness/src/prep.rs, crates/uor-r4-harness/src/preprules.rs, xtask/src/docs_links.rs, xtask/src/archive_verify.rs, xtask/src/figures_charts.rs, docs/figures/**, docs/roadmap/issues.json, docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md, docs/harness/labs.json, crates/uor-r4-harness/src/labs.rs, crates/uor-r4-harness/src/receipt.rs, crates/uor-r4-harness/tests/lab_registry_cli.rs, crates/uor-r4-harness/src/onboard.rs, crates/uor-r4-harness/src/loopguard.rs, crates/uor-r4-harness/tests/loop_guard.rs, archive/history/branch-archive/codex-d63-finite-probe-scope-20261026.*]
---

## Why it matters

Four labs from four vendors work on UOR-R4 at once, with nobody watching each step. There were five until Codex left on 26 Oct 2026; see `docs/harness/labs.json`. Records that drifted out of date have cost the project more than any other failure.

The loop gives every unit of work:
- a written order with a clear scope;
- gates anyone can rerun to prove "done";
- an exact-head review. Since D66 (24 Oct 2026) that is the lab's own fresh-context review; before that it was another vendor's, and Claude's fresh-context reviewers during the setup (D59).

It also renders the shared status pages from committed inputs, so nobody edits them by hand. See [the loop page](../../loop/README.md). It is adapted from the Loop Kit by Ron Brazil.

## What is missing

**For the build: nothing,** as of the claude-139 audit (27 Oct 2026).
- Every `loop …` subcommand on the build plan is on `main`: `next`, `arrive`, `claim`, `gates`, `render`, `review`, `queue`, `close`, `drive`, `guard`, `install` and `onboard`.
- So are `docs/harness/labs.json`, `gates.json`, the rendered STATE and HANDOFF, and the policy job.
- Items carry `scope:` and "Cancel if".
- Each build-plan PR's planted fault is held by a test that passes on `main`. Row 5's "one fixture per rule" is shown by test names.
- The four labs each have a committed onboarding receipt and a merged, verified loop unit.

See `docs/loop/tasks/infra-loop-harness/claude-139/REPORT.md`.

The policy gate stays advisory on GitHub, because there is no org admin to add a GitHub App or change the ruleset. So the supervisor's local queue step is the lock.

**This item is also the maintenance umbrella.** Its `scope:` holds `PLAN.md`, `docs/roadmap/items/*.md`, the mechanism pages and other shared records. Labs claim it for item-file maintenance (scope fixes, new items) and for the records around the loop, as in claude-106, -108, -121, -137 and -138 and antigravity-80 and -109. No other item covers those paths.

## Done when

Every PR in the build plan on [the loop page](../../loop/README.md#build-plan) is merged, each with its planted fault turning its gate red. The four labs (Kimi, Claude, DeepSeek, Antigravity) are onboarded, each with a committed onboarding receipt. Each lab has completed one loop task end to end: claim, probe, work, gates, report, review by another vendor, queue, and verify on fresh `main`.

## How to start

**For a maintenance unit now:** claim this item with `uor-harness loop claim infra-loop-harness --lab-topic <topic>`, and narrow the ORDER's scope to the files the unit touches. Shared registration files never go in `scope:` (`loopclaim.rs`, `SHARED_FILES`), because every loop PR may edit them.

**How the harness was built, kept as history.** Steps 1 and 2 describe the setup as it ran. D59 ended on 22 Oct and D66 replaced cross-vendor review on 24 Oct.

1. Read [the loop page](../../loop/README.md) and CHARTER §5.10. PR A (the charter change, this item and the page) must be in every build PR's base; check by hand until PR 5, then by the gate.
2. Team split: Claude leads the architecture, with Kimi as PM reviewing. Codex and DeepSeek build PRs 2–7 and 10–12. Antigravity builds PR 9. Every review comes from a different vendor than the author, with one exception. Until the other labs run the loop, each setup PR gets its exact-head review from Claude's fresh-context reviewers, whichever lab built it, and no other lab, Kimi included, is asked (D59). While Claude runs alone, Claude also builds every setup PR, and each other client's adapter is added when that lab joins (D52, amended).
3. Run PR 0 first: probes P-1 (hooks headless and inside subagents, per client), P-2 (headless launch from a supervisor), P-3 (tags survive the squash merge), P-4 (the policy result can feed a required check) and the optional P-6. Write each cancel line before running the probe. P-5 is dropped: Codex keeps full access.
4. Then the PRs in order: 1 (items for PRs 2–12, `labs.json`, `owner-paths.json`, `rule-sources.json`); 2 (item keys and validator); 3 (`loop render`); 4 (`loop gates` with breaks); 5 (`loop pr-check` and `gate.yml`, policy advisory); 6 (`loop review` with tagged review records); 7a (`claim`, `suborder`, `ask`); 7b (`next`, `arrive`, `report`, `close`); 7c (`queue`, `board`); 8 (migration of the notes); 9 (`guard`, `install`, `onboard`, local copyright check); 10 (`drive`, watchers, kill switch, soft 2-hour check-in); 11 (`--watch`); 12 (git hooks and trailers).
5. Each PR names one planted fault that must turn its gate red, carries an ORDER and REPORT in `docs/loop/tasks/infra-loop-harness/<lab>-<n>/`, and is built and tested on the laptop: `~/.cargo/bin/cargo build -p uor-r4-harness`, `~/.cargo/bin/cargo test -p uor-r4-harness` and `uor-harness check`.
6. Open each PR with `References #<this issue>`; the last PR says `Closes #<this issue>`.

## Rules that apply

- **Rust first** (CHARTER §3.11): the harness is Rust, and no new Python is added.
- **Delivery path** (§5.5): PR, the lab's own fresh-context exact-head review in the same session (D66), merge queue, verify on fresh `main`, then delete the branch and worktree. Other labs audit after the merge.
- **Owner-only boundaries** (§5.1, §5.2): the charter, owner-only registries, caps and releases stay the owner's. An owner release is approved in the lab's own chat and recorded verbatim with the time.
- **The kit's text stays private**: write in our own words; the copyright check runs locally only.

## Sources

- [`docs/loop/README.md`](../../loop/README.md)
- [`CHARTER.md`](../../../CHARTER.md)
