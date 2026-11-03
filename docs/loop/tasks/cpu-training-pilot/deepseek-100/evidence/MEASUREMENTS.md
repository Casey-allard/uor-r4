# Public corpus worker-budget readings

Host: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.
Repaired driver SHA-256:
`51d6c1b4c543e24bd5bb1b0fa2b769efb1554521891935e4ddba309a17903ffd`.
Configuration: public dual-c131 seed 1, rank-margin seed 1, 16 update attempts,
evaluation stride 2, panel cap 1,024 and checkpoint interval 16.

## One run at each worker count

The forward and reverse columns are independent process runs with the order of
worker counts reversed for the second invocation.

| Workers | Forward wall ms | Reverse wall ms | Speedup vs one (forward) | Speedup vs one (reverse) | Average CPU |
|---:|---:|---:|---:|---:|---:|
| 1 | 62,044.786 | 62,248.336 | 1.000x | 1.000x | 99% |
| 8 | 8,482.573 | 8,474.837 | 7.314x | 7.345x | 758-760% |
| 16 | 4,639.734 | 4,629.612 | 13.372x | 13.446x | 1,440-1,451% |
| 32 | 3,533.861 | 3,521.686 | 17.557x | 17.676x | 2,751-2,761% |

Peak RSS was 53,244-53,608 KiB at one worker, 67,264-67,496 KiB at eight,
86,436-86,444 KiB at 16 and 124,324-124,688 KiB at 32.

## Forward-order stages

| Workers | Setup | Baseline panel | Trainer setup | Training | Final panel | Gate |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 5.004 | 30,858.883 | 10.555 | 285.534 | 30,828.185 | 0.007 |
| 8 | 4.692 | 4,055.515 | 10.909 | 284.407 | 4,062.162 | 0.005 |
| 16 | 4.402 | 2,134.261 | 12.223 | 282.763 | 2,143.670 | 0.005 |
| 32 | 4.270 | 1,593.860 | 11.291 | 287.848 | 1,574.148 | 0.012 |

| Workers | Final checkpoint + reload | Served-forward audit | Publish + reload | Post-publish check | Unattributed |
|---:|---:|---:|---:|---:|---:|
| 1 | 13.505 | 35.305 | 7.031 | 0.775 | 0.003 |
| 8 | 12.556 | 34.944 | 16.610 | 0.771 | 0.002 |
| 16 | 13.054 | 35.098 | 13.478 | 0.777 | 0.002 |
| 32 | 12.957 | 35.342 | 13.359 | 0.771 | 0.003 |

All values are milliseconds. The two full-panel comparisons dominate wall time;
the online update chain and the recurrent 64-position served-forward audit are
dependency-ordered and remain effectively constant across worker counts.
Trainer setup, gate construction, audit computation, atomic publication/reload,
post-publish verification and the unattributed remainder are separate stages.

## State equality

All eight single-artifact runs produced:

- final artifact SHA-256
  `6b35fed04be8a98168b7a7a0a22030300de5383b7576c0c06a812f28e7bb38c8`;
- final checkpoint SHA-256
  `b3a9c501c42d94f88091f4360a9b0a67dc8d294ebf5bdbb0d58b98de20e6ce24`;
- panel `278 / 384 / 396` top-1/top-8/admitted;
- served-forward decision SHA-256
  `2ac6b0971afe222860453846f286385428bb870e802fccf66a6097965ea1b897`;
- zero query-coordinate, token or kept-cell mismatches over 262,144 Q12
  coordinates and 8,192 kept cells.

The state comparison is reproducible with the retained reports:

```sh
for workers in 1 8 16 32; do
  jq -cS '{panel_baseline,panel_final,accepted:.run.accepted_updates,
    artifact:.final_artifact.sha256,checkpoint:.final_checkpoint.sha256,
    audit:(.served_forward_audit | del(.path))}' "w${workers}.json"
done | sort -u | wc -l
```

The result is `1`: all four worker budgets produced the same compared state.

## Independent two-artifact scheduling

Two public artifact seeds were compared with the same 16-update work:

- two sequential 32-worker jobs: 7.06 s wall, 2,746% average CPU, 124,664 KiB
  peak RSS;
- two concurrent 16-worker jobs: 6.60 s wall, 2,937% average CPU, 89,304 KiB
  maximum per-process peak RSS.

The concurrent schedule was 6.5% faster but used two simultaneous process
footprints. Seed 1 trained and seed 2 accepted no update; each output matched
its corresponding sequential run exactly.

## Recommendation

Use 32 Rayon workers for a single panel-heavy CPU corpus run on this host. When
wall time is the only objective and two or more independent artifacts are
ready, run at most two 16-worker corpus jobs concurrently. Keep the online
rank-margin chain and recurrent served-forward audit serial; splitting them
would change dependency order rather than expose independent work.
