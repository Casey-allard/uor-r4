# Object-store exact-read successor to Stage 0

## Evidence and decision

kimi-40 established two different facts on the pinned T16_fz D11:

- The integer artifact reproduced the sealed Stage T receipt on all 14,287 DEV rows, with byte-identical repeats and zero aggregate position-strip delta.
- The model did not copy an explicitly supplied eight-letter value: 0/40 exact matches in the exact-hint, no-hint and random-hint arms, repeated at repaired head `11624e808`.

The next oracle dependency is an object-store composition whose value does not have to be recovered from the model's current writer. This is a different measurement from model copy, not proof that a curriculum or optimization change could not teach copy. It does not call the result unaided generation, a learned address mechanism, or a repair of T16_fz.

The first draft proposed a new integer token store and whole-crate scope expansion. D68 review rejected that route: the delivered object store and Stage 0 runner already provide the simpler matched instrument. This revision keeps `crates/uor-r4-object-store/**` as the only implementation scope and does not touch `crates/uor-r4-integer/**`.

## Claim boundary

Keep four quantities separate:

1. **Store correctness:** an exact `PrimeAddress` resolves to the requested version's verified content, or returns `Absent`/`Evicted` without substituting another value.
2. **Oracle assistance:** supplying the true address changes assisted output relative to deterministic sieve, no-address and no-memory controls.
3. **Address formation:** deriving that address from a fresh natural-language query. The true-address oracle supplies it and does not measure formation; the sieve is the deterministic address baseline.
4. **Model generation:** the D11 writer's own top-1/top-8 and free-running emissions. These remain the baseline and are never replaced in a baseline receipt.

A direct store emission is evidence for (1) and, against matched controls, (2). It is not evidence for (3) or (4). A high model baseline can make the measurable gain zero even when the store is correct; that is a ceiling outcome, not an instrument failure.

## Concrete object-store integration

Implement in a later ORDER within the existing scope:

- Reuse `PrimeAddress`, `AddressDirectory`, `ObjectTarget`, `AddressReadOutcome` and `VersionSelect` from `address_read.rs`. `PrimeAddress::with_pairs` supplies primality, canonical sorting/deduplication and ordered-pair endpoint validation.
- Use `AddressDirectory::read_exact`/`read_exact_from` for the true-address arm and `read_sieve`/`read_sieve_from` for the deterministic address baseline. The object store's `BTreeMap` exact read and content verification are the authority; do not add a token-map store with different version semantics.
- Keep `Current` and `Previous` only. A missing previous version is `Absent`, never `Current`. Exact-key equality, including eviction handling, must not degrade into shared-prime admission. Add an overlapping-but-unequal evicted-key control that returns `Absent`, not `Evicted`.
- Extend the existing `address_stage0.rs` instrument with a separately named oracle command/report rather than duplicating its artifact, tokenizer, identity, parity and stop machinery.
- All model evaluation, including every fallback, uses `m1-d11-artifact-scorer-instrument` on the GPU. Do not call `IntegerDiscreteModel::evaluate_prefix` on a host for evaluation. Host tests cover only pure manifest, address, version, outcome and emission-selection functions.

The default D11 artifact and production generation path do not change in the oracle unit. Any later promotion into default serving needs separate evidence, D11 audit, cost accounting and review.

## Versioned inputs and receipts

Use a small versioned panel manifest, schema `uor-r4.address-oracle-panel/1`, with SHA-256 and explicit limits for rows, slots, key atoms, value tokens and prompt tokens. Each row names its prompt, exact address, requested version, expected typed outcome and expected value tokens. Token IDs and occurrences are checked against the pinned tokenizer/object-store bindings.

The manifest versions the sieve lexicon and its word-to-prime registry. For each query, split the prompt into words, keep lexicon words in order, take their primes, and form ordered pairs between adjacent kept words. This deterministic rule is the sieve address baseline; the rejected panel and its bars are not reused.

Expected values must be absent from TRAIN and DEV token sequences. A cross-row scan must also show that no expected value occurs in any model-visible prompt or dialogue, not merely in its own row. Random-address keys must be present in the store, exactly unequal to the true key and bound to a different value.

The store is constructed deterministically from the manifest. No persistent sidecar format is needed for this instrument. Do not edit the D11, its checkpoint identity or the Stage T receipt.

Every run receipt binds:

- D11, original checkpoint, tokenizer, TRAIN/DEV, original/staged/normalized config and panel hashes;
- source HEAD, executable, PTX, cubin, geometry, kernel and driver-source identities;
- CUDA toolkit, CUDA driver API version and, when captured separately with `nvidia-smi`, the NVIDIA display-driver version;
- store capacity, value/prompt limits, per-row address/version/outcome, baseline and assisted emissions, memory accesses and bytes read;
- no-memory/no-address equality, sieve/true/random outcomes, typed abstentions, mismatches, throughput and complete pod cost.

## Five matched arms

Run every row under the same artifact and prompt boundary:

