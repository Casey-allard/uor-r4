# Injective pre-flight FAIL synced into front pages — 2026-10-19

#2733 recorded C2 PASS and the injective pre-flight stop, but NOW/STATUS/README still said
“next: the injective attention test”. This docs-only sync records the actual next state:

- B3_init(P_succ) = **127 / 3,847 < 500**; net G-Cap = **127 < max(500, 38+24) = 500**; per sheet 3 the run was
  **not dispatched**; G-Init/G-Occ PASS; remaining controls NOT_RUN.
- `INSTRUMENT_UNREACHABLE` (instrument reachability), **not** a mechanism negative (D22); nothing trained.
- Successor decision pending: larger K (≈34 at 256 cells for ~13% admission), address-structured injective
  cells, or a partially injective table. C2 PASS at its fixture-only scope is recorded in #2733.

No payload/table/config/executable bytes touched. Docs-only; claim-wording PASS with the existing checker;
no build, no compute, $0. On merge I will verify the merge commit and clean up.