# #1012 — A1Q-H2: localize R4 trace-state signal loss with a construction-only observability ladder

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-30T14:53:31Z
- **Updated:** 2026-08-30T17:50:15Z
- **Closed:** 2026-08-30T17:50:15Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1012
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Localize the first boundary at which transferable next-token signal is lost between the established R4/Spin causal-softmax trace and the completed negative R4SoftmaxTraceStateStudentV1 cell. This is a construction-only observability audit. It does not fit or promote another held-out model.

Parent: #973. Predecessor: #1011. Programme root: #820.

## Evidence that authorizes this rung

The #1011 authoritative run completed with exact artifact/replay integrity and zero forbidden reads, but did not establish decision-bearing geometric state:

- geometric covered teacher CE: 2.660705366832157;
- suffix CE: 2.6607210317310273;
- plain recurrent CE: 2.6607709186456363;
- transport-permuted CE: 2.660729214868547;
- all arms: teacher top-1 3/9 and actual-next top-1 2/9;
- permuted minus geometric CE: 0.00002384803639, below the frozen 0.10 materiality floor;
- all four arms decoded the same period-two loop;
- result CID: blake3:dc04a8a8b21750799db2d451c8237d1e62cf90ffa74561fb54272b1e9704c824.

Therefore no scale, state widening, resonance replacement, exact lowering, WASM promotion, or release work is authorized.

## Strict scope

Use construction documents only and perform leave-one-document-out transfer within that construction population. Document 13 and every field revealed in #1011 are burned for diagnosis only and must never qualify a repaired model.

Audit these four nested boundaries under matched folds, candidate support, work accounting, and simple regularized probes:

1. full ordered final-layer Q/K/V trace blocks;
2. the frozen signed 576-to-4 reduction;
3. token-derived role maps plus recurrent state features;
4. fitted residual readout and logit scale.

No teacher source calls are needed: consume only the frozen construction trace bundle and canonical loader already qualified in #1011.

## Decision table

- Full trace transfers but 4D does not: replace the fixed reduction with a structured per-head or multiscale geometric representation.
- 4D transfers but recurrent features do not: repair context-conditioned K/V induction and state update.
- Recurrent features transfer but logits are inert: repair readout calibration only.
- Even the full trace fails leave-one-document-out transfer: stop trace distillation and specify end-to-end cell training under a new independent holdout.

Only the earliest failed boundary becomes the next implementation issue. Do not tune later boundaries around it.

## Run contract

metric to move: leave-one-document-out covered teacher CE and teacher top-1 at each nested boundary
reachability ceiling: four construction documents and 38 recorded positions; report exact evaluable non-BOS support per fold before scoring
cheap instrument: canonical bundle reload, exact nested CIDs/shapes/order, finite-value audit, fold disjointness, feature rank/variance, target-blind preprocessing, and zero source calls
exit rule: report paired per-position and per-fold deltas against suffix/count baselines with bootstrap or exact sign uncertainty; no promotion threshold is inferred from document 13
if positive: open exactly one repair issue at the first boundary that loses a signal already present immediately upstream
if negative: stop trace distillation and freeze an end-to-end training specification with a new corpus split and untouched holdout
cost estimate: minutes, CPU-only; refuse any hours-long run

## Definition of done

- canonical construction-only fold manifest and input CID;
- executable nested-boundary probe with matched dimensions or explicitly reported capacity;
- feature liveness/rank and leakage audit;
- per-fold and aggregate CE/top-1 results with deterministic replay and work ledger;
- geometry/order destruction controls where geometrically applicable;
- append-only evidence record and structured raw result;
- one evidence-forced next action, with no reuse of revealed document 13 for promotion.

References #973
Depends on #1011

## Comments (2)

### comment 5469575398 — 2026-08-30T15:28:15Z — Casey-allard

## Construction-only intake and architecture freeze

Read-only inspection of the canonical #1011 loader and the 45,205,493-byte construction bundle completed. No source model, judge, document 13, #1011 seal, or #1011 reveal artifact was opened.

Frozen fold order is bundle order:

| LODO holdout | Train events | Test events | Non-BOS | Exact-prefix-novel |
|---|---:|---:|---:|---:|
| 14 | 30 | 8 | 7 | 7 |
| 657 | 30 | 8 | 7 | 3 |
| 4579 | 28 | 10 | 9 | 9 |
| 5121 | 26 | 12 | 11 | 7 |

