# ORDER: cpu-training-pilot, deepseek-134 (admission headroom)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `70fad10ad`.

## Goal

Follow deepseek-133's measured admission blocker with a CPU-only generated-state
cell-rank census. Test whether the existing K=8 writer can be expanded to the
artifact contract's maximum K=16, and whether ranks 9-16 expose enough
admitted, positive-margin target decisions to resume writer training.

No training updates occur in this unit. This is D84 development evidence, not
GPU qualification, language quality or a serving-contract change.

## Cancel probe

Deepseek-133 found only 30 true / 33 shuffled eligible rows on seed 1 and 32
/ 32 on seed 2 at K=8. Deepseek-111 and -120 diagnose admission, but no run
ranks target cells from the model's generated prefixes or tests the lawful
K=16 ceiling. Claude PR #3664 remains limited to its GPU scoring sheet.

## Fixed construction

Use the public 16-key, even/odd-document split and training-derived memory.
Scan generated and reference-feedback states with the existing exact
`self_prefix` path. For every reference content target:

1. rank its `target_id & 255` cell among all 256 orbit-pair cells;
2. evaluate target admission, positive margin and token top-1 at
   K=1,2,4,8,12,16;
3. repeat with eight deterministic random non-top cells as an equal-budget
   CPU control; and
4. construct a K=16 artifact from K=8 by changing only `shape.kept_cells`,
   preserving every table payload byte.

K=8 is the exact nested control. K=16 must parse and execute through the
same CPU FBS and integer serving path.

## Bar

Both seeds must pass:

- K=16 raises generated-state positive-margin training eligibility from the
  K=8 count to at least **640**, an increase of at least **610** rows, with at
  least **64** positive-margin same-cell pairs;
- K=16 adds at least **128** admitted held-out target decisions over K=8;
- K=16 target admissions and fixed-horizon matching depth do not fall below
  K=8, and teacher-forced top-1 does not fall;
- K=16 admits more held-out targets and positive-margin target cells than the
  equal-budget random-extra control;
- K=8 bytes remain unchanged; K=16 preserves every table payload byte;
- generated K=16 sequences retain exact CPU/integer query, token and
  kept-cell parity; and
- a seed-1 repeat matches every non-timing field and artifact.

Report rank histograms, minimum-K distributions, cost, exact replies and
stops. Exact replies are not a bar.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/claims.json`
- `docs/harness/prep/cpu-training-pilot-admission-headroom-2026-11-04.md`
- `docs/labs/cpu-training-pilot-admission-headroom-2026-11-04/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-134/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No integer-session, serving, CUDA, float-training or Claude-owned source
changes.

## Done when

1. Exact cell-rank and K-value diagnostics run on generated and reference
   states; K16 artifacts are reconstructed deterministically.
2. Tests cover rank order/ties, random-extra non-overlap, artifact payload
   identity, K8/K16 parsing, generated-sequence parity and refusals.
3. FIT and a fresh GO precede both seeds and a repeat.
4. REPORT and hashed evidence preserve the decision; notebook and roadmap say
   whether the 8→16 admission budget is sufficient.
5. Exact-head review, protected merge and fresh-main verification complete.

## Stop and ask if

- Any table byte changes between K8 and K16 beyond the shape header.
- A CPU/integer generated-sequence mismatch appears.
- The random-extra control overlaps top eight or is vacuous.
- The work requires serving, integer-session, CUDA or Claude-owned files.
