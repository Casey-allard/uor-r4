# Grounded-memory evidence and integration boundaries — 1 October 2026

Source review baseline: `b06b46a35142c219a3946209c3ffa03db8499e22`.
Live result cutoff: [#1552 work card v12, 06:09:18 UTC](https://github.com/UOR-Foundation/uor-r4/issues/1552#issuecomment-5925772796).
This record distinguishes reported model execution, inspected source and
independently checked artifact integrity. The review launched no build,
training or model inference. It changes no historical result or promotion gate.

## Latest results

| Component | Reported result | Scope and remaining limitation |
| --- | --- | --- |
| E1 exact lexical route | Development MQAR 747/747; all four reported cells and distance bins at 1.0; independently reproduced by Lab 2 | Untrained instrument route. Relations score 0. It does not establish learned retrieval or a geometric advantage. |
| `emit-1` plus the sieve, v7 | MQAR 106/109: d16 36/37, d64 34/36, d200 36/36 | One training seed; development evaluation; reference earlier assistant replies; instrument category chooses recall dispatch. All actual d200 cases have N=2. |
| `emit-1` plus oracle recall, v7 | Open relation 48/52; closed relation 16/17 | Tests emission given the correct record. The sieve itself yields only 1/52 open and 1/17 closed relation answers. |
| `emit-1` abstention, v7 | 8/14 for oracle and sieve recall | The emission gate's abstention requirement is not met. Given `Memory: none.`, six cases still emit a value. |
| `emit-1` recall off, v7 | MQAR 16/109, against the parent's 37/109 | The fine-tuned emitter depends on the recall channel. |
| Copy panel, v7 | 0/33 | The reply-initial versus space-prefixed BPE boundary remains unresolved. |
| E3 frozen `emit-1` trunk, v11 | Relation .712; act .709 | Teacher paraphrases changed v8's .688/.779. The unchanged .9 relation/.95 act gate remains unmet. |
| E3 frozen R1 7M trunk, v11 | Relation 1,737/2,098 (.828); act 2,829/3,664 (.772) | Improves v9's .714/.702; gate remains unmet. This is a different trained lineage, not a scale-only comparison. |
| E3 lexical control, v11 | Relation 1,756/2,098 (.837); act 2,811/3,664 (.767) | A useful control. Its accuracy does not prove the trunk has less information. |

Sources: [E1/E2 initial results](https://github.com/UOR-Foundation/uor-r4/issues/1552#issuecomment-5923351133),
[E1 independent reproduction](https://github.com/UOR-Foundation/uor-r4/issues/1552#issuecomment-5924653938),
[v7 emission result](https://github.com/UOR-Foundation/uor-r4/issues/1552#issuecomment-5924014037),
[v8 E3](https://github.com/UOR-Foundation/uor-r4/issues/1552#issuecomment-5924307922),
[v9 R1 E3](https://github.com/UOR-Foundation/uor-r4/issues/1552#issuecomment-5924584982),
[v11 E3](https://github.com/UOR-Foundation/uor-r4/issues/1552#issuecomment-5925661022).

V11's R1 action errors include **231 of 595 queries classified as assertions**
and only **5 of 313 updates classified as updates**. These are classifier
outputs, not observed store writes: E3 does not yet run an integrated store.
They identify a concrete false-write risk. The current sieve treats assertion
and update as latest-value replacement, whereas `StackStore` preserves
Assert/Correct conflict semantics. Collapsing those labels would require a
prospective, explicit interface decision; it cannot change v11's result.

V10 produced 373 teacher paraphrases from 84 training templates and screened
out 20 overlaps with development templates. This was local offline data
generation, not a model-training result. The record acknowledges meaning drift.
Exact-string and four-word-overlap screening does not establish semantic label
correctness. [V10 data and v11 work card](https://github.com/UOR-Foundation/uor-r4/issues/1552#issuecomment-5925363260).

### V12 diagnostic addendum

The [v12 result](https://github.com/UOR-Foundation/uor-r4/issues/1552#issuecomment-5925772796)
reports `emit-1` with the learned lexical relation route on one development
draw: MQAR **106/109**, open relations **25/52**, closed relations **2/17**,
and abstention **8/14**. Adding the first paraphrase batch changes none of
these scores. The relation table names 63/83 evaluated queries correctly.
This remains an E4 diagnostic with E3's gate unmet. The distance counts remain
d16 36/37, d64 34/36 and d200 36/36; the actual d200 cases have N=2. Open
relations improve over v7's sieve, but 25/52 does not establish equivalence
to the comparison model's 27/52.

The review inspected lab-branch commit
`9f89bc53bf2329f977b8fbfd707b6bade4ccc97b`, separately from the main-source
baseline. Its `RelationRoute::value` in `relation_compiler.rs` selects the
latest prior user clause classified as the query's relation and as assert or
update. It extracts values by removing every instrument-reserved word; this
does discard closed-relation values by construction. Its `m-world.rs` still
uses reference earlier assistant replies and gold-category recall dispatch,
with the lexical sieve tried before the relation route. It therefore does not
yet demonstrate text-only dispatch, generated-history conversation, learned
value spans or durable store integration.

The lab attributes the unchanged paraphrase result to its sparse unstandardized
fit giving the rare paraphrase words too little weight. That is the reported
diagnosis, not an independently executed causal comparison by this review.
The `e4-route` and `e4-route-para` roots and their executable bindings were
**not** independently checked here; the ten-root verification below does not
cover v12. The [v13 work card](https://github.com/UOR-Foundation/uor-r4/issues/1552#issuecomment-5925775273)
announces a second local offline teacher batch; no v13 completion or model
result is claimed at this cutoff.

## Artifact checks performed by this review

The following ten roots under
`/Volumes/UOR-Workspace/uor-r4-lab/claude-log-sieve/` were read and checked
against their existing sealed manifests:

- `e1-route`
- `emit-1`
- `eval-emit-sieve`
- `eval-emit-oracle`
- `eval-emit-off`
- `e3-compiler`
- `e3-compiler-r1`
- `paraphrases-1`
- `e3p-emit`
- `e3p-r1`

For each root, every recorded file's byte length and **BLAKE3** digest matched;
the actual complete file set matched the manifest, including no unlisted files.
For `eval-emit-sieve`, `e3-compiler` and `e3-compiler-r1`, the review additionally
checked the actual executable, tokenizer and all model files named by the
reports against their recorded **SHA-256** digests. Those checks matched too.
Executable/model identity was not independently rechecked for every other root.

This verifies the inspected records' integrity and binding to available files.
It is not an independent model generation replay, a compiler reproducibility
claim, or a new language result.

| File | SHA-256 |
| --- | --- |
| `emit-1/model/model.safetensors` | `4f40b4e9a3ddca3b084031ce1c7b93775f0f0441481d2be5f043ba8a55ed1d85` |
| `emit-1/model/config.json` | `e1009111a801518160db3b58a1c9981302ad2649fda296f3cf35e6d0af567627` |
| `eval-emit-sieve/m_world_evaluation.json` | `5dab35870510157a89ac58e6afae0e0929f8eb8895d97815006f925164642c16` |
| `e3-compiler/m_world_compiler.json` | `1edd464100080ffd3769c49b9071aae5579c568e240de946db345b8559f4684d` |
| `e3-compiler-r1/m_world_compiler.json` | `3b8de355d4b53d889345facc88bd90e5825baf91a664af53ad52500299846290` |
| `e3p-emit/m_world_compiler.json` | `d0aa32b645c777bf5ba6d3129a857128c77f781f9fe53e7ff945b874add0b610` |
| `e3p-r1/m_world_compiler.json` | `5858ff01f10730f23d7a50dfef09352d5ac403c1b6f62477b3c7f39403093a83` |

The v7 training executable is recorded as SHA-256
`9f4ab9367a907cc156bc76c469196198b57bbb314de89953d1b00ec73d4828f7`.
The checked sieve-evaluation executable has SHA-256
`26d00caa180f0eaab4cf10340ed5d0026b18029dd7442ea797a8dafa3e203112`.

## What the current evaluator and compiler actually do

The implementation boundaries below were inspected at the source baseline.

| Boundary | Source finding | Consequence for integration |
| --- | --- | --- |
| E1 sieve | `sieve_value` scans prior user clauses, intersects exact word sets, ranks by overlap then recency, and extracts the suffix after the first matching word. It uses `reserved_words()` from the instrument. | There is no prime registry, gcd, inverted index or learned relation compiler in this E1 path. It is a lexical baseline, not a persistent bounded prime index. |
| Recall dispatch | `takes_recall` selects the generator's MQAR/Relation categories. | Production dispatch must come from actual text/runtime state; gold categories may remain only inside evaluation/oracle controls. |
| Conversation history | `messages_through` inserts each earlier `Turn2.reply`, and the report explicitly labels history as reference. | The 106/109 result is not a closed-loop generated-history conversation result. |
| E3 features | `trunk_features` extracts last-plus-mean normalized states into host `Vec<f64>` values. | The fit is a frozen-feature probe. This path does not carry its classifier gradient into the trunk. |
| E3 outputs | Two independent heads predict ten relation classes plus none, and assert/update/query/none. | No entity, value-span, scope, confidence/rejection or multi-clause compiler is implemented. |
| Slot helper | `Example::slot_value` matches the known generation template for paraphrase filling. | It is training-data expansion, not runtime extraction from arbitrary text. |
| Saved head | `Softmax` contains private normalization and parameter arrays; `m-world compiler` writes a metrics report. | The evaluated classifier cannot yet be handed off as a saved/reloaded runtime component. |

Sources: [`milestone_world_v2.rs`, `sieve_value` and `takes_recall`](../../crates/uor-r4-training/src/milestone_world_v2.rs),
[`m-world.rs`, `messages_through`, `evaluate_v2` and `compiler`](../../crates/uor-r4-training/examples/m-world.rs),
[`relation_compiler.rs`](../../crates/uor-r4-training/src/relation_compiler.rs).

E3 training deliberately combines **training phrasings with both training and
development value pools**. Its report records
`train_phrasing_train_value` and `train_phrasing_dev_value`. Thus its dev×dev
score is a held-out-phrasing diagnostic, not evidence of unseen-value transfer.
The new integrated behavior evaluation must hold out value identities separately
if it claims that transfer.

The existing E3 failure therefore supports a narrow statement: this frozen
feature/readout/training combination misses its gate. A linear probe is not a
mutual-information bound and does not establish a family-wide geometric
capacity failure.

## Reusable components and compatibility gaps

[`StackStore`](../../crates/uor-r4-training/src/stack_store.rs) already bridges
the retained exact scoped memory to token-valued records. It preserves current,
previous assertion, previous distinct value, initial, Absent, NoHistory and
Evicted distinctions, along with conflict, source and pinned-view semantics.
Reuse these definitions. A compiler's Unresolved outcome must not be translated
into exact Absent, and assistant-generated text must not silently become a user
fact. The store is host-side infrastructure: lookup allocates, its per-chain
payload capacity does not bound all addresses/tombstones, and it is not a D11
serving kernel.

[`stack_checkpoint.rs`](../../crates/uor-r4-training/src/stack_checkpoint.rs)
provides sealed inference checkpoints, but its current save path explicitly
rejects QAT and transport-snap models. Its exact file set does not include
compiler heads or `transport.json`. It also does not persist an ongoing
generated transcript or model-session state. Consequently, it cannot yet
package R1/S4 snapped models or a complete grounded conversation unchanged.
Extend the schema and identity checks before removing those refusals. Model
weight reload and memory equality alone do not establish session continuation.

The real pointer in
[`geometric_stack.rs`](../../crates/uor-r4-training/src/geometric_stack.rs)
has separate learned query/key projections and a gate mixing the copy and
vocabulary probabilities. The
[`stack exporter`](../../crates/uor-r4-training/src/stack_export.rs) refuses
pointer artifacts. The integer session's
[`set_copy_scale`](../../crates/uor-r4-integer/src/stack/session.rs) instead
adds an externally scaled logit boost from head zero of the final ordinary read.
That scaffold is not the learned pointer's export. Substituting it would change
the model and cannot inherit the v7 retrieval result. Similarly, AERM rejects
pointer configurations; it cannot simply wrap `emit-1` without integration work.

## Keep the lineages separate

- **`emit-1`:** approximately 2.1M parameters; `rrarra`, width 128, MLP 502,
  Lorentz reads, free quaternion rotations, dot-pointer width 32, context 384.
  The inspected identity records **`transport_snap=None`**. Its retrieval
  result does not qualify trained-in 2I transport or integer serving.
- **R1:** approximately 7M parameters; its checked model hash is
  `59d58e39e5f7d001bf9e513b0247a577cf710a313b7867993409ce09426fea55`,
  with a saved icosian transport snap. Its v11 compiler uses frozen features
  from this different trained model; it is not the successful emit-1 emitter.
- **D11 stack:** supplies scoped integer execution of supported exported
  models, with dense weight-map and output-head access. It does not yet serve
  the successful pointer/compiler/store loop. Standalone integer flock selection
  likewise exists without integration into that loop.

The next engineering handoff should therefore bind a saved compiler, its actual
trunk and tokenizer, observed value spans, exact store, and the correct emitter
in one text-only session. A generated-history and fresh-process reload check
belongs in that first integration. Component correctness, oracle emission and
historical numerical parity remain useful evidence at their separate scopes.
