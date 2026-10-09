# Branch archive

Work that existed only on a branch and was not in `main` when branches were
unified into `main` on 9 October 2026 (owner direction: `origin/main` is the
single source of truth). Each patch is `git diff <merge-base> <tip>` of one
branch, so it holds exactly that branch's own changes. The branches were then
deleted; the tip SHAs below stay valid while GitHub retains the objects.

Most of these are superseded experiments, review copies or merge-train
attempts whose ideas later landed in another form. "Lines not in main" counts
added lines (12+ characters) that appear nowhere in `main` at archive time; a
high count means unique material, not necessarily live code.

To revive one: `git apply --3way docs/history/branch-archive/<file>.patch` in a
fresh worktree from `main`, fix the conflicts, and land it through a PR.

| Branch | Last commit | Commits | Lines not in main | Tip | Merge-base | Last subject |
|---|---|---|---|---|---|---|
| [`codex/eff-fused-read-kernels`](codex_eff-fused-read-kernels.patch) | 2026-10-05 | 2 | 0.33 of 332 | `16f3e0b137` | `578c3073c8` | Coalesce the read's age, key-self and lift reductions; skip the causal test off the diagonal tiles (References |
| [`codex/step2-parity-bench`](codex_step2-parity-bench.patch) | 2026-10-05 | 3 | 0.83 of 331 | `0bcbf67d68` | `578c3073c8` | mqar-bench decide: count only full-budget seeds and refuse mixed training configs |
| [`codex/tiered-eval`](codex_tiered-eval.patch) | 2026-10-05 | 2 | 0.96 of 125 | `f3737c2830` | `86e85291f1` | Tiered eval: fix #1724 review blockers (row checks, constant controls, leak check, text-only tier K) (Referenc |
| [`codex/tiered-eval-rescore`](codex_tiered-eval-rescore.patch) | 2026-10-05 | 3 | 0.96 of 125 | `896f14037e` | `86e85291f1` | Re-score the chat models on the #1724 tiers: tier C does not beat the 'I'm not sure' constant (References #820 |
| [`codex/tiered-eval-v2`](codex_tiered-eval-v2.patch) | 2026-10-05 | 4 | 1.00 of 22 | `b04baa473c` | `86e85291f1` | Tiered eval v2: close the #1734 review gaps in exact and abstain_exact (References #820) |
| [`pr1724`](pr1724.patch) | 2026-10-05 | 2 | 0.96 of 125 | `f3737c2830` | `86e85291f1` | Tiered eval: fix #1724 review blockers (row checks, constant controls, leak check, text-only tier K) (Referenc |
| [`pr1733`](pr1733.patch) | 2026-10-05 | 3 | 0.96 of 125 | `896f14037e` | `86e85291f1` | Re-score the chat models on the #1724 tiers: tier C does not beat the 'I'm not sure' constant (References #820 |
| [`pr1742-review`](pr1742-review.patch) | 2026-10-05 | 2 | 1.00 of 0 | `0984750464` | `71deab50ac` | Point the parallel session runner at the pod's evaluation build |
| [`review-1717`](review-1717.patch) | 2026-10-05 | 1 | 0.91 of 23 | `4b57dd1932` | `578c3073c8` | Trainer efficiency: multi-block bit-identical gradient-norm reduction on CUDA (References #820) |
| [`tev2-fix`](tev2-fix.patch) | 2026-10-05 | 4 | 1.00 of 22 | `b04baa473c` | `86e85291f1` | Tiered eval v2: close the #1734 review gaps in exact and abstain_exact (References #820) |
| [`tiered-eval-fix`](tiered-eval-fix.patch) | 2026-10-05 | 2 | 0.96 of 125 | `f3737c2830` | `86e85291f1` | Tiered eval: fix #1724 review blockers (row checks, constant controls, leak check, text-only tier K) (Referenc |
| [`codex/key-shift-production-stacked`](codex_key-shift-production-stacked.patch) | 2026-10-04 | 3 | 0.76 of 139 | `a22a9450d2` | `2e458888bb` | Save and load the read key shift; add key_shift= to train and dialogue-train (References #820) |
| [`codex/mqar-fixes-stacked`](codex_mqar-fixes-stacked.patch) | 2026-10-04 | 4 | 0.92 of 97 | `30502cd783` | `de3fa0e868` | Add the MQAR read probe, the age-slope spread and the j-turned previous-token key channel (References #820) |
| [`pr1691`](pr1691.patch) | 2026-10-04 | 2 | 0.98 of 12 | `e2309fae45` | `14fc7e5a64` | D11 serving: default the stack engine to one thread |
| [`pr1701`](pr1701.patch) | 2026-10-04 | 1 | 0.79 of 75 | `4793bb1ff6` | `2e458888bb` | Add the MQAR read probe, the age-slope spread and the j-turned previous-token key channel (References #820) |
| [`pr1704`](pr1704.patch) | 2026-10-04 | 1 | 0.68 of 70 | `dec3c096e4` | `84715ed2ff` | Save and load the read key shift; add key_shift= to train and dialogue-train (References #820) |
| [`codex/ci/farm/cuda-build`](codex_ci_farm_cuda-build.patch) | 2026-10-03 | 3 | 0.92 of 214 | `49d74078b7` | `95afc96aeb` | CI evidence: compile the CUDA stack kernels on a hosted runner |
| [`codex/ci/farm/cuda-rope-build`](codex_ci_farm_cuda-rope-build.patch) | 2026-10-03 | 6 | 0.89 of 315 | `4fa5e41e4a` | `a621514c21` | CI evidence: compile the CUDA RoPE read kernels on a hosted runner |
| [`codex/cuda-require-flag`](codex_cuda-require-flag.patch) | 2026-10-03 | 3 | 0.94 of 164 | `e92108a344` | `95afc96aeb` | CUDA parity tests: UOR_REQUIRE_CUDA=1 fails on a missing device; skips print SKIPPED |
| [`codex/cuda-rope`](codex_cuda-rope.patch) | 2026-10-03 | 5 | 0.90 of 268 | `1118f310cc` | `a621514c21` | Run RoPE reads on CUDA: rotate queries and keys on the device |
| [`codex/cuda-stack-ops`](codex_cuda-stack-ops.patch) | 2026-10-03 | 2 | 0.93 of 167 | `28f50a4e4b` | `95afc96aeb` | Add a weighted cross-entropy CUDA parity test |
| [`codex/track-b-conversion`](codex_track-b-conversion.patch) | 2026-10-03 | 37 | 0.14 of 9161 | `88c1f5f2d7` | `b88ba1666d` | Merge current main into Track B (current-state split, data-parallel/parquet manifest) |
| [`codex/d19-memory-paraphrases`](codex_d19-memory-paraphrases.patch) | 2026-10-02 | 1 | 0.20 of 24 | `0ab5ffc80a` | `0aa4a50c92` | teacher-paraphrase set=memory: wordings of v2 copy and MQAR training templates |
| [`codex/log-recall-reload`](codex_log-recall-reload.patch) | 2026-10-02 | 6 | 1.00 of 0 | `d539770ca5` | `4caf1e54e2` | Grounded session: a saved session records its log recall and reloads only with it |
| [`codex/metal-backward-recurrence-read`](codex_metal-backward-recurrence-read.patch) | 2026-10-02 | 1 | 0.85 of 25 | `9c2f5069af` | `0aa4a50c92` | Metal training runs the stack: host fallback for uncovered forwards; prepare-text-corpus tool |
| [`codex/compiler-combined`](codex_compiler-combined.patch) | 2026-10-01 | 8 | 0.96 of 17 | `b91a3cb049` | `3144a383c0` | relation_mode=combined_relation: the combined head names relations, the table keeps acts |
| [`codex/log-sieve-e3`](codex_log-sieve-e3.patch) | 2026-10-01 | 8 | 0.98 of 31 | `014fcbcfba` | `e2bbe26866` | recall=route: route_trunk= appends a frozen trunk's standardized features to the word table |
| [`exp/combined-1591-1593`](exp_combined-1591-1593.patch) | 2026-10-01 | 19 | 0.98 of 27 | `ea183faf7b` | `86188622a9` | Merge commit 'bdd023512dfaa1945e1b97a775557b36a88b1d93' into exp/combined-1591-1593 |
| [`exp/protocol-v2-routes`](exp_protocol-v2-routes.patch) | 2026-10-01 | 10 | 0.48 of 257 | `6ae1434dae` | `720cd4b727` | Merge branch 'codex/dialogue-protocol-v2' into exp/protocol-v2-routes |
| [`pr-1591-review`](pr-1591-review.patch) | 2026-10-01 | 4 | 0.97 of 7 | `a71771ce42` | `151ec10bb4` | Merge remote-tracking branch 'origin/main' into codex/dialogue-protocol-v2 |
| [`codex/s14-snap-safety`](codex_s14-snap-safety.patch) | 2026-09-30 | 3 | 0.78 of 52 | `9efa5f6ddd` | `ba266ad44f` | cargo fmt (rustfmt only) |
| [`lab/opencode/1548-adoptable`](lab_opencode_1548-adoptable.patch) | 2026-09-30 | 40 | 0.78 of 1825 | `96cd508a16` | `3b7c2d5414` | resolve: add missing select/pointer to two main-side StackConfig fixtures (#1548) |
| [`codex/merge-train-20260930`](codex_merge-train-20260930.patch) | 2026-09-30 | 42 | 0.97 of 232 | `fae9476331` | `d563b02f32` | merge-train: #1519 cb3c7f3f |
| [`lab/claude/a1-flock-pointer`](lab_claude_a1-flock-pointer.patch) | 2026-09-29 | 1 | 0.30 of 1064 | `14f026b2f6` | `e65aa96048` | A1: flock selection in the fused read and a pointer-copy head for the geometric stack |
| [`lab/claude/a1-mworld-cells`](lab_claude_a1-mworld-cells.patch) | 2026-09-29 | 7 | 0.79 of 1240 | `76c8dac21e` | `3bc296cafa` | run_rules: collect the replies into a vector instead of chaining an iterator over a const |
| [`lab/claude/a1-pointer-fixes`](lab_claude_a1-pointer-fixes.patch) | 2026-09-29 | 5 | 0.63 of 1835 | `b806fb2db9` | `3bc296cafa` | A1 pointer fixes: one flock selector, the pointer's own score and selection, no copy floor (#1511) |
| [`lab/claude/a1-retrieval`](lab_claude_a1-retrieval.patch) | 2026-09-29 | 4 | 0.69 of 1140 | `cb32e73c8b` | `3bc296cafa` | Merge remote-tracking branch 'origin/lab/claude/m-world-v2' into lab/claude/a1-retrieval |
| [`lab/claude/d15-d16-reconcile`](lab_claude_d15-d16-reconcile.patch) | 2026-09-29 | 1 | 0.00 of 50 | `ec123a66fd` | `b3c32170d0` | D17: post-merge reconciliation of D15 and D16 |
| [`lab/claude/integration`](lab_claude_integration.patch) | 2026-09-29 | 2 | 0.48 of 263 | `a6731e3912` | `05e766d22e` | stack_integration: gated on-policy registers, development scoring and greedy replies with the store in the loo |
| [`lab/claude/m-world-v2`](lab_claude_m-world-v2.patch) | 2026-09-29 | 1 | 0.97 of 76 | `c5b8417399` | `e65aa96048` | M-world v2: open value pools, MQAR and copy episodes, the #1516 oracle fixes (A1) |
| [`director/i1-merge`](director_i1-merge.patch) | 2026-09-28 | 4 | 0.98 of 27 | `7d60fad16b` | `8e0840b5f5` | I1: refuse to checkpoint a model in served (QAT) mode |
| [`director/merge-canonical-address-routing`](director_merge-canonical-address-routing.patch) | 2026-09-28 | 16 | 0.16 of 14614 | `45e3871c28` | `f85969e0ff` | Merge origin/main (f85969e0) into director/merge-canonical-address-routing |
| [`director/qat-merge`](director_qat-merge.patch) | 2026-09-28 | 4 | 0.32 of 783 | `f4cd5c51c1` | `2a7a0dec33` | Merge remote-tracking branch 'origin/main' into director/qat-merge |
| [`director/rerun-1468`](director_rerun-1468.patch) | 2026-09-28 | 6 | 0.97 of 65 | `e45bd6c43b` | `e4a15d1920` | docs(d4): add online orthogonal transform derivation and shared interface proposal (#973, #820) |
| [`director/resolve-1452`](director_resolve-1452.patch) | 2026-09-28 | 22 | 0.81 of 404 | `20e4cf0bbb` | `8e0840b5f5` | WASM docs: the module build needs the rlib and cdylib crate types together |
| [`director/resolve-1458`](director_resolve-1458.patch) | 2026-09-28 | 12 | 0.16 of 825 | `fc87c9521b` | `8e0840b5f5` | Merge origin/main (S1.1) into #1458 for the Lab 1 resolution |
| [`director/s1-1-merge`](director_s1-1-merge.patch) | 2026-09-28 | 9 | 0.94 of 154 | `c85f9e58b2` | `2a7a0dec33` | Merge remote-tracking branch 'origin/main' into director/s1-1-merge |
| [`lab/anti-gravity/capability-api-wasm-m1-cost`](lab_anti-gravity_capability-api-wasm-m1-cost.patch) | 2026-09-28 | 22 | 0.81 of 404 | `20e4cf0bbb` | `8e0840b5f5` | WASM docs: the module build needs the rlib and cdylib crate types together |
| [`lab/claude/integrate-1433`](lab_claude_integrate-1433.patch) | 2026-09-28 | 24 | 1.00 of 5 | `0ffe291fe3` | `fa9faf1629` | Merge main into #1433 (Lab 1 now executes it by owner transfer) |
| [`lab/codex/merge-review-stack-inputs-20260928`](lab_codex_merge-review-stack-inputs-20260928.patch) | 2026-09-28 | 12 | 0.18 of 32 | `33278454f5` | `8e0840b5f5` | Merge remote-tracking branch 'origin/main' into review/pr1467-input-safety-20260928 |
| [`pr-1472`](pr-1472.patch) | 2026-09-28 | 4 | 0.98 of 27 | `7d60fad16b` | `8e0840b5f5` | I1: refuse to checkpoint a model in served (QAT) mode |
| [`pr-1486`](pr-1486.patch) | 2026-09-28 | 2 | 0.60 of 2 | `b6c6cccd00` | `e3f9117909` | Memory port: the prime router is the address form (owner direction) |
| [`pr-1487`](pr-1487.patch) | 2026-09-28 | 5 | 0.98 of 24 | `020f974d4e` | `66235490a1` | Test that semiprime experts are exactly unordered pairs |
| [`claude/cloud-handoff-20260928`](claude_cloud-handoff-20260928.patch) | 2026-09-28 | 1 | 0.47 of 46955 | `16c77c973b` | `0994d114a2` | Cloud lab track handoff: HANDOFF.md and the sandbox's unique text work |
| [`lab/claude/resolve-1458-codec-cleanup`](lab_claude_resolve-1458-codec-cleanup.patch) | 2026-09-28 | 12 | 0.16 of 825 | `fc87c9521b` | `8e0840b5f5` | Merge origin/main (S1.1) into #1458 for the Lab 1 resolution |
| [`explore-matmul-fixed-a`](explore-matmul-fixed-a.patch) | 2026-08-20 | 1 | 0.00 of 7 | `260440b6ca` | `70586aa0ee` | explore: track WIP fork matmul (fixed-A #37) instead of the pinned rev |
| [`i639-3a-sentencepiece-unigram-core`](i639-3a-sentencepiece-unigram-core.patch) | 2026-08-14 | 1 | 0.97 of 17 | `014094d9f7` | `b6588b39c0` | core: SentencePiece ModelProto parser + Unigram Viterbi core (#639-3a) |
| [`i446-m2-latent-right`](i446-m2-latent-right.patch) | 2026-08-06 | 4 | 0.56 of 305 | `ce088264bc` | `4957bbd2a7` | Merge pull request #455 from UOR-Foundation/i446-m3-hard-select |
| [`graph-emission-shrinkage`](graph-emission-shrinkage.patch) | 2026-08-02 | 5 | 0.92 of 9 | `8ca83a2df0` | `42f6926fd4` | certify: test Witten-Bell shrinkage; refutes the sparsity explanation |
| [`graph-emissions-followup`](graph-emissions-followup.patch) | 2026-08-02 | 3 | 0.99 of 1 | `a501d012a5` | `35f83a36c5` | certify: measure emission selection against likelihood |

## Local-only tips

These local branch copies had commits that differed from their GitHub branch
(work never pushed, or a rebased copy). Each patch is the local tip's
`git diff <merge-base> <tip>`.

| Branch (local copy) | Last commit | Commits | Tip | Merge-base | Last subject |
|---|---|---|---|---|---|
| [`codex/step4-script-markers`](codex_step4-script-markers.local.patch) | 2026-10-05 | 2 | `fb6c449cec` | `86e85291f1` | Step 4 scripts: failure markers and loud control mismatch (References #820) |
| [`codex/step5-knowledge-corpus`](codex_step5-knowledge-corpus.local.patch) | 2026-10-05 | 1 | `076d0559af` | `0df3525935` | Step 5: decontaminated knowledge corpus tool, panel-clean subset and the 29M knowledge A/B pod script (Referen |
| [`codex/tiered-eval`](codex_tiered-eval.local.patch) | 2026-10-05 | 1 | `3564f9c191` | `86e85291f1` | Tiered chat evaluation: conversational panel, clean open panel, per-tier scoring (References #820) |
| [`codex/tiered-eval-v2`](codex_tiered-eval-v2.local.patch) | 2026-10-05 | 3 | `1b0390fcf3` | `86e85291f1` | Tiered eval v2: exact memory check, per-category constant controls (References #820) |
| [`codex/cuda-source-delivery-20261003`](codex_cuda-source-delivery-20261003.local.patch) | 2026-10-03 | 4 | `c17411b88d` | `a621514c21` | Bind retained CUDA log files and mark device-source identity pending |
| [`codex/compiler-combined`](codex_compiler-combined.local.patch) | 2026-10-01 | 9 | `b1219bd06a` | `a74f368606` | Merge remote-tracking branch 'origin/main' into codex/compiler-combined |
| [`codex/track-b-conversion`](codex_track-b-conversion.local.patch) | 2026-09-30 | 34 | `2cec249489` | `3b7c2d5414` | Refresh lab routing and supersede stale blanket build holds |
| [`lab/anti-gravity/capability-api-wasm-m1-cost`](lab_anti-gravity_capability-api-wasm-m1-cost.local.patch) | 2026-09-28 | 15 | `63a74d56e4` | `8c4e45de88` | Merge origin/main into lab/anti-gravity/capability-api-wasm-m1-cost |
| [`lab/claude/i1-stack-store`](lab_claude_i1-stack-store.local.patch) | 2026-09-28 | 2 | `b68666223e` | `ccd19cb63e` | Add the I1 sealed stack inference checkpoint with its session store |
| [`lab/claude/s1-integer-port`](lab_claude_s1-integer-port.local.patch) | 2026-09-28 | 8 | `724f8d64e4` | `df8411b17b` | Explain the boxed layer vectors to clippy and tidy two test lints |
| [`lab/claude/s1-qat`](lab_claude_s1-qat.local.patch) | 2026-09-28 | 3 | `e1038ea641` | `df8411b17b` | S1 QAT: init= and qat= in geometric-stack train |
| [`codex/dialogue-child-code-choice-20260927`](codex_dialogue-child-code-choice-20260927.local.patch) | 2026-09-27 | 18 | `88b4d825a5` | `0ac3df4679` | fix: supply missing end_weight field in radial-startup example and clean campaign warnings |
| [`codex/dialogue-child-export-20260927`](codex_dialogue-child-export-20260927.local.patch) | 2026-09-27 | 11 | `e5efafb8fa` | `0ac3df4679` | Record selected dialogue child native retention result |
| [`codex/native-dialogue576-observation-20260927`](codex_native-dialogue576-observation-20260927.local.patch) | 2026-09-27 | 9 | `0607b43be6` | `0ac3df4679` | Record packed runtime behavior and storage comparison |
| [`lab/opencode/read-localization`](lab_opencode_read-localization.local.patch) | 2026-09-27 | 6 | `6b3e0b9973` | `dccef74b4b` | Record the oracle read re-rank: ranking is sufficient for the distractor class |
| [`r5/ledger-register-dormant`](r5_ledger-register-dormant.local.patch) | 2026-08-10 | 2 | `f969fd8eb2` | `4c983fd268` | chore(#515): drop region-store-dormant — region_store is wired, not dormant |
| [`feat/510-r5-t1b-scoring`](feat_510-r5-t1b-scoring.local.patch) | 2026-08-08 | 2 | `796d83638e` | `27303528df` | feat(#510): R5 1b(iii) — inference_contract to the sanctioned surface |
| [`i446-m2-latent-right`](i446-m2-latent-right.local.patch) | 2026-08-06 | 1 | `b294ceb0ec` | `4957bbd2a7` | feat(certify): #446 M2 — latent right-context mixture (causally legitimate two-sided structure) |
| [`i380-row-distribution`](i380-row-distribution.local.patch) | 2026-08-03 | 2 | `0e67435d7e` | `73ab76b369` | fix(lint): remove the dead context_prediction shim (#383 CI) |
