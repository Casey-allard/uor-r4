# No CPU model compute, no non-decisive GPU work, and the softmax ratchet (12 October 2026)

**Milestone:** governance for M1 [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029)
and every other line · **Owner rule**, stated twice in one session.
**Line:** none (governance) · count unchanged · headline unchanged.

## What the owner said

> "no softmax at all anywhere. all softmax needs to be replaced with a geometric
> mechanism we design. no CPU runs at all. rewrite for GPU. do not waste GPU runs
> on bullshit that doesn't matter."

D23 ([#2251](https://github.com/UOR-Foundation/uor-r4/pull/2251)) had already recorded
the softmax prohibition, and it said plainly that it was "not a comprehensive
Rust/CLI interlock". The rule had been written down and the behaviour had not
changed, which is what the owner was reacting to. This change adds the two clauses
D23 did not carry, and - more importantly - the missing interlock.

## What this adds

1. **D24.** Model compute never runs on the CPU. Training, evaluation and every part
   of the forward that scoring, normalization or the value mix touches are GPU work;
   a host-side fallback is a defect to fix with a kernel. A host-fallback execution
   is not mechanism evidence: both the arithmetic and the step time are wrong.
   GPU time is spent only where it can change a decision.
2. **The replacement is a mandate, not a constraint.** Every prohibited softmax site
   is replaced by a designed geometric mechanism - exact `Z[phi]`/zeta-phase
   interference, typed H4/icosian pairing, Lorentz or address geometry, or exact hard
   selection by address - with a prospective TEST FITNESS review before compute.
3. **`scripts/check_no_softmax.py`** - the ratchet. It scans executable source
   (`crates/**` `.rs/.cu/.cuh/.wgsl`) and fails when a softmax site is added;
   `scripts/no-softmax-baseline.json` may only decrease, and `--update-baseline`
   refuses to raise any count. Verified: baseline 145 files / 1649 sites; clean run
   passes; adding one `softmax` token to a new file fails with exit 1.
4. **AGENTS.md and the policy contract** carry the rule where every agent reads it
   first, and `agent-execution-policy.json` carries the machine-readable form.

## Cancelled under the rule (recorded, not a negative)

The M1 #2029 invariant-addressed router campaign was **stopped** with the pod
deleted, at 2 of 3 arm-A seeds:

| Arm | Why it violates the rule |
|---|---|
| A (routed) | softmax over the k=16 kept candidates |
| C (routed + tangent null) | softmax over the kept candidates |
| B (full read) | *is* the full softmax read - a forbidden baseline |
| all | the routed read executes host-side (`cuda_covered()` false when a route is set) |

Retained reading: `icloud:UOR-R4/results/deepseek/router-abandoned-2026-10-12.tar`
(md5 `f411b7d1870afdabcd2195ce27a8c157`) - both completed fits' `report.json`,
`config.json`, `manifest.json`, `attempt.json` and the full run logs. Seed 20261009
reached dev response NLL 6.230 -> 0.311 with pointer hit 0.029 -> 0.697; seed 20261010
reached 6.230 -> 0.272 with hit 0.029 -> 0.703. Model weights were not retained: the
configuration is excluded and the pod volume was non-canonical with no canonical
companion. This is a configuration cancellation under an owner rule, so it is **not**
a mechanism negative and does not count toward D21/D22.

## Limitations

- The ratchet stops growth; it is **not** a claim that project-wide softmax removal is
  complete. 1649 sites remain in executable source at the baseline.
- Dated records under `docs/` are deliberately not scanned: they are evidence, and
  rewriting them would destroy the record.
- The gate sees the string `softmax`. A mathematically equivalent
  normalized-exponential written without that name (D23 forbids it) is not caught.
- Whether the frozen D10-era comparators should be deleted rather than preserved is an
  **open owner question**; this change preserves them as inactive evidence.
