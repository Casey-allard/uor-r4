# ORDER: m1-address-as-read, kimi-33

Lab: kimi. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md). Base: `38bec04ba`.

## Goal

The item's done-when reads:

> A read that resolves an address to stored content from the object store, with finite tests, and a registered serving-time oracle test that measures the gain from supplying the true address.

## Cancel probe (run first)

The address-read consumer and six finite controls are merged, and the owner registered the 40 stated-fact / 20 historical-version / 20 abstention composition at 2026-10-23T02:44:51Z. The sheet still has `SHA-256: PENDING` and no 80-row artifact. This is a PM instrument freeze, not a mechanism test. Do not redo the consumer, re-review FIT, or touch the held checkpoint.

## Files in scope

- `crates/uor-r4-object-store/**`
- `docs/mechanisms/exact-memory.md`
- `docs/labs/m1-address-as-read/**`
- `docs/harness/prep/m1-address-as-read*`
- `docs/loop/tasks/m1-address-as-read/kimi-33/**`

## Done when

A deterministic Rust generator emits exactly 80 canonical JSONL rows (40 current stated-fact, 20 previous-version, 20 abstention/evicted), validates every expected outcome through `AddressDirectory` and `Store`, and reproduces byte-identical output. The sealed bytes and SHA-256 are uploaded to `caseyallard/uor-r4-store/data/m1-address-as-read/` and read back identically; the sheet and lab record pin that hash and source rule. Finite object-store tests, full gates, REPORT and local PR checks pass, followed by another-vendor exact-head review. The sheet stays `NOT_FIT` for the remaining checkpoint/FIT prerequisites and the item stays `ready`; no GPU run or M2/M1 hold release occurs.

## Stop and ask if

A bar, checkpoint, FIT verdict, hold, item status or external destination must change; a non-deterministic or corpus-derived panel source is needed; or any file outside the stated scope must change.
