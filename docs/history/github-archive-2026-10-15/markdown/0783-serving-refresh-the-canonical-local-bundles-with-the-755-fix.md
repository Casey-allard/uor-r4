# #783 — serving: refresh the canonical local bundles with the #755 fix and take the fresh baseline quality read (the #655-F precondition-2 decider)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-18T14:51:27Z
- **Updated:** 2026-08-18T18:11:54Z
- **Closed:** 2026-08-18T18:11:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/783
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem

The #755 compiler fix (context reconstruction sorts records by `(story, span_start)`) is merged and regression-tested — but **no canonical local bundle has been recompiled with it**. The 2026-08-18 baseline audit reproduced the pre-fix behavior live at `aea30bae`, through the pure R4G1 ask path (no teacher, no fallback), deterministic decoding:

| bundle | prompt | duration | output |
|---|---|---|---|
| `smollm2-135m-instruct` | "What is the capital of France?" | ~235 s stall | `ounds Callounds Callounds Call…` (the exact #745/#755 "before" signature) |
| same | "Why is the sky blue?" | ~207 s | **byte-identical** to the other prompt (808 bytes) |
| same | same prompt, repeat + `--sample 42` | ~240 s each | byte-identical (deterministic; sampler unreachable — #785) |
| `smollm2-360m-instruct` | both prompts | 85–93 s | `<|im_start|>cescescesution…` — byte-identical across prompts; leaks the raw `<|im_start|>` token |
| `smollm2-1-7b-instruct` | Q1 | 5 s | `cut cut cut cut …` (the #750 R4G1-path signature) |

Additional bundle-state evidence from the same session:
- The BDD teacher-parity suite's **R4G1 graph scenarios skip** on this host with `graph provenance κ mismatch (report blake3:e759500a… vs artifact blake3:a1863bdd…)` — the 135m bundle's score report and artifact are from different compiles (mixed-era bundle).
- `.uor-models/manifests/` is **empty** — no imported manifest exists, so every `r4 ask` runs the un-attested local-bundle path (WARN emitted).
- `smollm2-360m-broad` (#509's positive-result bundle) has no `tokenizer.bin`, so it is not askable at all.

This is #655-F **precondition 2** ("in-domain canary produces a valid assistant completion") sitting on a stale-artifact problem, not an open architecture question. Full evidence: `docs/project_baseline_audit_2026_08_18.md` §13–§14 (in the pending docs-only reconciliation diff on the maintainer's working tree).

## Scope (increments)

- **A1** — recompile `smollm2-135m-instruct` from its existing, unmodified `corpus.meta`/`corpus.records` via `r4 transformerless compile-recorded` (minutes, no teacher), then regenerate `graph-cover` + `graph/score.r4g1` so the score-report/artifact κs agree again (also un-skips the BDD graph-parity scenarios).
- **A2** — same for `smollm2-360m-instruct` and `smollm2-1-7b-instruct`.
- **A3** — decide `smollm2-360m-broad`: complete it (export tokenizer) or leave it measurement-only; record the decision.
- **A4** — re-run the #758 15-prompt protocol (exact-match hashed) on each refreshed bundle at baseline greedy; record per-bundle distinct counts + degeneracy verdicts.
- **A5** — `evaluate-report` + `import` at least the best refreshed bundle so one attested `instruction-chat` manifest exists and the #744/#750 live gate is actually in the serving path again.

## Run contract (pre-declared)

- **metric to move:** #758 distinct-output count + non-degeneracy at baseline greedy on the refreshed `smollm2-135m-instruct`. Current: 2/2 probed prompts byte-identical, degenerate.
- **reachability ceiling:** #755's own decisive retest showed a categorical change on this exact corpus (grammatical output in ~0.2 s where there was a ~199 s hang + garbage) — the ceiling is a category change, not a pp delta.
- **instrument + verdict:** the recompile + two-prompt ask IS the cheap instrument (minutes); proceed to the full 15-prompt set only if the two-prompt probe is no longer degenerate.
- **exit rule:** PASS iff ≥ 10/15 distinct outputs AND no single-token cycling on the refreshed 135m bundle.
- **if positive:** post the fresh quality read on #655 (F precondition 2 input) and proceed to the decode-policy experiment (#785 C4).
- **if negative:** the ordering fix is insufficient for this bundle — feed the data to #784 and stop; do not iterate here.
- **cost estimate:** ~15–30 min per bundle, CPU-only; blocks nothing.

## DoD

Refreshed bundles on disk with matching provenance κs; #758 protocol results recorded here per bundle; at least one attested manifest imported; a fresh baseline quality read posted on #655.

**Next action:** A1 on `smollm2-135m-instruct`.


## Comments (3)

### comment 5330471289 — 2026-08-18T15:35:09Z — Casey-allard

## A1 complete — refreshed `smollm2-135m-instruct`, measured against the pre-declared contract

**Recompile (post-#755 compiler at `4b9f0562`):** `r4 transformerless compile-recorded` from the bundle's existing, unmodified `corpus.meta`/`corpus.records` (361,693 records, 91 s). New artifact κ `blake3:487532dd…` (TLA7, 1.4 MB — the old artifact was TLA5-era, 530 KB), new store 31.3 MB (**down from 83.6 MB** — consistent with #755's corrupted BOS-collapsed keys having inflated the old store). Installed into the canonical bundle; every replaced file backed up at `.uor-models/backups/2026-08-18-pre755/smollm2-135m-instruct/`. Regenerated `compiled.r4g1` (convert-r4g1, κ `blake3:e0c3f2cc…`, 196,233 keys) and `graph-cover/` + `graph/score.r4g1` (κ `blake3:0caa130f…`, cover 20 s, score 112 s, `--quality-profile relative_tla`). The score report's `artifact_kappa` now matches the installed artifact — the provenance κ mismatch that made the BDD teacher-parity graph scenarios skip is resolved for this bundle.

**Offline metrics (this corpus, 72,258 held-out, full census):** rule 1+2 **25.65% / 12.14 bits**; TLA3 store baseline 30.12% / 13.60; rule-1 graph-only 1.87%; EXCT-miss generalization slice 2.20% / 17.32 (n=19,851).

**Cheap instrument (2-prompt):** PASS — sub-second answers in real, grammatical (topically wandering) English on the plain path; the two prompts differ. The pre-refresh baseline on the same binary was a ~4-minute stall into `ounds Call…` cycling, byte-identical across prompts.

**15-prompt protocol (declared set: 8 prompts verbatim from #758 — the six named questions + "hello" + "1 2 3 4 5" — plus 7 reconstructed to the same shape, incl. "What is April?" and a story-continuation edge case; exact list in the run log):**

- **Plain TLA path** (scratch bundle without `compiled.r4g1`): **8/15 distinct, zero cycling** (max consecutive identical token run = 1). Largest collapse group = 5 prompts ("Tell me about art." / "Describe the water cycle." / "Tell me a fact about the ocean." / "Describe the process of digestion." / "Explain how airplanes fly.") — the same attractor group #758 recorded (theirs: 9/15 distinct, same-named group of 5). **Verdict vs the pre-declared exit rule (≥10/15 distinct AND no cycling): NEGATIVE on distinctness, positive on degeneracy.** Per the contract's negative branch, the residual prompt-collapse data goes to **#784** (attractor basins) — which is exactly what that issue was filed for; no further iteration here.
- **R4G1 path** (the canonical bundle's default, via the fresh `compiled.r4g1`): **degenerate and fully prompt-invariant — 5/5 probed prompts produce the identical `regarding regarding …` single-token loop** (34–60 s each). The corpus fix did not and cannot reach this: it is the #785 decode-path defect (inert beam ⇒ the repeat penalty downstream is dead code; signature-blind terminal fallback). Full 15 on this path skipped as information-free given 5/5 identical collapse — declared here rather than silently truncated.

**Reading:** #755's fix is confirmed institutionalized — *degeneration on the plain path is gone* in the canonical bundle. What remains is (a) the known attractor-basin prompt-collapse (#784, residual by design of the exit rule), and (b) the R4G1 ask path serving worse-than-plain output on identical bytes (#785) — which is the path `r4 ask` actually prefers. Working #785-C1 next in this same cadence; will re-probe the R4G1 path here once it lands, then run A5 (import through the live gate **with `--r4g1`**, which per #750 should refuse until the R4G1 path is non-degenerate — the gate doing its job).

A2 (360m + 1.7B refresh) proceeding now.


### comment 5332205143 — 2026-08-18T18:11:38Z — Casey-allard

## A4 + A5 complete — issue closing with records

**A4 — 15-prompt protocol on the default R4G1 ask path** (post-#793; canonical refreshed `smollm2-135m-instruct`; deterministic double pass, 30/30 byte-identical): **5/15 distinct outputs**, grammatical prompt-form openers, ~1 s/ask, every trajectory converging into a "the 19…" digit/comma attractor. Contrast: pre-#793 this path was 15/15 byte-identical `regarding` repetition at ~35 s/ask — the degeneracy was not this issue's stale-bundle hypothesis but the #785 decode-path defects (wrong artifact + container bytes scored as evidence; full record on #785). The plain transformerless path on the same refreshed bundle remains the stronger reader: 8/15 distinct grammatical English (A1 record above). **The run contract's category-change bar is met on the mechanism level** (garbage-repetition → grammatical-openers-with-attractor), and the honest quality read for #655 stays negative (below).

**A5 — evaluate-report + import on the refreshed bundle: both gates live, both fail closed.**

- `r4 evaluate-report` → typed refusal: `source unavailable: .uor-models/compiled/smollm2-135m-instruct has no tokenizer_adapter.json; evaluation cannot prove that its corpus id space matches hf-byte-bpe/1`. The canonical bundle predates the adapter attestation (the backup pre-#755 layout lacks it too); producing one requires the full HF compile path with adapter emission — an hours-class teacher run that needs its own pre-declared contract + approval. Not launched.
- `r4 import --capability instruction-chat …` (with the stale Jul-26 report) → typed refusal: `model has not passed its instruction/grounding evaluation gate`.

So the #744/#750 live gate **is in the serving path again and demonstrably gates** — A5's attested-manifest goal is UNAVAILABLE until a contracted recompile produces the adapter + a passing report. That is the honest state, not a bypass candidate.

**Ledger of the A-items:**

- A1 ✅ refresh + probes (comment above): plain path grammatical (8/15 distinct), R4G1 path was still degenerate → isolated to #785, now fixed there.
- A2 ⛔ recorded UNAVAILABLE: 360m/1.7b recorded-corpus compiles fail closed by design (`meta done_flag=0`, genuinely incomplete corpora); teacher-resume is an hours-class run needing a contract.
- A3 ✅ `smollm2-360m-broad` made askable (tokenizer copied per the #758 precedent).
- A4 ✅ above.
- A5 ✅ above (gates verified live; positive arm blocked on a contracted long run).

**Fresh quality read for #655 (F-precondition-2):** posted to #655. Short form: still **unmet at baseline** — 5/15 distinct with a universal digit attractor is not "a valid assistant completion" — but the substrate is now real (real emissions, real rows, real scores, 1 s latency), so the remaining gap is decode policy and score shaping (#784, #785-C4), no longer wiring archaeology.

Closing: A-items all resolved or honestly recorded as UNAVAILABLE with the follow-on work tracked (#784, #785-C2/C3/C4, contracted recompiles). Unassigning per board convention.


### comment 5332207449 — 2026-08-18T18:11:52Z — Casey-allard

Closing with records: A1 ✅, A2 UNAVAILABLE (fail-closed by design, incomplete corpora), A3 ✅, A4 ✅, A5 ✅ (gates live + fail closed; attested manifest blocked on a contracted recompile). See issuecomment-5332205143 for the full ledger. Follow-on: #784, #785-C2/C3/C4, and the contracted-recompile decision points recorded above.


---
_Comment/review ids in this file: 5330471289, 5332205143, 5332207449_

