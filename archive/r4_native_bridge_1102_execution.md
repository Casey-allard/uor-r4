# Native learned-reference bridge execution — #1102

## September 3 preparation recovery

The earlier [prebuild record](r4_native_bridge_1102_prebuild.json) remains an
accurate observation of the two failed offline dependency queries. The failure
was a local cache miss under Cargo `--offline`, not a client-service outage.
Independent review corrected the overly broad interpretation that exact cache
restoration was prohibited. The frozen #1086 clause applies to the native build;
its original offline release, 900-second and 2-GiB limits remain unchanged.

The selected dependency query restored 25 archives totaling **2,749,985 bytes**
in **0.775064 seconds**. Every new archive matched Cargo.lock's SHA256 and the
lockfile remained byte-identical. The same dependency query with `--offline
--locked` then passed. libm remains 0.2.16 with `default` and `arch`; its archive
matches the accepted checksum. Cache acquisition is separately recorded setup.
No source, dependency, input, operator, tolerance or scientific envelope changed.

The one independently accepted native build completed in **175.932499 seconds**
with a final observed-generation ledger of **682,391,353 bytes**, within the
original limits. Cargo returned zero. It reported one unused-constant warning
for `REQUEST_SCHEMA`; no second build or warning-cleanup rebuild was run.
The opt-in API research binary is 1,126,448 bytes, SHA256
`d423d8d3c3acd2d1c6215c21206e1bec7583e4dd37e84f30f70f79e77c40d53f`.
The successful build establishes compilation for this pinned target/profile,
not reference preservation, general portability or serving qualification.

The binary's metadata-only operation reported aarch64/macOS and FPCR zero:
round-to-nearest, gradual underflow, and the specified FZ/FZ16/AH/FIZ settings.
This is a zero-model runtime observation; it does not substitute for checking
the actual inference thread during the comparison. The build release binds 494
source files, rustc 1.97.1, target aarch64-apple-darwin, generic CPU, both
vectorizers disabled, incremental off and a minimal explicit environment.
The [preparation index](r4_native_bridge_1102_evidence/preparation-index.json)
links the exact release, independent review, completed/wall receipts and runtime
identity. The build output and cache evidence are retained locally.

Metadata preparation then caught a changed historical `campaign.py` in the new
checkout. All 169 original accepted reference files still match their recorded
hashes. The prospective comparison uses that entire original source closure,
plus the new bridge worker, instead of changing any accepted source pin. The
failed metadata helper created no comparison root and read no fixture/model
payload. Its source and the [location reconciliation](r4_native_bridge_1102_evidence/source-location-reconciliation.json)
are retained. No model envelope was consumed by this correction.

The [source-only capacity plan](r4_native_bridge_1102_evidence/comparison-budget-plan.json)
charges 73,435,655 mandatory bytes at the maximum manifest size plus a 48-MiB
metadata/log/failure reserve: 123,767,303 planned bytes, with 10,450,425 bytes of
headroom under the unchanged 128-MiB cap. The reserve is an engineering estimate;
unexpected output remains subject to the external supervisor's stop rule.

At this preparation checkpoint, exports, loader calls, model loads, forwards,
fitting, comparison and replay are zero. The comparison release is prospective
and awaits its independent concrete review. Historical Python preservation,
weak-control and descriptive findings remain unchanged. No mathematical proof
or native model-behavior result follows from these preparation observations.

## Sole admitted comparison result