| Arm | Store | Address supplied | Meaning |
|---|---|---|---|
| No-memory | Disabled | No | L-010 control: model without the memory path |
| No-address | Populated | No | Model with store present but no oracle address |
| Sieve | Populated | Deterministic sieve query | Address-formation baseline required by PLAN's cancel condition |
| True-address | Populated | Correct exact address and version | Oracle-assisted upper bound |
| Random-address | Populated | Different present key with different value | Planted-wrong control, never an accuracy arm |

No-memory and no-address baseline records must be byte-identical. A true-address exact hit emits verified stored content through the assisted path and is marked `ExactStore`. A sieve hit is marked `SieveStore` and graded separately. Record `sieve_address_match` (selected address equals the true address) apart from `sieve_value_check_pass` (emitted value is correct), so address formation and value correctness cannot be conflated. A random-address hit demonstrates that the control can emit a wrong value; it is not silently replaced by the expected answer. `Absent` and `Evicted` produce typed refusal outcomes, not another slot's value.

`read_sieve`/`read_sieve_from` selects the matched slot's current value and has no version selector. Therefore previous-version rows report that known `Current` fallback descriptively and are excluded from the true-address-versus-sieve contrast. The contrast uses current-value and abstention rows only unless a later reviewed change adds versioned sieve selection; true-address versus no-memory is still reported for every family.

## Gates before a hidden-fact panel

1. **Pure host controls:** exact current/previous hits, wrong key, overlapping-but-unequal active and evicted keys, composite factor, ordered-pair endpoint outside the prime set, missing previous version, absent, eviction, invalid token binding, bounded capacity/value lengths, deterministic manifest generation and no unauthorized fallback.
2. **End-to-end GPU instrument:** on 40 current, 20 previous and 20 abstention explicit-value rows, the true-address assisted path must produce **40/40 current, 20/20 previous and 20/20 typed abstentions**, with zero wrong-value substitutions. This validates hard store emission, not learned copy. No-memory/no-address must remain identical and random-address planted-wrong rows must remain wrong.
3. **Baseline preservation:** the no-memory D11 DEV GPU scorer still agrees with the sealed receipt on all 14,287 rows. Any mismatch, nondeterminism or identity failure is an instrument stop.
4. **Pre-register every GPU evaluation:** gate 2 and any later hidden-fact run each need their own sheet, fresh non-author FIT and D67 pre-run review before dispatch.
5. **Only then freeze the hidden-fact panel.** It must hide values from all model-visible text and use fresh, unexposed address/value combinations. The same five-arm protocol and exact hard-emission bars apply.

The Stage 0 model-copy failure is not relabelled a pass. The repaired instrument succeeds by adding an exact object-store oracle read, a different and explicitly bounded claim.

## Oracle evaluation and decisions

The true-address hard-emission bar is exact: 40/40 current, 20/20 previous, 20/20 abstention and zero wrong-value substitutions. The old `27/40`, `7/20` and mismatched `19/40` baseline are not reused, and Stage 0's `d > 24` model-copy floor is not imported.

Report sieve and model baselines separately, with paired `b`, `c` and `d = b - c` for true-address versus sieve on current/abstention rows, true-address versus no-memory, and sieve versus no-memory on the same supported populations. Do not pool families to create a win. No BPB claim is made without a lawful likelihood evaluator.

Outcomes:

- Wrong substitution, baseline drift, missing control or emission error is an **instrument failure**.
- Correct store emission, low baseline and a true-address gain over sieve/no-memory measures the store's contribution and justifies a separate address-formation measurement.
- Correct store emission with the model or sieve already at ceiling is a **ceiling result**: no gain is measurable on this panel. Report it and do not train to manufacture a difference.
- Correct true-address oracle reads but poor sieve/address formation localizes the next question to addressing, not storage.
- A model-mediated oracle is out of scope for this design. If later proposed, it needs a distinct arm whose value passes through the model writer and a separately derived bar; hard store emission is never substituted for it.

## Resource and speed accounting

Capacity and value lengths are bounded by the manifest. Exact reads use the object store's bounded `BTreeMap` path plus content verification; sieve work is bounded by the declared directory and candidate limits. Report exact/sieve directory reads, backend fetches, candidates, value tokens and bytes read per row. Do not call an unbounded corpus scan exact addressing.

Report baseline, sieve and true-address tokens/s separately for decode and context/evaluation, plus store resident bytes and per-query accesses. No energy value is inferred from uptime. A later default-serving proposal must show that the object-store path preserves the D11 integer/no-multiplier constraints and measure its actual cost.

## Implementation slices

1. Add failing pure controls for the exact/eviction/token-binding edge cases and the versioned manifest.
2. Add the five-arm composition to the existing object-store Stage 0 runner, with no host model evaluation.
3. Pre-register and run the GPU hard-emission instrument with baseline parity, exact 40/20/20 gates, sieve and no-memory controls.
4. Review that receipt before drafting or freezing the hidden-fact panel.

The current unit delivers this design and item-text pointer only. It implements none of these slices and claims no new model capability.
