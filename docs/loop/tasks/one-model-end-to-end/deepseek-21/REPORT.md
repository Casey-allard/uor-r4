# REPORT: one-model-end-to-end, deepseek-21

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Laptop only; no GPU, pod or spend.

## What did not work, first

- **Per-turn rendering does not make the model stop by itself.** It still stops on EOS for
  0 of 256 rows and runs to the 64-token cap on all 256. The ranking result explains why:
  EOS is close under this rendering, but another token still ranks first.
- **One-user rendering leaves EOS far outside the decode competition.** EOS is never in
  the top 8; its best rank is 18 and its median is 138. The free decode's 0 EOS stops are
  therefore consistent with both exposure and ranking, not with a serving stop-rule fault.
- **Same-scorer float/integer parity is still unavailable.** `served-reply-panel` consumes
  a `UORLUT01` integer artifact. The float checkpoint reply decoder and
  `load_reads` are private, and the float `score_rows` panel format is not the M2 packet
  schema. There is no interface today that renders an M2 packet, decodes the float
  checkpoint, and decodes the integer artifact through one scorer. The unit records this
  as `NOT_RUN` instead of comparing unlike scorers.
- **`m2-reply512` content remains unavailable.** A recursive store listing found only
  `data/m2-fresh256/`; the known `b9661606…` input and `84991e0…` label files are absent.
  A bounded repository/local search found their hashes in documentation and audits, not
  their bytes. It remains `NOT_RUN`.
- **The historical `dialogue-recall-corpus` source is not an active Cargo target.** This
  unit did not restore it or the excluded historical modules.
- **The first fresh review rejected the initial implementation.** It found that nearest
  ranks ignored the served point-key override, the default rendering newly refused empty
  context turns, and the tests did not exercise prompt-to-first-target alignment or ties.
  The revision passes the actual served token into the rank diagnostic, moves empty-turn
  refusal to `per-turn/v1` only, and moves the sequence walk into tested CPU-forward code.
- **The second review found two remaining boundary faults.** Teacher forcing had counted
  the final EOS as if it were fed back, rejecting one valid sequence at the context edge,
  and teacher-forcing time was included in generation tokens/s. Both are fixed and tested.

## What changed

- `served-reply-panel` now accepts `--rendering one-user-turn/v1|per-turn/v1`.
  `one-user-turn/v1` remains the default and keeps its prompt and report shape.
  `per-turn/v1` emits one explicit `User:` message per `Context`, then the query, before
  the open assistant marker. Empty turns are refused.
- Added `--teacher-forced 1`. It renders the same prompt, teacher-forces the shortest
  frozen accepted answer plus EOS, and records each target's rank from the same integer
  nearest reader generation uses. It reports top-1/top-8/top-32, min/p50/p90/p95/max,
  per-stratum totals and per-row EOS ranks. The default report gains no new key.
- Added packet-local nearest-rank helpers in `served-reply-panel`. They use
  `CpuFbsForward`'s public query state and shared table, follow serving order
  (larger score first, lower token id on a tie), record the actual served token
  and keep the diagnostic inside this item's scope. The sequence helper consumes
  every target exactly once and allows an unfed final target at the context
  boundary. Generation and rank timing are now reported separately.
- **The latest review found two remaining faults.** The first diagnostic ranked only the
  raw L1 order, so a point-key opening could be served at rank 1 but reported far below it.
  The diagnostic had also been placed outside this item's scope. The packet-local helper now
  reports `rank` in the actual served order and `raw_rank`, names `raw_nearest_token`, and
  marks `point_key_override`; a point-key opening therefore holds rank 1 even when its raw
  score would not. A fixture pins that case, and no CPU-train file changes.
- Added twelve focused tests covering the unchanged one-user render, explicit turns,
  malformed/empty turns, EOS/reference alignment, acceptance, stop order, rank-tail
  aggregation, switch parsing, nearest ties, the point-key override, teacher-forced target
  alignment and nearest rank/order.

## Measurements

Panel `m2-fresh256` was regenerated from its deterministic instrument and matched all four
registered seals: `plan 0ed1ab98…`, `inputs 1de2fc28…`, `labels 3b5c5477…`,
`manifest 7c33f3f7…`. The artifact is Claude-301 seed 1, `b64L-s1.uorlut`
`9a370566…`; the tokenizer is `d36d3e87…`.

| rendering | complete | EOS/BOS/cap/context | EOS rank min / p50 / max | EOS top-8 | EOS top-32 |
|---|---:|---|---:|---:|---:|
| `one-user-turn/v1` | 0/256 | 0 / 0 / 256 / 0 | 18 / 138 / 873 | 0/256 | 45/256 |
| `per-turn/v1` | 0/256 | 0 / 0 / 256 / 0 | 5 / 7 / 13 | 198/256 | 256/256 |

Across all reference reply tokens, one-user has 1,577/2,832 top-1 targets, median rank 1,
p90 400 and max 4,032. Per-turn has 1,573/2,832 top-1 targets, median rank 1, p90 10 and
max 4,027. Thus per-turn does not improve ordinary reference copying overall; it moves EOS
from far outside the candidate set to consistently near the top. That is a ranking result
for this checkpoint, not evidence that per-turn free generation works.

Outputs:

| run | report SHA-256 |
|---|---|
| default one-user, generation only | `553873fa3ede1242e350c16f203938552b44f8f30c74fcc9cb7bec3ddc6251b6` |
| one-user plus teacher-forced ranks | `759f69877c828ed4a7be6b611ccbd0ad4e978d3843f6411889428eb151aea4ac` |
| per-turn plus teacher-forced ranks | `2152bc597e7463c74837305f1c6aa000ffc0d71db1e1878a211e29ebc87c4141` |

## Checks run at the head

- `UOR_PANEL_TOKENIZER=<pinned tokenizer> cargo test --offline --locked
  --features cpu-parity --bin served-reply-panel --no-fail-fast`: 14 passed, 0 failed,
  including a real point-key override, teacher-forced target order, the context
  boundary and nearest ties.
- `cargo clippy --offline --locked --features cpu-parity --all-targets
  --no-deps -- -D warnings`: passed.
- `rustfmt --edition 2021 --check` on the changed binary: passed.
- `cargo run --release --offline --locked --features cpu-parity --bin
  served-reply-panel` on the regenerated panel in both renderings: exit 0.
- The one-user generation-only run reproduces claude-309's published totals:
  0 complete, 0 EOS/BOS/context and 256 cap stops.
- `git diff --check`: passed.
- Workspace-wide formatting was not applied because unrelated files carry existing drift;
  changed-file formatting was checked by review.

## Skills used

No system-provided Codex skill applied. The repository's loop, charter, geometry and
client-side CPU serving rules were used directly.

## Still open

- The short SFT run should use the per-turn rendering as its evaluation view. It must
  first make EOS top-1, then make accepted answers top-1, and finally make whole replies
  complete.
- Add same-scorer float/integer panel parity when the float checkpoint decoder exposes a
  public packet-level interface or the integer scorer accepts a float comparison input.
- Recover `m2-reply512` only if its original bytes become available; do not reconstruct a
  qualification panel from hashes.

## Plain summary

The model's failure has two parts. Under the current one-turn prompt, EOS is nowhere near
the token the model would choose. Under an explicit turn-per-message prompt, EOS is usually
within the top eight but is never first, so the model still talks to the cap. This makes
the next fine-tune concrete: train with reply-only loss, preserve the per-turn evaluation,
and watch EOS rank move to 1 before expecting complete answers.
