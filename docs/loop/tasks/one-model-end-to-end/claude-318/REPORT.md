# REPORT: one-model-end-to-end, claude-318

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Laptop only; no GPU, no spend.

## What did not work, first

- **No result yet.** This unit registers a run; it trains nothing. Whether (b64L) on D3 + `mw-balp/train` × 3 stops at both seeds, keeps its stopped replies varied, and keeps D3's held-out text within the cost ceiling is read by the next unit.
- **The first draft of the sheet had two majors that a run could have misread.** A fresh FIT found them (FIT WITH CHANGES, round 1, no blockers):
  - **Row 5 ignored collapse.** A run that stopped less than × 1 and collapsed would have been offered a higher ratio. Rows 5 and 6 now read collapse, and row 6 is split into 6a and 6b.
  - **A drop in stopping was classed only below 8 stops.** A seed that fell from 177 to, say, 60 would have read as "stopping not met", with no classification first. Row 5 now classifies any run where a seed falls below × 1's 122 (the exact-head review widened it from "both seeds", so a one-seed drop can no longer reach row 6a).
  - **Four minors,** all applied: I-3's window count now gates the full run; collapse has a near-bar band; the descriptive pair against (b64L-d3mw) names its confound; and "behind" is defined for a `mixed` pair verdict.
- **The D67 pre-run review of the scripts was NO-GO as written.** It found one major and three minors, all now fixed and tested:
  - **M1, unbounded billing:** the watcher renewed the lease every 25 minutes forever, stalled or not. A hung pod could then never be taken by the pod reaper, which deletes a pod 20 minutes after its lease lapses. Now no renewal is made while a STALL notice stands, and a changed line clears the notice. claude-317's watcher carried the same code on a live pod (`vast-59652743`); it was patched the same way and restarted at 03:05Z.
  - **m1:** the chain had no signal trap, so a TERM could end it before the seal ran. It now has a no-op trap.
  - **m2:** the job's sealing exit reset its traps, so a second signal could kill it mid-seal. It now ignores TERM, INT and HUP while it seals.
  - **m3:** the stall notice was never cleared, so after one stall renewals would have stayed paused. A changed line now clears it.
  - **Round 2 was GO,** with one minor: the m2 fix had no test. The watcher test now runs the job's own `end()` under two TERMs and expects EXIT 0 and DONE; both mutations of the fix are caught.
  - **One nit is kept:** a signal in the job's first few seconds, before its trap is set, leaves no EXIT. The chain then ends unsealed, and the watcher keeps the pod up for the session.
