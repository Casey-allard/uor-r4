# Independent U/R finite run2 outcome review

Reviewer: Codex independent reviewer `review_2551`. Verdict: **APPROVE actual finite outcome** at the pinned source/executable scope. This is not a timing or training execution release.

## Evidence and exact identity

Reviewed archive: `/Users/casey.allard/uor-r4-local/codex-cf1-reuse-finite-run2/reuse-finite-run2-archive`.

- Raw finite record: `finite-output/counterfactual-reuse-controls-record.json`, 426,092 bytes, SHA256 `c6a5b3f857d77bf979ed439e61575f04fce2f398b9854286681eac53203cd4f4`.
- Source: `5f533cbfd49ecef9a4ea2b217bb31a93002b3dc2`.
- Executable: SHA256 `671917acb13b541c148b6b8122b92ae613ac0cdff8577a3f72fb6c05faa06209`.
- Compiled composite: `aca299c484d5434291f7f65cc9315d6b52b976952bcd68f39a45cd3341bf2825`.
- Final pin: SHA256 `e3f5ff59036a164cfb15ab18cbc1be41ad1e409ebe00cebe5f3a34463504a7c4`.
- Release: `https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6227632827`.
- Independently verified 22 remote-manifest entries, 20 operator-manifest entries and four input identities. Actual identity compares byte-identical to expected identity; final pin differs from review pin only by the release URL. The invocation marker records one instrument invocation.

## Independent exact-integer audit

Metadata checker source: `audit.rs`, SHA256 `d9f8838c0f47cf639c88343329ee35f096ee6a34574815a49fe6a1b3fb2ed146`, in this directory. It uses Rust/serde_json exact u64 parsing and compares retained observations. It contains no host model/reference evaluator, GPU operation or kernel invocation. The serde dependency cache was read only; no Cargo build or repository mutation was performed by this reviewer.

Successful compilation used:

```sh
/Users/casey.allard/.cargo/bin/rustc +1.97.1 --edition=2021 /Users/casey.allard/uor-r4-local/review_2551/reuse-finite-run2/audit.rs -L dependency=/Users/casey.allard/uor-r4/local/harness-target/debug/deps --extern serde_json=/Users/casey.allard/uor-r4/local/harness-target/debug/deps/libserde_json-97f3c5da5ec49b54.rlib -o /Users/casey.allard/uor-r4-local/review_2551/reuse-finite-run2/audit
```

Actual successful audit invocation, exit0:

```sh
/Users/casey.allard/uor-r4-local/review_2551/reuse-finite-run2/audit /Users/casey.allard/uor-r4-local/codex-cf1-reuse-finite-run2/reuse-finite-run2-archive
```

`audit.stdout.txt` preserves the three stdout lines verbatim from that successful tool execution. It was written from the tool transcript, without repeating model or audit execution. Its PASS line is:

```text
PASS exact-u64 metadata audit: 32 U/R pairs, 56 success/8 status8 attempts, 512 R lanes, 414 producer rows, 16 typed refusals, 18 strict resource admissions, unchanged state/epoch/apply, zero endpoints/timing.
```

The checker verified all sixteen exact u64 losses/statuses in 32 U/R pairs, identical five proposal words in 28 successful pairs, and matching absent proposals/typed errors in four refused pairs. All 512 complete R lane records and 414 producer-row outcomes agree between raw and decoded storage. Producer masks form exact disjoint partitions; consumer/eligible/virtual masks and terminal statuses match the fixed case predictions, including full64 masks without floating conversion. All nine cases run under both policies; boundary16/17/64 are covered. All sixteen typed API refusals, unsubmitted-audit guards, state invariance and zero epoch/apply satisfy the fixed contract.

Additional readback inspection of the small exact tie values confirms VectorBlock candidate3 and candidate12 tie below identity: nearest-address448 to352; occupied-rank192 to160; candidate3 wins. Route ties preserve identity. The causal Route fixture reports `[0,64,64,0,1]` under both policies: sensitivity without an improving update. Mixed eligibility preserves the eligible reference; virtual-ineligible producers cover both rows while nonidentity consumers stop after row0. These checks compare observed receipts and prospective literal outcomes; they do not recompute model losses.

## Actual runtime resources and boundary

Actual hardware RTX5090, driver580.95.05. Command started00:58:01Z, ended00:58:04Z, exit0.

| Observation | U | R |
| --- | ---: | ---: |
| Registers | 88 | 115 |
| Local bytes | 21536 | 21536 |
| Static shared bytes | 0 | 512 |
| Maximum threads/block | 640 | 512 |
| AOT spill stores/loads | 0/0 | 0/0 |

All18 comparisons pass the unchanged own-AOT equality and block16 predicates. R's runtime shared attribute512 is observed separately from raw ELF/cuobjdump1536/reserved-cap1024; no subtraction was used. No shared-memory waiver, additional dose, endpoint rerun, timing or apply occurred.

## Preservation, closure and limits

Local operator receipts show cloud archive7,257,088 bytes, MD5 `db89eef2659326af31c0e6cad71034e1`, full-readback success before lease release00:59:08Z and deletion00:59:15Z. Operator exit0 and fleet0 are recorded; parent independently confirmed closure. Final lifecycle/cumulative costing is the parent's separately reconciled ledger responsibility.

This establishes finite fixture parity and admission for these exact bytes on580.95.05. It establishes no result on580.105.08, no cross-driver parity, original-checkpoint result, speedup, training-rate, language or energy result. Previous stopped run1 remains sealed. Protected finite-result delivery, cleanup/hygiene, distinct timing readiness/pins/release remain dependencies. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; seed2 and production activation excluded.
