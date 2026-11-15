<!-- freshness: sources=crates/uor-r4-discrete,crates/uor-r4-discrete/README.md,crates/uor-r4-discrete/src/device.rs,crates/uor-r4-discrete/src/discrete.cu,crates/uor-r4-discrete/src/metadata.rs,crates/uor-r4-integer/src/discrete_d11.rs checked=abfad060460503bbd468f5b8c74b892b2aaca800 -->
# The model, end to end

UOR-R4 turns text into whole-number token ids, carries those ids through geometric state and exact
memory, and chooses the next token without softmax or dense matrix multiplication. **M1 is
complete**: a saved discrete model passed all six bars on two seeds. No model holds a useful
conversation. Two model paths exist and have not joined: the discrete M1 path and the float-trained
FBS trunk.

<!-- native-space-contract: Native default: one persistent native geometric state uses one declared integer or fixed-point representation for token addresses, vector coordinates, carry, routes, memory keys and reads, learned state, gradients and optimizer updates; training and serving use the same typed operators; route choices, carry state, read choices, memory addresses and token ids are bit-identical on the same prompt and starting state; the artifact losslessly serializes that state instead of an F32 master model or an export gap; floating point remains reference or diagnostic code only. -->

**D85 status:** the persistent native state/operator implementation is pending; all float-to-integer measurements on this page are historical/export evidence.

**D85 correction:** these two paths record how the model arrived at M1. Under D85, the default
path is required to carry one persistent native integer/fixed-point geometric state through the
same typed training and serving operators. The first implementation item is
[`native-space-state-contract`](../roadmap/items/native-space-state-contract.md). The diagrams and
measurements below remain historical evidence until that contract and a joined native-space model
replace them.

![Two separate pipelines: the completed discrete M1 lane uses PositionHolonomyV1, Flock plus an orbit-root causal pointer, discrete Hamilton mixing, OrbitRootSnapNormV1 and OuterL1V1 readout; the float-trained FBS lane separately uses SpiralCore position, FBS recurrence, Hamming copy heads, exact turn memory and whole-table nearest readout.](../figures/model/pipeline.svg)

*The upper discrete M1 lane and lower FBS research lane are different models, not one joined pipeline; only the D11 integer serving boundary is shared.*

Labels follow [`docs/formal_vocabulary.md`](../formal_vocabulary.md): **proof**, **measurement**,
**hypothesis**. A construction control is not a language result, and `NOT_RUN` is never a pass.

## The discrete path that closed M1

The saved artifact is `trunk-joint-pointer-v1-d11`, a 19-table `uor-r4.lut-discrete-trunk/1`
`UORLUT01` model. Its composition is Flock `FrozenZeroV1`, a discrete Hamilton gate, discrete
quaternion recurrence, `PositionHolonomyV1`, `OrbitRootSnapNormV1`, a four-block orbit-root causal pointer, and post-rotation outer-L1 readout.

| Stage | Mechanism and representation | State at the M1 close |
|---|---|---|
| Tokenizer | Byte-level BPE, 4,096 pieces, identity `d36d3e87` | Fixed and pinned to corpora, panels and artifacts |
| Token table | Shared input/carry/output vector table; exact token identity stays separate from geometric address | Trained |
| Ordered context | At most 243 positions; recorded runs use 192 | Built and used |
| Reads | Flock read and orbit-root causal pointer using hard selection, not softmax | Trained in the M1 composition; Route and Relation are other discrete mechanisms, not isolated by this whole-model pass |
| Writer and readout | Post-rotation `OuterL1V1` candidate scoring and nearest-token choice | Trained; full 19-table export carries the final readout |
| Objective and learning | Integer rank-margin hinge and discrete legal moves, including BVG-K batched verified coordinates | Trained; produced the saved model |
| Position | Discrete quaternion recurrence and `PositionHolonomyV1` | Trained in the M1 composition |
| Normalization | `OrbitRootSnapNormV1` | Trained in the M1 composition; discrete normalization, not the FBS RMS norm |
| Feed-forward | Discrete Hamilton gate | Trained in the M1 composition |
| Position and normalization on the separate FBS path | SpiralCore transport spin and RMS norm with integer square root | FBS-only; not M1's projection or normalization mechanism |
| Export | Table-native `uor-r4.lut-discrete-trunk/1` | Exact 14,287-row DEV agreement and B4 pass |
| Serving | D11 integer engine on an Apple M1 | 10,876 / 10,566 decode tok/s under the bar-f protocol |

