# Complete-prefix dialogue learning candidate

References #973. **Execution update:** the [implementation and two-update
integration witnesses](dialogue-prefix-implementation-2026-09-27.md) are complete.
The substantive paired study remains NOT_RUN. The prospective design below is
preserved; its implementation requirements now have a separately bound result.

The
[completed reader comparison](radial-adaptation-result-2026-09-27.md) does not
support promoting either transferred radial configuration. This independently
justified dialogue intervention is the next fourth-lab learning investment.
This plan does not launch a fit, extend its budget or change native
multiplier-free serving. Its implementation must connect learning to retained
reload and actual replies before the paired model study begins.

## Question and evidence boundary

Does training with the complete original request/history improve useful
conditional responses compared with learning the same replies from the
assistant role alone? Both arms retain the same recurrent geometric cell,
learned starting arrays, response population, response-token objective and
local exposure. Prefix content, physical response position and history length
are deliberately changed together. This does not isolate content from position
or prove that historical window sampling caused the retained model's failures.

The [R1d replay](dialogue-artifact-replay-2026-09-27.md) establishes weak actual
dialogue output, not its cause. The completed
[training-context audit](dialogue-context-audit-2026-09-27.md) verifies the
historical mask shift and differentiable prompt-to-response path. Its training
inventory contains 129,486 response runs; **14,826** have complete document BOS
through response EOS within **256 total token IDs**. Only 7,490 whole documents
fit that limit. Select response runs rather than requiring an entire later
conversation to fit. The selected population is strongly source-skewed; retain
the audit's source counts and report the selected population explicitly.
Everyday Conversations and Smol Constraints provide 89.4847% of these response
runs, not necessarily that fraction of supervised tokens. Do not
present it as all chat-v0, or use the separate 257-stored-token count of 14,926.

## Exact episodes and paired arms

Bind the existing prepared training token store, response mask, source manifest,
tokenizer and shared `DialogueProtocol::literal_roles_v1` identity. The retained
manifest records zero literal special-token occurrences, allowing its BOS and
response-mask runs to identify document and response boundaries. This is a
condition of this import, not a general parser for arbitrary dialogue text.

For document BOS index `d` and selected response run `[r,e)`, require:

- `e-d <= 256`; the full prefix and complete selected response fit together.
- The selected run is nonempty, its actual last token is protocol EOS, and its
  content has no internal delimiter EOS under this bound corpus contract.
- The complete assistant marker immediately before `r` has mask 0 and the exact
  IDs produced by the bound protocol. No answer text is searched for role names.

Let `M` be the marker IDs from
`encoder.encode_assistant_prefix(&[]).tokens[1..]`, after verifying its first ID
is the bound BOS. The shared encoder emits BOS followed by independently encoded
`Assistant: ` for empty history. Verify `tokens[r-M.len()..r] == M`, then take
the **actual stored marker slice** for the control. This retains the trained BPE
segmentation and trailing space, rather than encoding a concatenated display
string or assuming a hard-coded marker token count.

| Arm | Input episode before causal shift |
|---|---|
| Full prefix | Exact stored `tokens[d..e]`: original BOS, all prior turns, actual separators, selected Assistant marker, complete response, real EOS. |
| Role only | Original BOS, actual stored Assistant marker `M`, then the identical `tokens[r..e]` response and real EOS. |

The full prefix includes the actual separately encoded newline before the
selected Assistant marker whenever prior turns exist. The role-only episode
has no prior turn, so it has **no leading newline**: BOS followed directly by
the marker is the protocol's valid empty-history assistant prefix. No user
placeholder, invented fact, synthetic EOS or re-encoded response is introduced.
The role-only arm cannot admit additional long responses excluded by the full
prefix rule. Record any episodes whose original prefix already equals the
role-only prefix; they are shared presentations with no conditioning contrast,
not additional evidence of prefix benefit. Do not silently remove them.

For either episode `x`, observe `x[..len-1]` and predict `x[1..]`. Supervise only
the selected response and genuine EOS. All earlier assistant replies are
observed context with loss mask 0 for this episode. Right-pad to the same
declared tensor time using an in-vocabulary filler and zero loss weights.
Padding is after the real EOS label and contributes no response targets; it
does not replace EOS. The causal graph needs no attention-mask change, left
padding, cross-document packing, replayed prior state or context eviction.

Sample eligible response IDs uniformly with replacement using a bound
counter-based step/lane schedule. Both arms receive exactly the same response
IDs in the same updates and lanes. Record actual response-target visits,
including EOS, separately from valid prefix positions and padded compute
positions. This avoids length-bin exclusion and target-position sampling bias;
it does not remove the declared population's length/source selection.

## Objective and optimizer lineage

Keep the historical **global response-token mean**:

`loss = sum(selected response and EOS token NLL) / total selected response tokens`.

Response-uniform sampling is not per-response loss normalization: longer sampled
responses still contribute more supervised tokens. Both arms use this same
denominator and receive identical supervised-target counts per update. There is
no punctuation weighting or per-episode mean. For CPU shards, combine shard
mean gradients by `shard_supervised / batch_supervised`, then perform one global
clip and one AdamW update. Do not use mainline's existing shard-local weighted
mean with equal batch-fraction combination for these variable response counts.
Reuse the narrowly reviewed R1d masked-reduction pattern without importing its
whole trainer or changing the retained termination objective.

