# REPORT: cpu-training-pilot, deepseek-142

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `ae8fce0b7`.

## Status

Attempt 1 is complete at the registered preflight stop. No held-out public
generation ran and no capability claim is made.

## What did not work

- **The gate learned too few endpoints.** Seed 1 recalled **19/84** generated
  terminal rows against the registered **26/84** floor while terminating
  **9/1,484** nonterminal rows inside the 14-row one-percent limit.
- **The label control performed the same.** The class-preserving shuffled
  density also recalled **19/84**, with 11/1,484 false stops and a lower
  fitted margin. The source arm showed no terminal-recall advantage over this
  matched control.
- **The registered run stopped before held-out generation.** The preflight is
  a hard stop rule, so seed 2, all public generation arms and the one-worker
  repeat did not run. There is no exact-reply, BOS-completion, top-1 or depth
  result for this representation.
- **This is not a language or serving result.** The integer artifact,
  `UORLUT01` format, CPU/serving content readout and Claude-owned paths were
  unchanged.

## What worked

- The release driver built and the source remained byte-identical to
  implementation pin `c8180c1576970069dcff395889460defad2a2d6c`.
- All five public inputs matched their registered SHA-256 values.
- The full `uor-r4-cpu-train` release library tests passed at **249 passed, 4
  expected ignores**; all **6** driver tests passed; scoped Clippy and
  targeted rustfmt passed.
- The gate retained all **1,568** generated training rows and recorded complete
  terminal/nonterminal memberships, offsets, full 2,048-bit signatures,
  deterministic leave-one-out choices and true/shuffled gate hashes.
- The true and shuffled gate hashes differed
  (`860715b8…d89140` and `f5d86970…bb8242`), and the selected k was 8 in both
  arms.

## What changed

The CPU content driver can now fit an offset-conditioned, class-balanced
k-nearest density gate over every generated training row without replacing
any earlier arm. It records the generated offset and complete 2,048-bit
signature for each decision, selects same-class neighbours within offset
radius 2, penalizes missing neighbours at maximum signature distance, and
chooses k from `{1,2,4,8}` and a margin by deterministic leave-one-out fit.
A score at or above the fitted margin emits BOS before the unchanged content
choice.

The driver also records the full decision digest and generated IDs so a
one-worker repeat can compare complete generated evidence. The registered
class-preserving shuffled control rotates terminal labels without changing
class counts or offsets. The implementation remains isolated to
`crates/uor-r4-cpu-train` and does not alter serving.

## Measurements

| Public seed | Terminal rows | Nonterminal rows | True recall | True false stops | Shuffled recall | Shuffled false stops | True / shuffled margin |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 84 | 1,484 | **19/84** | 9/1,484 | **19/84** | 11/1,484 | 916 / 844 |
| 2 | not run | not run | not run | not run | not run | not run | not run |

The preflight wall was **0.62 s** at **236,356 KiB** peak RSS. The release
build took **19.33 s**, occupied **267,727,456 bytes**, and peaked at
**1,196,084 KiB**. The three focused controls took **40.13 s** in total and
peaked at **1,199,660 KiB**. Measured attempt compute was **60.08 s** against
an admitted increment of 3,300 s and the prior cumulative
**52,027.677487699 s**, giving an admitted cumulative projection of
**55,327.677487699 s**. The named model-time ledger remains `ABSENT`; its
cumulative total is not treated as zero.

## Classification

This is a **bounded mechanism negative** for the full-row,
offset-conditioned, class-balanced K<=8 Hamming density representation at the
registered one-percent nonterminal false-stop limit. The controls and runtime
worked, and the preflight correctly prevented a misleading held-out
generation run. This result does not test whether the unrun generation arms
would complete replies, and it is not evidence against every geometric
termination representation.

The next termination work should change the objective or representation:
learn continuation relative to a generated-state trajectory, compare terminal
and near-terminal nonterminal density, or use a state representation that
separates approaching an endpoint from ordinary continuous text. Retuning this
same K<=8 density configuration after the observed result would not answer the
registered question.

The exact stopped output, input hashes and recursive manifest are under
[`evidence-attempt-1/`](../../../../labs/cpu-training-pilot-stop-density-2026-11-04/evidence-attempt-1/).
The evidence `SHA256SUMS` file is
`28c30f1a0c0b0937052f488210c62b937ee53fa2ae1967ad36c621cb6f382725`.
Measured and admitted accounting is in the
[resource ledger](../../../../labs/cpu-training-pilot-stop-density-2026-11-04/resource-ledger.json).

## Checks run at the head

- Fresh FIT and GO preceded the registered build and preflight.
- Release build, **249** library tests, **6** binary tests, scoped Clippy and
  targeted rustfmt passed.
- Source identity and all five public inputs matched.
- The true/shuffled gates had equal class row counts and different hashes.
- Recursive evidence and input checksums pass.
- Claim wording, docs catalog, roadmap schema/rendering, prep checks,
  `uor-harness check` and `git diff --check` are recorded at delivery.
- No integer-session, serving, CUDA, float-training or Claude-owned source
  changed.

## Skills used

No system-provided Codex skill applies. Prior FIT, GO and delivery reviews
used fresh contexts independent of the implementation run.

## Still open

- A termination objective or state representation that transfers from
  generated states and beats a matched control.
- Complete replies, fresh phrasing and useful local chat.

## Plain summary

The all-row density gate found only 19 of 84 training endpoints, exactly the
same count as its shuffled label control, and therefore stopped at the
registered preflight. The build, controls and evidence path worked; this
particular density representation did not learn enough endpoints to justify
held-out generation.
