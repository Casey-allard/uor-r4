# #354 — allocation_census passes vacuously when artifacts are absent (audit: only suite affected)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-02T23:02:54Z
- **Updated:** 2026-08-02T23:44:44Z
- **Closed:** 2026-08-02T23:44:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/354
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Tests that depend on artifacts under `.uor-models/` or `/tmp/ref/` currently
report **success** when those artifacts are absent. Success and "never actually
ran" are indistinguishable from the exit code, so a suite guarding a hard
invariant can be green for months while the invariant is false.

## Concrete instance

`cargo test -p uor-r4-core --test allocation_census`:

| artifacts | runtime | result |
| --- | ---: | --- |
| absent | ~1.9s | **pass** (assertions never executed) |
| present | ~100s | **fail** — R4G1 kernels allocate 594 bytes in steady state |

The failure is real and pre-existing (see companion issue). It is invisible in
CI purely because CI does not have `.uor-models/compiled/`. I hit this directly:
a fresh `git worktree` lacks the artifacts, and the first "pass" I recorded there
was vacuous — the 50× runtime difference was the only signal that anything
differed.

## Why this is a class, not a one-off

AGENTS.md already documents the same failure mode under "Things that bite":

> `/tmp/ref/out/model.bin` disappears on reboot/periodic /tmp cleanup — κ tests
> skip silently and report vacuous green.

So this pattern is known for κ tests and has now been observed in a second
suite. A test that silently degrades to "pass" when its fixtures are missing is
worse than no test: it actively certifies the thing it stopped checking.

## Proposal

A policy, applied across fixture-dependent suites:

1. **Never silently pass.** A fixture-dependent test with no fixture must either
   hard-fail, or report explicitly skipped through a channel that is visible in
   CI output and distinguishable from a pass.
2. **Make the intended mode explicit.** An env flag (e.g.
   `UOR_REQUIRE_FIXTURES=1`) that turns absent fixtures into a hard error, set in
   any CI job that is supposed to be exercising them — so "this job really ran
   the invariant" is asserted rather than assumed.
3. **Audit the existing suites** for the same shape: κ tests, the teacher-parity
   BDD suite, `allocation_census`, and anything else resolving paths under
   `.uor-models/` or `/tmp/ref/`. Report which are currently vacuous in CI.

Item 3 is the immediate value — the honest current answer to "which of our
certified invariants are actually being checked in CI?" is unknown.

## Note

Point 1 is deliberately not "always hard-fail": some suites are legitimately
optional locally. The requirement is that the *outcome is legible* — a skip must
never be reported as a pass.


## Comments (2)

### comment 5160888929 — 2026-08-02T23:36:06Z — auser

## Audit result: one suite, not a class. Correcting the scope I claimed.

I filed this saying the honest answer to "which certified invariants are actually
checked in CI?" was *unknown*, and implied the pattern was widespread. Measured,
it isn't. The repo's convention is followed almost everywhere.

### Method

Worktree at `origin/main` with **no** `.uor-models/compiled/`, and
`TLESS_CHECKPOINT=/nonexistent/ci-sim/model.bin` to simulate the absent
`/tmp/ref` checkpoint (which exists on my machine but not in CI). Ran
`cargo test -p <crate> --tests -- --nocapture` for `uor-r4-core`,
`uor-r4-graph-runtime`, `uor-r4-graph-compiler`, `uor-r4-router`.

### Result

**Zero `skipping:` lines were printed by any suite.** The
`eprintln!("skipping: …"); return;` bodies I found by grep sit behind `#[ignore]`
attributes that fire first, so those tests never enter the vacuous path in a
default run.

Every fixture-dependent test that cannot run reports as `ignored`, which libtest
displays distinctly from a pass — most with a reason:

```
test kappa_reproduction ... ignored
test dump_baseline_kappa ... ignored
test tla7_resid_kernel_plain_witness ... ignored
test convert_real_models_pair ... ignored
test test_dump ... ignored
test exports_runtime_tokenizer_with_text_roundtrip ... ignored, requires the downloaded SmolLM2 tokenizer
test loads_real_safetensors_and_runs_teacher_forward ... ignored, requires the downloaded 257 MiB SmolLM2 source
test fixture_corpus_end_to_end ... ignored, release-only fixture workload
test hopf_sector_occupancy_d3 ... ignored, D3 corpus measurement; run explicitly per issue #303
```

