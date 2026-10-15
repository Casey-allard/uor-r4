# #1388 — Complete D8 reference replay and matched count/cache baselines

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-24T22:42:16Z
- **Updated:** 2026-10-09T13:17:52Z
- **Closed:** 2026-09-24T22:44:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1388
- **Merged:** 2026-09-24T22:44:04Z
- **Merge commit:** c84b198df3b47bc8326307dd9964dc9c87cb830c
- **Base / Head:** main / UOR-Foundation:codex/reference-baselines-20260924
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Completes D8 rung 0 at its declared reference/comparator scope and proceeds to the canonical recurrent-memory learning campaign. No native model promotion and no neural optimizer steps. References #973 and #820; their full acceptance remains open.

## Implemented

- Shared detached batched reference evaluation in the offline Rust training crate, retaining the differentiable training path.
- Normalized count-pruned fixed-discount interpolated Kneser–Ney 5-gram, using lower-order continuation counts from unpruned raw types, plus a causal token cache.
- Pinned evaluator v2, train-only count fitting across both inherited stores, prefix calibration and separate comparison tail, exclusive/sealed report directories, actual per-target records and historical generation replay.
- Current-state/roadmap/navigation corrections and one concise result with retained source, executable, artifact, resource and review evidence.

## Actual results

All arms score identical targets. On the 233,472-target comparison tail, mean NLL (nats/token; lower is better):

| Arm | NLL |
|---|---:|
| #1017 reference | 1.574024 |
| Count-pruned interpolated KN | 2.405627 |
| KN + causal cache | 2.391786 |

Full reference NLL over 249,856 targets is 1.5802411896906157, within 1.173173e-7 of the historical result. All five actual generations reproduce all 582 generated IDs, decoded text and stop reasons. Batch isolation and future-input prefix deltas are zero. Selected count discount is 0.9; cache mixture is 0.05.

## Executed verification

- `cargo test --release --locked --offline -p uor-r4-training --lib --features metal,reference-accelerate -- --test-threads=2`: six focused tests pass.
- Source-bound optimized offline Rust build passes.
- Actual CPU32 numerical/gradient/finite-difference integrity check passes after shared-forward refactor.
- Actual full reference evaluation, five generations, one count fit/export/reload and count/cache evaluation all pass without retries or changed gates.
- Independent complete-file verification of all four sealed roots; direct output/decision replay audit; root audit of every input/target row and reported mean.
- Workspace formatting, claim-wording and diff checks pass.

The five CI status names are queue compatibility acknowledgements, not executed formatting or tests. Local results above carry validation. Existing unrelated dead-code warnings were left scoped.

## Scope and resources

The reference remains an offline ordinary floating-point transformer. Existing development was used for historical checkpoint selection; the tail is separate only from current count calibration, not a fresh final holdout. Replay equality is not answer correctness: repetition, semantic drift and three capped outputs remain. Count source population matches; sampled optimization exposure and artifact sizes differ. The selected cache comparison is specific to the declared pruning/calibration, not optimal count models. No geometric, native-language, final-serving or physical-energy claim follows.

Remote Desktop Commander supervised concurrent count/reference processes on the same local Mac, alongside separately owned implementation and independent review. Model/integrity process times sum to 85.48 seconds. New build plus retained storage is 713,314,304 bytes; the protected physical reserve and stop margin remain intact. Full engineering and delivery allowance is charged to the shared ledger. No paid compute or unique-material deletion.

Evidence: `docs/integration/reference-baselines-result-2026-09-24.md` and `docs/evidence/reference-baselines-result-2026-09-24.json`.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

