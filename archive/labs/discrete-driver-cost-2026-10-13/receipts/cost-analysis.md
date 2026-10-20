# D2 exact cost analysis — discrete driver A1

Source `e7d926fbe38537f4faa90b817705c61a780778d6`; release executable SHA256 `399ab42713589439dd963ec50515133e723abda79cd4b1d90330d9710d80c117`. Measurement JSON SHA256 `c39eb8fcd02378b534e573fb5709e2ab3d13cb02addb21279c0c53f790e6bab6`. Device: RTX5090. Price supplied by root: **908 milliUSD/hour = 908/1000 USD/hour**.

This analysis reads completed GPU receipts and performs only integer/Rational report arithmetic in Ruby; no model/reference, test, corpus, source change or new GPU execution occurred. Independently checked: all raw sample counts, integer minima/medians, floor rate calculations, projected nanoseconds, repeated proposal observations and qualification counts. No numeric JSON value is floating-point.

## What actually completed

- Qualification receipts: eleven Route PASS/updates0; six Flock PASS/updates0; eight driver PASS/updates8/apply_calls11; zero failed or NOT_RUN. The eight applications are disposable literal-fixture construction updates, not corpus learning.
- `measurement.json`: PASS, error null, training HELD, corpus false, applied_updates0. Thirty attempted timed API calls all completed; twelve attempted timed proposals all completed.
- Each mode has exactly three samples for carry, forward, evaluate and each of its two proposal families. One separate warm-up exercised each entry. All recorded proposal epochs remain0 and every proposal says applied:false. Repeated candidate/before/after observations agree exactly.
- `PINS.json` identifies prospective exact-head FIT confirmation6158277830 as the independent non-author Codex review, not Claude. Claude's conditional design review6158017847 is a separate receipt.

Shape: vocabulary4096, blocks4/width32, context192, writer kept cells8, rows64, seed7. Each forward handles **12288 physical context tokens and64 target outputs**. Flock neighbour kept count8 is separate from writer kept cells8. Active Flock uses fixed authored gates+1 with the seeded codebook; no gate/codebook update occurs in measurement.

## Exact median API timings and rates

All durations include the synchronized native API's allocation/readback/driver overhead. Rates are integer floors using `work_units*1000000000/median_ns`.

| Mode / operation | Median ns | Min ns | Median duration | Context tokens/s floor | Target rows/s floor |
| --- | ---: | ---: | --- | ---: | ---: |
| Route carry | 1314284 | 477648 | 1.314284 ms | 9349577 | 48695 |
| Route forward | 37837521 | 37807630 | 37.837521 ms | 324757 | 1691 |
| Route evaluate | 42143551 | 42133704 | 42.143551 ms | 291574 | 1518 |
| Route parameter proposal | 3959623663 | 3959484718 | 3.959623663 s | 3103 | 16 |
| Route VectorBlock proposal | 3903280515 | 3903199889 | 3.903280515 s | 3148 | 16 |
| Active Flock carry | 435395 | 433648 | 0.435395 ms | 28222648 | 146992 |
| Active Flock forward | 40576018 | 40536348 | 40.576018 ms | 302838 | 1577 |
| Active Flock evaluate | 44765762 | 44751584 | 44.765762 ms | 274495 | 1429 |
| Active Flock Gate proposal | 4180529788 | 4180170338 | 4.180529788 s | 2939 | 15 |
| Active Flock VectorBlock proposal | 4206217974 | 4204339674 | 4.206217974 s | 2921 | 15 |

**Context tokens/s are batch-prefix processing rates, not autoregressive decoding tokens/s.** Target rows/s count independent supplied-prefix next-token outputs; no generation loop, growing context or re-carry-on-generated-token throughput was measured. Evaluate additionally performs global top8 and integer aggregation; its rate is kept separate from plain forward. Proposal rates describe one selected parameter's candidate search on64 rows, not accepted learning updates or corpus training tokens/s.

Carry samples vary more than the compute samples (Route477648–1572698ns; Flock433648–897188ns). Three repetitions are observations at one fixture, not confidence bounds, cross-seed reproducibility or general throughput guarantees. Gate3 is not proportionately faster than Vector16 in these observations; the measured times must be used rather than scaling by candidate count. This report assigns no unmeasured hardware cause.

## Fixed-batch parameter-pass projection and price

The pinned inventory is S5, VB16384, BS20, total16404. Source/report formula:

`projected_ns = 16384*median_VectorBlock_ns + 20*median_Route_or_Gate_ns`.

