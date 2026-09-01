# R4 paired-H4 prompt-capacity result (#973)

- **Issue:** [#973](https://github.com/UOR-Foundation/uor-r4/issues/973)
- **Candidate policy:** `R4PairedH4PromptCapacityV1` /
  `R4PairedH4LanguagePathV1`
- **Frozen baseline:** qualified `R4RetainedLanguagePathV1`
- **Terminal:** `PAIRED_H4_PROMPT_CAPACITY_FAIL`
- **Result CID:**
  `blake3:508a4ff352f1e533d669d9616f65b972b0f13e8efe35867b7b095281ad940274`
- **Decision:** preserve V1, do not run another generation smoke, and
  independently freeze the prompt-state-to-logit readout seam

## Decision in one paragraph

The paired-H4 candidate retained the qualified V1 language fit on a fresh
nonoverlapping token slice, but it did not increase prompt-conditioned
capacity on the independently frozen prompt-swap population. Candidate prompt
gain was `0.0062477543` nats per target token versus `0.0063672952` for frozen
V1, a candidate-minus-V1 delta of `-0.0001195409`; the candidate won `282/512`
directions against the required `308/512`. Both state-off arms collapsed to
exactly zero prompt gain and zero paired-logit difference, and replay, causal,
and forbidden-read audits passed. The result rejects this paired-address
capacity seam. It does not revoke the already established ordinary causal
attention result or the qualified V1 retained-attention language path.

## Scope and single changed variable

This rung asked one bounded question: does assigning a separate exact-H4 token
address to each of the two retained decoder layers increase the qualified
cell's ability to preserve which prompt produced a continuation?

For token `t > 0`, the two layer addresses were frozen as

```text
layer 0: (t - 1) mod 120
layer 1: floor((t - 1) / 120) mod 120
```

Token zero remained the exact-H4 identity at canonical index 119. This gives
all 4,096 tokens distinct ordered address pairs. The candidate changed only
`LAYER_PAIRED_EXACT_H4_TOKEN_ADDRESS_ONLY`; it retained V1's learned parameter
names and count, recurrent field shapes, Q/K/V/O projections, decay and
delta-write gates, occupied-slot softmax, read-before-write ordering, training
slice, deterministic order, seed, optimizer dose, and state ledger. The
qualified V1 artifact was evaluated frozen and was not retrained.

The shared ledger was:

| Field | Frozen value |
|---|---:|
| Learned parameters | `252,160` |
| Full-context state | `23,040` f32 values / `92,160` bytes |
| Validity state | `240` bits |
| Optimizer steps | `2,730` |
| Token presentations | `5,241,600` |

## Construction-only address census

Before training or any prompt reveal, the token-only census compared repeated
cumulative prefix addresses on the 43,680 construction windows:

| Addressing | Total repeats | Mean | Median | P95 | Maximum | Collision-free windows |
|---|---:|---:|---:|---:|---:|---:|
| Shared single coordinate | `1,937,864` | `44.3650183` | `44` | `50` | `60` | `0/43,680` |
| Paired coordinates | `47,522` | `1.0879579` | `1` | `3` | `13` | `11,148/43,680` |

This is a `97.5477123%` reduction in repeated joint addresses. The codebook
used 4,096 unique token pairs; coordinate one had direct support 120 and
generated subgroup 120, while coordinate two had direct support 36 and
generated subgroup 120.

That census is construction evidence only. It read token IDs and exact-H4
action tables, not targets, model weights, logits, continuations, or prompt
labels. It therefore shows that the paired construction reduces address
collisions; it does not establish learned prompt capacity, H4 superiority,
attention quality, or an effective 14,400-slot memory. The implementation has
two separate 120-slot retained fields.

## Correct prompt population and rejected provisional scan

The binding `R4RetainedPromptSwapContrastV1` population is:

- population CID
  `blake3:c11a7c935139ca169460b90c01392d7c9e0929e4c10710e76e6c8f74cbdf0340`;
- commitment CID
  `blake3:873e7c9475cd004c77b91899cca0140de4cf77a489c9da1a1d201c68a682010a`;
- reveal CID
  `blake3:a74995831dd3783ca8a3dcf7fe6676fd77ff63822f3604116585451a3b756562`;
- 256 pairs, 512 bidirectional comparisons, and 8,192 scored continuation
  tokens; and
- 48-token prompts paired to share the last four token IDs while differing in
  both complete prompt and 16-token continuation.

The population was selected from TinyStories revision
`f54c09fd23315a6f9c86f9dc80f725de7d8f9c64`, raw-source SHA-256
`6418d412de72888f52b5142c761ac21a582f7d1166f0bfbdb5f03ccfdec90443`,
with tokenizer CID
`blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc`
and split-policy CID
`blake3:54f0886d3e906a4aeeaa9328ff236440d61d9f16b2f92dcb8c05cac96e54d1aa`.
It used strict UTF-8, source ordinals after 72,670, and the development bucket
defined by raw-story BLAKE3 digest modulo 100 in `90..94`. Token IDs 0, 1, and
2 were forbidden in the first 64 tokens. The population remained mode `000`
until both baseline and candidate artifact CIDs were fixed.

The earlier provisional feasibility scan with CID
`blake3:9e041283383713a2ce48037774adb1022f6137d63dedfa4c587bdbee9e9f47c1`
is invalid for evaluation. It omitted canonical whitespace normalization and
mostly overlapped model-training data. It was rejected before the public
freeze, never scored, and never used for model selection. No claim or threshold
in this record depends on it.

The canonical envelopes are the
[population commitment](r4_paired_h4_prompt_capacity_population_commitment_973_raw.json)
and [one-time reveal](r4_paired_h4_prompt_capacity_reveal_973_raw.json).

## Frozen prompt criterion

For each direction `d`, with own prompt `p_d`, paired prompt
`paired_prompt_d`, and 16-token continuation `y_d`, the empirical score was

```text
g_d = (log P(y_d | p_d) - log P(y_d | paired_prompt_d)) / 16
G   = mean(g_d) over 512 directions
```

Qualification required all of the following:

| Gate | Frozen threshold |
|---|---:|
| Candidate absolute mean gain | `G >= 0.0433216988` |
| Candidate-minus-V1 capacity gain | `>= 0.0253415693` nats/token |
| Candidate directional wins | `>= 308/512` |
| Candidate own-prompt NLL | no worse than V1 |
| State-off collapse | absolute residual `<= 1e-7` |
| Integrity | exact replay and zero forbidden reads |

The prompt population was independent of the construction training slice and
the fresh general-language slice. Its role was the terminal capacity decision,
not a post-hoc qualitative generation grade.

## Execution and evidence identities

The create-once preparation and CPU admission probe are preserved as the
[preparation envelope](r4_paired_h4_prompt_capacity_preparation_973_raw.json)
and [probe envelope](r4_paired_h4_prompt_capacity_probe_973_raw.json). Key
identities are:

- frozen implementation commit `8681129d`;
- preparation CID
  `blake3:385dafa8c269133bc5816ba5634c96f7210f8c31ecd6456ccc526dc449244d0b`;
- probe CID
  `blake3:3d897a47757e908ece6767fb044b08f9ba3ea2eca0571d1f0574490f4e4b820a`;
- execution-plan CID
  `blake3:16afa1dc185ad0451cf8f0a9927fe7ca00d1209d73b120e114dd28c7a082e61d`;
- started CID
  `blake3:c5f9adccfff554370b8331f4382f767033f52a7aa8c3d0d6e358b946a7988bbd`;
- run-contract CID
  `blake3:ded954f6ed54183d86843d3ea3a26a0d6c5564a3217832be5745da0bcde4ab94`;
- implementation-tree CID
  `blake3:5ab122985b7c539fb56dcbfb7e141a25e91509a9d9db8ffd5b22633135778e64`;
- frozen V1 artifact CID
  `blake3:d1417b325e7a545057cd38e9f1a723933a3682801877433d20e98774a5e9172d`;
  and
- candidate artifact CID
  `blake3:7c5c03a48c56f9a500b6a83a135d626d208580719e20739b137358d95aca7a09`.

The candidate artifact was 1,010,792 bytes and was fixed before prompt reveal.
The selected plan was one CPU process using Apple Accelerate and four threads;
CUDA was forbidden and MPS was not used. The five-step probe measured
`0.4270680333 s/step`, projected `1,600.231385 s` under the 3,000-second
projection ceiling, and used `1,177,354,240` of `17,179,869,184` budgeted
bytes. The run completed all 2,730 steps in `1,245.326918 s`; prompt evaluation
took `33.232104 s`, below its 300-second ceiling.

Preflight and terminal mechanics passed. The shared-prefix causal delta and
forbidden reads were zero. All 24 required learned tensors had finite nonzero
gradients. Artifact reload had zero maximum logit delta; the terminal direct
64-token replay delta was `0.0000104904`.

The immutable [started envelope](r4_paired_h4_prompt_capacity_started_973_raw.json)
and [result envelope](r4_paired_h4_prompt_capacity_result_973_raw.json) bind the
execution and terminal metrics.

## Fresh general-language result

The fresh slice had CID
`blake3:9de081d4a639dfebe885854ce7fbf850fe9a0b0a658307d88e3f9b49d579024d`,
249,986 tokens, 2,066 windows, and 247,920 decisions. It begins at source story
760,379, immediately after predecessor overlap ends at story 760,378.

| Arm | NLL (nats/token) | Top-1 |
|---|---:|---:|
| Paired-H4 candidate | `3.8832293739` | `0.2978017102` (`73,831/247,920`) |
| Frozen V1 | `3.8901151940` | `0.2970635689` (`73,648/247,920`) |

The candidate improved by `0.0068858201` nats and `0.0738141` percentage
points relative to V1, so every frozen fresh-language non-regression gate
passed. This qualifies the candidate's general-language fit only. It cannot
substitute for the independently frozen prompt-capacity test.

## Terminal prompt-capacity result

| Arm | Own NLL | Crossed NLL | Mean gain `G` | Direction wins |
|---|---:|---:|---:|---:|
| Frozen V1 | `3.6822566744` | `3.6886239696` | `0.0063672952` | `300/512` |
| Paired-H4 candidate | `3.6762286924` | `3.6824764467` | `0.0062477543` | `282/512` |
| Frozen V1, state off | `4.0539008513` | `4.0539008513` | `0` | `0/512` |
| Candidate, state off | `4.0504785991` | `4.0504785991` | `0` | `0/512` |

Candidate own-prompt NLL was better than V1, but prompt discrimination did not
reach the frozen absolute threshold and did not improve over V1. Its
candidate-minus-V1 gain was `-0.0001195409`; it also missed the directional-win
threshold by 26 directions. The candidate's maximum paired-prompt logit delta
was `3.271856308`, versus `4.071191788` for V1. Both enabled arms therefore
responded to prompt changes, but that sensitivity was small under the frozen
criterion and the paired-address change did not increase it.

Both state-off arms collapsed to exactly identical own/crossed NLL, zero mean
gain, zero wins, and zero maximum paired-logit delta. Baseline and candidate
replay passed, and all four prompt-score arms recorded zero forbidden reads.
The binding prompt verdict is `PROMPT_CONDITIONING_CAPACITY_FAIL`, yielding the
campaign terminal `PAIRED_H4_PROMPT_CAPACITY_FAIL`.

## Interpretation and next action

The evidence supports three narrow conclusions:

1. The paired exact-H4 radix construction substantially reduces cumulative
   address collisions on the construction token windows.
2. The trained candidate preserves and slightly improves fresh-slice language
   fit relative to qualified V1.
3. The same change does not improve prompt-conditioned continuation scoring on
   the independent frozen population.

Accordingly, the paired-address candidate is not promoted. Qualified V1 remains
the retained-language baseline. There is no generation retry, parameter sweep,
corpus expansion, or H4-specific claim from this rung. The next independently
frozen #973 mechanism is the prompt-state-to-logit readout seam: test whether
the existing causal retained state can be exposed to candidate logits more
effectively without changing the qualified V1 recurrence or reopening this
population for model selection.

Issue #973 remains open because prompt-conditioned generative behavior is not
yet qualified. Issue #954 remains blocked; C1-SB6 is not authorized. Reasoning,
correctness, H4-specific superiority, exact/table lowering, browser/WASM
integration, and release readiness remain `NOT_RUN`, `NOT_EVALUATED`, or
unestablished as applicable. Ordinary causal attention remains established by
the separate #1014 intervention, and the V1 retained-language result remains
qualified at its recorded scope.

## Readout-seam successor result — 2026-09-01

The independently frozen successor named above completed as
`R4DirectRetainedReadoutLanguagePathV1`. It restored V1's original exact-H4
addresses and recurrence and varied only whether the already-computed retained
layer outputs reached the tied language head. On a new story-disjoint V2 prompt
population, fixed `g=1` raised mean prompt gain to `0.0215897894` from the
matched `g=0`/V1 value `0.0076304198`, with `343/512` versus `313/512` wins.
Fresh held-out NLL/top-1 also improved to `3.7374367989` / `31.542433%` from
`3.9010778353` / `29.632946%`; state removal cost `1.1234286047` nats.

This cleanly localizes useful prompt information at the readout seam, but the
candidate missed the frozen absolute and incremental prompt-gain floors. Its
terminal is `DIRECT_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL`, result CID
`blake3:71dd85e610dcc50b74cb2bb2068e5a1a433ac5df5db2a4f8fde22fb41735889c`.
Generation, retry, widening, lowering, and gain tuning remain forbidden.

The only fresh successor authorized at that checkpoint normalized the two
retained layer outputs separately
before their fixed variance-preserving `1/sqrt(2)` sum. It introduces no
learned parameter or state and keeps recurrence, data, optimizer dose, work,
and gates fixed. It must use a V3 population and fresh held-out slice disjoint
from every V1/V2 scored item. A miss would end the parameter-free readout ladder.
See the
[direct-readout record](r4_direct_retained_readout_prompt_capacity_973.md).

## Final readout-ladder outcome — 2026-09-01

The layerwise-normalized successor after the direct-readout `PARTIAL` has now
completed. It restored the same qualified V1 substrate and varied only
normalization placement at the retained-state-to-logit seam. On disjoint V3
prompts it reached gain `0.0286980210` versus V1 `0.0073316237`, delta
`0.0213663973`, and `339/512` wins. Every fresh-language and mechanics gate
passed, including a `1.3495375637`-nat / `20,595`-decision state-off cost, but
the absolute and incremental prompt-gain floors did not.

Terminal:
`LAYERWISE_NORMALIZED_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL`. Result CID:
`blake3:35396bd6e64fc2c0bc7d86a84cc9e212ed913ce28e5353f5f2b8212b4cf2c532`.
Independent verification CID:
`blake3:3f316541dbab8061ed5ba891bf6a47ef22c55bca21fba01f6f97dbb3cb8497aa`.

The parameter-free readout ladder therefore ends. Qualified V1 remains the
baseline; the paired, direct, and layerwise artifacts remain bounded evidence,
not promotion to generation or exact runtime. The sole #973 successor is a
freshly frozen learned associative binding/readout. Generation, reasoning, and
lowering for this candidate remain `NOT_RUN`; #954 stays blocked and C1-SB6
remains unauthorized.
