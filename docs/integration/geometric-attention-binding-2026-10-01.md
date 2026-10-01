# Geometric attention: occurrence credit and identity alignment, October 1

This is research on the existing Rust `StackModel`, not a replacement model or a new delivery framework. The owner corrected the immediate focus to geometric attention while the SSD is being reformatted. Fits and retained artifacts use the internal drive. Grounded conversation and durable memory remain the first alpha capability.

## Mechanism and causal question

The earlier language-only and direct-binding fits attend to value-like positions but do not distinguish the requested key. Direct source-occurrence credit alone does not repair that failure. The saved-model probe observes much larger changes at key-token states than at the following value/answer-marker states, with near-zero query-specific source log-odds contrasts. This motivates changing the identity supplied to the read, rather than changing the Lorentz distance, extending the training dose, or tightening admission.

The opt-in causal predecessor carry is

`q_t = Wq u_(t-1), k_j = Wk u_(j-1), v_j = Wv u_j`,

with zero identity at position zero. `u` is the existing normalized, learned-gain input of the geometric read. Current value, NoRead, age, full-prefix support, residual paths and parameters retain their definitions. The read admits self; no teacher mask removes that competitor. The generic delay parses no token IDs. In this authored grammar it associates the preceding key with its value occurrence and the preceding query key with the answer marker. It is a fixed-lag structural prior, not learned arbitrary syntax.

Occurrence supervision appends one constant source-mask channel to the values of the SAME fused read. Its normalized output is the mass on the labeled occurrence set; the training loss is mean negative log mass. That channel is removed before the output projection and residual. The language prediction path receives no source labels. Duplicate value tokens remain distinct source occurrences. Language-only carry controls determine whether this auxiliary objective is needed for useful addressing.

## Fixed experiment

Two initialization/data seeds; Dot and Lorentz reads; no-carry/carry and answer-only/answer-plus-binding objectives. Width 32, two heads, `rra` (two native geometric recurrences followed by one read), MLP 64, context ceiling 64, no pointer mixture or candidate selector. Each fit uses 320 updates, batches of 16, learning rate 0.003, no weight decay, gradient clipping at one; binding weight one where enabled. Every arm retains its model and uses independent reload.

Training cycles 2/4/8 facts. Sixteen distinct key tokens and sixteen value tokens are independent; duplicate values are allowed. The input is `[BOS, FACT, key, value, ..., QUERY, key, ANSWER]`. Only the final answer position contributes to the answer NLL. This is a token-association task, not general autoregressive language training.

The original evaluation is 48 base episodes with four correlated rows each: base, changed query, swapped values, reversed occurrence order. Its 192 rows are not 192 independent trials. A source mass above 0.5 is sufficient for a unique top source over the entire causal prefix plus NoRead; it is not a complete argmax statistic. Read-output ablation zeros the entire selected layer's output projection, including both heads, and restores a deep-copied weight. It does not isolate head zero.

| Read / seed | Identity carry | Binding weight | Answers / 192 | Head-0 source mass > .5 / 192 | Read disabled answers / 192 | Query pairs both correct / 48 |
|---|---:|---:|---:|---:|---:|---:|
| Dot-language-s1 | 0 | 0.0 | 62 | 0 | 12 | 3 |
| Dot-binding-s1 | 0 | 1.0 | 27 | 24 | 19 | 0 |
| Lorentz-language-s1 | 0 | 0.0 | 54 | 0 | 12 | 2 |
| Lorentz-binding-s1 | 0 | 1.0 | 59 | 28 | 21 | 3 |
| Dot-language-s2 | 0 | 0.0 | 60 | 0 | 8 | 2 |
| Dot-binding-s2 | 0 | 1.0 | 49 | 31 | 16 | 2 |
| Lorentz-language-s2 | 0 | 0.0 | 72 | 0 | 18 | 3 |
| Lorentz-binding-s2 | 0 | 1.0 | 69 | 30 | 4 | 2 |
| Dot-binding-s1 | 1 | 1.0 | 188 | 188 | 9 | 46 |
| Lorentz-binding-s1 | 1 | 1.0 | 192 | 192 | 7 | 48 |
| Dot-binding-s2 | 1 | 1.0 | 192 | 192 | 10 | 48 |
| Lorentz-binding-s2 | 1 | 1.0 | 192 | 192 | 12 | 48 |
| Dot-language-s1 | 1 | 0.0 | 192 | 100 | 14 | 48 |
| Lorentz-language-s1 | 1 | 0.0 | 192 | 0 | 0 | 48 |
| Dot-language-s2 | 1 | 0.0 | 192 | 11 | 15 | 48 |
| Lorentz-language-s2 | 1 | 0.0 | 192 | 0 | 18 | 48 |

