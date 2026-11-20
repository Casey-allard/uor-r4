# [codex] Independent FIT for the D36 C128 split spool

**FIT PASS for one C128 finite CUDA spool on Wick**, conditional on the lab's
separate GO recording the active device lock and a fresh private output path.
I reviewed the spool producer as its non-author. It supplies the full-dose P1(b)
count path with a small multi-partition proof, using the already frozen C128 GPU
count file as an exact oracle. P1(b) still requires the actual pruned full-dose
state, an R-only fit and a separately fitted and measured paired DEV-test result.
This FIT covers TRAIN count materialization and finite replay.

| Dispatch binding | Value |
|---|---|
| Host and device | Wick RTX 4090, `GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41`, PCI `00000000:01:00.0`, driver 595.99.02, real target `sm_89` |
| Full spool executable SHA-256 | `7a77f3d29430c12016026a0a051c57bca672b3831cdd2e05fd70237ef4605e66` |
| Source manifest SHA-256 | `623b8f4dfb91cf0cef003ef253f242fd99315ed18332a1b1843b8a632b54d22a` |
| AOT image manifest SHA-256 | `df69021f705959b5fe2d34ee7cd119aaa8f0432190ae4412bb1e631b132f9d73` |
| Count CUDA source SHA-256 | `e693ef6a51d14ac4c04559aa564ec648ffb3f9b1ae8aad25c400cfd06eae8f0b` |
| Count derived PTX SHA-256 | `6bc5fa5841764d531e0efcad5310a19915c80fdbd1ab47c15066d80112ea4666` |
| Loaded count cubin SHA-256 | `88da8f1ebcb2f665b5bd6e32ffc29545e858f8db1ab19c88a612debccf010305` |
| C128 TRAIN SHA-256 | `1466d3790e1c53bc4ac3982712f669967cce7b3ad1d7e0e34259568154d19cb6`, 111,671 tokens across 128 records |
| D30 census SHA-256 | `bd03c2da8a64c6150bd71f2d13a5cfe922dd7389de4b9ce0dc0cf060a031f7d1` |
| Required C128 output SHA-256 | `eea859b5d7272ef8d4be3a4ee623c92c2b11da866478b05765a7b20557b3788e`, 16,459,406 bytes |

Use `full-compact-count spool c128 TRAIN D30_CENSUS NEW_PRIVATE_OUTPUT` with
only the bound device visible. The patched executable's read-only
[plan](finite-fit/plan-c128.json) reproduced 29 planned ranges for the order-zero
target ID and 29 for the order-one predecessor ID with a 4,096-target soft
budget. The single-ID range `[223, 224)` has 6,492 targets, proving that the plan
keeps an oversize key payload intact. Orders 1–10 partition on the most recent
context token, so a whole key stays in one range. Order zero partitions on
follower ID and the assembler combines their one shared context. At orders above
one, the GPU census excludes record starts with insufficient history and each
actual range must remain below its planned target count. The acknowledged queue
barriers release each pair before the next range goes through assembly.

The finite resource plan prices 269,624,846 GPU payload bytes and 35,112,512 host
payload bytes. The producer asks for a further 512 MiB of device headroom. The
storage requirement for the output file is 16,459,406 bytes plus its receipts.
The device snapshot, exact producer sources, compiler version and image records
are under [finite-fit](finite-fit/). The AOT loader consumes the cubin binary for
this GPU. The inspected producer and assembler edits contain no forbidden
operation names.

**Pass requires** a sealed receipt with exact matches for all eleven raw and
eleven adjusted census records and the listed whole-artifact digest, multiple
nonempty executed ranges in every order, and independent cold readback against
the frozen C128 source. The latter checks canonical keys and original IDs and
words, plus finite exact Q48 probes through the ordinary compact runtime. The
separate normal client witness checks packaging and traversal of the deployed
reader on a finite artifact. A pass allows review of the full-dose count launch
against its own live resource admission and dedicated GO.

**Failure** of the digest, range coverage, census, byte length or native
readback is an instrument discrepancy. Preserve the first divergent order,
range or byte and any unique partial output, then repair and repeat C128 with
fresh pins. Full C requires its own resource decision. This FIT grants no full
C dispatch or quality conclusion.
