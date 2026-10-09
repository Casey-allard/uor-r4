# Coupled residual mechanism diagnosis — 9 October 2026

The completed coupled Prefix/Generate learner's six remaining conditional errors have two different observed boundaries. Positions 7, 8, 9, 10 and 13 never become correct in any of the 14,292 recorded alternatives at their original incumbent epochs. EOS becomes correct in 33 alternatives: 26 fail strict combined cross-entropy descent, seven fail the first checked original guard (guard index 299), and none fail the seventeen-reference check. No recorded feasible alternative exceeds 9/15 task positions. This is evidence about the completed finite pass, not a proof that the parameter family cannot solve the task.

This report reads the already completed, independently audited construction delivered in [#2023](https://github.com/UOR-Foundation/uor-r4/pull/2023). It performs no training, encoder capture, gradient extraction, coefficient proposal, pool perturbation or autoregressive run. The original selected model remains unchanged. Useful conversation and whole-answer improvement are not established.

## Inputs, execution and checks

The [analyzer](analyze_saved.py) authenticates the original report SHA-256 `0aa9b3c0c2d1defaaa6d93611a78fb348be0016aefe5d7548563250b9a687b8c` and manifest `e7667312b7d18b502e72b0fc41bf2d7d52a4977a020501ecd527da6ef628f9aa`, then executes the pinned Rust complete-file/BLAKE3 verifier. It reads every one of the 391 native snapshots and all 1,920 coordinate records. It joins native winners, target masses and native denominators to the final journal endpoint, records all input snapshot SHA-256 values, and counts every one of the 14,292 saved alternatives. The earlier independent arithmetic audit is reused; this analyzer does not rerun that numerical reconstruction.

Final attempt 5 took **17.658458456397057 seconds**, with peak RSS **295,880 KiB**, in a two-CPU slot on the existing approved pod. The [sealed output](sealed/report.json) was exclusively claimed by the existing Rust claimant before input loading, then sealed and verified by the existing Rust tools. The `sealed/` directory is the exact report file set; surrounding source, logs and documentation are separate evidence. [Attempt 5's log](attempt5.log) records successful sealing, verification and terminal exit 0.

Earlier attempts are retained: attempt 1 failed a comparison between compact objectives and a summary with an additional derived phase field; attempt 2 reached report writing but assumed the launch directory was a Git checkout; attempt 3 completed the numerical analysis before source review; attempt 4 failed Python syntax before claiming a report. Attempt 5 applies the review corrections, including explicit native-denominator comparison and separating base Generate from Generate+U. These are analysis-tool failures, not model negatives. All attempt source and logs are retained. The projection includes preparation, retries, analysis, review and delivery, with the prior cumulative accounting preserved in [the ledger](resource-ledger.json).

## What the saved states distinguish

Each of the six residual current post states is unique among all 391 saved native frames. For the four copied-word residuals, target and current rival physical occurrences have different Prefix coefficient keys in seven or eight of the eight lanes. The observations therefore do not support a simple equality collision of these recorded states or target/rival Prefix keys. They do **not** prove separability under shared coefficients, distinguishability of Generate relative keys, global capacity, or feasibility under the full protected population.

`native.generate_q24` already includes the frozen continuation field U. The source adds U before the pool reduction (`native_bank_generate.rs`), and the saved Prefix reader subtracts U when reconstructing `base_generate_q24`. The table separates those quantities; Q24 values are exact saved integer score differences, target minus current pooled rival.

| Position | Target / rival token | Base Generate gap | Frozen U gap | Generate+U gap | Dominant observed competition |
|---|---|---:|---:|---:|---|
| 7 | 324 / 307 | -17,825,792 | +8,388,608 | -9,437,184 | Copy target and rival; eight distinct Prefix lanes |
| 8 | 91 / 471 | +19,922,944 | +16,777,216 | +36,700,160 | Copy rival wins despite favorable Generate pairwise gap |
| 9 | 2795 / 397 | -13,631,488 | 0 | -13,631,488 | Rival has two physical Copy aliases; seven/eight distinct lanes |
| 10 | 770 / 397 | -14,680,064 | -25,165,824 | -39,845,888 | Rival has two physical Copy aliases; eight distinct lanes |
| 13 | period 16 / 2795 | +120,586,240 | 0 | +120,586,240 | Period has no Copy alias; copied rival wins the common pool |
| 14 | EOS 1 / 3329 | +24,117,248 | -25,165,824 | -1,048,576 | Neither token has a Copy alias; U reverses this pairwise margin |

No residual token winner is changed by clipping. EOS has one low-clipped action elsewhere in its pool, so this is not a claim that all clipping is inactive. The favorable pairwise base Generate gaps at period/EOS are not whole-pool or whole-answer successes. Removing U, changing Copy weights or combining candidate states has not been evaluated here.

## What the learner did and did not examine

Generate unary examined all fourteen alternative Q4 codes at each of its 960 coordinates, at that coordinate's then-current incumbent. Its 13,440 alternatives include 1,740 full passes, 2,068 first guard vetoes and 9,632 objective-gate rejections. Prefix examined only one original-gradient-directed adjacent code at 852 coordinates; 108 original zero-gradient coordinates were skipped. Prefix's 852 alternatives include 556 full passes, 103 first guard vetoes and 193 objective-gate rejections. There was no revisiting or new gradient after the state changed.

Thus the completed pass is not an exhaustive search of Prefix codes or a stationarity test at the final incumbent. None of the five non-EOS residuals can be explained simply as a recorded winning alternative discarded by the acceptance rule: none ever won in the recorded alternatives. For EOS, all seven guard-vetoed winning alternatives first broke guard 299; later guards were not checked. Alternatives from different epochs cannot be combined into a feasible checkpoint.

## Next mechanism dependency

The current coupled learner activates `prefix.coefficients` and `generate.unary`. The existing native Generate operator and Rust training graph also support **ordered two-lane pair coefficients**, currently frozen by this constructor. This is an existing shared geometric interaction, not a new serving case, position cursor or token-specific exception. Source review identifies it as a distinct candidate mechanism, not a proven fix.

The next justified task is a saved **pair-incidence and constraint-sharing preflight**, before fresh learning. Use the actual native `factor_incidence_into` contract and bound artifact edge list. Relative codes use the padded algebra stride 128; each pair index uses the ordered `(left, right)` codes with radix 120. Retain both unary and pair key multiplicities, all legal Generate IDs and physical Copy aliases. Compare residual target/rival keys with the seventeen references and 380 guards. In particular, join EOS's seven first-veto witnesses to the exact guard-299 identity and each alternative's incumbent donor/post state; final endpoint states cannot explain an earlier veto by themselves.

A full 391 × 4096 × four-edge enumeration is 6,406,144 pair incidences, about 24.4 MiB as a flat u32 array before overhead; stream it rather than retaining a broad Python object graph. Confirm four edges and 4096 legal IDs from the actual artifact. Bind the original selected-parent and final-state scopes separately before choosing any learning initializer.

If pairs add no relevant distinction, reject that intervention at this scope. If they supply a distinct interaction, record its protected sharing and full-pool limitations, then prospectively specify a bounded pair-credit implementation and cost. Distinct keys alone do not authorize a training campaign. Preserve the complete episode, seventeen references, 380 guards, original selection authority and immediate whole-answer/EOS qualification after positive construction. Do not repeat the old coordinate pass or add serving exceptions.

## Preservation and pod ownership

The [complete analysis workspace archive](complete-analysis-workspace.tar.gz) preserves all ten new pod files; [its inventory](workspace-preservation.json) compares every member's SHA-256 and byte count with the remote source. The large parent scientific artifacts already have verified private-store copies linked from #2023. Small reports and source are delivered here on main.

After analysis, Codex released its `sol-residual-20261009` lease. At the owner's cleanup request, the live pod check showed DeepSeek's GPU1 job lock busy under a lease through 18:18:52 UTC. The shared pod was retained for that active owner; no further Codex pod compute was started. Final protected merge, source verification and local branch/worktree cleanup are recorded on the PR and #820.