Start both arms from independent exact copies of the retained R1d's 21 arrays:
5,429,826 F32 parameters, width576/read64/context256, Quaternion/Dot/Full. Bind
parent parameter SHA256
`95e3fbb06cb39b4354bd40c722873dac088d47554707777b291e588922a1a822`
and the loader's complete provenance. Create fresh Adam moments and a shared new
data clock/seed. This is an explicit parameter-only continuation with new local
exposure and artifact identity, not resume of an unavailable historical
optimizer. Keep the historical source label and retrospective mask/tokenizer
binding separate from the new compiled source, executable and campaign.

## Integrated implementation and ownership

Implement a new mainline `training::dialogue_episodes` module with bound span
inventory, paired batch construction, masked global-mean updates, selected
development evaluation and checkpoint metadata. Add a small bounded executable
entry point that actually connects retained load, updates, save, reload and
exact-token generation. A metadata compiler alone does not complete this work.

Use a narrow crate-private parameter-start adapter in `dialogue_artifact`; keep
its existing public replay API read-only. Add an explicit offline576 checkpoint
reload that retains inventory, finite-value, hash and numerical-contract checks.
Ordinary `JointModel::save` can write this shape, but canonical `load` rejects it;
do not leave a saved model without the matching offline reload path. Reuse
`NamedAdamW` and the existing exact-token generation loop. Do not widen canonical
campaign validation, integer loading, conversation/session APIs or serving.

The separately owned `codex/canonical-address-routing-20260925` worktree remains
at `d22e2b50da85e527efcd4f60acc1e82644194b91`. Its Rust sources are unchanged in
the inspected checkout; its two untracked R1d result/evidence files remain
preserved. The new mainline module does not edit its `dialogue.rs`, preparer or
records, and does not take over Google's CLI/session work. Coordinate the small
mainline artifact/model/parallel integration paths before implementation.

Focused checks address selected-span boundaries, real EOS, exact marker IDs,
padding causality, unequal-count shard aggregation, identical paired target
schedules, independent starting arrays/Adam state, and actual offline checkpoint
reload. Existing mask-shift and gradient-chain findings are retained. They do
not require another general audit or substitute for loaded output.

## Measurement, decisions and cost

Before any model work, freeze in the existing work card the local update dose,
episode/sampler hashes, optimizer settings, opened development response IDs,
small existing development request set, generation policy and complete resource
projection. Evaluate both saved models on the same original full development
prefixes and identical response targets. Review actual exact-token responses
against the same requests and retained parent output. Earlier development data
are already open; final held-out evaluation remains separate. Do not expand the
population or decoder settings after seeing the result.

The three substantive outcomes are:

1. **Useful conditioning signal:** full-prefix training produces better matched
   conditional response evidence and visibly more request/history-correct
   loaded output than both the retained starting artifact and role-only
   training, without an offsetting declared
   retention failure. Retain it as the next integration candidate; a further
   dose still needs a distinct projected work card.
2. **No useful differential:** both arms improve similarly, remain weak, or the
   loss difference has no useful output counterpart. Similar gains may support
   adaptation or the selected population, but do not identify a prefix benefit.
   A gap caused only by degrading the control is also insufficient. Retain the
   result and stop this intervention at its declared dose. Do not infer a
   universal capacity limit or automatically add training, decoder sweeps or
   more validation.
3. **Harm or reverse effect:** full-prefix learning worsens conditional output
   or violates a declared retention condition relative to the matched control.
   Preserve both checkpoints, decline promotion and use the concrete divergence
   to choose a different learning/state intervention.

An invalid episode binding, failed load or incomplete run is **UNAVAILABLE**
evidence, not one of those model-quality outcomes. No numerical improvement
threshold, extra fit or acceptance claim is introduced by this planning record.

Historical R1d receipts support CPU batch 16/context 256 with two gradient shards
at roughly 4 GiB reported memory, mean 8.90 seconds per update and 21,633 seconds
for its 2,237-update run including repeated evaluations. Those contended-host
figures are a projection basis, not a promised new rate. Larger batches and four
shards did not supply the projected throughput improvement. Keep width 576,
context 256, response visits and padded tensor work distinct; masking does not
eliminate forward/backward computation. The complete future projection must
include implementation/build/review, both arms, development output, checkpoints,
reload, storage and delivery, charged without overlapping-wall double counts.
**No fit or resource extension is launched by this document.**

## Research context

[DoReMi](https://arxiv.org/abs/2305.10429) studies how domain-mixture choices
change language learning. Its larger-model results motivate treating this
short-response population as an explicit intervention; they do not predict a
gain for UOR-R4 or justify adding a proxy-model/reweighting programme here.
[Instruction Tuning With Loss Over Instructions](https://proceedings.neurips.cc/paper_files/paper/2024/file/7ffb43adf37b3eeaba559098bc084cc6-Paper-Conference.pdf)
distinguishes response-only learning from adding prompt-token loss and reports
effects dependent on data conditions. This plan restores prompt **input** while
retaining response-only loss. The audited recurrent graph already carries
response gradients through prompt processing; adding prompt loss would be a
separate, unselected objective change. Neither paper establishes this native
model's capacity, useful dialogue or serving advantage.
