# Independent integration-first research PR triage

Reviewed live GitHub metadata, PR discussions, exact local Git objects and changed-file patches for PRs #1490, #1506, #1519, #1526, #1527 and #1528. Reviewer: `/root/independent_systems_review`; non-author separate pass, same provider/root launcher. No source edits, Cargo, model runs or GitHub mutations. A small source-transcribed finite codec calculation was executed; it is explicitly not a Rust test or model evaluation.

**Decision: none of these six exact heads is ready to merge unchanged.** This is a set of concrete, bounded blockers, not a requirement for a broad rerun campaign. The shortest valuable path is to repair and focus #1490, preserve #1519 as a narrowly scoped historical implementation-negative record, then integrate the snap serving repair. Keep experimental QAT qualification and full Metal capability claims separate from source delivery.

The last direct `branches/main` observation in this pass was `7895501d4e08ed50d8f716a494427a03d9020ca2`. PR view responses retained older base OIDs; do not assume their CLEAN status proves compatibility with this current main or with one another. Refresh the base and exact candidate before delivery. Every inspected PR had no formal GitHub review and only the five historical successful acknowledgement jobs; skipped jobs and green acknowledgements are not executed tests.

## Exact heads and actionable disposition

| PR | Exact head | Scope actually present | Decision and next action |
|---|---|---|---|
| [1490](https://github.com/UOR-Foundation/uor-r4/pull/1490) | `027b7847a8d4551e7615e9cb3230b3b4ff0f229c` | D4 codec adapters, export routing, tests and two documentation edits; B/C results removed | **Fix then focused validation.** Saved-codec export can silently revert to RTN; new real-shape reload test compares different source metadata. Fix these, compile/run the focused adapter/export tests, obtain current non-author review and deliver. No B/C training rerun is required to deliver this codec scope. |
| [1519](https://github.com/UOR-Foundation/uor-r4/pull/1519) | `2c7f9a462a5e4d73426945ad35d596562c4ad381` | B3 codec/harness/results plus old #1490 code and B/C results/binaries duplicated by #1527 | **Isolate and qualify the negative.** Encoder is not nearest-codeword on a finite counterexample. Remove universal mathematical claims; record execution-provenance limits; remove unrelated stale stacked changes. Preserve the negative without a new language run. A tiny Rust codec discriminator comes before any successor model experiment. |
| [1506](https://github.com/UOR-Foundation/uor-r4/pull/1506) | `19ec19a30c686ebb69498e297f9202f9115ecebb` | Integer snap format/kernels/session, offline reference parser, parity harness, exporter and callers | **Changes required at this unchanged reviewed head.** Close the D10 reference-parser bypass and saved-snap export regression; commit precise evidence and scope remaining mismatches. Either run the bounded forced-root diagnostic after admission or explicitly leave full parity open. |
| [1526](https://github.com/UOR-Foundation/uor-r4/pull/1526) | `99f6c039ebe10df7febf9e375b1222b98968f90f` | Metal shaders and geometric-stack dispatch, parity tests, claims | **Not ready.** Current test target has compile-time API mismatches. FusedRead changes model semantics by dropping Lorentz/null/age/RoPE behavior. Restrict unsupported dispatch or implement/test it. Existing ten-test transcript is not exact-head evidence for the eleven-test file. |
| [1527](https://github.com/UOR-Foundation/uor-r4/pull/1527) | `190194f21e8990911e62517b54844a23ccf04dc4` | #1490 as ancestor plus B/C documents/JSON, research binaries and replication scripts | **Reconcile after #1490; evidence wording still needs fixes.** Kernel versus float columns and registered gates remain conflated. Scope as historical reported evidence if independent fresh evaluation is not being promoted. Do not silently claim the earlier requested scientific reruns complete. |
| [1528](https://github.com/UOR-Foundation/uor-r4/pull/1528) | `128df8a7dbab4d41b0a8163205ac6daa697cfb03` | Draft integer flock selector/workspace plus an unrelated blocked-four-row GEMV optimization | **Source-only, not ready.** Private-field accesses prevent integration-test compilation; tie-boundary accounting and allocation/D11 claims need repairs and focused tests. Align with the shared training selector before serving integration. |

## #1490: two concrete remaining defects

The author responded to the earlier Lab 1 review by splitting out results and replacing heuristics with explicit shape-qualified names. The current `round_trip` and export routines now select the same matching shapes. This means `RecurrenceOutMinMseMapCodec::for_shape(288,288)` applies to all matching 288x288 maps, including attention maps, not only `rec_out`; document that semantics accurately rather than interpreting the name as site identity. The E8 direct-export guard is present.

1. **[P1] Real saved-model export can silently change representation.** In `examples/geometric-stack.rs:1809-1811`, `export_mode` reads the saved representation, calls the broadened `check_export_representation`, then loads the model. `StackModel::load` leaves `served: None`. No `codec_by_name`/`set_served_representation` follows before `export_stack` at line1849. Since this PR newly accepts saved MinMSE and compensated codec names in `stack_export.rs:652-663`, those saves can pass the gate and export plain RTN while their source metadata claims the trained representation was preserved. Small fix: restore the parsed codec on the actual loaded export path, or refuse newly unsupported names until that path exists. Validate save -> real export path -> dequantized matrix equality. Bare shape-dependent names must not be accepted as if enough configuration were known.
2. **[P2] The new real-shape round-trip test has incompatible whole-artifact inputs.** `tests/d4_map_codec_adapter.rs:1046` exports with `{"test":"s2_parity"}`; line1164 exports reloaded weights with `{"test":"s2_reloaded"}`; lines1166 onward assert the complete byte vectors equal. `export_stack` passes this metadata to `StackArtifactBuilder::new`, so it is part of the serialized artifact. Use identical source metadata when claiming identical containers, or compare the intended parameter/header fields with provenance excluded. The latest comment/body lists rustfmt/wording only; the historical nine-test runs predate this new real-shape test.

Minimum delivery gate: the above fixes, focused offline adapter/export tests at the resulting head, source caller review, then protected queue. Retained B/C experiments did not use these codecs, so their qualification should not hold this source-only PR hostage.

## #1519: preserve the result, correct the explanation

The current PR is forked from common ancestor `3617721f52719e74cd37199a1a46d22bdb800b4b` with #1490/#1527 and contains older, different blobs in their codec/export/test/result files. It is not a descendant of the corrected #1490 head. Its 20-file diff therefore cannot be treated as an independent B3 change.

The restored report root exists at `/Volumes/UOR-Workspace/uor-r4-lab/b3-e8-smollm2-mlp/attempt-full-32layers-rht-e8p`. Its `report.json` equals the committed evidence JSON structurally. Report SHA-256 is `c9674d4342eeec76e619d9ecbc9b13fb01a176f56b75cd86cd6b6dcf1a747854`; manifest SHA-256 is `0fa0e375bd18c4cab1637ac43ef5561949ee4f6ecfd850c9cfb5b6df0912f238`. The manifest lists attempt/report/summary. This pass did not execute the Rust report verifier or independently verify all BLAKE3 manifest payload entries.

The report retains the seven-arm numbers, including RTN4 mean NLL2.1936822161 and RHT-E8P3 mean NLL7.0849707574; their excess4.8912885413 exceeds the recorded0.05 threshold. These are retained report measurements, not a fresh independent model execution. The report/attempt files do not bind source or executable SHA; the documentation's `/tmp/uor-target/release/b3-e8-smollm2` binary is now absent. The evaluator source also hard-codes corpus/tokenizer/token hashes rather than deriving them from effective argument inputs. Preserve these provenance limitations; a current source fix cannot retroactively bind the old run.

**[P1] The encoder is not nearest-codeword even for an existing codeword.** In `b3_e8_codecs.rs:159-179`, each parity branch builds one sign pattern then tests only the256 magnitude patterns. The sign/parity mapping excludes correct candidates. Transcribing the exact pinned decode/encode functions into a finite arithmetic check gives:

- `y = decode_e8p_codeword(256) = [.75,.75,.75,.75,.75,.75,.75,-1.25]`, scale1.
- The encoder returns code128, decoded as `[-.75,.25,.25,.25,.25,.25,.25,-.75]`, squared error4.
- Code256 is an available zero-error choice. Codes257 and511 also produce nonzero-error self-codeword cases.

The calculation is saved as `b3-e8p-counterexamples.json`; it is a small source-transcribed check, not compiled Rust evidence. Translate this exact case into a Rust regression, then validate against an independent exhaustive finite-codebook reference before promoting any corrected codec. No model run is needed to establish the current claim boundary.

The document says serialization round-trip rules out serialization bugs completely and that uncompensated lattice coding is mathematically incapable of preserving deep language behavior, conclusively resolves the question, and permanently stops the broader arm. Those claims exceed the evidence. A retained implementation can miss its gate while its codec has an identified algorithmic defect. Record the negative for the executed implementation, scales, controls and provenance; do not present it as a qualified canonical QuIP# nearest-codeword negative or a theorem about all post-training E8. Its 3-bit scalar and lattice serialized budgets also differ (3.2766 versus3.0134 bpw), so describe approximate rather than exact bit-budget matching.

Fastest deliverable: a narrowly scoped evidence/documentation slice of #1519 with the original result unchanged, its implementation limitation and absent binary/source binding explicit, unrelated #1490/#1527 changes removed. Retain the experimental source at its existing commit and reference it. A later corrected algorithm is a successor experiment with a new work card; do not start a large rerun merely to close this historical record.

## #1506: existing source-reviewed blockers are still live

The only non-author review targets exactly the current head19ec19a3 and requests changes. I inspected the relevant source and confirmed the two unsafe paths remain:

- `Container::parse_for_reference` is public, while `uor-r4-lut/src/stack.rs:163` `StackModel::from_artifact` does not reject transport-snap metadata. Combining them can run the D10 path as unsnapped. Reject at the model construction boundary and test the bypass sequence.
- `StackModel::load` does not restore transport state, and the PR removes the existing export transport guard. `d4-float-reference` loads then exports with absent explicit snap. Preserve the saved snap into the artifact or refuse the export; check both callers.

The previously reviewed arithmetic/kernel work is useful and should be preserved. The reported near-tie diagnosis only covers first divergence per window; the approximately2461 later mismatches remain unclassified. The prior review explicitly allows either a bounded teacher-forced/root-force diagnostic, or narrower wording leaving the broader parity item open. Commit the artifact-bound kernel/first-divergence blocks with sixteen margins and hashes. Report the contended timing as an interleaved relative percentage, not a quiet absolute latency measurement. Focus tests on parser refusal and saved-snap export, and rerun the serving instruction audit only if the served path changes. No training rerun is required.

## #1526: exact-head compile and semantic defects

The document embeds a ten-test transcript with maximum observed error below5e-7. The current test file contains eleven tests, including a subsequently added throughput test. The transcript is useful historical reported evidence, but cannot establish this head compiled.

- **[P1] API mismatches in the new test.** `metal_stack_ops_parity.rs:595-607` calls `rms_norm(x,w,1e-5)`; `geometric_stack.rs:4097` defines only `(x,weight)`. Lines628-640 call `cross_entropy(logits,&Tensor)`; line4456 defines targets as `&[u32]`. Fix the test arguments and compile this specific target with Metal enabled.
- **[P1] Metal FusedRead silently computes a different operator.** `metal_stack_kernels.rs:471-523` ignores `aux` and always does scaled dot-product softmax. The CPU `FusedRead::block/transform` at `geometric_stack.rs:3400-3535` uses optional RoPE, NoRead/null denominator mass, age bias, and Lorentz lifting/beta/offset. The dispatch at3659 onward passes no flags implementing those semantics. This affects the actual geometric stack, not just numerical rounding. Gate unsupported configurations with a truthful error/fallback until exact support exists, or implement their semantics and add small CPU/Metal forward/backward parity fixtures. The existing tests cover neither FusedRead nor RecurrenceCore.
- The new Metal paths take raw storage buffers while ignoring layout offsets/strides; review actual callers and nonzero-offset/narrow views. Calling `.contiguous()` is not by itself evidence of zero storage offset. This is an identified interface risk requiring a bounded view fixture or explicit layout handling, not a broad corpus run.
- Performance timing loops do not visibly synchronize Metal completion before ending intervals, so no throughput advantage is established by those loops. Keep performance claims separate until a bounded synchronized measurement is run under the quiet slot.

Minimum useful scope can be smaller: deliver the tested primitive kernels with unsupported fused dispatch disabled, honest test/source bindings and a small gradient test. Do not use the current seven-operator completion claim to admit a long geometric-model GPU run.

## #1527: result reconciliation is mostly reading and editing

This PR is a genuine descendant of #1490 head027b7847; its d4 codecs/export/tests blobs are identical. Merge a corrected #1490 first, then update #1527 to avoid redeclaring that source. Research bins and replication scripts now live here as requested, but current evidence docs still contain prior review objections.

- The S2 row called `Kernel Agreement (Integer vs Served Forward)` still shows baseline14/58 and float-control5/58, which are integer-vs-float comparisons. The earlier evidence reader recorded baseline kernel agreement47/58. Relabel/split comparators explicitly.
- The registered29/58 agreement gate remains attached to kernel agreement56/58, although it was defined against float, where recorded comparisons are5–6/58. Mark the scoped gate missed; reported/pending language does not cure comparing the wrong quantities.
- The parent NLL is2.5209156608. Parent+0.03 is2.5509156608, not2.5255. Preserve the literal preregistered bound if necessary and separately document the original parent-value error; do not treat both values as the same gate. Served2.5680465593 misses both limits. Do not infer all change is caused by drift when own-float greedy agreement is still missing.
- QAT integer versus the QAT model's own unsnapped, unserve-quantized float weights remains a separately identified missing comparator. Existing numbers compare its served representation, prior parent or different float continuation; they cannot substitute for this one.

All eight S2 named manifest files exist and their SHA-256 values match the committed JSON. This verifies manifest identity, not the entire model payload or fresh execution. The earlier independent comment already re-derived56/58 and5/6/7 from reply records; reuse that valid evidence at its actual artifact scope. No current exact-head executable receipts for the newly edited research bins were supplied in the PR. Compile them only if delivered as functioning tools.

For immediate integration, choose whether this PR is historical evidence publication or scientific promotion. Honest historical publication can retain missing measurements and original scoped negatives without rerunning all training. Promotion retains the existing unresolved provenance/own-float/seed sensitivity requirements, unless prospectively revised by the council with rationale; never silently mark them done. The earlier S1 seed2 request addressed a0.001988-nat margin, so it remains relevant to robustness claims, not a prerequisite to preserving the original observation.

## #1528: focused fixes before integer serving integration

The PR is still a draft and explicitly says source-only, not compiled or executed. Its five successful queue acknowledgements do not alter that status.

- **[P1] Integration-test compilation:** `tests/stack_flock_tests.rs:110-137` accesses `scratch.slots` and `scratch.rest`, which are private fields at `stack/flock.rs:103-105`. Use a narrow capacity-inspection API or move private-state assertions to module tests; do not expose internals broadly only for tests.
- **[P2] Cutoff tie accounting:** both selectors compare `rest[k-1]` with `rest[k]` immediately after `select_nth_unstable_by`. The first partition is unordered; `rest[k-1]` need not be its worst element. Compute the actual boundary after sorting the selected slice, or select the needed boundary explicitly. Keep deterministic lowest-position ranking.
- **[P1 scoped serving contract]** rank helpers at lines356/363/381 use ordinary division despite declaring multiplier/divider-free operation. The output normalization alone uses `stack_div_u128`. Replace the remaining divisions or build immutable tables offline, then inspect the actual served instruction roots; no compiled instruction audit exists yet for this path.
- **[P2 allocation contract]** `FlockScratch::new` reserves only `min(129,capacity)` entries. Larger valid window/top-k support can allocate despite a full-capacity scratch. Stable `sort_by` may allocate its own scratch for larger slices. Existing capacity tests inspect only slots/rest, not entries or allocator activity. Reserve the maximum supported support and use an appropriate allocation-free unstable sort with total tie order, or narrow the supported contract. `MAX_FLOCK_CONTEXT` is currently not enforced.
- The PR additionally adds `stack_gemv_pairs_blocked4`, with a75% cache-fetch reduction claim but no measured cache traffic. Separate this unrelated optimization or label the theoretical reuse precisely; it increases overlap with #1506 and the opcode audit boundary.

Coordinate the integer API with OpenCode's canonical floating/training selector, including support order, cutoff ties and rank-weight quantization. This is an offline/serving numerical bridge, not permission to fork two independent selection specifications.

## Merge graph and recommended next lab goals

Actual overlapping blobs:

- #1490 -> #1527 (true ancestor, common codec blobs identical).
- #1519 forks before #1490 fixes and carries different codec/export/result/bin blobs. Isolate it first; merging it wholesale risks stale behavior and claims.
- #1506 and #1526 overlap `geometric_stack.rs` and `lib.rs` with different semantics. Integrate the snap contract before qualifying Metal against it.
- #1506 and #1528 overlap integer kernels/mod/tests. Coordinate exact numerical entrypoints and validate the actual merge candidate.

Assign these extended goals, with no new research branches until the assigned integration outcome is recorded:

1. **Antigravity: deliver codec and negative evidence.** Fix #1490's two concrete issues, execute only its focused checks in the shared slot, obtain an exact-head review and merge through the coordinator. In parallel prepare a scoped #1519 evidence-only cleanup preserving the original run and the encoder counterexample/provenance boundary. Do not run another full SmolLM2 experiment. Then correct #1527 comparators and merge it at honest historical scope. Continue to Metal only after these delivery outcomes are recorded.
2. **Claude: resolve #1506's existing review.** Repair D10 construction refusal and saved-snap export at callers; commit retained parity evidence and choose the already offered narrow-claim path or bounded forced-root diagnostic. Preserve the kernel arithmetic review, request only relevant new checks, and deliver the snap contract. Then resume the integrated model/retrieval goal using merged code.
3. **OpenCode–DeepSeek: own selector integration and cross-lab review.** Reconcile #1528 with the canonical training selector; address privacy/partition/allocations/division and execute focused selector tests plus the required actual serving audit. Provide independent review of #1490/#1519 after their authors' fixes. Preserve the original source-only status until executed evidence exists.
4. **Codex: integration steward, not another model fork.** Keep the ready queue current, bind exact heads/reviews and announce concrete fixes to the labs. Review/merge the first eligible source or scoped negative, then update claims and dependencies. Help narrow #1526 to supported operators and verify the combined snap/Metal path before longer GPU work. Record operational delivery honestly and move directly to the next eligible integration task.

Each goal should include the shared lab contract, current exact issue/PR links, owned paths, explicit acceptance, artifact/source identities, full cumulative cost and recovery packet requirements. Success means the reviewed change actually lands and its next task is recorded, not another branch or a green acknowledgement screenshot.

Raw live metadata, changed-file patches and inline comments are saved beside this report as `pr-<number>-live.json`, `pr-<number>-files.json` and `pr-<number>-inline-comments.json`. Those snapshots and these decisions are time-bound; current-head verification is required immediately before delivery.