The binding metric uses the 26 exact-prefix-novel causal events; all 34 non-BOS events are secondary. Documents 657 and 5121 share positions 0 through 4 and every document shares BOS, so an unstratified score would inflate apparent transfer.

### Frozen boundaries

1. Full final-layer ordered Q/K/V: 1,728 finite scalars in strict role, head, R4-block, lane order.
2. Existing signed reduction: 12 scalars, four lanes independently for Q/K/V.
3. Fold-fitted #1011 candidate-state tensor: 64 values from four banks by four response lanes by four candidate-query lanes, reported for plain and geometric replay.
4. Existing residual readout: base logit plus 64-weight residual, with a train-only fixed residual-scale diagnostic; scale 1 is the actual #1011 mechanism.

Every boundary uses the same fold-specific support of at most 32 candidates, teacher target conversion, loss reduction, regularization, optimizer schedule, and exactly 64 learned probe weights with no bias. The full and reduced boundaries use deterministic candidate-conditioned signed sketches that consume every input scalar; the state boundary uses its natural 64 features. Each fold recompiles its suffix/support artifact from only the other three documents.

### Cheap liveness result

- 38 total events, 34 non-BOS, 26 exact-prefix-novel.
- 31 exact unique full Q/K/V rows.
- 31 exact unique 12D reduced rows.
- Centered reduced rank is 12 in every training fold.
- The 576-to-4-per-role reduction creates no new exact collision in this population.

Therefore the reduction is numerically live, but predictive information loss remains unresolved. The bounded probe is reachable and remains a minutes-scale CPU-only job.

### Implementation seam

Reuse the canonical CID-bound bundle loader, teacher Q16 conversion, fold-specific suffix compiler, signed reduction, deterministic token R4 features, state compiler/runtime, stable softmax, and H4 frames. Add only:

- one read-only diagnostic snapshot API exposing the natural 64 state/readout features and base/residual/total logits without changing runtime counters;
- one strict r4-softmax-trace-observability command;
- the four-fold deterministic result schema and five focused tests.

The command accepts the construction bundle, predecessor freeze, implementation revision, and result output only. It will reject source, judge, document-13, reveal, hyperparameter, or fold-selection inputs.

No experiment longer than minutes is authorized. Implementation begins only after #1011 protected delivery so the new work starts from the exact merged state-student revision.

### comment 5470107464 — 2026-08-30T17:12:05Z — Casey-allard

## Authoritative result — attempt 02

The bounded construction-only run completed from implementation `3ef7af9860baf55fbe43a9b5207c60fd3a737564` in 45.56 seconds after attempt 01 stopped before scoring on an invalid cross-document control. Attempt 02 used one fixed within-support Q16 label rotation; it preserved support and mass exactly and did not tune the mechanism.

Result CID: `blake3:11f890def300cdafd689ef7cbfcf28e46b693e669a46edfa7431c672582db4a2`
Terminal: `INSUFFICIENT_SUPPORT_COVERAGE`
Exact replay: `true`
Forbidden source/model/document-13 reads: `0`

Binding primary slice:

- aggregate support coverage: `0.6202622204224402`; minimum fold: `0.3469116829611222`;
- suffix CE: `2.215064603216862`;
- full current-step final-layer Q/K/V probe CE: `2.215410922655504`;
- suffix-minus-full delta: `-0.0003463194386417179` nats, with full improving in `0/4` folds;
- both suffix and full: teacher top-1 `14/26`, actual-next top-1 `6/26`;
- full beat the destructive label control by `1.3807454322642605` nats in `4/4` folds, confirming the control was live but not a benefit over the real suffix baseline.

The frozen coverage floor blocks causal attribution to a downstream boundary. It does not justify another support-expansion or observability campaign: every real diagnostic and exact recurrent arm remained materially indistinguishable from the suffix, and the full trace was slightly worse in all four folds.

Project decision: stop this bounded current-step Q/K/V trace-distillation path. The next rung will directly train an ordinary causal-softmax attention decoder in R4 coordinates end-to-end on a fresh untouched split and will require autonomous decoded generation. Offline float arithmetic remains allowed; exact runtime lowering comes only after the learned mechanism works.

The raw result and synchronized documentation are being committed on the #1012 branch. No geometric advantage, coherent generation, reasoning, WASM readiness, or release readiness is claimed.


---
_Comment/review ids in this file: 5469575398, 5470107464_

