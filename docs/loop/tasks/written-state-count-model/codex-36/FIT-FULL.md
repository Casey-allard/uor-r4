# [codex] Full C non-author FIT for count spool and physical retention

**Wick resource HOLD for the 80-byte device payload bound.** The exact finite
CUDA producer completed the required split C128 spool. This review covers
full C count-only materialization and subsequent cap-limited retention for
P1(b). The purpose is to obtain a concrete native count component for a later
R-only fit, as specified by the [unit sheet](SHEET.md).

The full input is 4,151,216,778 physical bytes, with 2,075,608,357 framed
tokens and 515,520 records. Its SHA-256 is
`cd3adfe949ee34a62bc8d5e48592e091fcece796618d7910d1667d014a4bae2a`.
The pinned independent census [claude-439](../claude-439/run2/pick-p1000.json)
has SHA-256 `28d06be01c22439fb715d6ba516845c8ee5a0530c0b0c09ec9d062461d4a4a53`.
The [full plan](full-fit/plan-c-final.json) SHA-256 is
`c441106df98ee24c948f477bebf1fd9b384ffc34d7e72d4130f9c9993c89a873`.
Its two canonical range schedules each contain 127 ranges. One
range carries 147,763,001 targets. This is the exact histogram-derived
single-key exception to the soft 16,777,216-target bound. The planned compact
spool is 148,096,362,864 bytes. Keeping the original spool and the maximum
3,912,936,331-byte candidate simultaneously requires 152,009,299,195 bytes
plus small receipts.

The producer's pinned C128 [FIT](FIT-FINITE.md) used executable
`7a77f3d29430c12016026a0a051c57bca672b3831cdd2e05fd70237ef4605e66`
and produced the exact 16,459,406-byte frozen artifact. Every order executed
29 nonempty ranges, with maximum 6,492 targets. The actual raw and adjusted
receipts matched all 22 independent order records exactly. Direct `cmp` and
[cold native readback](verify/evidence/prod-c128-gpu-cold.json) passed against
the archived artifact.

The full plan prices 24,543,125,682 device payload bytes plus 536,870,912
bytes of driver and allocator reserve, for **25,079,996,594 required free
bytes after CUDA context initialization**. The same executable on the idle,
locked Wick device reported **24,838,406,144 free bytes after CUDA context
initialization** in the finite run, short by **241,590,450 bytes** even at that
idle reading. Wick's 35,496,674,672-byte host bound and the exact spool disk
requirement fit the [observed host and pool snapshot](full-fit/wick-resources.txt).
A device-idle report before CUDA context creation cannot close the GPU gap.

To pass a later full dispatch review, bind the exact eligible build and device
to the same frozen inputs and repeat finite split identity for any changed
binary. Retain a fresh headroom check after context initialization and the
admitted lock or lease. Count production must match all stored schema words
and digest domains against the census. The archived raw order-ten SHA fields
alias adjusted-domain hashes, so compare cardinalities and spectra, prove
word identity at order ten, and preserve the actual raw-domain hashes.
Full return requires a custody-ready original spool, the streaming pruner's
cap-selected candidate with complete context groups, independent byte and
count auditing and cold replay to prepare the separate fit. Preserve any
resource refusal, mismatch or partial unique output. This FIT records the
current resource hold and the evidence needed to lift it.
