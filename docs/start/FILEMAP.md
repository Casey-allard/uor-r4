# FILEMAP

One row per directory, two levels deep (the loop page, notes table). A folder without a row
fails, and so does a row whose path is gone. `uor-harness check`'s FILEMAP rule reads this.

| Path | What it holds |
|---|---|
| `archive/` | Retired text and records, byte-identical and never edited |
| `archive/adr/` | Architecture decision records from before the charter |
| `archive/assets/` | Retired images and media assets |
| `archive/catalog/` | Generated catalog pages (from `archive/catalog.jsonl`) |
| `archive/compute/` | Retired compute and pod records |
| `archive/evidence/` | Retired evidence bundles and receipts |
| `archive/explainers/` | Retired explanatory notes |
| `archive/figures/` | Retired figures |
| `archive/front/` | Retired front pages |
| `archive/governance/` | Retired governance and process text |
| `archive/handoff/` | Retired handoff notes |
| `archive/harness/` | Retired harness-era records |
| `archive/history/` | Byte-identical GitHub issue/PR archives |
| `archive/integration/` | Retired integration notes (incl. retired NOW.md pages) |
| `archive/lab-state-20261009/` | The 2026-10-09 lab-state snapshot |
| `archive/labs/` | Retired dated lab records |
| `archive/legacy-cli/` | The retired Python CLI mirror |
| `archive/mechanisms/` | Retired mechanism pages |
| `archive/ops/` | Retired operations records |
| `archive/plans/` | Retired plans |
| `archive/prototypes/` | Retired prototypes |
| `archive/r4_isolated_runtime_readiness_1096_evidence/` | Evidence bundle for run 1096 |
| `archive/r4_native_bridge_1102_evidence/` | Evidence bundle for run 1102 |
| `archive/r4_retained_assembly_1094_evidence/` | Evidence bundle for run 1094 |
| `archive/r4_retained_comparison_1094_evidence/` | Comparison evidence for run 1094 |
| `archive/r4_text_clause_adapter_1094_evidence/` | Clause-adapter evidence for run 1094 |
| `archive/relation-pools/` | Retired relation-pool records |
| `archive/research/` | Retired research notes |
| `archive/scripts/` | Retired scripts (reference only) |
| `archive/storage/` | Storage and cold-archive index records |
| `archive/superpowers/` | Retired superpowers-skill records |
| `archive/t2/` | Retired T2-era records |
| `archive/tests/` | Retired test fixtures and suites |
| `archive/tools/` | Retired tooling |
| `archive/transformerless/` | Retired transformerless-line records |
| `archive/vendor-snapshots/` | Vendor state snapshots |
| `conformance/` | Conformance fixtures |
| `crates/` | The Rust workspace |
| `crates/repo-conformance/` | Repository conformance checks |
| `crates/repo-model/` | The repository model crate |
| `crates/uor-r4-api/` | The API crate |
| `crates/uor-r4-claim-wording/` | The claim-wording rule crate |
| `crates/uor-r4-core/` | Core geometric types and anchors |
| `crates/uor-r4-discrete/` | The discrete SpiralCore/FBS model and exporter |
| `crates/uor-r4-float-train/` | The D37 scanned offline float training module (`24`-kernel geometric stack extract) |
| `crates/uor-r4-graph-certify/` | Graph certification |
| `crates/uor-r4-graph-cli/` | Graph CLI |
| `crates/uor-r4-graph-compiler/` | Graph compiler |
| `crates/uor-r4-graph-format/` | Graph format |
| `crates/uor-r4-graph-runtime/` | Graph runtime |
| `crates/uor-r4-harness/` | The multi-lab loop harness (`uor-harness`) |
| `crates/uor-r4-integer/` | The integer serving stack and D11 evaluator |
| `crates/uor-r4-lut/` | The UORLUT01 container format |
| `crates/uor-r4-model-source/` | Model-source loading |
| `crates/uor-r4-naf/` | The NAF crate |
| `crates/uor-r4-object-store/` | The exact object store and durable memory |
| `crates/uor-r4-pod-provider/` | Pod provider bindings |
| `crates/uor-r4-proof-model/` | The proof model |
| `crates/uor-r4-recon-dag/` | The reconciliation DAG |
| `crates/uor-r4-router/` | The router crate |
| `crates/uor-r4-simd/` | SIMD helpers |
| `crates/uor-r4-source-interface/` | The source-interface crate |
| `crates/uor-r4-tokenizer/` | The byte-level BPE tokenizer and trainer |
| `crates/uor-r4-training/` | Training driver, grader and evaluation tooling |
| `crates/uor-r4-wasm-router/` | The WASM router |
| `crates/uor-r4-workbench/` | The workbench crate |
| `data/` | Data fixtures and panels |
| `data/panels/` | Frozen evaluation panels and their MANIFEST |
| `data/tokenizers/` | Tokenizer artifacts |
| `docs/` | The documentation tree |
| `docs/compute/` | Compute and pod documentation |
| `docs/contributing/` | Contributor guides |
| `docs/figures/` | Documentation figures |
| `docs/geometry/` | The geometry vocabulary pages |
| `docs/harness/` | Harness docs: gates, prep sheets, labs, mistakes, rule sources |
| `docs/integration/` | Integration notes, DECISIONS, project track |
| `docs/labs/` | Dated lab records (one folder per line of work) |
| `docs/loop/` | The loop page, task folders and templates |
| `docs/mechanisms/` | The mechanism board and per-mechanism pages |
| `docs/model/` | Model architecture documentation |
| `docs/plans/` | Planning documents |
| `docs/products/` | Product pages (incl. the web client) |
| `docs/reference/` | Reference pages (storage, glossary, formats) |
| `docs/research/` | Research notebooks, lines, findings and open questions |
| `docs/reviews/` | Review records |
| `docs/roadmap/` | The roadmap: items, rendered page, issues map |
| `docs/rules/` | Rules changelog |
| `docs/start/` | Start-here docs and FILEMAP |
| `docs/transformerless/` | The transformerless line's documentation |
| `docs/wasm-serving/` | WASM serving documentation and its verifier |
| `projects/` | The small side projects (indexed, not the model) |
| `projects/ai-research/` | The ai-research side project |
| `projects/msa/` | The msa side project |
| `projects/prime-analysis/` | The prime-analysis side project |
| `projects/research-archives/` | The research-archives side project |
| `projects/riemann-lean/` | The riemann-lean side project |
| `projects/spiralcore/` | The spiralcore side project |
| `projects/wasm-gemm-gnaf/` | The wasm-gemm-gnaf side project |
| `rules/` | Additional rule fixtures |
| `scripts/` | Repository scripts (Rust-first; no new Python) |
| `scripts/pod/` | `uor-pod` pod management |
| `scripts/research/` | Research scripts (legacy; read-only) |
| `scripts/storage/` | Storage and hygiene scripts (`uor-hygiene`, research-data) |
| `src-tauri/` | The Tauri desktop client shell (infra-desktop-client-tauri) |
| `src-tauri/capabilities/` | Tauri capability grants for the desktop client |
| `src-tauri/src/` | The desktop client's Rust sources |
| `src-tauri/tests/` | Desktop client control tests |
| `tests/` | Repository-level test fixtures |
| `third_party/` | Vendored third-party material |
| `tools/` | Additional tools |
| `tools/r4-softmax-trainer/` | The retired r4 softmax trainer (reference) |
| `tools/uor-knowledge/` | The uor-knowledge tool |
| `web/` | Web assets |
| `web/studio/` | The Studio web client (M7) |
| `xtask/` | The `cargo xtask` automation crate (roadmap render/sync, links, catalog) |
| `xtask/src/` | The xtask sources |
| `conformance/features/` | (row added by the migration's own verifier; description to sharpen on use) |
| `tests/e2e/` | (row added by the migration's own verifier; description to sharpen on use) |
| `third_party/arrayref/` | (row added by the migration's own verifier; description to sharpen on use) |
| `third_party/candle-core-0.9.2/` | (row added by the migration's own verifier; description to sharpen on use) |
| `third_party/candle-kernels-0.9.2/` | (row added by the migration's own verifier; description to sharpen on use) |
| `third_party/candle-nn-0.9.2/` | (row added by the migration's own verifier; description to sharpen on use) |
| `third_party/cudarc-0.19.10/` | (row added by the migration's own verifier; description to sharpen on use) |
| `third_party/float8-0.6.1/` | (row added by the migration's own verifier; description to sharpen on use) |
| `third_party/helm-d-reference/` | (row added by the migration's own verifier; description to sharpen on use) |
| `third_party/uor-addr-0.2.0/` | (row added by the migration's own verifier; description to sharpen on use) |
| `tools/lab-runner/` | (row added by the migration's own verifier; description to sharpen on use) |
| `.uor-models/` | Local model artifacts (tracked) |
| `.uor-models/corpora/` | Local corpus artifacts |
| `.agents/` | Configuration and agent state |
| `.agents/skills/` | Configuration and agent state |
| `.cargo/` | Configuration and agent state |
| `.codex-lab/` | Configuration and agent state |
| `.devcontainer/` | Configuration and agent state |
| `.github/` | GitHub workflows and templates |
| `.github/ISSUE_TEMPLATE/` | Configuration and agent state |
| `.github/workflows/` | CI workflows |
| `.kimi-code/` | Kimi Code agent configuration |
| `.kimi-code/agents/` | Kimi Code agent definitions |
