# Charter diff notes: draft → adopted (19 Oct 2026)

## Clauses changed by each answer

- **Q1** (client serving): §3.5 drops "any run whose output is evidence" and governs training, evaluation and qualification only. §3.6 now states client serving (CPU, browser WASM, Apple silicon) is allowed, including tokens/s and energy, and that D36's browser-as-CPU clause is replaced. §2.2 names the Studio at `/studio`.
- **Q2** (pre-registration): §4.5 says "required only before", adds "a fresh-context subagent counts as non-author" and "with a fresh FIT" for archived sheets.
- **Q3** (releases): §5.1 adds one typed release per sheet covering arms, seeds and stop rules; D40 wording kept.
- **Q4** (records): §5.6 gives the full path `docs/research/notebook/<line>.md`, makes `NOW.md` plus notebooks replace `current-state.md`.
- **Q5** (coordination): §5.2 rewritten as owner plus one PM seat with owner-named cover; council retired; roster in `NOW.md`; PM operating notes may not contradict the charter. §5.3 opens "Labs self-direct from `NOW.md`".
- **Q6** (M1 bar): new §4.8.
- **Q7** (harness): §5.3 adds automatic presence, no manual cadence, charter-version claims, no `adopt_policy` hold, feature freeze with D27 ports continuing. §5.5 says the queue serializes itself.
- **Q8** (legacy crates): new §2.4, repair rather than archive (the draft's recommended option 1 was archive; the owner chose option 2).
- Other: §4.1 "no terminal status"; §6 lists newly retired items; new "Changing this charter"; Q references and the §3 *Why* lines removed to fit 1,700 words.

## Rule-map rows whose mapping changed

All 16 "owner decision needed" rows are now resolved:

- `D14`, `AGENTS:d14-charter`, `PROTO:council`, `PROTO:council-scope`: council **retired** (Q5); owner boundaries stay §5.1, GitHub authority §5.2.
- `R9`: **retired** as a rule; roster lives in `NOW.md` (§5.2).
- `OWNER:28`, `OWNER:32`: PM cover confirmed, owner names the cover lab (§5.2).
- `D39`: item 1(d) **replaced** for M1 by §4.8; other items unchanged and mapped as before.
- `D24`: kept as §3.5, scope confirmed; `D36`: browser-as-CPU clause replaced (§3.6); its crate item resolved as **repair** (§2.4).
- `AGENTS:cadence-oct9`, `C:cadence`: **merged into §5.6** (notebook cadence).
- `R3`, `PROTO:cadence`, `SG:contract-9`, `SG:status-card`, `AGENTS:harness-surface`, `AGENTS:authority-recovery`: manual cadence **retired**; harness kept as automatic view (§5.3).
- `R-queue`: **retired**; queue self-serializes (§5.5).
- `PROTO:versioned-authority`: `adopt_policy` hold **retired**; claims record charter version (§5.3).
- `R4`: **merged into §4.5** with the confirmed scope.
- `SG:contract-8`: the "two preparation PRs" cap stays retired (Q2 did not restore it).
