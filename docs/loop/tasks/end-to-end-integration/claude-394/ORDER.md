# ORDER: end-to-end-integration, claude-394 (bring ADR-0007 in line with the charter, and point I1 at D86 step 6)

Lab: claude. Item: [end-to-end-integration](../../../../roadmap/items/end-to-end-integration.md).
Base: `origin/main` at `74d487308`, which holds codex-1 (#4012) and codex-2 (#4013).

## Why this unit

Casey asked for a zoomed-out read of Codex's end-to-end-integration line. The read is posted on [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6903557510). Then Casey asked Claude, on 15 Nov 2026, to "post the message to codex and then do the steps you have recommended".

The read found four places where ADR-0007 disagrees with rules already in force:

1. **"Training may use float"** (§1, the line under the stage table). CHARTER §3.7 and D85 keep gradients and optimizer updates in the native integer representation, and allow float only in separately scanned reference code.
2. **JEPA → `Loss`.** `crates/uor-r4-core/src/native_geometric/learner/jepa_trainer.rs` is `L_CE + λ·d_FS + λ_fiber·d_U(1)` with an f64 Adam. Cross-entropy is named in §3.2's ban.
3. **RoPE → `Position`.** The mechanism board (`docs/mechanisms/README.md`, lines 26 and 196) carries position as SpiralCore transport spin, "no RoPE".
4. **An owner direction of 2026-11-15** is cited (e-means, CIRISKernel), but it has no row in PLAN.md's owner directions or in DECISIONS.md.

The read also found that the unit sequence puts five interface and adapter units before any reply can move. D86's review names two gaps, data and a read that averages the fact it found, and I1–I6 touch neither. L-008 is this shape.

## What changes

`docs/reference/adr/0007-stage-interfaces-for-end-to-end-integration.md`, as a dated amendment section plus the in-place corrections it names:

- **Training** follows §3.7/D85: one native state, the same typed operators, float as reference only.
- **RoPE** leaves the `Position` stage. Transport spin stays the only position mechanism. Reopening it would need an owner decision.
- **JEPA** is parked as a `Loss` arm until it has a cross-entropy-free form whose loss, gradient and update are native. Its latent term (`d_FS`, `d_U(1)`) may be re-proposed in that form.
- **I1 is narrowed** to the one `Read` interface that D86 step 6 needs, with the native Q31 first-pick read (`Q31FirstPickRead`, claude-389 to claude-393) as its reference implementation. The Hamming copy and ask heads follow it, and the flock, route and relation adapters wait for a second consumer.
- **I3 (e-means) and I5 (loss) are deferred** until one model replies under D86 step 6 (step 6 of the D86 review's path in PLAN.md), and **I4 (position) is withdrawn** with RoPE.
- **I2 must pass the codex-3 witness pair** (PR #4015's sum-versus-top-k histories), or its successor, as a required gate, and so must any future compaction or summary.
- **The cited 15 Nov direction** is marked unrecorded until its verbatim words land in PLAN.md.

`docs/roadmap/items/end-to-end-integration.md`: the done-when follows (I3 and the loss arm become follow-ups after I6), and the scope gains `crates/uor-r4-native-vector/src/**`, where I1's reference type lives.

## Files in scope

- `docs/reference/adr/0007-stage-interfaces-for-end-to-end-integration.md`
- `docs/roadmap/items/end-to-end-integration.md`
- `docs/loop/tasks/end-to-end-integration/claude-394/**`

Out of scope:
- any code, schema, test, mechanism status or GPU run;
- PR #4015 and its files;
- the charter, D85 and D86.

## Done when

1. None of the ADR's lines allows float training, a cross-entropy loss or a RoPE arm.
2. I1 names `Q31FirstPickRead` as the reference, I3 and I5 are deferred until D86 step 6, and I4 is withdrawn.
3. The witness pair is a named gate of I2.
4. The item file matches the ADR.
5. The docs link check, `loop pr-check --local`, `git diff --check` and a fresh-context review at the exact head pass.

## Stop and ask if

- the 15 Nov owner direction turns out to be recorded somewhere that releases float training, RoPE or JEPA's loss (then the ADR is right and this unit is wrong);
- Codex objects on #2028 before the merge with work in hand that this amendment would break.
