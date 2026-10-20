# Rule map: every rule in `rules.json` → the draft charter

Final, 19 Oct 2026. One row per rule id in `docs/labs/repo-renormalization-2026-10-19/audit/rules.json` (258 rules), in that file's order. Disposition values: **kept as charter §x** (the rule survives with its meaning), **merged into §x** (folded into a broader charter clause), **retired: reason** (history only; the text stays in `archive/`, nothing is deleted). Section numbers refer to the adopted `CHARTER.md`. "Status" is the source document's own claim, copied from `rules.json`.

| Rule id | Kind | Status | Source | Disposition |
|---|---|---|---|---|
| `D0` | technical | superseded | `docs/integration/DECISIONS.md:11` | retired: superseded by D0-b the same day; its sparsity test survives as D5 (§3.9) |
| `I1-I5` | technical | in-force | `docs/integration/DECISIONS.md:73` | kept as charter §3.9 |
| `D0-b` | technical | superseded | `docs/integration/DECISIONS.md:89` | retired: superseded by D11 (serving) and D25/D30 (no dense maps); history only |
| `D1` | process | superseded | `docs/integration/DECISIONS.md:138` | retired: superseded by D2/D12/D20/D22; "retire from critical path" conflicts with D22 (§4.1) |
| `D2` | process | in-force | `docs/integration/DECISIONS.md:153` | merged into §4.1 (an ablation is a measurement; enabler class survives as PARKED-with-reopen-conditions) |
| `D3` | owner-boundary | superseded | `docs/integration/DECISIONS.md:213` | retired: superseded by D25/D30 (offline matmul permission withdrawn) |
| `D4` | owner-boundary | in-force | `docs/integration/DECISIONS.md:223` | merged into §4.4 (geometric advantage is measured against lawful non-neural controls; neural "ordinary controls" barred by D28, §3.10) |
| `D5` | technical | in-force | `docs/integration/DECISIONS.md:244` | kept as charter §3.9 (D5-b) |
| `D6` | technical | in-force | `docs/integration/DECISIONS.md:263` | merged into §4.4 (long-range probe items plus count-table local controls) |
| `D7` | process | superseded | `docs/integration/DECISIONS.md:275` | retired: superseded by D8/D19 |
| `D8` | technical | superseded | `docs/integration/DECISIONS.md:285` | retired: superseded by D32/D37 (training method) and D28 (no neural reference) |
| `D9` | process | in-force | `docs/integration/DECISIONS.md:319` | merged into §4.4 (pivot note; repeat needs new causal evidence) and §5.3 (intent note replaces the work card) |
| `D10` | technical | superseded | `docs/integration/DECISIONS.md:367` | retired: superseded by D11 (SmolLM2 backbone withdrawn) |
| `D11` | technical | in-force | `docs/integration/DECISIONS.md:402` | kept as charter §3.6 |
| `D12` | process | in-force | `docs/integration/DECISIONS.md:431` | merged into §4.1 (gates promote, never kill; no retired status) |
| `D12:toolbox` | process | in-force | `docs/integration/DECISIONS.md:469` | merged into §4.1 (PARKED mechanisms stay building and documented on the mechanism board) |
| `D13` | process | superseded | `docs/integration/DECISIONS.md:477` | retired: superseded by D14/D18/D19 |
| `D14` | process | in-force | `docs/integration/DECISIONS.md:508` | retired: D14 three-seat council and fixed cadence (owner 19 Oct, Q5); GitHub authority kept as §5.2, owner boundaries as §5.1 |
| `D14:owner-boundaries` | owner-boundary | in-force | `docs/integration/DECISIONS.md:508` | kept as charter §5.1 |
| `D15` | technical | superseded | `docs/integration/DECISIONS.md:568` | retired: superseded by D17; converted-student serving is excluded by D28/D11 |
| `D16` | process | superseded | `docs/integration/DECISIONS.md:593` | retired: superseded by D17 |
| `D17` | process | in-force | `docs/integration/DECISIONS.md:623` | merged into §4.3 (paired seeds; tolerance from measured noise; serving stays transformer-free per §3.10) |
| `D18` | process | time-boxed | `docs/integration/DECISIONS.md:689` | retired: its 1-14 Oct window expired; its D5-b re-entry gate is cited by D38 item 5 and stays with the archived D18/D38 text (§3.9) |
| `D19` | process | in-force | `docs/integration/DECISIONS.md:812` | merged into §1 (conversation and durable memory first) and §5.5 (self-review allowed) |
| `D20` | owner-boundary | in-force | `docs/integration/DECISIONS.md:894` | merged into §3.1 (geometry first) and §4.1 (no retired status makes the D20 suspension procedure unnecessary) |
| `D21` | process | in-force | `docs/integration/DECISIONS.md:1051` | merged into §4.4 (pivot note after three non-moving results) |
| `D21:orders` | process | one-off | `docs/integration/DECISIONS.md:1073` | retired: one-off lab orders of 9 Oct, executed or overtaken (D22/D24) |
| `D22` | owner-boundary | in-force | `docs/integration/DECISIONS.md:1094` | kept as charter §4.1 |
| `D22:power` | process | in-force | `docs/integration/DECISIONS.md:1120` | kept as charter §4.3 |
| `D22:transfer` | process | in-force | `docs/integration/DECISIONS.md:1127` | kept as charter §4.3 |
| `D23` | technical | in-force | `docs/integration/DECISIONS.md:1143` | kept as charter §3.2 |
| `D24` | technical | in-force | `docs/integration/DECISIONS.md:1174` | kept as charter §3.5, scoped to research compute; serving carve-out in §3.6 needs owner confirmation: Q1 |
| `D24:cancel` | process | one-off | `docs/integration/DECISIONS.md:1202` | retired: one-off cancellation record; the tarball stays in the archive |
| `D25` | technical | superseded | `docs/integration/DECISIONS.md:1214` | retired: superseded by D32/D37; its squash and matmul bans live on in §3.2/§3.3 |
| `D25:rule-freeze` | process | superseded | `docs/integration/DECISIONS.md:1255` | retired: superseded by D32; the freeze ended with the substrate change |
| `D25:no-matmul-anywhere` | owner-boundary | in-force | `docs/integration/DECISIONS.md:1271` | kept as charter §3.3 (ternary/XOR stays a PARKED candidate on the mechanism board) |
| `D26` | ops | in-force | `docs/integration/DECISIONS.md:1310` | kept as charter §5.4 |
| `D26:caps` | ops | in-force | `docs/integration/DECISIONS.md:1328` | kept as charter §5.4 |
| `D27` | technical | in-force | `docs/integration/DECISIONS.md:1340` | kept as charter §3.11 |
| `D28` | owner-boundary | in-force | `docs/integration/DECISIONS.md:1384` | kept as charter §3.10 |
| `D29` | technical | in-force | `docs/integration/DECISIONS.md:1404` | kept as charter §3.4 |
| `D30` | technical | in-force | `docs/integration/DECISIONS.md:1447` | merged into §3.3 (no matmul, no laundering), §3.5 (GPU-only) and §3.11 (Rust only) |
| `D31` | process | in-force | `docs/integration/DECISIONS.md:1473` | kept as charter §4.2 |
| `D32` | technical | superseded | `docs/integration/DECISIONS.md:1512` | merged into §3.7 (the discrete integer path stays the active path); item 1 float ban revised by D37 |
| `D32:hamming-one-factor` | technical | in-force | `docs/integration/DECISIONS.md:1574` | merged into §3.1 and §3.12 (one shared vector writer) |
| `D32:zeta-exception` | technical | in-force | `docs/integration/DECISIONS.md:1597` | kept as charter §3.8 |
| `D33` | owner-boundary | one-off | `docs/integration/DECISIONS.md:1620` | retired: one-off delegated release for the A2 pilot; superseded by D40 |
| `D34` | process | in-force | `docs/integration/DECISIONS.md:1646` | merged into §5.8 (approved sequence, lives in NOW.md) |
| `D34:target-symlink` | ops | one-off | `docs/integration/DECISIONS.md:1666` | retired: one-off, executed 2026-10-14 |
| `D35` | process | superseded | `docs/integration/DECISIONS.md:1674` | retired: superseded by D39 |
| `D36` | ops | in-force | `docs/integration/DECISIONS.md:1714` | merged into §2 (web client) and §3.6; the clause "browser serving counts as CPU model compute" is replaced by §3.6 (Q1); docs deploy already amended by renorm decision 6; native crate repair-or-retire stays an open owner item (Q8) |
| `D37` | technical | in-force | `docs/integration/DECISIONS.md:1742` | kept as charter §3.7 |
| `D37:why-bans` | owner-boundary | in-force | `docs/integration/DECISIONS.md:1752` | kept as charter §1 |
| `D37:archived-sheets` | process | in-force | `docs/integration/DECISIONS.md:1761` | merged into §4.5 |
| `D38` | technical | in-force | `docs/integration/DECISIONS.md:1781` | kept as charter §3.9 |
| `D39` | process | in-force | `docs/integration/DECISIONS.md:1791` | merged into §4.8: M1 bar (d) replaced by integer top-1 vs both count tables; other D39 items kept as before (owner 19 Oct, Q6) |
| `D39:gcap` | process | in-force | `docs/integration/DECISIONS.md:1795` | merged into §4.2 (reachability gate is an instrument check) |
| `D39:injective-init` | technical | in-force | `docs/integration/DECISIONS.md:1796` | retired: no longer needed as a rule; under the charter it is an ordinary research option needing no permission; recorded in the m1-discrete-attention notebook |
| `D39:sequence` | process | time-boxed | `docs/integration/DECISIONS.md:1797` | merged into §5.8 (approved sequence W1-W4, binds until the owner changes it) |
| `D39:export-gap` | technical | in-force | `docs/integration/DECISIONS.md:1798` | kept as charter §3.7 |
| `D39:self-direction` | process | in-force | `docs/integration/DECISIONS.md:1800` | kept as charter §5.3 |
| `D39:float-horizon` | owner-boundary | time-boxed | `docs/integration/DECISIONS.md:1801` | merged into §3.6 (serving stays integer; a float-serving change needs evidence and an owner decision) |
| `D40` | owner-boundary | in-force | `docs/integration/DECISIONS.md:1804` | kept as charter §5.1 |
| `D40:ratify` | owner-boundary | one-off | `docs/integration/DECISIONS.md:1806` | retired: one-time ratification, explicitly no precedent |
| `D41` | owner-boundary | time-boxed | `docs/integration/DECISIONS.md:1811` | kept as charter §3.10 (sole exception) with its execution order in §5.8 |
| `D41:conditions` | ops | time-boxed | `docs/integration/DECISIONS.md:1814` | merged into §3.10 (conditions travel with D41 text in the archive and the NOW.md entry) |
| `AGENTS:d37-oct18` | technical | in-force | `AGENTS.md:3` | merged into §3.7 |
| `AGENTS:bans-goal` | owner-boundary | in-force | `AGENTS.md:12` | kept as charter §1 |
| `AGENTS:d32-oct13` | technical | superseded | `AGENTS.md:17` | retired: dated direction block superseded by D37 |
| `AGENTS:op-phase-oct12` | technical | superseded | `AGENTS.md:30` | retired: dated direction block superseded by D32/D37 |
| `AGENTS:holds-oct12` | owner-boundary | in-force | `AGENTS.md:47` | kept as charter §5.1 |
| `AGENTS:spiralcore-pod-permission` | ops | one-off | `AGENTS.md:50` | retired: one-off scoped pod permission, now covered by the general compute rule §5.4 |
| `AGENTS:shared-vector` | technical | in-force | `AGENTS.md:61` | kept as charter §3.12 |
| `AGENTS:no-softmax-oct12` | technical | in-force | `AGENTS.md:71` | merged into §3.2 |
| `AGENTS:no-cpu-oct12` | technical | in-force | `AGENTS.md:83` | merged into §3.5 |
| `AGENTS:consolidation-oct9` | process | in-force | `AGENTS.md:96` | merged into §5.2 (main is the source of truth) and §5.5 (verify merge before successor) |
| `AGENTS:codex-worktree-oct9` | process | superseded | `AGENTS.md:109` | retired: superseded by R10 (now §5.5) |
| `AGENTS:cadence-oct9` | process | in-force | `AGENTS.md:117` | merged into §5.6: notebook cadence replaces per-PR dated records (owner 19 Oct, Q4) |
| `AGENTS:d19-oct1` | process | in-force | `AGENTS.md:126` | merged into §1 and §5.5 |
| `AGENTS:sept19-d0b` | owner-boundary | superseded | `AGENTS.md:136` | retired: superseded by D25/D30 |
| `AGENTS:mode` | owner-boundary | in-force | `AGENTS.md:138` | kept as charter §1 |
| `AGENTS:d8-learning` | technical | superseded | `AGENTS.md:140` | retired: superseded by D28/D32/D37 |
| `AGENTS:d14-charter` | process | in-force | `AGENTS.md:142` | retired: council (owner 19 Oct, Q5); owner boundaries kept as §5.1 |
| `AGENTS:authority-recovery` | process | in-force | `AGENTS.md:146` | retired: 5, 20 and 30-minute cadence (owner 19 Oct, Q7); reading order becomes CHARTER, NOW.md, line notebook (§5.3) |
| `AGENTS:native-policy` | process | in-force | `AGENTS.md:153` | merged into §3 (agent-execution-policy.json stays as the machine-checked form of §3, regenerated from the charter) |
| `AGENTS:rust` | technical | in-force | `AGENTS.md:161` | merged into §3.11 |
| `AGENTS:primary-mechanisms` | technical | in-force | `AGENTS.md:175` | merged into §1 and §3.1 (priority is not measured advantage: §4.6) |
| `AGENTS:both-lines` | process | in-force | `AGENTS.md:180` | merged into §1 |
| `AGENTS:budgets` | ops | in-force | `AGENTS.md:187` | kept as charter §5.4 |
| `AGENTS:checks` | process | in-force | `AGENTS.md:199` | merged into §5.5 (focused checks at exact head) and §4.3 (development vs held-out) |
| `AGENTS:mech-briefs` | process | in-force | `AGENTS.md:212` | merged into §4.5 and §4.1 (mechanism board) |
| `AGENTS:d22` | process | in-force | `AGENTS.md:218` | merged into §4.1 and §4.3 |
| `AGENTS:three-negatives` | process | in-force | `AGENTS.md:226` | merged into §4.4 |
| `AGENTS:progress-control` | process | in-force | `AGENTS.md:234` | merged into §4.4 |
| `AGENTS:harness-clean-merged` | process | in-force | `AGENTS.md:255` | merged into §5.5 |
| `AGENTS:merge-forward` | process | in-force | `AGENTS.md:279` | kept as charter §5.5 |
| `AGENTS:min-state` | process | in-force | `AGENTS.md:285` | merged into §5.5 (branches deleted after merge) |
| `AGENTS:uor-lab` | process | in-force | `AGENTS.md:290` | kept as charter §5.3 |
| `AGENTS:harness-join` | ops | in-force | `AGENTS.md:300` | merged into §5.3 (onboarding is harness documentation, not charter) |
| `AGENTS:harness-surface` | ops | in-force | `AGENTS.md:314` | merged into §5.3: harness kept as an automatic view; manual 5-minute heartbeat retired (owner 19 Oct, Q7) |
| `AGENTS:architecture` | technical | in-force | `AGENTS.md:342` | merged into §1 (name, not a transformer) and §3.10 |
| `AGENTS:invariant-learning` | technical | in-force | `AGENTS.md:360` | merged into §3.7 |
| `AGENTS:invariant-inference` | technical | in-force | `AGENTS.md:367` | merged into §3.6 and §3.9 |
| `AGENTS:typed-geometry` | technical | in-force | `AGENTS.md:372` | merged into §3.1 |
| `AGENTS:frozen-tla` | technical | in-force | `AGENTS.md:376` | merged into §3.6 (scoped contract of the frozen R4G1 kernel stands; r4g1-graph is PARKED) |
| `AGENTS:artifact-determinism` | technical | in-force | `AGENTS.md:380` | kept as charter §3.12 |
| `AGENTS:errors` | technical | in-force | `AGENTS.md:384` | kept as charter §3.13 |
| `AGENTS:invariant-formal` | process | in-force | `AGENTS.md:387` | merged into §4.6 |
| `AGENTS:ownership` | process | in-force | `AGENTS.md:396` | merged into §2 (active model and serving crates named); full crate map moves to the docs architecture page |
| `AGENTS:codex-lab-hist` | process | superseded | `AGENTS.md:404` | retired: historical record of the 26 Sept Codex lab |
| `AGENTS:deepseek-default` | process | superseded | `AGENTS.md:416` | retired: superseded by D25/D39 (provider choice is the lab's, within §5.4 cost caps) |
| `AGENTS:resources` | ops | in-force | `AGENTS.md:441` | merged into §5.4 |
| `AGENTS:ci-acks` | process | in-force | `AGENTS.md:445` | merged into §5.5 (checks at exact head are posted; a queue acknowledgement is not a test) |
| `AGENTS:delivery` | process | in-force | `AGENTS.md:447` | kept as charter §5.5 |
| `AGENTS:d40` | owner-boundary | in-force | `AGENTS.md:449` | kept as charter §5.1 |
| `AGENTS:historical-manual` | process | in-force | `AGENTS.md:453` | merged into §6 (archived text is evidence, not authority) |
| `AGENTS:e8-shorthand` | technical | in-force | `AGENTS.md:455` | merged into §3.1 |
| `AGENTS:pitfalls` | process | in-force | `AGENTS.md:457` | merged into §5.7 (lessons register) |
| `AGENTS:tmp-build` | ops | in-force | `AGENTS.md:459` | merged into §5.7 (lessons register) and §5.6 (no /tmp data) |
| `AGENTS:ext-authorization` | ops | in-force | `AGENTS.md:462` | merged into §5.4 (standing local allowance extensions stay authorized; record them on the owning issue) |
| `AGENTS:gpu-pods` | ops | in-force | `AGENTS.md:430` | kept as charter §5.4 |
| `AGENTS:storage` | ops | in-force | `AGENTS.md:438` | kept as charter §5.6 |
| `AGENTS:no-keys` | ops | in-force | `AGENTS.md:439` | kept as charter §3.13 |
| `R1` | process | in-force | `docs/harness/pm-rules.md:10` | kept as charter §5.3 |
| `R2` | process | in-force | `docs/harness/pm-rules.md:18` | merged into §5.5 |
| `R3` | process | in-force | `docs/harness/pm-rules.md:26` | retired: manual 5-minute heartbeat (owner 19 Oct, Q7); presence is automatic (§5.3) |
| `R4` | process | in-force | `docs/harness/pm-rules.md:39` | merged into §4.5; scope narrowed to spend-deciding runs: Q2 |
| `R5` | technical | in-force | `docs/harness/pm-rules.md:46` | kept as charter §3.4 |
| `R6` | process | in-force | `docs/harness/pm-rules.md:65` | kept as charter §4.7 |
| `R7` | process | in-force | `docs/harness/pm-rules.md:73` | merged into §5.2 |
| `R8` | process | in-force | `docs/harness/pm-rules.md:82` | merged into §5.5 |
| `R9` | process | in-force | `docs/harness/pm-rules.md:91` | retired: fixed roster is not a rule (owner 19 Oct, Q5); roster lives in NOW.md (§5.2) |
| `R10` | process | in-force | `docs/harness/pm-rules.md:101` | kept as charter §5.5 |
| `R10:cadence` | ops | in-force | `docs/harness/pm-rules.md:140` | merged into §5.5 and §5.6 |
| `R11` | process | in-force | `docs/harness/pm-rules.md:182` | merged into §5.1 (only the owner changes the charter; the PM keeps operating notes); gh CLI usage moves to the harness docs |
| `R12` | process | in-force | `docs/harness/pm-rules.md:204` | merged into §4.2 |
| `R13` | technical | in-force | `docs/harness/pm-rules.md:276` | merged into §3.7 (integer qualification uses exact arithmetic) |
| `R14` | process | in-force | `docs/harness/pm-rules.md:319` | merged into §4.2 |
| `R15` | owner-boundary | in-force | `docs/harness/pm-rules.md:342` | merged into §5.1 |
| `R-queue` | process | in-force | `docs/harness/pm-rules.md:373` | retired: PM-called queue entries (owner 19 Oct, Q7); the merge queue serializes itself (§5.5) |
| `C:principles` | process | in-force | `CONTRIBUTING.md:14` | merged into §5.2, §4.6 and §4.7 |
| `C:humans-start` | process | in-force | `CONTRIBUTING.md:22` | retired: replaced by the new README/docs map from this reorganization |
| `C:workflow` | process | in-force | `CONTRIBUTING.md:47` | merged into §5.3 and §5.5 |
| `C:report` | process | in-force | `CONTRIBUTING.md:70` | merged into §5.6 (one notebook row; no work logs on issues) |
| `C:cadence` | process | in-force | `CONTRIBUTING.md:76` | merged into §5.6: notebook cadence (owner 19 Oct, Q4) |
| `C:harness` | ops | in-force | `CONTRIBUTING.md:114` | merged into §5.3 |
| `C:one-checkout` | process | superseded | `CONTRIBUTING.md:152` | retired: superseded by R10 |
| `C:data-compute` | ops | in-force | `CONTRIBUTING.md:173` | merged into §5.4 and §5.6 |
| `C:claims` | process | in-force | `CONTRIBUTING.md:190` | merged into §4.6 |
| `C:agents` | process | in-force | `CONTRIBUTING.md:200` | merged into §5.3 and §5.4 |
| `C:workflow-change` | process | in-force | `CONTRIBUTING.md:208` | merged into §5.1 (workflow changes are proposed to the owner) |
| `PROTO:github-authority` | process | in-force | `docs/labs/protocol.md:31` | kept as charter §5.2 |
| `PROTO:claim-record` | process | in-force | `docs/labs/protocol.md:49` | merged into §5.3 (claim plus intent note; the long claim schema becomes harness-recorded) |
| `PROTO:work-loop` | process | in-force | `docs/labs/protocol.md:66` | merged into §5.3 |
| `PROTO:cadence` | process | in-force | `docs/labs/protocol.md:86` | retired: manual cadence (owner 19 Oct, Q7); automatic presence (§5.3) |
| `PROTO:expired-lease` | process | in-force | `docs/labs/protocol.md:96` | kept as charter §5.4 |
| `PROTO:socratic` | process | in-force | `docs/labs/protocol.md:105` | merged into §5.7 (method guidance belongs in the lessons register) |
| `PROTO:council` | process | in-force | `docs/labs/protocol.md:122` | retired: council (owner 19 Oct, Q5) |
| `PROTO:council-scope` | process | in-force | `docs/labs/protocol.md:133` | retired: council scope (owner 19 Oct, Q5) |
| `PROTO:experiments` | process | in-force | `docs/labs/protocol.md:144` | merged into §4.3 and §4.6 |
| `PROTO:staged-parity` | process | in-force | `docs/labs/protocol.md:154` | merged into §4.5 (freeze data, seeds, tolerance before running) |
| `PROTO:status-vocab` | process | in-force | `docs/labs/protocol.md:171` | merged into §4.6 |
| `PROTO:review-classes` | process | in-force | `docs/labs/protocol.md:181` | merged into §5.5 (one review rule for all PRs; results also need sealed evidence §4.6) |
| `PROTO:delivery` | process | in-force | `docs/labs/protocol.md:194` | merged into §5.5 |
| `PROTO:versioned-authority` | ops | in-force | `docs/labs/protocol.md:208` | merged into §5.3: adopt_policy hold retired, claims record the charter version only (owner 19 Oct, Q7) |
| `SG:harness-start` | process | in-force | `docs/labs/session-goal.md:10` | merged into §5.3 |
| `SG:contract-1` | process | superseded | `docs/labs/session-goal.md:53` | retired: superseded by D39:self-direction |
| `SG:contract-2` | owner-boundary | superseded | `docs/labs/session-goal.md:55` | retired: superseded by D32/D37/D39; holds now in §5.1 |
| `SG:contract-3` | technical | superseded | `docs/labs/session-goal.md:58` | retired: superseded by D32/D37; bans now in §3.2/§3.3/§3.5 |
| `SG:contract-4` | process | superseded | `docs/labs/session-goal.md:62` | retired: superseded by D32/D39 (OP plan finished or overtaken) |
| `SG:contract-5` | process | in-force | `docs/labs/session-goal.md:69` | merged into §5.4 and §5.5 |
| `SG:contract-6` | process | in-force | `docs/labs/session-goal.md:72` | merged into §4.4 and §4.1 |
| `SG:contract-7` | process | in-force | `docs/labs/session-goal.md:78` | merged into §4.5 and §3.10 |
| `SG:contract-8` | process | in-force | `docs/labs/session-goal.md:84` | merged into §5.5; the "two preparation PRs per cycle" cap is retired under §4.5 (owner may restore in Q2) |
| `SG:contract-9` | process | in-force | `docs/labs/session-goal.md:91` | retired: manual cadence contract (owner 19 Oct, Q7) |
| `SG:status-card` | process | in-force | `docs/labs/session-goal.md:145` | retired: manual status card cadence (owner 19 Oct, Q7) |
| `SG:common-prompt` | process | superseded | `docs/labs/session-goal.md:110` | retired: superseded by D32/D37/D39; the charter is the session prompt |
| `SG:lab-addenda` | process | superseded | `docs/labs/session-goal.md:156` | retired: superseded by D24:cancel/D39:self-direction |
| `SG:finish-line` | process | in-force | `docs/labs/session-goal.md:149` | merged into §4.6 (a held source cycle is not milestone evidence) |
| `SG:wind-down` | process | in-force | `docs/labs/session-goal.md:226` | merged into §5.4 (release only your own resources; do not stop a healthy run) |
| `POLICY:three-lab-org` | process | superseded | `docs/integration/agent-execution-policy.md:122` | retired: superseded by D14 |
| `POLICY:class-abc` | process | superseded | `docs/integration/agent-execution-policy.md:188` | retired: superseded by D14/D19 |
| `POLICY:gates-promote` | process | in-force | `docs/integration/agent-execution-policy.md:180` | merged into §4.1 |
| `POLICY:evidence` | process | in-force | `docs/integration/agent-execution-policy.md:164` | merged into §4.6 |
| `POLICY:shared-machine` | ops | superseded | `docs/integration/agent-execution-policy.md:216` | retired: superseded by D19/D24/D26 |
| `POLICY:shared-vectors` | technical | in-force | `docs/integration/agent-execution-policy.md:57` | kept as charter §3.12 |
| `POLICY:architecture` | owner-boundary | in-force | `docs/integration/agent-execution-policy.md:235` | merged into §1, §3.6 and §3.9 (expert gating stays a conditional future option, not adopted) |
| `POLICY:learning-budget` | ops | in-force | `docs/integration/agent-execution-policy.md:293` | merged into §5.4 |
| `POLICY:progress-control` | process | in-force | `docs/integration/agent-execution-policy.md:327` | merged into §4.4 |
| `POLICY:verification` | process | in-force | `docs/integration/agent-execution-policy.md:384` | merged into §5.5 |
| `HR:no-squash` | technical | in-force | `docs/contributing/HARD-RULES.md:11` | kept as charter §3.2 |
| `HR:no-matmul` | technical | in-force | `docs/contributing/HARD-RULES.md:12` | kept as charter §3.3 |
| `HR:banned-module` | technical | in-force | `docs/contributing/HARD-RULES.md:13` | kept as charter §3.4 |
| `HR:gpu-only` | technical | in-force | `docs/contributing/HARD-RULES.md:14` | kept as charter §3.5 |
| `HR:serving-integer` | technical | in-force | `docs/contributing/HARD-RULES.md:15` | kept as charter §3.6 |
| `HR:float-offline` | technical | in-force | `docs/contributing/HARD-RULES.md:16` | kept as charter §3.7 |
| `HR:rust-default` | technical | in-force | `docs/contributing/HARD-RULES.md:17` | kept as charter §3.11 |
| `HR:no-artifacts` | technical | in-force | `docs/contributing/HARD-RULES.md:18` | kept as charter §3.13 |
| `HR:no-credentials` | technical | in-force | `docs/contributing/HARD-RULES.md:19` | kept as charter §3.13 |
| `HR:claims` | process | in-force | `docs/contributing/HARD-RULES.md:20` | kept as charter §4.6 |
| `HR:delivery` | process | in-force | `docs/contributing/HARD-RULES.md:21` | kept as charter §5.5 |
| `HR:archive` | process | in-force | `docs/contributing/HARD-RULES.md:22` | kept as charter §4.7 |
| `HR:external-compute` | ops | in-force | `docs/contributing/HARD-RULES.md:23` | merged into §5.4 |
| `OUT:no-harness` | process | in-force | `docs/contributing/OUTSIDE.md:16` | kept as charter §5.3 |
| `OUT:start-here` | process | in-force | `docs/contributing/OUTSIDE.md:19` | merged into §5.3 (outside contributor guide stays in docs/contributing) |
| `OUT:intent-note` | process | in-force | `docs/contributing/OUTSIDE.md:31` | merged into §5.3 |
| `KIMI:lead-supplement` | process | superseded | `.kimi-code/AGENTS.md:3` | retired: superseded by D14/R9 |
| `KIMI:d37` | technical | in-force | `.kimi-code/AGENTS.md:29` | merged into §3.7 |
| `KIMI:worktree` | process | in-force | `.kimi-code/AGENTS.md:31` | merged into §5.5 |
| `KIMI:team` | process | superseded | `.kimi-code/TEAM.md:3` | retired: superseded by D14 |
| `KIMI:research-protocol` | process | in-force | `.kimi-code/RESEARCH-PROTOCOL.md:12` | merged into §4.1 and §4.4 |
| `KIMI:tools-protected` | process | superseded | `.kimi-code/TOOLS-AND-STORAGE.md:23` | retired: superseded by C:workflow/R11 |
| `KIMI:storage` | ops | in-force | `.kimi-code/TOOLS-AND-STORAGE.md:31` | merged into §5.6 and §5.4 (standing local allowance) |
| `CODEX:fourth-lab` | process | superseded | `.codex-lab/README.md:15` | retired: superseded by D14/D19 |
| `CODEX:offline-matmul` | technical | superseded | `.codex-lab/README.md:22` | retired: superseded by D25/D30 |
| `CODEX:expert-bench` | process | in-force | `.codex-lab/README.md:59` | merged into §5.2 (subagents for review, adversarial tests and research) |
| `CODEX:coordination` | process | superseded | `.codex-lab/README.md:90` | retired: superseded by PROTO:claim-record/R1 |
| `CODEX:resource` | ops | in-force | `.codex-lab/README.md:147` | merged into §5.4 and §5.5 |
| `CODEX:no-daemon` | process | in-force | `.codex-lab/README.md:166` | merged into §5.4 (handoff records owned work and next action) |
| `OWNER:0` | owner-boundary | in-force | `memory/owner-direction-lock.md:11` | kept as charter §1 and §3.1 (verbatim) |
| `OWNER:1` | owner-boundary | in-force | `memory/owner-direction-lock.md:17` | kept as charter §3.2 |
| `OWNER:2` | owner-boundary | in-force | `memory/owner-direction-lock.md:18` | kept as charter §3.3 |
| `OWNER:3` | owner-boundary | superseded | `memory/owner-direction-lock.md:19` | retired: superseded by D25:no-matmul-anywhere/D30 (now §3.3) |
| `OWNER:4` | owner-boundary | in-force | `memory/owner-direction-lock.md:20` | kept as charter §3.5 |
| `OWNER:5` | owner-boundary | in-force | `memory/owner-direction-lock.md:21` | kept as charter §1 (lead metric) |
| `OWNER:6` | technical | in-force | `memory/owner-direction-lock.md:22` | merged into §3.1 |
| `OWNER:7` | owner-boundary | in-force | `memory/owner-direction-lock.md:23` | merged into §1 and §3.1 (flag and replace every non-geometric or expensive mechanism) |
| `OWNER:8` | process | superseded | `memory/owner-direction-lock.md:24` | retired: superseded by D39:self-direction (§5.3); approved sequences still bind (§5.8) |
| `OWNER:9` | process | in-force | `memory/owner-direction-lock.md:26` | kept as charter §5.1 |
| `OWNER:10` | process | in-force | `memory/owner-direction-lock.md:27` | kept as charter §5.1 |
| `OWNER:11` | owner-boundary | in-force | `memory/owner-direction-lock.md:28` | kept as charter §5.1 |
| `OWNER:12` | technical | superseded | `memory/owner-direction-lock.md:30` | retired: superseded by D32; G1 (relu squared) stays excluded by §3.1 |
| `OWNER:13` | process | superseded | `memory/owner-direction-lock.md:31` | retired: superseded by D32 (freeze ended) |
| `OWNER:14` | technical | in-force | `memory/owner-direction-lock.md:32` | kept as charter §3.3 |
| `OWNER:15` | process | superseded | `memory/owner-direction-lock.md:33` | retired: superseded by D32 (build order done or overtaken) |
| `OWNER:16` | ops | in-force | `memory/owner-direction-lock.md:34` | merged into §5.4 |
| `OWNER:17` | ops | in-force | `memory/owner-direction-lock.md:36` | kept as charter §5.4 |
| `OWNER:18` | process | one-off | `memory/owner-direction-lock.md:37` | retired: one-off (13 Oct blocker fixes) |
| `OWNER:19` | ops | in-force | `memory/owner-direction-lock.md:39` | merged into §5.4 |
| `OWNER:20` | owner-boundary | in-force | `memory/owner-direction-lock.md:40` | kept as charter §3.5 and §3.11 |
| `OWNER:21` | process | in-force | `memory/owner-direction-lock.md:42` | merged into §5.2 and §5.5; lab roles go to Q5 |
| `OWNER:22` | process | in-force | `memory/owner-direction-lock.md:48` | merged into §5.8 (D34 sequence) |
| `OWNER:23` | process | superseded | `memory/owner-direction-lock.md:50` | retired: superseded by D35, then D39 |
| `OWNER:24` | owner-boundary | superseded | `memory/owner-direction-lock.md:52` | retired: superseded by D37 |
| `OWNER:25` | technical | in-force | `memory/owner-direction-lock.md:55` | kept as charter §3.7 and §3.8 |
| `OWNER:26` | owner-boundary | in-force | `memory/owner-direction-lock.md:60` | kept as charter §1 (verbatim) and §5.2 (subagents) |
| `OWNER:27` | process | one-off | `memory/owner-direction-lock.md:61` | retired: one-off (row-parallel kernel built) |
| `OWNER:28` | process | time-boxed | `memory/owner-direction-lock.md:64` | merged into §5.2 (PM cover when Kimi is offline); time-boxed, confirm in Q5 |
| `OWNER:29` | process | one-off | `memory/owner-direction-lock.md:69` | retired: one-off (Dose 1 stopped and rerun) |
| `OWNER:30` | owner-boundary | one-off | `memory/owner-direction-lock.md:72` | retired: one-off release, executed |
| `OWNER:31` | process | one-off | `memory/owner-direction-lock.md:73` | retired: one-off (K16-A vs K16-R run) |
| `OWNER:32` | process | time-boxed | `memory/owner-direction-lock.md:76` | merged into §5.2 (PM cover when Kimi is offline); time-boxed, confirm in Q5 |
| `OWNER:memory-no-transformer` | owner-boundary | in-force | `memory/MEMORY.md:12` | kept as charter §3.10 |
| `OWNER:no-nvrtc` | technical | in-force | `memory/MEMORY.md:32` | kept as charter §3.4 |
| `OWNER:memory-owner-writes-release` | owner-boundary | in-force | `memory/MEMORY.md:37` | kept as charter §5.1 |
| `OWNER:memory-no-adhoc-python` | process | in-force | `memory/MEMORY.md:34` | merged into §3.11 |
| `OWNER:memory-no-technicality` | process | in-force | `memory/MEMORY.md:36` | merged into §4.2 |
| `OWNER:memory-proceed-autonomously` | process | in-force | `memory/MEMORY.md:15` | merged into §5.3 (continue unless redirected; pending owner questions still mean wait, §5.1) |

## Tally

- kept: 74
- merged: 111
- owner decision: 16
- retired: 57
