# Generated-trajectory copy stop gate

This record fits a geometric stop gate on the actual longest-first copy
trajectories from panel 2711 and evaluates it unchanged on panel 2712. The gate
uses the existing 256-bit sign-signature, eight-prototype-per-class Hamming
learner. The terminal label is the normalized reference length.

The main result is negative: on both artifact seeds the dev gate produced
**0/449** exact replies. It sometimes fired at the reference length, but those
bodies were not exact. Training-panel premature stops were **34** and **31**
against a 4-row 1% limit; dev premature stops were **3** and **8**.

The oracle ceiling remains 11/453 train and 13/449 dev. The current gate does
not recover it. This closes the fixed all-reference-length labeling with eight
signed prototypes; it does not close geometric termination generally.

Attempts 1–5 are preserved:

- attempt 1 used the wrong report schema literal;
- attempt 2 completed but wrote transient `/dev/fd` report paths;
- attempt 3 stopped after report reads were moved to hashes because a process
  substitution could not be read twice;
- attempt 4 reproduced the result but omitted per-row fired decisions and
  selected-trace hashes;
- attempt 5 added those records but counted no-fire cost only to the reference
  horizon.

Attempt 6 is final, includes per-row audit records and a byte-identical repeat
after timing removal.
`no_fire` means no gate firing through the reference horizon; token cost is
charged to the complete 64-token selected trace.
Published evidence is in [`evidence/`](evidence/).

## Exact-body follow-up

Deepseek-159 filtered training to the 11 trajectories whose generated body
already equalled the reference before fitting the same gate. Both artifact
seeds then fit **7/11** terminal rows with zero false stops on 308 selected
nonterminal rows.

Attempt 7 opened the earlier dev panel before an independent FIT record and is
preserved as exploratory evidence. It produced 1/13 exact replies per seed.

The registered confirmation used a new seed-2722 dev panel after FIT and GO.
It had 458 reachable replies and a 10-reply oracle ceiling on each artifact
seed, but the gate produced **0/10** exact replies on both. Premature stops
were **5** on seed 1 and **9** on seed 2, against the four-stop limit. The
shuffled, all-reference and no-gate controls each produced zero exact replies.
Attempt 8 is the sealed registered result, including byte-identical
fresh-process repeats.

This closes exact-body filtering for this eight-prototype gate, not geometric
termination. See
[deepseek-159](../../loop/tasks/cpu-training-pilot/deepseek-159/REPORT.md).
