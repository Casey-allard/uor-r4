# #2096 — Criterion 2's protocol pinned: the published float/served pair is cross-window-count (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T21:54:09Z
- **Updated:** 2026-10-09T21:55:22Z
- **Closed:** 2026-10-09T21:55:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2096
- **Merged:** 2026-10-09T21:55:22Z
- **Merge commit:** 3316080db3df5d027cda82a696bfc49a66de3aeb
- **Base / Head:** main / UOR-Foundation:deepseek/criterion2-pin-20261009
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Criterion 2's protocol is pinned, and the pinning changes what the criterion's own headline pair means. **References #2029.**

## The finding

The published "float 0.877 / served 0.933" pair is a **cross-window-count comparison**: the 0.87755 float cell is a **512-window** number and the 0.93277 served cell is a **64-window** number. On one artifact, same stream, same engine, the served column is **0.932773 BPB at 64 windows (24,576 targets)** and **0.886838 BPB at 512 windows (196,608 targets)** — **0.045935 BPB apart on identical bytes**, larger than the whole 0.033 gap the pair was used to measure.

**Matched triple, declared fixed count — 512 windows / 196,608 targets**, stream `e5f400b0…` whose own byte basis was re-verified on the copy used (6,194,589 tokens / 17,576,697 bytes / **2.837427 B/token**), one `lut-evaluate reference=true` run, checkpoint `cbef6906…` + export `721d4bdb…`:

| column | nll | BPB (stream basis 2.837427) | tool `bits_per_byte` (sample 2.829585) |
|---|---:|---:|---:|
| float | 1.7259248087 | **0.877550** | 0.879982 |
| reference (artifact dequantized) | 1.7441922849 | **0.886838** | 0.889296 |
| integer (multiplier-free served engine) | 1.7441928232 | **0.886838** | 0.889296 |

quantisation **0.009288 BPB**, engine **2.7e-7 BPB**, top-1 agreement 0.92820. The tool's `bits_per_byte` is on a second byte basis (the scored targets' lens bytes) — both are named in the record, because quoting one while computing the other is a second way to make a BPB number unreproducible.

## The artifact behind the pair

The recorded artifact's LUT `2ac4c306…` is **gone, established exhaustively**: every `*.lut` on the volume hashed, every file of its exact 11,891,652-byte size checked, the whole `/workspace` tree text-grepped for the digest and for the recorded nll `1.7250754982233047`, and all **604** cloud-store entries read — only checked-in docs copies survive. What the EU-RO-1 volume `rfsx702p68` does hold is the **independent retrain** of the same recipe (`chat-served2-20261008/model-a1`, `cbef6906…`, batch 32 / lr 4e-4 / 12,207 steps / chat-v0-p2 train only) and its GPTQ export `721d4bdb…` — the artifact behind the published float cell and the round-23 quantiser table; its float forward reproduces the published 0.87755 to five decimals.

## Criterion 2, honestly

At the recorded 64-window protocol the served cell misses 0.90 by **0.032773** (3.52 % relative, 0.41 % of the 8-bit/byte ceiling). At the pinned 512-window protocol the matched served cell is **below** 0.90 by **0.013162** — but that margin is **smaller than the 512-window re-sampling se of 0.014525 BPB**, and the same artifact misses at 64 windows. criterion 2 is **not met, not claimed and not measurable at the precision its threshold implies**; declaring it met by choosing the favourable window count is the protocol-shopping this round exists to prevent.

**Standing requirement: every BPB claim names its window count and byte basis.**

## Delivery

- `docs/labs/criterion2-protocol-pin-2026-10-09/README.md` — the record (identity, method, matched pair, pinned protocol, decision, limitations)
- `docs/evidence/criterion2_protocol_pin_2026-10-09.txt` — identities, search transcript, commands, tables, cost
- newest-first `current-state.md` entry ending in **Next:**
- stale-doc fixes: the previous round's "Next:" annotated as executed; the previous lab record's limitations 1–2 resolved for the surviving artifact; dated annotations on `chat_stack_served_d11_2026-10-08.txt` and `quantiser_arm_verdict_2026-10-08.txt` (their pair is no longer read as like-for-like)
- STATUS.md / ROADMAP.md / README.md / #2028 milestone cell: **the protocol qualifier only** — no headline number changed and no capability moved, because the served figure at the protocol it was recorded on is unchanged
- `python3 scripts/check_claim_wording.py` → **pass**

## Limitations, fully stated

1. Everything is measured on the **retrain**; the original artifact is unrecoverable, and its own matched pair is not inferred.
2. **One unverified cell: `d11-evaluate` at 512 windows.** It was killed when a hygiene pass removed this worktree mid-run (the pushed branch is why the completed run's numbers survived). It was **not** re-run: the pod cap was full (3 pods, $7.36/h of $8.00/h) and the check confirms rather than discovers. The artifact's own recorded 64-window `d11-evaluate` carries D11-exactness (`d10 = d11`, diff 0.0, top-1 1.0, max abs logit difference 0), and the completed run's integer-minus-reference term is 5.4e-7 nats.
3. One window draw (se 0.0145 BPB); no second window count was run.
4. The retrain pair is in no cloud-store index entry and survives only on the non-canonical EU-RO-1 volume; preserving it is the named next step.

**Pod spend: $0.4336** (1×4090, EU-RO-1, 1754 s at $0.89/h, read-only volume access, released and deleted; `ps` clean at release). A second pod attempt for the re-fetch was refused by the rate cap and cost nothing. No training, no export, no quantiser tuning, no float generation; no laptop grading.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

