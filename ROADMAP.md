# Roadmap

UOR-R4 marks progress with eight milestones, M1 to M8. **None is complete.** A milestone closes only
when its bar is met on a saved model. This page is a pointer: the plain-language road, its six stages
and the products at the end are in [where we're going](docs/start/where-were-going.md); the measured
starting point is in [where we are](docs/start/where-we-are.md).

![A road with six stops, Stage 0 to Stage 5, from "run what is already FIT'd" to "first conversation", with a "you are here" pin at Stage 0, owner-release gates between stops, and milestone flags M1 to M8 underneath, ending at the model file, the browser tab and the laptop app.](docs/figures/start/roadmap.svg)

*The road from today to the products. Each gate is a run on its own pre-run review (D67); the ordered steps are in [PLAN.md](PLAN.md).*

| Milestone | Bar to close it | Status |
|---|---|---|
| [M1 Language base](https://github.com/UOR-Foundation/uor-r4/issues/2029) | One saved discrete model: the right answer reachable (within the candidates the model opens) at ≥ 4,894 of 14,287 positions before training and kept on both seeds; more than 16 of the 256 address cells in use (the occupancy gate, [D39](docs/integration/DECISIONS.md#d39--m1-closing-bar-k16-reachability-gate-post-qualification-sequence-and-lab-self-direction-owner-2026-10-18) item 1); **integer top-1 through the integer export beats the unigram table (1,226) and the next-word table (6,300) on each of two seeds, with tokens per second reported**; the gain holds on a fresh sealed panel | in progress |
| [M2 Grounded reply from exact memory](https://github.com/UOR-Foundation/uor-r4/issues/2030) | ≥ 256/512 saved questions and ≥ 40% of genuinely fresh ones, with the existing graders unchanged | held |
| [M3 Durable conversation memory](https://github.com/UOR-Foundation/uor-r4/issues/2031) | Memory persists across sessions and answers stay grounded | paused |
| [M4 One served model](https://github.com/UOR-Foundation/uor-r4/issues/2032) | One model exported as an integer file under the serving contract (D11), usable from a command line, with held-out quality at or below 0.90 bits per byte | held |
| [M5 Laptop cost](https://github.com/UOR-Foundation/uor-r4/issues/2033) | Measured energy per token, memory and tokens per second on an M1-class laptop | not started |
| [M6 Reasoning and coding](https://github.com/UOR-Foundation/uor-r4/issues/2034) | Executable reasoning and coding on the same model | not started |
| [M7 API, browser and Studio](https://github.com/UOR-Foundation/uor-r4/issues/2035) | The model reachable through an API, in the browser (WebAssembly) and in the Studio, giving bit-identical tokens across the library, command line and browser | engine delivered; no model |
| [M8 Alpha release](https://github.com/UOR-Foundation/uor-r4/issues/2036) | One versioned, checksum-pinned model file, declared alpha by the owner | not started |

M1's integer top-1 condition was set by the owner on 19 October 2026 and replaces the earlier bar of
0.90 bits per byte, which returns at a later milestone once the model gives probabilities
([charter](CHARTER.md) §4.8). The shortest path the audit proposes: **0** run the already FIT-reviewed capacity
test (W2, step 1 in PLAN.md) → **1** make capacity measurable → **2** pick the learning law → **3** add the missing parts →
**4** export and serve → **5** first simple conversation. Training runs on each lab's own pre-run review
(D67, [charter](CHARTER.md) §5.1). The one plan is [PLAN.md](PLAN.md) (D77).
