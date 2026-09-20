# Exact-occurrence reader: correcting principal audit

September 20, 2026. Reviewed PR #1314 at `c6a3978484540eeae9f94623481621324c90aff4`; its code/result are unchanged at delivered head `fdae053c6198a6f75502f611e9a002e9818aa433`, merged as `fbf542aa4c1f7ed336cd028c6b7b3502cba4118c`. The intervening changes incorporate the structural-memory documentation. This review inspected source and retained outputs, verified all manifest BLAKE3 hashes, and derived corrected counts/provenance without a build, model forward or fit. Original sealed roots are unchanged.

**Decision:** retain the exact occurrence substrate and constructed selection gain. Do not promote the candidate. The next constructive task is learned contextual memory utility: improve source ranking, NoRead and bounded copy strength against actual predictive loss. The previous instruction to freeze ranking and amplitude while adding exactly one abstention signal is superseded. [Execution handoff](deepseek-memory-utility-step-2026-09-20.md); [architecture synthesis](geometric-attention-mechanism-synthesis-2026-09-20.md).

## What the retained measurements establish

| Population | Reader | Latest occurrence | Interpretation |
| --- | ---: | ---: | --- |
| All fresh reader-relevant positions | 411/443 = 0.927765 | 279/443 = 0.629797 | Substantial constructed selection gain, +0.297968 absolute |
| Slot-equivalence subset, same denominator convention | 99/120 = 0.825 | 64/120 = 0.533333 | Uses a supplied token-partition relation |
| Exact-role subset, same denominator convention | 312/323 = 0.965944 | Not reaggregated here | The old 0.138 number used all positions and is not comparable to 0.825 |
| Raw positions with candidates and target payload available | 5 correct reads / 10 positions | Not derived here | Five additional reads choose a wrong payload |
| Raw positions with candidates but target payload unavailable | 0 correct reads / 78 positions | Not derived here | All 78 read a wrong payload |
| All raw reads | 5 correct / 88 reads | Not derived here | 83 wrong = 5 covered errors + 78 uncovered errors |

Fresh synthetic all-position loss worsens by **0.6713 bits/token**; the eight-window raw probe worsens by **2.7891 bits/token**. Those negative results stand. Repetitive generation remains unqualified. The raw probe is small and already inspected; it is a regression/development panel for the next decision, not untouched final evaluation.

The ring retains exact session/sequence identity and observed successor payloads, with a 128-token bound and at most 24 candidates. The local scorer correctly uses frozen E plus S's query row without its history fold. Full exported selector/scale equality is checked, and a real three-process trainer continuation check exists. These are useful implementation results, not general language qualification.

## Instrument and attribution corrections

Source links below refer to the delivered source; line numbers are at the reviewed commit.

1. **Ranking counter:** `crates/uor-r4-core/src/bin/occurrence-reader.rs:1250–1251` increments `rank_hit` for `covered && any Read`. It does not require selected payload correctness. Thus “10/10 covered raw positions correct” is withdrawn. The arithmetic above follows from 88 reads and 83 wrong payloads, with all 78 uncovered candidates selected. Both ranking and abstention need improvement.
2. **Future intervention:** runner lines 1108–1123 reconstruct the original sequence into unused `t2`, then call `analyze(s)` again. The reported causal PASS is a tautological comparison. The inspected data flow appears causal, but the intervention is unvalidated. Mutate actual future tokens and pass that sequence to analysis, comparing earlier decisions, payloads and scores.
3. **Read-disabled comparison:** lines 1092–1104 establish that `apply_residual(None)` is an identity. They do not independently compare against the retained CPX3 S-query-only scorer. Keep the identity result and add a focused independent implementation comparison.
4. **Geometry scope:** `wm.slot(token) = palette[a_codes[token]]` is eight-class equality. No relative H4 product, orientation, Hopf operation or transport is used by this selector. Targets in `geom_slot_only` are authored from that same partition. This measures use of a supplied equivalence relation; semantic or uniquely geometric advantage is not established.
5. **Confounded coefficient ablation:** `f7=f0`, `f8=f2-f0`. The fitted role score simplifies to `(w0+w7-w8)*exact + (w2+w8)*slot = 5*exact + 7*slot`. Zeroing `w2,w3,w7,w8` leaves `4*exact`, changing exact-match weight too. The 0.825 versus 0.550 result is this particular ablation, not isolated geometric contribution. A later comparison should preserve the exact-information score and use a matched nongeometric partition with representation-independent targets.
6. **Decoy construction:** runner 281–290 requires a different role token, not a different slot. Wrong-payload same-slot alternatives occur in 33/158 fit, 10/57 tune and 24/91 fresh geometric sequences. Preserve these as ambiguous cases or explicitly repair and version the construction; do not describe it as uniquely solvable from slot equality.
7. **Paired support:** the saved bootstrap compares reader with local on the slot subset. Reader-versus-latest and reader-versus-ablated intervals were not computed. The observed gains remain, but the declared paired qualification against latest is incomplete. Reaggregate saved vectors where possible; do not rerun a model merely for arithmetic.
8. **Cost:** `run_uncached` performs ring admission/features before branching on `use_reader`. Both baseline and reader pay this cost, so their difference cannot establish negligible total reader overhead. It computes 29 predictions and divides by 32 tokens; corrected medians are about 1,341/1,339 microseconds per computed position under that mixed baseline. True incremental direct-path latency and energy remain UNAVAILABLE.
9. **Provenance:** `executable_sha256` contains the whole Mach-O binary encoded as 2,614,304 hex characters, not a hash. The preserved 1,307,152 executable bytes yield SHA256 `62d4cc51454a3e0859f5ba03ac8b9cbe25ccd2b72c8b0a637fdb700d65595f73`. Source digest fields are valid. Record corrected provenance separately; do not rewrite the sealed receipt.
10. **Decision/emission wording:** the reload trajectory comparison checks Read/NoRead Boolean, though separate full parameter equality protects this export. Compare complete occurrence, payload and logits in a reusable parity control. Altered-source 37/38 checks selected new payload, not final emitted argmax; retain it as source sensitivity, not generated-output success.

## Why utility, ranking and copy strength belong together

With `f_bits=10`, the `2^14` residual is **16 nats**, multiplying the selected token's unnormalized probability by about 8.9 million. For local probability `p_y`, selected payload `y`, true next token `x` and nonnegative logit boost `a`, the exact change in negative log likelihood in nats is

`delta_loss = log(1 + p_y * (exp(a) - 1)) - a * 1[y == x]`.

This is an offline objective identity, not a proposal to execute transcendental functions at serving. Its implication is concrete: a synthetic argmax threshold objective can choose destructive reads. Fit ranking and a small set of bounded copy-strength/NoRead actions against actual post-injection language loss or a justified aligned surrogate. Preserve the frozen full-vocabulary local scorer as an independent path. Add the smallest causal contextual feature if incompatible targets are aliased; more training cannot separate identical observations.

The 130-byte selector is not the whole model or runtime memory. Parent artifacts are 454,788 and 53,555 bytes, the precomputed local row table is 1,966,080 bytes, and scratch/ring state is additional. The current eight reachable local rows suggest a later table-compaction opportunity, but it is not the immediate learning bottleneck.

The live ledger observed by this review is **181238565/191300000 ms**. This source/document audit changes no model debit or allowance. Refresh balances before subsequent work; the previous run's 2700000-ms charge remains once. No paid compute, cleanup or artifact promotion was performed.
