# #2338 — Rung 0b source: the lawful native starling read, 16/16 finite CUDA controls (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T15:28:25Z
- **Updated:** 2026-10-13T15:33:11Z
- **Closed:** 2026-10-13T15:30:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2338
- **Merged:** 2026-10-13T15:30:14Z
- **Merge commit:** 0497528e20a9cc764628c170c0c2895bf24acb8e
- **Base / Head:** main / UOR-Foundation:claude/starling-rank-read-2026-10-13
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Rung 0b source: the lawful native starling / murmuration read. References #2032.

**Line:** starling read · **count 3/3** (source PR; headline unchanged) · headline "softmax at runtime: yes" → yes. The next action on the line is the pivot card naming the §5 decisive run. **No training, no quality claim, no D11 claim. Training stays held.**

## What
- **K1 neighbour selection on the GPU** (`starling_native.cu`): rings of depth M per 600-cell vertex, cleared at BOS; home vertex plus its 12 neighbours; K kept plus E exploration slots. **Built ahead of time by nvcc** (`build.rs`) into its own module and launched through the audited `launch_prebuilt`. No NVRTC (owner, 13 Oct).
- **Hard rank read** with a learned null score (`starling_native.rs`). Exact value and rank-table gradients.
- **Declared credit** through one `CreditCapture` op:
  - the pairwise utility-ordered hinge (content q/k and the null);
  - the off-home address margin (query address only);
  - exploration slots with zero forward weight.

  It replaces the D23-banned softmax surrogate.
- **Trunk:**
  - the native admission of `a` layers;
  - FBS address maps;
  - the dense NoRead, age and Lorentz tensors removed from native read layers;
  - no context bound;
  - `config.json` field `native_starling_read` with symmetric save/load.
- **CLI:** `native_read=starling_native:M:K:E` with a **mandatory** `starling_credit=`.
- **Docs:**
  - the [record](docs/labs/starling-rung0b-source-2026-10-13/README.md) with receipts;
  - the brief update;
  - the [qualification sheet](docs/harness/prep/m4-starling-rung0b-finite-2026-10-13.md);
  - the current-state entry.

## Review and fitness
- Design: card 6146209361, A1 6147797508.
- PM first pass FIT-recommended (6147268508).
- **Codex final second pass: TEST FITNESS: FIT for the §4 finite controls**, with binding fixture corrections (6148141029), all applied.

## Checks
| Check | Result |
|---|---|
| **GPU, RTX 5090**, nvcc 12.8.93, at `91788550a` | **16/16 PASS** (`test result: ok. 16 passed; 0 failed`). Training crate at this PR head is byte-identical to that source. Receipts with SHA256SUMS in the record; HF store `vast-archive/vast-55762148/` |
| local, without CUDA | `cargo check --lib --tests --examples` OK; the CUDA tests type-check ungated; 21 starling tests and the CLI contract test pass |
| ratchets | no-softmax 1648/1648 · no-matmul 93/93 · claim wording OK · fmt OK · harness check: no FAIL |
| crate lib suite, without CUDA | 948 pass; the 12 failures are identical on clean `origin/main` (snap / `joint_campaign` / `whole_path_learning`) and predate this change |

The pod was set up **without** the bootstrap's CPU-reference parity gate (D24, #2028 6148671105). GPU time ≈ $0.27.

## Known gap, recorded and proposed for §5
A1 trains the query address only, so the key-address map never learns.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6149079551 — 2026-10-13T15:28:34Z — Casey-allard

APPROVE at exact head `235c8a6c2f0334c77efdba77c9d36c5b6758de2b` (Claude lab self-review, owner exact-head rule). Codex holds the design FIT (6148141029). The training crate at this head is byte-identical to the GPU-qualified source `91788550a` (16/16 on RTX 5090). Local: check, starling tests, CLI contract test and ratchets green.


---
_Comment/review ids in this file: 6149079551_

