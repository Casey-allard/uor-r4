# Pre-registration: top-K retrieved branch endpoint

Run ID: `cpu-training-pilot-content-branch-endpoint-2026-11-06`

## Status

Complete as **REJECT**. FIT/GO were recorded at `75a126c97`. Top-8 made 48
endpoint stops per artifact and reduced premature BOS, but only 175/208 of 437
rows completed and exact/body stayed at the boundary values of 14 and 936/933.
The shuffled guard also made 14 exact replies. Repeats were identical.

## Question

Does retaining the top eight retrieved TRAIN reply branches avoid the
single-branch endpoint brittleness of deepseek-225 while preserving
deepseek-198's content trajectory?

## Fixed construction

Rank TRAIN replies by raw unigram-prompt Jaccard with lower-ordinal ties. Use
the top eight non-empty complete replies, capped at 64 tokens, as endpoint
branches. At each step suppress the boundary while any branch extends the
complete generated reply; stop when it exactly equals any branch; otherwise
restore the unchanged boundary path. Retrieved content is never injected.

Arms: top-8, top-1, original boundary, shuffled top-8, local top-8,
unconditional length, full retrieval and reference-branch oracle.

## Data split

TRAIN `tt-2711`: tokens
`140cf50d638d6066aec0a31345183712b6fc5d95502384223b91af9504b48f91`,
mask `d0aaa2b1007e62098ab943187f25e13bddbaed3508dce881a176c2827052c0ad`,
manifest `d570caedbb628167c59b96b14746863b99998c2f67289d1deca16dbebd402ba5`.

Fresh panel: `cell=dev,dev`, seed 2785, protocol 2, 128 conversations.

## Instrument gate

Pins match; panel/output roots absent at registration; suffix table has zero
BOS targets; suffix and retrieval-reply shuffles are non-vacuous; all branches
are non-empty and at most 64; every arm shares the panel and artifact; repeats
match after timing removal.

## Bar

Top-8 must, on both artifacts: make zero premature BOS stops; complete all
reachable replies; retain top-1 and boundary exact/body counts; beat shuffled
top-8 exact replies by two sigma; change a complete reply; stop at least 20
retrieved endpoints; and repeat byte-identically.

## Outcomes

**KEEP** if all pass; **REJECT** if active and a quality/termination bar fails;
**INSUFFICIENT_POWER** below 200 reachable, 100 deeper-global decisions or 20
endpoint completions; **CONTROL_FAULT** on drift, vacuous control or
nondeterminism.

## Stop rule

Do not retune K, suffix confidence, retrieval or comparisons after seed 2785
opens.

## Cost

Local CPU only; 30 minutes, 2 GiB peak RSS and 32 MiB retained evidence.

## Base and inputs

- base: `7b1e888ef`;
- tokenizer:
  `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`;
- artifact seed 1:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- artifact seed 2:
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`;

## Output

`local/deepseek/content-branch-endpoint/2026-11-06`; absent at registration.

## Steps

Add top-K retrieval and branch endpoint evaluation, extend the driver, run
focused/full tests, freeze FIT/GO, generate seed 2785, evaluate both artifacts
and repeats, seal evidence and deliver through review.

## Minimum test power

Two artifacts, 200 reachable rows, 100 deeper-global decisions, 20 endpoint
completions, top-1/boundary/shuffled/local/length/full/oracle controls.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030 top-K branch endpoint sheet:
docs/harness/prep/cpu-training-pilot-content-branch-endpoint-2026-11-06.md
reviewer: deepseek exact-head self-review (independent delegation unavailable)
note: Implementation 75a126c97 adds exact top-K unigram-Jaccard retrieval and
  a branch endpoint that suppresses boundary only while any retrieved TRAIN
  reply extends the complete generated prefix. One focused branch test, the
  driver test and the full CPU library pass (407 passed, four expected
  ignores). Top-K branches are used only for stopping and never inject content.
  The fresh panel and output root remain absent.
source:
  `bffcaa6c6f857960da89f83a9c289e1550bf5a3242ecd1a957d553dcc860edf6`;
retrieval:
  `b365f2fd1b59313291380c1c50f92c8af8fe2cd20e1cf81f764b044826349750`;
driver:
  `dccd006f6de5eb1a1520502f676692d31edbf61575b7c7d8b7e061aec9283e76`;
release_binary:
  `fa4d9218d31161938eb797f9a6be31abb4a3269f5ab610cc6658061aa9e7445c`;
date: 2026-11-06

## Pre-run GO

GO - exact-head self-review under current delegation policy at implementation
`75a126c97`. Source, retrieval, driver and release binary hashes match. This is
not an independent review. Seed 2785 may now be generated.
