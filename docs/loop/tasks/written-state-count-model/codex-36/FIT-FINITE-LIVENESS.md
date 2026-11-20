# [codex] Independent FIT for the revised C128 spool

**FIT PASS for one finite C128 split replay with the revised Wick binary**
under a separate GO and device lock. The CUDA count source, sm_89 cubin,
disassembled instructions and input are byte-identical to the original
[C128 spool FIT](FIT-FINITE.md). I reviewed the changed admission calculation
and the two fallible CUDA buffer clones as their non-author.

| Exact revised binding | SHA-256 |
|---|---|
| Executable | `d7687d9fb76fb1d54abf22c62d51521b2ab14b298c3ff76cdab84ac61935b3f0` |
| [Source manifest](full-fit/revised/source.sha256) | `f526c5ce9a4fea596ea29215d2bf512e6c3b404cae5290e800509e8608597d96` |
| [Build manifest](full-fit/revised/build.sha256) | `6a637704434fc4287269bef06f8067fa549f33316b8c853b36d5e1f73e7c98be` |
| [C128 plan](full-fit/revised/plan-c128.json) | `18c32d1820a2fcbda82f1505d01e49645228ad32a99fba93f42b64384c1331ae` |
| [Full C plan](full-fit/revised/plan-c.json) | `a2d032239fd0bcba5e0f1612dab182fddd01a6b9b1fdaf23e2e03fd6a0e6562e` |

The executable computes the largest simultaneous typed buffers across named
CUDA phases. The first allocation proof used 72 bytes per target plus a
selector reserve. The reviewed source uses the exact upper bound for each
phase and takes their maximum. At full C the phase that emits context records
dominates. Its bound is 50 bytes per distinct gram plus 20 bytes per distinct
context and 36 fixed bytes. Each distinct count is bounded by the number of
admitted targets. The selector's full scan window belongs to the earlier
selection phase. Cudarc's counted clones allocate and copy on the same stream
as later use and release. The two changed sites propagate CUDA allocation errors.
[Allocation and allocator review](full-fit/72-byte-bound.md)

This gives the full-C plan 22,797,060,244 logical device payload bytes and
23,333,931,156 required free bytes with the unchanged 512 MiB reserve. The
revised C128 plan instead peaks during selection at 269,262,526 logical bytes
and requires 806,133,438 free bytes. Both range schedules, TRAIN and census
identities match the original plans. The [earlier full-C resource HOLD](FIT-FULL.md)
preserves the failed admission calculation and its device measurement.

The finite replay must match the original physical C128 artifact byte for byte,
all 22 census rows, and the existing 29 nonempty ranges per order with maximum
6,492 targets. It must report the revised plan, required bytes, post-context
free memory and the same pinned real-device count cubin. Verify cold native
identity against the archived bytes after the spool seals. A readback or
resource discrepancy requires its own record. Passing the finite replay
establishes eligibility for a new full-C launch decision using exact live
post-context headroom and the reviewed full plan.
