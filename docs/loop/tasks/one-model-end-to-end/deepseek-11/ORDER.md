# ORDER: one-model-end-to-end, deepseek-11

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `0c76760e8a99e13c2a5f2ee92db03d871c5ddb90`.

## Goal

Consume the merged `CpuDiscreteTrunkReader` in the CPU-only `m2-evidence`
instrument. The public trained 19-table M1 report currently records full CPU
engine parity as unavailable and checks only the writer from the serving Q12
query. Replace that boundary with exact independent CPU query, admitted-cell,
top-8 and token comparison while retaining the four-table reader.

## Cancel probe

1. `CpuDiscreteTrunkReader` is merged as `0c76760e8` and passes the complete
   public 14,287-row DEV parity check.
2. `m2-evidence::observe_cpu_engine_parity` still explicitly accepts only
   `uor-r4.lut-discrete/1` with four tables.
3. The existing public smoke has the 19-table artifact, a 616-target panel,
   its generated control, and an exact writer-only report to compare against.
   No duplicate report or newer implementation answers this question.

## Files in scope

- `docs/labs/one-model-end-to-end/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-11/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. Four-table artifacts still use `CpuDiscreteProposalScorer`.
2. Valid 19-table artifacts use `CpuDiscreteTrunkReader`, checking every
   complete Q12 query, admitted-cell vector, top-8 and token against
   `IntegerDiscreteModel`.
3. Unsupported artifact schemas remain explicitly unavailable rather than
   being treated as parity.
4. Focused tests cover both dispatch paths and malformed/unsupported refusal.
5. The existing public 616-target smoke reruns with 19,712 query coordinates,
   zero mismatches and no `independent_cpu_trunk_query` unavailable entry.
6. The committed evidence JSON, README, REPORT and notebook row state the exact
   instrument boundary and keep complete-reply, fresh-phrasing, training and
   GPU qualification open.

## Stop if

- The four-table or 19-table CPU reader differs from integer serving.
- The 19-table reader cannot be used without changing model, training or
  serving source.
- The public panel, control or artifact identity differs from its prior smoke.
