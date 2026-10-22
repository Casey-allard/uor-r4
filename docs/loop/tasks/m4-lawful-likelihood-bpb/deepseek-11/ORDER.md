# ORDER: m4-lawful-likelihood-bpb, deepseek-11 (record the owner's Card 1 answer in the design note)

Lab: deepseek. Item: [m4-lawful-likelihood-bpb](../../../../roadmap/items/m4-lawful-likelihood-bpb.md) (#2861). Base: `1c6c396ccaec2f599bad738da8af2581eac726c3`.

## Goal

Record the owner's decision on Card 1 in the design note. Casey answered in Claude's chat at about 04:15Z on 22 October 2026 and Claude recorded it verbatim on #2028 at 04:17Z: "Yes, a linear ratio (Recommended)" — CHARTER §3.2 bans exponential and squashing transforms, and dividing non-negative potentials by their sum is lawful, and the item itself asks for an exact sum over the alphabet. The note's conditional wording ("nothing is called lawful until Card 1 is answered") is now obsolete and must be updated to the owner's reading, with the decision quoted and cited. Card 2 (`-log2(mass)` as a measurement) stands as read.

## History

- deepseek-8 `0a4549689` added the design-only note; Claude's CHANGES at `e9748f1` produced deepseek-9; a fresh-context review at `effe1e9c1` produced deepseek-10 (conditional lawfulness, rank-weight citation, emissions correction). This unit records the owner's answer to Card 1 in that repaired note.

## Cancel probe (run first)

Class **instrument** (a design record). The owner decision is recorded on #2028 at 04:17Z and answers the one card the note says gates its central word; leaving the note conditional would make the repository record contradiction the owner's decision. No code or measurement is involved.

## Files in scope

The item's seven globs and this task folder; this repair edits `docs/labs/m4-lawful-likelihood-bpb/README.md` and the task folder only.

## Done when

- The note quotes the owner's answer verbatim with its time and the #2028 source, and states that the construction is lawful under the owner's reading of §3.2.
- The conditional status, heading and property 6 are updated to match; Card 2 stays as the note reads it (a measurement), and the remaining choices (held-out stream, calibration, family arms) stay open.
- The REPORT-11 records the decision and the changed wording; the item stays `ready` (no code, no measurement).
- `loop gates --full` GREEN; `report --check` and `pr-check --local` pass.

## Stop and ask if

- The decision as recorded is ambiguous or contradicts the note's construction (it does not).
- A bar, status, constraint or spend change is needed; GPU work becomes implied.