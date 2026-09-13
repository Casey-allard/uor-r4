# Learned read-versus-emit control — #973

**PASS_TYPED_ADAPTIVE_READ_EMIT.** The single fit answers all 192 development examples correctly at the correct read depth, including all 32 four-read compositions absent from training. Both members of all 96 changed-source pairs pass. The new adaptive loop also reproduces all 8,256 earlier one-read examples and 2,176 dependent-read examples. This completes the scoped typed read/emit-control step, not H1 as a language model. Retain15baec48; no promotion.

## What changed

The [adaptive_attention module](../crates/uor-r4-core/src/native_geometric/adaptive_attention/mod.rs) wraps the retained [dependent query update](native_geometric_dependent_attention_973.md) and geometric reader/codec in one repeated loop. Each iteration reads using the current query, applies a learned content-conditioned emit decision, and either returns the decoded byte/EOS or updates the query from the selected payload and reads again. Every downstream read is recomputed. A four-read resource cap returns exhaustion with no answer; it never forces a final emission.

Only the 256-entry byte-indexed Boolean action table is trained. The reader, learned query update and byte/EOS codec stay byte-identical. No hop count, expected stopping position, answer or terminal marker crosses the inference interface. Serving uses finite geometric operations and Boolean/integer/table execution, with no matrix contraction or transformer/provider path. Bounded allocations and unused intermediate codec work remain; no optimized latency or energy claim follows.

The curriculum deliberately separates answer bytes a..h from permitted link-transform bytes. Link values exclude zero and values whose retained codec maps to an answer byte. This domain is documented, not inferred prose semantics. The runtime contains no hardcoded membership rule; training learns action entries from actual decoder-answer matches on retained geometric rollouts. Otherwise training continues with the learned query update. The depth and XOR oracle exist only in data validation/evaluation. The update remains lexical-address composition, not an arbitrary learned continuous R4 transport law.

Training uses final-answer-derived action supervision and Bernoulli BCE; it does not differentiate through hard retrieval or jointly train the full model. The best hard training-output checkpoint is selected at epoch1 and every16epochs, earliest tie. All development action-input bytes occur in training. This is transfer to new contexts, initial-query symbols and longer compositions, not to unseen byte-action classes.

## Frozen test and actual result

Training:1,024 paired families/2,048 rows, with read depths1/2/3 represented by1,024/512/512 rows. Development:96 pairs/192 rows, with depths1/2/3/4 represented by96/32/32/32 rows. Each pair changes only the initial record's payload: the short member can answer immediately, while the long member must follow two to four reads. Keys and other records remain fixed. Answers and initial slots are balanced within every depth; split families/contexts and initial query alphabets are disjoint. Derived record-key alphabets may overlap.

Prefit actual execution verifies all2,240 answer depths and all3,968 source accesses. Training covers231 action-input bytes:223 permitted link values and8 answer bytes. Zero development action bytes are unseen. This makes the representation and training coverage adequate for the declared content-classification task before fitting.

The unchanged first-fit gate requires at least183/192 exact answers and correct depths,91/96 complete pairs,95% correct within every depth, improvement over untrained/forced-emission controls, and all prior one-/two-read outputs preserved through both the old paths and new loop. Development was not redrawn and no thresholds changed after fitting.

| Policy/control | Development exact | Correct read depth | Longer-path answers |
| --- | ---: | ---: | ---: |
| Untrained policy | 0/192 | 0/192 | 0/96 |
| Learned Full | 192/192 | 192/192 | 96/96 |
| Always read | 0/192 | 0/192 | 0/96 |
| Always emit immediately | 96/192 | 96/192 | 0/96 |
| Fixed two-read schedule | 56/192 | 32/192 | 32/96 |
| Query update disabled | 96/192 | 96/192 | 0/96 |
| Update payload disabled | 96/192 | 96/192 | 0/96 |
| Read disabled | 0/192 | 0/192 | 0/96 |

Full achieves96/96,32/32,32/32,32/32 answers at depths1,2,3,4 respectively. Training is2,048/2,048. Forced two-read execution gets24 immediate cases correct by default selection and32 true two-read cases, explaining why answer counts alone cannot establish the stopping mechanism. The learned policy's exact stopping depths and disabled controls distinguish that behavior.

The128epoch fit at learning rate0.1 completed its configured dose; epoch16 was the earliest best checkpoint. Eight action entries changed from read to emit, for bytes97..104. There were zero conflicting action labels. Internal fit1.274seconds; complete fit/report/controls/retained replay4.019seconds. All2,240 new Full outputs reproduce exactly after export/reload. An optimizer-state exact-resume checkpoint is not claimed.

Candidate SHA256:f0e1f2c35a68e6228777c1a662a2e6b00a398957444f29fc3e4f2f9b8d473d54. Frozen parent SHA256:dcbca3d77bc7db8a30d962ac85450e821fa92f8d2bf5d8228c8c897f0d6e1d9d. Final independent qualification **NOT_RUN**. Outputs remain a single byte plus EOS; variable read depth is not variable-length language generation.

## Verification, preservation and resources

Three new focused tests and fourteen retained tests pass. They cover data causality/coverage, artifact validation, bounded exhaustion, retained query composition and relational arithmetic. The explicit fit/report entry point executes separately from unit checks. Review added the checkpoint deadline guard and strengthened retention to run the new loop on both prior corpora before the first fit. All35prior sealed roots/1,060files, original dirty checkouts, old model artifacts and source/binaries remain unchanged. The new attempt was exclusively claimed, sealed and Rust-verified; postflight verifies file sets/sizes and SHA256 identities.

Complete projection:30minutes model/build/corrections,3minutes engineering commands,2hours wall,6GiB RAM,512MiB new storage,128MiB margin, two build threads and one model process. Existing cumulative allowances covered the task without extension. Actual charged build/model/tests126,359ms; shared ledger124,857,145/132,950,000ms, parent5,106,572/7,710,000ms. No external spending, cleanup or V3–V7 replay. Final storage and delivery receipts remain at /Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/adaptive-attention-1/.

[Exact evidence](evidence/native_geometric_adaptive_attention_973.json) includes fit, per-depth controls, source/artifact identities and preservation. Sealed raw attempt:/Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/adaptive-attention-1/attempt-1/.

## Next H1 responsibility

The scoped typed construction now combines learned compatibility, dependent query composition and content-conditioned read/emit control in one loop. The next task should connect this loop to short variable-length text: update state after each emitted byte and learn state-conditioned byte/EOS decisions, with the interacting query/update/read/emission parameters trained against a common output objective. Freeze a small short-text development task and context-disabled/changed-source controls, preserve these typed controls, and inspect the actual representation dependencies before fitting. Declare any remaining supplied bindings or auxiliary supervision. General prose and the complete H1 learner remain unqualified; final replacement controls and promotion follow applicable acceptance.
