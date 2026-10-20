# #2061 — Token-identity copy stop: the open population is empty, with the mechanism measured (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T17:17:43Z
- **Updated:** 2026-10-09T17:20:52Z
- **Closed:** 2026-10-09T17:19:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2061
- **Merged:** 2026-10-09T17:19:12Z
- **Merge commit:** 0284e6eb40dabaa4c146842c08a4641ddcc6953e
- **Base / Head:** main / UOR-Foundation:deepseek/idstop2-record
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Token-identity copy stop: the open population is empty, with the mechanism measured (#2070)

Lands the second, independent run of the identity-stop classification together with the canonical
record and the current-state entry the delivery cadence now requires.

The 2x2 at floor 0.5 is fires+exact 20, fires+not-exact 3, no-fire+exact 52, no-fire+not-exact 9,
reproducing the sealed 72/84 = 0.8571 [0.7667, 0.9163]. What is new is WHY the 9 do not fire, and it
contradicts the premise both earlier briefs carried: THE SPAN OPENS IN 9 OF 9 OF THEM. The relaxation
premise (source right, first emitted id differs) holds for 1 of 9; a run OPENED BUT NEVER FIRED in 8
of 9. The premise sentence is 'My name is Ithmar.'; the value's span is window 7..10; the run opens
at position 4 and reproduces 4..11 - the whole premise sentence with the value inside it - so the
reply is the user's own sentence read back. The mirror confirms it: of the 52 non-firing-but-exact
cells, 52/52 opened a run and 50/52 opened exactly at the value start. All 61 non-firing cells ended
with EOS; none ran to the token cap, so a non-firing reply does not 'run on' at this floor.

MAXIMUM GAIN FROM THE MOST FAVOURABLE RELAXATION IS 0 EXACT CELLS - firing the earliest run covering
the value yields the premise sentence every time (8 ids against the value's 4; 7 against 5) and 4 of
the 9 have no covering run at all. It was therefore not implemented, and no interval or
template-firing count exists because nothing was run.

Template firing was NOT reintroduced: of the 23 fires the span is fully inside the value's own span on
21 and overlaps on 22, against the run-length rule's 20 of 79 at floor 1.0.

THE DEFAULT IS PROVED UNCHANGED TWICE, and the second run is the one that matters: rule-off at floor 0
compared field-for-field against the sealed references (9/9 arm-condition files, 288 records, 0
differing fields) with the original binary AND with a binary BUILT FROM MAIN. The pod tree carried the
pre-merge rule (CopyStop{identity: bool} against &window[..position]) while main carries the merged
one (CopyStopMode against the full window); main reproduces the identical 72/84, the identical 2x2 and
identical reply ids. Four focused copy-stop tests pass on main.

Scope: OFFLINE ONLY. Export refuses a pointer carrying an identity term, so there is no integer port
and none of this is served. The 4 class-A failures are a generator/EOS failure that no serving-time
read-out rule can touch. Remaining unknown: no trace on the identity arms, other floors, non-greedy
decoding, other panels.

Cost: approximately $1.41 of pod time across four pods, all released and deleted. Two of those pods
were downed by my own cleanup while this run still wanted them, and the run also lost about 25 minutes
to the canonical-vs-EU-RO-1 volume split; both are recorded in the evidence so the next runner does
not repeat them. Pin placement with UOR_POD_VOLUME_DCS=EU-RO-1.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

