# End-to-end R4/Spin causal-softmax language model (#1014)

- **Status:** `ATTENTION_ESTABLISHED / FULL_QUALITY_DOD_FAILED`.
- **Owner:** #1014 under attention issue #973 and programme root #820.
- **Implementation revisions:** attention-output policy
  `14a4b47f570f757c2492a3e7272e14e94e8c26de`; Rust local generation
  `abe2ad334a39c434f30a4cc46aa9f7a0d954fa7e`; isolated trainer
  `ab8644fca275c1b55f4f84003931bec2d649518a`.
- **Structured aggregate:**
  [`r4_softmax_end_to_end_attention_1014_raw.json`](r4_softmax_end_to_end_attention_1014_raw.json).
- **Binding local evidence root:**
  `.uor-models/research/issue-1014/` (ignored bulk data, checkpoints, reveal,
  five generation reports, and five replay reports).

## Result and decision

The one frozen campaign established that ordinary causal attention is
load-bearing in a learned language model executed through coherent R4/Spin
coordinates. On the same 249,856 sealed next-token positions, enabled attention
scored `2.127407277216677` nats/token while zeroing every attention output after
`W_o` and before the residual scored `4.804799838144271`. The intervention
penalty was `2.6773925609275944` nats/token, far above the predeclared `0.10`
minimum. This is direct held-out evidence for learned ordinary causal attention,
not merely R4 activity or donor equivalence.

The full #1014 Definition of Done is nevertheless negative. Enabled sealed-test
NLL missed its `<= 1.50` quality ceiling, and only prompts 1, 2, and 4 retained
their prompt subject or scene: `3/5` against the required `4/5`. All other
declared gates passed, including Rust/Python parity for both enabled and
attention-off arms, exact causal/R4 audits, zero future/provider/Ollama/trace
reads, valid UTF-8, no period-one-through-four loop, and exact `5/5` seeded
replay.

