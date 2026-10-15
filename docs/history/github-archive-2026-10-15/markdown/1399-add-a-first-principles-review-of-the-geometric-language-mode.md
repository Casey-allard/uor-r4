# #1399 — Add a first-principles review of the geometric language model programme

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-25T23:04:44Z
- **Updated:** 2026-09-26T01:45:19Z
- **Closed:** 2026-09-26T01:45:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1399
- **Merged:** not merged
- **Merge commit:** 6a51274acc504ccdc369969ee7b4655c88a3a21c
- **Base / Head:** main / UOR-Foundation:claude/blissful-wozniak-girwwq
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

This PR adds three documents and their index rows in `docs/integration/README.md`:
- **`docs/integration/first-principles-review-2026-09-25.md`.** The owner asked for a stress test of the programme's computer science, mathematics and physics, and for a view on whether the project has drifted off course. The review also sets out the correct theory and path to a geometric language model running on an M1.
- **`docs/integration/low-energy-plan-2026-09-25.md`.** The owner then asked for the best plan with one goal: stop wasting energy on matmul, with the earlier constraints set aside.
- **`docs/integration/geometric-attention-2026-09-26.md`.** The owner objected that the plan kept conventional attention and asked for attention redesigned from geometric mechanisms. The note makes each mechanism precise, tests it and records prior art. The plan's attention row now points to it.

**All three are evidence, proposal or design documents, not decision records.** They change no D-entry, gate, artifact or active work card. §11 of the review asks the owner for decisions.

### How the review was produced
- **Eight specialist agents** worked in parallel: mathematics, physics/energy, CS/ML architecture, literature/novelty, programme audit, code verification, a quantization experiment, and a state-tracking experiment.
- **The lead reviewer** ran three side experiments and wrote the synthesis.
- **An adversarial red-team pass** spot-checked 31 numbers and citations. It found 3 critical, 11 major and 14 minor defects, and all were corrected before the first commit.
- **Labels.** Every claim is labelled Source, Measured, Literature, Derived or Hypothesis.

### Commits
1. `bf92155`: the review and its index row.
2. `1edeb6e`: the quantization experiment's final results. It adds the norm-rescaling recall result and its prior art (NEQ, ScaNN), and records the router paper's citation errors. It also corrects several §6.1 figures and prices the owed codebook comparison.
3. `30dfe0a`: the completed WikiText-2 time-mixing comparison (§6.3), with §8.4, §9.3 and Appendix B updated to match.
4. `72028d8`: the recommended low-energy plan and its index row.
5. `29698ce`: the geometric-attention design note, its measurements and index row, plus the plan's link to it.

### Key findings
- **Engineering.** The D8 path is correct and bit-exact:
  - integer 24/24, tokenizer 3/3 and training 68/68 tests pass on x86;
  - no correctness bug was found.
- **Drift.** The geometry has left the model:
  - The active crates contain no prime, zeta, Hopf, icosian, E8 or lattice code.
  - The only geometric element, a quaternion lane rotation, lost to a control that is itself a quaternion map: H(v)H(e0)x = v·x·v.
  - The router-era thesis never had a valid test.
- **The original idea.** It is sound, but already published:
  - TurboQuant, PolarQuant and QJL all keep the radius.
  - The 4-D quaternion KV quantizer with a quantized radius appeared as HQMQ (2026).
- **Keeping the radius does help ranking**, as NEQ (2019) and Google's ScaNN (2020) established. Rescaling reconstructions to the stored norm raised TurboQuant's recall@10 at zero bit cost:
  - by 0.017–0.031 on Gaussian data;
  - by 0.058–0.101 on data with a common offset.
- **The router paper misstates TurboQuant's title, findings and date** (`ANGULAR_MANIFOLD_ROUTING_PAPER.md`). The review asks for a correction before any external use.
- **Physics.**
  - Removing multipliers saves ≤0.4% of DRAM-bound energy.
  - The integer session's software multiply is 4.4× slower than hardware multiply on x86, with identical outputs.
  - No joule has ever been measured.
- **Geometry's theorem-backed niche is exact non-abelian state tracking** (A5, NC¹).
  - Snapped 2I lanes served as a 120-state integer automaton stayed exact to length 4,096, where float32 drifted.
  - Commutative models stayed at chance.
  - Two learned reflections match the capability, so the 2I edge is exact, cheap serving.
- **On real text (WikiText-2 bytes, 0.56M parameters, 2 seeds), rotation gives no advantage in the recurrence.**
  - Diagonal decay: 1.869/1.873 bits/byte.
  - Quaternion: 1.881/1.879.
  - Forcing every lane onto an exact codebook costs 0.08–0.10 even with straight-through training.
- **Geometric attention** (design note):
  - **Attention as physics:** softmax attention is a Boltzmann/heat-kernel distribution. Geodesic ranking is one inner product in any of the three geometries.
  - **Recall at 1,024 candidates:** dot, cosine, hyperbolic, 64-bit Hamming (XOR+popcount) and E8 block codes all reach 100%.
  - **Hierarchy:** hyperbolic keys reach 98.5% at 8 dims on tree retrieval, where dot product reaches 54.1% (68.2% at 16). Binary codes fail because they discard the radius, which encodes depth.
  - **Routing:** trained chunk advertisements admit 95–98% of true keys while scoring 160 of 1,024; untrained ones fail.
  - **Text hybrids** (bits/byte): hyperbolic 1.899, dot 1.902, cosine 1.906, Hamming 1.916, E8 1.940. So codes admit candidates and exact scores rank them.
  - **Verified:** E8 is the [8,4,4] Hamming code lifted by Construction A.
- **Recommended plan.** The plan's reasoning and design:
  - **Where the energy goes:** bytes moved (weights, output head, context cache) and instructions, not multipliers.
  - **The model:**
    - ternary weights with lookup-table kernels;
    - a mostly diagonal-recurrent backbone with geometric routed attention;
    - speculative decoding drafted from exact memory;
    - a 4-bit output head.
  - **Training:** distill in the cloud and serve on the M1.
  - **Phase 0:** measure J/token on the M1 against bitnet.cpp and llama.cpp.

### Owner decisions requested (§11 of the review)
1. Does "no sparse routing" conflict with D5?
2. Should D0-b's "no multiplier instruction" rule be judged by measured joules?
3. What levels of teacher use are allowed?
4. Should compute stay M1-only?
5. Should the process be lightened?

## Validation
- `python3 scripts/check_claim_wording.py` passes on every commit.
- Documentation only; no Rust changes, so no build is affected.
- The numbers come from scratch runs in the review sandbox, labelled Measured. No Python is committed, per the repository's no-Python-model policy; adopted mechanisms should be re-measured in Rust.
- The WikiText-2 runs are deterministic: a rerun of one arm reproduced every number bit for bit. Geometric-attention results are one seed each, and the note says so.
- The plan's energy figures are labelled Derived. Phase 0 replaces them with M1 measurements.

References #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_018jzJYgXbX8FmctqekthjLJ

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

