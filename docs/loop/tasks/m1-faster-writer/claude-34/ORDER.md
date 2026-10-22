# ORDER: m1-faster-writer, claude-34

Lab: claude. Item: [m1-faster-writer](../../../../roadmap/items/m1-faster-writer.md). Base: `fb6df367b`.

## Goal

The item's done-when reads:

> A writer whose measured per-row comparison count is below the full-vocabulary scan (about 5×10⁶ today), with finite controls that match the full-scan writer, and a CUDA qualification. How far below the scan counts as enough is not yet registered; register it in step 4.

This unit covers steps 2 and 3, host only:
- **The design note,** posted on #2840 (comment 6346787358). It proposes a CSR bucket index over the 256 root-pair address cells, which replaces the per-row `token_cell` scans: V + K·V calls today, 0 bucketed. Admission becomes a one-pass counting sort on the 5-valued address key. The writer picks the bit-identical token, and serving prunes by the exact bound `address_distance(cell) ≤ token_distance` for members.
- **Finite CPU controls** that prove it on small vocabularies.

The planted fault: **a bucketed writer that differs from the full scan on any tie, any ABSENT token or any virtual move.**

## Cancel probe (run first)

At `fb6df367b`, does `crates/uor-r4-discrete` already have a host bucketed writer, or a per-cell token index? If it has one, this unit reuses it. Class: instrument.

## Files in scope

- `crates/uor-r4-discrete/tests/writer_bucket_controls.rs`, a new test-only host control. It adds no library writer and no CPU fallback.
- This task folder

## Done when

- A host reference of today's full-scan writer mirrors `discrete.cu:270-345`: roots, `token_cell`, `address_distance`, `token_distance`, `cell_precedes`, K admission, ABSENT returning MAX, and the `kind==1` single-token move.
- A host bucketed writer (CSR index, counting-sort admission, id-ordered bucket walk, serving prune) picks the identical token on seeded small vocabularies. That includes every cell-id and token-id tie, an ABSENT token, a move across cells, and pruned-against-full serving.
- A counter test pins 0 `token_cell` calls and 256 cell evaluations per row for the bucketed writer.
- **Negative gates:** breaking the id tie rule, or the prune bound (≥ in place of >), fails a test.
- `cargo test -p uor-r4-discrete --test writer_bucket_controls`, `report --check` and `pr-check --local` pass, and clippy reports no new finding. The crate already carries 67 clippy warnings on `main` under the pinned toolchain; they are out of this item's unit.

## Stop and ask if

- Mirroring the kernel needs a library change in `crates/uor-r4-discrete/src`.
- The kernel's semantics differ from the brief: the argmin over admitted cells, or ABSENT returning MAX.
