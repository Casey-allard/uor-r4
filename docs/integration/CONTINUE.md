# Continue UOR-R4 geometric model research

Read the [matched query-read result](query-read-result-2026-09-20.md) and its
[evidence receipt](../evidence/native_geometric_query_read_2026-09-20.txt), then refresh
origin/main, canonical plan/current state, live issues and resources.

## Where the programme stands

Corrected empirical E (`565cf98a…`, `head-projection-3/corrected/empirical.cpl2`) remains the
numerical incumbent at **7.170815718** bits/target and was reproduced exactly. The **[matched
geometric query read](query-read-result-2026-09-20.md) is executed and is a valid negative**: three
equally parameterized arms were fitted for 512 batch-8 updates each, exported as new `CPX3`
artifacts, reloaded and evaluated on the exact 36-document / 288-window / 17,342-target panel.

| Arm | micro CE | Gain over E (nominal 95% paired) |
| --- | ---: | --- |
| E | 7.170815718 | — |
| Q, joint `q*b` older/query read | 7.141505532 | +0.029310 [0.022866, 0.036005] |
| **S, separable older + query read** | **7.032227740** | **+0.138588 [0.125831, 0.150844]** |
| L, local-only matched read | 7.101049449 | +0.069766 [0.058944, 0.080294] |

Q fails the predeclared 0.10-bit practical screen, is 0.109278 bits worse than its separable
control S and 0.040456 worse than L, and shows no older-content sensitivity (donor penalty
−0.003830) and no query-route evidence (+0.000322). S's query channel *is* used (+0.190660). All
instrument checks pass; no candidate is promoted; every arm's generation remains degenerate.

## The one next task

**Continue from the separable read S.** Build one bounded mechanism on it and qualify it against a
**new independent behaviour panel**:

1. **Selective write/reset** — the current update is a bijective right product and supplies no
   forgetting. Add learned, typed write/reset semantics to the separable read and measure whether
   older content becomes usable rather than merely present.
2. **Or exact occurrence/version access** — reuse the preserved occurrence/version memory contracts
   so a selected record, not a lossy 120-state product, carries the content.
3. **Or a capacity question** — test whether the binding constraint is older-*history volume*
   rather than the query arrangement, at the same declared cost.

Do not rebuild the multiplicative read, start a projection/width/precision sweep, or fetch a new
corpus. Selective write/reset, exact memory and shared composition remain separate obligations;
so do useful conversation, executed Rust behaviour on one accepted artifact, and complete
consumer-machine cost. D0-b, the geometric priority and prime/zeta/R4/S3/H4/icosian roles are
unchanged. `#973`/`#820`/`#963`/`#964` remain open.

## Corrections already delivered

- `learner/head_projection.rs::project_row` sweeps the nearest-code seed at **every** admissible
  shift including `s0`; covered by a correlated-Gram regression and confirmed by reproducing the
  defect. The complete originally specified projection is still **NOT_RUN**; no QG campaign was
  rerun and no old report byte changed.
- `bin/head-projection.rs` reports `singularity: "NOT_MEASURED"` with
  `singular_inputs_supported: true` instead of a hardcoded rank claim.

## Resources

The standing-authorized **+7200000 ms** extension to **178100000 ms** was recorded in the live
ledger **before** consumption. This step charges **5900000 ms** once (four runner executions
including the two superseded probes and the superseded complete run, compile/test cycles including
a 636.5 s broad learner run, and a documentation/delivery allocation). **New cumulative
176138565 / 178100000 ms, remaining 1961435 ms.** Retained: 35,737,680 bytes for `query-read-2` plus
the preserved superseded `query-read-1`; peak RSS 763,002,880 B; the 128 MiB storage stop margin is
intact; no deletion, corpus download or paid/external compute. Reuse the retained final artifacts for
derived reporting rather than re-fitting.

Deliver through a protected PR, verify the actual merged tree, and update the owning issues and any
existing project items.