**`NATIVE_REFERENCE_PRESERVED` — measured on September 3.** The concrete release
SHA256 `2c3c2f73eb6cf804eb69b2afb0f979ae623a512ca0492e47df2af70d6cbaca8b`
and independent preexecution acceptance
`88c8f8b4223ab83cca072b263c6a4b2febe542173c040fbf9f73bbc6143f4647`
were posted to [#1102 before outcomes](https://github.com/UOR-Foundation/uor-r4/issues/1102#issuecomment-5528418573).
The external supervisor admitted exactly one comparison. It completed with no
stop receipt; its envelope is consumed and cannot be rerun.

Two independent exports produced the same **2,172,252-byte** artifact, SHA256
`2c209590a64cae16a4140fd43adc1cb1f87b357c02e3d4959f1e37f4ab8cd5ab`.
Its native-state SHA256 is
`4f453da12a9346356e64b6c16abfbaad1ca99e3966173cd79e9ddbc8c2d9341b`.
All eleven rejected loader fixtures matched their exact frozen error objects;
the one valid loader established the expected state and refused an ordinary
answer without trusted qualification. All twelve loader calls used zero
forwards. Seven rejected partial state validations remained within the separate
22-partial-validation cap.

Both Python and native arms, in both initial execution and fresh replay,
returned **320/320 correct answers**, **4,480/4,480 consumed-role selections** and
**16/16 expected typed refusals**. Input IDs/lengths, spans, hashes, token/clause
frame indices, full-head argmax/token and complete result records matched.
Padding attention remained exact positive zero. Both implementations reproduced
all compared tensor bytes and deterministic records exactly in a fresh process.

All four complete f32 tensors satisfied the per-row/per-tensor absolute-error
limit `1e-5` with relative tolerance zero. Initial and replay maxima were the
same:

| Tensor | Largest absolute difference | Row | Flat index |
|---|---:|---:|---:|
| Role attention | 3.5762786865234375e-7 | 240 | 172 |
| Role vectors | 8.940696716308594e-8 | 12 | 149 |
| Binding attention | 2.98023223876953125e-7 | 132 | 4 |
| Full 4096 logits | 4.76837158203125e-6 | 140 | 49 |

The comparison completed **1,280 logical forwards**, five successful engine
loads / ten successful model-state loads, zero optimizer updates, zero refusal
forwards and zero withheld reads. Export/integrity, execution and replay phase
receipts report 0.817788, 4.209747 and 3.765631 seconds. The final external wall
receipt reports **8.809784 seconds** including completion writing; its own write
and exit tail were checked before return. Combined sampled peak RSS was
**602,390,528 bytes** and the complete retained ledger **75,039,076 bytes**,
including the original 672,846 authoring bytes once. These lie within the fixed
120-second phases, 360-second total, 3-GiB RSS and 128-MiB byte limits. Sampling
limitations in the supervisor record remain explicit; this is no universal
operating-system resource guarantee.

The [measured summary](r4_native_bridge_1102_evidence/measured-summary.json),
[complete comparison result](r4_native_bridge_1102_evidence/comparison-result.json),
[loader receipts](r4_native_bridge_1102_evidence/loader-gates.json),
[initial tensor comparisons](r4_native_bridge_1102_evidence/comparison-initial.json)
and [replay comparisons](r4_native_bridge_1102_evidence/comparison-replay.json)
retain the decision evidence. The [92-file evidence index](r4_native_bridge_1102_evidence/retained-evidence-index.json)
binds both original exports, all mutation copies, row records, full tensor
streams and receipts at the exact local root. **26,910,720 bytes of full compared
tensors are retained**, with no tensor or artifact deletion. Public metadata
mirrors do not replace these local originals.

This comparison is empirical preservation on the existing authoring stratum,
with the accepted reader/core, vocabulary/query forms and four-fact context.
It adds no semantic worlds, unrestricted parser, longer context, generation,
reasoning/coding, geometric advantage, mathematical proof or final integer/table
kernel qualification. #1094 Python preservation, its unavailable preparation,
#1079 weak control and #1082 descriptive limits remain unchanged. #973 stays
open and #954 blocked. The result requires independent outcome acceptance and
protected delivery; those later closure records are appended below.

## Independent acceptance and qualification handoff

The independent [result audit](r4_native_bridge_1102_evidence/result-review.json),
SHA256 `5e75c5a3407d91bf14a6ec8d57981a72cfe110cdaf1ddfd0c71f6ea0a07974e8`, records
**`ACCEPTED_BOUNDED_NATIVE_REFERENCE_QUALIFICATION`**. It independently decoded
and checked all 6,727,680 retained f32 values, recomputed every row/tensor maximum,
verified full-head argmax/spelling, all fifteen diagnostic role maxima and the
fourteen consumed labels, original input hashes/spans, frame folds, all refusals,
both exact replays, exports/components/native-state identities, loader errors
and source/runtime bindings. This was retained-evidence analysis with no model
or loader rerun. The [resource cross-check](r4_native_bridge_1102_evidence/resource-audit.json)
also reconciles the exact 92-file ledger and work receipts; it discloses its
supervisor author and sampled-measurement limits.

The exact [qualification receipt](r4_native_bridge_1102_evidence/qualification.json)
is twelve-field `ascii-json-1086/1`, with no trailing LF. Its SHA256 is
`61d29aa80e6bcd3d163b2ff2a6da4faab04414ea9f4284d80b798c4e46cf5369`. It binds this **result** acceptance, the completed comparison,
artifact/state, binary/runtime, schemas and profile. It was constructed only
after independent outcome acceptance and without another model call. The
[handoff record](r4_native_bridge_1102_evidence/qualification-handoff.json) carries
additional provenance and the host's trust obligations without adding fields
to the strict receipt.

A host must verify those external identities before calling `qualify()`. The
successful ordinary qualification/answer path remains **NOT_RUN**: this
campaign measured the separately admitted comparison path and the missing-
qualification refusal gate. The measured binary has metadata/gate/comparison
modes and no serving endpoint. A newly linked #1084 service must bind and
qualify its own actual binary; it cannot copy this executable's identity or
reuse the consumed comparison as a serving CLI.

The scientific decision and independent result review are complete. Protected
[PR #1104](https://github.com/UOR-Foundation/uor-r4/pull/1104) carries the source,
evidence and current pointers; #1102 closes through that protected delivery.
**One next action after closure:** separately activate #1084 and freeze its
service/API and artifact-ownership ADR for `answer_four_fact_raw_text/v1`,
including truthful host-binary qualification. #1084 remains unassigned until
active. No successor implementation or second task is performed here.

## Original research and storage closure

The [original input/source audit](r4_native_bridge_1102_input_audit.md) and
[#1086 source manifest](r4_native_reference_1086_source_manifest.json) retain
pinned NEMESIS carrying criteria, W33 immutable-store source and UOR byte/value
identity source checks. They inform the explicit carrying, ownership and
identity boundaries. They supply no external proof of native numerical
preservation; the new empirical claim rests on the admitted comparison and its
independent retained-evidence review.

The [storage review](r4_native_bridge_1102_evidence/storage-review.json)
consulted the September 2 audit and observed 43,878,120 KiB available at delivery
review. The new build directory occupies 458,524 KiB and comparison directory
74,108 KiB; the larger historical build ledger also counts observed temporary
file generations. Source, caches, binaries, original models/corpus, historical
worktrees, full comparison evidence and uncertain user material are retained.
No files were deleted. The original mixed checkout remains untouched.

The [standalone independent audit source](r4_native_bridge_1102_evidence/audit_retained_result.py)
is published with its [machine result](r4_native_bridge_1102_evidence/result-review.json)
and [readable review](r4_native_bridge_1102_result_review.md). It is retained
for inspection of the measured evidence, not an automatic QA or model-rerun
entrypoint. Public delivery metadata is recorded separately from the frozen
campaign's complete original byte ledger.
