# Fresh-scoring eval-only path — source delivery, NOT_RUN

**Owner: Codex/Sol.** Implements [#2523 ruling 3](https://github.com/UOR-Foundation/uor-r4/pull/2523#issuecomment-6201872714)
and the [scope confirmation](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6202749743).
Independent exact-head review belongs to **Kimi PM's dispatched reviewer**.
This is required before KEEP/capability promotion, **not a training-start gate**.

The standalone Rust [instrument](instrument/Cargo.toml) reuses the production
CUDA checkpoint restore, evaluator and GPU aggregate. No production crate,
kernel or training-driver source changes; #2525's admitted driver hash is not
invalidated by adding this consumer. Host metadata tests/data inspection are
complete. **CUDA-feature compile, checkpoint-backed plan validation and GPU
evaluation are NOT_RUN**: no learned K16 checkpoint was supplied or constructed.
No pod, training, model forward/scoring/proposal/update or owner-release operation
ran during source delivery. There is no new model-quality result or KEEP.

## Immutable population and exact correspondence

The executable embeds the existing [packet data binding](../admission-redirect-packet-2026-10-15/data-binding.json),
not a replacement selection or caller-overridable mask. It requires exactly
**2,000 eligible targets in 200 documents**, 505,085 payload tokens, C192,
rows64, V4096, and `Split::Fresh`'s distinct identity namespace.

| Artifact | SHA-256 |
|---|---|
| Fresh UORT tokens | `82ead8db9584544a4cc461c394aaf9b00fa6c9fd8fdbf82e316392a71ecabd7b` |
| Fresh response mask | `8287d20d2c76d388ab7ed9becda6f209d224574a933fa2d14c7bdda3663f5ce4` |
| Frozen `fresh/rows.json` | `086443039e52cf5057fe81f3fa3d1bf938083936080b7cdff378ec7d547440a2` |
| Frozen `fresh/documents.json` | `6da8d07d5c6edddbf15c4d31c2d4d1c855bb727bcbd41684e1aca7f492f3f616` |
| Canonical ordered correspondence from this reader | `c3a85f250e84017410859621234c48bf6beaae0bab202d913eb201639a57124e` |

All 2,000 rows were checked locally against these actual private packet files;
the [inspection receipt](data-inspection.json) contains metadata only. The reader
checks ascending unique target positions, exact target tokens, context+target
SHA, contiguous causal occurrence order, document identity, version0 and Fresh
namespace. This reruns correspondence, not the earlier large train/dev overlap
search: the already-reviewed disjointness certificate is bound by the embedded
packet plus immutable payload/mask/row/document hashes. It cannot silently switch
to ft/dev or choose new targets. Provenance remains **held out from this D32
contrast, historically used pre-D32 chat data; not globally unseen data and not
2,000 independent conversations**.

## Checkpoint and admission authentication

The request schema is `uor-r4.fresh-scoring-request/1`; unknown fields are rejected.
Its fields are:

| Field | Required value / source |
|---|---|
| `checkpoint` | FilePin `{path, sha256}` of the sealed **completion wrapper**, not a raw/rebound model |
| `admission` | FilePin of that run's original `uor-r4.redirect-owner-admission/1` record |
| `expected_admission_sha256` | Canonical typed admission digest recorded in the completion wrapper |
| `expected_lineage_sha256` | Training execution/lineage digest from that admission |
| `expected_model_storage_sha256` | Model storage digest in the pinned driver checkpoint |
| `expected_cursor` | Exact five-field Cursor from the sealed checkpoint, including accepted updates |
| `fresh_directory` | Root of the existing private packet archive, containing `fresh/` |

The owning agent binds these from sealed run artifacts before execution; this
source delivery does not invent checkpoint pins or generate an example that could
be mistaken for a measured model. Only K16-A/R seed1/2, checkpoint0 or registered
quarter cursors 4,101/8,202/12,303/16,404 are admitted. Both families' attempted,
sensitive and accepted counters must agree with the cursor, including all 20
Route coordinates at the final checkpoint. No K8 pilot parent is allowed.

The reader validates the full wrapper hash, admission hash/schema/arm/seed,
configuration and lineage, current driver hash, model storage hash, model
source/PTX/geometry bindings, original train/dev/tokenizer pins, epoch and cursor.
`PreparedDriver::validate_checkpoint` and CUDA `restore` retain their existing
checks; the serialized snapshot must equal the actual restored snapshot.
There is no source migration, evaluation-time K widening, reseeding, objective
substitution or DEV-as-Fresh configuration rewrite. Preserve original relative
corpus paths when staging a sealed run; do not rewrite lineage metadata to make
another layout pass. Paths of the request's outer FilePins may be relocated
without changing pinned artifact contents.

This is **artifact and provenance authentication**, not cryptographic identity
authentication or independent proof that training occurred. It does not invoke
release validation to grant new training, create/copy an owner record, or issue
an update capability. The admitted checkpoint's historical accepted updates are
retained; only **evaluation's own updates are zero**.

## CUDA execution and registered metrics

One invocation evaluates one pinned checkpoint, 32 batches (31×64 + 16 rows).
CUDA performs every model read/writer/scoring operation and all aggregate metric
calculation. Host work is file/hash/identity/receipt orchestration only.

- Per-row output retains ordered position/token correspondence, context length,
  record identity and causal-row SHA alongside GPU observations.
- Production observations include top1/top8 and ranked IDs; target-cell rank,
  admitted/occupied cells and token inventory; T/A/R; both `8T+32A` and `8T+32R`;
  the active objective; strict and admitted-strict wins; scale-disclosure sums,
  exact ratios and hinge histograms/quantiles. The legacy total-loss field stays
  explicitly separate from the active objective; consume the named objective
  fields for R. No new host metric predictor or count-reference baseline is run.
- Row count, batch count, objective and complete ordered coverage must match
  before the final GPU aggregate is accepted. A truncated zip cannot silently
  drop rows.
- Snapshot SHA and epoch are captured before and after evaluation; equality is
  required, and epoch is checked per batch. No apply, proposal, transition or
  initialization call exists in the evaluator. A source ratchet checks that
  exclusion. The checkpoint is restored, never modified or re-saved in place.

Raw fresh evidence does not automatically promote KEEP. PM interprets the
matched arms/seeds alongside the registered DEV result. Missing or malformed
inputs, mismatched rows, GPU errors or state drift are instrument/input blockers,
never negative mechanism evidence; preserve and report them without silent retry.

## Commands and qualification boundary

From the repository root, metadata-only operations are:

```sh
MANIFEST=docs/labs/fresh-scoring-eval-2026-10-15/instrument/Cargo.toml
cargo test --locked --manifest-path "$MANIFEST"
cargo run --locked --manifest-path "$MANIFEST" -- inspect-data PACKET_ROOT
cargo run --locked --manifest-path "$MANIFEST" -- plan REQUEST.json
```

`plan` also requires the original pinned train/dev/tokenizer files used by
`PreparedDriver`; it never creates a CUDA context. The final two commands need
real input paths, not the uppercase placeholders. All metadata validation
refuses missing files or mismatches.

Only after exact-head review and separately authorized GPU execution, the owning
agent runs the following **inside `scripts/pod/uor-pod run`**, with the admitted
GPU architecture and a new empty output path. These commands were **not run**:

```sh
cargo build --release --locked --features cuda --manifest-path "$MANIFEST"
cargo clippy --locked --features cuda --all-targets --manifest-path "$MANIFEST"
cargo run --release --locked --features cuda --manifest-path "$MANIFEST" -- evaluate REQUEST.json NEW_OUTPUT_DIR
```

Archive the build/clippy transcript, exact source/executable/PTX/geometry,
toolchain, request and run receipts. The source review and seven host tests do
not discharge this CUDA pre-flight or qualify runtime behavior. This adds no
rental authorization and does not consume the current qualification lease.

## Receipts, failures and archive

An exclusive output directory prevents overwrite/resume. Each batch receipt is
synced before advancing. Success contains `request.json`, `plan.json`,
`checkpoint-input.json`, `admission-input.json`, `before.json`, `batches.jsonl`,
`after.json`, `report.json` and `SHA256SUMS`. The input copies preserve original
artifact bytes; owner-release records themselves are neither read nor copied.
The report binds the evaluator source/binary, production compiled identity,
training lineage, model, cursor, Fresh binding and complete row correspondence.

On failure, completed batch receipts remain and `failure.json` plus `SHA256SUMS`
are written when filesystem I/O permits. An incomplete/crashed directory without
a successful report is not a score. Never delete or reuse it; archive partial
attempts and diagnose. Before pod teardown, the executor verifies SHA256SUMS,
archives the directory with `cloud-store put codex DIR`, verifies the MD5
round-trip, then releases/terminates via `uor-pod`. Record lease/wall/storage costs.

## Verification and delivery

- Seven standalone host tests pass: checkpoint/admission substitution, quarter
  and Route-tail counters, row/context/namespace correspondence, frozen pins,
  state-change rejection, update-call exclusion and immutable failure receipts.
- Actual frozen data inspection: 2,000 rows, 200 documents, all pins and ordered
  correspondence match; model operations zero.
- Host check/clippy complete (dependency dead-code warnings, no CUDA claim),
  formatting and source whitespace checks pass.
- `git diff origin/main -- crates/` is empty. The training driver and its admitted
  digest remain unchanged; the consumer has its own source/binary identity.

No GPU time/cost, pod storage, training or model measurements. Local source/test
outputs use the existing owned cache. M1 headline 37/232 and D21/D22 count are
unchanged; M2/Stage B remain HELD. Next: PM-dispatched exact-head review, protected
merge, then authorized CUDA pre-flight and fresh scoring of actual sealed runs.
