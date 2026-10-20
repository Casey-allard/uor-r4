# #814 — Decode-default flip: sampled decode default on CLI + all serving surfaces (#655)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-19T01:14:19Z
- **Updated:** 2026-10-09T13:08:49Z
- **Closed:** 2026-08-19T01:34:45Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/814
- **Merged:** 2026-08-19T01:34:45Z
- **Merge commit:** 83fa97612fe84ec4fa59b6e09fcfbf1141f52d69
- **Base / Head:** main / UOR-Foundation:decode-default-flip
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Slice 2 of the #655 decode-default decision (2026-08-19, maintainer sign-off issuecomment-5335968364): **seeded weighted sampling is now the default decode on every product surface**; greedy is the explicit opt-out. Stacked on #813 (the machinery slice) — the first commit here is #813's.

- **CLI**: `ask`/`chat` default to sampled decode with the pinned `chat::DEFAULT_SAMPLE_SEED` (42 — the F-p2 canary's 15/15-valid seed). `--greedy` opts out; `--sample <SEED>` overrides the pinned seed.
- **Server**: `R4g1Decode` + `r4g1_decode_from_request`, derived from the RAW request fields on all three chat surfaces (native `/api/chat`, OpenAI chat-completions, OpenAI responses): `temperature <= 0` → greedy opt-in; otherwise sampled with the request `seed` override or the pinned default — identical requests reproduce identical completions. The autotuned geometric temperature is never read as a greedy opt-in.
- **Wire**: `seed` added to `VendorChatCompletionsRequest` / `VendorResponsesRequest` (previously failed closed under `deny_unknown_fields`) and the shared `InferenceRequest` (REST/WS/WASM).
- The opt-in witness envelope keeps greedy decode (witness claims bind the greedy selection). D4 abstention is decode-mode independent by construction (#813's engine tests pin it).
- `docs/SERVING_MODEL_DISCOVERY.md` Tier 1 gains the decode-default record.

## Canary evidence (this branch's binary, smollm2-360m-broad, the 20 declared F-p2 prompts × 2 passes)

| comparison | result |
|---|---|
| new default arm vs archived F-p2 `--sample 42` arm | **byte-identical, 0/40 diffs** — the default IS the measured 15/15-valid decode |
| new `--greedy` arm vs archived F-p2 greedy (old default) | **byte-identical, 0/40 diffs** — the opt-out is exactly the old behavior |
| default pass 1 vs pass 2 | **0 diffs** — pinned-seed default is deterministic |
| witnesses | default = `r4g1-sampled` 20/20; greedy = `r4g1-beam` |

The flip changes exactly the default binding and nothing else.

## Tests

`r4g1_decode_defaults_to_pinned_seed_sampling_with_greedy_opt_in` (mapping: default sampling + pinned seed, nonzero temperature stays sampled, request seed override, temperature-zero/negative greedy, greedy wins over seed) · workspace clippy `-D warnings` clean · fmt clean · api suites pass.

## Refs

#655 (decision + F-p2 canary packet issuecomment-5335796517) · #784 (attractor substrate) · #813 (machinery slice)

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

