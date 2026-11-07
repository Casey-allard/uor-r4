# REPORT: one-model-end-to-end, claude-314

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Laptop only; no GPU, no spend.

## What did not work, first

- **No result yet.** This unit registers a run; it trains nothing. Whether (b64L) on the D3 + mw-balp mix stops, varies and keeps D3's held-out text is read by the next unit.
- **The first draft of the sheet had two gaps that a run could have paid for.** A fresh FIT review found them (FIT WITH CHANGES, no blockers).
  - **The export had no control.** Both scorer controls scored artifacts exported earlier, so a faulty export of this run's seals could have read as "the mix never stops" (row 6). That is the L-007 pattern: a failed measurement cloaked as a mechanism negative. I-xc now re-exports claude-313's seal, which must reproduce its artifact byte for byte. It did.
  - **The held-out reading had an unlabelled confound.** At a fixed 16,384 steps, D3 is 69.2% of the windows seen: 4.73 passes over D3 against (b64L-D3)'s 6.84. A held-out cost would mix fewer D3 passes with mw-balp's content. The sheet now says this run cannot separate them.
- **The pre-run review found no blocker or major** (GO). It found two minors: no test pinned the TRAIN fetch target, and one variable name was reused. Both are applied.
- **Laptop slips on the way, all caught before any review:**
  - BSD `od` has no `-w`, so the first EOS count read nothing;
  - BSD `cmp -n` at a file's exact length reports "EOF" with exit 1, so the prefix checks were redone with `head`/`tail` hashes;
  - an edit to the gate test did not apply, so the base test failed and the first mutation tally was meaningless until it was fixed;
  - an edit to `mutate.sh` dropped a space and broke one mutation line.
- **Not decided:** mw-balp's licence and generator are recorded nowhere in the store or the repository. The mix's manifest and the sheet say so.

## What changed

- **The mixed TRAIN** in the store at `results/claude/one-model-end-to-end-b64l-d3mw-2026-11-07/train/`, built on the laptop with `scale-probe mix fraction=1/1`: D3's four roots whole, then `mw-balp/train` whole.
- **The prep sheet** [`one-model-end-to-end-b64l-d3mw-2026-11-07.md`](../../../../harness/prep/one-model-end-to-end-b64l-d3mw-2026-11-07.md):
  - **the recipe:** claude-301's (b64L) at the pin `ff5156bcc`, with only TRAIN changed, seeds 1 and 2;
  - **the bars:** claude-312's, unchanged (stopping at least 128 of 256 at both seeds, collapse under 64, the panel's 104 and 52);
  - **the held-out pair:** against (b64L-D3); it now decides between rows 2 and 3;
  - **the outcome map:** nine rows, with a ratio row and a no-stop row;
  - **new checks:** I-xc, and the window count in I-3;
  - **reviews:** FIT round 1, applied; the D67 pre-run review: **GO**, minors applied.
- **Run scripts** in `local/claude/claude-314/`, not committed, as for claude-312. They are claude-312's four scripts with:
  - TRAIN swapped to one folder fetch, and its I-3 block now checking three shas, the manifest included;
  - the window count recorded;
  - a new seal root;
  - new gate tests and two new mutations.
- **I-c and I-xc scripts** in `local/claude/claude-314/`: `ic-314.sh`, and the re-export in `xc/`.

## Measurements

**The mix:**

| part | documents | tokens | scored targets | scored EOS | EOS per million scored | usable windows |
|---|---:|---:|---:|---:|---:|---:|
| D3 | 18,016 | 27,968,113 | 27,416,554 | 20,729 | 756 | 153,253 |
| `mw-balp/train` | 60,000 | 7,512,797 | 2,425,925 | 212,968 | 87,788 | 68,218 |
| **the mix** | **78,016** | **35,480,910** | **29,842,479** | **233,697** | **7,831** | **221,471** |

