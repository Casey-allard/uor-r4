# riemann-lean

Casey Allard's Lean 4 formalization scaffold for the prime / Riemann bridge —
the O1–O5 / L1–L3 theorem chain — together with the research scripts behind it.

- `formal/lean/` — the `PrimeRiemannBridge` theorem spine (23 hand-written
  `.lean` files, no axioms) with `lakefile.toml`, `lake-manifest.json`, and
  `lean-toolchain`. The external Lean dependencies (Lean-RH, PhysLean, Carleson,
  PrimeNumberTheoremAnd, mathlib) are **pinned in `lake-manifest.json`** and are
  fetched with `lake exe cache get` / `lake update`; their vendored source
  copies are intentionally not committed here (third-party, large, regenerable).
- `*.py` — the probes, ledgers, majorant checkers, and proof-obligation tooling
  (the `a1`–`a4`, `o1`–`o5`, `k1`, `tau12`, `fixed_error_psi`, `spinning_top`,
  and related programs).
- `data/` — arXiv reference metadata.

This began as a standalone local project and was not previously in the
AI-Research or Prime-Analysis repositories. Excluded as regenerable or
non-research: the `.lake` build cache, compiled `.olean` artifacts, vendored
Python binaries (`.vendor`), `.npz` computation caches, and unrelated work files.
The RH manuscripts and proof narrative live separately under
`research/prime-analysis/research/theoretical-mathematics/formal-math-lean/`.
