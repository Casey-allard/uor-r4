# ORDER: m1-address-as-read, kimi-34 (pin the fresh-V2 BPB stream)

Lab: kimi. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md) (#2833). Base: `742bf59bc`.

## Goal

Resolve only the address-oracle sheet's remaining BPB-stream pin by naming the sealed fresh-V2 token and response-mask artifacts and quoting their full SHA-256 digests plus the seal-report digest from retained evidence. Do not change the checkpoint, bar, tolerance, panel, FIT verdict or any hold.

## Cancel probe (run first)

At `742bf59bc`, prerequisite 5 still says the stream's full digest should be quoted. The retained A8 evidence names `tokens.u16` and `response_mask.u8` with full token/mask digests, and the dispatch/packet records quote the seal-report digest. The pin is absent from this sheet, so the task stands.

## Files in scope

- `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md`
- `docs/loop/tasks/m1-address-as-read/kimi-34/**`

## Done when

- The data-split section names the sealed fresh-V2 `tokens.u16` and `response_mask.u8` artifacts and quotes all three full digests with links to retained evidence.
- Prerequisite 5 records the stream as pinned while the `0.01` tolerance remains the already-registered value.
- The held checkpoint remains PENDING and `NOT_FIT`, no compute is cleared, and no bar/panel/tolerance/hold/item status changes.
- Report/local PR checks, full gates and diff-check pass; another vendor reviews the exact head.

## Stop and ask if

The retained digests disagree, a new artifact must be created or uploaded, a tolerance/bar/checkpoint/hold must change, or a file outside scope is needed.
