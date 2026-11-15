# FACTS: the status facts the 14 Nov 2026 pages state, claude-382

Read at base `d5eb4a556`. Each row is a measurement or record with its source. A page in this unit states a number only if it is here. The labels: **M** measurement, **R** record (a merge, a decision, a publish).

## Project status

| # | Fact | Kind | Source |
|---|---|---|---|
| S1 | Pre-alpha; 1 of 8 milestones complete (M1, closed 30 Oct 2026); no model holds a useful conversation | R | [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029), [PLAN](../../../../../PLAN.md) "Goal" |
| S2 | M1: DEV 7,002 / 7,001 of 14,287 (unigram 1,226, T2 6,300); Fresh V2 7,215 / 7,228 of 14,479 (T2 6,582); p-f 7,850 / 7,872 of 15,953 (T2 7,375); Apple M1 decode 10,876 / 10,566 tok/s; 62.5% of p-f reference replies are verbatim TRAIN replies | M | [M1 close](../../../../labs/m1-close-saved-model/README.md), [Fresh V2](../../../../labs/m1-fresh-panel-transfer/README.md), [claude-211](../../m1-heldout-phrasing-transfer/claude-211/REPORT.md) |
| S3 | D84 (1 Nov): an owner-authorized CPU training research program, in isolated modules; GPU stays the qualification authority | R | [PLAN](../../../../../PLAN.md) owner directions, 2026-11-01; [CPU notebook](../../../../research/notebook/cpu-training.md) |
| S4 | D85 (13 Nov): the default route is one persistent integer/fixed-point geometric state, trained and served by the same typed operators with bit-identical replay; float master weights and export gaps become reference work | R | [PLAN](../../../../../PLAN.md) "Native-space cycle" and owner directions, 2026-11-13 |

## The model line since 31 Oct (the FBS trunk, `one-model-end-to-end`, #3407)