All sixteen fits completed the same 320-step dose, and independently reloaded logits match exactly on the same first fixed evaluation episode in all sixteen. Every carry-only arm answers all 192 original rows correctly with ordinary answer loss. Therefore direct occurrence supervision is not necessary for this measured association behavior. It does concentrate a designated head: carry-plus-binding Lorentz is 192/192 source majorities in both seeds. Low language-only head-zero majority is not incorrect-source accuracy; useful work may use head one or a decodable mixture.

The four Dot carry-plus-binding seed-one errors all query key 4 with distractor key 13; each prediction equals key 13's value. Other query-4 rows without that distractor succeed. This is an address-confusion hypothesis, not proof of an eight-fact capacity ceiling. All eight original no-carry arms get zero jointly correct pairs among the 44 query pairs whose expected answers differ; their 0–3/48 total jointly correct counts include the four same-answer pairs.

The fixed fresh development draw (seed 1920173) contains 2/8/16 facts, 64 correlated rows per cohort. All four carry-only arms score 192/192 again; both Lorentz carry-plus-binding arms also score 192/192 and both heads concentrate on the correct occurrence. Dot carry-plus-binding retains errors. Sixteen facts uses 52 input positions, within context64 but beyond training maximum28; it also activates untrained age distances and all sixteen identities. These are joint count/length changes, not a pure capacity test.

| Arm | Carry | Binding weight | Fresh 2-fact answers / 64 | Fresh 8-fact / 64 | Fresh 16-fact / 64 | Head-0 / head-1 source majorities, all 192 rows |
|---|---:|---:|---:|---:|---:|---:|
| Dot-language-s1 | 0 | 0 | 32 | 10 | 7 | 0 / 0 |
| Dot-binding-s1 | 0 | 1 | 13 | 8 | 7 | 27 / 0 |
| Lorentz-language-s1 | 0 | 0 | 32 | 9 | 8 | 0 / 0 |
| Lorentz-binding-s1 | 0 | 1 | 30 | 8 | 8 | 27 / 0 |
| Dot-language-s2 | 0 | 0 | 27 | 10 | 8 | 0 / 0 |
| Dot-binding-s2 | 0 | 1 | 21 | 6 | 2 | 31 / 28 |
| Lorentz-language-s2 | 0 | 0 | 30 | 8 | 7 | 0 / 0 |
| Lorentz-binding-s2 | 0 | 1 | 26 | 11 | 5 | 31 / 0 |
| Dot-binding-s1 | 1 | 1 | 61 | 62 | 51 | 172 / 155 |
| Lorentz-binding-s1 | 1 | 1 | 64 | 64 | 64 | 192 / 192 |
| Dot-binding-s2 | 1 | 1 | 63 | 62 | 59 | 181 / 171 |
| Lorentz-binding-s2 | 1 | 1 | 64 | 64 | 64 | 192 / 192 |
| Dot-language-s1 | 1 | 0 | 64 | 64 | 64 | 85 / 192 |
| Lorentz-language-s1 | 1 | 0 | 64 | 64 | 64 | 0 / 78 |
| Dot-language-s2 | 1 | 0 | 64 | 64 | 64 | 8 / 192 |
| Lorentz-language-s2 | 1 | 0 | 64 | 64 | 64 | 0 / 95 |

Observing both heads explains some language-only allocation: Dot's head one exceeds .5 on all192 fresh rows even where head zero does not. Lorentz language-only answers remain perfect with lower majority counts in both heads; a decodable mixture is possible, but not established by these aggregate observations. The independent answer/intervention behavior is the functional measurement.

The additional carry log-odds probe sealed an error, `probe mass invalid`: its logarithm requires four strictly positive finite cross-masses. The predicate admits nonpositive or nonfinite failures; the specific value/cause is unresolved. No floor, unchanged retry or model-quality conclusion follows. The original successful no-carry latent probe remains preserved.



