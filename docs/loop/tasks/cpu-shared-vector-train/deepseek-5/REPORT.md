# REPORT: deterministic masked-corpus driver, deepseek-5

Lab: deepseek. Item: [cpu-shared-vector-train](../../../../roadmap/items/cpu-shared-vector-train.md) (#3505). Base: `af1979886`.

## What did not work, first

- **`tokens.u16` is not a raw token array.** The public file has a canonical 64-byte `UORT` header. A first parser treated the header as 32 token ids and correctly failed when the payload and mask lengths disagreed. The final parser checks magic, version, payload count, vocabulary, zero reserved bytes, exact physical size and mask length.
- **The manifest's `tokens` field is payload count, not token-file size.** The token file is `85,776` bytes and its payload is `42,856` tokens; `tokens_bytes` is the physical size. The parser now validates both fields rather than deriving the wrong one.
- **A zero-step panel with eight held-out documents admitted and ranked none of its targets.** This is a real baseline observation on the selected capped panel, not a training failure and not evidence for a gain. The driver reports it without converting no change into a pass.
- **No corpus-scale training ran.** The bounded pilot requires its separate pre-registration and FIT. This unit stops at implementation and a zero-update host check.
- **The sealed `m2-fresh256` inputs and labels are private.** Direct public HTTP requests return `401`, so complete-reply/fresh-phrasing correctness remains `NOT_RUN` until credentials or an approved public fixture are supplied.

## What changed

- Added `CpuMaskedCorpus`, which strictly parses a `UORT` token stream and binary response mask, splits documents at BOS, rejects out-of-vocabulary ids, and refuses a token/mask length or header mismatch.
- Added causal sample construction: every response-masked, non-BOS target uses at most the artifact's full 192-token context from its own document prefix. Evaluation uses the same context and token representation.
- Manifest validation now binds the positive-mask count to `response_tokens` and refuses a `max_tokens` horizon below the artifact context.
- Training and evaluation samples are split by whole document with a caller-pinned positive stride. The cap on evaluation samples is deterministic and cannot pull any target into training.
- Corrected the rank-margin validator to accept a full-context input, matching the serving session's 192 valid steps and the reference corpus window rule.
- Added `cpu-corpus-driver`, which:
  - validates corpus manifest hashes, schema, split, tokenizer vocabulary and BOS against the artifact;
  - runs an explicit bounded number of rank-margin steps;
  - emits interval checkpoints and a final checkpoint;
  - reports accepted/refused steps, target visits, mean step time, target-visits/s, context-tokens/s, checkpoint time, wall time and peak RSS;
  - records baseline and final exact CPU/serving query, token and kept-cell gaps;
  - records target admission, top-1, top-8, margin loss and target rank;
  - applies the existing admission/export gate without calling a loss-only value success.

## Measurements

Zero-update public-fixture check on seed 1 (`a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`) against the public UORT corpus:

| Reading | Result |
|---|---:|
| Documents | 400 |
| Training documents / targets | 320 / 11,440 |
| Evaluation documents / capped targets | 8 / 8 |
| Pinned response-masked targets / maximum horizon | 14,287 / 256 |
| Model context used for every sample | 192 |
| Tokenizer source SHA-256 | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |
| CPU/serving query-coordinate mismatches | 0 / 2,048 |
| CPU/serving token mismatches | 0 / 8 |
| CPU/serving kept-cell mismatches | 0 / 64 |
| Target admitted | 0 / 8 |
| Top-1 / top-8 | 0 / 8, 0 / 8 |
| Peak RSS | 42,900 KiB |

The model's zero baseline is retained as a negative observation. The existing gate returned `accept` only because admission did not fall and export stayed exact; the report also records `panel_changed: false`. No update, corpus pilot or claim that CPU training improves that panel occurred.

## Checks run at the head

- Focused corpus suite: PASS, including exact malformed UORT header controls and two independent checkpoint resumes that reproduce the final artifact, panel reading and checkpoint bytes.
- Full release suite: PASS, 111 passed and 4 release benchmarks ignored.
- Strict clippy, formatting and D23/D25/D32 ratchets: PASS.
- The exact public zero-update fixture check: PASS with the readings above.
- Local PR check and all 11 loop gates: PASS.

## Skills used

No repository skill was invoked. The public UORT/payload distinction was checked against `uor-r4-discrete::corpus` before the parser was corrected.

## Still open

- A pre-registered bounded pilot on the Ryzen 9 9950X.
- A trained float shadow paired to the public integer artifacts.
- Complete-reply and genuinely fresh-phrasing correctness; the sealed panel is currently private (`401` without credentials).

## Plain summary

The CPU trainer now has a strict corpus front end and a resumable bounded-run CLI over the genuine trained FBS artifacts. It reports exact serving parity, ranking, admission, resources and checkpoints without turning a zero-step baseline or a loss value into a success claim.
