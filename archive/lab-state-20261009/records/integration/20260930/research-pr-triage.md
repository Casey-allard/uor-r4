# Research PR integration triage — 2026-09-30

Independent non-author review by `/root/independent_systems_review`; same provider/root launcher. Read live PR bodies/comments/files and exact Git blobs. No Cargo, model runs, source edits or GitHub mutations. The E8P counterexample below is a finite source transcription, not compiled Rust execution.

**None of these six heads can merge unchanged.** This does not require broad research reruns. The first useful deliveries are a corrected codec PR #1490 and a narrowly scoped historical negative from #1519. Root reports #1521 and #1505 have now merged; refresh main and exact candidate before delivery. Historical five green jobs are acknowledgements, not executed checks. No formal GitHub reviews were present in the inspected snapshots; two existing Lab 1 comment reviews still require disposition.

| PR | Exact inspected head | Immediate disposition |
|---|---|---|
| #1490 | `027b7847a8d4551e7615e9cb3230b3b4ff0f229c` | Fix saved-codec export and reload test; focused adapter/export tests, independent review, deliver. |
| #1519 | `2c7f9a462a5e4d73426945ad35d596562c4ad381` | Isolate stale stacked changes; preserve scoped implementation-negative with codec/provenance limitations. No full model rerun now. |
| #1506 | `19ec19a30c686ebb69498e297f9202f9115ecebb` | Existing exact-head changes-required review remains unresolved; fix two unsafe paths and narrow parity claims or run its bounded discriminator. |
| #1526 | `99f6c039ebe10df7febf9e375b1222b98968f90f` | Compile-time test errors plus wrong FusedRead semantics; restrict scope or repair before GPU model work. |
| #1527 | `190194f21e8990911e62517b54844a23ccf04dc4` | Genuine descendant of #1490; correct result comparators/gates, publish historical evidence separately from promotion. |
| #1528 | `128df8a7dbab4d41b0a8163205ac6daa697cfb03` | Draft/source-only. Fix private-field test, tie accounting, allocation and numerical-contract defects; focused selector integration. |

## #1490 — smallest path to code delivery

Changed files: `src/d4_codecs.rs`, `src/stack_export.rs`, `src/lib.rs`, `tests/d4_map_codec_adapter.rs` under training, plus two documentation files. Results B/C were removed. Latest head reports formatting/wording checks, not the new Rust tests.

