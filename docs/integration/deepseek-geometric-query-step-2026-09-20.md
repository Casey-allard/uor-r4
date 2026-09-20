# DeepSeek execution prompt — test a matched geometric query read

**Post-#1310 correction:** the [principal review](query-read-review-2026-09-20.md) preserves the numerical results while correcting history attribution, tokenizer binding, continuation, source/cost and cycle claims. The [frozen S attribution prompt](deepseek-separable-attribution-step-2026-09-20.md) now owns active work. Historical instructions/results below retain their original scope; the full original contract was not completely fulfilled.

Work on **UOR-R4 Geometric Language Model**, `UOR-Foundation/uor-r4`. Refresh origin/main beyond reviewed PR #1308, merge `d92bd072b53891e83bc5ccf09c07a7ba3e5eba2b`. Read this prompt and [the principal review](geometric-query-review-2026-09-20.md) completely, followed by the current plan/state/policy. Deliver one bounded implementation and real-text experiment: **a learned query-conditioned read of an ordered geometric history state**, compared with equally parameterized separable and local-only reads. Keep the corrected empirical head E frozen. Complete the actual experiment and protected delivery; a source module, authored fixture or proposed study alone is not the result.

## 1. Recover authority and exact inputs

Read AGENTS.md, DECISIONS.md D0-b/D1/D2, execution policy, README, project-track, current-state, model-direction and PROJECT_MAP. Current owner decisions override older contradictory serving language: offline Rust float/gradients/matrix products are allowed; bounded <=4-bit/ternary serving maps may execute through add/subtract/shift/table operations with no multiplier or floating/transcendental numerical kernel. Geometric routing remains the priority. No hidden transformer/provider or Python model implementation. Preserve the separate frozen R4G1 contract.

Use an isolated full `codex/…` worktree from refreshed main. Preserve the owner checkout, all research, old artifacts and negative candidates. Refresh #973/#820/#963/#964 and existing project items; assign work actually active. Claim every new report root exclusively after argument validation and before loading a model; seal and verify its complete file set. Never write derived data beneath a sealed root or repeat a complete run merely to add a report field. Support evaluation from retained final artifacts and save resumable fitting state independently.

Root: `/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/`. Verify:

| Relative path | SHA256 |
| --- | --- |
| `head-projection-3/corrected/empirical.cpl2` | `565cf98a01273af38425687adb522704b09a89bd7ab976b58b20bfbdaccfa0bf` |
| `head-projection-3/result.json` | `d7e8f89232ab822401dc9bd6aec254ddb6b0cc7d93ef0d6459b771010e6fbde2` |
| `head-projection-3/panel.json` | `184ed85af96e6387c57c64e9de73a4ab6b4cabfab58fbb245b87918478e8b5b7` |
| `prefix-recovery-2/recovery.json` | `15ab737cefd758d707e0f74fa18881c75bdca9f93faf00ba1d3c688fa4abf0d8` |
| `prefix-recovery-2/artifacts/arm0-learned_older_prefix.cpx2` | `c66bec57d28204f0580054950ccdc1e821669853e73111e1fd2e201ec64aefae` |
| `attempt-1/prior_realtext.ckpt` | `1c3dfedb2248ec76c21f320b7ecec585c7a522a2d47e35ed5dc9eaca2454bff2` |

Derived V=4096 tokenizer SHA256: `a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f`; source tokenizer: `9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c`. Reuse the verified HfBpeTokenizer derivation, pinned `inputs/docs` corpus commit `e9c04e80`, corrected indices in `derived-corrections-1`, exposure in `eval-replay-2`, and panel/donor identities in `prefix-pilot-2`/`prefix-recovery-2`. Do not tokenize a changing checkout.

Use the same 355/38/36 fit/tune/dev documents, same recovered **4096 consumed fit windows /257113 n−1 targets**, and exact **36-document /288-window /17342-target** open-dev panel. The original checkpoint is an exposure witness, not the new model parent. All new arms freeze the corrected E, never the original weaker parent or tokenizer-placeholder E. E dev micro CE is 7.170815717556 bits/target; reproduce within 1e-8 before claiming a new comparison.

