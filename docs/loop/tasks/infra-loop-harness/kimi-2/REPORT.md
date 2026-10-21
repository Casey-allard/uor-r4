# REPORT: infra-loop-harness, kimi-2 (repair: un-serialize the scope lines)

Lab: kimi. Order: folded into [kimi-3/ORDER.md](../kimi-3/ORDER.md) as history; this folder's original ORDER.md was removed so the PR adds exactly one ORDER, which `loop review --check` and `loop pr-check` both require.

## What did not work, first

- kimi-1's scopes failed review: the counted CHANGES record at `fa28f9339` (claude, uor-review v1) found the loop items all listed `crates/uor-r4-harness/**`, `docs/loop/**`, `docs/harness/**` and `DECISIONS.md`, and about 38 non-loop items shared `docs/labs/**`, `docs/harness/prep/**` or `docs/integration/NOW.md`. Because `globs_overlap` treats prefix globs as overlapping, `loop claim` would have refused nearly every parallel claim — the scopes unblocked claiming in name only. Every point is answered in this repair.
- The kimi-1 REPORT claimed `src-tauri/**` exists at origin/main; it does not, and neither does `models/**`. Corrected in `docs/loop/tasks/infra-loop-harness/kimi-1/REPORT.md` (both are new folders named prospectively).
- `docs/integration/project-track.md` was dropped from the six items that listed it, beyond the letter of the review: it serializes claims exactly like `NOW.md`, which the review named. Flagged here so the reviewer can disagree.
- `loop pr-check --local` cannot pass on any same-branch repair, by construction: the PR then adds two ORDERs (`order`, `scope`, `commit-order`, `report` rules), and kimi-1's commits changed paths before kimi-2's ORDER commit (`commit-order`). The repair stays on this branch because the review record said to ("post a new review request with the new head"); the gate that counts for the manual merge today is the `uor-review v1` record at the head, and CI's advisory policy job will record `policy: fail` for the same structural reason while the required checks stay green (P-4). Raised on #2028 for the architect.

## What changed

- HIGH 1: the eight loop items now name only their own modules and tests (for example pr09: `src/loopguard*.rs`, `src/loopinstall*.rs`, `src/looponboard*.rs`, `src/onboard.rs`, `src/presence.rs`, `src/skills.rs` and the matching test stems). `docs/loop/**`, `docs/harness/**` and `DECISIONS.md` are dropped where unedited, and the shared registration files (`src/lib.rs`, `src/main.rs`, the dispatcher, `src/doccheck.rs`, `docs/harness/README.md`, `docs/loop/README.md`) are in no scope, pending Casey's decision, which Claude is requesting.
- HIGH 2: non-loop items now use `docs/labs/<item-id>/**` and `docs/harness/prep/<item-id>*`; `docs/integration/NOW.md` and `docs/integration/project-track.md` are dropped everywhere.
- MEDIUM: pr03 gained `STATE.md`, `HANDOFF.md`, `docs/harness/LESSONS.md`, `AGENTS.md`, `archive/state/**`; pr08 gained `STATUS.md`, `STATE.md`, `HANDOFF.md`, `PLAN.md`, `archive/**`. pr03, pr08 and m8-release-packaging now say in their Rules section that the owner paths in their scope (`AGENTS.md`, `DECISIONS.md`, `.github/workflows/release.yml`) need the owner's approval recorded in the ORDER.
- LOW: the seven loop PR steps name their real issues (#2906, #2912, #2913, #2914, #2915, #2916, #2917).

Size by class: docs 62 insertions, 59 deletions across 52 files (51 items + kimi-1 REPORT); no code, no tests, no generated files. The ORDER set no estimate (D48).

## Measurements

- `grep` over `docs/roadmap/items/*.md`: no `scope:` line carries `docs/loop/**`, `docs/harness/**`, `docs/labs/**`, `docs/harness/prep/**`, `docs/integration/NOW.md`, `docs/integration/project-track.md`, `docs/integration/**`, `scripts/pod/**`, `.github/**` or `crates/uor-r4-harness/**` (the one `.github/**` hit is the explanatory note text in m8-release-packaging, not a scope glob).
- `cargo run -q --locked -p xtask -- roadmap check` → 87 items OK (ready 51).
- `cargo run -q --locked -p xtask -- roadmap render --check` → current.
- Known remaining overlaps are pairwise and intentional: the M1 items share `crates/uor-r4-discrete/**` (a dependency chain), a few item pairs share a mechanisms page, and `crates/uor-r4-integer/**` appears in about 8 items (m3-qualify, m4-export, m5, m6-causal, the two m7 items, object-store, the Tauri client, m8) — honest contention on the serving crate, not bookkeeping.

## Checks run at the head

- `xtask roadmap check` at `f1ae1f70b`: pass (87 items OK).
- `xtask roadmap render --check` at `f1ae1f70b`: pass (current).
- `uor-harness loop gates --quick` at `f1ae1f70b`: fmt-claim-wording, fmt-harness, claim-wording GREEN.
- `uor-harness loop report --check`: PASS at the head.
- `uor-harness loop pr-check --local`: fails only `scope` — the item still has no `scope:` at the merge base (expected until a scoped base lands); order, commit-order, report, owner-paths, rule-sources PASS.

## Skills used

- scope-guard: the written edge, and `git diff --stat` before content. The ORDER, the CHANGES record and the loop page were the opener; each review point was answered or explicitly flagged, not blindly applied (`project-track.md` note above).

## Still open

- `pr-check`'s one-ORDER-per-PR rule (and `commit-order`) needs a repair-aware variant — for example, count only the lab's highest-`n` ORDER, or read the ORDER the latest counted review record names. Raised on #2028 for the architect (loop PR 7b/5 area); this PR proceeds on the D59 review record per today's manual path.
- Decided (Casey, 21 Oct 23:43Z, recorded on #2028): the six shared registration files (`crates/uor-r4-harness/src/lib.rs`, `src/main.rs`, the `src/loopgates.rs` dispatcher, `src/doccheck.rs`, `docs/harness/README.md`, `docs/loop/README.md`) are exempt — no item scope lists them, any loop PR may make small additive edits there, and labs merge `origin/main` forward. Claude builds the exemption into `loop claim` and `pr-check` and will ask Casey to add `docs/roadmap/README.md` and `docs/harness/gates.json` to the shared list; both stay out of every scope until then.
- The `docs/labs/<item-id>/**` convention is new; existing evidence folders use dated names. Claiming labs should write new evidence under their item id, and may stop-and-ask to widen a scope when a sheet or folder genuinely needs another name.
- DeepSeek and Antigravity merge these item edits forward into their in-flight branches.

## Plain summary

The first version of the file lists ("scopes") that guard parallel work was too broad: every loop item claimed the whole harness crate and shared documentation folders, so the tool that hands out work would have let only one lab move at a time — the opposite of the point. This repair narrows each item to the files its own work touches, leaves the files every pull request shares out of all scopes until the owner decides how to handle them, and adds the missing notes about files only the owner may change.

References #2904.
