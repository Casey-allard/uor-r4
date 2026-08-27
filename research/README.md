# Research archive

Consolidated research supporting the UOR / uor-r4 programme. This tree is
**not** part of the Rust workspace (see `exclude` in the root `Cargo.toml`)
and is excluded from crate build, lint, and claim-wording gates.

For geometric-decoder work, `research/ai-research/ai-router/router-research/`
is the canonical research snapshot. The `ai-router` copies nested under
`research/prime-analysis/` are historical source snapshots with overlapping
and sometimes divergent results; do not treat three copies as independent
replication or edit them in parallel.

Research becomes active product work only through the
[Geometric Intelligence Programme](../docs/geometric_intelligence_programme.md)
and its live GitHub root, [#820](https://github.com/UOR-Foundation/uor-r4/issues/820).
That programme turns selected results into explicit route, serialization,
attention, inference, correctness, or reasoning contracts. Everything else in
this tree remains preserved research input: useful evidence and mechanism
proposals, but not a claim about what the current engine can do.

The archive is intentionally broader than the active implementation. It keeps
the project’s prime, semiprime, zeta-grid, torsion, Hopf, spherical-harmonic,
golden-radial, Riemannian, networking, graph-compiler, and formal work available
for synthesis. Inclusion here does not promote a formula to a product guarantee;
promotion requires a named consumer and an explicit, witnessed contract in the
active programme.

## Contents

- `ai-research/` — snapshot of the `Sky-Metrix/AI-Research` monorepo: the
  geometric-routing research program (`ai-router/`), the MUDBench agent
  benchmark, `ramsey/`, preprints, and experiment outputs.
- `prime-analysis/` — snapshot of the `Casey-allard/Prime-Analysis` repository:
  spectral / prime-structure research, `theoretical-mathematics/` including the
  `formal-math-lean/` Lean sources and the vendored `UOR-Framework/lean4`,
  routing-infrastructure prototypes, and manuscripts.
- `prime-analysis/photos/` — supporting figure/slide images (Git LFS).

## Provenance and handling

- Original repositories are retained upstream and were not deleted; this is an
  additive consolidation.
- **Full commit history** of each source repository is preserved on the refs
  `archive/ai-research-history` and `archive/prime-analysis-history` in this
  repository (media blobs over 1 MB stripped from those history refs to keep
  the object store lean; full-fidelity media remains in the originals).
- Binary media (images, PDFs, `.npz`/`.npy`, office documents, model blobs) is
  stored via **Git LFS**; text, code, CSV, and Lean sources are stored normally.
- Excluded from this snapshot: committed virtualenvs, `__pycache__`,
  `node_modules`, and OS junk (not research content); and — deliberately — any
  material under a separate confidentiality hold (e.g. unpublished MSA papers,
  personal/legal documents, and packaged artifacts), none of which were tracked
  in the source repositories.
