# #1864 — Isolate continuation energy at the failed native entry checkpoint

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T01:12:40Z
- **Updated:** 2026-10-09T13:17:08Z
- **Closed:** 2026-10-08T01:26:11Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1864
- **Merged:** 2026-10-08T01:26:11Z
- **Merge commit:** 830725c6c96296f7e19e71fe71b7089874bd3a42
- **Base / Head:** main / UOR-Foundation:codex/astra-entry-zero-u-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The delivered joint64 model fails all512 entry decisions despite nonzero continuation coefficients. This adds a pinned native diagnostic to determine U's conditional effect at that exact final Source/Generate checkpoint.

The explicit mode first authenticates the genuine joint64 checkpoint and learned U. It constructs and independently reloads a same-version v2 zero field, executes both empty-prefix native forwards on every original packet, and checks identical factual/local states, selected occurrence and complete action support. Every Generate and physical Copy score difference must equal that token's learned U delta. Labels determine metrics only after both forwards; the learned arm must reproduce saved final64 entry evidence.

The compact report retains tie-aware winner/target/rival masses, exact normalized probability and margin changes, original-eight entry retention, state/occurrence provenance, vector hashes and strict invariant receipts. It changes no model, learner, generation rule or normal consumer admission. Legacy own-prefix evaluation remains the default. No fit or full autoregressive rerun is introduced.

Four targeted tests address strict artifact admission, duplicate aliases and incorrect factual bases, tie handling, and cross-reference normalization. All13 example tests and the release build pass at executed sourcee53312a333f16785b1d61e5f003d0095aa047361. Exact-head mathematical review found no blocker.

Actual paired native execution and independent saved-evidence audit pass. Learned U and zero-v2 each produce0/512 correct entry decisions; neither retains any original8 parent entry.191 winner IDs change. Removing U improves target probability on133 rows and worsens it on379; normalized target-versus-rival margin improves on291 and worsens on221. The native process takes17.192s, with9.225s for1024 forwards; no fitting, full autoregression or GPU use occurs.

The independent reader verifies every row/packet hash, learned-arm saved-entry parity, exact rational arithmetic, transitions and original8 entry sets. Full-vector equality and global rank/max checks are source-reviewed producer invariants, with compact hashes rather than an independent second forward. Separate Rust seal verification passes. This establishes no rescue by removing U at the final endpoint; it does not establish no U effect or identify an upstream component.

Complete evidence is preserved at private HF revision `5195efaadc22134fc8d6bd884cc800742ca94634`, prefix `codex/astra-entry-zero-u/entry-e53312a333`, archive SHA256 `6faa55a6fccbe3705071fa9dcdf48deaebacafc5b26c020712474a196ac5d9c8`, freshly downloaded and verified. Final delivery documentation records the next parent-versus-joint1 Source/Generate onset attribution as NOT_RUN. Final head `4b1787047b0bb42b818156a921397878d70fd5aa` passes formatting, all13 focused tests, the release build and claim-wording validation. Independent exact-head source/claims review finds no blocker; the executed Rust source is unchanged. Final delivery checks, review, source patch and ledger are separately preserved at private HF revision `9fb20ac34d3700537d6351591624dc0da496be49`, prefix `codex/astra-entry-zero-u/final-delivery-4b1787047b`; all11 files freshly downloaded and verified.

No new model is promoted. Accepted parent, negative checkpoints and counterfactual remain preserved. No complete-reply, EOS, transfer, chat, energy or interaction claim is made for the zero arm.

References #820


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6050119397 — 2026-10-08T01:12:55Z — Casey-allard

# Source-only review of entry zero-U diagnostic implementation

Reviewer: `/root/math_history`. Scope: new diagnostic in
`crates/uor-r4-core/examples/native-bank-generalization.rs`, compared against
`/tmp/astra-zero-u-counterfactual-review.md`.

Reviewed working-file SHA256:
`977545e229a9cd91cddad174ddd1ba5f478aece84d9bf94d2e5bd1e6ca826963`.
Base HEAD: `b5f7bd7ad7d1eeb3218059af8f96a2af5c5b953f`.
Exact final reviewed head: `e53312a333f16785b1d61e5f003d0095aa047361`.
Verified the committed consumer blob has the stated SHA256 and the working file
has no diff against that commit. No edits to implementation, builds,
tests, model loads or numerical executions were performed by this reviewer.

