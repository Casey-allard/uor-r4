# REPORT: infra-loop-harness, deepseek-117

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `8f2414f2e`.

## What did not work, first

- The merged arm-r cycle fixture expected `short_cycle` at an eight-token
  budget. Against the real artifact it stopped at `max_new_tokens` before the
  cycle was observable, so both cycle cases correctly failed the audit.
- Capping generation longer was not sufficient by itself: one natural answer
  ended with EOS rather than a short cycle. The calibrated case therefore uses
  the recorded `My dog is called Rex.` prompt, whose arm-r reply reaches the
  declared period-one cycle after ten tokens.

## What changed

The cycle fixture now has two distinct checks:

- `recorded-stop` asks `My dog is called Rex.` with a twelve-token budget and
  requires the exact `short_cycle` stop.
- `no-unreported-exhaustion` asks `I grew up in Lisbon.` and accepts EOS, the
  token cap or a short cycle, while still requiring a reported terminal stop.

The contract fixture hash and generated views are updated. No serving code,
model artifact or cycle rule changed.

## Measurements

Against the reproduced arm-r artifact:

- `My dog is called Rex.`, twelve-token budget: ten generated ids,
  `short_cycle`, text `Got it, you grew up in in in`.
- `I grew up in Lisbon.`, twelve-token budget: six ids including EOS, text
  `Lisbon.`.

The reproduced artifact SHA-256 is
`707dfdc3b564df95497d57303decccebe42ac382045d7a0aae461137fec18ad0`.

### Stack contract identity

Arm r remains `conformance/stack/fbs-arm-r.toml`, schema
`uor-r4.stack-contract/2`; its updated semantic hash is
`6a3052a4aad786f0aac6e89f2057b99d6e316424bbdbfa1c9738698fc55700ae`.

## Checks run at the head

- Real arm-r `stack-audit` against the reproduced sealed artifact: all
  representation checks and all probes `MATCH`; only the two optional
  provenance checks are `NOT_RUN`.
- Real arm-rt `stack-audit`: all representation checks `MATCH`; the two known
  copied-value probes fail because the response says `It's 40.` instead of
  `47`.
- Contract generation/check, repo-model and xtask tests, report check, quick
  gates, planted fault, exact-head review, CI and fresh-main verification are
  recorded by the PR after the final head.

## Skills used

None.

## Still open

- The arm-rt copied-value negative remains a model/configuration result; this
  unit does not retrain or alter it.
- New artifacts can embed contract/tokenizer provenance; the historical
  artifacts audited here predate it and correctly report that detail as
  `NOT_RUN`.

## Plain summary

The cycle probe was asking the model to cycle before giving it enough tokens
to do so. This repair uses a prompt and budget that demonstrably reach the
existing cycle stop, while a second case checks that an ordinary reply reports
EOS or a cap instead of silently exhausting. The serving model itself is
unchanged.
