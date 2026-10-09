# Earliest-query intervention on the retained native parent

## Question and fixed comparison

Following #2091 and #2093, does an answer-relevant distinction at query token zero
survive admission and the consumed native scoring paths? This is a causal
input comparison, not a fit or a new history-reader implementation. Retain exact
Source/Context segments, empty actual prefix, parameters and the complete action
pool. The accepted Source48/Generate64 parent has no continuation U field.

Prospective M2 claim: https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6089708488.
Source implementation: `ab247227a3a10672144c25651d4f61b67c9d52d0`.
Parent: `native-prediction-control/recomposition-503d64b39-attempt1`, checkpoint0000,
report SHA256 `a1277d2be4ff962100f857d0da553257ad5596ac8bc72d9257299e74d2f2e278`,
manifest `b37e2ca588bf1cc4dc5971fa5da97b9e83c90a94f4ea0bfdb0655b8f92bf94ca`.
The unchanged public loader verifies its full seal and coherent native sidecars.

The prospective input-only preparation selects development rows 8 (job) and 9
(home) from the existing 512-row panel, retaining both original questions as
controls. Their Source/Context banks are identical. New requests are
` job: give my current stored value.` and ` home: give my current stored value.`.
The initial spaces are intentional: the artifact's vocabulary contains whole
space-prefixed `job` and `home` pieces. The actual Rust tokenizer verified exactly one changed token at index zero
(1541 versus 582) and an identical 11-token suffix before evaluation. Both target
first tokens are distinct: Your=2997, You=617.

This wording is novel diagnostic phrasing. Cloned frozen answer memberships come
from the corresponding existing current-job/current-home intent. Preparation
requires each membership set to have an unambiguous first token and the two
sets to have different first tokens. Thus this particular experiment has a
well-defined different correct next token; that is not true of every pair of
role questions. Four own-feedback generations were evaluated separately from the
fixed empty-prefix comparison.

## Implementation and admission

`native-bank-generalization prepare-early-query CONFIG.json` verifies sealed
input/label files, tokenizer identity, selected row IDs, complete segment equality,
empty prefixes, text/token round trips, a sole-token-zero intervention and
first-target separation. It claims an exclusive output only after path/ancestry
validation and seals both success and failure. Inputs and labels remain separate;
no model or predictions are accessed by preparation. Arbitrary request strings
cannot have their semantics proved by the generic preparer: original decoded
questions, frozen memberships and new requests are reviewed before evaluation.

The existing `own_prefix` evaluator gains only `retain_entry_bank_trace`, false
by default. When enabled it serializes the already-computed first bank trace:
chronological states, Cue features, per-Source scores and Prefix contributions.
Existing output already retains every Generate/physical Copy score and pooled
mass, donor/bridge state, actual output IDs, decoded response and EOS. No second
encoder replay, changed routing, added candidate or label reaches the model.

## Prospective decisions and limits

- Admission drops the cue: repair the caller.
- The distinction disappears before consumed comparisons: investigate explicit
  read-only occurrence access against the unchanged parent.
- The distinction reaches scores but the wrong token wins: repair the observed
  scoring/learning boundary, retaining protected-margin integration as a candidate.
- Correct distinct complete answers with EOS: scoped role discrimination on
  this pair, not a replacement for held-out or full512 evaluation.

Changing token zero changes its downstream paths together. Witness differences
localize influence; they do not independently attribute the effect to recurrence,
Cue, Prefix or bridge. Same output alone does not prove ignored input, and a
changed pool alone does not prove correct role use. No result from this novel
query pair can establish global capacity failure or explain all 512 failures.

## Execution

**KEEP** the bounded early-cue influence witness; learning/scoring integration is
next. New access machinery and a new geometric substrate remain **NOT YET
PROMOTED**. This pair does not support caller loss or total erasure of the early
cue before scoring. It does not prove which component should change, nor that
read-only occurrence access could never help.

| Query | Correct first token | Native chosen ID | Complete with EOS |
|---|---:|---:|---:|
| Original job | 2997 | 2997 | yes |
| Original home | 617 | 617 | yes |
| Token-zero job intervention | 2997 | 1888 | no |
| Token-zero home intervention | 617 | 340 | no |

The two original controls reproduce their exact accepted answers. The novel pair
has 0/2 correct entry tokens and 0/2 complete answers, each reaching the unchanged
32-token generation bound without EOS. These four rows are not a new full512
measurement; the accepted 8/512 development result remains unchanged.

The novel pair's chronological input has 51 tokens; the sole changed absolute
position is 39 (query token zero). Earlier states match. The difference persists
through all 12 query positions: all 8 final recurrent observations, all 8 final
latent states and all 8 post-bridge observations differ. Query-Cue latent states
differ in 8 lanes, observed codes in 7. Cue Copy contributions differ at 0/11
positions in head zero and 6/11 in head one; the combined bank scores differ at
11/11 positions in both heads. All 11 Source identities, the empty Prefix trace
and the chosen bridge donor remain identical. Generate raw scores differ for
3981/4096 tokens; final Copy raw scores differ for 11/11 positions. The resulting
pooled decisions differ, and both are wrong.