Inspect and reuse `learner/{prefix_state,prefix_artifact,prior_learning,lowbit,realtext_support}.rs` and their runners. Inspect the exact group source, artifact and optimizer interfaces instead of duplicating another independent learner. Primary research and limitations are in the review; verify actual source before adopting any additional mechanism. This prompt selects direct query transport, not a new literature-driven architecture sweep.

## 2. Record complete resources before work

Refresh `/Users/casey.allard/uor-r4/.uor-models/native-joint-learning-2026-09-04/model-time.json` and actual storage accounting. Review snapshot: **170238565 /170900000 ms**, remaining **661435 ms**. The last 4200000-ms debit is retained; its complete measured-wall basis is unverified. Do not refund it or debit it twice.

Propose **7200000 ms** total: 900 s implementation/build; 600 s reduced-form/learned fixtures and focused tests; 300 s data/throughput probe; 3600 s three fits; 1200 s final evaluation, controls, artifact checks and generation; 600 s checkpoint/report/delivery/stop reserve. Before consuming this allowance, record the standing-authorized **+7200000-ms limit extension to 178100000 ms**, giving 7861435 ms headroom at this snapshot. Refresh actual values and adjust before use if concurrent work changed them. This document is a proposal, not an applied ledger change.

One fit worker, at most four Cargo build jobs, <=8 GiB peak RSS, <=512 MiB new retained/temporary model/report data plus <=1 GiB incremental reused build output; retain the 128 MiB storage stop margin. Explicitly bound the parent-score cache: roughly 16 KiB per V=4096 context can consume about 1 GiB for 62973 fit contexts. Share it only across the identical E parent and include dev growth in RSS projection. No paid/external compute, corpus download or deletion.

Record UTC boundaries and monotonic elapsed durations for nonoverlapping preparation/build/test/fit/evaluation/retry phases. Charge actual cumulative work once, including failed work; do not charge the reservation. Probe a small actual training batch and evaluation batch before full fits. Reuse valid probe progress if it is part of the fixed run. If projected complete work exceeds headroom, record a revised complete local extension before proceeding under existing owner authorization. Do not quietly shorten the registered dose or exceed a cap. Checkpoint and report the unresolved stage if the configured limit is reached.

## 3. Repair two small reporting/search defects without another calibration campaign

In `learner/head_projection.rs`, run the nearest-code seed at **every** admissible shift including s0; retaining untouched Q0 separately is required but insufficient. Add a focused correlated-Gram regression: d=2, G=[[1,.9],[.9,1]], w=[.6,.6], s0=0, no empirical seed. Untouched [1,1] has J=.608; coordinate optimization at s0 can reach [0,1] or [1,0] with J=.088. These J values are rounded: use 1e-6 tolerance or independently compute from the actual f32-promoted weights. Verify direct objectives and the selected seed/scale; do not write a test that merely mirrors the loop.

Replace the runner's hardcoded `singular:true` with `singularity:NOT_MEASURED` and `singular_inputs_supported:true` for future outputs. No rank calculation is required. Preserve old report bytes. Document that #1308 measured the reduced search; the complete prescribed projection remains NOT_RUN. These source repairs do not authorize another full QG run and do not block the independent read experiment after their focused tests.

## 4. Implement exactly three matched residual arms

V=4096, frozen parent feature dimension128, F=10, H=64. Let the artifact-bound exact 2I table have identity e and the existing eight-element palette gamma. **Palette slot 0 and group identity ID are different domains** (historically e has group ID 1); resolve through the bound mapping.

Learn separate action logits A[V,8] for chronological writes and B[V,8] for the current-token query. Hard routing uses argmax, lowest palette slot on ties; serving stores packed action IDs. For a prediction after x_i:

- `q = product A(x_j)` in chronological order for `max(0,i−63) <= j < i−1`, at most 62 older tokens, excluding previous/current.
- `b = gamma[B(x_i)]`; the query is current-token-conditioned, not a claimed semantic query.
- `q_tail = gamma[A(x_(i−1))] * gamma[A(x_i)]`.
- R is 120×16 ternary reader codes; W is 4096×16 ternary output codes. Every arm has its own fitted A/B/R/W of identical shape and initialization protocol.

Add these residuals to E's integer logits:

```
Q: (sum_j W[v,j] * (R[q*b,j]      + R[e,j])) << 7
S: (sum_j W[v,j] * (R[q,j]        + R[b,j])) << 7
L: (sum_j W[v,j] * (R[q_tail*b,j] + R[e,j])) << 7
```

