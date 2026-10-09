# Token-identity copy stop: the open population is empty, and why

Date: 2026-10-09. Lab: deepseek. Issue: #2029. Scope: offline capability, not served.

## Outcome

The token-identity copy stop reaches **72/84 = 0.8571** exact recall of never-seen values
(Wilson [0.7667, 0.9163]) on arm-ptr at gate floor 0.5. This record establishes why it does not go
further: **not mis-tuning, and not a missing relaxation.** The instrument is exhausted at this floor.

## The 2x2 (84 cells, arm-ptr, F=0.5, identity stop, copy_trace=1)

|                  | exact | not exact | total |
|------------------|-------|-----------|-------|
| rule FIRES       | 20    | 3         | 23    |
| rule does NOT fire | 52  | 9         | 61    |
| total            | 72    | 12        | 84    |

Reproduced independently twice: by the idstop3 round and by the idstop2 round, on different pods, and
the second time with a binary **built from main** as well as the original. Identical 2x2, identical
reply ids.

## Why the 9 non-firing failures do not fire

The span **opens in 9 of 9** of them - the proposed relaxation premise ("source right, first emitted id
differs") holds for **1 of 9**. The real mechanism:

- pointer's argmax source wrong (no run opened): **0/9**
- source right but first emitted id differs: **1/9**
- **a run OPENED but never fired: 8/9**

The premise sentence is `"My name is Ithmar."`; the value's span is window 7..10; the run **opens at
position 4** and reproduces **4..11** - the whole premise sentence, value inside it. The reply is the
user's own sentence read back. The mirror image confirms it: in the 52 non-firing-but-exact cells,
**52/52 opened a run and 50/52 opened exactly at the value start (7)**, which is why those replies are
the bare value.

**All 61 non-firing cells ended with EOS; not one ran to the 40-token cap.** A non-firing reply does not
"run on" at this floor.

## Maximum gain from the relaxation: 0 exact cells

Forcing the most favourable relaxation - fire the earliest run covering the value - yields the premise
sentence every time (8 ids against the value's 4; 7 against 5), and 4 of the 9 have no covering run at
all. No interval and no template-firing count exist because nothing was run: the precondition was never
met.

## Template firing was not reintroduced

Of the 23 cells the identity rule fires on, the fired span lies **fully inside the value's own span on
21** and overlaps it on 22 - against the run-length rule's 20 of 79 at floor 1.0. This is the positive
control the earlier copy-stop round lacked.

## The default is unchanged, proved twice

Rule-off at floor 0 compared field-for-field against the sealed references: **9/9 arm-condition files,
288 records, 0 differing fields**. Run twice - with the original binary (sha256 `eb8cb85c...`) and with
a binary **built from main** (sha256 `9a320224...`, source md5s byte-identical to `origin/main`). That
second run matters: the pod tree carried the pre-merge rule (`CopyStop{identity: bool}` against
`&window[..position]`), main the merged one (`CopyStopMode`, against the full window) - and main
reproduces the identical 72/84, the identical 2x2, and identical reply ids. Four focused copy-stop
tests pass on main.

## Scope and limits

Offline only: export refuses a pointer carrying an identity term, so there is **no integer port** and
none of this is served. No trace on the two identity arms at F=0.5 (scored, not traced), none at other
floors, no non-greedy decoding, no other panel. The class-A cells (the value never emitted) are a
generator/EOS failure at this floor that no serving-time read-out rule can touch.

## Reproduction

Instrument: `crates/uor-r4-training/examples/bind-probe-exact.rs` (on main since #2054).
Scorers: `scripts/idstop-{score,crosschecks,turn1check,compare-records}.py` (on main since #2056).
Artifacts live on the NON-CANONICAL EU-RO-1 volume `rfsx702p68` (`uor-shared-EU-RO-1`); pin placement
with `UOR_POD_VOLUME_DCS=EU-RO-1` or a EUR-NO-1 pod mounts `lmd1pfah3y` and sees none of it.