For implementation detail, see [`crates/uor-r4-discrete`](../../crates/uor-r4-discrete/README.md),
[`discrete.cu`](../../crates/uor-r4-discrete/src/discrete.cu), and the
[pipeline map](pipeline-map.md).

## What the saved model measured

| Panel/contract | Seed 1 | Seed 2 | Controls |
|---|---:|---:|---|
| DEV integer top-1 | **7,002 / 14,287** | **7,001 / 14,287** | unigram 1,226; T2 6,300 |
| Fresh V2 integer top-1 | **7,215 / 14,479** | **7,228 / 14,479** | unigram 1,279; T2 6,582 |
| p-f held-out phrasing | **7,850 / 15,953** | **7,872 / 15,953** | unigram 1,409; T2 7,375; band 81 |

The older close grade covers bars (a)–(e); claude-211's panel score supplies bar (f). p-f has no
prompt value-masked templates in TRAIN at K = 1,024, but **62.5% of its reference replies are
verbatim TRAIN replies**. M1 therefore shows reading new prompt phrasings into known reply kinds,
not new reply kinds or useful conversation
([M1 close](../labs/m1-close-saved-model/README.md), [Fresh V2](../labs/m1-fresh-panel-transfer/README.md), [bar-f report](../loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md)).

## The separate float-trained FBS trunk

The FBS path trains offline in float under D37 and exports to integer D11. It is not the M1 model.

| Step | Part | Mechanism and current state |
|---|---|---|
| Token | Byte-level BPE, 4,096 pieces | Done |
| → vector | Learned FBS table, 4,096 × 256 | Trained and served |
| → vector | Position | SpiralCore transport spin, trained and served; no RoPE |
| → vector | Mixing | FBS recurrence, Hamilton gate and SpiralCore/FBS routes, with exact rotations and the DEV export gap inside the bar under `lut-stack/5` |
| → vector | Copy/look-back | Hamming read and copy heads; integer kernels exist, full joined serving remains open |
| → vector | Storage | Exact turn memory keyed by the turn's exact first token ids; strong on familiar DEV prompts, weak on new phrasing |
| → vector | Feed-forward and norm | Hamilton gate and RMS norm with integer square root, served |
| → integer | Export | `lut-stack/5` inside the export-gap bar at DEV scope |
| → token | Readout | Whole-table nearest chosen; the joined served session is not complete |
| Text in/out | R7 and accepted-model text/chat distribution | Open |

