# Shared briefing for the UOR-R4 deep review team (read this first)

Date: 2026-09-25. Repo: `/home/user/uor-r4` (shallow clone of UOR-Foundation/uor-r4, ~1,400 PRs of history on GitHub).
Scratchpad (write ALL your outputs here): `/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad`
Your report goes to `<scratchpad>/reports/<your-agent-name>.md`. Experiments go under `<scratchpad>/exp/<your-agent-name>/`.

## Hard rules for every agent
- The repo is READ-ONLY for you. Do not edit, commit, push, open PRs, or comment on GitHub. Do not post anything to any external service.
- Do not fabricate citations, numbers or file contents. Every claim in your report must carry one label:
  `[SOURCE file:line]` (verified by reading repo source/docs), `[MEASURED]` (you ran it; give command + output),
  `[LITERATURE url]` (you actually retrieved the paper/page this session), `[DERIVED]` (your own math, shown),
  `[HYPOTHESIS]` (plausible, unverified). If you could not verify something, say so plainly.
- CPU budget: the machine has 4 cores / 15 GB RAM shared by ~8 agents. Use at most 2 threads (`OMP_NUM_THREADS=1`, `cargo -j2`),
  keep any single experiment under ~10 minutes, and don't build the whole workspace unless your brief says so.
- Tools: many tools are *deferred* and must be loaded before use with ToolSearch, e.g.
  `ToolSearch("select:WebSearch,WebFetch")`, `ToolSearch("alphaxiv")`, `ToolSearch("+Parallel_Search web")`,
  `ToolSearch("+Exa web")`, `ToolSearch("+Firecrawl scrape")`, `ToolSearch("+Wolfram")`, `ToolSearch("+Scite")`, `ToolSearch("+github issue")`.
  Direct `curl` to arxiv.org is blocked by the proxy; use the MCP research tools (alphaXiv get_paper_content, Firecrawl scrape,
  Parallel web_fetch, Exa web_fetch) instead. crates.io and PyPI are reachable.
- Python: plain `python3` has no numpy; use `PYTHONPATH=<scratchpad>/pylib python3` (numpy 2.4 + scipy 1.17 installed there).
  Rust: `~/.cargo/bin/cargo` (toolchain 1.97.1). Standalone scratch cargo projects must live in the scratchpad, NOT in the repo.
  Python is fine for scratch analysis; the project policy only forbids adding Python model code to the repo.

## Who the owner is and what they asked
The owner (Casey) is a mathematician/computer scientist (Rust-first, cloud engineer, first-principles learner) developing a
"universal object standard" (UOR) with a small expert team; UOR concepts include geometric routing, hardware address pointers and
deduplication. UOR's stated aim is to "free computing from GPU and overheating". The owner asked for:
> a complete review of this repo, and stress test the computer science, the math, and the physics of this research project to see if we
> are actually capable of making a novel breakthrough here - diagnose whether or not we have fallen off course, or whether there are
> additional ideas we can add to the project that would ease our research. My only concern is that it must be transformerless, as in
> no traditional matmul in runtime (floating point), moe, or sparse-routing. Our goal is to create a geometric language model... the
> original idea was much like Google's turboquant, but instead of ablating the radial direction, we preserve it, and we use 4
> dimensions instead of 2. From that emerged prime least energy riemann manifold routing. Then I realized I could store and recall.
> From there it has kept evolving into an attempt to replace the wasteful matrix multiplication in the serving runtime, with geometric
> intelligence, coupled with geometric attention, reasoning, etc. Currently we are still working on attention, and have drifted a bit
> from pure geometry. ... figure out the correct theories and the correct path to the goal of this project. A geometric language
> model that can run locally on the equivalent of an M1 macbook pro.

Interpretation notes (for you to respect): "no traditional matmul in runtime (floating point)" = serving may not use float matmul;
the project's adopted decision D0-b ALSO permits <=4-bit integer/ternary linear maps executed as adds/subtracts/shifts/table reads with
no multiplier instruction. "no MoE or sparse-routing" most likely means no learned MoE-style expert gating; the owner's own
"geometric routing" (deterministic address/manifold routing) is part of the project's history. NOTE a possible tension: project decision
D5 makes *per-token parameter sparsity* (reading only a selected subset of parameters per token) the terminal serving invariant — that
is itself a form of routing. Flag this tension where relevant; do not silently resolve it.

## Governing documents (read what your brief needs)
- `AGENTS.md`, `README.md`, `docs/integration/current-state.md` (live state), `docs/integration/DECISIONS.md` (D0..D9),
  `docs/integration/direction-decision-2026-09-24.md`, `docs/integration/stuck-point-review-response-2026-09-24.md`,
  `docs/integration/model-direction-2026-09.md`, `docs/integration/project-track.md` (97 KB canonical plan), `docs/PROJECT_MAP.md`,
  `docs/integration/repo-review-direction-2026-09-23.md`, `docs/integration/principal-attention-plan-2026-09-24.md`.
