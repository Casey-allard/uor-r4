# ROADMAP stale tracker rows corrected 2026-10-18

Phase 1 of the D39 tracker-hygiene queue names concrete stale rows in `ROADMAP.md`: the D32
learning-permission paragraph predates D37, the M7 row still says `not started` after the WASM runtime
landed in #2654, and closed bugs are listed as open. This docs-only correction fixes those four rows.
No code, model, gate or evidence change.

## Change

- Section 0: D32 keeps the discrete path integer/GPU-only; D37 now allows offline float training in a
  separately scanned geometric module with the export gap reported, while D25 matmul/squashing and D24
  GPU-only rules stand. Training remains held.
- M7: `runtime delivered; model bundle not integrated`, pointing at the #2654 WASM runtime and its
  zero-matmul audit record; the selected model bundle is not yet reachable.
- `#820` owner quote annotated as closed (2026-10-09) and historical.
- Standing issues: `#1542/#1476/#1718/#1738` are closed historical citations, not open bugs.

## Verification

- Four anchored replacements in `ROADMAP.md`; `grep` confirms the new D37/M7/#820/closed-citation text.
- Docs-only; no Rust paths touched, no compute, $0.