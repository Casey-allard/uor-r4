# Blind semantic check

The two fresh-context readers received the 92 unannotated user messages and
row IDs. Each independently returned an exact object phrase, ABSENT for
missing support, or CONFLICT for incompatible supported assertions.
Before the scored output was available, the audit joined both sets of reader
votes with the locked host key by row ID. It required exactly one nonempty
vote per reader and per row. Host unsupported rows with zero selected
assertions were labeled ABSENT. Those with multiple incompatible selected
assertions were labeled CONFLICT.

Both readers matched the host key and one another on 92/92 rows. Each reported
80 supported rows, eight absent rows and four conflicts. The conflict rows
were 2709, 2746, 2886 and 2923. The absence rows were 2725, 2762, 2783, 2799,
2820, 2836, 2857 and 2894. Prompt, vote, key and audit output hashes are in
[semantic.sha256](semantic.sha256).

Before score analysis, we described mends/repairs and gathers/collects as
stronger transfer pairs within this single inventory frame. Colors/paints and
indexes/catalogs are weaker. Coloring leaves the method open and indexing can
refer to a narrower operation than cataloging. Both readers retained the same
intended object on all four cues. These descriptors stratify the diagnostic
only. Every adjudicated row keeps its original key and its place in the
registered transfer bar.

The original BPE tapes were also joined by cell, assertion order, object
assignment and instruction flag. On all 32 requested-relation pairs, the
tapes match exactly before and after the supplied query cue, and the cue
IDs differ. The row pairs and audit count are sealed in
`panel-02-pairs.audit.json`.
