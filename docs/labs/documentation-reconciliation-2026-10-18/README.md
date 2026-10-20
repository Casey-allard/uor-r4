# Documentation and figure reconciliation — 18 October 2026

## Question and scope

Can a contributor find the current implementation and distinguish implemented
mechanisms, measured results, mathematical foundations and future integration?
**Decision: documentation reconciliation; no model promotion or scientific verdict.**
Work began from fetched main `9a343425949939c4d02668c8583c56dedaf1254f`,
which includes the original Dose1 custody record at `c7cfc2e1`.
The [plan](../../../archive/superpowers/plans/2026-10-18-documentation-reconciliation.md)
tracks this bounded deliverable. The parent arranges independent review; delivery is draft-only.

## Primary sources and corrections

- [D37](../../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised): offline floats allowed; matmul/GEMM, softmax/squashing and GPU-only constraints retained; D11 integer export separately qualified. Corrected docs hub, contributor guidance and the stale local skill allowance.
- [`build.rs`](../../../crates/uor-r4-discrete/build.rs): 8D blocks, 16 carrier roots and 64 signed actions; four blocks give width32. [Architecture](../../architecture.md) traces each current data-flow arrow to the implementation.
- [`discrete.cu`](../../../crates/uor-r4-discrete/src/discrete.cu): Flock node scans and Relation sweeps precede bounded selection; the writer scans vocabulary occupancy and admitted-cell tokens. Bounded retained K does not imply bounded selection work independent of context.
- [`prime_zeta.rs`](../../../crates/uor-r4-discrete/src/prime_zeta.rs): C⁴ per block → R⁴ + R⁴ → normalized Q30 lanes → permutation → 16-root carrier alignment with allocation controls.
- [`anchors.rs`](../../../crates/uor-r4-core/src/native_geometric/anchors.rs): coordinates `(a+bφ)/2`, with coefficient pairs distinct from an orthogonal Euclidean E8 embedding. The [geometry guide](../../geometry.md) specifies the reduced inner product, 3D conjugation versus 4D left action, additive-state norm caveat and Hopf's absent M1 integration.
- [Rows parity attempt3 JSON](../counterfactual-rows-parity-2026-10-18/evidence/attempt3/rows-parity.json): 1,080 proposal cases plus 3 refusals and 2 forced terminals = **1,085 finite cases**, zero updates; the ~56× proposal timing has `qualified=false`. Older 1,080 totals in dated records retain proposal-only scope.
- [Original Dose1 record](../attention-dose1-strategic-stop-2026-10-18/README.md): owner-stopped at 904 treatment /1,065 frozen-zero batches, before Q1; evidence preserved and pod down. No mechanism negative.
- Live issue read through [Claude's whole-packet review](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6246433669), [scoped owner dispatch](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6246440910), and [06:24 pod-up/build-staging receipt](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6246536845). The 06:05 “awaiting review/release” audit snapshot is superseded by those comments. No successor model run or outcome was established by the latest retrieved receipt.

## Validation

All commands below ran with the pinned Rust1.97.1 toolchain. Model computation
was not invoked; the default-feature discrete tests are host/metadata/construction checks.

| Check | Outcome |
|---|---|
| `cargo metadata --locked --no-deps --format-version 1` | Core selects `native_support.rs`; integer selects `native_geometry.rs`; discrete selects its own `lib.rs`. |
| `cargo check --locked -p uor-r4-discrete --lib` | PASS; existing dead-code warnings retained. |
| `cargo test --locked -p uor-r4-discrete --lib` | 90 passed, 0 failed, 1 ignored. |
| `cargo run -q --locked -p xtask -- figures --write` then `… figures` | Seven generated Rust figures current; generator fit checks pass. |
| `rustfmt --check --edition 2021 xtask/src/figures.rs` | PASS. |
| `python3 docs/figures/geometry/generate.py` | Regenerates mathematical SVGs; repaired missing `connections` save so source edits reach that figure. |
| `python3 docs/figures/geometry/verify_connections.py` | 29/29 checks pass; finite enumeration and sampled consistency scopes remain distinct. |
| `cargo run -q --locked -p uor-r4-claim-wording -- .` | PASS; wording gate is not a scientific audit. |
| Local Markdown/HTML target and heading-anchor scan | Audited root/STATUS/CONTRIBUTING/AGENTS, docs hub/map/geometry/architecture/labs hub, discrete README, research index, local skill and audit/plan; zero unresolved targets. See [validation.json](validation.json) for final count. |
| Chromium SVG renders, all 22 figures at 1200px and 390px | Current architecture readable on mobile; mathematical vector originals linked for zoom. Corrected figures visually inspected; browser text bounds checked. |
| `git diff --check` | PASS. |

Source-controlled SVGs remain generated vectors. Historical pipeline/operator
figures retain dated scope. Mathematical figure prose now states assumptions;
29 passing checks are not described as 29 general proofs. The desktop architecture
is capped at 600px and retains its vertical reading order on mobile.

## Independent review corrections

Review of initial head `7ab658bb` requested corrections in eight related groups:

1. Historical figure scopes: removed present-tense softmax and running-VSA-test claims from generator and SVG output; marked retained M2/float/proposal figures explicitly.
2. README mechanism rows: replaced the stale running-D5/source-only-Relation passage with artifact-bound pilot, partial-dose, preflight and successor references.
3. Encoder arrow: `discrete_encode` performs token-table lookup; `input_block` reads encoded input and applies an optional virtual move.
4. Read semantics: Route transports one carrier per block without summing/snapping. Flock/Relation select, equally sum transported roots and snap. The diagram and prose now distinguish them.
5. Hopf boundary: Q30 is storage, while `hopf` and fibre rotation decode to `f64`, compute/normalize and requantize; rotation uses `libm::sin`/`cos`. These offline helpers are outside current M1, not integer execution.
6. Historical Hamming design: sign-bit/popcount/fixed rank weights are distinct from the implemented relation-score/L1/coordinate-Hamming/recency/node ordering and equal-sum Flock aggregation.
7. E8 representation boundary: E1 standard-coordinate signed-action preservation and E2 reduced-metric icosian roots remain separate results. No intertwiner is established. The figure has separate E8 nodes, and the README qualifies its shorthand.
8. Prime-Zeta uniqueness: phase examples and quantization do not guarantee injectivity; separate allocation controls enforce codebook uniqueness under the declared capacity conditions.

These corrections narrow claims; they do not disprove the unestablished cross-representation map or close a mechanism. The [wording regression guard](wording-regression.rs) rejects the specific misleading phrases found in this review; it is not a semantic/scientific audit.

### Reviewable check output and actual rendered previews

[Host check log](checks/discrete-check.log), [actual host-test log](checks/discrete-tests.log),
[local link scan output](checks/links.json), [figure freshness output](checks/figures-check.log),
[claim-wording output](checks/claim-wording.log), [geometry-check output](checks/geometry-checks.log)
and [regression guard output](checks/wording-regression.log) preserve the executed outputs.
These host tests are separate from GitHub's transport-only acknowledgements;
no CI transport status is treated as test evidence.

The bounds records are automated checks, not a substitute for visual inspection.
The following are actual Chromium render captures inspected during this revision:

- Architecture: [390px](previews/architecture-390.png), [1200px](previews/architecture-1200.png).
- Historical Hamming design: [390px](previews/hamming-rank-read-390.png), [1200px](previews/hamming-rank-read-1200.png).
- Separate E8 representations: [390px](previews/connections-390.png), [1200px](previews/connections-1200.png).
- [Hopf helper boundary](previews/hopf-fibration-1200.png), [zeta allocation distinction](previews/zeta-phases-1200.png), [historical VSA status](previews/vsa-hypervectors-1200.png).

```sh
rustc --edition 2021 docs/labs/documentation-reconciliation-2026-10-18/wording-regression.rs -o local/doc-wording
local/doc-wording
```

## Integration with current main

The follow-up merges main `e780b0b6` (full source identity in `validation.json`)
into the previously reviewed docs head `22e8d3c2`. The two conflicts were resolved
without dropping peer state: `docs/harness/claims.json` uses main's complete
registry plus only this documentation branch's claim, and
`docs/integration/current-state.md` retains both sets of entries with the docs
entry first. Upstream's cold-join benchmark, post-dose draft sheets and lab records
are preserved. No CI workflow or permission change was made.

Fresh [integration check outputs](checks/integration-e780b0b6/) include actual
[host tests](checks/integration-e780b0b6/discrete-tests.log) and
[selected Cargo targets](checks/integration-e780b0b6/selected-targets.json).
The 22 regenerated SVGs retain their recorded hashes; all 44 browser bounds
records are empty, and fresh captures match the nine committed preview PNGs
byte-for-byte. The committed previews therefore remain the actual inspected
output for this integrated source. CI transport acknowledgements, workflow runs
and model/host tests remain separate evidence; exact-head GitHub state is reported
on the draft after pushing this merge, not inferred from the local checks.

## Limitations and resources

External links were verified selectively through primary source/issue reads;
the local link audit is not an exhaustive external HTTP check. No CUDA controls,
model inference, evaluation, parity, training, provider action or paid resource
ran in this task. The Rust toolchain/build cache and browser renders live under
ignored `local/`/`target/` in the owned worktree. Historical data archives and peer
work are untouched. No model-quality, serving-speed or whole-training speed claim changes.

**Next:** independent review of the draft, particularly source-arrow semantics and
mathematical notation; refresh volatile status from #2029 before any later delivery.
