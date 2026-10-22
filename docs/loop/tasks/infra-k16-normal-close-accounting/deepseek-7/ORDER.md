# ORDER: infra-k16-normal-close-accounting, deepseek-7 (K16 normal close and final resource accounting)

Lab: deepseek. Item: [infra-k16-normal-close-accounting](../../../../roadmap/items/infra-k16-normal-close-accounting.md) (#2827). Base: `44ace17dc`.

## Goal

Record the K16 run's normal close and final resource accounting in a dated repository record. Done-when: "Normal close and final resource accounting recorded for the K16 run, or the stale statement in NOW removed with a pointer to where it was recorded."

## Cancel probe (run first)

Class **instrument** (records, no mechanism). At base `44ace17dc`: the close is already **complete** per the [front-door record](../../../../labs/k16-frontdoor-close-sync-2026-10-19/README.md) (release 00:38:29Z, native down 00:38:33Z, after-down custody 131,072 B), while three sentences still call it pending (`docs/labs/k16-four-role-execution-2026-10-18/README.md:24`, the `docs/labs/README.md` final-custody box, `docs/integration/NOW.md:81`). The accounting exists only in #2029 comments [6257200048](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6257200048) (34,633 s at $0.908/h = $8.735212222 resource-cost proxy, not an invoice; storage/transfer unmeasured; no idle cost) and [6257225762](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6257225762) (closure custody). No repository record consolidates it; `budget show` reports `ABSENT` for the repository resource ledger. The task stands: deliver the in-scope record, and ask about the out-of-scope sentences.

## Files in scope

`docs/harness/ledger/**`, `docs/labs/infra-k16-normal-close-accounting/**`, `docs/reference/storage.md`, the item's `status:` line, and this task folder. `docs/harness/ledger/**` is left unwritten (per-lab shards; the receipt CLI carries no cost) — see the REPORT.

## Done when

- A dated record under `docs/labs/infra-k16-normal-close-accounting/` states the close and the accounting, one source per fact.
- `resource-ledger.json` beside it carries the same facts machine-readably, and `uor-harness budget show` reads it.
- `docs/reference/storage.md` points to the record and says the storage and transfer charges are unmeasured.
- The three stale front-door sentences are named as outside this scope, with a scope widening or follow-up front-door sync asked for and this record as the pointer.
- `loop gates --full` is GREEN, and `report --check` and `pr-check --local` pass.

## Stop and ask if

- A front-door file must change (`docs/labs/k16-four-role-execution-2026-10-18/README.md:24`, the `docs/labs/README.md` final-custody box, `docs/integration/NOW.md:81`): asked, not edited.
- A lease still needed closing: it does not (release 00:38:29Z and native down 00:38:33Z are recorded; nothing to destroy).
- A figure is absent from the records: storage and transfer stay **unmeasured**; no number is invented.