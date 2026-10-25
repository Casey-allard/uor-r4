# ORDER: m1-w2-capacity-capture, claude-87 (W2 read-code capacity, through an exact-proven importer)

Lab: claude. Item: [m1-w2-capacity-capture](../../../../roadmap/items/m1-w2-capacity-capture.md) (#2845). Base: `0e7591592`.

## Goal

Run the registered W2 sheet once: `docs/harness/prep/m1-read-code-capacity-2026-10-17.md`, FIT at #2572. It records H_rep, H_law, H_mixed, writer-dominates or UNDERPOWERED as one notebook row, with the model-matched unigram U, the query-token bigram G (the next-word table) and the code table T.

The pinned Q4 checkpoint (`4477eb5c…`, 15 Oct) is refused by today's code, because the driver lineage and the snapshot's source and kernel hashes have all moved since then. Casey chose the route, 25 Oct, by 2026-10-25T02:50:00Z: "Importer, proven exact (Recommended)".
1. **Importer.** An evaluation-only importer in `uor-r4-discrete` loads Q4's state into today's device with the legacy defaults (no position, the existing normalization default). It can only run forward: no apply, save or export. It records the original and target hashes, which are the verifier's `original_to_target_bridge`.
2. **Gate R, run on the 5090 before anything else.** The imported Q4 forward over DEV must reproduce the sealed Q4 evaluation (`4fff1d3c…`) exactly, row by row, under the ordered row binding `e4ef44fb…`. If it does not, the run stops as an instrument fault, and no W2 number is produced.
3. **Producer.** The production forward runs over DEV (224 batches) and over the model-matched 1,049,856 TRAIN targets. That is the sheet's §4.5 fallback, which keeps the primary regime, drops only the secondary full-TRAIN table, and is labelled with its bias disclosed. Host counting builds T, U and G and the §4.6 aggregates, and the producer emits `uor-r4.read-code-capacity/1`, checked by `docs/labs/w2-k16-consumer-verifiers-2026-10-19/verify_w2_read_code_capacity.rs`.
4. **Pre-run review.** A fresh adversarial pre-run review (D67) runs before the pod: the whole chain, what each outcome unlocks, the inputs, and the caps.

## Cancel probe (run first)

At `0e7591592`:
- **No producer.** No producer for the record exists (the lab README says so), and `PreparedDriver::restore` and `Device::from_snapshot` refuse Q4.
- **Inputs found, pins checked.** TRAIN `3dee858f…` / `b5e7352d…` and DEV `0919e90f…` / `a838bb1a…` come from the HF store's `data/ft-balp.tar` and `data/ft-dev.tar`; the tokenizer `d36d3e87…` comes from `data/tokenizer.json.tar`. Q4 is in `local-archive/2026-10-20/uor-r4-local.tar`.

The unit stands. Class: instrument, then measurement.

## Files in scope

- `crates/uor-r4-discrete/**` (the importer, the producer, their tests)
- `docs/labs/m1-w2-capacity-capture/**`
- `docs/harness/prep/m1-w2-capacity-capture*`
- `docs/research/notebook/m1-*.md` (one row)
- `docs/loop/tasks/m1-w2-capacity-capture/claude-87/**`

## Done when

- The importer's host tests pass: it refuses a non-legacy config and any apply or save path, and the tests pin both.
- Gate R reproduces the sealed Q4 DEV evaluation exactly on the 5090.
- The record passes the consumer verifier, and the outcome band is recorded as one notebook row. The PR says `Closes #2845`.
- If Gate R or an instrument check fails, the instrument fault is recorded instead, and the PR says `References #2845`.
- A fresh-context review at the exact head counts.

## Stop and ask if

Gate R is not exact; any root decode is `ABSENT`; a pin mismatches; or the cost passes $8.
