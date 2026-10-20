# Saved discrete-feedback formation attribution — 9 October 2026

## Question and fixed scope

Why did #2117's fixed residual-feedback constructor form 28 distinct legal updates
without one protected descending displacement? This analyzes its saved arithmetic
to distinguish repeated destinations, persistent constraint conflicts and
cancellation between Prefix and Generate contributions. It does not run a model,
propose a new update, change a gate, or repeat the 411 producer backwards.

The [prospective M2 claim](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6092019941)
fixes all 32 rounds and their 28 unique destinations, with rounds 0 and 31 as
representatives. It retains all guard and coordinate records and reports adverse
and helpful contributions separately. Saved primal/feedback norms do not expose
the internal projected vectors; no unrecorded state or convergence claim is inferred.

**KEEP** the saved attribution. The fixed feedback candidate remains **REJECTED**;
a replacement legal-set formation method is **NOT YET PROMOTED**. No model was run.
Accepted complete-answer performance remains 8/512 on the frozen 512-episode
development panel, and the separate conditional 9/15 candidate is unchanged.

## Exact source and restoration

Source experiment: [#2117](https://github.com/UOR-Foundation/uor-r4/pull/2117),
executed Rust source `7381d670735acddb0fac1f7abc0d83498aed04fc`.

- Report SHA256: `3bbcc79268cd5f13ccfb63fe244bf99ff97d43820ef7dbe5ed4c5544d3440cc6`.
- Manifest SHA256: `f616174ce7c4b244b331ea48b9d61340201662e75faef086c2d6835a3ee9ef74`.
- Retained archive: `icloud:UOR-R4/results/codex/codex-protected-discrete-feedback-20261009.tar`.
- Archive bytes: 624,674,304; MD5: `fcc6b2a621e7668e5bf8dc08ad490b08`.

The analyzer verifies the complete source file set and seals, actual original
master bits, objective gradient, 380 raw Jacobians and all saved destination
bits/deltas, residuals, tolerances and duplicate identities before attribution.
The source used the original development input245 episode with 15 conditional
positions including EOS, 17 reference roles and 380 protected original winners;
these overlap in the 391-row union. No new data split is used.

## Measurements and interpretation boundaries

Per-round records separate objective descent, protected-screen failure, exact
noops and duplicate destinations. Guard frequencies, intersections and transition
sets use both all rounds and unique destinations. Signed family contributions
retain opposing terms; top10/top100 coordinate shares use adverse pressure on
the explicitly named violated set, not a fraction of native loss. Original
fixed-pair log margin plus raw-Jacobian displacement is a local surrogate only.
It does not rerank tokens, constrain other rivals, or establish native retention.

Repeated saved destinations establish repeated discrete offers, not a repeated
internal feedback state. Distinct terminal offers establish neither convergence
nor divergence. No finite observed set proves global legal-set feasibility or
infeasibility. The original screen and all native gates remain unchanged.

## Measured result

The single saved-arithmetic run authenticated all **1,689** manifest-listed source files plus the pinned manifest,
the ordered f32 objective-gradient sums and all **61,440** destination coordinates
(32 × 1,920). Every saved actual delta, objective dot, unit-Jacobian residual,
relative tolerance and first duplicate identity matched exactly. The original
32 rounds have 28 unique destinations and four repeats; all predict objective
descent, none passes protection, and no native candidate was scored.

| Recorded scope | Result |
| --- | --- |
| Violated guard union, all 32 / unique 28 | 338/380 / 338/380 |
| Violated guard intersection, all 32 / unique 28 | 0 / 0 |
| Most frequent guard across unique 28 | guard 17, 24/28 proposals |
| Changed coordinates per proposal | 7–47 of 1,920 |
| Consecutive violated-set Jaccard | 0.06934–1.0, repeats included |
| Top10 share of adverse coordinate pressure | 55.39–100%, per round's violated unit rows |
| Helpful/adverse coordinate pressure ratio | 2.31–77.49%, same scope |
| Opposing nonzero Prefix/Generate signs on violated row-round events | 1,192/3,507, all 32 including repeats |

The predetermined first and last proposals show what the full records retain:

| Quantity | Round0 | Round31 |
| --- | ---: | ---: |
| Changed Prefix / Generate coordinates | 11 / 0 | 32 / 11 |
| Violated protected rows | 102 | 52 |
| Prefix objective contribution | -0.3899608068 | -0.3662258585 |
| Generate objective contribution | 0 | -0.0068813899 |
| Adverse / helpful unit-coordinate pressure | 1.263689 / 0.058496 | 2.234035 / 1.307733 |
| Opposing nonzero family signs among violated rows | 0/102 | 28/52 |
| Positive original-pair linearized margins among violated rows | 99/102 | 50/52 |

Across all 32 proposals Prefix predicts descent; Generate is either unchanged or
also predicts descent. That objective agreement does not imply agreement on the
380 protected comparisons. In round 31, eight violated rows have both families
adverse, 28 have opposite signs, and 16 have a zero family contribution. Coordinate
cancellation can also occur within either family. The maximum absolute raw/unit
family-sum arithmetic residuals are 4.45e-16 / 1.67e-16; pressure closure differs
by at most 5.33e-15. Top100 shares are one up to ordinary floating rounding because
at most 47 coordinates changed; this is not an independent concentration finding.

**Decision:** the recorded failure is distributed across discrete destinations;
no single comparison vetoes every unique proposal. It also contains recurring
conflicts (guard 17 fails 24/28) and substantial signed compensation. Four repeated
destinations do not explain the 28 distinct failures. This justifies testing a
constructor that enforces the joint legal discrete constraints during formation,
rather than another continuous projection followed by rounding or a radius/rho
sweep. It does not establish which new solver will succeed, a defective family,
actual native winner loss, convergence or global discrete infeasibility.

## Reproduction, verification and retention

[`analyze.py`](analyze.py) SHA256:
`86c219bcb9b61cbe5faa00475ae6fb61c78ab53767e1288c6ac7fd97267e87da`.
The compact per-round receipt is [`result.json`](result.json). Full 31 MB outputs
contain every guard and coordinate, both frequency populations and exact input
hashes. Report SHA256:
`638d6b5e9e6c167fb251c765db8d14ae34dbc460cc127c0273fc51cf5ee1a382`;
manifest SHA256:
`f6b0603ee26e5544f616de86a57b8517d05dbdc681bd05fc3b03d908ee6ac41f`.

Restore the #2117 archive above, then use the retained small report helper (which
includes the unchanged `uor-r4-integer::report_output` source) to claim a fresh
output root. The helper's source, lockfile and executable are preserved with this
analysis. With Python's already-installed `blake3` evidence library:

```sh
python3 docs/labs/discrete-feedback-attribution-2026-10-09/analyze.py --self-test
local/report-tool-target/release/discrete-attribution-report-tool claim local/analysis/attempt1
python3 docs/labs/discrete-feedback-attribution-2026-10-09/analyze.py \
  --run local/restore/codex-protected-discrete-feedback-20261009/discrete-feedback/runs/attempt3 \
  --output local/analysis/attempt1
local/report-tool-target/release/discrete-attribution-report-tool seal local/analysis/attempt1
local/report-tool-target/release/discrete-attribution-report-tool verify local/analysis/attempt1
```

Use a new output path for any retry. Normative source and output seal verification
passed; focused arithmetic/bit/dedup/malformed-input self-tests passed. The helper
build took 12.87s and its two existing exclusivity/seal tests passed. No model Rust
path changed, so no model compilation or generated-behavior run was added.
The [arithmetic review](adversarial-result-review.json) independently recomputes
all 32 violated sets and both representative pressure sums. The
[mathematical review](math-result-review.json) checks interpretation and the next
causal task. These are saved-evidence reviews, not independent model replications. Claim wording and whitespace checks are part of protected delivery.

The complete new analysis, helper, reviews and restoration receipt are retained at
`icloud:UOR-R4/results/codex/codex-discrete-feedback-attribution-20261009.tar`
(31,706,112 bytes; MD5 `3588f378275c1ea35b9e72d6ae1ba008`), verified by
cloud-store upload/download before cleanup. The624MB original input remains in
its separately verified archive; it is not duplicated in this new result archive.
Final delivery metadata and wording corrections are authoritative on main.

## Resources and delivery

One worker completed attribution in 30.366s with 88,047,616 bytes peak RSS and
30,913,987 bytes of sealed outputs. There were zero model, native-score, gradient
or solver-replay calls. Complete restoration/review/delivery/cleanup costs are
charged separately to the shared ledger. The initial complete projection was 25 min,
two CPU threads, 1 GiB analysis RAM, 3 GiB transient storage and 50 MiB new reports;
free disk was 38 GiB after restoration, above the 30 GiB floor plus 128 MiB margin.
No GPU, pod, paid compute, Python model or product dependency was added. The
standing local allowance was extended by 30 min before use, from 1,456,200,028 to
1,458,000,028ms; cumulative charges continue.

Accepted artifacts, milestone state and geometric roles/basis/figures are
unchanged. STATUS, ROADMAP, tracker table, geometry figures and top-level README
therefore retain their existing scoped claims. The previous experiment record
links this follow-through without rewriting its prospective question.

**Next:** Specify a bounded constructor that selects legal coordinate destinations
under the joint protected inequalities and strict objective descent, with an
explicit no-solution-found outcome and unchanged independent native gates. Review
that generic mechanism before implementation; do not reuse a continuous-feedback
parameter sweep or claim feasibility from positive linearized margins.


Follow-through: [direct legal construction](../direct-legal-construction-2026-10-09/README.md) implemented this successor. Absolute and centered representations both encountered a backend singular-matrix error before any proposal; the next question is the failing solver basis, not another feedback sweep. This does not change the saved attribution above.