Products with ternary W mean add/subtract/skip in serving. Two reader accesses, code sum in −2..2, width 16, fixed residual magnitude bound 4096 per vocabulary row. At positions i<2 set the whole residual to zero in **all** arms; no older prefix is different from an existing prefix whose product equals e. Count all original n−1 targets in CE denominators, including these zero-residual positions. S has an active query channel; L controls added local capacity and the identity-row term. Older fold costs differ and must be measured.

Q−S at identical parameters is `128 W(R[q*b]−R[q]−R[b]+R[e])`. It vanishes when q=e or b=e. The geometric hypothesis is a nonseparable history/query interaction, not more history bits: one 120-state register has at most log2(120)≈6.91 bits and fixed-b right multiplication only permutes it. Writing remains the existing right product; this is **not** selective forgetting or a query-conditioned recurrent update.

Use the exact serialized multiplication/inverse/mapping/palette tables for training hard paths and serving, reusing the validated CPX2 table construction offline. Preserve signed roots, q/−q, historical root order and exact Z[phi] identity. Do not call a lazy floating group-table constructor in the served path. Do not add disconnected prime/hash semantic distances, phases or paired-H4 state without an implemented purpose.

Version the new artifact/API distinctly: old CPX2 uses reader shift 5 plus output shift 3 (total 8); this design uses reader shift 4 plus output shift 3 (total 7), or equivalent direct code-sum shift 7. Never reinterpret existing CPX2 bytes. Bind arm, raw E parent, tokenizer, table/palette identities, dimensions/H/F, both action maps, R/W codes and scale semantics. Validate every allowed shift/code/index, parent/residual combined overflow envelope, byte lengths and input IDs; the old loader's unrestricted per-row W shifts are not acceptable in the new format. Export/reload, validate full-panel integer logits, then use reloaded artifacts for reported final CE, controls and generation.

## 5. Verify representability and learned query credit cheaply

Construct a V=4 constant parent with zero context heads, F=10, bias_scale_bits=10, bias codes [0,−1,−7,−7]. Let a=gamma[1], b=gamma[2], verify noncommutation and distinct e,a,b,a*b. Set A(0)=e,A(1)=a; token 2 is the separator. Set B(0)=e,B(1)=b. On reader coordinates 0..7 set R[e]=R[a*b]=+1, R[a]=R[b]=−1; all other codes zero. Set class 1 W on those eight coordinates to +1 and other W codes zero.

Use four windows `[0,2,0,1]`, `[0,2,1,0]`, `[1,2,0,0]`, `[1,2,1,1]`. **Score only i=2 predicting the last token**, not incidental positions. Q class 1 minus class 0 differences must be +1024,−1024,−1024,+1024, giving 4/4 unique correct argmaxes after actual export/reload. Any separable binary log-odds function satisfies D00+D11=D01+D10, making that strict checkerboard impossible. This is an authored representability witness, not a training result.

Separately freeze this authored A/R/W, initialize B identity-preferred with logit .1 versus 0, and train **only B** for 128 updates on the balanced four selected observations per batch, using the same B optimizer/rate/Jacobian below. Require learned exported B to obtain 4/4 unique correct predictions. Swapping source 0/1 with query/target fixed and swapping query 0/1 with source/target fixed must each destroy the four correct predictions; read-disabled output is the constant parent, correct 2/4. Save actual logits/routes/outputs. This validates learned query credit only, not learned writes or generalization. If it fails, diagnose source/gradient/fixture faults within the recorded budget before real text; do not relax acceptance or search configurations until it passes.

Additional focused tests: future-token causality; window reset and absent-prefix masking; identity Q/S logit and R/W/A-gradient equality; correctly distinct B gradients; mixed group difference; repeated-token/aliased-row credit; new artifact round trip and malformed bounds; L's exact older-prefix invariance. R[e] receives gradient like any reader row. Two coincident selected rows receive **two** credits.

## 6. Use the correctly scaled surrogate gradient and a resumable fixed fit

For integer score Z and natural logits Z*2^−F, bits loss gives:

