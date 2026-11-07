# REPORT: one-model-end-to-end, claude-312

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Laptop only; no GPU, no spend.

## What did not work, first

- **No result yet.** This unit registers a run; it trains nothing. Whether (b64L) stops when trained on `mw-balp/train` is read by the next unit.
- **The first draft of the sheet had three faults that a run would have paid for.** A fresh FIT review found them (FIT WITH CHANGES, no blockers).
  - **Its collapse bar could not tell the corpus from the recipe.** It counted the most common reply over all 256 rows. (b64L-D3) seed 1 already repeats one looped, capped reply 76 times (claude-309), so a D3-like model would have read as collapsed.
  - **Its scorer control could pass a broken scorer.** It replayed a reading of 0 EOS stops, which a scorer that never sees EOS would also produce.
  - **Its repetition figure was wrong.** It counted positions and gave 26.8 passes. The trainer reads padded document windows, one or more per document, which gives about 10.6 to 17.5 passes.
- **The first draft of the scripts would have billed idle.** A fresh D67 review found it (GO WITH CHANGES, no blockers). claude-306's watcher exits at its time or spend limit. After that, nothing takes the pod down when the run seals. Here those limits are 4 h, close to the run's own length (about 13,000 s projected), so a healthy run that ran late would have left a pod billing (L-004). Its price ceiling was also still claude-306's $1.15/h (`watch-306.sh:26`), not this sheet's $0.70/h.
- **Not checked:** that the corpus's dialogue layout matches the panel's rendering. The sheet calls this an inference: claude-259's models trained on this corpus stop under the same rendering.

## What changed

- **The prep sheet** [`one-model-end-to-end-b64l-mwbalp-2026-11-07.md`](../../../../harness/prep/one-model-end-to-end-b64l-mwbalp-2026-11-07.md).
  - **The recipe:** claude-301's (b64L) at its pin `ff5156bcc`, with only TRAIN changed from D3 to `mw-balp/train`. Seeds 1 and 2.
  - **The bars:**
    - **stopping:** at least 128 of 256 EOS stops at both seeds;
    - **the panel's bar:** 104 rows and 52 pairs, reported;
    - **collapse:** at least 64 identical replies among EOS-stopped rows.
  - **The outcome map** has nine rows, each with one next unit.
  - **I-c,** two scorer controls, and **I-x,** the export check.
  - **The cost:** $3.6–4.5, with a budget notice at $5.60. It is not a cap: a healthy run is never cancelled.
  - **FIT round 1** and the **D67 review**, both applied, and the **pre-run review: GO.**
- **Run scripts,** in `local/claude/claude-312/` (not committed, as for claude-301 and claude-306): `job-312.sh`, `chain-312.sh`, `seal-312.sh`, `watch-312.sh` and `gt/`. They are claude-306's scripts with these changes:
  - TRAIN swapped, and its I-3 block reduced to the two sha checks (the store folder has no `SHA256SUMS` or manifest);
  - a new seal root;
  - MAXSEC of 4 h and a budget of PRICE × 4 h, both now notices rather than stops;
  - a $0.70/h ceiling;
  - an I-t margin of 300 s;
  - `trained_ok` requires the checkpoint.

## Measurements

**I-c, the scorer controls,** with `served-reply-panel` at `be3d4cb0…`, panel `m2-fresh256`. Rows are compared with `jq -S .rows` and `cmp`.

| Control | Artifact | Reference output | EOS / cap stops | Rows equal |
|---|---|---|---|---|
| Negative | claude-308 seed 1 (b64L-D3), `9a370566…` | claude-309 `d02ec432…` | 0 / 256 | yes |
| Positive | claude-259 (b) seed 1 | claude-311 `19849a18…` | 242 / 14 | yes |

**The collapse measure on the existing outputs.** The most common reply text among EOS-stopped rows, with the same count over all rows after the slash:

| (b) s1 / s2 / s3 | (h) s1 / s2 / s3 | (b64L-D3) s1 / s2 |
|---|---|---|
| 129 / 128 / 128 | 152 / 146 / 128 | 0 / 0 (76 / 36) |

For claude-259's arms the two counts are equal.

**TRAIN:**
- `tokens.u16` `3dee858f…`: 15,025,658 bytes, which is 7,512,797 tokens;
- `response_mask.u8` `b5e7352d…`;
- 60,000 documents.

16,384 steps × 64 windows = 1,048,576 windows seen, over 60,000 to about 99,130 windows.

## Checks run at the head

- **The laptop gates,** on the scripts at the sheet's hashes:
  - `gt/test-gates.sh`: 24 ok, 0 FAIL;
  - `gt/mutate.sh`: 8 of 8 mutations caught (`if-split-row`, `i3-no-micro`, `i3-no-seed`, `mw-no-mask-check`, `mw-no-tok-check`, `pin-other`, `ok-no-checkpoint`, `it-rate-32`);
  - `gt/test-seal.sh`: 7 ok;
  - `bash -n` ok on every script.
- **Both I-c controls** exited 0, with rows equal.
- **The trainer at the pin,** read by the D67 reviewer and by me:
  - `load_named_split` re-checks both shas, the UORT header and the token count;
  - `TrainSplit` serializes `tokens_sha256` and `mask_sha256`, which the job's I-3 check reads;
  - `mw-balp/train` is the pin's own pinned default TRAIN.
- **`scale-probe grade`** loads TRAIN with `need_manifest = false` (`lib.rs:555`), and `pair` checks TRAIN only within each cell (`lib.rs:1190`).
- `loop report --check` on this REPORT.

## Skills used

None beyond the loop's own steps.

## Still open

- **The run:** the next unit, on two RTX 5090 pods under this sheet and the scripts at the recorded hashes.
- **The laptop reader for Step 6** (export, `served-reply-panel`, `grade` with `train=` `mw-balp/train`, the pairs and the DEV curves) is to be written in the run unit. claude-306's `read-306.sh` grades D4 and does not fit.
- **`served-reply-panel`'s timer** still re-parses the artifact for every row (claude-311). (b64L)'s artifact is small, so its tokens-per-second stays a decode figure.
- **Recall** stays open whatever stopping reads. The sheet's rows 2 and 3 name the next step after a stop.

## Plain summary

The model we serve never stops talking. Models trained on a corpus of short chat replies do stop, but they were also trained differently, so we could not tell whether it was the data. This unit sets up the clean test: train our model again, the same way, on that chat corpus alone, and count how often its served replies stop on their own. The rules are fixed in advance: what counts as stopping, what counts as giving the same canned reply, and what each result leads to.

Two fresh reviews checked the plan and the scripts before any money is spent. They caught a collapse measure that would have blamed the wrong thing, a scorer check that could not fail, a wrong count of how often the model sees its data, and a watcher that could have left a pod billing idle. All of these are fixed. The run costs about $4 and is the next unit.
