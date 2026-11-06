# Pre-registration: aligned retrieved endpoint with boundary fallback

Run ID: `cpu-training-pilot-content-prefix-endpoint-2026-11-06`

## Status

Complete as **REJECT**. TEST FITNESS: FIT and pre-run GO were recorded at
repaired implementation `7afcac83c` before seed 2784 opened.
The fresh seed-2784 panel and output root remain absent.

## Question

Can a reference-independent retrieved endpoint remove premature BOS without
discarding deepseek-198's content trajectory by deferring the boundary only
while the generated reply exactly follows the retrieved TRAIN reply?

## Fixed construction

Retrieve the raw-unigram-Jaccard top-1 TRAIN reply with lower-ordinal ties.
Keep deepseek-198's local `Longest32To1`, full prompt-plus-reply suffix memory,
two observations, strict-majority successor, global-only-when-deeper rule and
local ties.

At each generated step:

- while every generated reply token equals the corresponding prefix of the
  retrieved TRAIN reply and the retrieved endpoint has not been reached,
  suppress the boundary decision;
- if the generated reply exactly reaches the retrieved reply, stop at the
  retrieved endpoint;
- after the first divergence, use the unchanged reply-boundary decision.

The retrieved reply never injects a content token. It only guards the endpoint.

Arms:

- full-sequence content with aligned retrieved endpoint and boundary fallback;
- full-sequence content with original boundary stopping;
- full-sequence content with unconditional stored-length stopping;
- aligned endpoint using shuffled-retrieval reply associations;
- local longest copy with the aligned true-retrieval endpoint;
- full stored retrieval;
- reference-reply alignment oracle;
- writer with boundary fallback.

## Data split

TRAIN `tt-2711`:

- tokens:
  `140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91`;
- mask:
  `d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad`;
- manifest:
  `d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5`.

Fresh evaluation panel `cell=dev,dev`, seed 2784, protocol 2, 128
conversations, generated only after FIT and GO.

## Instrument gate

Source, artifact, tokenizer, sidecar, TRAIN and fresh panel pins match; the
panel and output roots are absent at registration. The suffix table has zero
BOS transitions. The suffix shuffle preserves support, observations and
successor multisets while changing assignments. The retrieval-reply shuffle
changes at least one panel selection. Every retrieved reply is non-empty and
at most 64 tokens. Candidate, boundary and shuffled guard use the same panel
and artifact. Every arm reports all reachable rows. Repeated processes match
after timing removal.

## Bar

On both artifacts, the aligned true-retrieval arm must:

- make zero premature BOS stops and emit every reachable reply;
- retain at least the boundary arm's exact and body-position counts;
- beat the shuffled-retrieval guard on exact replies by two sigma;
- change at least one complete token sequence from boundary and shuffled;
- stop at least 20 replies at the aligned retrieved endpoint;
- repeat byte-identically.

The unconditional length, full retrieval and alignment oracles are reported
controls.

## Outcomes

**KEEP** if every bar passes; **REJECT** if active and a quality or termination
bar fails; **INSUFFICIENT_POWER** below 200 reachable rows, 100 deeper-global
decisions or 20 aligned endpoint completions; **CONTROL_FAULT** on drift,
vacuous retrieval shuffle, invalid retrieved reply or nondeterminism.

## Stop rule

Do not retune suffix confidence, endpoint semantics, retrieval or comparisons
after seed 2784 opens. Stop on any prohibited path or failed instrument gate.

## Cost

Local CPU only; 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

## Base and inputs

- base: `07349193e`;
- tokenizer:
  `a9fb13f70c69ffe5afed04c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- artifact seed 1:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- artifact seed 2:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;
- sidecar seed 1:
  `2a298b44ea34d1a8e2d40191951d3da9b3300e2ac7de65e6c2ec13950a0b6424`;
- sidecar seed 2:
  `6504dcb047ce7157dd5e968c29441505cf48d993f01ec63222125634c61963b7`.

## Output

`local/deepseek/content-prefix-endpoint/2026-11-06`; fresh panel and output
root absent at registration.

## Steps

Extend the suffix-arbitration evaluator with an aligned retrieved endpoint and
boundary fallback; extend the content driver with this endpoint and its
shuffled-retrieval and oracle controls; run TRAIN preflight; generate seed
2784; evaluate both artifacts and repeats; seal source, panel, reports and
artifacts.

## Minimum test power

- two artifacts;
- at least 200 reachable replies;
- at least 100 deeper-global decisions per artifact;
- at least 20 aligned endpoint completions per artifact;
- boundary, unconditional-length, shuffled guard, local, writer, full and
  oracle controls;
- paired exact/body comparisons and deterministic replay.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 aligned retrieved endpoint sheet:
docs/harness/prep/cpu-training-pilot-content-prefix-endpoint-2026-11-06.md
reviewer: deepseek exact-head self-review (independent delegation unavailable in this run)
note: Implementation 7afcac83c keeps the deepseek-198 content and boundary
  arbitration unchanged until the generated reply diverges from the retrieved
  TRAIN reply. The endpoint evaluator receives only the retrieved reply; the
  evaluation reference is used for scoring and the separately labelled oracle.
  Two focused tests prove exact retrieved-endpoint completion, empty endpoint
  refusal and non-stopping after divergence. The full CPU library passes 406
  tests with four expected ignores; the driver test passes; scoped Clippy and
  formatting pass. The first TRAIN-only preflight exposed that token-decision
  comparison cannot detect a stop-only endpoint change; a driver repair now
  compares complete outcomes including stop reason. The fresh panel remains
  absent.
source:
  `90fb52752b3a09315a43da775e0521a7d78524f8830a4613e4cad69ebed2c54b`;
retrieval:
  `68f19750e8c14370a41a07b4897eec238e1d48313683250667e02052a58970c4`;
driver:
  `6be78357935a900aba8e36611066c99e78218f85bffbe4dae4a637214ca0b3e5`;
release_binary:
  `4671b279340132fc1d9fa078de533d7f2ecf118d229deafe0118f02f78b4191e`;
date: 2026-11-06

## Pre-run GO

GO - repaired exact-head self-review under the current disabled-delegation
policy. Implementation `7afcac83c`, source
`90fb52752b3a09315a43da775e0521a7d78524f8830a4613e4cad69ebed2c54b`,
retrieval
`68f19750e8c14370a41a07b4897eec238e1d48313683250667e02052a58970c4`,
driver
`6be78357935a900aba8e36611066c99e78218f85bffbe4dae4a637214ca0b3e5`
and release binary
`4671b279340132fc1d9fa078de533d7f2ecf118d229deafe0118f02f78b4191e`
match. This was not an independent review. Seed 2784 may now be generated under
the fixed construction and stop rule.

## Outcome

**REJECT.** On both artifacts the aligned true-retrieval endpoint made 17 exact
replies and 644/642 correct body positions, identical to the original
boundary arm. It converted 23 boundary stops into retrieved endpoints and
reduced premature BOS from 73/91 to 67/85, but only 206/229 of 471 reachable
replies reached BOS or the retrieved endpoint. It did not beat the shuffled
guard on exact replies.

The unconditional stored length still made only 497/495 body positions, while
full retrieval made 50 exact and 955 body positions. Both fresh-process repeats
matched byte-for-byte. This closes the fixed full-prefix alignment strategy;
the next termination mechanism must use generated-state branch evidence rather
than a prompt-level retrieved reply.
