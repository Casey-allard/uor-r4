# Legacy crate repair — 19 October 2026 (claude)

**Owner direction (Casey, 19 Oct 2026, charter question Q8).** The packages broken since
[D36](../../integration/DECISIONS.md#d36--github-pages-keep-the-last-good-deploy-live-deploy-job-skipped-until-a-d11-served-wasm-engine-lands)
item 3 are to be **repaired on the current core, not archived**. Their mechanisms
(the R4G1 compiled lookup graph and the September native-geometric engine) stay
**PARKED** under D22: this unit restores buildability and does not change their status,
any model behaviour or any published result. References #2028.

## What was broken and why

#2345 narrowed `uor-r4-core`'s library to `src/native_support.rs`. That surface no
longer compiles `transformerless::{compiler, runtime}` or the pre-#2345
`native_geometric` model API. `uor-r4-graph-compiler` imported both, so it failed with
11 errors (13 with tests). Every package that depends on it failed at the same point:
`graph-certify`, `api`, `proof-model`, `graph-cli`, the root package, and the test
targets of `graph-runtime` and `model-source`.

The retired code cannot simply be compiled again. Four of its paths are banned in every
compiled module (D23/D25/D29/D30):

| Path | Why it stays out |
|---|---|
| `uor-r4-model-source` (the transformer teacher) | its library compiles normalized-exponential attention and dense matrix products, and links `uor-matmul-gemm`; `scripts/native_dependency_boundary.rs` lists both as D29-excluded packages |
| `transformerless::compiler` top-3/top-8 samplers | normalized-exponential sampling of teacher logits |
| `transformerless::runtime` dot assignment (`simd::dot_argmax`) | scores all 256 classes as Σ_d work[d]·W[class][d] with the product done by shift and sign: a dense matrix-vector contraction behind a table wrapper (D30) |
| `src/r4_source_relation_head.rs` | a dense 32×288 affine layer, ReLU and output layer (a dense MLP head) |

D36 item 2 already keeps `transformerless::{compiler, runtime}` retired. Restoring an
R4G1 engine without these paths would be a port (see *Not done*), not a repair.

## Method

The repair applies the #2345 pattern to each package. A new `src/native_support.rs`
becomes the library path; it compiles only the modules that build on the current core
with none of the paths above. **Every historical source file is byte-identical**; this
unit adds six `native_support.rs` files and edits manifests and docs only. Modules are
kept or excluded whole: no function inside a historical module is gated or edited. No
Cargo feature re-enables a historical surface.

Manifest changes:
- the library path points at `src/native_support.rs`;
- `uor-r4-model-source`, `uor-r4-graph-cli` (from `api` and the root) and the
  `uor-matmul-core` dev reference are removed;
- the BLAS features `observation-blas-exception` and `local-inference-accelerate` are
  removed with the links they enabled;
- targets that drive the historical surface are not built (`autotests`/`autobins`/
  `autobenches` off, with the compliant targets listed explicitly);
- the six `[profile.test.package.*]` overrides for the removed packages are dropped;
- the root's `legacy-engine` forward to `uor-r4-router` is removed, so no feature of a
  repaired package re-enables a historical engine;
- the git `uor-addr@165b51e3` (whose `uor-prism` dependency brings in
  `uor-prism-tensor` and `uor-prism-numerics`, both D29-excluded) is dropped from the
  root and made optional in `uor-r4-router`, linked only by its non-default
  `legacy-engine`. Neither crate's compiled surface uses it.

`Cargo.lock` loses `uor-r4-model-source`, the five `uor-matmul*` packages and `bytemuck`.
None of them remains in the workspace graph. The git `uor-addr` and the two prism packages
remained, reached only through `uor-r4-graph-format`, until the 20 October follow-up below
removed that path.

`graph-cli`'s two kept modules name their error type as
`uor_r4_model_source::SourceUnavailable`, which that crate re-exports unchanged from
`uor-r4-source-interface`. The new surface binds the interface crate under that name
(`extern crate uor_r4_source_interface as uor_r4_model_source;`). They get the identical
type without the teacher link.

## Result per package

| Package | Kept (compiled) | Lines kept / total | Not compiled (preserved) |
|---|---|---|---|
| `uor-r4-graph-compiler` | 22 of 33 modules: planning, semantic state/transitions, stage DAG, executor/jobs/memory budget, pack, residual, reproducibility, monograph, rate-distortion, reference IR, behavioral probes, observation shards, and the three feature-gated modules | 6,090 / 34,424 | `lib.rs` (teacher `compile`/`observe`), `observation`, `observation_text`, `induction`, `recorded_corpus`, `route_fit`, `routing`, `segment_fit`, `skipmix_fit`, `quantum_cover`, `trace_profile`, `probability_calibration` |
| `uor-r4-graph-certify` | 12 of 26: certificate, compiler scaling, corpus-signed transport attention, frame consistency, long context, octeract, prime-route worker canary, route attention, and four feature-gated modules | 4,717 / 25,799 | `score`, `score_runtime`, `fmm`, `certify`, `compare`, `msa_*`, `octeract_trace_screen`, `performance_certificate`, `predictive_sufficiency`, `fairness_provenance`, `route_cost`, `route_fit_report`, `target_operator_certificate` |
| `uor-r4-api` | none of its 12 modules; the surface re-exports the current core's `native_geometric` | 0 / 17,920 | every module: each depends on the R4G1 `engine`/`compile` path or the pre-#2345 model API (`native_geometric::{Model, durable_memory, vsa}`); the `r4-native-chat` and `r4-learned-reference-1102` bins; both examples |
| `uor-r4-proof-model` | 6 of 9: allocation, inference audit, Kani proofs, PDF traceability, proof matrix, range bounds | 936 / 1,970 | `deterministic_topk_proof`, `theorem7_proof` (core's retired `score_q`, `transitions`); `structural_guarantees` (its #161 obligation needs certify's `performance_certificate`, which uses the retired runtime's `OpKernel`) |
| `uor-r4-graph-cli` | 2 of 6: `prime_route_canary`, `recommend_scale` | 1,178 / 20,184 | `lib.rs` (the R4G1 command dispatcher), `cover_sweep`, `convert_r4g1`, `runtime_corpus`, `scenarios` |
| root `uor-r4-wasm-router` | facade re-exports of the current core and router, `r4_source_span_pointer` (a weighted cosine, no weight matrix), `telemetry` | 486 / 75,885 | `lib.rs`, the `r4` binary (`main.rs`, `server.rs`, `chat.rs`, `model.rs`, `r4g1.rs`, `tless_uor.rs`, release bundling), the `r4_*` teacher-trace and reference-generation experiments, the relation head, the native-geometric CLI/service, the wasm bindings |
| `uor-r4-graph-runtime` | library unchanged; 3 of 4 tests, the `vp_tree` bench | — | `r4g1_runtime_test`, the `session_signature` bench |

[`checks/compiled-sources.txt`](checks/compiled-sources.txt) lists every source file the
seven packages compile, read from cargo's dep-info after a clean rebuild. Three entries
(`src/chat.rs`, `src/main.rs`, `src/server.rs`) are read as text by a static
provider-scan test (`include_str!`), not compiled. A banned-token scan of the compiled
Rust finds hits only in comments and in `inference_audit.rs`'s forbidden-opcode lists,
which name instructions to reject. The new files name no banned operation.

## Checks (at the PR head, on the owner's laptop; no model compute)

| Check | Result |
|---|---|
| `cargo check --keep-going -p <pkg> --all-targets`, per workspace member (not `--workspace`: core raises a deliberate `compile_error!` under feature unification) | 24 of 28 members pass, including all seven touched here. `uor-r4-core` 947, `uor-r4-integer` 167 and `uor-r4-router` 40 `error[` lines are pre-existing historical targets; none of them depends on a changed package. `uor-r4-workbench`'s 1 error is core's `compile_error!` reached through its features, as before. |
| `cargo check --all-targets --all-features` for `graph-compiler`, `graph-certify` | pass |
| `cargo test -p <pkg> --no-fail-fast` | `graph-compiler` 91/91 (95/95 with all features); `graph-certify` 63 pass, 3 ignored (77 pass with all features); `proof-model` 10/10; `graph-cli` 17/17; `graph-runtime` 30 pass, 1 ignored; root 6/6; `api` builds no tests (see below) |
| `uor-harness ratchet d23` | 2,253 across 154 files, equal to baseline |
| `uor-harness ratchet d25` | 93 across 26 files, equal to baseline |
| `cargo tree -e normal,build -p <pkg> \| grep -cE 'uor-prism-(tensor\|numerics)'` | 0 for `router` and `graph-runtime`; 3 for `graph-format` and every package that links it with `alloc` (root, `graph-compiler`, `graph-certify`, `api`, `proof-model`, `graph-cli`); see *Not done* |
| `uor-harness check` | 0 errors |
| `cargo run -q --locked -p uor-r4-claim-wording -- .` | pass |
| `cargo fmt --check` | pass |

The #846 binding test `compositional_planning_certification_846` passes 3/3 after #2794, which
restored the six source files the docs cutover (#2789) had rewritten.

`uor-r4-api`'s `canonical_quality_baseline_934` test builds on the native surface but is
not registered. Three evidence links in the archived
`archive/canonical_quality_baseline_934_result.json` (`docs/attested_broad_baseline_833.md`,
`docs/skipmix_endtoend_causal_908.md` and its `_result.json`) were moved to `archive/` by
the cutover, so its link check fails (1 pass, 1 fail, 1 ignored when run). The archived
result must not be edited. The 20 October follow-up below registers it with a move-map
resolver.

## Not done, for the owner

- **R4G1 serving and compilation stay unavailable.** The `r4` CLI, the R4G1 `R4Engine`,
  the compile/observe/score pipeline and the wasm bindings all need a banned path. A
  compliant R4G1 engine would need a port: split `compiler.rs`'s artifact types from its
  teacher, move the `TeacherOracle`/`RepresentationSource` traits out of
  `model-source`, and replace or gate the dense dot-argmax with the Hamming
  (xor + popcount) assignment the runtime also has. That changes the qualified core
  boundary and model behaviour, so it needs its own owner decision and pre-registration.
- **The release pipeline is paused, not restored.** Neither release asset can be built
  under the bans, so `.github/workflows/release.yml` skips all three jobs (`if: ${{ false }}`,
  reason and re-enable condition in-file), as D36 did for `deploy.yml`. Rewriting it for
  the UORLUT01 artifact is the separate M8 task.
- **Closed 20 Oct 2026; see the follow-up below.** *`uor-r4-graph-format` still links the
  D29-excluded prism packages.* It depended on
  the git `uor-addr@165b51e3` (`default-features = false, features = ["alloc"]`), whose
  unconditional `uor-prism` dependency brings in `uor-prism-tensor` (a dense i8 matrix
  product at `tensor.rs:133` and a quantized sigmoid at `activation.rs:115`, crate
  0.4.0) and `uor-prism-numerics`. `scripts/native_dependency_boundary.rs` excludes both. `graph-format` reaches every repaired package except `graph-runtime`
  (which uses it without `alloc`), so `cargo tree -e normal,build` still counts 3 prism
  lines for the root, `graph-compiler`, `graph-certify`, `api`, `proof-model` and
  `graph-cli`. Moving it to the vendored `third_party/uor-addr-0.2.0`, as core does, has an
  API gap. `src/r4g1.rs` computes every R4G1 section and artifact κ with
  `uor_addr::cbor::address_blake3`, and the vendored facade has only JSON
  (`json::address_blake3`); upstream's CBOR canonicalizer (`src/cbor/value.rs`, about 600
  lines) is built on prism types (`ShapeViolation`, `TermValue`). Closing the gap means
  porting that canonicalizer into the vendored facade and showing byte-identical κ
  labels for every existing R4G1 artifact. That is a change to the frozen R4G1 identity
  path, so it is left for an owner decision. Pre-existing; the active M1 crates do not
  depend on `graph-format`.
- **Proof obligation #161** (`structural_guarantees`) stays out because `OpKernel`, a
  plain operation-count struct, lives in the retired `runtime.rs`. Moving that struct
  would touch the qualified core surface.

## Follow-ups found on the way

- The D23/D25 ratchets scan `crates/` and lab instruments only, so the root `src/`
  (including `server.rs`) has never been ratchet-tracked.
- The manual QA jobs in `.github/workflows/ci.yml` still name the removed `r4` binary and
  the `bdd` test, and `--workspace --all-features` reaches core's `compile_error!`. They
  were already failing before this unit.
- `uor-r4-core`, `uor-r4-integer`, `uor-r4-router` and `uor-r4-workbench` still fail
  `--all-targets` on historical bins and tests.

## Follow-up, 20 October 2026: prism-free R4G1 κ and the #934 link fix

**Owner decision (Casey, 20 Oct 2026): merge #2793, then port.** This follow-up closes the
prism exception above.

**The port.** `uor-r4-graph-format` no longer depends on the git `uor-addr`.
- **New module:** [`src/cbor_address.rs`](../../../crates/uor-r4-graph-format/src/cbor_address.rs)
  (no_std + `alloc`, 221 lines) ports upstream's CBOR re-encoder for exactly the four
  major types an R4G1 skeleton uses: unsigned integers, byte strings, text strings and arrays.
  It follows upstream's rules for them: minimal re-emission of non-minimal heads, definite
  re-emission of indefinite strings and arrays, UTF-8, reserved values, trailing bytes, the
  1,024-level depth bound, and additional information 31 on an integer read as 0.
- **Label:** `blake3:` plus the BLAKE3 hex of the canonical bytes.
- **Other major types:** refused. Upstream accepts them, but no skeleton contains one.
- **Call site:** `r4g1.rs` calls the module instead of `uor_addr::cbor::address_blake3`.
- **Upstream crate:** kept only as a dev-dependency, for the parity tests.
- **Type identity (#618):** unaffected. The function returns a `String`, and no
  `uor-foundation` type crosses the boundary.

**Byte identity, checked against upstream (rev `165b51e3`):**
- **`tests/cbor_address_parity.rs`, 9 tests:**
  - every unsigned-integer boundary in every head width;
  - byte and text strings at every length boundary up to 70,000 bytes;
  - indefinite and chunked forms, including a character split across chunks;
  - nested and non-minimal arrays;
  - malformed input and every prefix of a valid item;
  - the depth bound at 1,023, 1,024 and 1,025 levels;
  - 20,000 generated subset items, each also with a 1-bit mutation;
  - 48 built R4G1 artifacts, compared at the artifact and section level.

  For each input, both implementations give the same label or both refuse it.
- **`r4g1::tests::skeleton_addresses_match_upstream_at_every_head_width`:** the private
  section and artifact skeleton builders, at every id, flags, payload-length and section-count
  head width.
- **No committed R4G1 artifacts:** the repository has none (no `R4G1` magic in any tracked
  file), so the artifacts are built in-test with `ArtifactBuilder`.
- **The tests catch real faults:** removing the port's UTF-8 check fails 3 parity tests, and
  moving its depth bound to 1,023 fails 1.

**#934.** `canonical_quality_baseline_934` is registered. Its link check now follows a
missing `docs/…` path through the cutover move map
(`docs/labs/repo-renormalization-2026-10-19/cutover/move-map.tsv`) to its byte-identical
`archive/…` copy. The archived result is not edited. Without the resolver the test fails on
`docs/attested_broad_baseline_833.md`; with it, 2 pass and 1 is ignored (that one needs the
local #833 bundle).

**Checks at the follow-up head (owner's laptop; no model compute):**

| Check | Result |
|---|---|
| `cargo tree -e normal,build -p <pkg> \| grep -cE 'uor-prism-(tensor\|numerics)'` for root, `graph-format`, `graph-compiler`, `graph-certify`, `api`, `proof-model`, `graph-cli`, `graph-runtime`, `router` | 0 for every package; no normal or build graph links the git `uor-addr` |
| `cargo check --keep-going -p <pkg> --all-targets` for the eight packages above (router excepted) | pass, with 0 errors and 0 own warnings; `router` keeps its 40 historical-target errors |
| `cargo check -p uor-r4-graph-format --no-default-features` / `--features alloc` | pass |
| `cargo test -p <pkg> --no-fail-fast` | `graph-format` 197/197; `graph-compiler` 91/91; `graph-certify` 63 pass, 3 ignored; `proof-model` 10/10; `graph-cli` 17/17; `graph-runtime` 30 pass, 1 ignored; root 6/6; `api` 2 pass, 1 ignored |
| `uor-harness ratchet d23` / `d25` | 2,253 across 154 files and 93 across 26 files, equal to baseline |
| `cargo run -q --locked -p uor-r4-claim-wording -- .` | pass |
| `cargo fmt --check` | pass |
| `uor-harness check` | 0 errors |
