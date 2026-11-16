# A1b: research notebooks and lines, 16 Nov 2026 (origin/main ef1f4a641)

Read-only. "checked" = read at path:line (L = docs/research/lines/, N = docs/research/notebook/). "hyp" = my inference.
Coverage: all 17 line pages' Key-results tables and dated tails; float-training.md all row titles + 13 rows in full; cpu-training, m1-discrete-attention, m2-grounded-reply, m1-prime-zeta notebooks by last rows only. NOT read: data-tokenizer, harness-governance, legal-construction, m3, m4, m6 notebooks in detail; lines compute-ops, eval-panels, data-tokenizer, m5, m7, spiralcore, harness (no model numbers expected).

## Q1. Best measured result per line

| Line | Best number | Source |
|---|---|---|
| m1-discrete-attention (19-table joint-pointer Hamilton UORLUT01 trunk) | next-token top-1 DEV 7,002/7,001 of 14,287 (49%) vs T2 6,300, unigram 1,226; fresh V2 7,215/7,228 of 14,479; held-out phrasing p-f 7,850/7,872 of 15,953 vs T2 7,375; M1 laptop decode 10,876/10,566 tok/s | checked L/m1-discrete-attention.md:68-71, N/m1-discrete-attention.md:92 |
| m1-float-rung0 / FBS trunk (float-trained, integer-served) | ~60% held-out next word vs 44% word pairs, 49% count model (claude-142); quaternion weights 1.4959 vs dense 1.5136 BPB | checked L/m1-float-rung0.md:59, :46 |
| float-training notebook, human text (FBS, width 256, D3 = 28M tokens wiki+OASST2) | (b64L) 16,384 steps x batch 64 (~201M positions): panel top-1 331,698/342,250 of 1,460,697 (22.7/23.4%), ahead of T2 by +75k/+86k (band 10.5k) | checked N/float-training.md:115 |
| Reply panels (FBS + memory add-ons) | pair rows: exact replies 431->527 and 413->521 of 2,218; E8 address memory 188/225 -> 218/254 of 1,108 | checked N/float-training.md:31, :42 |
| m2-exact-memory | 437/512 exposed dev, then 0/128 fresh (bar 52/128); read binding wrong-value 47->30/120 | checked L/m2-exact-memory.md:44-45 |
| m3-durable-memory | store works (380 B for 7,776 raw; conformance PASS); model use 16/20 dev with value supplied | checked L/m3-durable-memory.md:41 |
| m4-serving | FBS trunk served 4,491-4,829 tok/s on M1 with byte-identical replies; old stack 1.20-1.24 BPB served vs 0.90 target | checked L/m4-serving.md:44, :57-62 |
| geometric-stack-pretraining | 1.998 vs 2.011 nats (transformer control) on repo code; served 1.20-1.24 BPB; but softmax/matmul | checked L/geometric-stack-pretraining.md:43 |
| native-geometric-sept | 35/180 uncopied correct dev answers; A3 complete answers 0/12 | checked L/native-geometric-sept.md:48 |
| r4g1-transformerless | greedy = teacher on 29.52% of 72,130 positions; 2.25 bits/target served | checked L/r4g1-transformerless.md:42,45 |
| geometric-intelligence-aug | mixer +4.44% over permuted control (gate 5%); manifold CE 7.71 vs 4.48 | checked L/geometric-intelligence-aug.md |
| m1-prime-zeta | construction only: 4,096/4,096 unique; 25 Oct zero-update admission 127/3,847 vs nulls 178-188 | checked L/m1-prime-zeta.md (Where it stands / Current state) |
| cpu-training (D84) | 256-step CPU rank-margin pilot: no gain (top-1 278 stayed 278, needed +43) | checked N/cpu-training.md:150 |

## Q2. The three model lines, same measure where possible

- A, float FBS trunk: ~63,744 trainable values plus a 4,096x256 word table (~94% of the model) (checked L/m1-float-rung0.md:59). Best language: 23.4% panel top-1 on D3 human text (N/float-training.md:115); width 256 beats 128 by +125k rows, 512 does not fit the card (N:87). Replies: 527/2,218 exact (N:31); fresh m2 replies 0/256, no EOS (N:105). No BPB measured anywhere in the notebooks; the only BPB row says the M4 rank scorer's floor is 0.7188 and 0.90 is out of reach once >1.892% of targets fall outside top-8 (N/float-training.md:59).
- B, native Q31: only parity and serving contracts. Codex's 48-byte CUDA-trained Q31 state: loss 17.18e9 -> 13.37e9, "no language claim" (N/float-training.md:26). First native training of the Q31 read's gates: negative, outcome 6 (N:15). No quality number on DEV or panels.
- C, discrete native (crates/uor-r4-discrete): claude-404/406/407/408 are one native step, scoring speed (4.73 s -> 1.98 ms per coordinate) and the Pick read's exactness (0 mismatches) (N/m1-discrete-attention.md:93-95). No trained quality number yet. hyp: the M1 19-table trunk (49% DEV top-1) is the discrete line's older served artifact, but it was trained in "Path A" (N:87), so it is not evidence for crates/uor-r4-discrete training.
- Common measure: only A and the M1 trunk have next-token top-1; DEV 14,287 for the M1 trunk (49%) vs a different 1.46M-row human-text panel for A (23%). Not comparable as-is (hyp: DEV is templated dialogue, much easier).

## Q3. Cheap retries (one configuration, or instrument fault)

- Human-text flat loss at --objective cell: classified as objective, not mechanism; (m1)/(near) objectives learn (N/float-training.md:118-119). Already retried.
- Width 1,024 runs: two diagnostic stops + one stop by decision, "closes nothing about width" (N:112-114). Memory-limited; retry with micro-batching.
- (b64L) at 32,768 steps on D3: stopped before launch, "closes nothing about steps" (N:109). Cheapest open lever: tokens at batch 64 were "ahead" (N:115).
- D4 (74M tokens): seed 1 DEV collapsed after step 1,024 and still rising; "inconclusive, configuration only" (N:110-111). Retry with a third seed or longer.
- DeepSeek aligned-corpus copy smoke: underfit/instrument, 1,024 steps (N:52).
- Point key integer: instrument fault, form C untested (N:156).
- Batch 64 OOM: instrument fault, later fixed by accumulation (N:116, :115).
- Prime-zeta: configuration loss; trained-in test vs its two nulls never run (L/m1-prime-zeta.md, Current state).
- K16-A 65/463: dated configuration negative (L/m1-discrete-attention.md:72).
- CPU rank-margin pilot 256 steps (N/cpu-training.md:150): tiny budget.
- Q31 gate training: bar 4b only; "rule or states' reachability still open" (N/float-training.md:15).

## Q4. Language at scale (>10M tokens, held-out falling with data)

Yes, one line, weakly: FBS trunk A on D3 = 27,968,113 tokens (N/float-training.md:111). D1->D3 data lever "ahead" at 256 (N:119, :117: d -20,217/-17,386 against D1, band 2,831), and tokens at batch 64 "ahead" (N:115); width 128->256 doubles hits (N:87). But: the measure is no-memory panel top-1, not BPB; the unseen sub-panel is "ahead" for b64L only; D4 (74M) did not beat D3 (N:110, mixed); and the cell objective's loss stayed flat at ~1.0035 on human text (N:119). No held-out BPB curve exists in any notebook (checked by grep: BPB appears only at N/float-training.md:59, N/m3:30, N/m4:110). Lines B and C have never trained on language data.