For the two accepted first-token alternatives, normalized native pooled mass is:

| Novel query | P(Your), ID 2997 | P(You), ID 617 |
|---|---:|---:|
| job | 0.00004545955 | 0.00058952841 |
| home | 0.00063407043 | 0.00026431988 |

These alternatives reverse their desired relative preference, but neither is the
actual winner. This is numerical sensitivity without successful role-conditioned
prediction. It is not a measurement of preserved semantic information, an isolated
Cue/bridge effect, a training-gradient diagnosis or a proof of representational
sufficiency. `summarize.py` independently recomputes saved Generate/Copy alias
sums, total mass and deterministic winners for all 85 saved steps; it performs
no new forward passes. Full exact fractions and changed indices are preserved.

## Exact execution and validation

Execution root: `/workspace/uor-r4/codex/sol-early-query-20261009`.
Successful report: `evaluation-attempt2/report.json`; input-only preparation:
`preparation-attempt1`. Runtime SHA256
`77407b3c28758e893d1f53a603c60ab61e64ba6815bcf7b209c036f67f890c94`.
Source and actual run are bound to `ab247227a3a10672144c25651d4f61b67c9d52d0`.
Input SHA256 `04b48aa3a32b39398738e5219b59cc8ff90c57dd766b07b7cf8f794775a8e8b0`;
label SHA256 `ec58bf3a84935b407cc5b5e7fb1eba4d5381be602c7df9bf5b6295f92ffa6d60`.
Submitted retry config SHA256
`705b832c0157400e16cb3cde4961a0d1886b75d6d3fcd4abc6f711d2150d84bb`;
sealed config hash differs because the evaluator reserializes the JSON value;
the submitted and sealed objects are identical.
The tokenizer SHA256 is
`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.

The scoped release build passed (242.26 seconds); all 19 example tests passed,
including preparation invariants and opt-in trace admission (3.05 seconds command
wall time). The initial offline build lacked cached `bit-set v0.8.0`; retrying
locked dependency resolution repaired setup. Evaluation attempt1 hit its 128 MiB
report boundary after saving three rows and was sealed FAILED. It is not a
selected three-row verdict. Attempt2 changed only output/config paths and the
report allowance to 512 MiB; all four rows completed in 4.00 seconds with
998,834,176 bytes child peak RSS. Failed-attempt model execution was 4.59 seconds;
complete charged model execution is 8.59 seconds, with no training or CUDA.
The first three row hashes match across attempts in the private receipt.
The public loader and output seals passed; full-pool saved arithmetic passed.
The source-only prospective expert reviews passed with semantic admission checks;
final review and exact-head receipts accompany the PR.

The pod was admitted before #2095 updated the CPU-only compute rule. On refresh,
that update was read; no additional model compute is needed. Future CPU-only
work follows current admission rules. This run used one RTX5090 pod at $1.19/h,
but the native integer implementation ran on its CPU; no GPU speed claim is made.
Tool-managed bootstrap took 200 seconds, including its 137-second build and
47-second parity checks (37 pass, 3 ignored), which are setup evidence only.

Projection: 90 minutes complete preparation/recovery/build/evaluation/review/
delivery/preservation/cleanup, one short-lived approved GPU pod, two scoped build
workers, at most 16 GiB pod RAM, 10 GiB new pod storage and 256 MiB local evidence
plus checkout. Native integer serving executes on the pod and has no CUDA
backend; no laptop model training or grading. Tool-managed bootstrap is accounted
separately from scoped build/model time. Preserve the cumulative ledger and
30 GiB local floor plus 128 MiB margin. Main holds source; iCloud holds results.

## Delivery and preservation

STATUS, ROADMAP and the programme tracker headline remain unchanged: this is a
scoped diagnostic, not a newly qualified model or milestone. README capability
claims and product commands remain unchanged. The dated prior records retain
their prospective decisions with a follow-through link; current geometry text
and its figure reflect the measured influence. Geometry verification remains
29/29 PASS, and claim wording passes.

The first derived summary lacked explicit cross-file identity joins. Adversarial
review caught that gap; version2 binds report/preparation input and label hashes,
all five row IDs, first-token verdicts and the actual query suffix. It reproduces
the same numerical findings without a new model call. Both derived versions are
retained. Raw results, runtime, exact configs, failed attempts and receipts are
preserved in `icloud:UOR-R4/results/codex/codex-early-query-causal-20261009.tar`;
the store index binds archive bytes and MD5. Delivery is pending at this source
head; the PR will carry final checks, resource charge, preservation and cleanup
receipts before completion is reported.

**Next:** Integrate protected winner/rival margins and authenticated Prefix credit
through the existing coupled graph, with finite direction policy frozen before
new gradient work. Keep the full native action pool and independent acceptance
conditions; this query pair is exposed diagnostic data.
