# [codex] Fresh role and literal-boundary retest on Wick

## What did not work

On the 24 supported prompts, the normal client emitted the complete answer through EOS on 7. All 17 failures started at generated token 0. Eight lost the request while ranking eligible word spans, and nine had the correct requested span and chose the wrong start of its value. All eight request losses occurred on the same question family. The correct name and the final word “away” each scored 12 on the requested-role axis. Twelve meets the evidence floor. The native winner also requires a strict lead over the runner-up, so every one of those ties cleared the requested span. Both artifacts reproduced this trace. [Original spans, scores, token pieces and six native keys](evidence/role-ties-seed1.json) · [Second artifact](evidence/role-ties-seed2.json) · [Winner operator](../../../../../crates/uor-r4-integer/src/paired_role.rs#L245)

Giving the selector the correct requested span alone recovered six of the eight ties. It left 11 initial-token failures across all 24, including two of the repaired roles. The selected value started too early in seven and too late in four. Of the 18 rows with two equally reachable choices, the existing continuation and the earlier WordStart succeeded on the same seven and failed on the same eleven. Globally switching to WordStart recovered zero and lost two among the six other rows. This panel gives no basis for adopting that switch.

The four absent-person and four contradictory-record questions all received acknowledgments. The public reader nominated their requested spans on all eight, admitted no source, and gave zero factually responsive answers on either artifact. We read every emitted answer. One example was “Got it, I'll remember that.” Several others were garbled. These responses received zero credit even when the client emitted EOS.

## What changed

The project owner asked for an instrument that could demonstrably be right. The first blind semantic reader rejected eight questions before a model saw them. We retained the original panel and changed only the questions for those eight. A second blind reader independently recovered all 24 literal answers, four missing people and four conflicts on the revised text. Runtime tokenization and an independent assembly of the protocol frame matched on all 32 prompts per seed. The Rust test rejected seven kinds of wrong answer per supported row, including truncation and incorrect EOS. [Prospective amendment](AMENDMENT.md) · [Reader answers](evidence/blind-semantic-v2.json) · [Input and scorer identities](evidence/inputs.sha256)

## Measurements

Each column represents one stored D11 artifact. Both saw the same 32 distinct prompts. Index 0 denotes the first generated token. Exactness includes the terminating EOS. Forced lanes receive an original source address and exercise the real GPU cursor. The normal client receives ordinary text.

| Arm | Seed 1 exact / 24 | Seed 2 exact / 24 | First wrong on the other supported replies |
|---|---:|---:|---|
| Public client | 7 | 7 | 17 at index 0 |
| Public client, read disabled | 0 | 0 | 24 at index 0 |
| Correct requested span, existing continuation | 13 | 13 | 11 at index 0 |
| Same correct span, earlier WordStart | 11 | 11 | 13 at index 0 |
| Force the independently identified value start | 24 | 24 | None |
| Force the alternate start | 0 | 0 | All 24 differ from the expected answer |

The cursor emitted the exact rival text through EOS on all 22 rivals within its copy limit. Eighteen were also inside the reader's candidate window. Four exceeded that window and two further rivals exceeded the writer cap. Only the 18 are matched comparisons. Regrading the correct writer output against the other person's factual item changed 24/24 to 0/24 with unchanged emitted IDs and CUDA trace. A separate planted normal client admitted original source 7 and wrote “cobalt” through EOS. Disabling the reader changed that reply and cleared the source. These checks establish that correct and incorrect behavior are observable on the measured paths.

For “Inez saved woven baskets,” the public selector retained Inez but began at original position 10 instead of 8. It emitted ID 1912 for the beginning of “baskets” where ID 1017 starts “woven.” For the entry with “set aside striped napkins,” it began at 19 instead of 21 and emitted ID 450 for the first piece of “aside” where ID 2830 starts “striped.” For “What did Sabine pack or put away?” after “Sabine packed padded envelopes,” the public reader lost the 12–12 tie, emitted ID 1155, and answered “Got it, I'll remember that.” Supplying the requested endpoints recovered “padded envelopes” with the same writer. [Seed 1, every prompt and intervention](evidence/results-seed1.json) · [Seed 2](evidence/results-seed2.json)

## Checks run at the head

The independent non-author FIT and implementation identities preceded collection. The two observed-emission scorer tests passed on both the workstation and Wick. The two fresh-process stages each ran independent public, disabled, selector, direct-address controls and GPU trajectories, with the original grading file absent beside the public input in primary and a false one present in repeat. Cold manifests matched on each seed. Each of the four stage and seed combinations took 62–64 seconds and peaked near 1 GiB process RSS. The 152,225-byte raw archive matched on Wick, the workstation and an independent download of its pinned private-store revision. [Pre-run and environment](PRE-RUN.md) · [Cold digests](evidence/cold.txt) · [Private-store retrieval](evidence/storage.json)

## Skills used

The native-space cycle tied measurements to the frozen integer artifact, source choice, emitted ID and cursor. The research-architect review set up independently answerable source and negative controls.

## Still open

Run a prospective tie-discrimination test with independently varied requested role, position, recurrence and trailing clause. Include cases where a later word is the correct requested name, two equally plausible people, and a true absence. Grade the exact endpoints, erroneous admissions and full answers. A forced preference for an earlier position could pass all eight observed ties by accident.

For literal boundaries, trace how the two retained policies arrive at the same incorrect address on all 11 matched failures. Cross short and long connectors with the same item and cross the same connector with different items before fitting another operator. The frozen writer is a validated control on these short literals. Useful free replies, turn-to-turn memory, the equal-byte prediction comparison and device efficiency each require further measurements on a common deployable artifact. [Program milestones and dependencies](../codex-9/CLOSURE-PLAN.md)

## Plain summary

The strongest new lead is a repeatable tie between the person's name and “away.” Removing that tie by supplying the name recovers six answers. A second defect puts the cursor on the wrong word inside a correctly identified fact. The current onset rules share that defect on every comparable failure in this panel. With the correct address the same writer emits all 24 complete answers.
