# Learned variable-length grounded text — #973

**PASS_GROUNDED_VARIABLE_TEXT.** The first fit generates all 64 development spans exactly, including EOS, and both answers in all 32 changed-source pairs. All 48 development answers longer than the longest training answer pass. This establishes a learned byte/EOS writer and cursor-advance decision over the retained geometric reader. It is grounded copying of supplied spans, not general prose or completion of H1. Retain `15baec48`; no model promotion.

## Implemented path and learning boundary

The [text_attention module](../crates/uor-r4-core/src/native_geometric/text_attention/mod.rs) extends the learned one-read boundary to variable-length output. For every emitted symbol, it recomputes geometric source selection, reads the selected span at the current cursor, executes nine learned Boolean tables to predict a byte or EOS, and applies a learned hold/advance decision after the actual non-EOS event. EOS ends generation; exhausting the 33-symbol resource bound reports failure. Every output byte passes through the learned writer; there is no direct source-to-output copy override or target argument at inference.

The exact two-byte query, record bindings and span boundaries remain supplied. Query encoding stays fixed and the query does not change within this text loop. State is an integer cursor; its decision sees source presence and whether the emitted symbol is EOS, not the identity of the emitted byte. Incrementing the cursor by one is a declared typed operation. This result does not establish content-conditioned semantic state, raw-language query interpretation or arbitrary R4 transport.

Training is one alternating output-directed fit: byte/EOS likelihood at teacher positions, next-symbol utility for hold versus advance, and complete decoded-span utility for the warm-started reader. The latter derives useful source labels from actual forced-span predictions. Teacher positions enter offline training only; evaluation uses closed-loop generation. The update is neither end-to-end differentiation through hard trajectories nor joint learning of query encoding. The reader is trainable, but its exported tables did not change in this fit; it already selected every correct source before training. Eight writer tables and one advance entry changed. The final advance table is `[0, 0, 0, 1]`: advance when a byte is present and the actual output is non-EOS. Some table rows remain unexercised; behavior outside the declared character/task domain is unqualified.

The artifact retains the prior adaptive reader/update/action artifact unchanged and identifies the active text reader separately. The old adaptive mode still runs its original byte/link curriculum. It is **not integrated into this text loop**: multi-hop variable-length text and a common recurrent query/read/update/emission learner remain unfinished. In particular, the old terminal/link byte partition cannot simply be assumed to classify arbitrary text correctly.

Serving uses the retained finite geometric read and Boolean/integer/table operations, without matrix products, a transformer or a response provider. Repeated source selection, validation and bounded allocations remain. Optimized throughput, allocation and energy qualification were not run.

## Frozen environment and actual results

The corpus has 256 training pairs/512 rows and 32 development pairs/64 rows. Each pair fixes the query and record keys while swapping two source spans, requiring a different answer. All exact span strings are disjoint across splits; development characters occur in training answers. Source slots are balanced: 128 training and 16 development answers per slot. The longest training answer is 18 bytes; development includes 25–32-byte answers, as well as short outputs and punctuation. Supplied source availability was checked before fitting: 576/576 actual geometric selections were correct.

Acceptance was frozen before model loading: at least 61/64 exact development outputs, 31/32 complete pairs, 95% of the longer-than-training group, improvement of at least 32 over initialization, zero exact answers for every disabled/reversed control, and the declared retained controls. No threshold, development row or training recipe changed after the fit.

| Artifact or intervention | Development exact, including EOS |
| --- | ---: |
| Untrained text writer/state | 0/64 |
| Learned Full | 64/64 |
| Read disabled | 0/64 |
| State feedback disabled | 0/64 |
| Selected payload disabled | 0/64 |
| Query reversed | 0/64 |
| EOS disabled | 0/64 |

Training is 512/512 exact; development is 1,520/1,520 aligned target symbols including EOS. Disabling feedback repeats the first byte until exhaustion; disabling EOS retains all 1,456 development text bytes at their expected positions but fails to terminate correctly. Thus the zero exact counts have different causal meanings, recorded in the raw traces.

One actual changed-source pair keeps query `[70, 71]` fixed. Full outputs `the quiet young owl jumps.` followed by EOS; after the span swap, it outputs `the bright otter quietly jumps!!` followed by EOS. Both strings are absent from training. These are new copied strings, not invented prose.

The 256-epoch fit at learning rate 0.1 completed its dose. Epoch 32 was the earliest best hard training-output checkpoint; later epochs were still executed. Internal fitting took 4,980 ms; the full fit/report/control command took 7,455 ms. All 576 new Full trajectories reproduce after export/reload. An exact optimizer-resume checkpoint is not claimed. Final independent holdout: **NOT_RUN**.

The new text loop reproduces all 8,256 prior one-read answers using the declared one-byte span adapter. The separately retained historical adaptive mode reproduces all 2,240 mixed-depth examples. These two checks do not imply that the text loop itself performs adaptive multi-hop reads.

Candidate SHA256: `22b1df1c6b5721ea71d0bad40ab968434b63397c7819213aec046c7def6af63e`. Historical adaptive parent SHA256: `f0e1f2c35a68e6228777c1a662a2e6b00a398957444f29fc3e4f2f9b8d473d54`.

## Checks, preservation and resources

Four new focused tests and seventeen retained relational/dependent/adaptive tests pass. Checks cover data separation and causal pairs, character/length coverage, artifact round-trip and malformed fields, bounded state behavior, and rejection of unsupported reader topology before training parameter indexing. The explicit actual fit/report test runs separately. The release build reports an unused training import and two existing fixture-constant warnings; no check failed. Formatting, claim wording and whitespace checks accompany delivery.

All 36 prior sealed report roots/1,070 files, bound prior source/executable, both original dirty checkouts and the normal model verify unchanged. The new attempt was exclusively claimed, sealed and Rust-verified; postflight checks full file sets, sizes and SHA256 identities. [Source-bound evidence](evidence/native_geometric_text_attention_973.json) includes acceptance, data validation, fit, controls, samples and preservation. The experiment does not run or replace the old memory-repair qualification.

Projection: 1,800,000 ms model/build/checks/corrections; 180,000 ms engineering commands; two hours wall; 6 GiB RAM; 512 MiB new storage; 128 MiB stop margin; two build threads and one model process. No allowance extension was needed. Actual model/build charge is 130,613 ms; shared ledger 124,987,758/132,950,000 ms and parent 5,237,185/7,710,000 ms. Sampled peak process-tree RSS was 2,845,540,352 bytes during compilation. No paid compute, cleanup or V3–V7 replay occurred.

The final storage, command, delivery and restart receipts live at `/Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/text-attention-1/`. The sealed raw report is its `attempt-1/` directory. These local paths remain authoritative; artifacts have not been moved into the repository.

## Remaining H1 work

Connect dependent source selection and variable-length emission in one recurrent path. The next task should require a short grounded answer assembled from more than one selected source, with an earlier selected/emitted result affecting the later query and all downstream decisions recomputed. Inspect the representation and action-class collisions before freezing the task; do not concatenate the separate APIs and call that learned integration. Use output-directed learning, exact changed-early-source and feedback/read-disabled interventions, and preserve both this text result and the prior typed read controls through the combined path. Keep supplied bindings, fixed operators and auxiliary supervision explicit. Common recurrent learning, raw-language inputs, novel prose and final replacement qualification remain responsibilities of #973; #964 serving constraints remain unchanged.