- **P1 — saved-codec export silently becomes RTN.** [`examples/geometric-stack.rs:1809-1811`](https://github.com/UOR-Foundation/uor-r4/blob/027b7847a8d4551e7615e9cb3230b3b4ff0f229c/crates/uor-r4-training/examples/geometric-stack.rs#L1809) reads/checks saved representation then loads a model with `served=None`; it never restores the codec before export at1849. This PR newly allows MinMSE/compensated saved names in [`stack_export.rs:652-663`](https://github.com/UOR-Foundation/uor-r4/blob/027b7847a8d4551e7615e9cb3230b3b4ff0f229c/crates/uor-r4-training/src/stack_export.rs#L652), so actual CLI export can write RTN while claiming preservation. Restore the codec on that caller or refuse these saves until supported. Reject bare shape names consistently.
- **P2 — new reload test compares different serialized provenance.** [`d4_map_codec_adapter.rs:1046`](https://github.com/UOR-Foundation/uor-r4/blob/027b7847a8d4551e7615e9cb3230b3b4ff0f229c/crates/uor-r4-training/tests/d4_map_codec_adapter.rs#L1046) exports source `s2_parity`; line1164 exports `s2_reloaded`; then asserts whole byte vectors equal. Source is serialized by `StackArtifactBuilder`. Use identical provenance or compare intended payloads.
- Shape parity is now internally consistent, but `RecurrenceOutMinMseMapCodec::for_shape(288,288)` selects all matching square maps, including attention; it is not recurrence-site identity. State that accurately.

**Next:** fix these, run the focused adapter/export tests including actual save/export/reload, independent exact-head review and merge. B/C training reruns are not required to deliver this codec-only scope.

## #1519 — retain a negative without misdiagnosing geometry

The restored sealed-root `report.json` equals committed evidence. Report SHA256 `c9674d4342eeec76e619d9ecbc9b13fb01a176f56b75cd86cd6b6dcf1a747854`; manifest SHA256 `0fa0e375bd18c4cab1637ac43ef5561949ee4f6ecfd850c9cfb5b6df0912f238`. I did not execute the Rust seal verifier. Recorded excess4.8912885413 nats exceeds the stated0.05 gate. This is retained measured output, not a fresh independent model run. The report/attempt have no source/executable binding and documented `/tmp/uor-target/release/b3-e8-smollm2` is absent. Evaluator lines805 onward hard-code input hashes rather than compute identities of effective inputs.

- **P1 — encoder is not nearest-codeword.** [`b3_e8_codecs.rs:159-179`](https://github.com/UOR-Foundation/uor-r4/blob/2c7f9a462a5e4d73426945ad35d596562c4ad381/crates/uor-r4-training/src/b3_e8_codecs.rs#L159) restricts sign/parity candidates incorrectly. Exact source-transcribed finite case: `decode(256)=[.75,.75,.75,.75,.75,.75,.75,-1.25]`; encoding it at scale1 returns code128, `[-.75,.25,.25,.25,.25,.25,.25,-.75]`, squared error4, despite code256 having zero error. Cases257/511 also fail. Saved `b3-e8p-counterexamples.json`. Add this tiny Rust regression and an independent finite-codebook oracle before any successor language experiment.
- Result document §4 says lattice quantization is mathematically incapable, the question conclusively resolved and the arm permanently stopped. One known-defective implementation does not support those claims. Serialization round-trip is not nearest-codeword correctness. Preserve the exact run's negative, controls and missing provenance; do not present it as canonical QuIP# or all E8 disproven.
- **Stacking:** PR carries older #1490 codec/export/tests plus B/C results/binaries duplicated by #1527. Common ancestor with corrected #1490 is3617721f; corrected027b7847 is not its ancestor. Their overlapping blobs differ.

**Next:** isolate historical result/docs, unchanged measurements, explicit encoder/provenance limitation and existing source commit references; remove unrelated stale changes. Review and deliver that scoped negative. No full SmolLM2 rerun needed to preserve it. A corrected codec would be a new experiment, not a rewritten old result.

## #1506 — resolve the existing four-point review

Lab 1 review targets the unchanged current head19ec19a3. It already verifies kernel arithmetic, tie rules and ranges. Two concrete unsafe paths remain:

- **P1:** Public `Container::parse_for_reference` followed by [`uor-r4-lut/src/stack.rs:163`](https://github.com/UOR-Foundation/uor-r4/blob/19ec19a30c686ebb69498e297f9202f9115ecebb/crates/uor-r4-lut/src/stack.rs#L163) `StackModel::from_artifact` bypasses D10 snap refusal. Add refusal at construction plus the bypass-sequence test.
- **P1:** `StackModel::load` does not restore snap and the PR removes the prior export guard; `d4-float-reference.rs:281,396` loads then exports without it. Restore saved snap or refuse, covering callers.
- Commit kernel/first-divergence evidence with hashes and sixteen margins. Approximately2461 later mismatches remain unclassified; choose the review's permitted narrow wording/open parity item, or run the predeclared bounded forced-root diagnostic. Do not claim all mismatches are near-ties from first-divergence data.
- Keep contended cost as relative +9.17% from three interleaved pairs, not quiet absolute latency. Re-audit changed serving roots if the numerical path changes.

**Next:** focused safety repairs/evidence, exact-head review and merge; no training rerun. Coordinate with the separate snap-restore work root is reviewing.

## #1526 — compile and semantics before long GPU work

Changed training dispatch/shaders/tests and dependencies. The embedded ten-test transcript predates the current eleven-test file; no exact-head execution receipt supplied.

- **P1 compile:** [`metal_stack_ops_parity.rs:595-607`](https://github.com/UOR-Foundation/uor-r4/blob/99f6c039ebe10df7febf9e375b1222b98968f90f/crates/uor-r4-training/tests/metal_stack_ops_parity.rs#L595) calls `rms_norm(x,w,eps)` but helper at `geometric_stack.rs:4097` takes two args. Lines628-640 pass target Tensor to `cross_entropy`, whose helper at4456 expects `&[u32]`.
- **P1 semantics:** [`metal_stack_kernels.rs:471-523`](https://github.com/UOR-Foundation/uor-r4/blob/99f6c039ebe10df7febf9e375b1222b98968f90f/crates/uor-r4-training/src/metal_stack_kernels.rs#L471) FusedRead ignores `aux`, always dot-product softmax. CPU block/transform uses Lorentz beta/offset/lifts, NoRead/null probability, age and optional RoPE. Metal dispatch at `geometric_stack.rs:3659` supplies no semantics flags. Gate unsupported cases or implement exact behavior. Neither FusedRead nor RecurrenceCore appears in the parity tests.
- Layouts/offsets are ignored by raw-buffer dispatch; add a bounded narrow-view check or correct offset handling. Throughput loops do not synchronize completion before timing ends, so they do not establish GPU speedup.

**Next:** repair test API, narrow the supported operator set or implement the missing semantics, run focused actual-GPU forward/backward tests with a device-present receipt. Do not require a corpus run for source delivery. Coordinate `geometric_stack.rs` with #1506.

## #1527 — historical publication versus promotion

True descendant of027b7847; codec/export/test blobs equal #1490. Update after corrected #1490 lands. Eight S2 manifest SHA256s independently match committed references; this is manifest identity, not full payload/re-execution verification. Reuse earlier valid independent reply-record arithmetic at its actual artifact scope.

In `docs/integration/d4-s2-dialogue-qat-result-2026-09-29.md` §3/4:

- Baseline14/58 and control5/58 are labeled kernel agreement but compare integer versus float. Prior independent reader found baseline kernel agreement47/58.
- Registered29/58 float gate is attached to56/58 kernel agreement; float comparisons5–6/58 miss it. Separate quantities and gate outcomes.
- Parent2.5209156608 +0.03 =2.5509156608, not2.5255. Preserve literal preregistration and parent-value correction separately. Served2.5680465593 misses both.
- QAT integer versus its **own float weights** remains missing; current served/parent/other-float comparisons cannot substitute. Do not infer all difference is drift.

**Next:** correct record, compile edited research bins if delivering them as working tools, publish honest historical scope. Scientific promotion retains unresolved own-float/provenance/S1-seed-sensitivity requirements or an explicit prospective council revision. Those need not block preservation of the original observation.

## #1528 — source-only selector fixes

Draft with no compile/test execution. Also carries an unrelated four-row GEMV optimization; separate it or give it its own scoped validation.

- **P1 compile:** integration test `stack_flock_tests.rs:110-137` accesses private `FlockScratch.slots/rest` (`stack/flock.rs:103-105`). Move assertions to module tests or add narrow inspection API.
- **P2 tie accounting:** compares `rest[k-1]` with `rest[k]` before sorting the first partition; `select_nth_unstable_by` leaves that partition unordered. Compute the actual kth boundary after ordering.
- **P1 serving claim:** [`flock.rs:356,363,381`](https://github.com/UOR-Foundation/uor-r4/blob/128df8a7dbab4d41b0a8163205ac6daa697cfb03/crates/uor-r4-integer/src/stack/flock.rs#L356) still uses `/` despite divider-free declarations. Replace or precompute; verify actual compiled serving roots before qualification.
- **P2 allocation:** `new` reserves at most129 entries (`flock.rs:117`); larger support reallocates. Stable `sort_by` can allocate temporary storage. Tests only inspect slots/rest capacities. Reserve full supported support/use suitable allocation-free total-order sort, or restrict the contract. MAX_FLOCK_CONTEXT is not enforced.

**Next:** OpenCode and Antigravity reconcile the canonical training/integer selector contract, then focused compile/parity/tie/allocation tests and applicable opcode audit. No language run required.

## Concrete lab assignments and integration order

1. **Antigravity:** finish #1490 fixes and protected delivery; isolate/qualify #1519 negative in parallel; correct #1527 historical claims next. No new large lattice/QAT runs to close records.
2. **Claude:** fix #1506 existing review, preserve kernel work, choose scoped parity delivery or admitted short forced-root check, merge then resume integrated retrieval/session work.
3. **OpenCode–DeepSeek:** coordinate canonical selector and #1528 fixes; review Antigravity's #1490/#1519 exact new heads independently. No duplicate selector specification.
4. **Codex:** keep integration queue/live main/claims current, provide complete prompts and exact findings, validate combined changed interfaces and merge each eligible item. Narrow #1526 with its author rather than starting another model branch.

Each lab goal continues through actual merge, evidence/current-state update and the next eligible task, with shared storage/budget/lease/handoff rules. The full reasoning and source observations are retained in `research-pr-triage-detailed.md`; raw PR snapshots/diffs/comments are in neighboring `pr-*-*.json` files. Recheck exact head before posting or delivery.
