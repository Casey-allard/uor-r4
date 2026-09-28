# Geometric LM Lab — shared briefing (phase 2, 2026-09-26)

Read this first. Scratchpad root: `/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad`
(below: `$S`). Your report goes to `$S/lab/reports/<your-name>.md`; experiments to `$S/lab/exp/<your-name>/`.

## The owner's goal and instructions (latest, verbatim excerpts)
> "resume this autonomous ai research lab with adversarial agents, expert agents from every needed discipline, and a full
> autonomous project aimed to push this exploration to its final goal, a geometric language model that we can chat with"
> "continue autonomously with this very advanced frontier line of research - investigate from first principles with mathematical
> and computer science/machine language experts advising novel work (relax strict gates or we will never be able to do novel work)"
> Earlier: "all I want to do is stop wasting all of the energy on matmul"; "the goal is to redesign attention in our new
> mechanisms, not just copy all of the inefficiencies of transformers"; ideas the owner raised: geodesics, a "Hamiltonian heatmap"
> over a mutable Riemannian manifold, geometric triangulation, Hamming distances as OSPF-like routing where locations advertise
> their data, E8/icosian and quaternion spaces, semiprimes/CRT/Galois fields, "arcosh quantum tokens", recursion over embedded spaces.

The owner (Casey) is a mathematician/computer scientist, Rust-first, first-principles learner. Final target: a **geometric language
model you can chat with, running locally on an M1-class laptop**, with far less energy than dense float matmul serving.

## Owner decisions, 2026-09-26 (supersede anything below that conflicts)
1. **Weight transfer is approved.** A student may be initialised from an open transformer's weights (e.g. SmolLM2-135M/360M-Instruct,
   Apache-2.0), provided the served model performs no matmul at runtime: no floating point and no multiplier-based matrix products;
   <=4-bit/ternary maps executed as adds, shifts and table reads remain allowed (decision D0-b).
2. **Push the hyperbolic direction.** Hyperbolic (Lorentz) geometry is the lead mechanism to develop for attention/memory.
3. **External (offline) training is as important as the runtime.** Heavy offline training/conversion is in scope; the owner's M1 is
   the main large-run machine. Paid cloud compute still needs the owner's explicit go-ahead before any cost is incurred.

## Relaxed gates (owner instruction)
Novel, bold, first-principles proposals are wanted. You do not need proofs or multi-seed campaigns to propose something. You DO
need: (1) honest labels on every claim — **Measured** (you ran it: command + output), **Derived** (your math, shown),
**Literature** (you retrieved the source this session; give the URL/arXiv id), **Hypothesis**; (2) for each proposal, the
cheapest experiment that could falsify it. Never fabricate numbers or citations.

## Hard facts about the environment
- 4 CPU cores, 15 GB RAM, no GPU. The container restarts every few hours (processes die; files survive). Checkpoint long jobs.
- Disk: ~26 GB free, the owner is short on space. Keep your outputs small (≤200 MB per agent), delete temporary files, never
  download models/datasets >1 GB, and never create build caches outside `$S/target` (use `CARGO_TARGET_DIR=$S/target`).
- CPU etiquette: at most ONE thread per experiment (`OMP_NUM_THREADS=1 RAYON_NUM_THREADS=1`, JAX:
  `XLA_FLAGS=--xla_cpu_multi_thread_eigen=false intra_op_parallelism_threads=1`), each run ≤15 min unless the lead approves.
- Python: plain `python3` has no numpy. Use `PYTHONPATH=$S/pylib` (numpy 2.4, scipy) and `$S/leadlib` (jax 0.10, numpy) —
  e.g. `PYTHONPATH=$S/leadlib:$S/pylib python3 x.py`. Scratch Python is fine for experiments; project model code is Rust.
- Rust: `~/.cargo/bin/cargo` (1.97.1). Scratch cargo projects live under `$S/lab/exp/<you>/`, NOT in the repo. Examples of
  scratch crates that link the repo's training crate: `$S/c3/Cargo.toml` (copy its `[patch.crates-io]` block and Cargo.lock).
- Web research: load tools with ToolSearch, e.g. `select:mcp__alphaXiv__get_paper_content,mcp__alphaXiv__discover_papers`,
  `+Parallel_Search web`, `+Firecrawl scrape`, `+Exa web`. Direct curl to arxiv.org is blocked; PyPI/crates.io/HF may work.
- The repo `/home/user/uor-r4` (branch `claude/blissful-wozniak-girwwq`) is READ-ONLY for you. Never edit, commit, push, open
  PRs or post on GitHub or any external service. The lead integrates.

