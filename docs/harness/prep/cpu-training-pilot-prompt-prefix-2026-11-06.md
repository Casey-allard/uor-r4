# Pre-registration: retrieval-prefix geometric continuation

Run ID: `cpu-training-pilot-prompt-prefix-2026-11-06`

## Status

COMPLETE - **REJECT**. The selected four-token retrieved prefix improved body
positions over model-only (**513/499 vs 46/61**) and shuffled prefixes
(**112/116**), but no prefix arm produced an exact reply or a BOS completion;
all 450 rows consumed the 64-token cap. Record:
[deepseek-217](../../loop/tasks/cpu-training-pilot/deepseek-217/REPORT.md).

## Question

Can a short exact-memory reply prefix anchor the geometric FBS model to produce
more complete correct replies than model-only generation on fresh phrasing?

## Fixed construction

Retrieve the raw unigram-Jaccard top-1 TRAIN reply with lower-ordinal ties.
Candidate forced prefix lengths are 1, 2 and 4. Prefix tokens are fed through
the exact CPU `CpuFbsForward` writer heads after the prompt; generation then
continues greedily using the existing integer stop contract.

Arms:

1. model-only;
2. full stored reply;
3. retrieved prefix 1/2/4 plus continuation;
4. shuffled retrieved prefix 1/2/4 plus continuation;
5. reference prefix 1/2/4 diagnostic.

TRAIN validation uses leave-one-out nearest retrieval to choose one prefix
length. Ties choose the shorter prefix. The choice is frozen before seed 2781.

## Data split

TRAIN `tt-2711` and a fresh protocol-2 seed-2781 panel. No evaluation reply
enters retrieval. Both public dual-c131 artifacts run with a 64-token cap.

## Instrument gate

Fresh panel and output roots are absent at registration; source, artifact,
tokenizer and sidecar pins match; the selected prefix is recorded before
evaluation; all arms emit on every reachable row; repeated runs match after
timing removal.

## Bar

On both artifacts, the selected retrieved-prefix arm must beat model-only on
paired exact replies, retain model-only body positions, beat its
same-length shuffled-prefix control on paired exact replies, and emit every
reachable row with zero premature BOS.

## Outcomes

**KEEP** if all bars pass; **REJECT** if the prefix is active but a primary
quality bar fails; **INSUFFICIENT_POWER** below 200 reachable rows;
**CONTROL_FAULT** on input, control, emission or determinism failure.

## Stop rule

No prefix retune after seed 2781. Stop on any prohibited path or failed
instrument gate.

## Cost

Local CPU only; 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

## Base and inputs

- base: `423dcf4d7`;
- TRAIN tokens:
  `140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91`;
- TRAIN mask:
  `d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad`;
- TRAIN manifest:
  `d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5`;
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
- implementation: `b2adf76e7`;
- reply-eval source:
  `ce2d7fb0f13d936a9167e9ccb0a91bf46456d22c4f871d133225f71e2a32c04f`;
- CPU crate export:
  `7a9b3e5cf0376e42a5273054863c8d2ce526502cb864601b1492bb3ab2c41878`;
- continuation binary source:
  `94cf981067761a202da2be9d77bdcf2fa8c2bd5be724754bc8dcec2673f1fd33`;
- release binary:
  `1b64025637b2d4665757bec592f92cc88df66800388c06e98c073429b99c49a8`.

## Output

`local/deepseek/prompt-prefix/2026-11-06`; fresh panel
`.../panels/dd-2781`. Both absent at registration.

## Steps

Build `cpu-prompt-prefix-continuation`; validate TRAIN leave-one-out prefix
quality; freeze one K in the process report; generate seed 2781; run alternating
fresh processes per artifact; seal JSON, timings, panel bytes and source pins.
Exact command after GO:

```sh
"$BIN" "$SIDECAR" "$TRAIN" "$PANEL" "$ARTIFACT" "$TOKENIZER" \
  --protocol 2 --expected-artifact "$ARTIFACT_SHA256" \
  --expected-tokenizer a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377
```

## Minimum test power

- two artifacts;
- 200 reachable rows;
- three processes per artifact;
- all prefix lengths and matched shuffled controls;
- paired exact and body comparisons;
- zero premature BOS and deterministic repeats.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 retrieval-prefix continuation
sheet: docs/harness/prep/cpu-training-pilot-prompt-prefix-2026-11-06.md
reviewer: deepseek exact-head self-review (independent delegation unavailable in this run)
note: Implementation b2adf76e7 is clean. The exact CPU prefix-forcing path,
  model-only, full retrieval, three retrieved/shuffled/reference lengths and
  paired comparisons compile; the full 399-test CPU suite passes with four
  expected ignores. Inputs and source pins match; seed 2781 and its output root
  remain absent.
date: 2026-11-06

## Pre-run GO

GO - exact-head self-review under the current disabled-delegation policy.
Implementation `b2adf76e7`, source pins and release binary
`1b64025637b2d4665757bec592f92cc88df66800388c06e98c073429b99c49a8`
match. This was not an independent review. Seed 2781 may now be generated under
the fixed construction and stop rule.