**Reply openings, through claude-232.** Claude-217's cycle and echo steps end every registered loop,
but content is underpowered and greedy remains the decode. On p-s, teacher-forced reply openings are 17.4%, 13.7%
and 18.6% at three seeds, against 80.8%, 80.2% and 79.9% on later tokens. Exact-reference replies
are only 3, 1 and 4 of 256, and exact-key memory finds little on new phrasing
([claude-217](../loop/tasks/one-model-end-to-end/claude-217/REPORT.md)). Claude-218 found
trunk-state keys offline; [claude-219](../loop/tasks/one-model-end-to-end/claude-219/REPORT.md)
then tested them in free generation on fresh p-v. The point-key opening improves templates by
+13/+8/+12 and exact replies by +9/+8/+12 at three seeds, but only partially. The sign key is
inconclusive at this panel size. [Claude-220](../loop/tasks/one-model-end-to-end/claude-220/REPORT.md) then served the point key inside one float model on fresh p-w: templates improve by +41/+40/+42 at three seeds with exact claude-219 parity, while recalling TRAIN replies rather than generalizing. [Claude-222](../loop/tasks/point-key-integer/claude-222/REPORT.md) has built the integer point table, search and `/8` record, with 380 library tests and a full serving audit. [Claude-224](../loop/tasks/point-key-integer/claude-224/REPORT.md) has built the panel decode and grade. [Claude-227](../loop/tasks/integer-retention-rates/claude-227/REPORT.md) passed retention GPU parity within the 3% bar; [Claude-228](../loop/tasks/point-key-integer/claude-228/REPORT.md) built the run's reference/read-back/BOS checks; [Claude-229](../loop/tasks/point-key-integer/claude-229/REPORT.md) found an instrument fault: the integer session uses the native writer instead of the trained whole-table nearest readout, so no bar result or throughput is reported. [Claude-231](../loop/tasks/integer-nearest-readout/claude-231/REPORT.md) built the nearest readout and export on native and browser paths; [claude-232](../loop/tasks/integer-nearest-readout/claude-232/REPORT.md) then missed greedy parity in every cell.

**State on 14 Nov 2026.** Since 10 Nov the work is recall in controlled conversations. The current recipe reads with four Hamming reads and an exact turn memory, not the Route, Flock or Relation reads, and training carries no state from one window to the next. The pure-vector baseline, [claude-376](../loop/tasks/one-model-end-to-end/claude-376/REPORT.md), found that almost every fact the model gets right comes from the turn memory's table. On 13 Nov D85 made one persistent integer geometric state, trained and served by the same typed operators with bit-identical replay, the default route. V1, the state carried across training windows (PR #3979), and V2(a), three vector-native reads behind flags (PR #3978), are in progress. Useful chat remains unproven.

## What remains unbuilt or unqualified

- **M2 and M3:** complete correct replies and durable memory, not just next-token scores.
- **M4:** the accepted model's full `UORLUT01` export, quality, session and audit path. A lawful
  likelihood (`PRLL`) exists, but GPU held-out scoring remains open.
- **A joined model:** the discrete M1 path and FBS trunk do not share a table, trunk or served session.
- **M5:** whole-path latency, memory traffic and energy; arithmetic counts alone do not establish them.
- **M7:** library/CLI/browser parity on the accepted model, not only a synthetic stand-in.
- **Learned address formation:** the true-address 80/80 oracle is exact-read evidence, not proof
  that the model has learned to form addresses or earned M3.

## Historical K16, kept in scope

K16-A scored 65 and 463 of 14,287, below unigram 1,226 and T2 6,300. K16-R collapsed to 19 or 20
occupied cells. Those negative configurations remain important evidence, but they predate the M1
close. The older address-key, read, ranking and schedule hypotheses belong to that history; they do
not erase the later saved model's result ([K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md)).

## Go deeper

- [How the model learns](./training.md) and [serving and export](./serving-and-export.md).
- [Pipeline map](pipeline-map.md) for each stage's domain, size, hardware and cost.
- Mechanisms: [discrete learning](../mechanisms/discrete-learning.md), [admission](../mechanisms/admission.md), [D11 serving](../mechanisms/lut-serving.md), and [the mechanism board](../mechanisms/README.md).
- Code: [`discrete.cu`](../../crates/uor-r4-discrete/src/discrete.cu), [`metadata.rs`](../../crates/uor-r4-discrete/src/metadata.rs), [`device.rs`](../../crates/uor-r4-discrete/src/device.rs), and [`discrete_d11.rs`](../../crates/uor-r4-integer/src/discrete_d11.rs).
- Records: [M1 close](../labs/m1-close-saved-model/README.md), [bar-f](../loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md), and [claude-217](../loop/tasks/one-model-end-to-end/claude-217/REPORT.md).
- Terms: [glossary](../start/glossary.md).
