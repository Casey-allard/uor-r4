# Native rung-0 masked TRAIN count-reference repair

References #2029, #2028 and #2030. Frozen source
`2fd6358bbc52d2a202a872ad0954ebb12894606f` implements the owner's requested
response-masked TRAIN frequency reference, separate diagnostic DEV response
oracle, exclusion control and reference/input binding. **Qualification:11 finite controls PASS at this frozen source (9 metadata,2 CUDA).**
Protected [PR #2328](https://github.com/UOR-Foundation/uor-r4/pull/2328) merged at `9e904a83448c451f738d5c1d45224dfdf73eb84e`. No training or optimizer update is authorized
or executed by this unit. Global training and the separate M2 training/rental
holds remain **HELD**.

Line: unified geometric route/action/learning integration, count **11/3 →12/3
in protected source PR2328**; documentation closeout belongs to this same
integrated unit with no additional increment, no reset. M1 memory20/40, reply37/232; M2
saved437/512, opened0/128 and genuinely fresh transfer NOT_RUN remain unchanged.

## Current decision and mechanism

Claude's count-reference contest and Kimi's
[PM ruling6145669094](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6145669094)
identified an evaluation-scope error: whole-development-stream frequencies
were serving as the decisive rank reference for response-only evaluation.
The [Amendment5 registration](../../harness/prep/m1-rank-margin-rung0-2026-10-13.md)
now requires the primary reference to use **response-masked targets from the
TRAIN split**, with a within-document predecessor. Development response counts
are a separate diagnostic oracle, never the decisive reference. The +5pp rank
bars, token-only anti-vacuity floors, paired seeds1/2 and other acceptance bars
remain unchanged. **Independent scientific CONFIRM FIT is recorded** by the
[Claude non-author review6146716704](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6146716704)
at `origin/main` `9e904a834`/#2328 plus Amendment5/#2322. The reviewer confirms
masked TRAIN-only counts, no DEV leakage, smallest-ID ties, the actual GPU
exclusion receipt, diagnostic DEV oracle and artifact/input binding. An
independent data-only check confirms2,425,925 eligible TRAIN targets.
The **physical sheet verdict remains PENDING** until Kimi adds the mechanical
TRAIN-mask path/checksum, expected-count/CLI/new-source pins and restores FIT
under [PM board6146134405](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6146134405).
This confirmation does not rewrite Kimi's sheet or release an owner hold. The
former Amendment4 FIT remains historical at its original reference scope.

The previous [complete-path unit](../rung0-complete-path-2026-10-13/README.md)
was protected-merged in #2318 at `5b22739a31eafb131d25e098bb7f02ab803d4fe7`,
with18 finite controls at source `05c84bece50789163bd0f049a2ae76b8a314ef45`.
Its core/head qualification remains scoped evidence; the previous evaluator's
whole-DEV reference is superseded by this repair, not a valid new comparator.
Historical source/results and the original model/data are retained. A tool or
reference-scope defect is a blocker to repair, not a mechanism negative.

## Source and interface contract

`MaskedTargets::new` validates token/mask metadata, resolves verified BOS
membership and selects only eligible response-masked target identities. It
performs no host frequency, ranking or scoring computation. Both count tables,
exact smallest-token-ID ties, raw-count hinge and metric reductions execute on
CUDA. The evaluator receives one explicit prepared TRAIN inventory and creates
a separate DEV response oracle. Report schema2 labels `count_predictor` as
`source_split=train`, and `dev_response_oracle` as `source_split=dev` with
`diagnostic_only=true`; raw count units are never compared numerically with the
model's mean-L1 margin units.

The native CLI requires **exactly one** training stream with its response mask
and a positive expected response-target count. Missing masks, invalid binary
membership/lengths, out-of-vocabulary IDs, empty eligible inventories and count
mismatches refuse before model allocation. Multiple native training streams
are refused rather than silently using the first stream for the reference.
The prepared TRAIN inventory reaches zero-step, periodic and final evaluation.
The mask selects reference membership; the registered model training objective
continues to score all sampled training positions.

TRAIN token/mask/tokenizer content identities, fixed TRAIN reference policy,
actual payload/eligible-target counts, expected count and verified BOS bind
settings, resume lineage, reports and the `native_count_reference.json` sidecar
in final and checkpoint model directories. Old native checkpoints differ in
lineage and cannot resume with the new reference. Native standalone evaluation
requires separately pinned `train_reference` and `train_mask`, rejects a
same-content DEV-as-TRAIN alias, and compares the artifact's saved binding
before loading the model. Missing/stale bindings refuse; the CLI cannot merely
label arbitrary DEV data as TRAIN. Historical inactive evaluation source is
preserved; native evaluation remains CUDA-only and has no likelihood scorer.

Preparation-only train additions, to be pinned by Kimi in the owned sheet:

```text
train=/workspace/rung0/data/mw-balp/train/tokens.u16
train_mask=/workspace/rung0/data/mw-balp/train/response_mask.u8
expected_train_targets=2425925
```

Standalone native evaluation additionally requires
`train_reference=/workspace/rung0/data/mw-balp/train/tokens.u16` and the same
TRAIN mask/count, separately from `tokens=DEV`, `valid_mask=DEV_MASK`, pinned
tokenizer, `expected_dev_targets=14287 max_blocks=111 device=cuda`.
The complete deferred CLI form is in the
[coordination card6146099258](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6146099258).
Kimi owns the sheet update; Codex does not edit it. The original training mask
member SHA256 must be verified/pinned with the preserved parent archive; no
new corpus or mask is generated. Expected train inventory is7,512,797 tokens
and2,425,925 masked response targets; decoded DEV remains42,856 tokens,
14,287 targets across400 documents,111 global384-token blocks plus232 tail.
Actual inventories and content identities are validated and reported, not
silently adjusted to satisfy these expectations.

## Qualification and outcome

**Frozen-source qualification: PASS,11 controls (9 metadata,2 CUDA).** The
corrected CLI CUDA test compilation passes in2.05s. Four helper metadata controls
and five CLI argument/metadata controls pass. The two CUDA controls pass:

| CUDA control | Observed result | Wall time |
| --- | --- | --- |
| `cuda_masked_train_reference_excludes_unmasked_ids_and_separates_dev_oracle` | Frequent unmasked identities contribute zero counts; changing them leaves primary counts/ranks unchanged; distinct TRAIN/DEV target distributions and diagnostic role labels remain separate at both rival counts | 9.69s |
| `cuda_document_evaluator_known_answers_and_native_artifact_roundtrip` | M1/M8 known answers, explicit TRAIN inventory, separate DEV oracle, native model save/load/emission and head-binding refusals | 4.33s |

Pure controls cover missing masks, multiple streams, zero/incorrect counts,
mask length/binary errors and missing/stale/changed content-policy bindings.
They execute no CPU model or numerical reference. An initial metadata filter
selected zero tests; that attempt is **NOT_RUN**, not a PASS. The corrected
filter then selected and passed all four helper metadata controls. Formatting,
claim-wording and diff checks pass separately.

The ten predecessor core CUDA controls at05c remain inherited, scoped receipts;
they were **not rerun by this unit**. These11 controls qualify the changed
count-reference/evaluator/caller boundaries, not a new full-shape core result.
No optimizer update or corpus/training/quality/transfer measurement executes.

| Source file | SHA256 at frozen source |
| --- | --- |
| `examples/geometric-stack.rs` | `0900358886a221aa36757ac06dbb83527aac2a67ea9ef0e3708f2e3f480712ca` |
| `examples/geometric-stack/native_rung0.rs` | `9b03e0fb1293c0143be705cda0ae49b63168e463b6b67c15abbf1adcaddb8045` |
| CLI CUDA test executable,630476208 bytes | `5bcaf8cd2b57b365e7e34c46a6fa5a48dce6388faf1c02202d081b85ae5c852d` |
| Normal CLI executable,575501032 bytes | `3db7978ab84c8cc83d11b5c522143cffd5e34e85b77200df2a45ccb68bce63a4` |

Source paths are relative to `crates/uor-r4-training/`. Qualification machine:
RTX5090, Ubuntu24.04.3, CUDA12.8.93, driver580.95.05, pinned Rust1.97.1.
The normal CLI compiles successfully in55.04s on the same frozen source; its
identity and exact byte size are recorded above. Build target storage is11GiB.
Memory peak is unmeasured for this unit. No new full-trunk
qualification, optimizer execution, training, corpus-quality result, fresh
transfer result, language gain or energy claim is inferred from this source
repair. Full M2 learned intent/compiler integration and D11 remain unqualified.

## Resources and delivery

The refreshed peer base has no changes to `Cargo.toml`, `Cargo.lock`, training,
core/integer/model-source crates or third-party sources relative to the build
base `3c8ef2e85e1e9af413311f34b4dd7d63cb9bac2b`; the owning session verified
that scope. The preserved frozen source
and executable/control identities remain the qualification authority. Actual protected merge source/tree equality is verified below.

Owned worktree `codex-rung0-standby-20261013-1217`; source delivery branch
`codex/rung0-count-reference-20261013` is released and deleted remotely/locally.
Claimed branch `codex/count-reference-closeout-20261013` carries this same unit's
living record corrections; Kimi's sheet and every peer's source,
claims, jobs and artifacts remain preserved. Root owned the scoped CUDA build
and finite qualification pod, now released/deleted. Pod closeout is
2026-10-13 at13:17:31Z:1371 seconds at$0.536/h, estimated$0.2041266667 before
provider billing (invoice unverified). No owned active pod remains. These
receipts cover the whole owned pod interval, including failed setup/compiler
attempts and qualification, not only successful CUDA execution. Two attempted builds lacked required
documentation metadata and are preserved as setup evidence. Build3 then exposed an `Option<Value>`
assignment error in reference-input serialization; frozen source
`2fd6358bbc52d2a202a872ad0954ebb12894606f` corrects it with explicit optional
JSON serialization, preserving absent input as null. Corrected Build4 and
all11 selected controls pass as recorded above. All setup/compiler
failures remain corrected tooling blockers, not mechanism negatives. No CUDA
control result is inferred from a build attempt or queue acknowledgement.
The [inactive restoration patch](../../history/branch-archive/rung0-whole-dev-count-oracle-inactive-20261013.patch),
SHA256 `b11645321f42d3a89136968b987dda045a58c0da7d16ecf0b2ee31f79b678b64`,
preserves the superseded whole-DEV helper/CLI source. Retention does not permit
its re-execution as the decisive reference. The preserved pod receipt tar
has SHA256 `d7a9d5b54877eab0f71b524d5ac26b5eef71ec63a55b5df0023cb64de60cd334`.
Preservation verification PASS: private HF dataset `caseyallard/uor-r4-store`,
commit `fa6f56f4ac6864e82030ee45365e679faaaa1a6e`, prefix
`pod-setup/rung0-count-reference/20261013-2fd6358bb`. Actual downloads verify
this pod receipt tar, source bundle SHA256
`a9a4c8be8a321d660a54139859cee37247fa723aa81b590ca3e9a1cd46ca9129`,
and qualification receipt SHA256
`8dff49e931c07abcc30d8d5e81e1d6470dcb9cf273a939acaa97836e3959eb20`.
The iCloud `cloud-store put codex` bundle
`codex-rung0-count-reference-20261013.tar`,6508544 bytes, MD5
`c3116f90d70007c81a138ee7950e8b5c`, passes tool round-trip and exact index
verification. Source and failed/corrected attempts remain recoverable.
Protected source PR2328 merge is `9e904a83448c451f738d5c1d45224dfdf73eb84e`
at2026-10-13 13:30:26Z, reviewed head
`4f1c08b3bd5d421f3bc5ed538e98e0499019bc65`. Fresh main contains it; actual
merge/full reviewed tree `ad87c29fb7d864467f39db3f48bdb78c3fb2f2f3` matches the
expected tree. Training source tree `85817cc162028b07476f3b3732818964b5c0fe58`
equals frozen source2fd. [Coordination6146731834](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6146731834)
requests Claude CONFIRM and Kimi's exact TRAIN-mask/CLI/new-source pins.
Amendment5 independent scientific CONFIRM FIT is recorded above; Kimi's
mechanical sheet pins and physical FIT verdict restoration remain next.

**Advisory harness CI: `receipts.parse` FAIL, preserved.** Rust `cmd_receipt`
defaults to kind `receipt`; the Python checker rejects it as `unknown kind receipt`.
The standalone onboarding invocation emitted that generic row at12:36:36Z in
Codex shard27. Rewriting the committed row would violate the append-only
multiset contract, so the row/source/logs remain unchanged and preserved in
main and the verified qualification bundle. [Report6146770185](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6146770185)
requests compatibility repair by the owning tooling line. This documentation
closeout changes no harness source, claims no parser fix and does not relabel
the advisory failure as PASS. Compiler and the11 geometry/interface controls
are separately qualified.
The independent confirmation accepts the delivered reference law; it is not
a language/quality result. Only the named finite qualification is scoped work; all training and separate
M2 holds remain HELD. Scientific FIT cannot release them. Delivery through a
protected source merge and fresh main/source verification are complete;
closeout record delivery precedes cleanup of its redundant owned transport
material.
