# Claude #1533 check reconciliation — read-only evidence

**Disposition:** preserve `UNKNOWN`, the reserved attempt and both admission indicators. Do not run the currently installed `reconcile-stopped`: its boot-identity predicate can falsely establish a reboot. At the current observation there is no live bound worker; the only original group member is zombie PID38331. A repaired tool may establish a separate stopped reconciliation from fresh positive process evidence. Neither the original runner receipt nor the check results should be changed into PASS.

Observed 2026-09-30T04:39:07.206233+00:00. Source HEAD `9efa5f6ddda1d8f58c3aee1e7237a607a2ce731d` remains clean in `/Users/casey.allard/uor-r4/.worktrees/lab-claude-director`. Bound `/bin/zsh` and script SHA-256 both match the immutable job specification. All evidence hashes and scope are in `claude-check-reconciliation.json` alongside this file. No source edit, rerun, signal, hold change, coordination transition or sealed S8-panel content read was performed.

## Runner outcome versus completed checks

Attempt `claude-1533-checks-e2-a1`, host `host-284b64fb2f70567aee465dc4941430076137e580d19d9568b15726d14b157cec`, source9efa, claim Claude epoch2. The durable original `exit.json` records outcome/process_state `unknown`, null exit status, reason `identity lost; no signal sent`, ended04:05:47Z, measured elapsed281,968ms and peak RSS2,108,320KiB. Its SHA-256 is `cc677aeb9e2ca2820fa69ee3ca779c62ad20a3665c38b9678a436d35eda38716`. Exactly one corresponding original execution charge exists. Preserve its bytes and charge identity.

The sequential check script continued after that receipt. Its summary finished at04:08:38.307048Z, and the runner's stdout contains all nine returned command records. The script performs no background submission; each command is awaited before the next. Eight command statuses are0; `training-geometric-stack` is101. Summed coarse per-command durations are449seconds, not a substitute for a measured complete runner duration. The script's final predicate would return1 given that summary, but no authoritative payload wait/exit receipt survives; report this as script semantics and logged check failure, not a recovered actual process exit code.

| Check | Recorded result |
|---|---|
| fmt | exit0 |
| lut library | exit0;22 passed |
| geometric_stack library filter | exit101;30 passed,2 failed,1 ignored;275 filtered |
| stack_export library filter | exit0;9 passed |
| stack_d11_snap integration | exit0;2 passed |
| d4_map_codec_adapter | compile-only exit0; not executed |
| training bins/examples | cargo check exit0; not behavioral execution |
| diff check | exit0 |
| claim wording | exit0 |

## Two concrete check failures and minimum corrections

1. `a_served_and_snapped_model_saves_both_records_for_explicit_reapplication`, `crates/uor-r4-training/src/geometric_stack.rs:6579–6625`, fails at6619. The old test assumes load restores neither mode, while the new intended loader at1118–1152 restores the saved transport snap but deliberately leaves the served codec unset. Correct the test to assert `served=None`, `transport=Some(Icosian)`; explicitly disable transport for the raw-float comparison, and reapply saved snap plus served codec for the original combined-forward equality. Preserve the original distinctions and loader regression checks. This is a stale test contract after the snap-loader fix, not evidence that snap restoration failed.
2. `the_d10_engine_refuses_a_snapped_export_read_for_reference`, geometric_stack.rs:6512–6533, errors with `inconsistent or unsupported stack shape` before reaching the intended constructor rejection. It calls `tiny(...)` at6514: that fixture has width16 (4788–4800), while the served artifact's `StackShape::validate` in `crates/uor-r4-lut/src/format.rs:229–246` requires width divisible by GROUP32. Use the existing exportable fixture (geometric_stack.rs:5089–5102, width64) or another explicitly valid served shape, preserving integer parse success and the exact parse_for_reference→from_artifact refusal assertion. Do not weaken shape validation or accept any unrelated earlier error as proof of the D10 guard.

Only the affected focused checks and applicable source/format validation are needed after these corrections; the existing unchanged check evidence remains scoped to9efa. No model training, corpus rerun or sealed-panel evaluation is justified by these two failures.

## Root cause and runner safety defects

