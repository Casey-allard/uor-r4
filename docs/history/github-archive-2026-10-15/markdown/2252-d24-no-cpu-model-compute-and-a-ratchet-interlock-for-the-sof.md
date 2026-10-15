# #2252 — D24: no CPU model compute, and a ratchet interlock for the softmax ban

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T12:34:00Z
- **Updated:** 2026-10-12T12:36:57Z
- **Closed:** 2026-10-12T12:36:25Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2252
- **Merged:** 2026-10-12T12:36:25Z
- **Merge commit:** fe8b076a124e4004dc66bc8c2236db60d8d57053
- **Base / Head:** main / UOR-Foundation:deepseek/no-softmax-rule
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line · count · headline

Governance for every line (D24) · count unchanged · no headline change. **References #2028**, and records the cancellation of the M1 [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) campaign. Does not close anything.

## Why

The owner restated the rule in session: *"no softmax at all anywhere. all softmax needs to be replaced with a geometric mechanism we design. no CPU runs at all. rewrite for GPU. do not waste GPU runs on bullshit that doesn't matter."*

[D23](https://github.com/UOR-Foundation/uor-r4/pull/2251) had already recorded the softmax prohibition the same hour. The problem was not the wording — D23 says so itself: *"not a comprehensive Rust/CLI interlock"*. The rule was written down and behaviour did not change, so this PR adds the two clauses D23 omitted and, more importantly, the interlock it lacked.

## What changes

1. **D24** — model compute never runs on the CPU; a host-side fallback is a **defect**, not a measured cost and not mechanism evidence (both the arithmetic and the step time are wrong). GPU time is spent only where it can change a decision; an excluded configuration does not get a run.
2. **Replacement is a mandate** — every prohibited site is replaced by a designed geometric mechanism (exact `Z[phi]`/zeta-phase interference, typed H4/icosian pairing, Lorentz or address geometry, or exact hard selection by address) with prospective TEST FITNESS before compute.
3. **`scripts/check_no_softmax.py`** — the ratchet. Scans `crates/**` (`.rs/.cu/.cuh/.wgsl`); a softmax site may not be added and `scripts/no-softmax-baseline.json` may only decrease. `--update-baseline` **refuses to raise any count**, so it cannot bless new softmax.
4. **AGENTS.md + `agent-execution-policy.json/.md`** carry the rule where agents read it first, and in machine-readable form.

## Checks at this head

- `python3 scripts/check_no_softmax.py` → `OK: no softmax added. 1649 remaining across 145 files (baseline 1649).`
- Negative test: adding one `softmax` token to a new file → **exit 1**, `crates/uor-r4-training/src/__ratchet_probe.rs: 0 -> 1 (+1)`. Probe removed.
- `--update-baseline` self-test: refuses a raise (first-run case allowed to set the ratchet).
- `python3 scripts/check_claim_wording.py` → passed.
- `docs/integration/agent-execution-policy.json` parses; schema unchanged.

## Cancelled under the rule (recorded, not a negative)

The M1 #2029 invariant-addressed router campaign was **stopped and its pod deleted** at 2 of 3 arm-A seeds. Arms A/C normalize over the kept candidates with softmax, arm B *is* the full softmax read, and the routed read executes host-side (`cuda_covered()` false when a route is set) — all three violate D23/D24.

Retained: `icloud:UOR-R4/results/deepseek/router-abandoned-2026-10-12.tar`, md5 `f411b7d1870afdabcd2195ce27a8c157` — both completed fits' `report.json`/`config.json`/`manifest.json`/`attempt.json` plus full logs. Seed 20261009: dev NLL 6.230 → 0.311, pointer hit 0.029 → 0.697. Seed 20261010: 6.230 → 0.272, hit 0.029 → 0.703. Weights not retained (configuration excluded; non-canonical volume, no canonical companion). This is a configuration cancellation under an owner rule, **not** a mechanism negative and **not** a D21/D22 count.

## Limitations

- The ratchet stops growth; it does **not** claim project-wide removal is complete. 1649 sites remain at the baseline.
- Dated `docs/` records are deliberately unscanned — they are evidence, and rewriting them would destroy the record.
- The gate sees the string `softmax`. A mathematically equivalent normalized exponential written without that name (D23 forbids it) is not caught by the gate; the decision text covers it.
- **Open owner question:** whether the frozen D10-era comparators (`uor-r4-lut`/`lut-chat`, SmolLM2 refs) should be deleted rather than preserved as inactive evidence. This PR preserves them.

## Next

The replacement design — a geometric normalization/mix to stand in for softmax in the read, plus the CUDA routed-read kernel that removes the host fallback — is the following unit and needs a prospective TEST FITNESS review before any compute.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