```
d_v = (softmax(Z*2^-F)_v - 1[v=target]) * 2^-F / ln(2)
g_j = 128 * sum_v d_v * W[v,j]
grad_W[v,j] = 128 * d_v * (R[first,j] + R[second,j])
```

Add g to both selected reader rows. For Q/L, older/local terminal-state adjoint is `u(s)=g dot R[s*b]`; query categorical credit k is `g dot R[q*gamma[k]]` (use q_tail for L). For S, `u(s)=g dot R[s]`, query credit k=`g dot R[gamma[k]]`. The constant identity-row term has no direct A/B derivative. Feed u into the existing reverse chronological A-chain derivative, with the exact ordered right-product convention. Accumulate tied-token credits, then apply separate A/B temperature 1 softmax Jacobians `pi[k]*(credit[k]−sum_l pi[l]*credit[l])` exactly once. Do not reuse the old factor 256.

Check these expressions against finite differences of an explicitly defined continuous multilinear surrogate with hard anchors/adjoints held fixed; finite-differencing hard argmax is not a valid STE test. Also check continuous reader/output relaxation. Log post-Jacobian A/B gradient norms separately from categorical adjoints and record actual hard code changes.

For each of Q/S/L: frozen E; new R seeded ±1; W exactly 0; A initial per-token preferred palette slot from existing seed 13 routine, logits .1/0; B preferred identity for every token, logits .1/0. Same R/W/A/B initialization across arms. At step 0 all logits must equal E. First 64 updates train R/W with A/B frozen; Q/S scores and R/W gradients/updates must be identical. Their B gradients generally differ and are not applied during warmup. Frozen A/B moments and optimizer ages remain unchanged: their first applied update has age 1, and final R/W and A/B update counts are 512 and 448. A verified common Q/S warmup state may be reused only by creating separately arm-bound checkpoints with explicit provenance; production resume must still reject an arm mismatch. Charge shared work once.

Total **512 batch8 updates per arm**, with warmup included, one fixed seed 13 Fisher–Yates pass over the same 4096 consumed windows. Do not substitute repeated512-window exposure. Updates 65–512 train A/B/R/W. Adam beta(.9,.999), eps1e−8, decay 0; R/W lr .03, A/B lr .003. Average by actual target count once; global R/W gradient block clipped at 1, post-Jacobian A/B block clipped at 1. Preserve the existing post-update master clamp [-1,1] for all R/W/A/B arrays. Quantize reader/output with fixed ternary threshold |master|>=.5 to sign and otherwise0; use fixed serving scales above, no new adaptive scale, normalization or bias. Compute stable uncapped log-sum-exp bits CE from the actual hard forward.

Save hard artifacts and complete optimizer checkpoints at 0/64/128/256/512. Primary candidate is final 512, not the best observed dev checkpoint. Use a fixed small fit/tune observation panel for descriptive curves at saved checkpoints, declared before scores; retain its occurrence keys and losses. Do not repeat full development/control/generation at each checkpoint or select extra steps on them. Final registered dev evaluation occurs once.

Checkpoint all A/B/R/W masters, moments, optimizer ages, completed update count, warmup phase, permutation/cursor/RNG state, source/configuration/quantizer, input/parent/tokenizer identities. Run a small on-disk split-versus-uninterrupted continuation check comparing next batch, next update and resulting artifact; do not inherit incomplete CPXS state. A production resume must validate all identities and continue the fixed schedule. Distinguish complete training continuation from merely loading an inference artifact.

## 7. Evaluate final reloaded E/Q/S/L with matched controls

Save per-occurrence losses and aligned document sums/counts on the exact 17342-target panel, plus local pair, older length, action/state/read IDs and donor identity sufficient to reconstruct every claim. Compute micro and macro CE. Use the retained 2000 document bootstrap draws, seed 0x12345678, ratio-of-resampled-loss-sums; preserve exact document alignment. These are nominal repeated open-dev intervals, not final-held-out significance.

Predeclare Q as primary. Practical screen: **CE_E−CE_Q >=.10 bits/target and paired 95% lower bound>0**. Matched-architecture screen: **CE_S−CE_Q and CE_L−CE_Q each have positive point gain and paired lower bound>0**. Do not choose whichever of three arms has the lowest observed CE and call it the preregistered success.

