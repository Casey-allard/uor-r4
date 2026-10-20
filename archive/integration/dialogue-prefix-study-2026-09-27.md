# Fixed complete-prefix dialogue learning study

References #973. **Status: RUNNING**, from the principal's launch receipt at
08:50:17 UTC on September 27. Coordinator PID 41259 has launched full-prefix
supervisor 41263 and model 41266; role-only is queued. The 08:54:42 snapshot
verifies those live process identities and 51 complete update records. The
source-bound Rust build and data-only panel preparation completed
before launch. The preparation process exited 0 and retained
161 development responses, with **zero model forwards, backwards, optimizer
updates or generated replies**. This record fixes the substantive comparison;
it does not report a completed fit or promote a new model. The
[evidence packet](../evidence/dialogue-prefix-study-2026-09-27.json) binds
execution and review receipts. It is a dated observation; recheck the actual
process and artifacts for current execution status. The [canonical current state](current-state.md)
owns subsequent launch, completion and programme decisions.

The [complete-prefix plan](dialogue-prefix-learning-plan-2026-09-27.md) and
[executed two-update integration](dialogue-prefix-implementation-2026-09-27.md)
establish the mechanism and usable learning/reload path. The retained
[R1d replay](dialogue-artifact-replay-2026-09-27.md) remains weak dialogue evidence.
The [reader comparison](radial-adaptation-result-2026-09-27.md) remains a scoped
negative for its transferred Lorentz and affine configurations; this study
neither retries those readers nor changes the native multiplier-free target.

## Question and fixed intervention

Does restoring the complete original request and history during response
learning improve useful replies beyond both the retained parent and learning
the same responses from the Assistant role alone? Both arms use the same
selected response IDs, complete response/EOS targets, starting parameters,
optimizer, counter schedule and update dose. Prefix content, history length
and physical response positions change together. The comparison identifies
this complete-prefix training package on the selected short population, not
content-only causality or the cause of every historical dialogue failure.

| Condition | Exact presentation before causal shift |
|---|---|
| `full_prefix` | Original document BOS through the selected complete response and genuine EOS, including request/history, original separators and Assistant marker. |
| `role_only` | Original BOS, the actual stored Assistant marker, then the identical selected response and genuine EOS. |

The bound literal-role protocol supplies the marker IDs; response text is never
re-encoded. Prior assistant responses are context with zero loss. Only the
selected response and its real EOS receive loss, with right-padding after the
last real target. No response is cropped, left-padded or packed across a
document boundary. Eligibility is **at most 256 total episode IDs**, distinct
from 257 stored IDs for a 256-position historical shifted window. Tensor time
remains 256, including zero-weight trailing padding.

The strict import reproduces 129,486 training response runs, of which 14,826 are
eligible, with 1,048,098 eligible response/EOS tokens and no identical full/role
prefixes. Its six eligible source counts are 404 / 6,037 / 85 / 401 / 669 /
7,230. Everyday Conversations and Constraints supply 89.4847% of eligible
response runs; this is not their supervised-token share. The training sampler
keeps this population unchanged and draws responses uniformly with replacement.
The study does not repair or evaluate the excluded long conversations.

## Bound parent, source and campaigns

Both arms independently reload the retained R1d at
`/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/r1d/curve-2`:
21 shared arrays, 5,429,826 F32 parameters, parameter SHA256
`95e3fbb06cb39b4354bd40c722873dac088d47554707777b291e588922a1a822`.
They start with fresh Adam and local step zero, not from either two-update
integration descendant or an unavailable historical optimizer.

