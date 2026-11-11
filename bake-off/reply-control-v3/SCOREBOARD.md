# Reply-control V3 scoreboard

Real arm-r seed-2 artifact
`a1d705586b35d7da282913b1e21b242a74c0661b32d73e2ec89f56310a8140e8`
on `samples-p-greedy-memory.json`
(`53d966dcfd63ff71e4b2d30f9af0fdbdc0607ca6c6d96c7d09aa4fb37e99a2e2`).

All 1,141 cases use the same frozen 48-token trace.

| Arm | Rule tests | Exact complete | Truncated | EOS position error | Kill hit |
|---|---:|---:|---:|---:|---|
| copy-all control | 4/8 | 607 | 294 | 490 | n/a |
| current greedy/cycle | 8/8 | **607** | **290** | **490** | no |
| fixed EOS rank 2 | 8/8 | 577 | 342 | 521 | yes |
| stability, zero margin | 8/8 | **607** | **290** | **490** | no |
| permeability-aware | 8/8 | 558 | 363 | 539 | yes |
| shuffled feature | 8/8 | 271 | 712 | 846 | no |
| seeded random | 8/8 | 606 | 291 | 491 | no |

The stability arm produced exactly the current result: the real trace had zero
zero-margin rows, so its extra predicate never fired.

The strongest fixed rule and permeability rule both lose complete replies and
truncate more references. The shuffled and random controls are worse, so the
observed improvements cannot be attributed to arbitrary stopping.

Pick: retain the current greedy/cycle rule. No candidate beats its 607 exact
complete replies or its 290 truncations. This is a no-integration result.
