# ORDER: cpu-training-pilot, deepseek-138 (generated-state terminal rows)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `11f8303cc`.

## Goal

Test whether adding BOS as a content-readout target when an actual generated
training trajectory reaches its reference endpoint can terminate held-out
replies without losing the modulo-256 content gain.

Deepseek-137's code readout sharply improves aligned content and depth but
every reply still hits the token cap. Deepseek-132's teacher-forced stop
classifier found reference endings yet stopped generated replies prematurely.
This unit uses the same generated-state code signature for both content and
the reference endpoint.

This is D84 CPU development research. It does not change integer serving,
CUDA, float training or Claude's track.

## Cancel probe

Deepseek-137 is merged at `11f8303cc` with a positive code readout on both
public seeds and zero exact/BOS-complete replies. No other open order uses a
generated-state terminal row. Claude PR #3664 changes only its scoring sheet
and ORDER.

## Fixed construction

Use the deepseek-137 98/103 split, training-derived memory and 2,048-bit
modulo-256 query code.

The baseline code table contains one row for each generated decision whose
offset is inside the reference reply. The stop-augmented table adds one row
with target BOS when a generated training trajectory reaches exactly the
reference length. Rows after the reference length are not added.

Fit each table's training-only radius over at least 90% of leave-one-out rows.
Decode chooses the commonest minimum-distance target, breaks ties to the
smaller token and falls back to the exact writer beyond the threshold.
Target BOS ends the reply through the existing stop contract.

## Arms and bar

Run writer, code, code-shuffled, code plus endpoint rows and
code-stop-shuffled over complete 16-token generations. The stop-shuffled
control uses a deterministic effective-map-changing target rotation over the
same augmented rows.

Both seeds must pass:

- baseline and augmented tables retain at least 640 rows, 16 targets and 90%
  leave-one-out coverage;
- both shuffled controls change their effective signature maps;
- code-stop aligned top-1 and matching depth are not below ordinary code;
- code-stop BOS-complete replies exceed code and code-stop-shuffled by the
  prompt-level rule `candidate_only >= max(2, ceil(3 * sqrt(control_only)))`;
- at least **8 of 103** code-stop replies exactly match the reference and stop
  at BOS; the paired BOS rule makes **9** the effective minimum when the
  control emits none;
- at least **8 of 103** code-stop replies stop at BOS and are not longer than
  their reference; and
- seed-1 repeat matches every non-timing field and generated evidence file.

Report exact, BOS-complete, premature, cap and context stops, top-1/depth,
openings, readout/fallback decisions, distinct/repeated replies, workers,
wall and RSS.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/claims.json`
- `docs/harness/prep/cpu-training-pilot-content-stop-2026-11-04.md`
- `docs/labs/cpu-training-pilot-content-stop-2026-11-04/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-138/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No integer-session, serving, CUDA, float-training or Claude-owned changes.

## Done when

1. The readout can derive and evaluate an optional reference-end BOS row while
   preserving the ordinary code arm.
2. Tests cover endpoint inclusion, no post-reference rows, stop execution,
   shuffle refusal, content non-inferiority and deterministic evaluation.
3. FIT and a fresh GO precede both seeds and a repeat.
4. REPORT and hashed evidence preserve either outcome; notebook and roadmap
   classify generated-state stopping.
5. Exact-head review, protected merge and fresh-main verification complete.

## Stop and ask if

- Endpoint rows cannot preserve at least 640 rows or 90% coverage.
- The stop shuffle leaves the effective map unchanged.
- Evaluation requires serving, integer-session, CUDA or Claude-owned changes.
