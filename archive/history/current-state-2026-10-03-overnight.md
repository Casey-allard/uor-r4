# Current UOR-R4 research state — delivered overnight results, 3 October 2026

Moved verbatim from [`docs/integration/current-state.md`](../integration/current-state.md)
on 3 October 2026 (Eastern Time), as it stood at commit `bc967c39`, to keep that
page within 150 lines after newer 3 October entries were added above it.
Mechanical edits only: same-directory relative links gained a `../integration/`
prefix. No prose was changed. Earlier entries:
[current-state-2026-09-25-to-2026-10-02](current-state-2026-09-25-to-2026-10-02.md).

---

## Delivered overnight results and geometric chat blocker — October 3

The four 8M training arms complete 16,000/16,000 steps without early stopping.
The [loaded report receipt](../evidence/overnight-ladder-delivery-2026-10-03.json)
binds model, executable, tokenizer and data identities plus completed evaluation
reports. Training inputs have matching hashes across arms, with mixture
0.6/0.15/0.25, width288, context384 and batch16. Geometric arms have7,155,396
parameters; the transformer control has7,155,360. The transformer is an offline
comparator, not a served backbone.

| Model | TinyStories final NLL | TinyDialogues full / comparison NLL | chat-v0 full / comparison NLL |
|---|---:|---:|---:|
| quaternion/Lorentz seed1 | 1.597414 | 1.894191 / 1.895699 | 2.211019 / 2.211190 |
| quaternion/Lorentz seed2 | 1.595082 | 1.889056 / 1.890565 | 2.208109 / 2.208357 |
| quaternion/flat L2 seed1 | 1.589840 | 1.887794 / 1.889293 | 2.202534 / 2.202729 |
| matched transformer seed1 | 1.607901 | 1.910046 / 1.911598 | 2.252799 / 2.253005 |

NLL is nats per target. TinyStories final validation has196,608 targets.
TinyDialogues full/comparison has7,678,848/7,654,272 targets; chat-v0 has
6,194,304/6,169,728. The comparison partition excludes the24,576-target tuning
partition. All model identities and token hashes match the saved reports.
The evaluator's prose protocol label says256-token blocks while these report
counts/configuration use384; retain the actual counts/configuration rather than
using that label to infer a different window. These are text-loss comparisons,
not response-quality or integer-serving qualification.

Both Lorentz seeds and the L2 arm have lower loss than the one transformer seed
on these panels. The registered geometric-worse-by0.05 criterion does not fire;
the geometric development stack retains the emitter role. L2 now wins all three
panels at the full budget, supporting the selected read configuration. There is
one L2 seed, one transformer seed and two Lorentz seeds; no general geometric
advantage, full-path energy result or cross-hardware speed claim follows.
Training times reflect different machines/backends and are not matched runtime
measurements. The completed dialogue reports supersede the earlier running status.

The short matched attribution comparison remains: at1,000 updates/two seeds,
quaternion/Lorentz mean NLL2.2138, U(1)/Lorentz2.2372, quaternion/L2 2.1793
([#1639](https://github.com/UOR-Foundation/uor-r4/pull/1639)). This supports those
measured operator choices; it does not establish universal necessity of
noncommutativity or impossibility of Lorentz read.

### Grounded response, persistence and verified source consumption

The retained chat-8m-a session gets959/1,075 accepted turns versus emit-6r889:
Copy31/33, Instruction67/98, MQAR107/109, Relation75/83 and Responsive679/752,
with214/300 complete conversations. Open relation45/52 misses the0.9 gate.
These are authored development conversations using the saved compiler/exact
store/UnlessQuery/sieve; MQAR includes exact log recall rather than proving
learned open-history binding. The recall-off distance16 pre-test remains1/37
for chat-8m-a and emit-6r versus13/37 for the recency rule, with the unequal
8M/2.1M model capacities explicit.

The saved session reload report at
`~/uor-r4-local/ladder/evals/session-chat-8m-a-reload/report.json` now records
20/20 equal continuation checks after saving to disk and reloading compiler
bytes/checkpoint **within the same process**. This does not certify a
fresh-process chat-8m-a continuation. Earlier fresh-process interface fixtures
retain their separate scope.

The [executed source-consumption diagnostic](../integration/geometric-chat-source-binding-2026-10-03.md)
is merged in[#1647](https://github.com/UOR-Foundation/uor-r4/pull/1647) atb482a161;
the adopted roadmap[#1646](https://github.com/UOR-Foundation/uor-r4/pull/1646) is
merged at0376cff8. All20 read-on cases deliver the exact selected payload in the
current input;16 satisfy the finite complete-answer set and4 emit the unrelated
distractor. Read-off0/20 finite membership includes5 acknowledgements mentioning
the right value, so this is not zero factual recall. All40 arms stop at EOS;
there is no history truncation. This supports repairing source consumption after
correct delivery, without proving a unique metadata/attention/capacity cause.

The paired chat-panel instrument remains separate. Under the7B judge,
chat-8m-a acceptable6/64 versus its derangement1/64 gives discordant5:0 and
p=.0625; emit-6r's separate actual count is3/64. Under the1.5B judge,
chat-8m-a acceptable32/64 versus its derangement20/64 gives p=.011818;
emit-6r's separate actual count is26/64. The chat-8m-a relevance comparison
38 versus36 has p=.814529. These significance tests compare each actual arm
with its own derangement, not the two model counts. Do not promote an
acceptable/fluency gain to relevance qualification.

### DeepSeek research and next integration

The [surface-form result](../integration/surface-form-breadth-2026-10-03.md), merged in#1645,
improves unseen-statement value presence37.8%→94.6% but leaves unseen-question
58.1%→63.5% unresolved. These are74 rows/cell, one seed, one core statement frame
with varied prefixes; F1 has7.1% more supervised target visits. Its phrase/value
presence scorer differs from the complete-answer session oracle. Broad semantic
paraphrase invariance and novel-value transfer remain unmeasured.

The [distance result](https://github.com/UOR-Foundation/uor-r4/pull/1642) retains
14/120 versus3/120 for its stated scorer. The distant recipe also halves binding
episodes30,000→15,058 and answer targets296,423→148,563. Preserve this negative
without treating it as an isolated distance effect or a proven capacity ceiling.
Eight query probes show some question dependence, not a qualified retrieval
mechanism or proof that slot addressing uniquely explains all failures.

**Next Codex implementation:** the typed geometric selected-record occurrence→BPE
consumer in the [active plan](../integration/project-track.md#owner-adopted-sequencing-correction--october-3).
Preserve exact record/version/token-offset identity separately from H4/K2 state;
reuse native context/potential/NoRead/integer normalization and train ordinary
answer credit. Require connected actual-BPE forward/backward, source-bound native
reload and measured complete cost before fitting. No32→288 padding, forced oracle
answer or synthetic-V40 language claim. An initial connected floating parent
reference is explicitly hybrid research; complete no-float/no-multiplier serving
remains unfinished. Coordinate shared session edits on#1552. The exact-H4 finite
capture direction audit remains a separate diagnostic, not a D19 blocker.

Claude's latest[#820 update](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5970583776)
reports actual CUDA device parity18passed/0failed/1ignored and short training
throughput at8M/20M/29M. The local parity/throughput logs exist; their exact source
and binary binding still needs a delivery receipt. This optional offline training
port is absent from main at the start of this reconciliation; it is not CUDA
serving or automatic authorization for another paid run. Preserve owner-approved
external work while completing source delivery. No new compute is run by this
read-only/documentation reconciliation.
