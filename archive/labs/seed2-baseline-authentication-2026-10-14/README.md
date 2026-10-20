# Seed-2 checkpoint-zero authentication — 14 October 2026

**Design and source for review; no seed-2 model compute or deployment.** Seed-2
remains HELD for Casey's decision after seed-1. This change does not modify the
running seed-1 executable, plans, release records, checkpoints, geometric kernels,
initializer, writer, objective or schedule. It adds an operational metadata guard
before the completion controller can issue update admission to seed 2.

Line: evaluation integrity; no model headline or research-count change.

## Gap and mechanism

Previously `validate_progress` checked the baseline's epoch and lineage string.
Resume required the current baseline to equal the baseline in the immutable
initial checkpoint. That preserved the baseline but did not fully authenticate
its initial model, row-derived strict-win numerator **u**, or count-reference
tables before the first apply.

`completion.rs` now calls `baseline_authentication::require_before_apply` before
`issue_corpus_update_admission`. The helper returns immediately for seed 1. For
seed 2, both a fresh invocation and every resume must:

1. Read and hash-verify `initial.json`'s immutable checkpoint FilePin. Require a
   fresh seed-2 checkpoint: all cursor fields zero, epoch zero, no pilot parent,
   and zero VectorBlock/Route counters.
2. Bind the exact plan/admission, complete config and lineage (including TRAIN,
   DEV, masks, tokenizer and traversal policy), numerical source/PTX/geometry,
   model bindings, stored model checksum and baseline snapshot metadata.
3. Bind the baseline evaluation to the same checkpoint-zero snapshot. There are
   two existing digest domains: checkpoint storage serializes a JSON `Value`,
   whereas the evaluator serializes the typed CUDA `Snapshot`. The guard uses
   the existing `Snapshot` deserializer/serializer to verify the latter; it does
   not equate the two hashes or invent a duplicate snapshot schema. This step is
   storage processing, not model initialization or forward computation.
4. Require the **entire** count-reference object to match canonical compact
   sorted-JSON SHA-256
   `b92da0bf1ccda1c9de56a33de41d4698dc214bd71c588a6938a87443b9aca0b9`.
   This reference includes both complete 4,096-entry tables and the DEV
   diagnostic-only marker. Its source is the authenticated archived pilot report
   `e4bcf43859fa0645db2ca0d5fe620f18620990cbb7efb4f28e6873f5a57f90c3`.
   The fixed TRAIN/DEV/tokenizer/mask inputs are identical across seeds. **Only
   their seed-independent reference is reused; pilot u is never reused.**
5. Recount seed 2's own 14,287 saved observations in 224 DEV batches, requiring all
   row/batch/panel sums, weighted loss identities, rank identities and admission
   bounds to agree. Compute `u = count(token_rivals>0 && token_hinge<32)` and B3
   from those observations. This reads GPU receipts; it runs no CPU predictor.
6. Durably publish `baseline-authentication.json` and its adjacent `.pin.json`
   under the existing run lock, or verify both against the recomputed expected
   bytes on resume. Only then may update admission be issued.

The authentication receipt pins the original checkpoint, plan, admission,
lineage, both snapshot digest domains, complete baseline evaluation and reference,
`u` numerator/14,287 denominator, B3 and evidence scope. `u > 95%` is recorded as
`instrument_invalid`; the registered sheet says **bars decide alone**, so this
case does not invent a training stop or a mechanism negative.

## Failure and recovery

A missing or mismatched baseline, reference, receipt or pin refuses update
admission. Resume cannot substitute seed-1 evidence or a trained checkpoint for
checkpoint zero. It verifies that the current checkpoint retains the original
baseline and snapshot metadata.

If interrupted between checkpoint-zero persistence and authentication publication,
a verified **cursor-zero** resume may finish publishing the receipt/pin. Once the
cursor has advanced, missing authentication is a blocker; it is never reconstructed
as if it predated apply. Existing immutable names refuse replacement, and the
checkpoint/run-state machinery retains partial files for diagnosis.

This is metadata provenance and internal-consistency authentication, **not a
cryptographic attestation that a GPU executed**. The existing receipts omit
explicit target/occurrence IDs per row and bindings/epoch per batch. Their order
still relies on the pinned deterministic DEV traversal and qualified evaluator.
There is no independent measured seed-2 u yet.

## Verification and adoption boundary

Five focused metadata controls pass with no CUDA/model execution: successful
synthetic checkpoint-zero metadata and ten perturbations; high-u instrument-invalid
semantics; immutable publication, missing-pin refusal and cursor-zero recovery;
typed-versus-Value serialization domains; and the actual authenticated pilot
reference/row check (u=105 belongs to **pilot seed 1**, not seed 2). The external
artifact check is explicitly ignored by default and was expressly included in
this validation with its pinned report supplied. Synthetic fixtures are labelled
storage observations, not generated model results.

The full default-feature metadata suite also passes: **29 passed, 1 explicitly
ignored retained-artifact check**; that retained-artifact check passed in the
separate five-control invocation above. D23/D25 ratchets remain 1,641 and 93;
formatting and claim-wording checks pass. The numerical source files and released
packet have no diff from the starting main commit `ae3aa4dc3`.

The new module is included in `driver_source_sha256`. Thus **the operational
driver identity changes**, while numerical model source remains untouched.
Before any future seed-2 execution, Claude/Kimi must review the exact source,
qualify the new CUDA build and actual gate, bind its new driver/executable/lineage
identities in a seed-2 packet and review, and obtain Casey's personal release for
that exact packet. Existing SEED2_PLAN pins are deliberately unchanged and cannot
silently admit this new binary. No seed-1 migration or restart is requested.
The CUDA-gated typed-Snapshot integration is source-reviewed only here; default
metadata tests do not qualify it. No additional GPU work is authorized by this
record or by merging its source.
