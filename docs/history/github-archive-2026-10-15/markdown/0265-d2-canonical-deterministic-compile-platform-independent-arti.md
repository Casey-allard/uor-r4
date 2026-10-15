# #265 — D2 canonical deterministic compile: platform-independent artifacts and kappas (make the promise in AGENTS.md trackable)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T15:06:30Z
- **Updated:** 2026-08-03T16:28:26Z
- **Closed:** 2026-08-03T16:28:26Z
- **Labels:** enhancement
- **Assignees:** auser, Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/265
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
AGENTS.md acknowledges the gap this issue makes trackable: κ baselines are **macOS-pinned** because teacher logprobs are libm-sensitive — identical pinned inputs produce different artifact bytes (hence different addresses) on Linux. "The canonical deterministic compile mode (D2) is what makes it a true cross-platform gate" — promised in AGENTS.md and the CI comments, tracked nowhere.

Universal addresses that differ by platform are not universal. Every distribution/verification story (region objects, patches, cross-node serving) inherits this ceiling until D2 exists: today "the address" is really "the address on a Mac."

### Direction (to be designed, not prescribed here)
The nondeterminism enters through float transcendentals in teacher observation/softmax (libm variance) and any platform-variant SIMD reduction order. Candidate routes: fixed-point/integer teacher observation for the canonical mode (compile-time is contract-unrestricted, so cost is acceptable); a pinned softfloat/deterministic-libm build of the observation path; or canonical-mode quantization applied before any platform-variant operation. The D2 mode need not be the fast default — it must be the **canonical** one that κs are defined against, with the fast path certified equivalent per platform.

### DoD
Same pinned inputs ⇒ byte-identical artifact (and κ) on macOS arm64 and Linux x86_64, demonstrated in CI (the κ-reproduction job stops being "informational on Linux"); baselines re-pinned once under the canonical mode with an era note; AGENTS.md Gate E section updated.

*Independent of the rest of the queue; load-bearing for #263/#264 and the entire universal-address thesis.*


## Comments (3)

### comment 5153570105 — 2026-08-01T21:40:24Z — auser

## Status audit (2026-08-01): mechanism landed, acceptance items remain

PR #324 (merge 3b70d28) delivered the canonical mode itself (`--canonical-deterministic`, src/main.rs:208,432; docs/d2_canonical_compile.md; same-host double-compile test). Verified against origin/main @ ff58b5c, the three DoD acceptance items are still open:

1. **No cross-platform CI demonstration** — every ci.yml job is ubuntu-latest; no macOS runner, no differential-compile job. The κ step is still literally named 'κ-reproduction (macOS-pinned baseline; informational on Linux)' with `continue-on-error: true` (ci.yml:110-112).
2. **No canonical-mode re-pin** — baseline_kappa.json is still the 2026-07-30 TLA6 macOS pin; no canonical-mode era note in docs/transformerless/BASELINE.md. (Note interplay with #335's pending 500k+TLA7 re-pin — the two re-pins should be sequenced into one pin event.)
3. **AGENTS.md Gate E not updated** — still says 'not expected to reproduce on Linux until the D2 canonical deterministic compile mode lands' (AGENTS.md:83-85).

Remaining work: (a) CI job compiling the same pinned inputs under --canonical-deterministic on macOS arm64 + Linux x86_64 asserting byte-identical artifact+κ; (b) drop continue-on-error and re-pin under canonical mode; (c) rewrite Gate E in AGENTS.md.

### comment 5154716564 — 2026-08-02T02:37:05Z — auser

Follow-up complete via PR #343 (merged as c597ed31c18ebbe8ab9c1881efa7b68bc6ba2194). The D2 merge-group run 30728871222 passed on both macos-14 and ubuntu-24.04, including D2 differential equality. Root cause was nondeterministic JSON key ordering from HashMap-backed hierarchical token prefixes; the fix uses BTreeMap and adds a serialization determinism test. Local main is synced to c597ed3. Remaining issue-265 acceptance work is the planned canonical re-pin/policy decision for the legacy κ check; leaving the issue open.

### comment 5169036164 — 2026-08-03T16:28:26Z — Casey-allard

## Closing: DoD verified met on main (73ab76b) — every acceptance item delivered

Verification against main, item by item from the DoD and the 2026-08-01 status audit:

1. **Cross-platform demonstration in CI** — delivered by #342 (D2 differential proof job) + #343 (BTreeMap serialization determinism fix): merge-group run 30728871222 passed D2 differential equality on macos-14 + ubuntu-24.04, and every merge-group run since (including today's five merges, #371–#378) runs the same differential-equality job.
2. **κ job no longer informational** — `ci.yml` step "κ-reproduction (canonical D2 baseline)" runs `TLESS_CANONICAL_DETERMINISTIC=1` with no `continue-on-error`; the workflow header records "Gate E now runs the canonical deterministic compiler mode."
3. **Canonical re-pin** — `baseline_kappa.json` carries `compiler_mode: "canonical_deterministic"` and the 2026-08-02 re-verification note: all pins and fixture bytes unchanged under D2 canonical math, so the existing TLA7 pins ARE the canonical pins (no bytes moved; the pin event was a verification, which is the best possible outcome).
4. **AGENTS.md Gate E** — updated to the canonical-mode language ("required for the cross-platform Gate E claim"; "re-pinned under the portable canonical math path"); the stale "not expected to reproduce on Linux" sentence is gone.

Universal addresses are no longer "the address on a Mac." Closing per delivery discipline; residual none — the promise this issue made trackable is now tracked by an enforced CI gate.


---
_Comment/review ids in this file: 5153570105, 5154716564, 5169036164_

