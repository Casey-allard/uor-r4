# Dialogue training context: retained R1d data diagnostic

References #973. The native, multiplier-free serving target is unchanged.
This is a Rust data-only diagnostic of the retained dialogue training stream,
with no model fit, generation, objective change or new held-out evaluation.

## Question and source context

The [loaded R1d replay](dialogue-artifact-replay-2026-09-27.md) produced weak,
repetitive answers. That result does not identify capacity, data, objective,
exposure or representation as its cause. The historical conclusion that close
train/development loss excludes data or objective problems is too strong:
both populations can share the same conditioning mismatch.

The retained trainer lives on the separately owned
`codex/canonical-address-routing-20260925` branch at `d22e2b50`. Its
[preparer](https://github.com/UOR-Foundation/uor-r4/blob/d22e2b50da85e527efcd4f60acc1e82644194b91/crates/uor-r4-core/src/bin/prepare-chat-corpus.rs#L255)
serializes complete conversations with BOS, literal role segments and genuine
assistant EOS. Only assistant content and its EOS receive response mask1.
The [sampler](https://github.com/UOR-Foundation/uor-r4/blob/d22e2b50da85e527efcd4f60acc1e82644194b91/crates/uor-r4-training/src/dialogue.rs#L170)
then selects arbitrary 256-position windows in the concatenated token stream.
The recurrent forward path begins with zero state at each sampled start.
It does not restore the state from the preceding conversation prefix or reset
that state again when a later document BOS appears inside the window.

The next-token indexing is consistent: inputs use `start+j`, and both target
and mask use `start+j+1`. Masked loss divides by actual response-target count;
the R1d sharded path combines gradients using that count. Mask0 on prompt
targets does not block the gradient through prompt-dependent state. The old
report's duplicated nominal supervision count is an accounting defect, not
evidence of an incorrect optimizer denominator.

## Fixed observation and interpretation

The audit reconstructs the retained run's 2,237 updates, batch16, context256
and data seed20260926. It verifies every reconstructed response-target count
against the corresponding retained curve row. It reads only the training
token/mask files and their provenance; it opens no held-out tokens or fresh
panel requests. A separate pass measures structural eligibility of the whole
training corpus and its source groups.

For a sampled input start `s`, a supervised target `t`, its contiguous response
mask run `[r,e)`, and that response's document BOS `d`:

| Condition | What it establishes |
|---|---|
| `r < s` | The response's first token lies before this window. |
| `r <= s` | Every token preceding this response lies outside the window. At equality the first response token is an input, but the Assistant marker is absent. |
| `d < s` | The complete original document prefix is unavailable. A later complete user turn may still be present, so this alone does not prove the latest request is absent. |
| `d == s` | The zero-state start coincides with the original document BOS. |
| `d > s` | The document prefix is present, but recurrence also consumed preceding-document input. |

The legitimate first response target `t == r` need not have its own token in
the input; that is ordinary causal prediction. Presence of one pre-response
token does not establish the whole Assistant marker or user request. The
audit therefore does not infer semantic request boundaries by searching for
literal role text inside content.

A complete response beginning from document BOS fits within 256 total IDs when
`e-d <= 256`. The analogous limit of257 stored tokens is reported separately:
the historical training window consumes256 input positions and predicts256
shifted targets. The serving adapter reserves its declared256 total-ID horizon.
Neither quantity is the read-vector width64 or a selected-event count.

## Decision boundary

Substantial missing-prefix exposure would support an explicit
conversation-prefix response sampler as a candidate for a later controlled
learning change. It would not show that this mismatch caused the observed
weak replies. A small eligible population would mean that such a sampler also
selects a materially different short-dialogue dataset; report and choose that
population explicitly rather than presenting it as a neutral implementation
fix. Little missing-prefix exposure would deprioritize this explanation.

A future option should preserve the historical sampler as its control, retain
the selected response's complete original prefix, supervise the selected
response and genuine EOS only, and report padding/exclusions and actual
supervised exposure. It must not manufacture a target EOS at an arbitrary
capacity cut. Its population and comparison would be new, so the old
count-comparator NLL cannot be reused as a score for it.

The current mainline contains the artifact replay but not the foreign dialogue
trainer/preparer. An integrated sampler change therefore needs coordination
with that branch's owner. This audit imports neither trainer nor new dependency.
The frozen reader study and separately owned termination/stack experiments
retain their source, objectives and decisions.

## Execution record

The [work card](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5853377261)
declares the 45-minute complete envelope, one short data-only process and the
unchanged cumulative/resource ceilings. The
[source-bound receipt](../evidence/dialogue-context-audit-2026-09-27.json)
retains the result, execution/build records, input identities and cumulative
accounting. Executed source is `35beb8a58d7230b9cb7be23d9aee8508b82ba161`;
subsequent publication changes are documentation/evidence only.

Three focused interval/boundary checks passed. The complete Rust data pass
finished in 3.210 seconds with 261,144,576 bytes maximum child RSS and no resource
stop. Build/check work took 89.734 seconds. It reconstructed all 35,792 windows;
every one of the 2,237 per-update denominators matched, totaling 6,280,627
response-supervised target visits out of 9,162,752 sampled positions. Independent
review checked the saved row inventory, sums, source totals and interpretation.
There were zero optimizer updates, model forward calls or generated tokens.

| Actual retained-run exposure | Target visits | Share of response-supervised visits |
|---|---:|---:|
| All pre-response prefix outside the window (`r <= s`) | 4,685,553 | 74.6033% |
| Response start strictly before input start (`r < s`) | 4,672,230 | 74.3911% |
| Complete document prefix unavailable (`d < s`) | 6,075,537 | 96.7346% |
| Fresh start exactly at document BOS (`d == s`) | 2,743 | 0.0437% |
| Complete document prefix present after earlier-document input (`d > s`) | 202,347 | 3.2218% |
| Supervised EOS | 14,461 | 0.2302% |

These are target-exposure counts, not counts of distinct conversations, examples
or windows. There were 4,179 windows with no supervised targets; the 16-row batch
still had supervision. The 8,109 windows spanning a document boundary count the
combined input/target interval; a BOS appearing only as the final target has
not yet entered recurrent state.

Only 7,490/73,632 whole documents (10.1722%) and 14,826/129,486 complete response
prefixes (11.4499%) fit within 256 total IDs. Allowing 257 stored training tokens
raises these counts to 7,575 documents and 14,926 responses; it does not change
the serving horizon.

| Training source | Original response runs | Complete response from original BOS, <=256 IDs | Actual sampled response-target visits |
|---|---:|---:|---:|
| Smol Magpie Ultra | 41,340 | 404 | 3,336,341 |
| Smol Constraints | 17,900 | 6,037 | 472,054 |
| Smol Rewrite | 17,200 | 85 | 475,693 |
| Smol Summarize | 12,300 | 401 | 223,359 |
| UltraChat | 32,121 | 669 | 1,734,493 |
| Everyday Conversations | 8,625 | 7,230 | 38,687 |

Everyday Conversations plus Smol Constraints comprise 20.4848% of the original
response runs but 89.4847% of the eligible complete response runs. This is an
eligibility-count comparison, not a measured future supervised-token mixture.
For example, Everyday Conversations supplies only 0.6160% of this historical
run's supervised target visits; drawing uniformly from eligible responses would
be a different population and weighting policy.

**Decision:** conditioning loss is substantial in the observed stream, so an
explicit request-preserving data/sampling intervention remains a concrete
candidate. A simple complete-prefix filter is not a neutral repair: it excludes
most responses and sharply changes source composition. Before the next dialogue
fit, declare the selected population, source weighting, real-EOS handling and
matched sampler control with the dialogue branch owner, then choose its role
alongside the pending reader/termination/stack results. No fit, filtering of
retained artifacts or context-limit change is triggered by this diagnostic.
Incomplete-prefix targets are not shown to be useless, and the cause of the
weak loaded output remains unresolved.
