# REPORT: native-space-state-contract, codex-12

Lab: codex. Order: [ORDER.md](ORDER.md). Design: [DESIGN.md](DESIGN.md).

## What did not work, first

- `loop claim native-space-state-contract --lab-topic native-lane-gates` was refused because this
  item's `docs/model/**` scope overlaps the still-open order m4-export-uorlut01-cli/codex-7. The
  unit therefore lives entirely in its task folder and adds no `docs/model/` file or code.
- This is a **decision unit**, not evidence: it runs no code and no GPU work. The two negative
  configurations it is based on are m4 codex-6 and codex-7:
  - raw writer target (`q12 << 16`): dot `2,476,676,547,100,606,464 → 2,142,903,011,938,598,912`;
  - normalized unit target: dot `18,514,432,582,351,337,035 → 17,116,574,668,483,326,999`;
  - served tokens unchanged in both.
- **The D86 donor pilot is already active on wick under another session**: a `llama-server` running
  `gpt-oss-20b.gguf` (16 GB, port 8086, `/workspace/d41/...`). It is not this lab's workload; it was
  not touched, and this unit does not compete with it or duplicate the pilot.
- A per-lane gate has only 14 records per lane on the tiny fixture, so it can overfit; and the
  alignment dot is a proxy for the token objective, so the served tokens may still not move even
  if per-lane alignment improves. Both are recorded as risks, not hidden.

## What changed

- `DESIGN.md` records the evidence, options A (per-lane gates shared across layers), B
  (per-layer-lane), C (a different objective/update) and D (stop, rejected), and chooses **A
  first** with B and C as explicit fallbacks.
- The design fixes the state and schema: a new `uor-r4.lut-stack/13` with `K = width/4` gates and
  the shared 16-byte cursor (`32K + 16` bytes; 272 bytes at `K = 8`, 2,064 at `K = 64`), the same
  `q31_hamilton_rotate` selected by lane, `K` per-gate AOT CUDA calls, per-lane training and an
  independent per-lane dot. Older engines refuse `/13` by schema.
- The bar, the cancel order (A → B → C), the cost, the risks and the three implementation steps are
  named.
- No production code, artifact, serving, kernel or test change; the implementation is separate.

## Measurements

No new measurement in this unit; it decides between the two measured negatives above:

- raw writer target: dot `2,476,676,547,100,606,464 -> 2,142,903,011,938,598,912`;
- normalized unit target: dot `18,514,432,582,351,337,035 -> 17,116,574,668,483,326,999`;
- served tokens `[125, 236, 10, 186, 8, 161, 92, 205]` unchanged in both.

## Decision review

A fresh-context decision reviewer read the whole project (charter, D85/D86, the native-space
contract, the two negative reports, the current implementation) and returned **APPROVE**: per-lane
gates shared across layers is the correct, smallest-state next configuration, with the A -> B -> C
fallback order; it found no better alternative. Its five findings are folded into `DESIGN.md`:

- **F1:** exactly one shared `(epoch, step_shift)` cursor, advanced once per training round by a
  new `Q31LaneGates` type (the byte shape is not `K` separately serialized models).
- **F2:** a **token-level co-primary** measure in the GPU sheet, so a dot-only improvement is
  reported as proxy-only, not as token or model progress.
- **F3:** the AOT kernel already accepts one gate per batch item; no new kernel is needed.
- **F4:** the dot is explicitly **in-sample independent** (not the training loss), not held out.
- **F5:** per-lane gates are a prerequisite of the state shape, not a sufficient condition for
  consuming the D86 corpus; that also depends on the token objective and the hard-target framing.

## Checks run at the head

- `uor-harness loop report --check`, `loop pr-check --local` and the exact-head decision review —
  at the head.
- No code, test or format gate applies to this docs-only unit; the item's task folder is always
  allowed and the `docs/model/**` overlap is avoided.

## Skills used

- `solution-integrity`: separated the two measured negatives from the inference about the state
  shape, and kept the decision reversible.
- `trace-back`: both target scales failed the same way, so the suspect moved from the scale to the
  state shape.
- `scope-guard`: stayed inside the task folder and avoided the open order's `docs/model/**`.
- `native-space-cycle`: the design keeps one representation, one typed operator, bit-identical
  replay and lossless serialization.

## Still open

- The `/13` implementation (schema, parser, exporter, serving, bit-identical replay) is the next
  unit.
- Then the per-lane token-objective test on the synthetic trunk, then a GPU qualification with a
  sheet and a non-author FIT if the per-lane result is positive.
- D86's donor corpus/pilot belongs to the session already running it on wick; this lab's state
  shape must be able to consume that data once available.

## Plain summary

Two tries at training the one shared 4-word gate from token directions made the alignment worse,
so the likely problem is the shape of the state, not the target. The decision is to give each lane
its own gate first, storing them in a new `/13` artifact section with the same operator and a
per-lane measure; if that fails, per-layer-lane gates, then a different objective. This unit only
records the decision; the code comes next.
