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

**D21 disposition:** the protected/discrete-constructor/solver line is closed as
negative. Its already-activated historical source and dated records remain on
main; the absolute legal representation, initial diagnostic labels and unpromoted
residual-backend patch below retain otherwise superseded source. Archive recovery
instructions do not authorize resuming that closed line.

To revive one: `git apply --3way docs/history/branch-archive/<file>.patch` in a
fresh worktree from `main`, fix the conflicts, and land it through a PR.

| Branch | Last commit | Commits | Lines not in main | Tip | Merge-base | Last subject |
|---|---|---|---|---|---|---|
| [Saved145 cross-state continuation negative](cross-state-saved145-negative-20261010.patch) | 2026-10-10 | 1 | Not counted | `fdc3dfa8a` | `eb6f1c32d` | Fixed256 additional updates from saved145 score139/512,7 gains/13 losses; REJECTED,accepted145 retained. New saved145 profile retained here rather than activated. Apply to eb6f1c32d in a fresh isolated worktree for reproduction only; further dose/rate/seed continuation stopped. [Record](../../labs/m2-cross-145-2026-10-10/README.md). |
| [Joint Context-prototype reply negative](joint-reply-negative-20261010.patch) | 2026-10-10 | 2 | Not counted | `1833cb4dd` | `52022ae10` | Executed fixed24/96 joint Context-prototype run scores0/512 and0/24; REJECTED. D21 decisive ordinary-learning continuation exhausted; new joint mode is retained here rather than activated in main. Apply to52022ae10 in a fresh isolated worktree for reproduction only. [Record](../../labs/m2-joint-reply-2026-10-10/README.md). |
| [Joint reply precompile source](joint-reply-precompile-20261010.patch) | 2026-10-10 | 1 | Not counted | `2b0651a41` | `1833cb4dd` | Compile-failed Context receipt error conversions; apply to repaired producer1833cb4dd to restore the failed source. No model ran at that revision. [Record](../../labs/m2-joint-reply-2026-10-10/README.md). |
| [Reply-completion precompile source](reply-completion-precompile-20261010.patch) | 2026-10-10 | 1 | Not counted | `698b41333` | `9a1394f4e` | Superseded compile-failed endpoint binding calls; apply to corrected producer9a1394f4e to restore the failed source. No model ran at this revision. [Record](../../labs/m2-reply-gradient-2026-10-10/README.md). |
| [`codex/legal-basis-diagnosis-20261009` initial labels](legal-basis-initial-label-20261009.patch) | 2026-10-09 | 1 | Not counted | `bfd6adb0e` | `fa9f7cba0` | First diagnostic source, superseded only for missing direct fallback label. Numeric captures remain valid; [record](../../labs/legal-basis-diagnosis-2026-10-09/README.md). |
| [`codex/direct-legal-construction-20261009` absolute encoding](direct-legal-absolute-20261009.patch) | 2026-10-09 | 1 | Not counted | `dd69a64965` | `4d10555f5` | Superseded numerical representation; backend singular-matrix execution failure, no model verdict. Full first commit retained; [record](../../labs/direct-legal-construction-2026-10-09/README.md). |
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

## Other labs' stale branches (9 October)

Branches from other labs (and older unprefixed work) whose changes were not in
`main`, with no open PR, no commit in the 48 hours before the pass, and not
checked out in a worktree. Patches are in [other-labs-20261009/](other-labs-20261009/);
`.local` marks a local copy whose tip differed from GitHub. `gh-pages` (the
Pages source) and `codex/lab-state` were excluded.

