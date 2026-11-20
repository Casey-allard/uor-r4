# Decision review: claude-440's card on P1's S5 copy path

- **Reviewer:** a fresh-context, read-only reviewer subagent of the claude top session (Opus). It saw
  the card, its ORDER and the track's sources at exact refs, and never the reasoning behind the card.
- **Read at:** card head `0d1ea2fc0`, base `origin/main` `67d54aa0a`.
- **Verdict:** APPROVE WITH CHANGES. Every change is made at the next head; see "Disposition" below.

## The reviewer's findings (summarised; severity and evidence as returned)

**No number mismatch.** Every number in the card matched claude-439's REPORT (score table, bands,
fire counts, the 190‰ and 10‰ weights, the read times, the 24 to 232 edge; "3½‰" is 4,774 /
1,360,806). The reviewer did not open the run2 JSONs.

1. **MAJOR.** The reversal rule's bits condition ("includes 0") misses a band wholly above 0, the
   strongest case for C2. Required: "does not lie wholly below 0", and a DEV-test C2 arm must be
   registered in P1(b)'s sheet before its sealed run (§4.5).
2. **MAJOR.** The card said codex-28's mixture carries A1's column. codex-28's REPORT (line 48)
   speaks of "the original S5 fit" and names no arm. Required: say that codex's bound fit is not yet
   shown to be A1's read, that the packing unit adopts or reconciles A1's column, and that the
   question goes to #4063.
3. **MAJOR.** "It adds no state" overstates. A1 needs each session token's D11 code and a
   per-position code history; C2 needs token ids. Neither footprint was measured, and codex-14 asks
   to record the entire deployed footprint. Required: mark it as an estimate, and have the P1(b) unit
   record S5's deployed bytes and per-session state for A1 and C2.
4. **MAJOR.** No D85 replay requirement is carried forward. Required: the unit that packs A1
   carries a bit-identical CUDA-to-CPU Pick replay on ≥ 64 prompts, with the count's source cited,
   and claude-439's unrecorded gate negative.
5. **MINOR.** The controls list leaves out three of codex-14's: A0 pure count, original-id ties and
   the reloadable D11 carrier.
6. **MINOR.** The claude-409 quote is a paraphrase; "over the trunk's codes" is the card's gloss.
   Quote it exactly.
7. **MINOR.** P2's row shows the operator stays in the tree, not that P1 planned A1 for S5. Soften
   to "A1 reuses the operator that P2 already carries".
8. **MINOR, no change needed.** The speed condition rests on an existing bar; cite it.

On the charter: choosing A1 does not conflict with §3.1 or §3.7 once the replay of finding 4 is
required. C2 is not itself a §3 violation, but as the sole S5 path it would weaken the geometry
share that claude-409 says must carry the gain from 90% to 80% of R\*. The reviewer would make the
same choice.

**Not read by the reviewer** (context budget): CHARTER §3.1, §3.7 and §5.1 in full; codex-26 and
codex-27's REPORTs; the run2 JSONs. The top session closed those gaps below.

## Disposition by the top session

| Finding | Done |
|---|---|
| 1 | The bits condition now reads "does not lie wholly below 0" and requires P1(b)'s sheet to register a C2 arm before the sealed run. |
| 2 | The card now says that codex-28's REPORT names no arm, and that the refit-and-pack unit adopts A1's column or reconciles its own on the record. A note went to #4063. codex-28's `count/INPUTS.md` checks claude-439's split hashes, so the fit it binds is on claude-439's split. |
| 3 | "It adds no state" is replaced by "Its footprint is not measured", marked [est], with the P1(b) recording duty. |
| 4 | The replay is now required: route, tie and read choice on ≥ 64 prompts (the count in claude-409's bar B), plus claude-439's gate negative. |
| 5 | A0, original-id ties and the writable, reloadable D11 carrier are added to the kept controls. |
| 6 | The claude-409 row is quoted exactly. |
| 7 | Softened to "A1 reuses the operator that P2 already carries". |
| 8 | The speed condition now cites claude-409's bar B. "One M1 performance core" is that bar's own wording, not a narrowing. |

**Gaps closed by the top session** (one stays partly open):
- **The charter.** The top session read §3.1 (geometry only, `CHARTER.md` line 26) and §5.1 item 1
  (line 58: the decision review, D68) at `67d54aa0a`. It did not re-read §3.7 in full. The card's
  native-space duty rests on D85 through claude-409's bar B (replay on ≥ 64 prompts), which it
  quotes.
- **The run2 JSON.** At 100% (`run2/pick-p1000.json`), each arm's DEV-tune `bpb_micro` (A0
  1,432,620; A1 1,360,806; C2 1,365,580; C3 1,360,932; novel 1,747,432, 1,651,188, 1,657,701,
  1,651,364) matches the card. So do the bands (A1 − C2 −5,232 / −4,318 overall, −7,141 / −5,889 novel;
  A1 − C3 −232 / −24) and the pick statistics (A1 fired 10,802,077 with 4,111,759 exact; C2 4,108,760).
- **codex-26 and codex-27.** A grep of both REPORTs for S5, claude-439, exact-pair and Pick expert
  finds one line each. codex-26 (line 25) cites claude-439's 24 to 232 micro-BPB band and reports a
  packaging size test. codex-27 (line 130) says the next prediction unit should package the
  reserved-TRAIN mixture into an active D11. Neither line chooses a copy path, so neither conflicts
  with the card. The REPORTs were not read in full.