## What exists (read the ones your brief names; do not redo them)
Repo documents (on the branch):
- `docs/integration/first-principles-review-2026-09-25.md` — wave-1 synthesis: math, physics, architecture, literature, audit.
- `docs/integration/low-energy-plan-2026-09-25.md` — recommended low-energy plan (ternary + LUT kernels, recurrent backbone,
  distillation, speculative decoding from exact memory).
- `docs/integration/geometric-attention-2026-09-26.md` — cycle 1: the owner's mechanisms made precise and measured (MQAR
  recall/precision, Hamming/E8 codes, routing with advertisements, binding memory, tree retrieval, text hybrids).
- `docs/integration/hyperbolic-cycle2-2026-09-26.md` — hyperbolic keys on real code hierarchies; radius+direction quantizer.
- `docs/integration/hyperbolic-cycle3-2026-09-26.md` — the Lorentz read trained in the project's Rust D8 learner (4 seeds).
Wave-1 specialist reports: `$S/reports/{math,physics,arch,lit,quant,verify,audit,statetrack,geoattn_lit,redteam}.md`.

Key measured facts so far:
- **Current project model (D8 joint learner, Rust/Candle, integer serving exists):** ~1.7M params (≈1.05M tied embedding),
  vocab 4096 BPE on TinyStories, width 256, GRU-like recurrent state with quaternion/Householder lane transport, ONE soft read
  head (read width 64) over all prior positions in a 256 window with a learned NoRead slot, vocab/copy (pointer) mixture output.
  TinyStories dev NLL 2.11 (a 7.2M-param transformer reference: 1.57). Generations: varied but semantically unreliable.
- **Geometric attention (cycle 1, JAX scratch):** all scorings recall MQAR at 1K candidates at 64 dims; Hamming/E8 codes are
  good for routing/admission, worse as final scores in an LM; trained chunk advertisements route 97–98% at 1K candidates with
  ~16% of keys scored; a superposed binding memory's capacity is bounded by its width; hyperbolic scoring wins decisively on
  hierarchical retrieval at low width.
- **Hyperbolic (cycle 2):** on this repo's real code-scope tree, only hyperbolic keys learned the hierarchy (77.5% vs 0.4% for
  dot product, non-root queries, 16 dims); the owner's radius+direction quantizer keeps 87–98% of full precision at 36 bits/key.
  In small LMs hyperbolic attention helped a little (code −0.020 bits/byte; prose −0.005).
- **Rust D8 with a Lorentz read (cycle 3):** consistently but slightly better than Dot (3 of 4 code seeds, −0.018 nats/token;
  −0.037 on one WikiText seed); identical training cost. A training failure mode, **read dependence** (read mass → 1 while the
  read-free prediction degrades), hit sharp-start runs of every geometry; a flat initial read temperature prevented it.
  Next-token training gave keys only a weak radius↔scope-depth correlation.
- Prepared local data (4,096-token BPE, u16): `$S/c3/data/code/` (this repo's Rust, 7.0M/0.21M tokens) and
  `$S/c3/data/wiki/` (WikiText-2, 3.0M/0.32M); `lens.u16` gives bytes per token. The owner's TinyStories data is on their Mac.
- Committed Rust tool: `crates/uor-r4-training/examples/joint-read-geometry.rs` trains the D8 learner from scratch on u16 files
  (options: geometry, lorentz_start, read_dropout, checkpoint/resume). Binary: `$S/c3/bin_example`.

## Project policy you must respect in proposals (owner decisions; flag tensions, don't silently violate)
- Serving: no floating point, no multiplier in the declared numerical kernel; ≤4-bit integer/ternary linear maps executed as
  adds/shifts/table reads are allowed (decision D0-b); geometric routing and shared typed operators preferred; no transformer
  backbone at serving; no learned MoE/sparse expert gating without evidence.
- Training (offline): Rust, float, gradients and matmul allowed; an offline teacher model may be a training source/comparator
  but may not author serving responses. Initializing a student from a teacher's weights is APPROVED (owner decision above).
- Claims about capability must be scoped (the project distinguishes proof, measured behavior and hypothesis).

## Output format (all reports)
1. **Verdict** — 5–10 bullets.
2. **Findings / proposals** — numbered; each labeled; each proposal with expected gain, cost, and its cheapest falsifier.
3. **Recommended next experiments** — ranked, each runnable here (≤15 min, 1 core) or on the owner's M1 (state which).
4. **Open questions for the owner** (only real decisions).
≤3,000 words unless your brief says otherwise. Candid beats diplomatic.
