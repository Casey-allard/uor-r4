# REPORT: written-state-count-model, claude-415 (three counter fixes before the full-corpus count)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **claude-412's counter let n-grams cross document boundaries** [claude]. The D87 stream marks each record with BOS (id 0) and ends it with EOS (id 1). Before this unit, a context could reach back into the previous record, and BOS was counted as a target. Every count claude-412 published therefore differs from this unit's. Those counts were finite kernel checks, and nothing downstream had read them.
- **The first restored device build was stale** [claude]. In the device negative gate, restoring `ngram_count.cu` from the tarball also restored its old modification time. `build.rs` did not rerun, so the "restored" run still used the sabotaged cubin and failed 3 of 4 tests. After `touch`, the rebuild passed 4 of 4. This was a fault in the gate procedure, not in the counter.
- **Builders ran out of context before finishing** [claude]. The scout and the first builder stopped at the 150k mark, so round 1 and the device half of round 2 were written in the top session. The second builder did the host half of round 2 and handed back.
- **The unit was first named claude-413a** [claude]. `loop pr-check` parses `<lab>-<n>` as an integer, so it was renamed claude-415 before any run.
- **This builder's worktree started at `main`, not at the unit's tip** [claude]. It was fast-forwarded before work began; nothing was rewritten.
- **The fresh review found two refusals untested** [claude]. The cross-check's refusals of a trailing extra context and a missing last context had no test. A new test, `ngram_count_assembler_refuses_an_extra_or_missing_context`, now covers both. With the `next != contexts.len()` check set aside it fails, and 16 others pass; restored, 17 of 17 pass. The review also asked for a guard on `ngram_admitted`'s `k <= j` precondition, so a `debug_assert!` was added. Its verdict was APPROVE-WITH-LOW, at `0b0f6efcf`.

## What changed

- `ngram_count.rs` [claude]:
  - The assembler's cross-check (`check_census`) streams over the triples. It compares each context record and n1..n4 with what the triples imply, without building a second context vector.
  - `plan_partitions` refuses a partition above 2^30 targets.
  - `NGRAM_BOS` and `ngram_admitted`: order k admits target j only when `t[j]` is not BOS and no BOS lies in `t[j-k+1..j-1]`. BOS may sit only in a context's oldest slot, so a record's first n-gram keeps its start marker. The rule drops targets rather than padding them; back-off covers record starts.
- `ngram_count_oracle.rs`: the oracle skips targets that are not admitted [claude].
- `ngram_count.cu` [claude]:
  - A device `ngram_admitted` with the same predicate.
  - `ngram_token_hist` histograms the partition token over the admitted targets of one order.
  - `ngram_select_flags` and `ngram_select_scatter` take the order and select only admitted targets.
- `ngram_count_device.rs` [claude]:
  - The planning histogram runs on the device for each order.
  - The host unigram copy and its edge fields are removed.
- The CUDA oracle test plants BOS every 61 ids [claude].
- Three new host tests [claude]:
  - the plan bound;
  - the document boundaries on a two-record stream;
  - an extra or missing context record, which the cross-check refuses.
- `ngram_key_preserves_order` now separates its records with EOS (1), not 0, because 0 is now a boundary [claude].
- The probe's first timing is relabelled `upload_ms` [claude].
- One notebook row, in `docs/research/notebook/m4-serving.md` [claude].

## Measurements

Wick's RTX 4090 (`compute_89`), on the 15,471,412-token D87 student slice `c404/stream/tokens.u16` [claude].

- The slice holds **4,096 records**, and `t[0]` is BOS [claude]:
  - orders 0 and 1 count 15,467,316 targets;
  - each order above 1 counts exactly 4,096 fewer than the one below, one per record;
  - order 8 counts 15,438,644.
- Order 1 has 3,968 contexts: every token id in use except BOS, which never appears as a target, and EOS, which is only ever followed by BOS [claude].
- Order 5 [claude]:
  - 8,363,157 grams;
  - triple SHA-256 `bf068dad4aae3d28d08d188fb29a5660a7d2c2ce625c36f680e65d5039f8cc26`.
- Time per order, one partition [claude]: 17 ms (order 0) to 782 ms (order 8), that is 891M to 20M targets/s. The card was shared, so these times are not a qualification.

## What the numbers say

- **The GPU and the oracle agree under the boundary rule** [claude]. They match byte for byte on random streams with planted BOS and on the real slice. Two probe runs are identical.
- **The per-order drop is the check that the rule is the one intended** [claude]. A record of length L loses exactly one target per order above 1. That every order loses exactly 4,096 means each of the 4,096 records is longer than 8 tokens, and that the device drops no target the rule keeps.
- The counter is ready for the full-corpus count of the R\* units [claude]. That count has to use this rule, so that no n-gram mixes two conversations.

## Checks run at the head

- G1, laptop [claude]: `cargo test -p uor-r4-discrete --lib ngram_count`, 17 passed at the head (16 before the review's test). The review's own run at `0b0f6efcf` also passed 16 of 16.
- Stub-nvcc [claude]: `cargo check -p uor-r4-discrete --features cuda --all-targets` is clean.
- G2, negative gates on the host [claude]:
  - plan bound reverted to `u32::MAX`: 14 passed, 1 failed;
  - streamed comparison disabled: 14 passed, 1 failed (`refuses_a_corrupted_context_n1`);
  - predicate forced true: 15 passed, 1 failed (the boundary test);
  - each passes when restored.
- G3, wick [claude]: `UOR_NVCC_ARCH=compute_89 cargo +1.97.1 test --release -p uor-r4-discrete --features cuda --lib ngram_count`, 20 passed, 1 ignored.
- G4, wick [claude]: `UOR_NGRAM_STREAM=…/tokens.u16 … ngram_count_cuda_real_stream_matches_the_oracle -- --ignored` passed. `ngram-count-probe tokens.u16 8 4294967295 1000000`, run twice: after timings are stripped, the outputs are `cmp`-identical.
- Device negative gate, wick [claude]:
  - with the device `ngram_admitted` forced true, 3 of 4 CUDA tests fail (`matches_the_oracle`, `max_order_and_short_streams`, `sha_covers_a_planted_single_count_change`);
  - restored and touched, 4 of 4 pass.
- G5 [claude]: `UOR_LAB=claude uor-harness loop pr-check --local` at the committed head: see the PR.

## Skills used

None invoked [claude]. I followed AGENTS.md, the native-space rule (integer only, and the oracle and device share one predicate) and the D87 protected-asset rule: the slice is a copy, read only.

## Still open

- R\* [claude]:
  - continuation counts N1+(•wh);
  - modified-KN discounts;
  - BPB equal to the oracle on a slice;
  - then the sheet, a non-author FIT and the sealed DEV-test run, scored on all targets and on the novel subset.
- The span dedup of claude-414 is claude-416, PR #4051 [claude]. The full-corpus count waits for its split v2.
- In a gate that restores a CUDA source from an archive, `touch` the file before rebuilding (a candidate convention) [claude].

## Plain summary

The project's exact word-sequence counter on the GPU used to let a sequence run from the end of one conversation into the start of the next [claude]. It now stops at each conversation's start marker, on both the GPU and the reference implementation. The two still agree exactly on random test data and on 15 million real tokens [claude]. Two smaller safety fixes landed with it: a memory-saving change to the self-check, and a refusal of oversized work blocks. The counter is ready to count the full training set once the deduplicated split is done [claude].
