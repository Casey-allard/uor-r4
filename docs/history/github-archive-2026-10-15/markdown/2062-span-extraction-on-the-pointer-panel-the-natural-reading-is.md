# #2062 — Span extraction on the pointer panel: the natural reading is a negative, a fitted variant is a non-significant point estimate (#2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T17:23:20Z
- **Updated:** 2026-10-09T17:28:36Z
- **Closed:** 2026-10-09T17:26:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2062
- **Merged:** 2026-10-09T17:26:24Z
- **Merge commit:** 9731faac30f582888d240756a49393024ad9aa5c
- **Base / Head:** main / UOR-Foundation:span-extract-2026-10-09
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Pre-registered span-extraction read-out on the 84-cell pointer panel. The natural reading is a decisive NEGATIVE; a fitted variant is a non-significant point estimate. The accepted best remains 72/84.**

Pre-registration frozen and sent before any run: `docs/evidence/span-extraction-preregistration-2026-10-09.md` (sha256 `55469a43…`, addendum `27a84aa0…`), including its own weaknesses and the A1–A9 acceptance criteria.

## Measured

| configuration | baseline | exact | gained | lost | fired | template-disjoint |
|---|---:|---:|---:|---:|---:|---:|
| mode 1 (headline: mid-reply runs, anchor kept) F=0.5 | 72/84 | 72/84 | 0 | 0 | 5/84 | 0 |
| mode 1 F=0.75 | 72/84 | 72/84 | 0 | 0 | 5/84 | **1** |
| mode 1 F=1.0 | 71/84 | 71/84 | 0 | 0 | 5/84 | **1** |
| mode 2 (**fitted**: anchor dropped) F=0.5 | 72/84 | **77/84 = 0.9167** | 5 | 0 | 5/84 | 0 |
| mode 2 (**fitted**) F=0.75 / F=1.0 | 72 / 71 | 76 / 75 | 4 / 4 | 0 / 0 | 5/84 | 1 / 1 |
| mode 3 (broad control: any run) F=0.5 | 72/84 | 70/84 | 0 | **2** | 82/84 | **3** |

The headline fires on exactly the five class-C cells and gains nothing: in all five the pointer re-acquires the premise one id before the value, at the copula `" is"`, so the span it locked onto is `" is Ithmar."` — the failing prefix is *inside* the span the pointer selected. At F=0.75/1.0 it also fires on a template span (`natural/n_drovik`), which the pre-registration makes a FAILURE.

The fitted variant's +5/0 at F=0.5 is **p=0.0625, Newcombe [−0.0391, +0.1594] — a better point estimate, NOT an improvement**, and it is fitted to the same five cells it recovers (that word must travel with its number). The broad control shows the mid-reply gate is load-bearing: it loses `natural/n_candrake` and `natural/n_kelbrin`, keeps three other truncated cells exact, and produces three template firings. Structural scan: exactly 5 of 84 sealed traces have a mid-reply pointer-locked run of ≥ 2 ids, and they are exactly the five class-C cells, so the mid-reply gate cannot touch the other 79.

## Evidence discipline

- **A1** default proven field-for-field: 9 files / 288 records, 0 differing fields, 0 missing, 0 added records.
- **A2** parser self-check PASS (arm-ptr 35/84, channels 0.5149/0.8351/0.5610/0.4901) before any new number.
- **A3** the rule-off run at the 72/84 configuration reproduces the sealed idstop3 trace run field for field (96 records).
- **Replay witness** 84/84 span decisions and 84/84 reply ids in every measured configuration; cross-checks disagree as they must.
- Three focused Rust tests pass; they caught a real bug in the first build (a locked run ending on a non-locked step was dropped), so the first pod run fired 0/84 — every number here is from the fixed binary `f8bb8c47…`.

Cost: one EU-RO-1 pod (2×4090, ~26 min ≈ $0.80, 33 runs). Runs and binary on the volume under `bindprobe/span-extract/` and in iCloud (`span-extract`, md5 `e2692d9d…`); pod released and deleted.

Record: `docs/labs/span-extraction-2026-10-09/README.md`; result `docs/evidence/span-extraction-2026-10-09.txt`. References #2029, #2032.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6085834949 — 2026-10-09T17:23:38Z — Casey-allard

Exact-head checks at `8a663718f` (branch `span-extract-2026-10-09`, base `0284e6eb4`), all run locally on the laptop CPU; GPU work ran on pod `ajrklwvw9uwvzr` (EU-RO-1) and is archived.

- `cargo check -p uor-r4-training --example bind-probe-exact` → exit 0 (CARGO_TARGET_DIR=/tmp/span-target).
- `cargo test -p uor-r4-training --lib span_extract` → 3 passed; 0 failed:
  `the_span_extract_rule_takes_the_span_the_pointer_locked_onto`,
  `the_span_extract_rule_is_off_by_default_and_refuses_a_non_span`,
  `the_span_extract_decoder_emits_exactly_what_the_rule_finds`.
- `python3 -m py_compile` on the four sealed scorer scripts plus the new `scripts/idstop-span-extract.py` → OK.
- `cargo fmt` applied to the touched files only (unrelated pre-existing drift in `mqar-bench.rs`, `relation_compiler.rs`, `demand-store-2turn.rs`, `mask-scan.rs` reverted and NOT part of this PR).
- `python3 scripts/check_claim_wording.py` → Claim-wording gate passed.
- On the pod, from the patched checkout: default proof 9 files / 288 records field-for-field identical to the sealed references; parser self-check PASS; rule-off run reproduces the sealed trace run; 33 generation runs, binary sha256 `f8bb8c47696d10f8d8c593df3761ce59f65967875bd11b49380d41bda19a8117`.
- Offline replay witness on the laptop: span decisions 84/84 and reply ids 84/84 for mode1-f0.5, mode2-f0.5, mode3-f0.5, mode1-f0.75, mode1-f1.0, mode2-f0.75, mode2-f1.0.
- `cargo fmt --check` is NOT clean on main for the four unrelated files above; that drift pre-exists this PR and is untouched here.

Pod released and deleted after the run set; artifacts on the volume (`bindprobe/span-extract/`) and in iCloud (`span-extract`, md5 `e2692d9d4c7ed5f98df41d9d7a10d1b7`).



---
_Comment/review ids in this file: 6085834949_