That is correct, legible behavior. An ignored test is visibly not-run.

### The one exception

`allocation_census` — **`test result: ok. 1 passed` in 2.10s** with artifacts
absent, having asserted nothing. It has no `#[ignore]`; it runs, hits
`let Ok(bytes) = std::fs::read(&path) else { … }`, and returns success. With
artifacts present it runs ~100s and does real work.

### Revised proposal

Drop the "policy across suites" framing and the audit item — both are answered.
What remains is one suite that does not follow a convention the rest of the repo
already follows. Either:

- add `#[ignore]` with a reason, matching its neighbours; or
- hard-fail when artifacts are absent, if this suite should be mandatory in the
  jobs that have them.

The second is more appropriate given what it guards, ideally gated on an
explicit `UOR_REQUIRE_FIXTURES=1` in the CI job that provisions artifacts, so
"this job really exercised the invariant" is asserted rather than assumed.

### Note on the AGENTS.md precedent

The docstring warning about κ tests reporting "vacuous green" still stands as
written, but the κ tests themselves are `#[ignore]`d and report honestly today.
Retitling this issue to match the measured scope.


### comment 5160918852 — 2026-08-02T23:44:44Z — auser

## Closing as invalid — the premise was wrong. This suite never passed vacuously.

I filed this claiming `allocation_census` reports success without asserting
anything when artifacts are absent, inferring it from a 1.9s vs ~100s runtime
gap. That inference was wrong, and the follow-up audit comment repeated it.

Run on a bare checkout with no `.uor-models/compiled/`, the census executes
every assertion:

```
[parse] artifacts: …/tests/fixtures/tless_artifacts.bin (1346836 bytes, vocab 32000) → 24 allocations, 1347208 bytes
[parse] store: …/.uor-models/compiled/smollm2-135m-instruct/tless_store.bin not present; using a synthetic store
[runtime] steady state: assign_window + predict + predict_witness + generate_greedy_into(32 tokens) → 0 allocations, 0 bytes
[runtime] #318 residual assign (plain + kernel, synthetic TLA7) → 0 allocations, 0 bytes
[r4g1 kernels] predict_token + predict_distribution + predict_candidates + step × 10 → 0 allocations, 0 bytes
test result: ok. 1 passed
```

The runtime difference is **fixture size**, not skipping: vocab 32000 against a
synthetic store versus the 53,779-key real store. `parse_artifacts_measured`
documents the behavior explicitly —

> Prefers the real compiled smollm2 artifacts; falls back to the repo fixture so
> the census still runs on a bare checkout.

— which is deliberate and correct. The `let Ok(bytes) = … else { continue; }` I
flagged is a fallback between candidate paths, not an early return from the test.

### What was actually happening

One defect, already fixed in #355:

- **fixture store** → ~2s → never crosses libtest's 60s threshold → passes.
- **real store** → ~100s → crosses it → `get_timed_out_tests` allocates 594
  bytes → process-wide gate charges them to a measured section → spurious
  failure (#353).

So the census was always running and always correct; only the instrument was
broken, and only in long runs.

### The audit's real finding

The audit is still worth having recorded: across `uor-r4-core`,
`uor-r4-graph-runtime`, `uor-r4-graph-compiler` and `uor-r4-router` with
artifacts absent and `TLESS_CHECKPOINT` pointed at a nonexistent path, **zero
`skipping:` lines were printed**, and every fixture-dependent test that cannot
run reports as `ignored`, mostly with reasons. The repo's convention is sound and
consistently followed. There is no vacuous-green class here.

### One narrow, true residual

CI exercises the census against the fixture plus a **synthetic** store, so the
real compiled-store path is never covered there. That is a mild coverage
observation, arguably by design, and explicitly *not* the defect this issue
alleged. Not opening a follow-up for it unless someone wants it.

Apologies for the noise — closing.


---
_Comment/review ids in this file: 5160888929, 5160918852_