| Mode | Exact projected ns | Exact projected seconds | Hours (decimal truncated9) | USD at908milliUSD/hour (decimal truncated9) |
| --- | ---: | ---: | ---: | ---: |
| Route | 64030540431020 | 64030.540431020 | 17.786261230 | 16.149925197 |
| Active Flock | 68998285881776 | 68998.285881776 | 19.166190522 | 17.402900994 |

Exact monetary fractions: Route `726746633892077/45000000000000 USD`; Flock `978913180947697/56250000000000 USD`. These use only integer/rational arithmetic: `USD = projected_ns*908/(1000000000*3600*1000)`.

**Both projections remain UNVERIFIED.** They extrapolate one sampled VectorBlock and one sampled read parameter across a full inventory on one fixed64-row batch. No full parameter pass ran; these are not a corpus pass, two-seed budget or selected training dose. They omit accepted-update/re-carry costs, changing geometry/eligibility, checkpoint cadence, corpus iteration, development/fresh-transfer evaluation, setup/build and preservation. The eventual one-coordinate-per-batch driver must bind its actual corpus/schedule before any training projection. Initial fixture/rival distribution and GPU occupancy may differ in a learned model.

The recorded measurement-controller interval is66.633432500s. At this price, that interval alone is exactly `6050315671/360000000000 USD`, approximately `$0.016806432` (truncated9). **It is not total session cost.** Root separately reports2544s provider uptime through00:48:44Z: `2544*908/3600000 = $0.641653333…`; setup/build/qualification/archive/provider idle time remain charged. Root reports prior cumulative cost around$1.078, bringing cumulative cost around$1.720; those cumulative amounts are root ledger inputs, not reconstructed from measurement JSON.

## GPU memory and setup scope

Both modes begin at33136836608 free bytes of33668726784 total. Route post-model delta33554432bytes/32MiB; active Flock35651584bytes/34MiB. Post-warm-up free-memory changes from each mode's baseline:

- Route:4244635648bytes =4048MiB; free28892200960bytes.
- Active Flock:4246732800bytes =4050MiB; free28890103808bytes.

The per-repetition after-carry/evaluation points remain at the respective post-warm-up free-byte values. **These are whole-device point samples, not peak memory and not a measured allocation solely attributable to model parameters.** Driver/context/local-memory reservation can contribute; no attribution was measured. Do not infer a consumer-device or energy guarantee from them.

Both modes report per-thread `CU_FUNC_ATTRIBUTE_LOCAL_SIZE_BYTES`: forward16624, counterfactual17136, evaluation16784. The compiled dynamic mode dispatch has the same attributes in both observations; these are function attributes, not peak resident usage.

Separate context setup/load-JIT-init/warm-up durations:

| Mode | Context/memory setup ns | Combined load/JIT/init ns | Warm-up ns |
| --- | ---: | ---: | ---: |
| Route | 271459155 | 136535238 | 7949718160 |
| Active Flock | 197416863 | 127749825 | 8476495931 |

The load timing does not isolate only JIT, and Flock includes its authored active-gate snapshot/reload. Snapshot-before/after setup events are recorded separately outside sample statistics. Warm-up is charged: four proposal attempts/51 active candidates,128 forward rows,128 evaluation rows and24576 carried physical tokens. Timed capacity is twelve proposal attempts/153 active candidates/9792 candidate rows,384 forward rows,384 evaluation rows and73728 carried tokens. Baseline replay is additional actual work within proposal time. The report labels these capacities rather than using them to fabricate completed work after a failure; here all expected API calls completed.

## Kimi inputs and limitation

The immediate cost finding is **about3.9–4.2s per coordinate proposal**, with the current full-inventory fixed-batch extrapolation **about17.8–19.2h / $16.15–$17.40**. Plain forward is about37.8–40.6ms for64 target outputs at this shape. This resolves the assumption that inference-like throughput also prices coordinate-search learning. Choose a source-bound schedule and affordable dose from these observations; do not launch an unspecified full sweep, infer language quality from synthetic rank counts, or treat current implementation cost as a mechanism negative. No optimization campaign or new compute is recommended by this report.

All measurement aggregates have64 token-eligible and64 address-eligible rows,512 admitted cells and denominator256 per row; they are not vacuous zero-rival timing fixtures. Route repeated total numerator29184, active Flock63296. Virtual observations are Route7:29184→28928, Route-Vector CURRENT29184→29184, Gate2:63296→58944, Flock-Vector7:63296→63232. These are repeated virtual fixture costs with no committed updates, not language/learning evidence.

Source, price, GPU, initial fixture, mode/gates, candidate inventories, shape, readback/synchronization protocol and sampler count must accompany these numbers. Training and M2 remain HELD, and no full E8, D11, useful-chat, energy or decoding claim follows.
