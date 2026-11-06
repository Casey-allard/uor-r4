# LexLean-R4 transfer audit

Audited `casey-allard/lexlean-r4` at `origin/main` `1135694`, against
UOR-R4 `origin/main` `ff5156bcc` and the one track in
[`one-model-end-to-end`](../../../../roadmap/items/one-model-end-to-end.md).

This is an engineering audit, not a language-model comparison. LexLean has a
working browser product and useful exact-memory machinery. It has not published
a held-out next-token or complete-reply grade showing generalized language
capability.

## Accept now

### Teacher-forced query cache

`gpu/train_uorlut_v8.rs` walks the model over human turns, accumulates each
target token's preceding hidden state, and fits its writer row to the mean state
for that token. UOR-R4's active roadmap already names this as the centroid-refit
step after Claude-297's data result.

This unit ports only the useful separation: the active FBS scorer writes the
exact query rows, targets and frozen decisions once, then the workstation CPU
fits and evaluates output rows without another trunk pass. It does not change
the trunk, optimizer, writer admission or serving defaults.

The LexLean implementation is not copied literally. It covers only the first
256 turns and blends each new mean with the old row at one eighth without a
held-out result. This cache keeps every requested split, binds the exact
checkpoint and corpus, preserves panel order, and reports original/centroid/
matched-control evidence separately.

## Worth studying next

### Canonical point anchors and packed reply memory

`crates/wasm/src/lut_stack_exec.rs` stores a PointKey row with a canonical reply
turn anchor and packs causal reply continuation into
`memory.reply.packed12`. Its audit refuses orphaned point rows.

UOR-R4 already has active point-key and integer-memory work, so this is not
accepted as a drop-in implementation. The transferable idea is the
idempotency/provenance contract around an admitted memory row:

- identify a row by canonical provenance, not insertion timing;
- make exact retries no-ops;
- preserve an explicit reply-anchor relation;
- refuse rows that point to absent continuation data.

This should be evaluated only inside the active M2/M3 memory track. It does not
justify replacing the current integer writer or memory format.

### Strict-update admission

LexLean's `docs/LEARNING.md` checks that an update is retained only when its
loss strictly falls and preserves epoch and binding identity. UOR-R4 already has
stronger CPU whole-path transaction and replay work. Reuse the test question,
not a duplicate update engine: every proposed output/memory move should retain
evidence for the strict-decrease predicate, the matched control and exact replay.

### Browser delivery

LexLean demonstrates a live browser surface, source-tagged output and indexed
binary delivery. UOR-R4's Studio and WASM work are already owned by Kimi, so this
is product evidence and a parity checklist, not something to port into the
active trainer.

## Do not port

- **The `/8` runtime as a UOR-R4 model.** Its `UORLUT01` artifact is a fork
  dialect. It is not the canonical UOR-R4 `/8` format and cannot be treated as a
  compatible checkpoint.
- **The sigmoid lookup path.** `stack_sigmoid_q15` and `SIGMOID_LUT` feed the
  recurrence in `lut_stack_exec.rs`. UOR-R4's serving contract bans this
  exponential squash path.
- **Heuristic "proof" as training.** `xtask/src/engine.rs` promotes claims using
  word counts, stem counts, reply length, cycles and windows, then compiles the
  authored prompt/reply pair into memory. That is deterministic curriculum
  ingestion, not evidence that the model learned a new reply.
- **Literal taught prompt/reply memory as generalization.** A prompt repeated
  after teaching can be answered from the inserted row while an adjacent
  history question fails. This establishes exact memory, not language ability.
- **Generated writes directly to the deployed artifact.** A self-updating
  workflow can bypass model-first review. Any UOR-R4 transfer must use the
  existing checked proposal, strict-loss and replay path.
- **Large committed WASM/binary artifacts.** They obscure source identity and do
  not advance the current trainer.

## Current consequence

The query-cache/readout instrument is the only immediate code transfer in this
unit. Point-anchor admission and rank-margin work remain bounded follow-ups on
the active roadmap, not parallel training tracks.
