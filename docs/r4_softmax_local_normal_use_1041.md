# #1041 normal-use scope decision for the frozen #1017 surface

- **Issue:** [#1041](https://github.com/UOR-Foundation/uor-r4/issues/1041)
- **Parent:** [#973](https://github.com/UOR-Foundation/uor-r4/issues/973)
- **Date:** 2026-09-01
- **Implementation revision:**
  `c6e3d95ee39fbb4797b0c8d51507eb5fef978a4a`
- **Result:** `KEEP_RAW_CONTINUATION_ONLY`

## Decision being made

[#1039](r4_softmax_local_reference_surface_1039.md) exposed the immutable
#1017 checkpoint through an opt-in loopback endpoint and the native-served
dashboard. It deliberately left the next product decision to actual use of
that surface. #1041 made that decision without training, prompt tuning, a new
model, or a new inference harness.

The prompts and structural predicates below were frozen before generation.
The predeclared branches were:

- `ADD_SOURCE_BACKED_MULTITURN_ADAPTER` if at least two of three narrative
  probes passed and both history arms recovered their supplied binding while
  their no-history controls did not;
- `KEEP_RAW_CONTINUATION_ONLY` if narrative continuation passed at least two
  of three but either context-binding comparison failed;
- `STOP_FURTHER_PRODUCTIZATION` if narrative continuation passed fewer than
  two of three or any model response failed a mechanical validity condition;
  and
- `REPAIR_OBSERVED_SURFACE_DEFECT` only if the dashboard or transport prevented
  a behavior decision.

Every request reached the existing #1039 endpoint. No committed evaluation
harness was needed because the endpoint already returns the decoded text,
model shape, exact-backend provenance, attention/decode/source audits, and
content identifiers needed for the decision.

## Request-horizon clarification

The frozen export config declares `max_position_embeddings = 256`. The
response field `model_shape.sequence_capacity` is not that checkpoint maximum;
it is the state horizon allocated for the current request:

```text
input_tokens + requested max_tokens
```

An initial reading incorrectly treated N1's 64-token request horizon as the
checkpoint limit and temporarily shortened N3, H1, and H2. Code and config
inspection corrected that interpretation. The live issue restored its original
frozen `max_tokens = 32` rule, and N3, H1, and H2 were rerun with 32 tokens.
Those shorter exploratory observations are superseded and are not part of the
canonical seven-row ledger below. No prompt, predicate, or decision branch
changed.

## Native dashboard observation

The release server ran on loopback with the immutable #1017 export, four
exact-executor workers, and the Apple Accelerate CPU backend. `/api/sysinfo`
reported `enabled = true`, `checkpoint_preflight_ready = true`, attention on,
greedy decoding, an eight-token default, a 32-token request maximum, and no
static-WASM execution.

The actual dashboard offered and selected **R4/Spin Local #1017 — bounded raw
completion**. Its visible status said **#1017 bounded raw completion (no chat
memory)** and its explanatory copy said that only the current prompt and token
cap are sent. Submitting N1 through that UI used the endpoint default of eight
tokens and displayed:

> made a loud noise. Elk

The dashboard reported `0.1 s`, valid UTF-8, no short cycle, exact attention
audits, zero provider/Ollama calls, and the following response identities:

- decision:
  `blake3:3e538fa60e523320483aa7b8903593366b92eeda2bc8d3d82251faaf111fe105`
- output:
  `blake3:390fd4a68657cb1c41ceb7cd94f2122fe1518ccfb9a63511a358dd107e28bfbb`
- audit:
  `blake3:dfdd822a9d4d8ab980abf55d3e8d54a507e756c7522534cbf22396a9eb4305bc`
- persistent state:
  `blake3:2c892a9a1647427abcc84476bc459ba5bb05387ffc85f9569e8cb73f3e62f144`

This establishes the UI/transport precondition only. The decision uses the
capacity-respecting direct endpoint observations below.

## Frozen prompts and observed behavior

Before the #1041 contract was published, the exact N1, N2, N3, H1, and H2
prompts and their defining phrases were absent from the complete pinned
TinyStories source and pre-existing repository records. This excludes the
contract and evidence record that necessarily preserve the prompts. It is
exact-string and defining-phrase freshness, not a claim that generic story or
question patterns were semantically unprecedented.

### N1 — causal scene: `PASS`

Prompt:

> At sunrise, Elka placed a copper bell beside the blue gate. When the wind
> began to blow, the bell

Observed continuation:

> made a loud noise. Elka was so excited that she ran to the gate and opened
> it. Inside, she found a beautiful garden full of flowers

The combined first sentence keeps the bell/wind scene and plausibly completes
the pending clause. It passes the frozen N1 predicate.

### N2 — entity/object binding: `FAIL`

Prompt:

> Milo carried the yellow kite, while Sora held the wooden boat. At the pond,
> Sora carefully set down

Observed continuation:

> the yellow kite and waved goodbye to the yellow kite.
> Milo was so happy that he had found the yellow kite. He ran to the pond and

The first clause swaps the frozen `Sora -> boat` and `Milo -> kite` binding by
having Sora set down the yellow kite. It fails the frozen N2 predicate.

### N3 — dark-lantern scene: `PASS`

Prompt:

> One rainy afternoon, Tessa carried a red lantern into the quiet cave. The
> lantern suddenly went dark, so Tessa reached

Observed continuation:

> out to grab it. She pulled and pulled, but the lanana was too strong. Tessa
> was so sad that she started to cry.

The frozen predicate concerns the first completed sentence. That sentence
keeps Tessa as the actor and completes `reached` as an immediate response to
the dark lantern. The later malformed `lanana` shows quality degradation but
does not alter the predeclared first-sentence verdict.

### H1 — natural-dialogue binding: `FAIL`

History prompt:

> Nia whispered, "I put the silver spoon inside the green box." Later Oren
> asked, "Where is the silver spoon?" Nia answered, "

Observed continuation:

> I don't know, but let's look for it together."
> They looked and looked, but they could not find the silver spoon. Then, they

No-history control:

> Oren asked, "Where is the silver spoon?" Nia answered, "

Observed control continuation:

> I don't know, let's look for it together."
> They looked all around the house, but they couldn't find the silver spoon.
> Then,

The control does not invent the bound location, as required, but the history
arm also fails to return `green box`. H1 therefore fails.

### H2 — literal role transcript: `FAIL`

History prompt:

```text
User: I put the blue marble under the wooden chair.
Assistant: I will remember that.
User: Where is the blue marble?
Assistant:
```

Observed continuation:

> "I'm going to find it!"
> The blue marble was excited. She ran to the blue marble and started to look
> for it.
> She looked around

No-history control:

```text
User: Where is the blue marble?
Assistant:
```

Observed control continuation:

> "I'm going to get it!"
> The blue marble was so excited. She quickly ran to the door and grabbed the
> blue marble. She ran to the

The control does not supply `under the wooden chair`, but neither does the
history arm; the history arm also turns the marble into an acting character.
H2 therefore fails.

## Direct-endpoint evidence ledger

Every row below selected Apple Accelerate, reported four effective
exact-executor workers, remained within the configured 256-token checkpoint
context, and reported a request horizon equal to `input_tokens + 32`. Every row
passed the causal, projection, R4, all-layer, zero-future-read, output-policy,
UTF-8, no-short-cycle, zero-provider, and zero-Ollama checks.

| Probe | Input + generated | Total seconds | Decision CID | Output CID | Audit CID | State CID |
|---|---:|---:|---|---|---|---|
| N1 | 32 + 32 | 0.561831792 | `blake3:ab46c00072d5c6d08764c955b803f1c7fcb3d2a6480a8693fea652ba9c898f96` | `blake3:f76c1dadcb3a5efcb44a1b30a77ad87db8ff22d5b96adff3d6042c5bebc8d238` | `blake3:9b826cd5948f11b0abd52e60aca83efd6f69dc61f45092327da6219a26e3f01c` | `blake3:a03c4f1ab8cbd516c1ad87b855b1006e376d149285d0dfbc2e13b19127d8985e` |
| N2 | 29 + 32 | 0.621228542 | `blake3:ee0f6c76e53ca181ea5d9ef42f3114b2375624b1ed23082a20f1b510fa25f90b` | `blake3:571bbec870047235eaa5867761f0ad949425e29f18f0052f0278870354c7c31f` | `blake3:d04f99f7cfe922249f98d2d7b94a4798e527c2eb978eaa90d87c195e5983f6c1` | `blake3:b2f2d9ef623fe29d75b52e112bae0991ac94af18533342943b8d1ce3601e7731` |
| N3 | 35 + 32 | 0.287240542 | `blake3:53251f22efac30bb0f77877fe42038887622f7cdefb9b56ea382e51e2123c292` | `blake3:936711f8111763ee09800aad30839da165bbe15d0bb076dd989ec66bafadb5f9` | `blake3:5467f8b5d0ba234bc07f3247978bd8eb141719e97bd737f6479ab2d796f2751c` | `blake3:248efc6ce0d3fa7f5d59088a83377bb58e08ade7d431d83035040179b9a2d31d` |
| H1 | 36 + 32 | 0.275976958 | `blake3:5d00054f113c7e72b1b09b6d2e1e51b00c93e5cd3611e08bf87101d592ca5128` | `blake3:18b390c642f8f051f785b9bc05d496c83a4a0d5ae6333d3e4070ae8eaf58b30c` | `blake3:04eaafedd63355621f0df0961fd25823838b3b086420a4f6b8c66fb725d2b8c0` | `blake3:b165e2045e6d3696661d0687afa5f99ad1bae8ac7d09748c35f9a9200d12c119` |
| H1 control | 18 + 32 | 0.565559833 | `blake3:44a19ef8b77f78616bfe9ba23689f4d675eba9d8afb8629584d8f3a794ad660a` | `blake3:8d1536febb85628aa0d0bb8ae5e59682a4c2e9baaf1735c491244691be0ed751` | `blake3:a831c7143f244828f8aa3215a13ba71de38852942d51c9dc694e8cefd8edfe3a` | `blake3:f5f7b81c9c0734f1177fff03672b85935d6b3375a87a6b015fd291dc76f121ba` |
| H2 | 43 + 32 | 0.371321333 | `blake3:67c44f94481615906a8def2b503a11cf9bc270d0f480696e2aa3a44080fada89` | `blake3:9ae67efa01ecc9869e48b08905b1cda1affa5e868f3ff732344a5f454e0a2f68` | `blake3:8fa332722c0bd94f4261b0ed7f0231d0d0016ec9a8837231baad772ab367299d` | `blake3:867ed3510eba635c1eea46c3d5405b95d0c3a21da0c45af326c812f7501bfd56` |
| H2 control | 17 + 32 | 0.500380875 | `blake3:55c149328392f21f64f87419e94ce3785147c2563447284c79a39d5a2f08a3e8` | `blake3:f0d3ee82407318970a16127d062fcbe27018945dcae5ca435dfd1d35b02deb90` | `blake3:d3162c4b260e1e6ae8d273b0657e49a610a81699d4af1a2dc74563f9818bdce4` | `blake3:f4e3f63ad9f0df4198ca6da4c8b74f19b83309222f83a5995df78dbeade57253` |

The common admitted loader identities were:

- checkpoint tree:
  `blake3:66ee347b23e818f1816682f0b942737c88f1eca831cd6d4f00b3d14fc00aaa37`
- config:
  `blake3:1f1ddb6de22f5c81c04d3093eeff8e0991d63b79ee33bc8ff3cf7c68ef0a9497`
- weights:
  `blake3:c5bf31aa97a567b3aaad4461ce2fac9cebc12b0a38becb6d02d21b43b493bf5d`
- tokenizer:
  `blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc`

The exact-executor worker setting bounds UOR's executor. Apple Accelerate owns
its internal CPU scheduling; the observation does not claim a four-thread BLAS
cap.

## Terminal decision and project consequence

The narrative result is `2/3`: N1 and N3 pass, while N2 fails its explicit
entity/object binding. Both supplied-history arms fail to recover their bound
facts even though neither no-history control happens to guess the target.
Every mechanical condition and both UI/endpoint paths pass.

The predeclared terminal is therefore:

```text
KEEP_RAW_CONTINUATION_ONLY
```

The #1039 raw loopback endpoint and native-dashboard option remain useful as a
bounded source-backed story-continuation reference. #1041 does **not** authorize
a history serializer, multi-turn adapter, chat endpoint, conversation-state
adapter, prompt retuning, additional #1017 training, or another product wrapper
around this checkpoint. Building any of those would overstate behavior that the
frozen normal-use probes did not show.

This result does not revise #1014's load-bearing ordinary-attention evidence or
#1017's bounded autonomous generation evidence. It does not activate or
complete #962, unblock #954, establish correctness or reasoning, establish a
geometry advantage, or make the source-backed floating-point/matmul/softmax
checkpoint transformerless. The active source-free science question remains
owned by #973; any successor mechanism requires a separately authorized fresh
contract.
