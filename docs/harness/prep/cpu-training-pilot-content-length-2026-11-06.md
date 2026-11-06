# Pre-registration: full-sequence content with stored-length completion

Run ID: `cpu-training-pilot-content-length-2026-11-06`

## Status

Complete as **REJECT**. TEST FITNESS: FIT and pre-run GO were recorded before
seed 2783 opened. Both artifacts and repeats ran under the fixed construction.

## Question

Can the content-positive full-sequence suffix selector from deepseek-198 beat
matched length-completion controls when the nearest TRAIN reply's stored length
replaces its premature boundary stop?

## Fixed construction

Retrieve the raw-unigram-Jaccard top-1 TRAIN reply with lower-ordinal ties.
Use its stored reply length as the completion budget, capped at 64, without
reading the evaluation reference.

At each token, keep the deepseek-198 content policy: local `Longest32To1`,
full prompt-plus-reply suffix memory, two observations, strict-majority
successor, global only when deeper than local, and local ties. Stop by BOS,
stored length or context.

Arms:

- full-sequence content with stored length;
- full-sequence content with the original boundary memory;
- suffix-shuffled content with stored length;
- shallow-local-global depth 4 with stored length;
- local longest copy with stored length;
- writer with stored length;
- full stored retrieval;
- reference-length oracle with full-sequence content.

## Data split

TRAIN `tt-2711`:

- tokens:
  `140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91`;
- mask:
  `d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad`;
- manifest:
  `d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5`.

Fresh evaluation panel `cell=dev,dev`, seed 2783, protocol 2, 128
conversations, generated only after FIT and GO.

## Instrument gate

Source, artifact, tokenizer, sidecar, TRAIN and fresh panel pins match; the
panel and output roots are absent at registration. The suffix table has zero
BOS transitions. The shuffle preserves support, observations and successor
multisets while changing assignments. Every length is positive and at most 64.
Every arm emits all reachable rows. Repeated processes match after timing
removal.

## Bar

On both artifacts, the full-sequence stored-length arm must:

- beat local-longest length completion on exact replies by two sigma;
- beat shuffled-suffix length completion on exact replies by two sigma;
- retain at least the better body count of the boundary and local-length arms;
- emit all reachable replies with zero premature BOS;
- change at least one token from local and shuffled controls;
- repeat byte-identically.

Full retrieval and the reference-length oracle are reported controls.

## Outcomes

**KEEP** if every bar passes; **REJECT** if active and a quality bar fails;
**INSUFFICIENT_POWER** below 200 reachable rows or 100 deeper-global decisions;
**CONTROL_FAULT** on drift, vacuous shuffle, invalid length or nondeterminism.

## Stop rule

Do not retune the suffix confidence rule, length source or comparison after
seed 2783 opens. Stop on any prohibited path or failed instrument gate.

## Cost

Local CPU only; 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

## Base and inputs

- base: `bbb8c5332`;
- tokenizer:
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- artifact seed 1:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- artifact seed 2:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- sidecar seed 1:
  `2a298b44ea34d1a8e2d40191951d3da9b3300e2ac7de65e6c2ec13950a0b6424`;
- sidecar seed 2:
  `6504dcb047ce7157dd5e968c29441505cf48d993f01ec63222125634c61963b7`.

## Output

`local/deepseek/content-length/2026-11-06`; fresh panel and output root absent
at registration.

## Steps

Extend the suffix-arbitration evaluator with an exact stored-length endpoint;
build the dedicated driver; run TRAIN preflight; generate seed 2783; evaluate
both artifacts and repeats; seal source, panel, reports and artifacts.

## Minimum test power

- two artifacts;
- at least 200 reachable replies;
- at least 100 deeper-global decisions per artifact;
- boundary, local-length, shuffled-length, writer, full and oracle controls;
- paired exact/body comparisons and deterministic replay.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 full-sequence content with stored-length completion
sheet: docs/harness/prep/cpu-training-pilot-content-length-2026-11-06.md
reviewer: deepseek exact-head self-review (independent delegation unavailable in this run)
note: Implementation 74793da76 adds an exact length endpoint while retaining
  the existing boundary mode and content arbitration unchanged. The driver
  derives reference-independent lengths from the raw-Jaccard selected TRAIN
  reply, evaluates full and shuffled suffix content, shallow global, local,
  writer, full-retrieval and reference-length controls, and checks arm
  reachability before grading. Two focused tests prove length completion is
  accepted as exact and zero/over-cap budgets are refused. The full CPU library
  passes 404 tests with four expected ignores; scoped Clippy passes with the
  pre-existing type-complexity lint exempted; formatting and diff checks pass.
  Inputs and output-root absence were verified.
source:
  `b404b33856e21f4599d30434acd9ef71ba00ac339cc93aded3676aed2d1fad49`;
driver:
  `4a4da3804e32b6faec78a24357146ed6e61b43a804fc576d688e8397512273a5`;
release_binary:
  `609343405c1f8ae38f699cb971755ff51fe6144be26743858478bcf423725977`;
date: 2026-11-06

## Pre-run GO

GO - exact-head self-review under the current disabled-delegation policy.
Implementation `74793da76`, source
`b404b33856e21f4599d30434acd9ef71ba00ac339cc93aded3676aed2d1fad49`,
driver
`4a4da3804e32b6faec78a24357146ed6e61b43a804fc576d688e8397512273a5`
and release binary
`609343405c1f8ae38f699cb971755ff51fe6144be26743858478bcf423725977`
match. This was not an independent review. Seed 2783 may now be generated under
the fixed construction and stop rule.

## Outcome

**REJECT.** Full-sequence content with stored-length completion made **6**
exact replies and **560** correct body positions on both artifacts. It beat
local and shuffled length controls and emitted all 457 reachable replies with
zero premature BOS, but failed the registered retention comparison against the
boundary control at **14** exact and **760** body positions. The seed-1 and
seed-2 repeats matched byte-for-byte after timing removal.

Full retrieval remained stronger at **42** exact and **879** body positions.
The reference-length oracle reached **16** exact and **599** body positions.
This closes the fixed stored-length substitution for the deepseek-198 content
selector, not full-sequence content or geometric termination generally.
