# Addressed geometric attention primitives and causal credit — #973

**PASS_ADDRESSED_ATTENTION_PRIMITIVE_CREDIT_GATE.** September 13, 2026. The first release execution passes **18 tests, zero failed or ignored**, using actual Rust circuit, exact object and offline credit primitives. This implements the bounded gate in the [reviewed specification](native_geometric_addressed_attention_spec_973.md); it does not implement the complete eight-phase learned model. Retain `15baec48`; no model promotion or language fit.

## Implemented path and actual checks

The isolated [addressed_attention module](../crates/uor-r4-core/src/native_geometric/addressed_attention/mod.rs) exposes four components. Existing native dispatch and old source-bound shared-core loaders are unchanged; the previous native module only gains the new module declaration.

| Component | Executed evidence |
|---|---|
| Fixed shared LUT4 circuit | 1024 inputs through 256/64/16/9 widths, 345 gates. Every input changes a parity-circuit output; an independent deterministic interpreter matches compiled lowering. Actual per-invocation Bernoulli callbacks revisit shared parameter rows and replay exact event keys. Structural reach is not learned sensitivity. |
| Head lowering and primitive wire format | All 512 contexts use root/symbol modes and full legal extent/control preference orders. Tests exercise every single legal length, fallback when the preferred action is unavailable, empty-support errors, numeric ties including signed zero, unknown domains, duplicate preferences, truncation and trailing bytes. The primitive payload is 49,022 bytes plus an eight-byte format header. It is not a normal model artifact. |
| Causal feature packing | The 1024-bit layout preserves zero padding, hides A/B before selection and in KEY, masks extent before its choice, and hides an unacknowledged active lease during QUERY_A. Numeric/result/pending flags are explicit. A future full-model caller still must supply committed state, genuine geometric signatures and causal availability. |
| Exact objects and publication | 256 occurrences, owned 64-byte spans, eight committed results and complete epoch/sequence/turn references. Tests exercise duplicate text with different identities, overwritten sources with retained leases, invalid extents, ordered/aliased scalar operations, `i64::MIN`, overflow, dependent result reuse and the two-publication cap. |
| Offer and snapshot protocol | Wrong, consumed, canceled and reset offer IDs reject without mutation. Matching and mismatching observations, ordinary and provisional lease cancellation, partial/EOS/Clear/turn cancellation, pending and ready snapshots and final-byte KEY publication execute directly. A saturated 256-occurrence/eight-result/pending snapshot roundtrips below 64 KiB. Corrupt and structurally inconsistent retained payloads/derivations reject. |
| Offline credit | Per-invocation counter-keyed Bernoulli/categorical decisions, masked probabilities, mean conditional CE/direct derivatives and serial leave-one-out accumulation. Changed/nonfinite parameters and incomplete/excess particle batches reject. Four-particle accumulation matches an independently written explicit leave-one-out calculation. |

No source byte replaces the offered emission symbol. Typed scalar decoding applies only to an already selected span; it does not recognize requests or select an answer family. Exact signed H4 product/inverse tables are reused in the causal fixture. No hash bits become attention distances. Hamming remains the specification's unselected comparator; its full geometry certificate and matched model comparison have not run.

## Object-backed causal credit result

Two actual ObjectSession records contain A/B with canonical I/−I keys. Selection reads their two stored key lanes through exact signed H4 compatibility, verifies their complete occurrence identities, and reacquires the chosen lease. The first hard decision changes both the selected object and which parameter row the second decision uses. All four paths, their deterministic shared-row exports and all sixteen ordered two-particle combinations are checked.

| Probabilities | Expected loss, matched by enumeration | Logit gradient p | Logit gradient q |
|---|---:|---:|---:|
| p=.2, q=.7 | .78 | −.144 | +.042 |
| p=.65, q=.25 | .285 | −.102375 | +.121875 |

