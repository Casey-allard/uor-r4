# Roadmap

UOR-R4 marks progress with eight milestones, M1 to M8. **M1 is complete: 1 of 8.** A milestone
closes only when its full bar is met on a saved model. The evidence and limitations are in
[where we are](docs/start/where-we-are.md); the route and open questions are in
[where we're going](docs/start/where-were-going.md) and the one [PLAN](PLAN.md).

![The project road: M1 marked complete on the saved discrete model, while the current end-to-end line works toward grounded replies, durable memory and the remaining milestones.](docs/figures/start/roadmap.svg)

*M1 is complete; the current work is replies from the model's own state and one useful model end to end.*

| Milestone | Bar to close it | Status |
|---|---|---|
| [M1 Language base](https://github.com/UOR-Foundation/uor-r4/issues/2029) | **Original D39 bar, preserved at item start:** one saved discrete model: the right answer reachable (within the candidates the model opens) at ≥ 4,894 of 14,287 positions before training and kept on both seeds; more than 16 of the 256 address cells in use (the occupancy gate, [D39](docs/integration/DECISIONS.md#d39--m1-closing-bar-k16-reachability-gate-post-qualification-sequence-and-lab-self-direction-owner-2026-10-18) item 1); integer top-1 through the integer export beats the unigram table (1,226) and the next-word table (6,300) on each of two seeds, with tokens per second reported; the gain holds on a fresh sealed panel. **Dated close outcome:** the pre-existing D68 decision at [log 6515151261](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6515151261), not a new bar in this PR | **complete, 30 Oct 2026** |
| [M2 Grounded reply from exact memory](https://github.com/UOR-Foundation/uor-r4/issues/2030) | ≥ 256/512 saved questions and ≥ 40% of genuinely fresh ones, with the existing graders unchanged | in progress |
| [M3 Durable conversation memory](https://github.com/UOR-Foundation/uor-r4/issues/2031) | Memory persists across sessions and answers stay grounded | in progress |
| [M4 One served model](https://github.com/UOR-Foundation/uor-r4/issues/2032) | One model exported as an integer file under the serving contract (D11), usable from a command line, with held-out quality at or below 0.90 bits per byte | not complete; accepted model path not started |
| [M5 Laptop cost](https://github.com/UOR-Foundation/uor-r4/issues/2033) | Measured energy per token, memory and tokens per second on an M1-class laptop | not started |
| [M6 Reasoning and coding](https://github.com/UOR-Foundation/uor-r4/issues/2034) | Executable reasoning and coding on the same model | not started |
| [M7 API, browser and Studio](https://github.com/UOR-Foundation/uor-r4/issues/2035) | The model reachable through an API, in the browser (WebAssembly) and in the Studio, giving bit-identical tokens across the library, command line and browser | engine/harness delivered on a synthetic stand-in; accepted model open |
| [M8 Alpha release](https://github.com/UOR-Foundation/uor-r4/issues/2036) | One versioned, checksum-pinned model file, declared alpha by the owner | not started |

M1's two-seed results are **7,002 / 7,001** on DEV (T2 6,300), **7,215 / 7,228** on Fresh V2
(T2 6,582), and **7,850 / 7,872** on the held-out-phrasing p-f panel (T2 7,375) ([M1 close](docs/labs/m1-close-saved-model/README.md), [Fresh V2](docs/labs/m1-fresh-panel-transfer/README.md), [bar-f](docs/loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md)). The p-f panel's
62.5% reference-reply overlap with TRAIN means this is a scoped language-base result, not useful
conversation. M2 is the next capability milestone.

**Since 31 Oct (14 Nov 2026).** The work on M2 is recall in controlled conversations, on the
separate FBS trunk. [Claude-358](docs/loop/tasks/one-model-end-to-end/claude-358/REPORT.md)'s
three models state a told value on 314, 320 and 325 of 578 recall questions. Served without their
exact turn memory they give 171 to 211 exact replies of 2,218, against 377 to 441 with it, and
almost every fact comes from that memory's table, not the learned vectors
([V0, claude-376](docs/loop/tasks/one-model-end-to-end/claude-376/REPORT.md)). Since 13 Nov the
route is D85: one persistent integer geometric state shared by training and serving, with float
master weights and export gaps as reference only ([PLAN](PLAN.md#native-space-cycle)). Carrying
that state across training windows ([#3979](https://github.com/UOR-Foundation/uor-r4/pull/3979))
and vector-native reads ([#3978](https://github.com/UOR-Foundation/uor-r4/pull/3978)) are in
progress.

Every run goes ahead on its own pre-run review (D67, [charter](CHARTER.md) §5.1). The one plan is
[PLAN.md](PLAN.md) (D77).