- Math/geometry: `docs/integration/geometric-attention-research-2026-09/{mathematical-foundations,repository-mathematics,attention-and-learning,README}.md`,
  `docs/integration/architecture-2026-09/{mathematics,engines,imports,README}.md`, `docs/formal_vocabulary.md`.
- Original router research (the "prime least-energy Riemann manifold routing" era): `research/ai-research/ai-router/router-research/*.md`
  (CORE_PROJECT_GOALS.md, geometric_routing_architecture_summary.md, THEORY_SKETCH.md, hyperbolic_router_math_review.md,
  phase_transport_hypothesis.md, minimal_theorem_for_spectral_emergence.md, geometric_routing_kill_tests.md, EVIDENCE_SUMMARY.md ...),
  `research/prime-analysis/`, `research/riemann-lean/`, `research/spiralcore-v68/`.

## Facts already established by the lead (verify if your conclusions depend on them)
- Current model path (D8 ladder): offline Rust/Candle F32 training crate `crates/uor-r4-training` (key: `src/joint_model.rs`,
  `src/joint_campaign.rs`, `src/joint_parallel.rs`, `src/joint_quantization.rs`, `src/joint_rounding.rs`) and a standalone integer
  serving crate `crates/uor-r4-integer` (`src/model.rs`, `src/math.rs`, `src/sampling.rs`, `src/generation.rs`, `src/config.rs`).
- Architecture (from `crates/uor-r4-integer/src/config.rs` shapes + legacy contract string): vocab 4096 (BPE on TinyStories),
  width d=256, read width 64, context 256. Params: embedding [4096,256] (tied output, plus output.norm & output.bias),
  GRU-like recurrent input/state weights [3d,d] each, single-head soft read (Q,K: [64,d]; V: [d,d]) over ALL previous events in the
  256 window with a learned per-age bias and a learned NoRead null slot, update [d,2d] + scalar update gate, scalar copy/pointer gate
  mixing vocabulary softmax with a pointer distribution over previously observed tokens. "Transport": per 4-D lane,
  q = normalize(e0 + 0.1*raw) then LEFT Hamilton multiplication of the state lane (quaternion arm) OR a product of two Householder
  reflections (ordinary control arm). Lead's rough count: ~1.68M parameters total, ~1.05M of which is the tied embedding,
  i.e. only ~0.6M non-embedding parameters. (Please verify.)
- Training: full differentiable unroll, no detach, AdamW; ~29.999M target visits per arm; measured ~1,362 target visits/second per
  arm on an 8-core 16 GB M1 (CPU, two workers per arm; Metal was slower).
- Results (comparison-tail NLL, nats/token, 233,472 targets, lower is better): quaternion 2.110368; matched ordinary (Householder) control
  2.085241; interpolated 5-gram + cache 2.391786; historical 7,155,360-parameter 6-layer width-288 transformer (#1017, trained on
  ~150M tokens) 1.574024. NoRead ablation: ~2.59/2.56. Generated text "varied but semantically unreliable" (role confusion, drift).
- Quantization/integer serving (Sept 25): 4-bit signed weights with power-of-two row scales; learned rounding recovers most loss
  (hard minus continuous +0.024/+0.046 nats); integer serving session executes full-256 attention with integer arithmetic, ~3.7 ms per
  Read model call, 4.5–6.6x speedup from a "signed4 product-table" optimization; dense per-token parameter access remains
  (D5 non-compliant); no energy measurement exists.
- Decisions: D0-b (<=4-bit additive/LUT maps allowed at serving, no multiplier, no float), D4 (Goal R "geometric predictive advantage"
  re-scoped to a gated hypothesis: *no geometric mechanism has beaten a matched ordinary control on any task in the repo*),
  D5 (per-token parameter sparsity = terminal serving invariant), D6 (target objective = long-range information probe),
  D8 (reference -> joint learner -> discretize -> bounded admission -> integer export ladder), D9 (stop experiment loops).
- An older "best served language artifact" (`learner/transferable_lexical.rs`) was a 64-dim integer RNN with *no geometry at all*.
- The repo is huge: `crates/uor-r4-core` ~410k lines of Rust; ~660k lines of Rust total; `docs/integration/` ~280 files (many
  per-experiment "DeepSeek step" / "principal review" documents); `research/` ~11k files.

## Output format for your report
1. **Executive verdict** (5–10 bullets).
2. **Findings** — numbered, each with label(s) and evidence.
3. **What is genuinely strong / novel** vs **what is weak, wrong, or irrelevant**.
4. **Recommendations** — concrete, ranked, each with expected impact, cost, and how to falsify it quickly.
5. **Open questions** you could not resolve.
Be candid. The owner explicitly wants to know whether a breakthrough is realistic and whether the project has drifted. Tact is good;
flattery is useless. Keep the report under ~2,500 words unless your brief says otherwise, but include all key numbers.
