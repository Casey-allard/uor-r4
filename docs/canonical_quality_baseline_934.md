# Canonical quality-baseline genealogy and remediation order (#934)

- **Parent / stage:** [#821](https://github.com/UOR-Foundation/uor-r4/issues/821),
  S0 truth and inference closure.
- **Status:** audit complete at teacher-free certifier/reachability scope; normative
  deployed-serving quality remains **NOT ESTABLISHED**.
- **Canonical subject:** `smollm2-360m-broad-clean`, full 72,130-position D3
  held-out census, recorded teacher-argmax labels, greedy top-1.
- **Source revision audited:** `9aeb5427c7fd3a965b0e4a0228f137889dbafa9b`.
- **Machine-readable companion:**
  [`canonical_quality_baseline_934_result.json`](canonical_quality_baseline_934_result.json),
  schema `uor-r4-canonical-quality-diagnostic/1`.
- **Evidence policy:** append-only. Historical rows below retain their original
  scope and verdict. Later measurements append a dated outcome; they do not
  replace this genealogy.

## Decision in one sentence

There is **no universal absolute 30% requirement** for the canonical broad
bundle. The nearby numbers belong to different distributions, selectors, and
release decisions: **29.7%** is a legacy-fixture tolerance, **28.121%** is the
canonical bundle's same-report TLA comparator, and **29.702%** is an empirical
`R4Engine` skip-mix result whose gate was a paired **+20 permille** effect over
its sections-absent control.

The current canonical Rule 1+2 row is 17,595 / 72,130 = **24.393%**, below its
same-position TLA comparator at 20,284 / 72,130 = **28.121%** by 2,689 hits or
**37.280 permille**. That is the quality deficit to remediate. It is not a
failure to clear a manufactured absolute 30% floor.

## What this audit did and did not run

This run was teacher-free. It re-hashed the local canonical source, corpus,
tokenizer, artifact, cover, graph, score-report, and release-manifest bytes;
checked them against the #833 identity chain; parsed the canonical graph to
inventory optional sections; and read the already-produced schema-26
full-census `score_report.json`. It did **not** run a teacher forward, observation,
corpus rebuild, canonical Gate C score pass, or parity suite. Consequently the
canonical counts below are a current identity-verified read of #833's existing
full-census report, not an independent Gate C reproduction.

After that audit, the separate #908 teacher-free harness was rerun under its
original `R4Engine` reference/off-serving scope. It completed PASS in 128.4 s
with identical counts, base graph CID
`blake3:aaf98b68a78dd615f06dbb727a22dc4e170a152f055313fcc4fa574309f42d1e`,
skip graph CID
`blake3:19eb04d7dbf3fccd126069982ad8cbc1de31d536fff7e77ef2dacb26e64106cc`,
and result CID
`blake3:e32e4e33d70f342ae3c0913ba00d9aef0cf789b539b9e1b658a9366c51402a26`.
That is a current reproduction of #908 within its declared scope. It does not
turn the different canonical release graph into a sections-present artifact and
does not establish `R4G1Runtime` production quality.

## Canonical input identity

| Input | Bytes | Verified BLAKE3 CID |
|---|---:|---|
| SmolLM2-360M source `model.safetensors` | 723,674,912 | `blake3:eb23c3e8527110b83c091f8660aba676ec4993c9212a9e147503878d6087191f` |
| `corpus.meta` | 25 | `blake3:aa9d176779c1d2411e872c49c95ed585ee805ded5fa1b808ddf2f517a245b0ce` |
| `corpus.records` | 31,761,312 | `blake3:4692307368fecce481a4aac452fd8df4e63d2f1bd07ee0f2932108f8595f8f62` |
| aggregate corpus identity | - | `blake3:7db27ffb488ad996f2317c99f3eb627ca964b28c3e730d050d1e51136c7a335e` |
| `tless_artifacts.bin` | 1,415,444 | `blake3:6324aabec22fca5af371333cefc206f9b6762bfb52dccfb8efa0dc8fe5a1efaa` |
| `tokenizer.bin` | 528,975 | `blake3:70af0cb08bbcd3b323d3387ca1d7d33da39873820604d183711e8e99f9903fc1` |
| raw `tokenizer.json` definition | 2,104,556 | `blake3:944d1262d516abd56a8156dd3058a73a1bf3dc19419527592d854d162f288073` |
| `graph-cover/cover.r4g1` | 192,432 | `blake3:df6902bc3ed3e5dee2f93fd8cc08b0614f298a6c059cb5806e9bda0b9f61520b` |
| `graph/score.r4g1` | 29,871,608 | `blake3:bc2366f18377718da744f72fe03648b09df1a9cf320ffceadf656c7e94aa9d48` |
| report-declared graph kappa | - | `blake3:169cbeb8374d2ab4a905eb744c814b5c805b99ff678c439a6492a77c29062ddb` |
| `graph/score_report.json` | 24,593 | `blake3:a26522bffa999de36737520bdc92367922ec1b63a3d1b77ec0095a6a7154e3ea` |
| `release-bundle.json` | 1,736 | `blake3:e274c7e674f7e6d3a9039632d79e427a37db5aeb9ddd06bdf96c9859b23c9f69` |

The tokenizer adapter is `hf-byte-bpe/1`, with adapter digest
`blake3:1a6ab67d2145f8f96989f529f787fc74b59952ea1be6739b612f041a15f00b5e`.
The score report declares `quality_profile = "relative_tla"`, population
72,130, and `positions_sampled = 0`. These are the #833 canonical broad-clean
identities, not the 135M BDD fixture identities.

## Threshold provenance

### The exact genealogy

| Date / source | Number | Metric, population, and selector | What it governed | Current reading |
|---|---:|---|---|---|
| 2026-07-22, legacy #65/#64 fixture chain | 31.7086% top-1; 9.8612 bits/token | 30,036 held-out positions from the legacy stories15M fixture; Rule 1+2 certifier row, argmax-identical to its TLA3 row | Historical M.V.G. Gate C anchor | A distribution-specific empirical anchor, not a cross-model constant. See [`BASELINE.md`](transformerless/BASELINE.md) and [#64](https://github.com/UOR-Foundation/uor-r4/issues/64). |
| 2026-07-23, [#110](https://github.com/UOR-Foundation/uor-r4/issues/110) / [PR #111](https://github.com/UOR-Foundation/uor-r4/pull/111) | **29.7%** and 9.96 bits/token | `31.7% - 2 percentage points`; `9.86 + 0.10`, applied by `validate_quality_report` to the pinned/legacy profile | Artifact-load regression tolerance for reports comparable to the legacy fixture | This is a real absolute floor in source, but its declared scope is the pinned legacy profile only. It does not apply to the canonical broad bundle. |
| 2026-07-23/24, commit `05c08f3d` / [PR #121](https://github.com/UOR-Foundation/uor-r4/pull/121) | `relative_tla` | Dynamic Hugging Face reports compare Rule 1+2 top-1 with the TLA row measured on the same corpus; on this profile the legacy absolute top-1 and bits tolerances are skipped | Dynamic-build admission semantics | The graph-versus-TLA comparison is evaluated first and remains binding. The canonical broad report therefore requires at least its own **28.121%**, not 29.7% or 30%. See [`engine.rs`](../crates/uor-r4-api/src/engine.rs). |
| 2026-08-08, [#509](https://github.com/UOR-Foundation/uor-r4/issues/509) | **29.0%** | Latent-mix, left-key-only empirical arm on 4,358 held-out positions from a thin 21,235-record SmolLM2-360M Simple-Wiki observation; Rule 1+2 was 10.19%, TLA 25.70% | Decided whether a competent broad teacher warranted the broad-corpus programme | **POSITIVE but limited.** It was an empirical mechanism result, not an admission threshold and not the later dense canonical population. See [`smollm2_teacher_baseline_320.md`](smollm2_teacher_baseline_320.md). |
| 2026-08-09, [#516](https://github.com/UOR-Foundation/uor-r4/issues/516) | 24.30% Rule 1+2; 31.48% best-live; 28.21% TLA | Dense 360M broad re-pin, 72,864 held-out positions; `relative_tla` | Adopted the 360M broad distribution as the working broad-text baseline while retaining stories15M for continuity | Re-pin decision, not an absolute 30% floor. “Best live” is a conditional arm/slice and is not interchangeable with full-population Rule 1+2. |
| 2026-08-18, [#783](https://github.com/UOR-Foundation/uor-r4/issues/783) | **25.648%** Rule 1+2 vs **30.118%** TLA | Full 72,258-position report for refreshed `smollm2-135m-instruct`; `relative_tla` | Fresh 135M bundle read and later #932 preflight | The 30.118% number is that bundle's comparator, not a universal floor. It belongs to a different teacher, corpus, artifact, and BDD role. |
| 2026-08-20, [#833](https://github.com/UOR-Foundation/uor-r4/issues/833) | **24.393%** Rule 1+2 vs **28.121%** TLA; best-live 31.11% | Clean #755-native 360M broad full census, 72,130 positions | Source/corpus/determinism attestation and M.V.G. baseline retention | The attestation remains valid. Its `load_accepting_quality` canary did not establish strict admission, and the full-population Rule 1+2 row fails `relative_tla`. See [`attested_broad_baseline_833.md`](attested_broad_baseline_833.md). |
| 2026-08-21, [#908](https://github.com/UOR-Foundation/uor-r4/issues/908); rerun 2026-08-24 | **29.702%** skip; 26.857% base; paired **+28.449 permille [25.574, 31.323]** | Re-emitted experimental SKMX/PSIB and sections-absent graphs, full 72,130 positions, independently selecting `R4Engine` | RF-31 promotion criterion: paired lower bound at least **+20 permille** over the exact sections-absent control | **POSITIVE and reproduced at reference/off-serving scope.** The 128.4 s teacher-free rerun returned identical counts, graph CIDs, and result CID `e32e4e33...`. The 29.702% observation is numerically close to 29.7% by coincidence; it is not the same metric, artifact, selector, or decision gate. See [`skipmix_endtoend_causal_908.md`](skipmix_endtoend_causal_908.md). |
| 2026-08-21, [#910](https://github.com/UOR-Foundation/uor-r4/issues/910) | RF-31 registered as normative/deployed | Compiler emits SKMX/PSIB and `R4Engine` consumes them | Intended activation | **Scope correction required.** `R4G1Runtime` did not consume the sections, the canonical release predates them, and production surfaces were not one candidate owner. |
| 2026-08-24, [#933](https://github.com/UOR-Foundation/uor-r4/issues/933) | `delta_tla >= 0`; paired `delta_lane` lower bound at least +20 permille; zero surface mismatch | Proposed CID-bound `R4G1Runtime` deployed-quality profile | Restore one normative selector and strict production admission | A local checkpoint exists, but it is unmerged, lacks canonical evidence, and has no empirical verdict. #934 supplies this missing quality rationale. |
| 2026-08-24, [#932](https://github.com/UOR-Foundation/uor-r4/issues/932) | 25.648% vs 30.118% | Teacher-free preflight of the distinct 135M parity bundle | Admission before opening the live teacher | **REFUSED / NOT_RUN.** Zero teacher forwards; no threshold was lowered and no historical bundle was substituted. |

### Authoritative answer

The repository contains a scoped legacy absolute tolerance of 29.7%; it does not
contain an evidence-backed universal absolute 30% requirement. For the canonical
360M broad report, the current binding comparison is `relative_tla`, so the
immediate empirical target is non-negative paired movement relative to **28.121%**
on the same positions. RF-31 separately retains its **+20 permille paired lower
bound** against sections absent. Any future absolute release floor requires a
new distribution, selector, power study, and pre-registered governance decision;
it cannot be inferred by rounding one of these numbers.

## Append-only research ledger

The table distinguishes mathematical/reference evidence from compiler,
artifact, runtime, and production reachability. “Implemented” never means
“production eligible.” The detailed history remains in [`RESEARCH.md`](RESEARCH.md)
and the linked per-issue records.

| Mechanism / record | Evidence and exact disposition | Scope and reachability consequence |
|---|---|---|
| Plain TLA store / legacy comparator | **RETAINED as comparator.** The canonical TLA3 row is 20,284 / 72,130 = 28.121%. | Compiler/artifact/reference path present. It is not the ADR-0001 production candidate owner and does not by itself establish a served-token result. |
| Original sum-over-cloud graph, [#64](https://github.com/UOR-Foundation/uor-r4/issues/64) | **NEGATIVE / retired scoring formula.** Correlated sibling residuals collapsed the legacy fixture to about 0.3% top-1. | Reference arithmetic failure; replaced by chain-telescoped Rule 1 plus EXCT precedence. |
| Rule 1 chain + Rule 2 EXCT precedence, #64 | **POSITIVE on the legacy fixture; LIMITED on broad text.** It restored 31.7% on an all-EXCT fixture. The canonical broad full census is 24.393%, with 96.24% of its correct predictions coming from EXCT positions. | Compiler, artifact, certifier, and runtime mechanisms exist. Current broad strict admission fails its same-report TLA comparator. |
| Transition/F emissions, [#66](https://github.com/UOR-Foundation/uor-r4/issues/66) | **NEGATIVE for the implemented rank-neutral offset; dropped as deployed default.** It changed bits without top-1 value on the measured fixture. | Edges remain representable for structure/witnesses. No current baseline-remediation credit is assigned to the retired uniform offset. |
| Emission selection/calibration, [#364](https://github.com/UOR-Foundation/uor-r4/issues/364) | **MIXED / limited.** Probability selection fixed 1.57% mass coverage, contrast weighting recovered ranking value on a real cover, and Witten-Bell count shrinkage was effectively inert. Later real-bundle work revised fixture-only “parity ceiling” language. | Compiler/certifier options exist. The canonical artifact declares `ratio` selection and `none` shrinkage. Historical opt-in rows do not authorize a production default or close the 2,689-hit deficit. |
| Reconstructability and IPF, [#456](https://github.com/UOR-Foundation/uor-r4/issues/456) / [#457](https://github.com/UOR-Foundation/uor-r4/issues/457) | **NEGATIVE.** EXCT-disabled reconstruction was sub-unigram; IPF reconciled marginals but landed at the unigram null. | Certifier/reference evidence only. The mechanisms were not promoted into canonical serving. |
| Cover capacity/codebook fit, [#460](https://github.com/UOR-Foundation/uor-r4/issues/460) | **LIMITED.** Scaled cover capacity moved region-path top-1 +5.0pp, but historical headline reachability was capped near 0.15pp; codebook fit added +0.44pp, below its exit rule; added key resolution otherwise measured negative. | Useful compiler instruments remain. Prior evidence rules out indiscriminate region/key expansion as a sufficient baseline fix. |
| FWDA/right-context, [#399](https://github.com/UOR-Foundation/uor-r4/issues/399) | **Mode-specific positive; generation negative.** Supplied-anchor A-mode infill is retained. Self-drafted two-pass generation went negative and stayed negative under a strict confidence gate. | FWDA is valid for explicit infill where the anchor is input. Oracle/right-context rows are not causal left-to-right production evidence. |
| Two-sided/latent classes, [#446](https://github.com/UOR-Foundation/uor-r4/issues/446) / [#509](https://github.com/UOR-Foundation/uor-r4/issues/509) | **POSITIVE at thin reference scope; LIMITED at canonical scale.** The thin 29.0% latent-mix row warranted the broad programme. In the canonical score report latent-mix is 18,647 / 72,130 = 25.852%, a +14.584 permille movement over Rule 1+2, below the retained 20-permille causal floor; the report records `latent_exit_rule_met = false`. | Certifier arm, not a packed production section. Oracle-right is explicitly non-causal; shuffled-class is a null. |
| Broad teacher swap and dense re-pin, [#509](https://github.com/UOR-Foundation/uor-r4/issues/509) / [#516](https://github.com/UOR-Foundation/uor-r4/issues/516) | **POSITIVE for programme direction and re-pin.** A competent 360M broad teacher lifted the substrate far above the narrow teacher's broad-text floor; dense observation raised Rule 1+2 from 10.2% to 24.3%. | Establishes the canonical broad distribution and teacher/corpus choice. It does not establish parity, universalize 29.0%, or replace the continuity baseline. |
| Corpus ordering and bundle refresh, [#755](https://github.com/UOR-Foundation/uor-r4/issues/755) / [#783](https://github.com/UOR-Foundation/uor-r4/issues/783) | **POSITIVE mechanism fix; quality still limited.** Sorting by `(story, span_start)` removed a real corrupt-context failure. Refreshed 135M Rule 1+2 remained 25.648% vs TLA 30.118%, and production-quality import stayed fail-closed. | Compiler fix is current. The 135M artifact is a separate BDD subject and cannot substitute for the 360M canonical baseline. |
| Normative scorer and evaluation constitution, [#831](https://github.com/UOR-Foundation/uor-r4/issues/831) / [#832](https://github.com/UOR-Foundation/uor-r4/issues/832) | **POSITIVE specification and evidence contract.** ADR-0001 designates `R4G1Runtime`; the CID-bound suites freeze workload, partition, scorer, attribution, null, and absent-fixture semantics. | Establishes what production evidence must bind and how absence is reported. It does not itself establish the canonical artifact's quality, make #910's lane reachable, or turn a reference selector into serving evidence. |
| Canonical clean rebuild, [#833](https://github.com/UOR-Foundation/uor-r4/issues/833) | **POSITIVE attestation; strict quality NOT ESTABLISHED.** Source completeness, corpus integrity, deterministic compilation, and full-census score rows remain valid. | Canonical release is current and content-bound, but predates SKMX/PSIB. The local accepting-quality canary was a bypass, not strict admission. |
| Prompt-conditioned arms, [#834](https://github.com/UOR-Foundation/uor-r4/issues/834) | **MIXED, then REVISE.** Current/longer local context showed no prompt-conditioning; a segment reference arm recovered +17.5 permille and conditional residuals +16.2 permille, both below the frozen 20-permille floor. | Reference signal existed, but the measured effect could not clear the promotion gate. No current canonical production credit. |
| Segment-lane lowering, [#836](https://github.com/UOR-Foundation/uor-r4/issues/836), [#886](https://github.com/UOR-Foundation/uor-r4/issues/886), [#887](https://github.com/UOR-Foundation/uor-r4/issues/887) | **RETIRED from promotion track.** Bounded lowering was exercised; the reference ceiling itself sat below the unchanged gate, and governance retained the 20-permille floor. | Optional/dormant machinery is not a canonical active lane. The threshold was not lowered after the result. |
| Skip-mix fit/lowering, [#897](https://github.com/UOR-Foundation/uor-r4/issues/897) / [#904](https://github.com/UOR-Foundation/uor-r4/issues/904) / [#906](https://github.com/UOR-Foundation/uor-r4/issues/906) | **POSITIVE structural diagnosis and repair.** The initial 41/87 lowering gap was candidate-breadth-bound; SKMX/PSIB candidate injection lifted favorable-pair follow to 58/87 and cleared the predeclared fidelity bar. | Fit/emit and `R4Engine` consumption exist in current source. This still was dormant, reference/off-serving work and not canonical `R4G1Runtime` evidence. |
| Full skip-mix causal run, [#908](https://github.com/UOR-Foundation/uor-r4/issues/908) | **POSITIVE and reproduced at reference/off-serving scope.** Base 19,372, skip 21,424, null 2,281; paired +2,052 hits and +28.449 permille [25.574, 31.323], with the shuffled null collapsing. The current 128.4 s rerun reproduced the committed counts, graph CIDs, and `e32e4e33...` result CID. | Different re-emitted graph identities and independently selecting `R4Engine`; not evidence for the current canonical graph or normative production selector. |
| RF-31 activation, [#910](https://github.com/UOR-Foundation/uor-r4/issues/910) | **CORRECTED / NOT ESTABLISHED at claimed scope.** Compiler emission and the `R4Engine` reroute landed; registration overstated `R4G1Runtime` and cross-surface reachability. | The correction is owned by #933. Historical #908 measurements remain valid under their narrower scope. |
| Content calibrator, [#931](https://github.com/UOR-Foundation/uor-r4/issues/931) | **NEGATIVE.** No selectable arm cleared frozen CAL release or research gates; TEST remained sealed. | Does not supply a ranking/calibration remediation for the canonical deficit. |
| Route-attention alternative, [#804](https://github.com/UOR-Foundation/uor-r4/issues/804) | **DEGENERATE / unavailable for promotion.** The real-teacher instrument could not separate fitted routing from temporal-smoothness null behavior. | Source operator remains registered dormant and reaches no serving path. It is not a baseline remedy. |
| Exact live-teacher BDD work, [#932](https://github.com/UOR-Foundation/uor-r4/issues/932) | **REFUSED / parked.** The 135M graph failed teacher-free `relative_tla` preflight, so the expensive teacher path correctly remained NOT_RUN. | Verification infrastructure, not a quality mechanism; resumes after #933 restores an admissible normative bundle. |

## Canonical teacher-free diagnosis

### Same-position result and comparator cross-tab

**Empirical Criterion. Status: Empirical. Execution scope: teacher-free
certifier evidence.** Both rows use the same 72,130 positions and recorded
teacher argmax labels.

| Row | Correct | Top-1 | Bits/token |
|---|---:|---:|---:|
| Rule 1+2 precedence | 17,595 | 24.393456% | 12.020306 |
| TLA3 comparator | 20,284 | 28.121447% | 13.608812 |
| Rule 1+2 minus TLA | **-2,689** | **-3.727991pp** | not a paired bits decision |

| Same-position outcome | Positions |
|---|---:|
| both correct | 17,269 |
| Rule 1+2 only correct | 326 |
| TLA only correct | 3,015 |
| neither correct | 51,520 |

The TLA-only set has a maximum headline ceiling of 3,015 / 72,130 =
**41.800 permille**. After the 326 Rule-1+2-only cushion, parity requires a net
2,689-hit movement: a no-regression intervention confined to the TLA-only set
would need to recover 89.19% of it. The schema-26 report does not cross-tab those
3,015 TLA-only positions by status, candidate presence, active section, or rank.
That attribution is **UNAVAILABLE** from the committed report and is a required
field in the bounded normative replay; it must not be guessed.

### Resolution status and candidate headroom

Here “target present” means the recorded teacher argmax is present anywhere in
the Rule 1+2 candidate set. “Teacher top-3 present” means at least one of the
corpus-recorded teacher top-three tokens is present; it is not top-3 selected-token
accuracy.

| Status | Positions | Correct | Top-1 | Target present | Teacher top-3 present | Present but not selected | Median target rank when present |
|---|---:|---:|---:|---:|---:|---:|---:|
| ExactContext | 52,398 | 16,933 | 32.316% | 35,085 (66.959%) | 46,283 (88.330%) | 18,152 | 2 |
| Graph | 19,436 | 660 | 3.396% | 9,962 (51.255%) | 16,465 (84.714%) | 9,302 | 9 |
| Novel | 296 | 2 | 0.676% | 100 (33.784%) | 215 (72.635%) | 98 | 15 |
| **All** | **72,130** | **17,595** | **24.393%** | **45,147 (62.591%)** | **62,963 (87.291%)** | **27,552** | status-dependent |

Target-rank buckets are `[1, 2, 3, 4-8, 9-16, 17-32, 33-64, 65-128,
129+]`:

- ExactContext: `[16933, 5209, 2746, 5411, 2568, 1378, 840, 0, 0]`;
- Graph: `[660, 708, 935, 2526, 1472, 1079, 1018, 927, 637]`;
- Novel: `[2, 16, 5, 11, 22, 16, 10, 0, 18]`.

This separates two live deficits:

1. **Retrieval/candidate coverage:** the target is absent on 26,983 positions
   (37.409% of the population), so ranking-only work cannot touch them.
2. **Ranking:** the target is present but not selected on 27,552 positions
   (38.198%). On Graph positions only 660 / 9,962 = 6.63% of retrieved targets
   rank first, with median rank 9. Candidate injection alone cannot select those
   already-present targets correctly.

These are oracle ceilings, not promised improvements. Their overlap with the
3,015 TLA-only set is not present in schema 26, and a production intervention
can also create away transitions. The counts establish decision value for a
split diagnostic; they do not establish that either lever will close the gap.

### Probe depth, EXCT dominance, and residual arithmetic

The raw graded-prefix probe-depth histogram is
`[0, 1110, 9237, 9385, 52398]` for depths root through full code. Probe-absent is
zero; full-depth support is 52,398 / 72,130 = **72.644%**. Graph plus Novel
therefore own 19,732 positions = **27.356%**, so their population ceiling is
arithmetically large enough to close a 3.728pp gap.

Actual performance sharply limits that reading:

- ExactContext supplies 16,933 / 17,595 = **96.24%** of all Rule 1+2 hits.
- Graph plus Novel produce only 662 / 19,732 = **3.355%**, below the report's
  4.820% unigram top-1 on the same generalization slice.
- On Graph status the score report's residual-alpha sweep records root-only
  (`alpha = 0`) at 860 hits versus the shipped residual (`alpha = 1`) at 660.
  Suppressing that residual would add only 200 hits = **0.277pp** overall, far
  below the 2,689-hit deficit. Blanket residual reweighting is therefore
  **retired as a sufficient remediation**, even though a status-specific scorer
  may still be measured after candidate attribution.
- ExactContext moves in the opposite direction: its full residual raises top-1
  from 16.791% at `alpha = 0` to 32.316% at `alpha = 1`. A global residual-off
  change would discard the dominant positive lane.

The deficit is thus not explained by one unreachable graph path, one stale
threshold, or one scalar weight. The committed report supports a mixed diagnosis:
the broad graph/reference path is overwhelmingly EXCT-carried; its Graph/Novel
lane is quality-negative against a simple null; substantial target candidates
are both absent and under-ranked; and the strongest later candidate source is
absent from the canonical artifact. The exact TLA-only attribution remains the
next cheap falsifier.

## Current source-to-admission reachability map

This table describes `origin/main@9aeb5427`, the audited canonical artifact,
and current production evidence. The parked #933 checkpoint at `fb71e4ce` is
listed separately because unmerged code is not current production behavior.

| Mechanism | Current source | Canonical artifact | `R4G1Runtime` | Production surfaces | Strict admission / claim |
|---|---|---|---|---|---|
| TLA / EXCT evidence | Present | Present; artifact CID `6324aabe...` and EXCT-linked Rule 1+2 report | Consumed as bounded exact-context evidence | Reachable on some current paths | Comparator/evidence source only; the canonical report fails `relative_tla`. |
| Rule 1 graph chain / root prior | Present | Present in `score.r4g1` | Present | Reachable, but current surfaces do not yet share the #933 candidate-owner contract | Full-census certifier evidence exists; exact cross-surface served-token quality is NOT ESTABLISHED. |
| FWDA / right context | Present for explicit modes and certifier arms | Historical/certifier rows present; not a canonical causal next-token admission lane | Explicit infill support is separate from ordinary left-to-right selection | Supplied-anchor infill only | No credit toward ordinary generation; oracle-right is ineligible. |
| Latent mix / prompt reference arms | Certifier/reference implementations present | No active packed canonical production section for the reported arm | Not a canonical normative lane | Not active | Historical empirical evidence only. |
| SKMX/PSIB fit and emit | **Present** | **ABSENT / ABSENT** in canonical `bc2366f1...` graph | **Consumer absent** on audited main revision | `R4Engine` reference consumer exists; production ownership is split | Current canonical lane unreachable; #908 remains reference/off-serving. |
| RF-31 via #910 | Registration and `R4Engine` reroute present | Canonical release predates the sections | Not implemented in the normative selector | Not uniform across CLI, sampled, beam, HTTP, library, and WASM | Prior deployed-serving classification corrected to NOT ESTABLISHED. |
| #933 checkpoint | Local branch adds bounded SKMX/PSIB consumption, one-adapter design, report/admission schemas, and tests | No checkpoint-built canonical graph/report/manifest identity exists | Implemented in unmerged checkpoint source | Mechanical fixture coverage only; no canonical surface artifact | No final gate ladder, deterministic canonical rebuild, sample, census, package, or admission verdict. |

The current canonical graph's SKMX and PSIB sections both parse as absent. This
single artifact fact makes RF-31's quality effect unreachable even through a
runtime that knew how to consume the sections. Conversely, source support and a
sections-present experimental artifact would still not establish production
quality without the #933 selector, surface, identity, absent-section, witness,
and strict-admission bindings.

## The 360M canonical baseline is not the 135M BDD bundle

| Role | Canonical baseline audit | Exact-live-teacher BDD preflight |
|---|---|---|
| Bundle | `smollm2-360m-broad-clean` | `smollm2-135m-instruct` |
| Teacher | SmolLM2-360M broad teacher | SmolLM2-135M instruction teacher |
| Held-out population | 72,130 | 72,258 |
| Rule 1+2 / TLA | 24.393% / 28.121% | 25.648% / 30.118% |
| Corpus aggregate CID | `7db27ffb...` | `d36361f1...` |
| Artifact CID | `6324aabe...` | `487532dd...` |
| Graph/report role | #833 broad canonical attestation | #783-refreshed fixture for #932 parity infrastructure |
| Current decision | canonical remediation and #933 admission | teacher-free refusal; live tuner/parity NOT_RUN |

The #932 failure is valuable BDD behavior: it demonstrates fail-closed
preflight before teacher cost. It is not a measurement of the canonical 360M
baseline and cannot raise, lower, or replace the 28.121% broad comparator.

## Pre-registered bounded remediation order

The target is the normative `R4G1Runtime` selector on the exact canonical
population: first meet the same-position TLA comparator, then retain the RF-31
paired sections-present lower bound of +20 permille, with zero binding and
cross-surface mismatches. No phase below may exceed 60 projected minutes. A
phase that projects beyond that stops and requires a separately approved run
contract; no teacher, observation, source download, or broad parameter sweep is
authorized.

### 1. Complete #933 structural reachability and binding first

- **Metric/current value:** one normative candidate/token owner and complete
  binding; current canonical production verdict **NOT ESTABLISHED**.
- **Reachability arithmetic:** no quality gain can be credited while
  `R4G1Runtime` cannot consume SKMX/PSIB and the artifact has neither section.
- **Binding instrument:** absent-section candidate/token/status/witness identity;
  planted SKMX and PSIB reachability; zero cross-surface candidate mismatch;
  wrong-selector/report/graph/artifact/tokenizer binding rejection.
- **Proceed if:** every structural falsifier passes with zero mismatches.
- **If positive:** continue to deterministic canonical re-emission.
- **If negative:** record **NOT ESTABLISHED**, repair only the named structural
  failure, and do not run a quality sample or census.
- **Cost/claim boundary:** at most 60 minutes per bounded verification tranche;
  structural runtime/admission work only, no empirical quality promotion.
- **Why prior evidence did not decide it:** #908 used independently selecting
  `R4Engine`; #910 did not add `R4G1Runtime` section consumption or one
  cross-surface owner.

### 2. Re-emit deterministic sections-present and control artifacts

- **Metric/current value:** canonical SKMX/PSIB presence and byte identity;
  current canonical sections are **ABSENT / ABSENT**.
- **Reachability arithmetic:** an absent section has a zero-hit ceiling. The
  #908 reference experiment changed 39,360 positions, so a re-emitted lane has
  decision value but no canonical credit before measurement.
- **Binding instrument:** two immutable production-compiler builds at distinct
  worker counts; require byte-identical graph/report identities, SKMX and PSIB
  both present, plus same-generation sections-absent and rotated-label controls.
- **Proceed if:** all content-bound outputs reproduce and both real sections are
  present.
- **If positive:** run the deterministic 6,000-position sample.
- **If negative:** record **UNAVAILABLE** or **NOT ESTABLISHED** and stop before
  replay.
- **Cost/claim boundary:** at most 60 minutes; compiler/artifact determinism, no
  teacher and no production-quality claim.
- **Why prior evidence did not decide it:** #908's base, skip, and null graph
  CIDs differ from the canonical release; #833's graph predates #910.

### 3. Run the binding 6,000-position normative sample

- **Metric/current value:** normative top-1 versus same-position TLA and
  sections absent; current normative SKMX/PSIB result **NOT ESTABLISHED**.
- **Reachability arithmetic:** parity needs +2,689 net hits in the full
  population. The existing report exposes 26,983 target-absent and 27,552
  target-present-but-not-selected positions, while the TLA-only ceiling is
  3,015; the sample must cross-tab these sets instead of assuming overlap.
- **Binding instrument:** first 6,000 pristine positions through the exact
  normative selector, with status x TLA win/loss x candidate presence x rank x
  SKMX/PSIB contribution; exact internal absent-section census; shuffled-label
  control; witness replay; every production surface.
- **Proceed if:** cross-surface, binding, absent-identity, and witness failures
  are zero; shuffled effect is non-positive; paired TLA lower bound is at least
  zero; paired lane lower bound is at least +20 permille; and projected census
  time is at most 60 minutes.
- **If positive:** conditionally authorize the full census in step 5.
- **If negative:** emit the typed **LIMIT**, **RETIRE**, **NOT ESTABLISHED**, or
  **UNAVAILABLE** result and stop. The failing attribution chooses step 4's
  bounded child; it does not authorize threshold tuning.
- **Cost/claim boundary:** at most 60 minutes, teacher-free normative-runtime
  sample; never labeled full-census production evidence.
- **Why prior evidence did not decide it:** neither schema 26 nor #908 binds the
  TLA-only errors, section contribution, and served candidates to normative
  `R4G1Runtime` on the same graph identity.

### 4. Retrieval before ranking, only if the sample isolates the deficit

- **Metric/current value:** target candidate presence before selected rank.
- **Reachability arithmetic:** 26,983 / 72,130 targets are absent from the
  current Rule 1+2 candidate set and 27,552 / 72,130 are present but not
  selected. Each child must declare which disjoint sample ceiling it touches
  and show that ceiling can close its declared gap.
- **Retrieval branch:** if TLA-only errors lack the target, test only bounded
  SKMX/PSIB or other already-recorded candidate sources. Require enough
  same-position reachable errors to close the target without reducing the
  other-only cushion.
- **Ranking branch:** opens only when the target is already present. Test a
  bounded fixed-point status-specific scorer on those positions, with the
  identical candidates and paired loss accounting. Do not reopen blanket
  residual scaling: its recorded Graph ceiling is only +200 hits.
- **Binding instrument:** status-split candidate/rank replay on at most the
  same 6,000 positions, with an exact sections-absent comparator and an
  away-transition ledger.
- **If positive:** file exactly one implementation child with its frozen
  metric, control, and affected claim boundary, then return through steps 2-3.
- **If negative:** retire that intervention, retain the measured baseline, and
  do not broaden the search or weaken a gate.
- **Cost/claim boundary:** at most 60 minutes per mutually exclusive branch;
  no combined candidate-and-ranking change before each independent instrument
  has teeth.
- **Why prior evidence did not decide it:** #904/#906 established candidate
  breadth on favorable pairs, while the canonical score report establishes
  population-wide retrieval/rank headroom; neither provides their required
  TLA-only intersection through the normative selector.

### 5. Conditional full normative census

- **Metric/current value:** full-population Rule 1+2 versus same-position TLA
  and sections-present versus sections-absent paired effects.
- **Binding cheap instrument:** step 3; its typed `PROCEED` is mandatory.
- **Proceed if:** the sample still leaves both verdict branches reachable and
  the measured projection remains at most 60 minutes.
- **Positive verdict:** zero cross-surface/binding/witness failures, paired TLA
  lower bound at least zero, paired lane lower bound at least +20 permille,
  non-positive shuffled control, exact absent identity, and strict admission
  accepting only the correctly bound full-census report.
- **Negative verdict:** retain and publish **LIMIT**, **RETIRE**, **NOT
  ESTABLISHED**, or **UNAVAILABLE** with exact counts and identities; do not
  substitute #908 or a sample.
- **Cost/claim boundary:** at most 60 minutes, teacher-free. If the projection
  exceeds the cap, the full census is **NOT_RUN**.
- **Why prior evidence did not decide it:** no current result combines the
  canonical graph identity, normative selector, all production surfaces,
  strict admission, and both paired controls.

## Claims that remain unavailable

- No current evidence establishes an absolute 30% production floor for all
  models, corpora, or selectors.
- No current evidence attributes the 3,015 TLA-only errors by resolution status
  or candidate/rank cause; schema 26 lacks that joint table.
- No current canonical release evidence exercises SKMX or PSIB; both sections
  are absent.
- No current evidence credits #908 as `R4G1Runtime` deployed serving.
- No current evidence establishes the #933 checkpoint as ready, merged,
  deterministic on canonical outputs, admitted, or empirically positive.
- No current evidence makes the 135M BDD refusal a 360M canonical verdict.
- No hours-class teacher or parity run is authorized by this audit. Missing or
  stopped evidence is **UNAVAILABLE** or **NOT_RUN**, never PASS.

## Dated outcome append

### 2026-08-24 — teacher-free audit outcome

**Outcome: threshold premise corrected; canonical deficit empirically
established at certifier scope; remediation pre-registered; production quality
NOT ESTABLISHED.**

The audit found a scoped 29.7% legacy tolerance but no universal 30% rule. It
verified the canonical identity chain and read the existing full census: Rule
1+2 24.393% versus TLA 28.121%, with a -2,689-hit paired deficit. The report
shows material candidate-absence and under-ranking headroom, weak Graph/Novel
accuracy, and an insufficient +200-hit ceiling for blanket Graph residual
suppression. A separate 128.4 s teacher-free #908 rerun reproduced its committed
reference/off-serving counts, graph CIDs, and `e32e4e33...` result CID. Current
source can fit/emit SKMX/PSIB and `R4Engine` can consume them, but the canonical
graph has neither section and audited-main `R4G1Runtime` cannot consume them.
#933 therefore remains the first structural implementation, followed only by
the bounded artifacts, sample, isolated retrieval/ranking child, and conditional
census above. #932 remains downstream.