**No blocking source finding.**

- Admission still authenticates the actual joint final64 report, manifest,
  receipt and learned U path/hash before constructing the diagnostic control.
  Exact input/label hashes and all512 cases are required. The zero field cannot
  pass through ordinary joint admission in place of the original field.
- Zero construction uses `compile_shared_action` with correct packed length,
  reloads saved bytes, asserts v2/shift22/Copy application, zero payload and
  metadata equality except payload hash. Both arms retain the same bound native
  Source, Generate, bridge, prototypes and tables.
- Both production `step(&bank, &[])` calls finish before accepted text is
  tokenized to define the target. Label IDs are checked for alignment earlier,
  but do not select or alter the bank, carrier, candidates or scores.
- Equality is enforced for complete bank replay, bridge selection/action,
  factual post-state, local U state and ordered aliases. Full4096 support and
  each physical Copy occurrence are checked. Every Generate/Copy raw difference
  must equal the learned field's token delta; control deltas must be zero. Both
  arms use the existing common reducer and clip, with exact action checks.
- Learned arm must reproduce each authenticated saved final canonical entry:
  target, pool summary, factual state, Copy IDs, Generate hash, target mass,
  denominator and local-U witness. A replay mismatch fails the attempt rather
  than being presented as a counterfactual result.
- Served token masses, not raw score ranks, define the outcome. Target minus
  strongest rival uses signed arithmetic, and normalized between-arm changes
  use checked cross-products. With positive denominators their numerator sign
  is exactly the sign of the rational probability/margin difference. Wide
  integer values serialize as strings. Strongest-rival identity may change
  between arms, which is appropriate for measuring the actual decision margin.
- The comparator resolves equal masses toward the smallest token ID and checks
  the result against the actual served winner. Zero margin is not treated as
  automatic success or failure. Target tied-for-maximum status, strict mass rank
  and smallest-ID tie-broken rank are recorded. The latter correctly adds only
  equal-mass IDs smaller than the target to its strict mass rank.
- Report interpretation correctly limits the result to the conditional U
  effect at one coadapted upstream endpoint. It makes no complete-reply, EOS,
  transfer or measured interaction claim. Legacy own-prefix evaluation remains
  the default mode.

Meaningful proposed tests cover admission, duplicate-alias differences and base
score drift, tie resolution, and normalized wide arithmetic. Their source was
reviewed; actual pass/fail evidence belongs to the executing lab.

The final engineering additions were reviewed: target/rival Generate preclip
and U scalar readings occur after the forward and identify the current arm's
strongest rival; target Copy multiplicity and original8 retained/lost ID lists
are evaluator-only. Raw reducer diagnostics are correctly described as
tail-limited and distinct from served common-clipped selection.


### comment 6050255077 — 2026-10-08T01:25:17Z — Casey-allard

Exact final head: `4b1787047b0bb42b818156a921397878d70fd5aa`.

Actual remote CPU-only checks: formatting PASS (0.030s); locked/offline/release consumer tests **13 passed,0 failed** (3.574s including build); release consumer build PASS (7.083s). Local claim-wording validation PASS. No GPU or model rerun. Rust source byte-identical to executed E533.

Independent mathematical source review and final adversarial source/claims review: no blocking findings. Executed512-row paired control, exact compact-evidence audit and separate Rust seal verification are documented in the PR body; none are inferred from CI compatibility acknowledgements.

Final checks/logs, independent review, source reconstruction patch and cumulative ledger preserved at private HF revision `9fb20ac34d3700537d6351591624dc0da496be49`, prefix `codex/astra-entry-zero-u/final-delivery-4b1787047b`; all11 files freshly downloaded and SHA256 verified. Manifest SHA256 `31e7e01219c8cea5ca899605152dfd0a20e22fad5c7d590587a6b9ab540fabc8`.

No model promotion. Next parent/joint1 component-cross attribution is NOT_RUN.



---
_Comment/review ids in this file: 6050119397, 6050255077_

