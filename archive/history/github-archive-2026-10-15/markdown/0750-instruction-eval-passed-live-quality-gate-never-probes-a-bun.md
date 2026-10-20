# #750 — instruction_eval_passed live quality gate never probes a bundle's R4G1 graph

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-17T01:09:55Z
- **Updated:** 2026-08-17T07:30:47Z
- **Closed:** 2026-08-17T07:30:47Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/750
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

Follow-up from #744 (PR #749). The new live-quality gate in `r4 import` (`evaluate_live_quality` in `src/model.rs`, `chat::engine_from_bytes`) derives `instruction_eval_passed`/`grounded_answer_rate`/`repetition_rate` by loading the imported `--artifacts`/`--store`/`--tokenizer` bytes and running a fixed probe set through the real `ask` generation path — but `engine_from_bytes` always constructs its `ChatEngine` with `r4g1_bytes: None`. It only ever exercises the plain TLA/TLS1 greedy-generation path.

# Problem

`ChatEngineBuilder::build()` (the real path `r4 ask`/`r4 chat` use in normal operation) auto-discovers an R4G1 graph at a fixed, manifest-name-keyed convention path: `.uor-models/compiled/{manifest.name}/compiled.r4g1`. If a file exists there, `hologram_answer` takes a completely different code path — R4G1 beam search instead of plain greedy generation — and the two paths can produce substantially different output quality for the *exact same* underlying compiled artifact bytes.

This is not hypothetical. While verifying #744's fix, I tested the exact same compiled bytes (`smollm2-1-7b-instruct`, artifact `blake3:fca5bdfb85d0c78ab3ff0669f882f0ddfb6d3eccb9fc1b00284d9c71a3f5583d`) through both live code paths:

- Directory-fallback + R4G1 beam-search path (`r4 ask --model <directory>`): literal `"cut cut cut cut cut ..."` repeated dozens of times — extreme single-token degenerate repetition.
- Imported-manifest plain-path (`r4 ask --model <imported-manifest-name>`, no matching `compiled.r4g1` convention path): the #745 word-salad transcript (`tetra`/`gru`/`gulick`/`chall`/`caption`-style cycling).

Both are bad, but *differently* bad, and #744's gate can currently only ever see the second one. A bundle whose plain-path probes pass the gate is not thereby shown to be non-degenerate if served via a same-name-keyed R4G1 graph at runtime — the gate's pass/fail decision does not actually cover what `ask`/`chat` will serve in that case.

# Goal

The live quality gate's pass/fail decision should reflect what a served bundle will *actually* produce, for whichever code path `ask`/`chat` will actually use for it — not just the plain path.

# Scope

- Decide how `import` should learn about an R4G1 graph for the bundle being imported. Options include: an explicit new `--r4g1 <path>` flag on `ImportArgs` (parallels the existing explicit `--artifacts`/`--store`/`--tokenizer` flags rather than relying on directory-naming convention); or importing whatever `.uor-models/compiled/<name>/compiled.r4g1` convention resolves to automatically, matching how `ChatEngineBuilder::build()` behaves for an already-imported manifest.
- Extend `chat::engine_from_bytes` (or add a sibling constructor) to accept optional R4G1 graph bytes and construct a `ChatEngine` with `r4g1_bytes: Some(...)` when supplied, so probes actually exercise the beam-search path.
- Decide how to combine results when both a plain-path and an R4G1-path probe run exist for the same bundle — e.g. does the gate require *both* to pass, or does it record path-specific attestations? Given the observed divergence (word-salad vs. "cut cut cut"), silently averaging or taking the better of the two would defeat the point.
- Update `QualityAttestation` and/or the manifest schema if a single pass/fail boolean can no longer honestly summarize "this bundle is safe under every code path `ask` might use to serve it" — or document explicitly, if the resolution is to only ever probe one path per bundle (e.g. because a manifest can only ever resolve to one convention path at serving time), why that's sufficient.
- Update `docs/MODEL_LIFECYCLE.md`'s known-limitation note (added in #744/#749) once this is resolved.