- `tokens.u16` `1081853b3871ecb66e33fe1f212ae4415cccaba0fb123754949832ef3d2f223f`
- `response_mask.u8` `5bf91524fd65c05fc5284e3cbfc1d9d038d54a9eb2de12355b9bf9e9d86c1efa`
- `manifest.json` `802b60ba80784e50e68f329957ca92471cab59f53f4a36df16e91169da9bfc9f`

**How the counts were taken:**
- The EOS counts are positions with token 1 and mask 1, over the whole mix.
- mw-balp's window count is claude-313's trainer log ("usable TRAIN windows 68218"). D3's is 1,048,576 ÷ claude-301's final epoch 6.842124.

**Exposure at 16,384 steps × 64:**
- 4.73 passes over each part;
- mw-balp is 30.8% of the windows seen and 8.1% of the scored targets;
- about 1.06 scored EOS per window seen, against 0.135 for (b64L-D3) and 3.12 for (b64L-mw).

**I-c and I-xc,** with `served-reply-panel` at `be3d4cb0…` on `m2-fresh256`:

| Control | Artifact | Reference output | EOS stops | Rows equal |
|---|---|---|---:|---|
| I-c negative | claude-308 seed 1 (b64L-D3), `9a370566…` | claude-309 `d02ec432…` | 0 | yes |
| I-c positive | claude-313 seed 1 (b64L-mw), `1c9ae43c…` | claude-313 `5175c29b…` | 253 | yes |
| I-xc | claude-313's seal re-exported (`724a3b78…`, `--raw-gates --raw-transition`) | claude-313's artifact | | sha256 equal: `1c9ae43c3db8917766a06160075d0683bcfa116dcac42bdec24ac06bfbcb6ff0` |

## Checks run at the head

- **The mix against its parts:**
  - the D3-only rebuild reproduced D3's tokens `ed8f2537…` and mask `ec6e520c…`;
  - the mix's first 27,968,113 mask bytes, and its token body over that range, hash equal to D3's;
  - its last 7,512,797 mask bytes hash to `b5e7352d…`, and its token body over that range equals mw-balp's;
  - the boundary token is BOS.
- **The store copy:** `uor-store ls` lists the three files at the local sizes. The put went to a path that held nothing before (a 404 first).
- **The laptop gates,** on the scripts at the sheet's hashes:
  - `gt/test-gates.sh`: 29 ok, 0 FAIL;
  - `gt/mutate.sh`: 10 of 10 mutations caught, each producing only its own gate's FAIL lines;
  - `gt/test-seal.sh`: 7 ok;
  - `bash -n` ok on every script.
  - The D67 reviewer re-ran all three suites and got the same summaries.
- **The cancel probe,** re-run at the sheet: #3838 (deepseek-25) is still open and fine-tunes; no other open PR trains (b64L) from scratch on this mix.
- `loop report --check` on this REPORT.

## Skills used

None beyond the loop's own steps.

## Still open

- **The run:** the next unit, on two RTX 5090 pods under this sheet and the scripts at the recorded hashes. It costs about $2.4–3.0, with a budget notice at $5.60 that is not a cap.
- **The laptop reader for Step 6** is to be written in the run unit, from claude-313's `read-313.sh`. It needs `train=` the mix, the pair against (b64L-D3), and I-xc and I-c repeated first.
- **mw-balp's licence and generator** are not recorded anywhere.
- **`served-reply-panel`'s timer** still re-parses the artifact for every row (claude-311).
- **Recall** stays open whatever stopping reads.

## Plain summary

Our model never stops talking when trained on human text alone. Trained on a short-reply chat corpus alone, it stops, but it loses a lot on human text it has not seen. This unit builds the middle option: both corpora together, in full. It also fixes in advance how the next run will be judged:
- does the model still stop;
- are its replies still varied;
- does it keep its score on unseen human text.

The combined corpus is checked byte for byte against its two parts and stored. Two fresh reviews checked the plan and the scripts before any money is spent. One found that a broken export step could have looked like "the model never stops", so that step now has its own check, and it passed. The other found nothing that blocks the run. The run costs about $2.4–3.0 and is the next unit.
