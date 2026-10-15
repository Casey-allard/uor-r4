# #864 — docs(#833): S0-E preflight, run contract, and HELD verdict for the attested broad baseline

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T06:21:52Z
- **Updated:** 2026-10-09T13:08:54Z
- **Closed:** 2026-08-20T06:40:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/864
- **Merged:** 2026-08-20T06:40:21Z
- **Merge commit:** 9c1f2c10491f688550b75a459c530a3aa6b07fd9
- **Base / Head:** main / UOR-Foundation:issue-833-attested-broad-baseline
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

S0 item E (#833) asks for a source-complete, tokenizer-attested, #755-native,
deterministic rebuild of the broad-canonical baseline, deployed admission, and an
explicit retain/revise/retire verdict on the M.V.G. gates. That is the flagship
hours-to-days Accelerate-teacher run, and `AGENTS.md` (long-run discipline) requires
a posted run contract plus a passing binding cheap instrument **before** it launches.

This PR adds the preflight evidence record `docs/attested_broad_baseline_833.md`:
the reachability arithmetic, the run contract, the measured cheap-instrument
verdicts, and a **HELD** verdict. The cheap instrument surfaced three preconditions
that must clear before the expensive run — so running it now would have burned a
multi-day teacher compile against a corpus and a build that cannot yet produce an
attested, byte-reproducible bundle.

## Scope and non-goals

- **Scope:** the preflight record only (one new `docs/*.md`). No code, no
  `model/ids.toml`, no `CONFORMANCE.md`, no bundle bytes.
- **Non-goals:** launching the rebuild; changing any M.V.G. threshold; fixing the
  determinism defect or re-pinning the source (both named as next actions with
  owners).

## Acceptance-criteria status (#833)

Partial — this is preflight, not the baseline. Against #833's acceptance criteria:

- "Reconstructable from pinned inputs / all CIDs present" — **blocked**: pinned
  teacher revision `2366112999…` 404s upstream (UNAVAILABLE); maintainer re-pin
  needed.
- "Two deterministic builds reproduce byte identities" — **partial FAIL**:
  `tless_artifacts.bin` (κ = attested signature `blake3:0b85c43d…`),
  `tless_store.bin`, `hamming_calibration.json` are byte-reproducible across two
  fresh recompiles; `hierarchical_codes.json` is **non-deterministic**.
- "Empty-root offline admission + canary pass" — the #516 pin **loads and predicts
  offline** through R4Engine (~1.5 s), but **without an instruction-quality
  attestation** (the gap #833 closes).
- "One reconciled current baseline" — unchanged: the reachability arithmetic RETAINS
  the #516 pin pending a clean run.

## Tests and verification

- `python3 scripts/check_claim_wording.py` — pass (docs claim-wording gate).
- Diff is docs-only (single new file), so PR CI runs the claim-wording gate; the
  merge queue re-verifies the full ladder on the merged content.
- Preflight instruments were run live on `origin/main@70586aa0`
  (`aarch64-apple-darwin`, offline): `r4 ask` admission canary,
  `subsample-recorded-corpus` integrity guard, two `compile-recorded` runs with
  `cmp` byte-identity comparison.

## Evidence artifacts

The record itself, with the measured cheap-instrument table, the run contract, and
provenance (κ) of the pinned bundle and the two fresh recompiles.

## Compatibility, migration, conformance, and claim boundary

No behavior, format, or ABI change. No conformance effect (binds existing RF-09/15/
21/22/23/29; no ids/CONFORMANCE change). The record is a preflight and a hold
verdict — it promotes no claim and changes no claim boundary.

## Negative / unavailable results (kept, not hidden)

Three FAIL/UNAVAILABLE verdicts are recorded, not omitted: the pinned-revision 404
(UNAVAILABLE), the post-#755 corpus-integrity rejection (FAIL), and the
`hierarchical_codes.json` non-determinism (FAIL). These are the gate the discipline
exists to enforce.

## Intentionally excluded

The flagship rebuild, the determinism fix, and the source re-pin are out of this
PR by design — each is a named next action with an owner in the record.

References #833


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