Saved boot marker: `{ sec = 1790723628, usec = 114456 } Tue Sep 29 19:13:48 2026`.
Current marker: `{ sec = 1790723628, usec = 41370 } Tue Sep 29 19:13:48 2026`.
The seconds and formatted boot time agree while microseconds changed. The same daemon PID24197 remains and original supervisor PID38331 retains its original00:01:09 start, now as a zombie. The captured markers demonstrate a full-string mismatch without needing to invent a process restart. Current macOS also exposes `kern.bootsessionuuid=763E5ACF-75D5-44F0-9384-F80B31312894`; it was not stored in the old identity and cannot be retroactively claimed as a captured old UUID.

- **P1, boot identity:** `tools/lab-runner/src/process.rs:64–68` uses full `kern.boottime`; `matches` at109–110 rejects a microsecond adjustment. More seriously, `owned_members` at148–151 interprets any mismatch as a verified reboot and returns an empty group. In a still-live job this can let reconciliation falsely certify stopped ownership. Replace new captures with a stable boot-session UUID, version/identify the field, and handle old timestamp identities conservatively. Do not merely drop microseconds and treat timestamp disagreement as reboot proof. Legacy reconciliation must use fresh positive stopped/no-live-member evidence or remain unknown.
- **P1, live-path destruction on uncertainty:** `daemon.rs:537–547` immediately finalizes unknown on identity mismatch; `jobs.rs:637–674` moves the running directory to done regardless of unknown process liveness. The still-running wrapper was given absolute paths under running/. Actual stderr at04:08 reports both missing `running/.../payload.exit` and missing `running/.../supervisor-hold.fifo`. This destroys the final-status path and anchor mechanism while the workload is alive. Hold admission and retain runtime paths/tracking while state is unknown; move only after positive stopped evidence, preserving the original uncertain receipt. Retain/reap the known child handle safely so an unknown observation does not leave an unreaped zombie. This repair should remain scoped to the observed lifecycle failure.

Necessary runner regressions: (a) time-of-day/boot-time text drift with stable boot-session UUID does not lose an owned live worker; (b) actual changed boot UUID and PID reuse cannot adopt/signal another process; (c) legacy boot-marker mismatch with a live group remains unknown and cannot return empty as reboot proof; (d) injected identity observation failure mid-job keeps payload.exit/FIFO paths usable, holds admissions and prevents duplicate launch/charge; (e) eventual known exit is reaped, original UNKNOWN preserved, and stopped reconciliation is separately bound/idempotently charged. These require bounded inert fixtures only, after protected source correction and appropriate admission; none was executed in this audit.

## Current safety evidence and takeover packet

At 2026-09-30T04:39:07.206233+00:00, the complete process-group observation was:
`38331 24197 38331 Z Wed Sep 30 00:01:09 2026 <defunct>`.
No descendants through current parent links, no live Cargo/rustc or executable in the bound target directory, and no open handles beneath the job/check-output/cache paths were observed. This supports current stopped status independently of the flawed boot-string predicate; repeat the observation immediately before repaired reconciliation. It does not retrospectively recover the exact payload exit status.

Shared policy remains `ba266ad44f21c9df2dafa9d56a1e2051c1565d91`. Claude is unavailable; issue1533 epoch2 remains claimed but expired at2026-09-30T04:20:49+00:00, checkpoint null, and its attempt remains reserved. Preserve source/work card and all results. A successor may claim under current policy with this hashed packet after refreshing liveness and native issue blockers, but must adopt/reconcile the existing attempt rather than resubmit it. Use the repaired `reconcile-stopped` only after its positive stop/legacy identity path is validated; keep the original UNKNOWN and use the generated separate reconciliation receipt for `finish_attempt`. The existing reconciliation implementation charges a conservative estimate at least wall3600s+5s grace (3,605,000ms total), less the281,968ms original debit, not a new measured duration. Preserve that estimated distinction or review a successor accounting policy explicitly.

`admission-blocked.json` contains an earlier04:01:09 exclusive-lane message; `admissions-held.json` is the04:05:47 lost-identity hold. Neither was cleared. Continue independent documentation/source review under holds. Claim recovery, stopped reconciliation, any source fixes, new executed checks and later admission release remain root-owned next actions.