| Branch | Last commit | Commits | Author | Tip | Merge-base | Last subject |
|---|---|---|---|---|---|---|
| [`ask-form-breadth`](other-labs-20261009/ask-form-breadth.patch) | 2026-10-05 | 5 | Casey Allard | `41f975e135` | `71deab50ac` | Revert the closed-label fallback: measured ZERO fires in 60 real conversations |
| [`ask-form-breadth.local`](other-labs-20261009/ask-form-breadth.local.patch) | 2026-10-05 | 5 | Casey Allard | `41f975e135` | `71deab50ac` | Revert the closed-label fallback: measured ZERO fires in 60 real conversations |
| [`open-relations-wiring`](other-labs-20261009/open-relations-wiring.patch) | 2026-10-05 | 9 | Casey Allard | `edc43742c4` | `416e25a387` | cargo fmt on the prose probe |
| [`open-relations-wiring.local`](other-labs-20261009/open-relations-wiring.local.patch) | 2026-10-05 | 9 | Casey Allard | `edc43742c4` | `416e25a387` | cargo fmt on the prose probe |
| [`geometry-priority-suspension.local`](other-labs-20261009/geometry-priority-suspension.local.patch) | 2026-10-05 | 3 | Casey Allard | `870ffd3468` | `9d77908755` | Expose the pointer head and route on mqar-bench: the ngram successor route was unreachable |
| [`open-relations-extractor`](other-labs-20261009/open-relations-extractor.patch) | 2026-10-04 | 2 | Casey Allard | `807c28c748` | `416e25a387` | Make the extractor branch truly extractor-only (review: three blockers) |
| [`open-relations-extractor.local`](other-labs-20261009/open-relations-extractor.local.patch) | 2026-10-04 | 2 | Casey Allard | `807c28c748` | `416e25a387` | Make the extractor branch truly extractor-only (review: three blockers) |
| [`pr1678.local`](other-labs-20261009/pr1678.local.patch) | 2026-10-04 | 1 | OpenCode DeepSeek | `415fc9edab` | `f5436e1db7` | Align a located value to the source's word spans, and locate at word starts |
| [`pr1678b.local`](other-labs-20261009/pr1678b.local.patch) | 2026-10-04 | 2 | OpenCode DeepSeek | `46ba5a42dc` | `e9d761a9bf` | Replace parse_op's duplicated doc with one block; tidy the new tests |
| [`pr1690.local`](other-labs-20261009/pr1690.local.patch) | 2026-10-04 | 1 | OpenCode DeepSeek | `92699f10bd` | `e8226b7641` | Port the held-out question-form builder to Rust, byte-identical |
| [`pr1690b.local`](other-labs-20261009/pr1690b.local.patch) | 2026-10-04 | 3 | OpenCode DeepSeek | `92dff3da10` | `e8226b7641` | Add held-out v2 from genuinely new forms; keep v1 reproducible |
| [`lab/anti-gravity/d4-qat-results-backup-20260930.local`](other-labs-20261009/lab_anti-gravity_d4-qat-results-backup-20260930.local.patch) | 2026-09-30 | 30 | Casey Allard | `e595de685d` | `e65aa96048` | docs(d4): reconcile S2 dialogue QAT findings per non-author review (#973, #820) |
| [`pr-1526-review.local`](other-labs-20261009/pr-1526-review.local.patch) | 2026-09-30 | 8 | Casey Allard | `064adb2c2c` | `7b11e1d803` | fix(training): restore rayon alongside half in Cargo.toml and format (#1526, #820) |
| [`lab/anti-gravity/d4-qat-results-staging.local`](other-labs-20261009/lab_anti-gravity_d4-qat-results-staging.local.patch) | 2026-09-29 | 26 | Casey Allard | `3617721f52` | `e65aa96048` | Merge origin/main into lab/anti-gravity/d4-qat-adapters |
| [`opencode-b0-flock.local`](other-labs-20261009/opencode-b0-flock.local.patch) | 2026-09-29 | 5 | Casey Allard | `bd14a2791f` | `ba266ad44f` | Merge remote-tracking branch 'origin/main' into lab/opencode/b0-flock |
| [`lab/anti-gravity/d4-e8-codec-backup.local`](other-labs-20261009/lab_anti-gravity_d4-e8-codec-backup.local.patch) | 2026-09-28 | 6 | Casey Allard | `dfc4217c80` | `49feed4826` | docs(qat): author D4 QAT representation plan for learning-to-serving gap reduction |
| [`lab/anti-gravity/d4-fidelity-study`](other-labs-20261009/lab_anti-gravity_d4-fidelity-study.patch) | 2026-09-28 | 3 | Casey Allard | `71aba3a00e` | `8c4e45de88` | docs(d4): align audited symbol count to 63 in current state and evidence record |
| [`lab/anti-gravity/d4-fidelity-study.local`](other-labs-20261009/lab_anti-gravity_d4-fidelity-study.local.patch) | 2026-09-28 | 3 | Casey Allard | `71aba3a00e` | `8c4e45de88` | docs(d4): align audited symbol count to 63 in current state and evidence record |
| [`lab/anti-gravity/dialogue576-chat-serving`](other-labs-20261009/lab_anti-gravity_dialogue576-chat-serving.patch) | 2026-09-28 | 13 | Casey Allard | `a8d59d31c9` | `73a21c2645` | feat(integer): full-path M1 cost qualification and zero-matmul WASM capability runtime |
| [`lab/anti-gravity/dialogue576-chat-serving.local`](other-labs-20261009/lab_anti-gravity_dialogue576-chat-serving.local.patch) | 2026-09-28 | 13 | Casey Allard | `a8d59d31c9` | `73a21c2645` | feat(integer): full-path M1 cost qualification and zero-matmul WASM capability runtime |
| [`lab/anti-gravity/runtime-efficiency-t3`](other-labs-20261009/lab_anti-gravity_runtime-efficiency-t3.patch) | 2026-09-28 | 9 | Casey Allard | `b7e20b8b6b` | `aa7a68f5e1` | fix(energy): commit raw-root SHA-256 manifest, align default horizons to 10,240/32,768, and export s |
| [`lab/anti-gravity/runtime-efficiency-t3.local`](other-labs-20261009/lab_anti-gravity_runtime-efficiency-t3.local.patch) | 2026-09-28 | 9 | Casey Allard | `b7e20b8b6b` | `aa7a68f5e1` | fix(energy): commit raw-root SHA-256 manifest, align default horizons to 10,240/32,768, and export s |
| [`pr-1479.local`](other-labs-20261009/pr-1479.local.patch) | 2026-09-28 | 5 | Casey Allard | `700872f763` | `a10d71ec5d` | test(codec): add head_quantization_mse_comparison evaluating geometric_s1 output head (#973, #820) |
| [`lab/opencode/g-v1.local`](other-labs-20261009/lab_opencode_g-v1.local.patch) | 2026-09-28 | 4 | Casey Allard | `8b72954106` | `2a7a0dec33` | G v1 result: precision fixes from the independent headline re-read |
| [`lab/anti-gravity/geometric-intelligence-synthesis`](other-labs-20261009/lab_anti-gravity_geometric-intelligence-synthesis.patch) | 2026-09-27 | 8 | Casey Allard | `7175230f4b` | `73a21c2645` | fix(integer): fail-closed stream drop safety, opcode symbol coverage, and roadmap track alignment (R |
| [`lab/anti-gravity/geometric-intelligence-synthesis.local`](other-labs-20261009/lab_anti-gravity_geometric-intelligence-synthesis.local.patch) | 2026-09-27 | 8 | Casey Allard | `7175230f4b` | `73a21c2645` | fix(integer): fail-closed stream drop safety, opcode symbol coverage, and roadmap track alignment (R |
| [`lab/anti-gravity/phase-b-speedups`](other-labs-20261009/lab_anti-gravity_phase-b-speedups.patch) | 2026-09-27 | 7 | Casey Allard | `bc69b869fa` | `73a21c2645` | fix(integer): ensure mul_fraction_radix16 covers all 64 bits of u64 and add scalar equivalence test  |
| [`lab/anti-gravity/phase-b-speedups.local`](other-labs-20261009/lab_anti-gravity_phase-b-speedups.local.patch) | 2026-09-27 | 7 | Casey Allard | `bc69b869fa` | `73a21c2645` | fix(integer): ensure mul_fraction_radix16 covers all 64 bits of u64 and add scalar equivalence test  |
| [`archive/causal-experiment-source-20260924`](other-labs-20261009/archive_causal-experiment-source-20260924.patch) | 2026-09-23 | 1 | Casey Allard | `4af04cf56f` | `7fb232ad55` | Add exact learner checkpoints, causal feedback fitting, learned relative actions and sparse read pla |
| [`archive/causal-experiment-source-20260924.local`](other-labs-20261009/archive_causal-experiment-source-20260924.local.patch) | 2026-09-23 | 1 | Casey Allard | `4af04cf56f` | `7fb232ad55` | Add exact learner checkpoints, causal feedback fitting, learned relative actions and sparse read pla |
| [`archive/principal-evidence-source-20260923`](other-labs-20261009/archive_principal-evidence-source-20260923.patch) | 2026-09-23 | 7 | Casey Allard | `fc843a0122` | `f5fa0977a1` | Preserve frozen selection provenance in transferred bundles and scope Hamilton interpretation |
| [`archive/principal-evidence-source-20260923.local`](other-labs-20261009/archive_principal-evidence-source-20260923.local.patch) | 2026-09-23 | 7 | Casey Allard | `fc843a0122` | `f5fa0977a1` | Preserve frozen selection provenance in transferred bundles and scope Hamilton interpretation |
| [`archive/principal-20260923/uor-r4-v3-principal-1352.local`](other-labs-20261009/archive_principal-20260923_uor-r4-v3-principal-1352.local.patch) | 2026-09-22 | 5 | Casey Allard | `fe79249b44` | `6ae7625ffa` | Record corrected V3 review charge without extension |
| [`issue-973-learned-lexical-prefix-city-transfer.local`](other-labs-20261009/issue-973-learned-lexical-prefix-city-transfer.local.patch) | 2026-09-07 | 1 | Casey Allard | `e55295dc50` | `47c5b3f4b5` | Learn lexical-prefix-to-copy transition and resolve city transfer (#973) |
| [`issue-1107-workbench-candidate.local`](other-labs-20261009/issue-1107-workbench-candidate.local.patch) | 2026-09-03 | 1 | Casey Allard | `5b20051261` | `8ba5887e06` | Implement native Four-fact workbench candidate (#1107) |
| [`issue-973-score-readout-result.local`](other-labs-20261009/issue-973-score-readout-result.local.patch) | 2026-08-30 | 1 | Casey Allard | `9c3da95067` | `df6e2dcba0` | docs: publish #973 localization result and reset direction |
| [`r5/gate-core-runtime-dormant`](other-labs-20261009/r5_gate-core-runtime-dormant.patch) | 2026-08-10 | 1 | Claude | `634fa4bb3d` | `9a00460ab5` | feat(#515): PR-2 — compile-gate core & graph-runtime dormant modules (default OFF) |
| [`r5/gate-core-runtime-dormant.local`](other-labs-20261009/r5_gate-core-runtime-dormant.local.patch) | 2026-08-10 | 1 | Claude | `634fa4bb3d` | `9a00460ab5` | feat(#515): PR-2 — compile-gate core & graph-runtime dormant modules (default OFF) |
| [`i450-repro-hardening`](other-labs-20261009/i450-repro-hardening.patch) | 2026-08-06 | 4 | Casey Allard | `275a4e1d70` | `4957bbd2a7` | Merge pull request #453 from UOR-Foundation/i451-canonical-tiebreak |
| [`i450-repro-hardening.local`](other-labs-20261009/i450-repro-hardening.local.patch) | 2026-08-06 | 2 | Claude | `921be67058` | `4957bbd2a7` | fix(compiler): #451 canonical tie-break in emit_r4g1 emission selection |
| [`issue-264-r4g1-realization`](other-labs-20261009/issue-264-r4g1-realization.patch) | 2026-07-31 | 3 | Ari | `80b794893f` | `0c27ab446a` | Merge pull request #319 from UOR-Foundation/issue-263-region-objects |
| [`queue-sim.local`](other-labs-20261009/queue-sim.local.patch) | 2026-07-30 | 7 | Casey Allard | `48fc78f501` | `4cc195d7b2` | Merge remote-tracking branch 'origin/issue-243-phase-c' into queue-sim |
| [`fix/r4g1-abstain-fallback-and-routing-speed`](other-labs-20261009/fix_r4g1-abstain-fallback-and-routing-speed.patch) | 2026-07-26 | 22 | Casey Allard | `d65b5ab87f` | `4d323527e1` | feat(cli): update banner ASCII art to UOR-R4-CLI, intro title, and prompt label |
| [`fix/r4g1-abstain-fallback-and-routing-speed.local`](other-labs-20261009/fix_r4g1-abstain-fallback-and-routing-speed.local.patch) | 2026-07-26 | 22 | Casey Allard | `d65b5ab87f` | `4d323527e1` | feat(cli): update banner ASCII art to UOR-R4-CLI, intro title, and prompt label |
| [`issue-125-shared-node-edge-algebras.local`](other-labs-20261009/issue-125-shared-node-edge-algebras.local.patch) | 2026-07-24 | 1 | Casey Allard | `4d63710702` | `624a1cfeaa` | feat(format): introduce 9 unified edge algebras over shared node space (#125) |
| [`issue-126-holographic-encoding.local`](other-labs-20261009/issue-126-holographic-encoding.local.patch) | 2026-07-24 | 1 | Casey Allard | `5a73af2548` | `624a1cfeaa` | feat(compiler): introduce holographic encoding, partial reconstruction, and progressive fidelity (#1 |
| [`issue-127-information-bottleneck.local`](other-labs-20261009/issue-127-information-bottleneck.local.patch) | 2026-07-24 | 1 | Casey Allard | `c4901c61b4` | `624a1cfeaa` | feat(compiler): introduce predictive entropy and Information-Bottleneck objectives (#127) |
| [`issue-124-semantic-state-model.local`](other-labs-20261009/issue-124-semantic-state-model.local.patch) | 2026-07-24 | 1 | Casey Allard | `84e072002a` | `624a1cfeaa` | feat(compiler): introduce reference semantic state space model and typed graph dynamics (#124) |
| [`issue-128-behavioral-probes.local`](other-labs-20261009/issue-128-behavioral-probes.local.patch) | 2026-07-24 | 1 | Casey Allard | `a23ee825df` | `624a1cfeaa` | feat(compiler): introduce unsupervised intervention and counterfactual behavioral probes (#128) |
| [`issue-129-reference-compiler-ir.local`](other-labs-20261009/issue-129-reference-compiler-ir.local.patch) | 2026-07-24 | 1 | Casey Allard | `71861de00a` | `624a1cfeaa` | feat(compiler): introduce reference floating-point semantic compiler and intermediate representation |
| [`issue-130-lower-semantic-regions.local`](other-labs-20261009/issue-130-lower-semantic-regions.local.patch) | 2026-07-24 | 1 | Casey Allard | `342502892e` | `624a1cfeaa` | feat(compiler): lower reference semantic regions into Boolean, mask, popcount, and fixed-point progr |
| [`issue-131-future-state-planner.local`](other-labs-20261009/issue-131-future-state-planner.local.patch) | 2026-07-24 | 1 | Casey Allard | `62e8b9b63b` | `624a1cfeaa` | feat(compiler): implement bounded future-state optimization and planning over graph transitions (#13 |
| [`issue-161-performance-certificates.local`](other-labs-20261009/issue-161-performance-certificates.local.patch) | 2026-07-24 | 4 | Casey Allard | `b5c8b277fb` | `f1b4859e65` | fix(compiler): gate disk and oracle compiler functions under cfg(not(target_arch = wasm32)) (#161) |
| [`issue-165-compiler-executor.local`](other-labs-20261009/issue-165-compiler-executor.local.patch) | 2026-07-24 | 1 | Casey Allard | `0368fbec97` | `991a0d7cdb` | feat(compiler): add deterministic compiler executor abstraction (#165) |
| [`copilot/scoring-redesign-correlated-residual-stacking`](other-labs-20261009/copilot_scoring-redesign-correlated-residual-stacking.patch) | 2026-07-23 | 2 | copilot-swe-agent[bot] | `e4de206ab0` | `a50e9dbcaa` | scoring redesign: add score_candidates_legacy and sibling-stacking test (issue #64) |
| [`copilot/uor-64-f-emissions-ablation`](other-labs-20261009/copilot_uor-64-f-emissions-ablation.patch) | 2026-07-23 | 2 | copilot-swe-agent[bot] | `4933f0d19b` | `a50e9dbcaa` | Drop ΔT offset after F-emissions ablation |
| [`copilot/uor-foundation-add-one-vs-smoothing-comparison`](other-labs-20261009/copilot_uor-foundation-add-one-vs-smoothing-comparison.patch) | 2026-07-23 | 2 | copilot-swe-agent[bot] | `143c2dff70` | `94501bd031` | Apply remaining changes |
| [`copilot/uor-foundationuor-r4-64-chain-telescoped`](other-labs-20261009/copilot_uor-foundationuor-r4-64-chain-telescoped.patch) | 2026-07-23 | 2 | copilot-swe-agent[bot] | `e7d9631429` | `a50e9dbcaa` | Fix scoring redesign: add score_candidates_legacy, sibling double-counting test, Gate C comparison,  |
| [`issue-74-top-k-teacher-evidence`](other-labs-20261009/issue-74-top-k-teacher-evidence.patch) | 2026-07-23 | 7 | Casey Allard | `042d5fcee6` | `7c7b862464` | Potential fix for pull request finding |
| [`issue-75-d3-gate-c`](other-labs-20261009/issue-75-d3-gate-c.patch) | 2026-07-23 | 6 | Ari | `414951faa1` | `8bfe2b4b82` | Merge remote-tracking branch 'origin/main' into issue-75-d3-gate-c |
| [`issue-77-ci-gate-c-trend`](other-labs-20261009/issue-77-ci-gate-c-trend.patch) | 2026-07-23 | 3 | Casey Allard | `ee8a3eb927` | `a5a1e6321a` | Merge branch 'main' into issue-77-ci-gate-c-trend |
| [`feature/quantum-graph-integration.local`](other-labs-20261009/feature_quantum-graph-integration.local.patch) | 2026-07-23 | 7 | Casey Allard | `1100eb16b8` | `8a573f45cf` | fix(model-source): add #[allow(dead_code)] to fast_matmul_backend on non-macOS targets |
| [`copilot/fix-width-newtypes-serialization`](other-labs-20261009/copilot_fix-width-newtypes-serialization.patch) | 2026-07-22 | 2 | copilot-swe-agent[bot] | `f9da5f38c0` | `aaa3478156` | fix: eliminate usize at router wasm/JSON serialization boundaries |
| [`issue-78-resolution-status`](other-labs-20261009/issue-78-resolution-status.patch) | 2026-07-22 | 2 | Casey Allard | `d270b6f6f9` | `139209c615` | Implement ResolutionStatus behavior in the deployed path |

## Codex coupled export recovery and retained negative — October 9

| Branch | Execution source | Patch base | Disposition | Retained evidence |
|---|---|---|---|---|
| [`codex/coupled-export-recovery-20261009`](codex_coupled-export-recovery-20261009.patch) | `a6be8f699eef3841a1a4dd7a51faefc06d83dd7c` | `17819bec55cc330806702c0c854b09b20e536148` | Export repair and learner source delivered through PR #2023; 9/15 conditional candidate remains unselected, with no repeated construction | [Native391, full saved audit and exact artifact receipts](../../labs/coupled-export-recovery-2026-10-09/README.md) |

The patch preserves the five executed Rust source paths at the stated source and base; it passed reverse-application checking against the delivered source. Its SHA-256 is `6e50cfb17ae69ad181e93503e25d0a5889eba772a782d314233941cbd4b5d91b`. Source delivery does not select the negative learned artifact.

## DeepSeek branches (archived 2026-10-09 by the DeepSeek lab)

Six `deepseek/*` branches held work that was not in `main`. They were identified by PATCH-ID
comparison (`git cherry`), not by ancestry: deliveries here are squash-merged, so a branch tip is never
an ancestor of `main` even when its content is fully present, and an ancestry test reports 141 of the
146 `deepseek/*` branches as unmerged when only these six genuinely were. Content diffs mislead for the
same reason, since a branch is cut from an older `main`.

Status: RETAINED, NOT ACTIVATED - an archive for restoration, not a promotion of code into the live
tree. All six branches have been deleted from GitHub and locally. They are dated 5-7 October 2026 and
are superseded experiments, a frozen gate and handoff, a control battery, and a retraction.

NOTE: `deepseek/d20-control-land` (local, deleted) carried the SAME two commits as
`deepseek/d20-control` (5a37b9814 + cf1a807c8); it is a duplicate and has no separate entry.

| Branch | Last commit | Commits | Lines added | Tip | Merge-base | Last subject |
|---|---|---|---|---|---|---|
| [`deepseek/aa-dispatch`](deepseek_aa-dispatch.patch) | 2026-10-05 | 2 | +2346 lines | `c83c25e57` | `258e1c6698` | addressed_attention: arithmetic oracle — the operator computes exactly; the emission head canno |
| [`deepseek/bf16-phase2b-gate`](deepseek_bf16-phase2b-gate.patch) | 2026-10-05 | 3 | +533 lines | `10e2577f6` | `6a918a4349` | Phase 2b gate: run the frozen read A/B and print the decision |
| [`deepseek/d20-control`](deepseek_d20-control.patch) | 2026-10-07 | 2 | +1440 lines | `cf1a807c8` | `d2cae5ba57` | Preserve the D20 matched-control battery and its analysis |
| [`deepseek/d20-run`](deepseek_d20-run.patch) | 2026-10-07 | 4 | +3597 lines | `d5172e7f6` | `d2cae5ba57` | Preserve the D20 matched-control run products |
| [`deepseek/entry-ceiling`](deepseek_entry-ceiling.patch) | 2026-10-07 | 4 | +219 lines | `e56c1fc74` | `d99e999220` | Add the one-token-prefix rank control to the entry ceiling |
| [`deepseek/entry-isolation`](deepseek_entry-isolation.patch) | 2026-10-07 | 2 | +136 lines | `1d98faaea` | `84459eaf62` | Retract the drowned-entry diagnosis in the 2026-10-07 handoff |

A verified restorable bundle of all six (complete object closure, ~115 MB) is in iCloud rather than
this repository - GitHub rejects files above 100 MB, and bulk material belongs in iCloud with `main`
holding the index. Object `icloud:UOR-R4/results/deepseek/ds-bundle-icloud.tar`, tar md5
`3d6233391fa48623e18589e2778d326a`, bundle md5 `14d17ccd22f1ee196e1479866823be4f`. Verified 2026-10-09:
`git bundle verify` reports a complete history, and fetching it into a scratch clone restored all six
branches with tips matching the table above. The patches here are readable records but are NOT reliably
applicable on their own (0 of 6 plain, 1 of 6 three-way), because the branches predate `main`'s
movement - use the bundle to restore.

## Codex saved-incidence reader superseded I/O — October 9

| Retained source | Status | Restore base | Evidence |
|---|---|---|---|
| [Unbuffered reader](codex_saved-pair-incidence-unbuffered-20261009.patch) | Superseded analysis I/O; no model change | Apply to the buffered `native-saved-pair-incidence.rs` delivered with this row; final file SHA-256 `bc24d467eef6a4714bc7404715ed1cb3d45d480d4d7467a5e8cdb9912490665a` | [Both sealed attempts and exact source](../../labs/saved-pair-incidence-2026-10-09/README.md); restored source SHA-256 `2a1a5713f0a275eceaf7c9f687f3e79183c17ae438e787707c6d783a8fc068b8` |

## Retired coordination tail

| Branch | Base already archived on main | Final tip | Disposition |
|---|---|---|---|
| [`codex/lab-state`](codex_lab-state.final-heartbeats.patch) | `669fc6dafb578a338560c446028d00e2e81a5692` | `d9eca2ab1d8240bd854e2ae92bf29f8f1781c765` | Two final heartbeat commits; no source/research changes. Complete 1,976-commit history retained in the independently restored [full bundle](../lab-state-20261009/lab-state.bundle). Standing branch retired. |

## DeepSeek answer-span supervision instrument — paused, October 9

The instrument behind the [answer-span supervision round](../../labs/answer-span-supervision-2026-10-09/README.md):
`answer_span_supervision=0|1` (default 0 = off), which restricts a labelled response's language-loss
weights to the answer span (the positions whose next token is a token of the answer's expected value)
plus the response's terminating target. It is **PAUSED / UNACTIVATED**: the round measured that the
recorded training corpus has no target population for it — 0 of 2,012 supervised answer runs in the
`copy` source contain sentence scaffolding, so the answer span IS the response run there, and the failing
prefix is a verbatim copy of the prompt, i.e. context rather than a supervised target. The intervention
is **inapplicable to the recorded corpus by measurement**; do not reactivate it expecting a result.
It becomes applicable only to a store that carries `binding_labels.json[l]` for the supervised source
(a `dialogue-recall-corpus binding_labels=1` store).

| Branch | Last commit | Commits | Lines not in main | Tip | Merge-base | Last subject |
|---|---|---|---|---|---|---|
| [`deepseek/answer-span-supervision`](deepseek_answer-span-supervision-20261009.patch) | 2026-10-09 | 1 | 0.82 of 289 | `d46bbd9e8` (local-only; the patch is the durable record) | `fa786e4ef` | Answer-span supervision instrument: keep the response loss on the value's own span plus the terminating target (paused) |

The patch is `git diff fa786e4ef d46bbd9e8` of the two instrument paths only; the debug-underflow guard
from the same branch landed separately as live source in the round's PR, and the owner's own
`Reply.span_extract` compile fix (#2066, `fa786e4ef`) is the rebase base and is not carried here. It
applies cleanly to `fa786e4ef` (`git apply --check` PASS) and its SHA-256 is
`b380c8ca32aea92b789a4bd07d6e80fba6a115ddaef59693d7a4ebced3bc69e4`. Its two focused tests are in the
patch (`cargo test -p uor-r4-training --lib answer_span`) and need the `dialogue_episodes.rs:613`
debug-underflow guard that lands as live source in the round's PR — without it they panic in a debug
build before reaching any assertion.

## Codex residual-qualified LU experiment — unpromoted, October 9

| Source | Base | Disposition | Evidence |
| --- | --- | --- | --- |
| [Offline backend numerical patch](basis-residual-repair-20261009.patch) | microlp 0.6.0 plus the corrected observation installer from main d4628eff9 | CLOSED negative line under D21; backend NOT PROMOTED. Both old captured bases qualify, but the unchanged constructor still returns no assignment after two later basis rejections | [Record](../../labs/basis-residual-repair-2026-10-09/README.md), source freeze 1fe9602d22fa19d9c8765952c6fb20a4a6f45ac7 |

This dependency patch includes the new repair module and isolated workspace declaration.
Its prerequisite is the pinned upstream copy installed by
docs/labs/legal-basis-diagnosis-2026-10-09/apply-observer.py. It is not a production
Cargo dependency override. The standalone reproduction harness, installer and tests
remain in the dated record; all execution evidence is retained privately.
SHA256: 39ef8a574a492a9857bf8fda7bb1993aaebfafae3c569877e5204d5f4984cb0a.

## DeepSeek pointer/mixture line — closed negative under D21 §1, October 10

| Source | Base | Disposition | Evidence |
| --- | --- | --- | --- |
| **None** — every arm in the line was trained with options the trainer already carries (`pointer_gate_supervision`, `read_binding_supervision`, `pointer_identity`, and the generated recall mixtures); no trainer, model or panel source is deactivated by this closure | `main` `020a13fe7` | **CLOSED as negative after four cycles**: the mixture dose moved v5 memory 10/40 → 20/40 at a 10 % recall share but saturated, the copy gate and the read-binding objective were flat on the panel, the binding-dense mixture was worse, and the token-identity pointer — the one decisive run the 3/3 pivot named — left memory at 19/40 against the pre-registered 21/40 bar. Superseded by the next M1 piece: an addressed-memory operator in the dialogue stack | [Mixture dose](../../labs/mixture-dose-2026-10-10/README.md) · [Read-binding](../../labs/read-binding-2026-10-10/README.md) · [Binding-dense](../../labs/binding-dense-2026-10-10/README.md) · [Token-identity](../../labs/pointer-identity-2026-10-10/README.md) |

No patch is carried because no unique branch source exists: the four records' commands are the reproducible
form of the line, their acceptance reports are in the cloud-store bundles named in each record, and the pivot
card that closed it is [#2029 comment 6097789124](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6097789124).
