# #899 — research(#822): S1 redesign — D2 region-conditional instrument (REVISE: real, sub-floor)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-21T12:12:25Z
- **Updated:** 2026-10-09T14:17:17Z
- **Closed:** 2026-08-21T12:29:56Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/899
- **Merged:** 2026-08-21T12:29:56Z
- **Merge commit:** 201a747adcbf55735d08c8ee12ac955930517365
- **Base / Head:** main / UOR-Foundation:issue-822-d2-region-conditional
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Executes the S1 redesign RFC's step 3 (`docs/s1_redesign_rfc_822.md` §4-D2 / §6-3) under the contract posted to #822 before the run. The holographic question: does the artifact's own region assignment — which #784 showed is collision-free at full depth while continuation distributions converge — carry conditional signal the coarse convergence result hides?

**Verdict under the pre-registered consultation (region[4] → rxs): `REVISE`** — region[4] measured **+19.5‰ (paired 95% CI [+17.0, +21.9])**: real, code-specific (the codeshuf null collapses by −162.5‰), but its lower bound sits below the frozen 20‰ floor, and the rxs product key is sub-floor with sparsity (+10.4‰, covered 34,656/72,130). No lowering track opens from D2. The #784 convergence null is **refined rather than confirmed**: only 10.3% of covered region[4] tables argmax to the corpus-marginal token — fine-granularity regions are differentiated. The exploratory depth sweep peaks at depth 2 (+23.3‰ [+21.0, +25.7] — floor-clearing, below the 25‰ opening bar); recorded with the λ observations strictly as #897 design input.

## Scope and non-goals

Reference-only / off-serving-path instrument + records. Region codes are read from the released artifact via the deployed quantization path (`runtime::bundle_plain` → `runtime::assign_code_for_bundle` — the exact #784 instrument path, per the Q2 approval), cached through the κ-gated code sidecar; no recompile, no deployed changes, no format changes, no new capability, no gate moves.

## Changes

- `crates/uor-r4-api/tests/region_conditional_run_822.rs` (new, `#[ignore]` heavy): pre-registered arms (base reproduction; region[1..4]; rxs; codeshuf planted null), reproduction gates (base 246.6‰ ±0.05, minimal pairs 4,722 exact, base-follow 0), coverage + convergence diagnostics, double-run determinism check, CID-bound JSON writer.
- `docs/s1_redesign_instruments_822.md`: §5 appended (contract disposition + outcome; append-only record).
- `docs/region_conditional_822_result.json` (new): CID-bound run record (result_cid `blake3:4c70f8f1…`).
- `docs/RESEARCH.md`: the S1-redesign entry extended with the D2 result.

## Acceptance-criteria / contract status

Contract posted before the run: done (#822 comment). Reproduction gates: PASS exact. Anti-vacuity: PASS (tables non-empty at every depth; 37,587 distinct full-depth codes on held-out; codeshuf changed 56,988). Pre-registered verdict rule applied unmodified; the negative branch's obligations are met (honest sub-floor recording; design observations attached to #897; §6-4 decision request posted on #822).

## Tests and verification

- `cargo test -p uor-r4-api --release --test region_conditional_run_822 -- --ignored --nocapture` → ok (5.0 s; output mirrored in the record doc §5).
- `cargo check -p uor-r4-api --tests`; `cargo clippy -p uor-r4-api --all-targets -- -D warnings`; `cargo fmt --check`; `python3 scripts/check_claim_wording.py` → all clean.
- The new test is `#[ignore]` (needs the local #833 bundle + artifact) and self-skips without them; CI's default suites are unaffected.

## Compatibility, conformance, claim boundaries

No wire, format, or runtime surface changes. Adds an off-serving **Empirical** REVISE result with formal-vocabulary labels; deployed-path claims remain governed by the frozen gates.

## Negative / unavailable results

The headline result IS a recorded negative at the pre-registered consultation: region[4] sub-floor (+19.5‰, lower bound 17.0 < 20), rxs sub-floor (+10.4‰). region[1] is null (+0.8 [−1.8, +3.4]) — coarse regions carry nothing, consistent with #784 at its granularity.

References #822


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

