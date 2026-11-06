# Pre-registration: reference-free complete-branch closure

Run ID: `cpu-training-pilot-reference-free-closure-2026-11-06`

## Status

Complete as **INSUFFICIENT_POWER**. FIT/GO were recorded at `4166a6f1b` before
seed 2788 opened; artifact seed 1 then hit the fixed power floor and stopped the
run before seed 2 and repeats.

## Question

Can the earliest sufficiently supported closure among complete generated
branches choose content and termination without a reference endpoint label?

## Fixed construction

Use the unchanged deepseek-198 full-sequence content spine and deepseek-231
branch candidates. For a candidate `i`, define:

- `advantage = endpoint_global_advantage`;
- `tail_support = tail_local_decisions + tail_global_decisions`.

A candidate is **closed** when `advantage == 0` and `tail_support >= 4`.
Choose the shortest closed candidate; ties use deeper boundary, more terminal
observations, then candidate order. If no candidate is closed, choose minimum
advantage, then maximum tail support, deeper boundary, more terminal
observations, shorter length, then candidate order.

The matched control cyclically shifts the `(advantage, tail_support)` tuples by
one nonzero deterministic offset within each candidate list. Candidate lengths,
boundary fields and the tuple multiset stay fixed.

Arms: closure, shuffled-closure-features, first-boundary, longest-spine,
boundary-terminal-volume, reference-exact endpoint oracle, full retrieval.
The learned and shuffled-label readings from the existing driver are reported
as secondary controls but are not the primary arm.

## Data split

Suffix and boundary memory come only from TRAIN `tt-2711`:

- tokens:
  `140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91`;
- mask:
  `d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad`;
- manifest:
  `d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5`.

Fresh evaluation: `cell=dev,dev`, seed 2788, protocol 2, 128 conversations,
both public artifacts. No reference content enters candidate generation or
selection.

## Base artifact

- seed 1:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- tokenizer:
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`.

## Instrument gate

Pins match; suffix tables contain no BOS targets; the feature shuffle changes
at least one selected candidate; every arm shares the panel and artifact; the
fresh panel and output root are absent at registration; all reachable rows emit
or the run is rejected; repeats match after timing removal.

## Power

Per artifact: at least 200 reachable rows, 100 closure selections different
from first-boundary, and 20 rows with an exact branch candidate.

## Bar

On both artifacts, closure must:

- emit every reachable reply with zero premature BOS;
- retain first-boundary exact and body counts;
- beat shuffled closure and first-boundary exact replies by the paired
  two-sigma rule;
- change at least one complete reply from both controls;
- repeat byte-identically after timing removal.

Full retrieval and the reference-exact oracle are reported controls.

## Outcomes

KEEP if every bar passes; REJECT if active and a quality or termination bar
fails; INSUFFICIENT_POWER below a power floor; CONTROL_FAULT on a vacuous
feature shuffle, input drift, nondeterminism or unexpected reference use.

## Stop rule

Do not retune the closure definition, tail window, support threshold, fallback
order or shuffle after seed 2788 opens. Stop on any prohibited path or failed
instrument gate.

## Cost

Local CPU only; 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

## Steps

1. Implement and unit-test closure and matched feature shuffle.
2. Run scoped/full CPU checks and obtain TEST FITNESS.
3. Generate fresh seed 2788 only after GO.
4. Run both artifacts and fresh-process repeats.
5. Seal reports and evidence and deliver through exact-head review.

The reviewed fresh run form is:

```sh
cpu-joint-branch-driver \
  TRAIN_TOKENS TRAIN_MASK TRAIN_MANIFEST \
  PANEL_TOKENS PANEL_MASK PANEL_MANIFEST ARTIFACT \
  --primary reference-free-closure --seed 2788 \
  --expected-train-tokens TRAIN_TOKENS_SHA256 \
  --expected-train-mask TRAIN_MASK_SHA256 \
  --expected-train-manifest TRAIN_MANIFEST_SHA256 \
  --expected-panel-tokens PANEL_TOKENS_SHA256 \
  --expected-panel-mask PANEL_MASK_SHA256 \
  --expected-panel-manifest PANEL_MANIFEST_SHA256 \
  --expected-artifact ARTIFACT_SHA256
```

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 reference-free closure sheet.
reviewer: deepseek_235_fit2 fresh-context subagent
note: The closure rule is deterministic and label-free; the matched control
  preserves every endpoint-advantage/tail-support tuple and candidate length
  while changing associations. The driver emits a primary-specific claim and
  requires `--primary reference-free-closure --seed 2788`. Focused tests pass
  9, the driver test passes 1, the CPU library passes 417 with four expected
  ignores, scoped Clippy and Rustfmt pass, and seed 2788/output are absent.
source: `a2d93b3999e548f804115c076675c773c5deabdc8eb79debc14bd3f21b815597`;
driver: `92c1a0183c93e1a6e707006712a086e6a0f177ed45b204553071786f35dff084`;
release_binary: `e7f958e8f609515f565c62c9e71f6a2191e4d651c52334b8a18a058d9b7bb02b`;
date: 2026-11-06

## Pre-run GO

GO at `4166a6f1b6d48da1f48e7405ebb79b3a2fed335b`. Source, driver and
release binary hashes match the FIT record. Seed 2788 may now be generated
under the fixed construction and stop rule.

## Outcome

**INSUFFICIENT_POWER on artifact seed 1.** Closure selected **195** candidates
and changed **21** complete branches from first-boundary, below the required
100; only **18** rows had an exact candidate, below the required 20. Seed 2 and
repeats were not run. The seed-1 readings remain reported but are not promoted
to a mechanism negative.
