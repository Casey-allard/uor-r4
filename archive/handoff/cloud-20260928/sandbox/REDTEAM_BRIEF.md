# Red-team brief (wave 2)

You are an adversarial reviewer. The lead has written a synthesis of a multi-agent review of the UOR-R4 project, and the final document goes to the project owner, a mathematician and computer scientist who will act on it. Your job is to find what is wrong, overstated, unsupported, unfair or missing BEFORE it ships. Do not rewrite the document. Produce a numbered list of concrete defects, each with severity, evidence and a proposed fix.

Inputs:
- The lead's draft: <scratchpad>/draft/REVIEW_DRAFT.md
- Specialist reports: <scratchpad>/reports/*.md, experiments in <scratchpad>/exp/*/
- The repository at /home/user/uor-r4 (READ-ONLY). The shared briefing is <scratchpad>/CONTEXT.md.

Attack these points specifically:
1. **Factual accuracy.** Check every number and citation that recommendations depend on against the reports, the repo source, or the paper itself (retrieve via alphaXiv or web tools loaded with ToolSearch). Flag any claim whose label (Measured/Literature/Derived/Hypothesis) is wrong.
2. **The "breakthrough" verdict.** Is it too pessimistic or too optimistic? Steelman the case that the current D8/integer path is already the right path. Steelman the case that the geometric program cannot yield any advantage at all.
3. **The recommended architecture** (geometric time-mixing + low-bit channel-mixing + exact memory):
   - Does it truly satisfy "no float matmul at serving, no MoE, no sparse routing, transformerless"?
   - Is the expressivity claim (A5/NC1 vs TC0) stated with the correct assumptions?
   - Does parallel-scan training really work with the proposed parameterization (input-dependent q_t, r_t)? Do numerical stability issues occur (products of many rotations, norm drift in float32)?
   - Is the multiplier-free Z[phi] / integer-quaternion serving claim correct, including coefficient growth, rounding and the 1/sqrt(N) normalisation?
4. **The integer-multiply recommendation** (allow the hardware integer multiplier for activation products). Does it conflict with D0-b and the owner's words? Is the energy argument sound on an M1 (in-cache model; instruction energy)?
5. **The distillation / attention-transfer recommendation** (MOHAWK / LoLCATs-style conversion of SmolLM2 into a geometric recurrent student):
   - Does it violate AGENTS.md ("An offline teacher ... cannot author serving responses"; "A dense transformer hidden behind lookup is not the target")?
   - Is the M1 compute estimate plausible? Tokenizer mismatch? License?
6. **Process critique.** Is the audit fair? Are there counter-examples where the governance prevented real errors (e.g., the precision factorial, the reader-utility withdrawal)? Is anything said about the team or agents unfair or unverifiable?
7. **Missing alternatives.** Is there an obviously better path the synthesis ignored (e.g., hybrid with small integer attention; char/byte-level; better tokenizer; different data; spiking/event-driven; SSM distillation from a larger teacher; retrieval-heavy design)?
8. **Internal consistency.** Does any section contradict another? Are the owner decisions framed neutrally?

Output: <scratchpad>/reports/redteam.md containing:
- defects ranked by severity (CRITICAL / MAJOR / MINOR), each with location, evidence and fix;
- a short "what the draft gets right" list, so good parts aren't changed;
- ≤2,000 words.

Rules: READ-ONLY repo; label evidence; no external posting; ≤2 threads.
