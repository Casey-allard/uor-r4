# ORDER: object-store-consumer-m3, deepseek-63 (record the owner-registered M3 thresholds)

Lab: deepseek. Item: [object-store-consumer-m3](../../../../roadmap/items/object-store-consumer-m3.md) (#2873). Base: `690fafc4c`. Partial unit: `References #2873`; the item stays `ready` (model selection, FIT and typed release still open).

## Goal

The frozen-panel pre-registration still says it "awaits owner ratification of the CHARTER §5.2 bar decision card", but the owner already registered card `6339008955`: Casey selected "Yes, register all six (Recommended)" at **2026-10-23T02:36:36Z** (owner decisions comment `6356034555`). Repair the record: mark the card registered with the quote, time and link; restate the registered bar text verbatim; keep the sheet **DRAFT** for the remaining prerequisites (M1/M2 model selection, non-author FIT of the frozen configuration, Casey's typed release). No threshold is changed, and registration releases no compute.

## Cancel probe (run first)

At base `690fafc4c`: `docs/labs/object-store-consumer-m3/README.md` lists card `6339008955` only as a proposed decision card and its status line says "awaits ... owner ratification", while #2028 comment `6356034555` (2026-10-23T02:36:36Z) registers exactly that card. Task stands (record repair only).

## Files in scope

- `docs/labs/object-store-consumer-m3/**`
- this task folder

## Done when

- The README marks the card owner-registered with the owner quote, ISO UTC time and comment link; the registered bar text is reproduced verbatim; the table is labelled as restating the registration (registered text governs).
- The status remains **DRAFT**, naming the remaining prerequisites (M1/M2 model selection, non-author FIT, typed release), and the stop conditions keep the frozen SHA-256/Manifest pin requirement.
- No threshold value, bar, hold, item status, code or model change; `loop report --check`, `loop pr-check --local` and `loop gates --full` pass; a counted non-deepseek review records at the exact head.

## Stop and ask if

- A threshold value, bar, hold, code or file outside the listed scope would change.