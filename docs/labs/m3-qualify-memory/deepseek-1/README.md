# Automatic serving-side memory smoke

Lab: deepseek. Item:
[m3-qualify-memory](../../../roadmap/items/m3-qualify-memory.md).
Order:
[deepseek-1](../../../loop/tasks/m3-qualify-memory/deepseek-1/ORDER.md).
Sheet:
[FIT/PRE-RUN PASS](../../../harness/prep/m3-qualify-memory-auto-sieve-2026-11-02.md).

## Result

The deterministic serving-side sieve extracted and injected the expected value
in all six registered conversations:

| Seed | Conversation | Emitted memory | Model reply contains value | Stop |
|---|---|---|---|---|
| 1 | puppy name | `Memory: Pickle.` | no | `eos` |
| 1 | tomato action | `Memory: tomatoes.` | no | `eos` |
| 2 | puppy name | `Memory: Pickle.` | no | `max_new_tokens` |
| 2 | tomato action | `Memory: tomatoes.` | no | `max_new_tokens` |
| 1 repeat | puppy name | `Memory: Pickle.` | no | `eos` |
| 1 repeat | tomato action | `Memory: tomatoes.` | no | `eos` |

This is `AUTO_MEMORY_OBSERVED` with `MODEL_RECALL_NOT_OBSERVED`. It proves the
opt-in extractor reaches the merged literal-role memory bridge on both public
artifacts. It does not show model recall, useful conversation, fresh-phrasing
transfer or M3 qualification.

## Instrument

- All 6 processes exited 0 with empty stderr.
- All 15 replies used protocol 2 and produced valid stops (10 EOS, 5
  token-cap).
- The seed-1 repeat matched the first pass exactly after excluding only the
  registered `id/s` timing field.
- Peak RSS was 11,888 KiB; process wall times were 0.17–0.22 seconds.
- Observed display rates were 229.69–350.27 ids/s on the local Ryzen 9 9950X.
- The postflight executable SHA-256 was
  `ed1e082fb485862368062b68d4b996242d1e200b0eb31e610ee63c05abbb38a9`.

Raw stdout, stderr, `/usr/bin/time -v`, status and postflight records are under
`local/deepseek/m3-auto-memory-2026-11-02/` in the measuring worktree. They are
not committed; the exact source, executable, tokenizer and artifact pins are
in the prep sheet.

## Boundary

Every model reply was malformed boilerplate within its seed. Neither factual
reply named `Pickle` or `tomatoes`. The automatic memory line was consumed by
the encoder but did not produce a grounded answer.