- **The claim's number collided.** `loop claim` named this unit claude-317, which is the claude lab's open scale-probe unit (#3472, PR #3842). The claim reads merged folders only. The folder and title were renamed to claude-318 before any other work, and the ORDER says so.
- **Laptop slips on the way, all caught before review:**
  - BSD `sed` has no `\b`, so the first rename missed "d3mw" in prose; then an over-broad rename changed a claude-314 attribution, and was reverted;
  - zsh reads `======` as an `=` expansion;
  - the first watcher test hung: a mutant run deleted its own input folder. Mutants now use their own folder and a bounded wait;
  - `uor-store put` printed "0 uploaded" (dedup); the copy was fetched back and its hashes matched.
- **Not decided:** mw-balp's licence and generator are still recorded nowhere.

## What changed

- **The mixed TRAIN** in the store at `results/claude/one-model-end-to-end-b64l-d3mw3-2026-11-08/train/`, built on the laptop with `scale-probe mix fraction=1/1`: D3's four roots whole, then `mw-balp/train` whole three times under the root names `mw1`, `mw2`, `mw3`.
- **The prep sheet** [`one-model-end-to-end-b64l-d3mw3-2026-11-08.md`](../../../../harness/prep/one-model-end-to-end-b64l-d3mw3-2026-11-08.md):
  - **the recipe:** claude-301's (b64L) at the pin `ff5156bcc`, with only TRAIN changed, seeds 1 and 2;
  - **the bars:** claude-312's, unchanged;
  - **the deciding pair** against (b64L-D3), and a **cost ceiling** of three times (b64L-d3mw)'s mean d (−58,212). It is a decision rule, not a bar, and splits row 3 into 3a and 3b;
  - **a descriptive pair** against (b64L-d3mw), and the role-header counts;
  - **the outcome map,** first match decides, with rows 5, 6a and 6b reading collapse;
  - **I-3's window count** of 357,907 now gates the full run;
  - **reviews:** FIT round 1, applied; the D67 pre-run review, round 1 NO-GO, fixes applied, round 2 **GO**, its one minor applied.
- **Run scripts** in `local/claude/claude-318/`, not committed, as for claude-314. They are claude-314's four scripts, with:
  - TRAIN swapped (the three shas, the folder fetch, the config check, the window count);
  - claude-317's job signal trap;
  - the window gate after I-f;
  - the D67 fixes above;
  - a new seal root, and new gate tests and mutations.
- **The laptop reader** `read-318.sh`, from claude-315's: `train=` the mix, the deciding and descriptive pairs, the cost ceiling, and the role-header counts.

## Measurements

**The mix,** from `scale-probe mix` and claude-314's per-part counts:

| part | documents | tokens | scored targets | scored EOS | EOS per million scored | usable windows |
|---|---:|---:|---:|---:|---:|---:|
| D3 | 18,016 | 27,968,113 | 27,416,554 | 20,729 | 756 | 153,253 |
| `mw-balp/train`, each copy | 60,000 | 7,512,797 | 2,425,925 | 212,968 | 87,788 | 68,218 |
| **the mix** | **198,016** | **50,506,504** | **34,694,329** | **659,633** | **19,013** | **357,907** |

- `tokens.u16` `3cf9ebe2abdf6e6a252df545192af27aea8ec2ac419484aac8932158b5a57b67`
- `response_mask.u8` `a245fef488d4bfb0e32a447f14a6fb2543caa28a6bb8252163a8523891c22dff`
- `manifest.json` `788b206cd57c86df71de1cc10d9f76db8b2715b0ed0757b4de846a6f78a29e3b`

**Exposure at 16,384 steps × 64:**
- 2.93 passes over each part; mw-balp's content is seen 8.79 times;
- mw-balp is 57.2% of the windows seen and 21.0% of the scored targets;
- about 1.84 scored EOS per window seen, between (b64L-d3mw)'s 1.06 and (b64L-mw)'s 3.12.

**I-c and I-xc,** with `served-reply-panel` at `be3d4cb0…` on `m2-fresh256`:

| Control | Artifact | Reference output | EOS stops | Rows equal |
|---|---|---|---:|---|
| I-c negative | claude-308 seed 1 (b64L-D3), `9a370566…` | claude-309 `d02ec432…` | 0 | yes |
| I-c positive | claude-313 seed 1 (b64L-mw), `1c9ae43c…` | claude-313 `5175c29b…` | 253 | yes |
| I-xc | claude-313's seal re-exported (`724a3b78…`, `--raw-gates --raw-transition`) | claude-313's artifact | | sha256 equal: `1c9ae43c3db8917766a06160075d0683bcfa116dcac42bdec24ac06bfbcb6ff0` |

## Checks run at the head

- **The mix against its parts,** re-run on 8 Nov against claude-314's `d3mw`:
  - the mix's first 35,480,910 mask bytes hash to `d3mw`'s mask `5bf91524…`, and its first 35,480,910 body tokens to `d3mw`'s body;
  - each of the three mw-balp segments (7,512,797 mask bytes, 15,025,594 body bytes) hashes equal to `d3mw`'s mw-balp segment (mask `b5e7352d…`);
  - `scale-probe mix` over claude-314's five roots rebuilt `d3mw` byte for byte (the cancel probe).
- **The store copy:** fetched back; the three hashes matched.
- **The laptop gates,** on the scripts at the hashes in `local/claude/claude-318/script-hashes.txt`:
  - `gt/test-gates.sh`: 44 ok, 0 FAIL;
  - `gt/mutate.sh`: 15 of 15 mutations caught;
  - `gt/test-seal.sh`: 7 ok;
  - `gt/test-watch.sh`: 8 ok, and 5 of 5 mutations caught (the renewal pause, the notice clear, the chain trap, and two of the job's sealing ignore);
  - the round-2 reviewer re-ran all four suites and got exit 0 on each;
  - `bash -n` ok on every script.
- **The cancel probe:** no open PR other than #3846 trains (b64L) from scratch on a repeated-mw-balp mix. deepseek-28 (#3845) fine-tunes.
- `loop report --check` on this REPORT.

## Skills used

None beyond the loop's own steps.

## Still open

- **The run:** the next unit, on two RTX 5090 pods under this sheet and the scripts at the recorded hashes. About $3.3, or $4.1 at the $0.70/h ceiling; the budget notice at $5.60 is not a cap.
- **claude-317 (#3842)** is still training on pod C; its REPORT is separate.
- **mw-balp's licence and generator** are not recorded anywhere.
- **The watcher's notice files** persist across a watcher restart, so a restarted watcher inherits a standing stall notice. Minor; a changed line clears it.

## Plain summary

Trained on human text alone, our model never stops talking. Trained on human text plus one copy of a short-reply chat corpus, it stops at one seed (177 of 256) and just misses at the other (122). This unit builds the next step: the same human text with three copies of the chat corpus, and fixes in advance how the run will be judged. Does it stop at both seeds? Are the replies still varied? And how much of its score on unseen human text does it give up, against a ceiling set now?

The new corpus is checked byte for byte against its parts and stored. One review found two holes in the plan's outcome map, now closed. A second found a billing hole in the pod watcher: a stuck pod could have billed forever. That is fixed here and in the watcher on the pod running now. The run costs about $3.3 and is the next unit.
