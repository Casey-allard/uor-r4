<!-- Direction review of 30 September 2026. It was produced by the Claude lab's council workflow wf_0efd43f8-274: four evidence readers, three proposals, capability and cost critiques, and a judge. The owner approved the resulting D18 on 30 September; see DECISIONS.md D18 and #820. It is kept as written, except for one correction: the #1548 merge preceded its evidence run by about 90 s, not 17 s. -->

# Direction memo: 1 to 14 October 2026

*To Casey. From the Claude lab, acting as judge of the 30 September direction review: four evidence briefs, three proposals and two critique lenses. This memo is read-only work; nothing was run. Each claim is tagged* **measured**, **derived** *(my arithmetic on document figures)*, **proof** *or* **hypothesis**.

## Where the project stands

**Capability: the model cannot chat (measured).**
- S2 (7.15M parameters) answers 2 of 38 development requests.
- R1 is the best chat artifact (#1503). On development phrasings it scores:
  - Responsive 0.523;
  - Instruction 0.265;
  - Relation 0.013, with recall 0/64.
- The milestone needs 0.80 in each category.
- The 10 memory requests score 0/10 for S2, for R1 and for every AERM checkpoint with the store in the loop (#1492).
- On trained phrasings R1 reaches 0.93 / 0.65 / 0.59. It learned reply forms and closed-pool values, not copying.
- R1 failed its own NLL guard: 2.547954 to 2.635044, against a limit of 2.597954.

**Retrieval: exact storage works; learned retrieval from language does not.**
- Measured:
  - the D2 store scores 1.000 in distribution;
  - masking plus the prime sieve answers 264/273 held-out Updated queries in a synthetic world (one seed);
  - G v2 held-out key accuracy is 0.327 against 0.491 for softmax, with the guard failed.
- Hypothesis: the in-context copy mechanism is at fault. That is not isolated. R1-X and A1, the two experiments that would separate mechanism from phrasing, have never run.
- The A1 source (pointer head, flock reads, M-world v2.1, English probe) merged in #1548 as 302e0ad7. It is untrained.

**Runtime cost: no win exists anywhere (measured).**
- The D11 engine has no multiplier, divider or float in its audited step path, and its logits equal D10's exactly (difference 0).
- It is about 3–5× slower than D10 NEON: 5.40 ms/token against 1.11, self-reported.
- It reads 100% of the weights on every token (7,238,304).
- No valid J/token exists.
- The icosian snap adds 9.17% time.
- Derived: at the 256-token milestone context, the reads are about 4% of per-token work. So no retrieval or flock choice can move cost by more than a few percent. Cost is set by dense weight access, and D5 selected weight access is unbuilt.
- **None of the three proposals addresses the runtime-cost thesis.** That thesis remains a hypothesis.

**Track B: no converted student and no teacher-relative quality number.**
- #1518's 1e-4 parity gate failed on every host, including unmodified upstream Candle (1.432e-3).
- Proof: log-softmax is 2-Lipschitz in the max norm, so a 1e-4 logit gate certifies NLL to 2e-4 nats. That is 100× finer than the 0.02-nat parity tolerance, so the failure is a rounding-semantics gap, not a fidelity defect.
- B3 is a negative, but its encoder has a known defect.
- D17 has no operational test of what a transformer-free converted student is.

## What the last two days changed

1. **R1 sealed.** It showed the model learns form, not content.
2. **QAT Result C** kept served NLL (+0.0032) but not behaviour: greedy agreement was 5–6 of 58 against a gate of 29. Result B passed its gate by 0.001988, one seed.
3. **Real CI exposed a broken main.** PR checks had been executing nothing, and main carried about 110 compile errors. #1547 and #1549 fixed it.
4. **The A1 pieces are in place.** #1548 merged, the §8 panel was sealed (revision 3), and the M-world v2.1 stream was pinned. The decisive retrieval run needs no new source.
5. **You cut the labs to two and allowed self-review.**
6. **Process machinery consumed the period.** Heavy work was gated for about 13.4 h, and a 58-second test waited about 11.5 h. None of the plan's six "first decisive" experiments produced a number.

## Recommendation

Spend two weeks on one question: **can the native geometric stack copy a value it was just told, and does that survive D11 serving?** The base is proposal 3, amended by both critiques. It is the only proposal the cost lens did not refute, and it ends in a four-way decision rather than an open programme.

**Why this question.**
- The §8 conversations appear to fit inside 256 tokens. The longest is 137 words against a ≤140-word proxy. This is a hypothesis until you run the exact token count.
- If that holds, in-window retrieval is the binding capability. The durable store, the memory port and long context can wait.

**The work.**
1. **A1 as pre-registered.** The arms are P, P+ptr (Lorentz), T (the transformer control) and C (the Dot control). The gate is MQAR ≥0.9 at distances 16, 64 and 200, and open-relation recall ≥0.9. The parity rule decides Lorentz against Dot.
   - Amendment: the budget is a fixed step count, measured under the same concurrency. The 30-minute wall is only a stop.
   - Reason: this M1 has 4 performance and 4 efficiency cores. Running two arms at a time under a wall would give unequal steps, the failure that made G v2 uninformative.
2. **R1-X on the sealed R1 weights**, evaluation only.
3. **TruncatedPrefix data unlock.**
   - Currently 88.55% of chat-v0 responses are discarded.
   - Excluded: mid-response windows. That condition produced weak answers before (74.6% of R1d visits).
   - The gate adds non-regression on relation and abstention cells, because truncation can teach the model to answer without evidence.
4. **A staged 7M fit, only if A1 passes.**
   - It stops after 12M new positions unless development Instruction is ≥0.40 and pure retrieval is ≥0.5.
   - Reason: R1's own decision table already points to scale for Instruction (0.265 < 0.40), and a pointer cannot fix that.
5. **A D11 port of the trained pointer.**
   - Gated on served retrieval within 0.03 of the model's own float, compared with the training forward pass. It is not compared with D10, which refuses pointer models (verified in `geometric-stack.rs`).
   - Flock is recorded for parity only. It is not ported for serving at 256 tokens, because it saves no bytes there (derived).

**The four outcomes on 14 October.**
- **A, serve:** request §8.
- **B, scale or phrasing:** you decide on 30M, Metal coverage or teacher data.
- **C, read defect:** T retrieves and the stack arms do not.
- **D, no learned retrieval, T included:** the exact log plus the prime sieve, with no third round.

B is the most likely outcome. The plan is built so that B is cheap and informative.

**Ideas taken from the losing proposals.**
- From proposal 1: gate on served behaviour rather than token agreement, and add a simple eviction policy in the D11 session if outcome A arrives, since §8 requires a declared eviction policy.
- From proposal 2:
  - a *prospective* Track B host gate, frozen now and not run: dense NLL on the D3 split against the independent 3.8425 bits/token referee, plus an identity arm. That gives Track B a valid instrument when it resumes.
  - a teacher-ceiling measurement, only if R1-X says the limit is phrasing.

**Why not the others.**
- **Proposal 1:**
  - Its end gate (≥0.80 everywhere at 7M) contradicts R1's own decision table.
  - Its serving gate compares against D10, which refuses pointer models.
  - It makes QAT with the pointer mandatory, but `qat=true` refuses pointer models.
- **Proposal 2:**
  - A converted 135M student still reads about 100% of about 134.5M weights per token, roughly 0.1 s/token (derived), and meets neither D17 nor D5.
  - It breaks the #1017-bound instruments.
  - Its NLL gate cannot see recall.
  - Its 2.5e-3 threshold was set after observing 1.432e-3.

## Stop for two weeks

Everything below is parked, not killed, and has a re-entry condition in D18.
- Track B conversion: the #1518 chase, B0, B1, B2, harmonics and the B3 rerun.
- QAT and codec reruns on the old lineages.
- Kernel-speed and energy work, except the D15 J/token capture at export.
- The memory port, AERM, G v2 probes and the store integration.
- A2 beyond 256 tokens.
- New governance or runner machinery.
- Reply tallies from a single informal reader.

Separately, #1548 merged about 90 s before its exact-head test run finished. The post-merge check found no new failure. From now on, merge only after the exact-head run has completed.

## Risks

- **A1 may not show what we need.** A pass at 1.5M on synthetic M-world may not carry to English. A fail at this budget means "not learned here", not "impossible".
- **Serving may lose retrieval.** 4-bit serving can keep NLL and still lose behaviour, as Result C did.
- **Outcome D has only a direction.** The exact log plus prime sieve has no designed experiment behind it yet.
- **Single seeds and self-review** leave errors that one lab can make and not catch. Frozen gates, sealed roots and exact-head CI are the only guards.
- **The ledger figures are unreconciled:** 768.4M/780M, 782.5M/1,130M and 850.4M/1,130M ms. The plan needs at most about 22 h of laptop time (about 80M ms, derived), most of it conditional on A1 passing.
- **Staffing.** A quota outage in either lab stalls its lane.