Older-content intervention: reuse the fixed older-prefix donor map, grouped by exact `(previous,current,older_length)`, with targets/local pair/query unchanged. Recompute each donor q using the **recipient arm's own learned A**, never transplant another arm's learned state IDs. Reuse identical donor occurrence IDs across arms. Prior support is 1177 eligible observations across 36 docs; verify counts against the saved panel rather than assuming them. Report singleton strata, excluded no-history positions and unchanged donors. Q's intervention penalty must be **>=.10 bits per eligible target with paired lower bound>0** for a positive older-content conclusion. Report S's penalty and require L to be byte-identical before/after. No relaxed strata to manufacture support. If support is too weak for a positive interval, report inconclusive at this budget.

Disable the entire residual: all arms must equal E exactly. Also neutralize only Q's query to b=e and report its paired loss penalty on positions with an older prefix; a positive lower bound is required before claiming demonstrated use of the learned query route. This is an additional mechanism diagnostic, not a substitute for Q-versus-fitted-S/L. Reverse the older order as a descriptive intervention with L invariance; natural text does not guarantee reversal harms every useful representation. Save raw mixed-difference diagnostics, action changes and occupancy; none alone demonstrates language utility.

Generate the same six retained prompts for 64 greedy tokens from reloaded E/Q/S/L with lowest-ID ties, no sampler/repetition penalty/blacklist/output override. Save token IDs and decoded bytes. For Q/S, cycle certificates need the complete bounded token ring and length/PAD/availability state (or a proven equivalent); repeated `(prev,current)` alone is insufficient. L needs its complete local state plus availability. Validate prompt/length bounds without panics. Report all degeneration even if CE improves.

Measure optimized complete actual inference on E/Q/S/L with the same prompt/token/timing protocol, including parent scoring, state fold/query/read/output and token handling. A fit-time parent-score cache is not a serving latency optimization unless explicitly implemented/bound and included in memory/cost. Report parameter/artifact/state bytes, two reader accesses, group transitions, arithmetic envelope and actual peak RSS scope. Do not relabel whole-training RSS as serving RSS. Physical energy remains UNAVAILABLE without measurement. A compliant numerical path is not complete-path energy evidence.

## 8. Decide, preserve and deliver

If all primary/matched/older-content/query-use screens and instrument checks pass, retain Q as a bounded evidence-supported query-read candidate. Next consider selective write/reset or exact occurrence/version access, with a new independent behavior panel; do not immediately claim alpha, fluent conversation, coding, frontier performance or a special advantage of 2I over other finite groups.

If L alone improves, that is local capacity/learning evidence. If S improves and Q does not beat it, older information may help while this multiplicative query read does not add demonstrated value. If routes never learn, distinguish optimization from representation. A valid negative rejects this fixed architecture/dose, not geometric language modeling. If controls are underpowered or an instrument fails, name the unresolved condition. Preserve every candidate and choose one smallest evidence-supported successor; no automatic width/precision/corpus sweep or unrelated calibration rerun.

Run rustup-managed Cargo formatting, touched-package offline checks and focused tests for the changed causal/arithmetic/serialization/gradient/resume paths. Run actual learned fixture and real-text generation; queue compatibility acknowledgements are not tests. Run `python3 scripts/check_claim_wording.py` for changed claims. Preserve raw historical receipts and report new source defects as scoped corrections, not retroactive edits of old artifacts.

Seal one complete final report with source/binary/config/input/parent hashes, table/action/scales, all arm artifacts/checkpoints, curves, aligned losses/donor map, outputs, decisions, nonoverlapping phase timings and storage receipts. Superseded attempts must retain their own sealed sets and why they were superseded. Reuse final artifacts for derived reporting if a field needs correction.

Update README, current-state, project-track dependencies, model-direction, PROJECT_MAP, CONTINUE, EVIDENCE and the resource ledger. Mark this prompt executed only to the extent actually completed and point to the successor. Update live issue bodies/statuses and any existing project items, preserve historical bodies, and leave broad #973/#820/#963/#964 acceptance open. Stage named paths, push `codex/…`, create/attach protected PR, ordinary merge/queue only, verify actual merged tree/content. Return exact successes/failures, surviving artifact, controls, generated behavior, remaining resources, and **one comprehensive next handoff** tied to the larger shared geometric language-model roadmap.
