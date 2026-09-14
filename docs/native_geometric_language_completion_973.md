# Typed language completion — #973

**PASS_TYPED_LANGUAGE_COMPLETION.** The UOR-R4 Geometric Language Model now distinguishes a finished answer from a required continuation that its current routes cannot resolve in this bounded language experiment. [Evidence](evidence/native_geometric_language_completion_973.json). Retain normal model15baec48; no promotion.

## Demonstrated failure and correction

The unchanged predecessor emitted an intermediate word plus EOS on all 1,024 unresolved rows in a separately sealed 1,792-row diagnostic; all 768 valid rows passed. Missing compatible continuations and conflicting continuations each contributed 512 failures. This was actual generation, not a source-only prediction.

The old observation merged final completion and unavailable pending continuation into row3. The new [completion runtime](../crates/uor-r4-core/src/native_geometric/dependent_language/completion.rs) adds one pending-clause bit, expands eight policy rows to sixteen and introduces a typed Unresolved action. Both banks start from the retained eight-row table. Actual output-constrained search changes only row11 from Emit to Unresolved. Training finds a conflict between 384 Emit and 512 Unresolved decisions when the pending bit is collapsed, and no conflict with the bit retained. Reader, query updater, encoder, writer and existing transition executor remain unchanged.

The typed terminal result identifies the pending clause and actual routing reason. On these panels, NoCompatibleCandidate and Ambiguous each occur 256 times in development. It emits no intermediate bytes or EOS and is distinct from resource exhaustion. These reasons describe the admitted routing environment; they do not establish global fact absence. This is an internal experimental result type, not a generated explanation or an integrated product abstention.

## Environment, learning and actual output

Rust data preparation creates 256 matched seven-row families: direct valid; two-read valid/missing/conflicting; three-read valid/missing/conflicting. Source-only edits preserve earlier resolvable paths. An independent raw grammar oracle validates the data; the diagnostic and correction use exactly the same frozen corpus. Families split 128/128, giving 896 training and 896 development rows. Name offsets differ, but familiar vocabulary and grammar overlap. Supplied question punctuation remains explicit. Independent final holdout is NOT_RUN.

The [learner](../crates/uor-r4-core/src/native_geometric/dependent_language/completion_learning.rs) searches actual transitions, supervised by final answer/EOS or typed unresolved reason. Source paths, intermediate values and blocked-clause labels are evaluation-only. It warm-starts the scheduler and freezes other parameters; this is not end-to-end learning. Search is bounded to 512 nodes per example and 96 runtime steps.

| Policy/control | Correct valid answers | Correct unresolved results | Total correct |
|---|---:|---:|---:|
| Warm start |384/384|0/512|384/896|
| Fitted Full |384/384|512/512|896/896|
| PendingHidden |384/384|0/512|384/896|
| UnresolvedDisabled |384/384|0/512|384/896|

Training passes 896/896; all 128 development families and 896 resolved-prefix checks pass. All 896 traces reproduce after artifact reload. PendingHidden reproduces the frozen diagnostic output, exhaustion and final clause on all 896 development rows. Malformed action and parent digest are rejected.

## Preservation and validation

The corrected Full policy preserves all 2,816 earlier successful core traces exactly: 1,536 mixed one/two-read and 1,280 three-read examples. Separately, unchanged-parent replay retains every earlier control: 13,248 old outputs; 512 recurrent; 640 ordered; 1,408 language; 128 old-language through relative; 9,984 relative; 9,984 dependent; 19,968 scheduling; 19,968 output-credit; 19,200 depth rows. The latter is legacy replay, not a claim that every old negative control was applied through the new policy.

Optimized offline compilation and 40 focused tests pass (two new and 38 retained). The two ignored reports were explicitly executed: one diagnostic and one fit/evaluation. No retry or old fitting campaign. Queue acknowledgements do not execute these tests. All 49 earlier sealed roots / 1,234 files, predecessor frozen source/executable, both original dirty checkouts and normal model verify unchanged. This step adds 13 diagnostic files and 18 candidate report files, all sealed.

Candidate SHA256 f9e1f5aeef918d0fe6b463cd75986847d551aa8049c9f1c85b1d6d46c55a1e1f. Parent SHA256 62da0bfaddb37452a5ec7eb1aa220fbda7b33c79ef3dbcd4da1dcb11293e7a95; nested parent parameters are byte-identical. Both diagnostic and correction have separate frozen source and binary receipts. Local evidence: /Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/language-completion-1/.

Model/build/diagnostic/fit/controls cost 376916/1800000 ms; parent 8223685/9810000 ms; shared 127974258/132950000 ms. Peak sampled process-tree RSS 3090448384 bytes, under 6 GiB. Projection includes two build threads, one model process, 384 MiB new storage and 128 MiB stop margin. Before work, the parent local storage allowance increased by 256 MiB to 3,623,878,656 bytes under standing authorization; no time extension, paid compute or cleanup. Exact final charges/storage are in resource-final.json.

Inference continues through geometric routing/state and integer/table operators, without a transformer, serving matrix products or provider. Signed H4/Hamming comparisons retain their prior structural role; this adds no metric advantage. One-word output, explicit sequential-reference protocol and authored familiar grammar remain limitations. General prose, arbitrary discourse and energy advantage are unqualified.

**Next:** Extend the existing occurrence reader and recurrent writer to a bounded contiguous multiword answer, beginning with a source audit of candidate admission and span boundaries. Current relative_language candidates each expose one word, so preserve exact occurrence identity separately from the selected span and do not expect more scalar features to reconstruct an erased boundary. Freeze a small raw-language mixed one-word/multiword task with changed active text, unchanged inactive text, cursor/boundary controls and retained one/two/three-read plus unresolved behavior. Learn selection/emission from final text and EOS; no gold span or answer length enters serving. Reuse this completion policy and existing geometry, without another depth ladder or broad audit. Phrase copying will still not qualify general prose.
