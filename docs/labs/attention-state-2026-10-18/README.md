# State of attention — 18 October 2026

PM-commissioned independent synthesis (DeepSeek specialist, read-only, no compute) answering the
owner's question: six months in, where is attention? PM framing correction posted as
[#2029 6255963920](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6255963920).

**Scope.** "Attention" here means the model's context access: the `ReadSpec` that selects which
earlier positions the writer sees, plus the codebook that gives those positions an address. There
is no Q/K/V module and no transformer block. Pre-alpha: 0 of 8 milestones complete
(`docs/integration/NOW.md:11`).

## 1. Every attention mechanism, what it measured, what it never tested

| Mechanism | Actually measured | Never fairly tested | Status |
|---|---|---|---|
| **`ReadSpec::Route`** — learned orbit action selects a ternary-tree port, carries a leaf | Full dose, K=8, seed 1: **1,920** targets admitted, **top-1 149**, **top-8 747** of 14,287; **16 of 20** route updates accepted (`alias-census-REPORT.md:76,148`; `m1-discrete-rung0-admission-redirect-2026-10-15.md:31`) | A read-off/read-on ablation; any full dose at K=16; more than **20 read proposals in 16,404 updates** (`finite-relational-head.md:156-159`). Its contribution to the 149 is unattributed | **active** |
| **`ReadSpec::Flock`** — murmuration rank read, no softmax | 6/6 finite CUDA construction controls; **never trained on the corpus** (`mechanisms/README.md:40`; `starling-read.md:26-28`) | Any corpus training at all | **active source, untrained** |
| **`ReadSpec::Relation`** — Flock + 5 edge-type trits (copy-successor primitive) | 9/9 construction controls; one trained arm, Q1 stop at 4,096 batches: admission **2,034 → 225** (control 205), **94 relation trits accepted** (`attention-rows-q1-diagnosis-2026-10-18/README.md:10,80-89`) | Full dose, accuracy, any paired Relation-vs-FrozenZero verdict. "No trained run yet isolates whether the read can learn" (`finite-relational-head.md:182-190`) | **active; verdict NOT_RUN** |
| **Starling / 600-cell read** | Rungs 1–3: source enters kept-7 **73%** at peak vs 10% init, **43–56%** on the hard read, recall still at chance; content gate recall **0.010**; training-only softmax surrogate ranks source first **0.55–0.78** and recall rises only to **0.044/0.288/0.091** (`starling-read.md:41-53`) | The lawful no-softmax decisive run (rung 0b / murmuration §5) | **dormant** (float source; all rungs measured beside softmax) |
| **Flock / rank-table read (M4 float stack)** | 19.93M params, 2 seeds, softmax baseline **0.8772 BPB**: fixed rank **+0.040**, Hamming **+0.259**, learned rank table **+0.0185** (54% of gap) against a **0.01** bar (`flock-rank-reads.md:12-17`) | Rank-consistent gradient (training used a flock-softmax straight-through), GPU kernel, long context, K sweep | **dormant / archived-negative at config** |
| **Invariant-addressed read** (route by address, not similarity) | Lorentz vs l2 geometric read: wrong-value **49→32/120**, memory **48→59**, below its pre-registered per-seed margin; geometric ops were **0.05%** of stack arithmetic (`invariant-addressed-read.md:24-30`) | Addressing as addressing, with tangent and normal nulls | **dormant** |
| **Exact addressed memory / product-key** | Product-key MLP replacement: wrong-value **47→34/120**, memory flat **57/57**, **+36.9M** params, ~**20×** training wall; the record's own correction: the read was never replaced (`exact-addressed-memory.md:20-27`) | Address-instead-of-search in the model/read path | **dormant** |
| **Pointer-copy with identity key** | Real-key run: hit **0.377→0.54**, memory **19/21/23** vs **20/40** control, wrong-value **17/15/9** vs 17; bar not met (`pointer-identity-key.md:16`) | The forced-copy oracle, and the entity-address key | **archived-negative at config / dormant** |
| **Read binding** (supervised steering of the read) | 3 seeds from step 0: bound mass **0.732→0.896/0.899/0.905**, wrong-value **47→30/120**, memory **57→65/120**, BPB at or below anchor — KEEP (`read-binding.md:11`) | Any discrete/geometric read; it is a softmax read | **dormant (float/softmax), re-registrable under D37** |
| **Prime-zeta injective codebook** | 4,096 vectors, **256/256** cells, DEV alias ceiling **14,287/14,287**, **0** tie mass; 17+6 construction controls PASS (`m1-attention-injective-init-DRAFT-2026-10-18.md:32-40`) | Trained attribution; the attention sheet is a DRAFT, `TEST FITNESS: PENDING` | **constructor active; attribution dormant** |

The one powered result that a learned read *can* move a headline is float read binding (47→30/120
wrong-value, 3 seeds) — in a form D23/D32 made inactive. Nothing in the current integer substrate
has that result.

## 2. The measured bottleneck

The census confirms all three numbers, with one correction of framing:

- **16 distinct vectors at checkpoint 0.** 4,096 tokens, 256 per vector, 16 cells, **4,096 tokens in
  tied cells / 12,301 DEV rows** (`alias-census-REPORT.md:72`).
- **Alias ceiling 1,986 vs bar 1,941.** Confirmed — but it is *above* the bar by only **45 rows**,
  not below it (`:36`). At Q1/Q4 the ceiling is **8,908 / 8,948** (`:37-39`). What actually binds at
  Q4 is the **admitted-cell ceiling: 1,920 < 1,941** (`:48-51`).
- **143 of 674 DEV targets predictable at Q4.** Confirmed (131 at Q1, **4** at checkpoint 0)
  (`:55-57`). Q4 has 264 distinct vectors and 189 occupied cells, after **391 accepted vector moves
  out of 16,384 attempts (2.4%)**, at **0.63 new vectors per accepted move** (`:20-24,76`). On a
  uniform-target panel the Q4 ceiling would be **≈921 rows** (`:58-60`).

**In plain terms.** The addressing scheme is a cabinet with **16 drawers**, each holding **256
words jammed in ID order**. The read can choose a drawer but cannot choose a word inside it; ties
resolve to the smallest token ID (`:179-183`). So 4,096 words collapse to 16 rankable identities.
Attention is not refusing to select; there is nothing distinct to select.

**Why it gates everything.** Admission is upstream of ranking, and ranking is upstream of any read
mechanism's effect. Top-1 can only be a token in an admitted cell; only the lowest-ID member of an
alias group can ever win a tie; and the read's output is itself one of 16 key classes at init
(`m1-attention-injective-init-DRAFT-2026-10-18.md:26-28`). Training then makes it worse: the K=8
artifact *started* at **5,680** admitted rows and eroded to **1,920** as occupancy diversified
(`m1-discrete-rung0-admission-redirect-2026-10-15.md:66-68`). Every attention comparison run on that
codebook measures **codebook drift** as much as the read — which is exactly why Dose 1's two arms
both collapsed (2,034→225/205).

## 3. What tonight's K16 run decides — and what it does not

**Correction the record requires.** The run is **not** "A = Relation attention vs R = reference."
In the sheet, **K16-A = the nearest-rival address margin `8·T + 32·A`** (control) and **K16-R = the
occupied-cell rank hinge `8·T + 32·R`** (candidate); both arms use **`ReadSpec::Route`**, and the
sheet says explicitly **"no `ReadSpec::Relation` arm"**
(`m1-discrete-rung0-admission-redirect-2026-10-15.md:12,19-20,83`; `NOW.md:19-20`). A and R are
writer **admission objectives**, not reads.

**It decides:** whether the rank-hinge objective at K=16 preserves target admission **≥ 4,894** on
both seeds, without occupancy collapse, and beats the matched A control by more than the two-seed
noise band; secondarily, whether either arm also clears top-1 **1,941** and top-8 **4,775**
(`:103-123`).

**It does not decide:** any read mechanism; any Relation/Flock arm; whether a learned read can
learn; the copy-successor capability; or any language capability. An admission pass is explicitly
**never KEEP** without both ranking bars (`:113`). At best it clears the road. The branch that would
actually hand work to the read is `RANK_BOTTLENECK_ISOLATED` — admission reached, ranking still
failing (`:209`).

## 4. If tonight is negative

| Outcome branch | Next configuration | What a fair test needs |
|---|---|---|
| `ADMISSION_STILL_CAPPED_AT_K16` | **Full-vector / interior-block / cell-envelope address metric** (`:211`; `project-track.md:59-63`) | Same K16 A/R contrast, one variable; ≥2 seeds, 14,287 frozen rows, fresh transfer |
| `OCCUPANCY_COLLAPSE_TRIGGERED` | **Occupancy-regularized / load-balanced cell-rank objective** (`:210`) | Report occupancy vs control at every checkpoint; `G-Occ` non-vacuity |
| `RANK_BOTTLENECK_ISOLATED` | **W2 read-code capacity capture** (already FIT'd, queued, same lease; `NOW.md:31`), then the Stage M2 Relation-vs-Route discriminator | Matched interleaved schedule, zero-trit ablation, `P_succ` bar with noise floor |
| `K16_A_KEEP_R_NULL` | **Relation + A at K16** (width-only successor; `m1-learned-attention-link-R-DRAFT-2026-10-18.md:9-12`) | One variable: read enablement; 2 seeds or ≥200 fresh rows; fresh transfer before KEEP |

**The preferred next attention test is the injective-init sheet**
(`m1-attention-injective-init-DRAFT-2026-10-18.md`): exactly one variable changes from Dose 1 — the
codebook, to PrimeZeta **4,096 vectors, 0 tie mass, ceiling 14,287**. It carries pre-run gates
G-Init (pinned table, corrupted-copy status 6), G-Occ (>16 cells, tie mass 0), G-Cap (raw and **net
≥ 500 P_succ rows**), and `B3_init ≥ 500` (`:58-77`). Cost ≤ **$11.4** full dose or ~**$2.8** at a
Q1 stop (`:122-128`). It needs FIT (currently PENDING), PM dispatch and owner release.

**Honest risk:** with 256 occupied cells and K=8, `B3_init` may fall below its own 500-row gate
(`:77`). If so, the joint configuration — injective codebook + K=16 + R — is the natural successor,
not a mechanism negative.

**Other untested candidates in the mechanism pages:** starling C1 (float-trained coupling over
fixed Flock topology; needs zero-coupling and random-coupling controls, ≥2 seeds/200 rows, export
gap; `starling-read.md:84-92`), C2 (rank-consistent soft-to-hard selection; source-in-flock and
rank-1-when-kept by distance bucket; `:90-91`), C3 (FBS tree, only after C1 learns), exact addressed
memory read/store (needs `DialogueSettings` fields; bar +8/40 over 19/40, BPB within 0.01;
`exact-addressed-memory.md:37-41`), the pointer forced-copy oracle (`pointer-identity-key.md:20-22`),
and PrimeZeta-vs-null attribution as a separate sheet.

## 5. Why six months without demonstrated attention

The evidence shows four causes, none of them mysterious. First, **the measuring instrument was the
bottleneck**: the M1 headline reply panel is 232 rows and is now formally BLOCKED under R14, and
D39 replaced it with an admission number rather than a language number (`DECISIONS.md:1795`); the
memory panel is 40 rows with ±3 noise. D21 exists because 48 merged PRs in eight hours moved
neither headline (`:1055-1059`). Second, the early attention work was **not merely negative but
uninformative**: D22's audit found a constructor closed on solver numerics, context learning judged
on 24 rows, a pointer keyed on the wrong context (hit 0.003), and read binding called a negative
because a 40-row panel moved one row (`:1098-1105`) — time spent producing closures that were later
reopened. Third, **the rules removed the standard toolbox, repeatedly, and each removal invalidated
a body of evidence**: no softmax (D23, Oct 12), no CPU compute (D24), no matmul (D25), no float at
all (D32, Oct 13-14), float restored offline only (D37, Oct 18). Starling's rungs, the M4 flock
result, the powered read-binding KEEP and every pointer run are now inactive evidence
(`mechanisms/README.md:33-43`), and each pivot restarted the artifact lineage. Fourth, **the
rebuild and its qualification cost dominate the record**: an integer-only CUDA substrate with
bivector moves, BFS transport, ternary gates and one writer, then 11→17→25 construction controls,
PTX zero-float gates and sealed custody per run (`mechanisms/README.md:14`; October's
`current-state.md` is almost entirely instrumentation). And the schedule **starved the read by
construction**: the canonical pass gives it **20 proposals in 16,404 updates (0.12%)**, and
Relation needed a new interleaver even to be reachable (`finite-relational-head.md:156-159`). That
is not an excuse for the sequencing choice — attention was tested *after* admission, and admission
is still unsolved — but it is what the record shows: the project spent its time building an
instrument that could judge the question, and then discovered the codebook could not pose it.

## Where the documents contradict the measurements

1. **The K16 "attention" framing.** Any brief of tonight as A=Relation vs R=reference is
   contradicted by its own sheet: both arms are `Route`, A/R are writer objectives, no Relation arm
   (`m1-discrete-rung0-admission-redirect-2026-10-15.md:83`).
2. **`NOW.md`/D39 item 4 call W1 "~1.2 h"**, while `project-track.md:53-56` states that phrase was
   "a rough projection, not a measurement" and the measured basis is 2.37 GPU-h proposal-only,
   5–8 h proposed. The two live documents disagree.
3. **`finite-relational-head.md:183`** still contains the bold sentence "Zero code for
   `ReadSpec::Relation` exists … and zero GPU or CPU runs," immediately followed by a 2026-10-17
   correction saying the opposite. The correction is right; the stale sentence should be struck.
4. **The census's own limits matter for the bottleneck claim**: Q4 is a *different* run (Route/K8,
   source `e1836149…`, 407 accepted updates); no `rows_v1`/K16 Q4 checkpoint exists, the 1,920
   admitted figure is quoted not recomputed, and the ceiling is panel-specific
   (`alias-census-REPORT.md:226-251`). The binding-constraint conclusion is sound, but it describes
   the closest full-dose artifact, not tonight's arm.