The executed study source is
[`3c43c6aa21631c5f335f74c287ac19a7000f32a6`](https://github.com/UOR-Foundation/uor-r4/commit/3c43c6aa21631c5f335f74c287ac19a7000f32a6).
The CPU executable is compiled in **release with `cpu-accelerate`**, SHA256
`c283aec4d733493ba9bfe9d2e5409abd0a8db88fa24641433168e70c4f810e8c`.
The [driver](https://github.com/UOR-Foundation/uor-r4/blob/3c43c6aa21631c5f335f74c287ac19a7000f32a6/crates/uor-r4-training/src/dialogue_learning.rs)
and [development helper](https://github.com/UOR-Foundation/uor-r4/blob/3c43c6aa21631c5f335f74c287ac19a7000f32a6/crates/uor-r4-training/src/dialogue_development.rs)
bind the imported inventory, exact panel, executable and campaign before fitting.
Preparation seals and verifies its complete report root; each fit requires the
frozen panel hash and exact equality with its regenerated selection metadata.

The investigation root, abbreviated `R` below, is
`/Users/casey.allard/uor-r4/.uor-models/investigations/fourth-research-lab-20260926`.
The sealed campaign inputs are `R/dialogue-study-inputs-1/full_prefix.json` and
`role_only.json`, respectively SHA256
`b30eca1da3cf9f564f45ee9d2a66b88eb7c5f843b9978209b123436e737b409d`
and `7d206c61803929d3098a94768d3313e38d2eeae35a22ced1157388f4051f8c7d`.
The prepared manifest SHA256 is
`a66d52473cac24b28cc09a751ede7b680c40e7b56218e9074e246cf4fd1a99a5`;
tokenizer SHA256 is
`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
The shared literal-role protocol identity is
`blake3:0099a613c8fcffc78210ed7b7387841917472d976f307a33526c8424ecf5d327`.

| Fixed learning condition | Value |
|---|---|
| Native model | Quaternion / Dot read / Full causal access |
| State width / read coordinate width | 576 / 64; read width is not a 64-token memory limit |
| Context / tensor time / batch | 256 / 256 / 16 |
| CPU gradient shards | 2 whole-sequence shards |
| Common training seed | 20260929 |
| Endpoint | 1,024 updates per arm; only this endpoint is the selected candidate |
| Nominal work per arm | 16,384 response visits; 4,194,304 padded tensor positions |
| Adam configuration | Learning rate 0.0001; weight decay 0.01; parameter absolute limit 1,000,000; no allowed missing gradients |
| Objective | Global mean over selected response and genuine EOS target visits; no prompt-token loss or end weighting |
| Development observations | Local steps 0 / 256 / 512 / 768 / 1,024 |
| Checkpoint role | Step 512 for recovery; final 1,024 for the comparison |

Response visits may repeat IDs. Actual supervised-token exposure, real prefix
positions, padding, per-source visits and schedule-chain/target hashes are
recorded separately. Longer sampled responses contribute more loss tokens;
this is not an average of per-response losses. Whole-row shard gradients are
combined by their actual supervised-target counts before the optimizer update.
Development observations are descriptive: they do not select an earlier
checkpoint, a different dose or a replacement panel.

## Frozen open development panel

Preparation selects up to 32 eligible responses per source without replacement,
using seed **20260930** and domain-separated SHA256 of seed, source index and
response ID, with ID tie ordering. Short sources contribute all eligible
responses. The exact ordered IDs and selection identities are retained in
`R/dialogue-study-panel-1/development-panel.json`, SHA256
`8d53682a3e45d26a0609c8d8989698aef40faa33e36936237430c3bfb423a3dc`.
The companion `development-episodes.json` retains original document/response
spans and source labels; the following denominators are derived directly from
those selected spans, without evaluating a model.

| Source family | Eligible development responses | Selected | Response/EOS targets | First-four targets |
|---|---:|---:|---:|---:|
| Smol Magpie Ultra | 23 | 23 | 3,266 | 92 |
| Smol Constraints | 597 | 32 | 3,811 | 128 |
| Smol Rewrite | 10 | 10 | 769 | 40 |
| Smol Summarize | 39 | 32 | 1,325 | 128 |
| UltraChat | 63 | 32 | 3,233 | 128 |
| Everyday Conversations | 382 | 32 | 949 | 128 |
| **Total** | **1,114** | **161** | **13,353** | **644** |

Each selected response supplies one genuine EOS target: 161 per evaluation.
Both arms are evaluated under the **full original development prefixes**, with
batch 4, response-token mean NLL and first-four-response-target mean NLL. The
first-four definition includes EOS if it occurs within those positions. Reports
retain each source's response, target, first-four and EOS denominators as well
as the pooled result. F32 masked reductions are accumulated by actual target
counts in F64; bitwise equivalence to one F32 reduction is not claimed.

The pooled score is this selected panel's token mean, not the original corpus
mixture or an equal-source average. These are already exposed development
records from a directory named `heldout`; that name does not make their repeated
use fresh held-out evidence. The small 23- and 10-response strata are complete
eligible strata. The separate fresh request panel remains closed.

## Actual replies and independent decisions

Each final checkpoint must be saved, strictly reloaded and used to generate
**58 actual replies across the same 38 existing requests**: five greeting,
eight factual, seven instruction, ten multi-turn memory and eight refusal
cases. Generation remains greedy with a 32-selected-token cap, the bound
protocol and exact generated token history. Non-EOS stops receive an explicit
caller EOS only when needed before the next user turn. There is no decoder
sweep. These endpoint replies are pending at this launch snapshot.

Reuse the sealed parent's existing 58-response replay. Its executable differs
from the study executable, so it is compatible-path baseline evidence rather
than a same-executable bitwise replay. Join parent and both endpoints by request
ID, category, turn, user request, tokenizer/protocol and decoding policy.
After the first reply, generated histories can differ; review complete memory
trajectories and retain each arm's actual assistant IDs. The 58 turns are not
independent trials, and an earlier incorrect assistant claim does not replace
the user's stated facts as the correctness reference.

Independent review records request fulfillment, factual/history correctness
where applicable, intelligibility, obstructive repetition or distraction,
payload completion, exact stop, and a short output-supported interpretation.
Preserve concrete gains, losses, unchanged failures and material reviewer
disagreements against **both** parent and role-only. EOS alone is not success;
a token cap alone is not failure. Fluent generic assistance is not a correct
factual answer, and greeting improvements cannot average away new memory errors.

| Observed endpoint outcome | Programme decision |
|---|---|
| Full-prefix gains useful request/history-correct output over both parent and control, with supportive conditional evidence and no offsetting material retention loss | Retain a candidate at this population and dose; any further fit needs a distinct causal/resource decision. |
| Both arms improve similarly over parent | Retain common adaptation/selected-population benefit without attributing it to complete-prefix conditioning. |
| Development loss improves without useful output, or the gap only reflects damage to role-only while full-prefix fails to improve over parent | No useful-conditioning promotion; preserve both artifacts and stop at the fixed dose. |
| Both remain weak or observed differences are not useful | Preserve the scoped negative; no universal capacity verdict or automatic longer fit, seed repetition or decoder sweep. |
| Material correctness/retention harm or reverse effect | Decline promotion or explicitly narrow any retained use; choose a next mechanism from the actual divergence. |
| Binding failure or incomplete fixed endpoint | Report unavailable/incomplete evidence and retained recovery state, not model ineffectiveness. |

No invented percentage gate is added. The prepared panel, successful build,
earlier gradient checks and two-update witnesses are not useful-chat results.
Width-576 F32 learning does not establish integer serving, geometric advantage,
multiplier-free speed/energy savings, or broad coding/reasoning capability.
Other labs retain their architecture, trainer and CLI ownership.

## Complete cost and execution limits

`R/projection-dialogue-prefix-study.json` reserves **eight hours for the whole
phase**, including implementation/build, preparation, both sequential fits,
five development evaluations per arm, checkpoints, reload, all 116 endpoint
replies, independent review and protected delivery. Per-fit new-update admission
stops at 12,600 seconds including preparation/evaluation; the external hard
process limit is 14,400 seconds. Phase/cumulative ceilings and a 1,800-second
closeout margin take precedence. Recovery preserves the exact source/campaign,
Adam state and next data step; it is not a fresh parent restart or favorable
checkpoint selection.

CPU execution permits one owned model process and one Cargo process, Cargo jobs
2, Rayon 2, and BLAS/OMP/GEMM threads 1. Model RSS is capped at 6 GiB; build RSS
at 4 GiB. Launch requires 25% free memory; the stop threshold is 10%. New internal
material is capped at 1 GiB, shared SSD growth at 2 GiB, while preserving
25,971,130,368 physical bytes and the additional 128 MiB stop margin. No paid
compute or deletion of unique evidence is authorized by this study.

Historical 8.90-second updates project about 5.06 hours of paired update work.
The earlier four integration updates measured 4.39–5.70 seconds each, with
4,767,744,000-byte peak child RSS; neither short observations nor startup time
promise sustained throughput. Masked targets do not remove padded recurrent
work. Four midpoint/final model-plus-Adam payloads are projected at about
261 MB; the envelope also includes logs, reports and recovery overhead.

The source-bound build receipt records three focused development tests and the
release example build, both exit 0, taking **56.360 seconds** combined with
824,164,352-byte maximum child RSS. Data-only preparation finished at
08:49:18 UTC, with 4.679 seconds internally, **6.085 seconds of supervisor wall
time**, and 340,508,672-byte maximum child RSS. These executed costs are separate
from pending model work; no duplicate model witness or broad suite is required.

The prospectively recorded standing local extension raises this lab's owned
elapsed ceiling from 16 to 18 hours, adding two hours of closeout/recovery reserve.
The verified shared allowance remains 722,400,000 ms; the foreign-reported
744,000,000 ms is not adopted. Charge preparation, review, build, fit, evaluation,
retries and delivery as overlapping elapsed once from the retained ledger
cursor, not by summing overlapping subprocess durations. Preserve all prior
negative candidates, sealed roots and the R1d parent regardless of outcome.

At the 08:53:07 UTC accounting cutoff, owned cumulative elapsed is 30,003,414 ms
and the shared charged total is 678,158,994 ms. The new 1,304,074 ms increment
includes preparation, review, build and the overlapping live fit through that
cutoff, with no foreign increment observed. The ongoing tail remains open.
Physical free storage is 32,396,988,416 bytes internally and 172,545,351,680 bytes
on the SSD at that same receipt; these are observations, not retained artifact
sizes or a completed-phase cost.
