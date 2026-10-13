# Final D29 finite qualification receipts

The final `g` run records **31/31 fixed controls PASS** at source
`1f1547bfd70f01edc9aa36d34a46e5bc43edc34c`. These are 15 selected library controls,
11 native CLI controls and 5 compiled-module interlocks. They combine CUDA
arithmetic controls with metadata/refusal/static checks; this is not a claim
that 31 GPU models ran. No optimizer update, corpus evaluation or training was
performed. The global training hold and separate M2 training/rental hold remain
**HELD**. This qualification establishes the declared finite boundary and
controls, not language quality, complete D11 serving or the future murmuration
integration.

The replacement device was an NVIDIA GeForce RTX 4090, UUID
`GPU-494cf136-4a85-b823-00f7-af2801405222`, driver `575.57.08`, with 24,564 MiB
reported total memory. The archived setup identifies Ubuntu 24.04.1 LTS,
CUDA toolkit `12.8.93`, Rust `1.97.1` and AOT target `compute_89` (device compute
capability 8.9). The GPU CSV contains **72 observations**, with a **sampled
maximum of 9,192 MiB**. Sampling was requested every 200 ms; an unsampled absolute
peak is not established.

The original receipt archive is
`codex-d29-qualification-receipts-20261013.tar.gz`, SHA256
`28c356d733e136de1300d58804ffbafb7fed3af4d2b6c7a09b0483c91b03d245`.
This compact selection contains 97 raw files (37,271 bytes), copied byte-exact
from that verified archive. Earlier attempts and the complete recoverable
archive remain separately preserved by the controller. No executable, PTX
payload or fixture model weight is included here.

- [controls/RESULT.txt](controls/RESULT.txt) is the final controller result.
  `controls/control-01.log` through `control-31.log` preserve every selected
  command, status, elapsed time and raw stdout/stderr. The three `list-*.log`
  files preserve executable preflight inventories. Source, clean tracked state,
  device, scope and executable SHA256 receipts are beside them.
- [VERIFICATION.txt](VERIFICATION.txt) lists all 31 exact names and verifies
  one occurrence in the appropriate preflight inventory, exit 0, exactly one
  executed/passing test, zero failures and zero ignored tests for every log.
- `static/` preserves checker self-test, vendor PTX/features and four executable
  import-census pass logs, together with the helper binary hashes. Their original
  static-only and indirect-`dlopen` limitations remain intact; numerical control
  results are recorded separately.
- `setup/` preserves the final setup manifest, stages, compiler/device/source
  identities, Cargo.lock hash and twelve small PTX SHA256 receipts. The setup
  manifest correctly records `SETUP_ONLY` and `tests: NOT_RUN` at its creation;
  the later control result supplies the finite qualification evidence.
- [ARCHIVE_PATHS.tsv](ARCHIVE_PATHS.tsv) maps every committed raw file to its
  original archive member. [SHA256SUMS](SHA256SUMS) binds the contents of this
  directory; verify it from this directory with `shasum -c SHA256SUMS`.

The run's prospective FIT receipt is preserved in `controls/scope.txt`:
[M1 finite qualification FIT](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6150514242).
Qualification and preservation do not lift either owner hold. Protected
delivery and cloud round-trip verification are controller responsibilities.
