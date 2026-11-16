# A1 repo inventory (PARTIAL), 16 Nov 2026

Scope reached: only the mechanism board, docs/mechanisms/README.md at ef1f4a641 (lines 11-211; rows were cut at 260 characters, so some "best number" cells are partial).
I stopped early: this agent's context was already 166k after two listing calls, because the system prompt is large. Notebooks, lines, archive/CATALOG.md, docs/labs (224 entries) and the crates (32) are NOT covered.
Board counts (checked, README.md:13-15): ACTIVE 12, READY 5, PARKED 11.
"checked" means read on the board at the line given (:N = docs/mechanisms/README.md:N). The potential and failure-class columns are my judgement (hypothesis) unless a line is cited.

| Mechanism | Computes | Status | Best number (source) | Compliant? | Failure class | Potential for 1B goal |
|---|---|---|---|---|---|---|
| Discrete geometric learning | tries each legal rotation of one table entry, keeps it if the integer loss strictly drops | ACTIVE (:149) | pilot and K16 are dated failures (:149) | yes, native integer | configuration failures | HIGH: the only gradient-free native optimizer; coordinate search alone likely does not scale to 1B (hypothesis) |
| Prime-zeta codebook | fixed, unique starting address per token from primes and zeta phases | ACTIVE (:150) | the 25 Oct loss is marked a "configuration loss" (:150) | yes | CONFIGURATION, not mechanism (:150) | MED: a free injective init. FLAG: closed on one configuration |
| Address cells (A8) | places each token on a 16x16 board next to its usual predecessors | ACTIVE (:151) | starting admission 127 to 1,351; finite qualification 5/5 (:151) | yes | none | HIGH: cheap, and it moved a number |
| Routed read | a tree of addresses picks one earlier token | ACTIVE (:152) | 20 proposals in 16,404 updates (:152) | yes | weak learning signal (hypothesis) | MED |
| Flock read | rotates a few nearest earlier positions into its own frame and moves toward their consensus | ACTIVE (:153) | 25/25 controls (:153) | yes | none | MED-HIGH: a softmax-free mixing primitive |
| Relation read | follows links, e.g. the token after the earlier copy (induction) | ACTIVE (:154) | 9/9 controls; two of three trained runs stopped by admission loss (:154) | yes | coupling with admission, not the read itself (hypothesis) | HIGH: copying is the measured gap. FLAG |
| Admission | narrows 4,096 tokens to the opened "drawers"; tokens in closed drawers can never be chosen | ACTIVE (:155) | the first measured limit, on rung 0 (:155) | yes | recall ceiling | HIGH: the cheap output head, and its recall bounds every model |
| Rank-margin hinge | the right answer must beat its strongest rivals by a fixed gap | ACTIVE (:156) | the K16-R variant collapsed cell occupancy (:156) | yes, the softmax-free loss | one variant | HIGH: the loss for the whole model |
| Tokenizer 4,096 | BPE in and out | ACTIVE (:157) | a 16,384-piece trainer exists as tooling only (:157) | yes | n/a | HIGH. A 4,096 vocabulary may cap a 1B-equivalent model (hypothesis) |
| D11 integer serving (LUT) | serves with integer adds, shifts and tables; audited float-free | ACTIVE (:158) | owner ruling; integer multiply allowed since D80/D81 (:158) | yes | n/a | HIGH: the M1-laptop serving path |
| Hamilton gate G2 | turns the gate signal into a rotation of the value; no sigmoid | ACTIVE (:159) | the FBS feed-forward block (:159) | yes (in a float-trained trunk) | n/a | HIGH |
| SpiralCore/FBS routes | each block picks one stored block and moves it by exact E8 steps; transport carries position | ACTIVE (:160) | the trunk of the served model (:160) | yes in serving; training is float (D85 gap, hypothesis) | n/a | HIGH: the matmul replacement |
| Recurrence gates | fade rate from a power-of-two table, clamped input, state carried by rotation | READY (:166) | table error at most 5.5e-8, 18 CUDA controls (:166) | yes | never trained into a model | HIGH: cheap sequence carry. FLAG: built and unused |
| Object store D19 | keeps bytes exactly in a hashed tree | READY (:167) | the 384-item panel passes host controls (:167) | yes | n/a | MED: durable memory |
| Hopf kept-fibre map | a 3D shadow of a 4D rotation, with the lost angle kept apart | READY (:168) | the language test was never run (:168) | yes | NEVER TESTED | MED. FLAG: closed with no measurement |
| Spin-native weights (icosian) | each 4x4 weight block is one quaternion snapped to 120 icosians | READY (:169) | 1.5148 BPB, level with dense, but only through matmul (:169) | not as trained | the matmul path (instrument) | HIGH: about 16x weight compression that matched dense quality; needs a non-matmul apply. FLAG: cheap and moved a number |
| Ternary/XOR | weights -1/0/+1, popcount similarity | READY (:170) | export 7.416 vs 7.171 bits (:170); its Hamming read form is in the recipe (:49) | yes (popcount form) | a dense ternary sum is still dense (D25) | HIGH: the Pick read (#4045) is this form |
| Carrier credit | sends learning credit back to the geometric objects that carried it | PARKED (:176) | 437/512 fell to 0/512; the sampled step used softmax (:176) | no (softmax) | INSTRUMENT: a banned sampler | MED: revive with a hard argmax step |
| Legal construction | picks parameter changes from integer-representable values and guards correct answers | PARKED (:177) | D78's exact solve holds all 380 guards (:177) | yes | scale (hypothesis) | MED-HIGH: an exact native step |
| E8 orbit memory | an E8-addressed table replaces the feed-forward block | PARKED (:178) | the half-MLP form cost +0.033 BPB, inside its bar; learned keys beat E8 by 0.023 (:178) | yes | one configuration, and it PASSED its bar | HIGH. FLAG: it passed and was parked anyway; the E8 address memory (:67) is its successor |
| Exact memory | writes a fact once under a prime address and reads it back exactly | PARKED (:179) | the true-address oracle scored 80/80 (:179); trained forms 19/40 against a 27/40 bar | the oracle yes; some trained forms used softmax | INSTRUMENT (address learning or softmax), not the store | HIGH: turn memory is already in the recipe (:50) |
| Pointer key | copies by the slot's exact name | PARKED (:180) | hit rate 0.377 to 0.545, with a softmax pointer (:180) | no | INSTRUMENT: the softmax form | HIGH: revive on the Pick/XOR read. FLAG: moved a number |
| Read binding | a teacher marks the asked-for value and the read is rewarded for landing on it | PARKED (:181) | wrong-value errors fell from 47 to 30 of 120 over 3 seeds (:181) | no (softmax read) | INSTRUMENT | HIGH: an auxiliary target for the hard read. FLAG |
| Hopf sector routing | 8x8x8 bins of Hopf coordinates pick a route | PARKED (:182) | 7 of 512 sectors used (:182) | yes | configuration (the binning) | LOW |
| VSA | 4,096-bit codes bound by XOR, compared by Hamming distance | PARKED (:183) | BPB improved by 0.014 to 0.023 over 2 seeds, on the banned CPU learner (:183) | yes | INSTRUMENT: where the learner ran | HIGH. FLAG: cheap and moved a number |
| Geometric stack | the old 7M-214M layered quaternion-recurrence model | PARKED (:184) | 1.20-1.24 BPB against a 0.90 bar (:184) | no | used softmax and matmul | MED: parts reusable at scale |
| Native geometric engine | a Rust model with prime addresses and version chains | PARKED (:185) | 437/512 fell to 0/128 on fresh rows (:185) | yes | mechanism (no prose) | LOW |
| R4G1 graph | compiled a teacher model into integer LUTs | PARKED (:186) | no coherent text; needs a banned teacher (:186) | no | rule | LOW |
| E8 address memory | 32 learned rows per layer and token, read by the state's E8 cell | experiment (:67) | claude-346 | yes | open | HIGH |
| Pair experts | counts of TRAIN replies per pair of user-turn tokens | experiment (:68) | good replies rose from 137 to 170 of 239 (claude-367) | yes (counts) | none | HIGH: cheap, and it moved a number |
| Pair rows | trained rows addressed by token pairs | experiment (:69) | exact replies rose from 431 to 527 of 2,218 (claude-370) | yes | none | HIGH |

## Not covered (for a fresh agent)
- docs/research/notebook/*.md (12 files) and docs/research/lines/*.md (17).
- archive/CATALOG.md (83 lines) and its entries.
- docs/labs/* (224 entries, including about 30 cpu-training-pilot-* units from 4-9 Nov).
- The crates list (32 crates, including discrete, integer, lut, router, naf, native-vector, simd, recon-dag, graph-*, vision and cpu-train).
- The retirement reasons in PLAN.md and DECISIONS.md.

Every "best number" above should be re-checked against the linked REPORT.