Therefore this exact campaign closes negative without a rerun, hyperparameter
tuning, or another diagnostic ladder. Attention is no longer the immediate
unknown; dependable coherent language quality is. The smallest next step is
[#1017](https://github.com/UOR-Foundation/uor-r4/issues/1017), a separately
frozen quality-capacity rung that reuses this exact attention,
split/tokenizer discipline, Rust generation path, and replay contract while
changing only training exposure: the next `119,996,416` fresh deterministic
training tokens plus fresh development and sealed-confirmation tranches. It
must treat the attention-off
question as closed by this result rather than rerunning that intervention. It
must not reactivate intrinsic/readout comparisons, resonance, softmax
replacement, recurrence, or exact lowering before coherent quality qualifies.

## Frozen model and population

The model was trained from random initialization with no teacher logits,
source-model weights, trace artifact, suffix table, Ollama, or hosted inference:

- 7,155,360 learned parameters;
- 4,096-token train-only byte-level BPE;
- width 288; six layers; six Q heads and six KV heads;
- head width 48, exactly twelve R4 blocks per head;
- FFN width 768; context 256; tied embedding and LM head;
- RMSNorm, RoPE, SwiGLU, learned Q/K/V/O, complete-prefix scaled dot product,
  stable causal softmax, and weighted-value aggregation.

The pinned TinyStories snapshot is revision
`f54c09fd23315a6f9c86f9dc80f725de7d8f9c64`, 2,227,753,162 bytes, SHA-256
`6418d412de72888f52b5142c761ac21a582f7d1166f0bfbdb5f03ccfdec90443`.
All 2,717,495 stories were assigned before tokenization by the full
`BLAKE3(story_bytes) mod 100`: train buckets `0..89`, development `90..94`,
sealed test `95..99`. The tokenizer saw training stories only. The prepared
stores contain 30,000,000 train tokens, 250,000 development tokens, 249,880
sealed scored-store tokens, and five 24-token prompts, for exactly 250,000
revealed test token IDs.

Key identities:

| Artifact | CID |
|---|---|
| Dataset manifest | `blake3:3e4d2ddb006771e5be0d4c76580c8971e6c67a23f8e223da8d81668d03bd9a01` |
| Split policy | `blake3:54f0886d3e906a4aeeaa9328ff236440d61d9f16b2f92dcb8c05cac96e54d1aa` |
| Training view | `blake3:8be705ce259eeb6ff91942422465e8aaa6796f6e8b70f9a9f96a10e1c8a5848b` |
| Run contract | `blake3:608005f95c12f3674bda6aead92b154db6d7e081b01bd4092636afb183b9aff4` |
| Training result | `blake3:4fc19989763194cd02e7b5cb84d1e23b1f8c4a8d2ced452529dde55baa62ed8a` |
| Selection manifest | `blake3:ea383bcea6b1d725a922cfad6d8f1b5f4fe06c3b47234e32e5e6aec5e5b8784f` |
| Selected checkpoint | `blake3:9c36e109d8dee67deec0362307ba4a471c967ff574835210f87653d628c95c91` |
| Exported weights | `blake3:7d7c26e1a71866dc46973cea3b23b819f4b5060b345d2a0ec1bd067aa493bb7d` |
| Tokenizer | `blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc` |
| Export manifest | `blake3:a2d30d87a2d5ea1065ecddb14b93f67effb64f19ead474559e4f42ad5be78b2f` |
| Reveal manifest | `blake3:53371f3ccd86e32e00e4cd2078010492e87b2304427e7acfaadad6136c11493b` |
| Reveal result | `blake3:d9b074a00647f9b6d493f2b0e34d5c66d6b03e6e061d8e31de61cc2a75ded6fb` |

## Admission and training

The only pre-run gates were implementation admission, not a comparative
research campaign:

| Gate | Frozen rule | Result |
|---|---:|---:|
| 64-sequence MPS overfit | loss reduction `>= 80%` within 300 s | **PASS**: `8.412553071975708 -> 1.6134384125471115`, `80.821061%`, 132.36 s |
| Smoke Python/Rust enabled parity | same top-1 and maximum logit delta `<= 0.005` | **PASS**: `0.0000171661376953125` |
| Smoke Python/Rust attention-off parity | same top-1 and maximum logit delta `<= 0.005` | **PASS**: `0.00001239776611328125` |
| Main campaign | one run, `<= 30M` tokens and `<= 6 h`, MPS only | **PASS**: 29,999,104 tokens, 1,831 steps, 4,220.42 s |
| Checkpoint selection | minimum complete-development-token NLL before test reveal | step 1,831, dev NLL `2.131356526593693`; test still `UNOPENED` |

There was no hyperparameter sweep. The final checkpoint and export were frozen
before the sealed test store was opened.

## Sealed attention and quality verdict

| Criterion | Frozen rule | Observed | Verdict |
|---|---:|---:|---|
| Enabled sealed-test NLL | `<= 1.50` | `2.127407277216677` | **FAIL** |
| Attention-off penalty | `>= 0.10` nats/token | `4.804799838144271 - 2.127407277216677 = 2.6773925609275944` | **PASS** |
| All learned layers through coherent R4/Spin transport | all six | six | **PASS** |
| Exact causal work | zero future reads | zero | **PASS** |
| External/source closure | zero teacher, provider, Ollama, or prior-trace calls | zero | **PASS** |
| Autonomous continuations | archive five | five | **PASS** |
| Prompt subject or scene | at least `4/5` | `3/5` | **FAIL** |
| Decode integrity | valid UTF-8; no period-1..4 loop | `5/5`; zero short loops | **PASS** |
| Reload replay | reproduce selected seeded generation | exact `5/5` | **PASS** |

The binding Python reveal terminal is `FAIL_PYTHON_REVEAL`. That terminal is a
quality-gate failure, not a failure to establish attention: the independently
declared attention-off intervention passed by 2.677393 nats/token.

## Rust parity and runtime audit

The final 32-token qualifier loaded the exact exported checkpoint and ran both
policies through every learned layer. Enabled Python and Rust selected token
`269`, with maximum absolute logit delta `0.00000762939453125`. Attention-off
Python and Rust selected token `16`, with delta `0.00001239776611328125`.
Both are inside the frozen `0.005` tolerance.

The qualifier recorded all six layers, exact causal, projection, R4, and
output-policy audits, zero future reads, zero provider/Ollama/prior-trace calls,
and an unchanged checkpoint tree. Its decision CID is
`blake3:24d64c078318b10800a2e108ddd3a57dd7e68fba2fdd4f1f097116a3d62ae9d2`.

## Exact autonomous outputs and frozen rubric

The sampler was
`r4-local-top-k-q32-splitmix64/1;temperature=0.8;top-k=40;rank=logit-desc-token-asc`.
The rubric asks only whether the continuation retains the prompt subject or
scene; it is not a grammar, correctness, or general-quality claim.

### Prompt 0 / seed 1014 — FAIL

Prompt:

> One day, a little cricket wanted to play. He was a healthy cricket who loved
> to jump and balance on things.

Exact response:

> The big lizard had a big smile on his face and wanted to show off.
> The little boy tried to fly fast, but he was too fast. He tried all the way
> to the little girl. So, he decided to go on an adventure. He walked and
> walked.
> As he got to the big tree, he started to rain. The rain was hot, but the
> little boy was upset. He didn't want to play, but the rain did not fall.
> The next day, while the little boy was not happy. He ran back to his friends
> and said, "We are sorry we should not have listened."

The continuation replaces the cricket and its play/balance scene with a lizard
and a little boy. Output CID:
`blake3:3ae15af09ffa19ecd75097e8f3c2315155e058c134f6df72039b393802ad5dab`.

### Prompt 1 / seed 1015 — PASS

Prompt:

> Once upon a time, there was a little boy named Tim. Tim was a bit selfish. He
> did not like to

Exact response:

> be sad. One day, his mom had a key for him. She had to go to Tim's house with
> a big gift.
> Tim said to his mom, "Mom, I'm hungry. Can you help me?" His mom said, "I'm
> sorry, Tim. I thought you could help me. I promise my new toy and I'll be your
> friend."
> Tim was happy to help. He took his toy. He showed his big red toy to his mom.
> After a long day, Tim played with his new toy. He was very good at carrying
> things.
> When he was done, his mom was

The continuation retains Tim and his immediate personal/family scene. Output
CID: `blake3:9fd81e5331f5addf9a0eb005d6419bae339225a030f799b76bcbb97e6406ec2b`.

### Prompt 2 / seed 1016 — PASS

Prompt:

> Once upon a time, in a land where the sun always shined, there was a popular
> ice cream shop. This

Exact response:

> toren't love if it was a nice place where people had never been there.
> One day, a little girl named Lucy came to the park. She saw the ice-cream
> truck and thought it was a fun toy. She wanted to be her friend. So, she asked
> the ice cream man if she could have it. The ice creams said yes and said yes.
> The next day, Lucy went to the kitchen. She had to get a big, yummy cookie.
> She put the ice cream on the ice cream and ate it all up. Then, she made a big
> plate of water for the ice cream dog

Despite a malformed first word and weak consistency, the continuation retains
the ice-cream setting. Output CID:
`blake3:afb33a5768ddf21dbdd92328aa9d4b71215a0d172f7389b78d885e5a24115dd0`.

### Prompt 3 / seed 1017 — FAIL

Prompt:

> One day, Grandma and Grandpa were walking around their garden. Grandma said
> to Grandpa, "Look at that old

Exact response:

> people's pretty. We have to show it to those friends."
> Molly was excited and said, "Yes, let's play together!"
> The two kids began to run as fast as they could. Everyone was amazed again,
> but then Mary said, "Me too!"
> When they arrived, they felt so happy they had a new friend. They played with
> the new toys and had a great time together.

The continuation drops Grandma, Grandpa, and the garden scene for unrelated
children and toys. Output CID:
`blake3:09fa7651864669f53c1052ce6d1152a607f53bf5a94be0c44e7a3f483589f159`.

### Prompt 4 / seed 1018 — PASS

Prompt:

> Once upon a time, there was a little boy named Tim. Tim loved to watch
> cartoons on TV. His favorite

Exact response:

> cartoon was his grandma. One day, Tim was playing with his best friend, Sue,
> and her big red coat.
> Tim and Sue played in the big red book. They had so much fun together. But
> then, Tim had to take a nap. He saw Lucy's favorite place. Sue thought, "Maybe
> I can play with a big red ball." Tim was very happy.
> Sue and Tim played with the ball in the car. They laughed and had fun. But
> then, Tim's big sister, Sue, came over. She said, "Oh no! My ball is in your
> bag! Let's play here!"

The continuation retains Tim and begins by completing the cartoon subject
before staying in his play scene. Output CID:
`blake3:c6eda6ff6978802d1fe9a5e2fdc2b1a7187695861e857affff0d8bc53f671bf9`.

All five generation reports selected all six layers and matched their expected
causal, projection, R4, and output-policy work exactly. Every report recorded
zero future reads, zero provider calls, zero Ollama calls, zero prior-trace
reads, valid UTF-8, and no period-one-through-four loop. Prompt 3 stopped on
EOS after 84 generated tokens; the others reached 128. Each replay reproduced
the exact generated token IDs, response text, output CID, audit CID, decision
CID, and persistent-state CID.

## Runtime boundary and nonclaims

Training and qualification intentionally used floating point, multiplication,
allocation, autograd, Apple Metal, and ordinary softmax. The Rust generator is
a local floating-point research runtime. This result does not establish a
geometry advantage over ordinary coordinates, transformerlessness,
multiplication-free or table-native execution, dependable broad generation,
inference, reasoning, chat, scaling, browser-WASM operation, release readiness,
or frontier capability.

The negative quality verdict applies to this exact model, data budget, and
training campaign. It does not falsify ordinary causal attention; the
attention-off intervention establishes the opposite at this bounded scope.