## Decisions and next mechanism

Preserve the successful Lorentz reader and separate identity/address formation from payload content. The prior negative does not establish a lack of geometric capacity: identity was insufficiently usable at the compared positions under these training conditions, and carry repaired the measured association. Dot is an ordinary control and also benefits from the alignment intervention, so predictive superiority of geometry is not established.

The next research mechanism is a typed identity latch across variable gaps, with a separate current role/payload channel. Capturing and holding an identity is insufficient by itself: filler positions would share the held address and recreate occurrence ambiguity. Associate identity with the value at a learned commit event; retain query/write/hold/reset distinctions. Draw write and query gaps independently, include explicitly nonbinding distracting identities, and require rebinding. Keep the current scorer, full candidate support, training-only occurrence labels and changed-query/value/order interventions. Do not begin another metric, selector or quantizer sweep without a distinct causal question.

A delay register has no inherent multiply/float requirement, but the present implementation and evaluation are floating-point. Integer export, grid reference, served mode and the checkpoint envelope refuse carry until their corresponding representation exists. Direct model save/load binds a versioned carry sidecar to operation, canonical config and weights, and a config marker pins the sidecar hash. Default model behavior/save format stays unchanged. A future integer port must preserve source/value alignment and report complete-path cost; this study retains full-prefix scanning and establishes no bounded index lookup, natural-language binding, memory capacity, energy or frontier result.

Related research distinguishes associative recall/state size from efficient memory editing. BASED analyzes recall versus recurrent-state size; Gated DeltaNet separates uniform forgetting from targeted associative updates. They motivate separate address, payload and update mechanisms; neither validates the project's geometric implementation or its serving target. Sources: [BASED](https://arxiv.org/abs/2402.18668), [Gated DeltaNet](https://arxiv.org/html/2412.06464v3).

## Source, checks, artifacts and cost

Binding production source: `3092ff978dce29be191d173a2b0fbd863c2a4d1a`; saved baseline probe: `8b69ede9df44d4af5870393ba503cf9168bb375a`; carry source: `23c3579bb0510659578010f0a8b4512fcb8ccafb`; carry-only credit control: `3b6a01daafd9a6dec9c741fd07bd60bb17e2a581`. Independent principal engineer and mathematician reviewed the actual mechanism and controls. Source approval is separate from executed results.

Four occurrence-credit tests and five carry/boundary tests executed locally, all passed. They cover prediction/gradient independence from labels, gradient flow and finite differences, exact occurrence sets, invalid/noncausal targets, predecessor q/k with current values, save/reload and metadata refusal, served/export/grid/checkpoint refusal. Formatting and diff checks passed. Compiler mistakes in added observer/test code were repaired before their runs; they are execution history, not negative model evidence.

Retained internal root: `/Users/casey.allard/.local/share/uor-r4/research/attention-binding-20261001/`. Fits/probes/challenges claim distinct roots and seal/verify their actual file sets. `execution-identity.json` supplies the source/executable binding for the original fit binary, whose in-report source field is `UNAVAILABLE`; later binaries pin source at build time. No sealed root is amended. The [compact repository evidence](../evidence/attention-binding-2026-10-01.json) records per-arm counts and model hashes; full rows, weights, sidecars and executables stay at the retained root.

The sixteen fit/evaluation/save/reload arms took 1,328.981 seconds (22 minutes 9 seconds), including 610.423 seconds for the original eight arms and approximately 359 seconds for each four-arm carry continuation. Peak fit footprint stayed below 61 MB; two Rayon threads. Training inputs total 1,472,000 token positions across 5,120 optimizer steps. This token count includes all presented positions; only final answers are language-loss targets. No accelerator or external compute was used.

The complete overlapping session elapsed time, builds, review, delivery and physical storage are charged separately to the existing cumulative ledger; an issue does not reset it. Focused test/example compilation took approximately 14.6 minutes including the final build after importing unrelated merged source. Observers, failed diagnostics and delivery remain part of complete cost. Disposable target allowance is 6 GiB; unique retained allowance 160 MiB (owner-authorized increments from 4 GiB/64 MiB, recorded before use). This lab removes its disposable internal build cache and clean worktree after protected delivery, preserving all models, negative results, executables and source branches.

References #1512; this partial mechanism result does not close geometric attention or its language/serving acceptance.
