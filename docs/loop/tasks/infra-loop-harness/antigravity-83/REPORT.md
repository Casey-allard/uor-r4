# REPORT: infra-loop-harness, antigravity-83

- Item: `infra-loop-harness` ([docs/roadmap/items/infra-loop-harness.md](../../../roadmap/items/infra-loop-harness.md), issue #2904)
- Lab: `antigravity`
- Branch: `antigravity/unblock-m1-m4-sheet-scopes`
- Order: [ORDER.md](ORDER.md)

## What did not work

- **Direct `uor-harness loop claim` on `infra-loop-harness` was refused (`instrument`).** `uor-harness loop claim infra-loop-harness --lab-topic unblock-m1-m4-sheet-scopes` refused because idle draft `PR #3169` (`codex-1`) holds the umbrella item (`PLAN.md` and `docs/integration/DECISIONS.md`). Per the umbrella exception (`CHARTER.md` §5.3, `docs/loop/README.md`), verified zero file overlap against `#3169`, `#3230`, and `#3231` and authored `ORDER.md` in `docs/loop/tasks/infra-loop-harness/antigravity-83/`.
- **Four ready M1/M4/research items were blocked by stale crate/doc globs in their `scope:` lists (`instrument`).**
  - `m1-normalization-mechanism` (`#2842`, milestone M1) still carried `docs/mechanisms/hamilton-gate.md`, which overlaps `PR #3231` (`hamilton-gate-offline-float-sheet`), even though its remaining unit only updates `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md` to `TEST FITNESS: FIT` and closes `#2842`.
  - `m4-flock-coupling-prereg` (`#2860`, milestone M4), `r-prime-zeta-vs-nulls` (`#2881`, milestone research), and `r-read-off-vs-read-on` (`#2885`, milestone research) still carried `crates/uor-r4-discrete/**`, which overlaps `PR #3230` (`m1-address-key-full-vector`), even though all their code and host controls (`flock_coupling_controls.rs`, `prime-zeta-codebook`, `read_off_ablation_controls.rs`, `RouteBypassV1`, `RoutePermutedActionsV1`, `pz-nulls-battery.sh`, `read-off-battery.sh`) are already merged on `main`.

## What changed

| File | Class | Lines (+/-) | Purpose |
|---|---|---:|---|
| `docs/roadmap/items/m1-normalization-mechanism.md` | docs | +3 / -1 | Remove `docs/mechanisms/hamilton-gate.md` from `scope:` for the sheet/`TEST FITNESS: FIT` close unit |
| `docs/roadmap/items/m4-flock-coupling-prereg.md` | docs | +4 / -1 | Remove `crates/uor-r4-discrete/**` from `scope:` and add `notebook:` + task folder glob for the sheet/`TEST FITNESS: FIT` close unit |
| `docs/roadmap/items/r-prime-zeta-vs-nulls.md` | docs | +4 / -1 | Remove `crates/uor-r4-discrete/**` from `scope:` and add `notebook:` + task folder glob for the sheet/`TEST FITNESS: FIT` and evaluation unit |
| `docs/roadmap/items/r-read-off-vs-read-on.md` | docs | +4 / -1 | Remove `crates/uor-r4-discrete/**` from `scope:` and add `notebook:` + task folder glob for the sheet/`TEST FITNESS: FIT` and evaluation unit |
| `docs/loop/tasks/infra-loop-harness/antigravity-83/ORDER.md` | docs | +35 / -0 | Task order and cancel probe |
| `docs/loop/tasks/infra-loop-harness/antigravity-83/REPORT.md` | docs | +61 / -0 | Task report |

- **Size:** 6 files changed (0 code, 0 tests, 6 docs, 0 generated), +111 / -4 lines. Ratio to estimate: within single-unit scope-narrowing budget (following `claude-105` `#3223`, `claude-108` `#3229`, and `antigravity-80` `#3232`).

## Measurements

- **No-overlap check against all open PRs (`#3169`, `#3230`, `#3231`):**
  - Command: `gh pr diff 3169 --name-only && gh pr diff 3230 --name-only && gh pr diff 3231 --name-only`
  - Result: `0` overlapping paths with `docs/roadmap/items/{m1-normalization-mechanism,m4-flock-coupling-prereg,r-prime-zeta-vs-nulls,r-read-off-vs-read-on}.md`.
- **Roadmap and harness validation:**
  - Command: `~/.cargo/bin/cargo run --quiet -p xtask -- roadmap check && uor-harness check`
  - Result: `roadmap check: 92 items OK`, `uor-harness check` `0 error`.
- **D68 Decision Review (`CHARTER.md` §5.1, §5.2):**
  - Reviewed in a fresh context against `CHARTER.md`, `PLAN.md`, `docs/integration/DECISIONS.md` (D68, D77), and the four item files (`DECISION REVIEW: APPROVE`, logged for Casey on `#2904`, `#2842`, `#2860`, `#2881`, and `#2885`).

## Checks run at the head

- `~/.cargo/bin/cargo run --quiet -p xtask -- roadmap check` — PASS (`92 items OK`)
- `uor-harness check` — PASS (`0 error`)
- `uor-harness loop report --check --order docs/loop/tasks/infra-loop-harness/antigravity-83/ORDER.md` — PASS
- `uor-harness loop pr-check --local` — PASS
- `uor-harness loop gates --full` — PASS (`11/11 GREEN`)

## Skills used

- `scope-guard`: Kept changes strictly to the four roadmap item files and the `antigravity-83` task folder.
- `solution-integrity`: Verified that all discrete code and host controls for the four items are already merged on `main` and that the narrowed scopes pass `roadmap check` and `uor-harness check`.

## Still open

- Claim `m1-normalization-mechanism` (`#2842`), `m4-flock-coupling-prereg` (`#2860`), `r-prime-zeta-vs-nulls` (`#2881`), and `r-read-off-vs-read-on` (`#2885`) to complete their pre-registration sheet `TEST FITNESS: FIT` reviews and close/evaluate them.
- The umbrella item `infra-loop-harness` (`#2904`) remains `status: ready`.

## Plain summary

Four ready roadmap items (`m1-normalization-mechanism`, `m4-flock-coupling-prereg`, `r-prime-zeta-vs-nulls`, and `r-read-off-vs-read-on`) already have their Rust code and finite host controls merged on `main`, and their next work units only update their pre-registration sheets (`docs/harness/prep/`), obtain non-author `TEST FITNESS: FIT` reviews, or run zero-update evaluations. Because their item `scope:` lists still included `docs/mechanisms/hamilton-gate.md` (held by `PR #3231`) or `crates/uor-r4-discrete/**` (held by `PR #3230`), `uor-harness loop claim` refused claims on them due to scope overlap. Following the `claude-105`/`claude-108` and `antigravity-80` precedent and under a fresh-context D68 decision review, this unit narrows the four items' scopes to their documentation, pre-registration, lab, and task directories so they can be claimed and closed immediately.

References #2904
References #2842
References #2860
References #2881
References #2885