Analytic and probability-weighted score gradients agree within 1e−14. Centered finite differences agree within 1e−7. The explicit negative control that freezes the second row loses q's gradient; counting only the first visit also gives the wrong p gradient. These are finite causal checks of the declared stochastic objective, not measured learning or semantic transfer.

The late-emission fixture uses a real owned `1x` lease. The offered symbol is followed by actual observation of `1`; acknowledgment or cancellation and KEY commit determine the subsequent fixed loss. At p=.2, the normalized direct derivative is −.4 and the future offered-symbol score is −.12, totaling −.52. Omitting future credit fails the fixture. This checks a downstream consequence of emission using the actual protocol; the following loss is deliberately a fixed analytic fixture, not a language-model score.

## Validation, resources and preservation

Executed once against the frozen source:

```text
cargo test --offline --release -p uor-r4-core --lib addressed_attention -- --test-threads=1 --nocapture
18 passed; 0 failed; 0 ignored; 538 filtered out
```

Compilation plus tests charged **101,708 / 240,000 ms**; the test runner reports 0.07 seconds after compilation. This is test timing, not serving latency. Peak sampled process-tree RSS was 2,571,583,488 bytes, within 4 GiB. The shared cumulative ledger is **122,749,331 / 132,950,000 ms**; parent cycle **2,998,758 / 3,200,000 ms**. No allowance extension, paid compute or cleanup occurred. The recorded 96 MiB storage limit and 128 MiB margin include preserving a separate 16,228,688-byte copy of the tested executable; final delivery storage accounting appends locally.

All **21 prior sealed roots / 733 files** verify unchanged. Of 141 prior bound source files, 140 are byte-identical; the sole change is the new isolated module declaration in `native_geometric/mod.rs`. Both original checkouts retain their heads and dirty status; retained `15baec48` verifies unchanged. Previous failed candidates and memory repair/V3–V7 remain parked. No old model campaign was repeated.

The [tracked evidence](evidence/native_geometric_addressed_attention_primitives_973.json) binds source files, executable, actual log, independent review, projection and gate receipt. Local notes and receipts remain in the established handoff `shared-core-first-step/addressed-attention-primitives-1/`. PR/queue acknowledgements perform no tests; the local release command supplies the executed evidence.

## Explicit limits and next work

The complete eight-phase model, general candidate scanner, automatic canonical hemisphere/zeta feature construction, normal training-bound artifact envelope and corpus learner are **not implemented** by this gate. General prose/Rust generation, full-path allocations, deterministic-export quality and latency/energy remain **NOT_RUN**. Primitive export parity does not establish parity of the future complete model.

SerialLoo owns the three specified sufficient-statistic arrays but accepts materialized dense path-score/direct-gradient slices. A full caller may therefore require two additional parameter-sized scratch arrays unless event replay is streamed. At maximum dimensions this is six f64 arrays including parameters, 31,930,368 bytes, before tapes and other state; do not claim the four-array theoretical tape implementation has already been realized. The 512 MiB planned training-work reservation remains a projection.

Session epochs are a host-assigned non-reused namespace. Retained source content is checked on lease use/restore, but entirely evicted owned content cannot independently detect a host reusing the same namespace. The snapshot checksum detects corruption, not an authenticated author. These are explicit interface trust boundaries, not language evidence.

**Next:** Connect the tested primitives into the specified eight-phase forward/observe schedule with exact geometry/token/zeta bindings and a separately versioned normal artifact envelope. Check complete deterministic interpreter/export and snapshot continuation traces. Then execute one frozen-parameter four-particle forward/backward dry-run, measuring event counts, scratch/tape memory and complete cost before selecting any training dose. No SGD update, language fit, parameter sweep or fresh qualification draw follows automatically. Proposed complete local allowance: 180,000 ms (100,000 build/checks, 30,000 dry-run, 50,000 correction), two build threads/one model process, 4 GiB RAM, 128 MiB new storage and 128 MiB stop margin; refresh and record before use. Source-separated language/coding acceptance must be frozen before a later fit.