# Non-goals

- Do not use this issue to relax #744's gate or reintroduce manually-typed quality fields.
- Do not attempt to fix the underlying generation-quality problem (#745) here — this issue is only about whether the *measurement* covers the code path that's actually served.

# Acceptance criteria

- [ ] `import`'s live quality gate for `--capability instruction-chat` exercises the same code path(s) that `ask`/`chat` will actually use to serve the resulting manifest, including the R4G1 beam-search path when applicable.
- [ ] A bundle that is degenerate via its R4G1 path (like the "cut cut cut ..." case found in #744) must not report `instruction_eval_passed: true`, even if its plain-path probes look fine.
- [ ] A falsifier test demonstrates this using a synthetic case (e.g. inject R4G1 bytes that force degenerate beam-search output while plain-path probes pass).
- [ ] Workspace fmt/clippy/tests pass.

# Verification

- Re-run the gate against a bundle known to have divergent plain-path vs. R4G1-path behavior (the `smollm2-1-7b-instruct` bundle referenced above, if still available locally) and confirm it is now rejected regardless of which path is probed.

References #744, follow-up to #749.

## Comments (2)

### comment 5312662445 — 2026-08-17T06:33:49Z — Casey-allard

Starting on this one next, continuing the backlog cadence after #759 (which turned into a corrected-diagnosis research finding rather than a shippable fix — see that issue for the write-up).

Working in `.worktrees/i750-r4g1-quality-gate` off `main@25fd9d6e`.

Plan, following this issue's own scope: read `evaluate_live_quality`/`chat::engine_from_bytes`/`ChatEngineBuilder::build()` in full first to confirm the exact current shape (this repo's own discipline of verifying against live code rather than assuming, learned the hard way on #759 tonight), then decide the `--r4g1` flag vs. auto-discovery question and the "both paths must pass" combination semantics before writing code, since this touches the manifest schema and #744's existing gate.


### comment 5312950786 — 2026-08-17T07:10:25Z — Casey-allard

PR up: #761.

Landed as planned in the orient comment: `chat::engine_from_bytes_with_r4g1`
splits out of `engine_from_bytes`; `evaluate_live_quality` gains an
`r4g1_bytes: Option<&[u8]>` parameter and runs the probe set through the
R4G1 path too when supplied, via a new `run_live_quality_probe_set` helper
and `combine_path_quality` policy function (both paths must independently
pass); `QualityAttestation` gains `r4g1_grounded_answer_rate` /
`r4g1_repetition_rate: Option<f32>` (`#[serde(default)]`, no schema bump);
`r4 import` gains `--r4g1 <path>`.

Full verification ladder passed (fmt, clippy -D warnings, xtask
audit-limits, full `cargo test --workspace --offline`), plus a real-bundle
check: ran `r4 import` against the real local `smollm2-1-7b-instruct`
bundle (the one this issue's own filing references) both with and without
`--r4g1 compiled.r4g1`, using a scratch `UOR_MODEL_STORE` so nothing
touched the checkout. Both correctly reject via `ModelError::QualityGateFailed`.
Worth noting honestly: this particular bundle turns out to already fail on
the plain path alone (consistent with #745's prior word-salad finding on
the same bundle), so it doesn't demonstrate the specific
plain-passes/R4G1-fails split #750 exists to catch -- it only confirms
"rejected regardless of which path is probed," which is what the issue's
acceptance criteria literally asked for. The specific split scenario is
covered directly by the new synthetic falsifier test
(`combine_path_quality_requires_both_probed_paths_to_pass`), following
`#744`'s own established pattern of testing the pass/fail policy function
in isolation rather than only against real bundles.

`docs/MODEL_LIFECYCLE.md`'s known-limitation paragraph rewritten to
describe the fix and the new `--r4g1` flag.

PR auto-merge enabled, headed into the merge queue once CI clears.



---
_Comment/review ids in this file: 5312662445, 5312950786_