| # | Fact | Kind | Source |
|---|---|---|---|
| F1 | Human text, held-out panel of 1,460,697 rows: batch 64 with twice the training text gets 331,698 and 342,250 correct next tokens at two seeds against T2's 256,660, ahead of T2 at both seeds (6 Nov) | M | [claude-301](../../one-model-end-to-end/claude-301/REPORT.md) |
| F2 | Width 256 gets about 125,000 more rows right than width 128 on that panel; width 512 did not fit the GPU's memory (8 Nov) | M | [float-training notebook](../../../../research/notebook/float-training.md), claude-317 |
| F3 | Since 10 Nov the work is recall in controlled conversations. claude-358's three `p` models (p-s1 to p-s3) state a value told earlier on 314, 320 and 325 of 578 recall questions (13 Nov) | M | [claude-358](../../one-model-end-to-end/claude-358/REPORT.md) |
| F4 | claude-358 p-s2 retrained, served speed, Apple M1, `fbs-toks --read`: decode 1,567 to 1,636 tok/s; context prefill with the read 4,309 to 4,499 tok/s | M | [claude-358](../../one-model-end-to-end/claude-358/REPORT.md), "Speed" |
| F5 | E8 address memory, first training run: exact replies of 1,108 rose from 188 to 218 (seed 2) and 225 to 254 (seed 3), against a bar of a mean gain of 30: not called. Served in integers at three seeds (claude-366): not called, M = +5.3 against its bar | M | [claude-346](../../one-model-end-to-end/claude-346/REPORT.md), [claude-366](../../one-model-end-to-end/claude-366/REPORT.md) |
| F6 | Pair experts (counted from TRAIN, not trained): good everyday replies of 239 rose from 137, 135, 134 to 170, 171, 170 held out. Opt-in in `uor-chat --pair-experts`; when the experts agree, the reply is a stored TRAIN reply, not the model's | M | [claude-367](../../one-model-end-to-end/claude-367/REPORT.md); [mechanism board](../../../../mechanisms/README.md#results-since-31-oct-with-no-board-row-yet) |
| F7 | Pair rows (trained, addressed by the turn's token pairs): exact replies of 2,218 rose from 431 to 527 (seed 2) and 413 to 521 (seed 3); most of the gain is the turn's own words carried into the reply. Two weighings did not add (claude-373, claude-374). The export refuses pair rows, so they are not served | M | [claude-370](../../one-model-end-to-end/claude-370/REPORT.md), [claude-373](../../one-model-end-to-end/claude-373/REPORT.md), [claude-374](../../one-model-end-to-end/claude-374/REPORT.md) |
| F8 | V0, the pure-vector baseline (14 Nov): claude-358's three models served on the laptop over claude-374's panels (2,218 rows). Exact replies: 171, 173 and 211 with the exact turn memory dropped, against 377, 380 and 441 as exported. Almost every fact the model gets right comes from the turn memory's table. Served tokens/s, Apple M1, shared laptop: about 1,420 to 1,460 without the memory and about 1,600 with it, where the load held | M | [claude-376](../../one-model-end-to-end/claude-376/REPORT.md) |
| F9 | Training carries no state from one window to the next; the current recipe reads with four Hamming reads and an exact turn memory, not the Route, Flock or Relation reads | R | [mechanism board](../../../../mechanisms/README.md#the-current-recipe-claude-358s-p-recipe) (claude-375) |
| F10 | In progress, not merged at this base: V1, the state carried across training windows (claude-380, PR #3979); V2(a), three vector-native reads behind flags (claude-381, PR #3978) | R | open PRs [#3979](https://github.com/UOR-Foundation/uor-r4/pull/3979), [#3978](https://github.com/UOR-Foundation/uor-r4/pull/3978) |

## The native-space line (D85)

| # | Fact | Kind | Source |
|---|---|---|---|
| N1 | codex trained one Q31 vector state on CUDA: a finite 64-step test on an RTX 4090, the CPU and CUDA transitions equal after every step, aggregate L1 loss 17,179,869,184 to 13,373,147,056. A contract test, not language-model training | M | [codex-5](../../native-space-state-contract/codex-5/REPORT.md), PR #3968 |
| N2 | The discrete path's parity contract: CUDA training, CPU training and integer serving compared on the same fixture bytes, two fixtures (Route, Flock), all 39 prefixes | M | [claude-371](../../one-model-end-to-end/claude-371/PARITY.md), [claude-372](../../one-model-end-to-end/claude-372/REPORT.md), [codex-5 parity](../../one-model-end-to-end/codex-5/REPORT.md), [codex-6](../../one-model-end-to-end/codex-6/REPORT.md) |
| N3 | The integer crate restores a trained 48-byte Q31 state without numerical conversion, saves it losslessly and runs the same shared transition that training uses; a 64-step trained fixture matches as raw words. No token generation, language result or throughput is claimed (PR #3977, merged 14 Nov 23:54Z, after this sheet's base) | M | [codex-8](../../native-space-state-contract/codex-8/REPORT.md), [codex-9](../../native-space-state-contract/codex-9/REPORT.md) |

## Products and the hosted pages

| # | Fact | Kind | Source |
|---|---|---|---|
| P1 | The live Studio is gh-pages `4a435ab8a`, published 9 Nov 2026 from `4160a5347` with the typed publisher: the neutral UOR Harness workbench, an 807,378-byte M1-capable engine bundle, and a catalog holding only the synthetic demo | R | [web client](../../../../products/web-client.md), [kimi-17](../../studio-ide-harness/kimi-17/REPORT.md) |
| P2 | #3915 (kimi-1, merged 11 Nov) added a browser-owned task workspace to the Studio source; it was not on the live page before this unit's publish | R | PR [#3915](https://github.com/UOR-Foundation/uor-r4/pull/3915) |
| P3 | `uor-chat` has an OpenAI-compatible loopback (2 Nov) | R | PR #3606, [PLAN](../../../../../PLAN.md) owner directions, 2026-11-02 |
| P4 | The live book was last published to `gh-pages` on 20 Oct; `docs-site.yml` publishes every push to `gh-pages-preview` only, and a live publish needs a dispatch with `target=gh-pages` | R | this unit's ORDER; `.github/workflows/docs-site.yml` |
| P5 | No product ships; the web and desktop clients are research explorers | R | [products](../../../../products/README.md) |
| P6 | The live `/studio/` page reads "Pre-alpha · M1 complete · 1 of 8", the same text as `web/studio/index.html` at the base | R | `curl https://uor-foundation.github.io/uor-r4/studio/`, 15 Nov ~00:05Z |
