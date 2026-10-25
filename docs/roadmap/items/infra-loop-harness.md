---
id: infra-loop-harness
title: Build the five-lab loop harness (the loop subcommands of uor-harness)
milestone: infra
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: []
scope: [PLAN.md, docs/roadmap/items/*.md, crates/uor-r4-harness/src/loopreview.rs, crates/uor-r4-harness/src/loopnext.rs, crates/uor-r4-harness/src/loopreport.rs, crates/uor-r4-harness/src/loopclose.rs, crates/uor-r4-harness/src/loopclaim.rs, crates/uor-r4-harness/tests/loop_review*.rs, crates/uor-r4-harness/tests/loop_next*.rs, crates/uor-r4-harness/tests/loop_report*.rs, crates/uor-r4-harness/tests/loop_close*.rs, crates/uor-r4-harness/tests/loop_claim*.rs, crates/uor-r4-harness/tests/loop_repair_orders*.rs, crates/uor-r4-harness/tests/loop_shared_files*.rs, crates/uor-r4-harness/tests/loop_prcheck.rs, crates/uor-r4-harness/src/loopbench.rs, crates/uor-r4-harness/tests/loop_bench*.rs, docs/harness/d59.json, docs/harness/owner-paths.json, docs/integration/DECISIONS.md, AGENTS.md, CHARTER.md, docs/rules/changes.md, docs/mechanisms/README.md, docs/mechanisms/lut-serving.md, docs/mechanisms/hopf-observation.md, docs/mechanisms/spin-weights.md, docs/mechanisms/ternary-xor.md, crates/uor-r4-harness/src/loopdrive.rs, crates/uor-r4-harness/tests/loop_drive*.rs, crates/uor-r4-harness/src/prep.rs, crates/uor-r4-harness/src/preprules.rs, xtask/src/docs_links.rs, xtask/src/archive_verify.rs, xtask/src/figures_charts.rs, docs/figures/**, docs/roadmap/issues.json, docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md]
---

## Why it matters

Five labs from five vendors work on UOR-R4 at once, with nobody watching each step. Records that drifted out of date have cost the project more than any other failure. The loop gives every unit of work a written order with a clear scope, proves "done" with gates anyone can rerun, needs a review from another vendor at the exact head (during the setup, from Claude's fresh-context reviewers; D59), and renders the shared status pages from committed inputs so nobody edits them by hand. See [the loop page](../../loop/README.md). It is adapted from the Loop Kit by Ron Brazil.

## What is missing

None of the loop tooling exists. No `loop …` subcommand of `uor-harness` is built: not `next`, `arrive`, `claim`, `gates`, `render`, `review`, `queue`, `close`, `drive`, `guard`, `install` or `onboard`. There is no `docs/harness/labs.json`, no `gates.json`, no rendered STATE or HANDOFF, and no policy job in CI. Roadmap items carry no `scope:` or "Cancel if" yet. Today labs follow the loop by hand: an ORDER and REPORT in `docs/loop/tasks/`, and an exact-head review posted in free text, which for the setup PRs comes from Claude's fresh-context reviewers (D59). The policy gate will stay advisory on GitHub, because there is no org admin to add a GitHub App or change the ruleset, so the supervisor's local queue step is the lock.

## Done when

Every PR in the build plan on [the loop page](../../loop/README.md#build-plan) is merged, each with its planted fault turning its gate red. The five labs (Kimi, Claude, Codex, DeepSeek, Antigravity) are onboarded, each with a committed onboarding receipt. Each lab has completed one loop task end to end: claim, probe, work, gates, report, review by another vendor, queue, and verify on fresh `main`.

## How to start

1. Read [the loop page](../../loop/README.md) and CHARTER §5.10. PR A (the charter change, this item and the page) must be in every build PR's base; check by hand until PR 5, then by the gate.
2. Team split: Claude leads the architecture, with Kimi as PM reviewing. Codex and DeepSeek build PRs 2–7 and 10–12. Antigravity builds PR 9. Every review comes from a different vendor than the author, with one exception. Until the other labs run the loop, each setup PR gets its exact-head review from Claude's fresh-context reviewers, whichever lab built it, and no other lab, Kimi included, is asked (D59). While Claude runs alone, Claude also builds every setup PR, and each other client's adapter is added when that lab joins (D52, amended).
3. Run PR 0 first: probes P-1 (hooks headless and inside subagents, per client), P-2 (headless launch from a supervisor), P-3 (tags survive the squash merge), P-4 (the policy result can feed a required check) and the optional P-6. Write each cancel line before running the probe. P-5 is dropped: Codex keeps full access.
4. Then the PRs in order: 1 (items for PRs 2–12, `labs.json`, `owner-paths.json`, `rule-sources.json`); 2 (item keys and validator); 3 (`loop render`); 4 (`loop gates` with breaks); 5 (`loop pr-check` and `gate.yml`, policy advisory); 6 (`loop review` with tagged review records); 7a (`claim`, `suborder`, `ask`); 7b (`next`, `arrive`, `report`, `close`); 7c (`queue`, `board`); 8 (migration of the notes); 9 (`guard`, `install`, `onboard`, local copyright check); 10 (`drive`, watchers, kill switch, soft 2-hour check-in); 11 (`--watch`); 12 (git hooks and trailers).
5. Each PR names one planted fault that must turn its gate red, carries an ORDER and REPORT in `docs/loop/tasks/infra-loop-harness/<lab>-<n>/`, and is built and tested on the laptop: `~/.cargo/bin/cargo build -p uor-r4-harness`, `~/.cargo/bin/cargo test -p uor-r4-harness` and `uor-harness check`.
6. Open each PR with `References #<this issue>`; the last PR says `Closes #<this issue>`.

## Rules that apply

- **Rust first** (CHARTER §3.11): the harness is Rust, and no new Python is added.
- **Delivery path** (§5.5): PR, exact-head review (for the setup PRs, from Claude's fresh-context reviewers under D59), merge queue, verify on fresh `main`, then delete the branch and worktree.
- **Owner-only boundaries** (§5.1, §5.2): the charter, owner-only registries, caps and releases stay the owner's. An owner release is approved in the lab's own chat and recorded verbatim with the time.
- **The kit's text stays private**: write in our own words; the copyright check runs locally only.

## Sources

- [`docs/loop/README.md`](../../loop/README.md)
- [`CHARTER.md`](../../../CHARTER.md)
