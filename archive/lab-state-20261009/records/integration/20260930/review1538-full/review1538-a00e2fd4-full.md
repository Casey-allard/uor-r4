Codex independent **full main→head source review** at `a00e2fd493067af457701bfc69aac3dc63699385`, base `ea03435808c67791b76ae0e9868978e3188cba04`. Covers all eight changed files and the relevant load/export/reference callers. This expands the earlier test-only delta review; no Cargo/model execution.

**One remaining documentation correction, no additional production-code defect found:**
- `TransportSnap::file_record` in `crates/uor-r4-training/src/geometric_stack.rs` around1460 still serializes “StackModel::load loads them without the snap” into every newly saved transport.json. Correct this to the new automatic snap restoration and explicit free-transport API. Do not rewrite historical artifact records or change their hashes.
- Training comments in `examples/geometric-stack.rs` around1470 and3234 still say a saved model holds no snap/neither mode. Their actual explicit argument-based override is retained correctly; update the comments to describe that override rather than absent saved state.

**Reviewed behavior:**
- D10 constructor refuses a reference-parsed snapped artifact before accessing matrices; the header-only test exercises that bypass and distinguishes an ordinary missing-matrix failure.
- `StackModel::load` validates saved root identity and model configuration through existing readers/setters, restores snap, leaves served-codec restoration to existing callers.
- Load/save/free-view/invalid-root tests cover the changed contract; the combined-mode reapplication assertion remains bitwise. Format-valid refusal fixture and corrected `?` expression are consistent.
- `snap_evaluate` explicitly obtains the free baseline. D10-only rounding/lut comparisons and d4-float-reference refuse incompatible snapped inputs. Training init/resume explicitly applies requested settings; AERM's existing explicit reapplication is idempotent. Export guard prevents silently dropping a loaded snap. Signature repairs in d4_map_codec_adapter add the required snap argument without changing its semantics.
- Current-state and parity prose preserve first-divergence scope, unclassified later mismatches and contended-host cost limits.

**Retained evidence independently checked now:** `snap-parity.json` SHA256313645996327328adedfee37fd8f3bc78d619ba5e6cd35792ca0966a1342fe6f and `attempt.json` SHA256ddce53b7dc358d8b81142ff074d29235ceb3b39a9e763ba11db5a86793829f04 match the actual restored files. Both `kernel` and `kernel_first_divergence` objects equal the committed extract. The source root is unsealed as declared; this is file-bound historical evidence, not a new execution or a sealed-root claim.

**Delivery disposition:** source review is otherwise satisfactory; correct the three stale statements in one narrow commit, retain this review for unchanged code and review the documentation delta. Applicable compiled/focused checks at the actual candidate remain NOT_RUN and required before protected delivery. Compose with #1534 using the shared export contract; do not overwrite either fix. No model rerun or broader research campaign requested.
