#!/usr/bin/env bash
# Compile/inspect only. Provisioning, admission, independent review, archive
# verification and release/down belong to the external reviewed operator.
# No CUDA context, Device, model controls, timing command or apply runs here.
set -euo pipefail

construction_head=${1:?exact 40-character reviewed Git head}
construction_session=${2:?unique lower-case construction session}
[[ "$construction_head" =~ ^[0-9a-f]{40}$ ]] || exit 2
[[ "$construction_session" =~ ^[a-z0-9][a-z0-9-]+$ ]] || exit 2
construction_repo=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)
construction_out="$construction_repo/evals/$construction_session"
construction_target="/workspace/cuda-reuse-construction-target-$construction_session"
test ! -e "$construction_out"
test ! -e "$construction_target"
mkdir -p "$construction_out" "$construction_target"
export PATH=/root/.cargo/bin:/usr/local/cuda/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
export UOR_LAB=codex UOR_NVCC_ARCH=compute_120
export CARGO_TARGET_DIR="$construction_target"
export NVCC_AUDIT_LOG="$construction_out/nvcc-commands.log"
construction_cuda=/usr/local/cuda/bin/nvcc
construction_ordinary_source=f679f8736500925014ff721ab035f3c284446baa6af0763f27b09bc388763008
construction_ordinary_ptx=1a21d6c3e2773a427c14ebff9499e03386147ab41b3ce413b7b5c9123925fd28
construction_ordinary_cubin=1648564346740520c125fcfb5bc407cbbca40e66e95e2e3ad89b526a321ffcee

capture_outputs() {
    local phase directory label destination artifact capture_status=0
    mkdir -p "$construction_out/compiled" || return
    # Preserve every partial build set, including failed check/release builds.
    # Never require ordinary.cubin to exist before retaining the other files.
    for phase in debug release; do
        for directory in "$construction_target/$phase/build"/uor-r4-discrete-*/out; do
            [ -d "$directory" ] || continue
            label=$(basename "$(dirname "$directory")")
            destination="$construction_out/compiled/$phase-$label"
            mkdir -p "$destination" || { capture_status=$?; continue; }
            for artifact in geometry.rs discrete.ptx decomposition.cu decomposition.ptx \
                baseline-reuse.cu baseline-reuse.ptx ordinary.cubin decomposition.cubin \
                baseline-reuse.cubin ordinary-resources.txt decomposition-resources.txt \
                baseline-reuse-resources.txt cuda-image-target.txt; do
                if [ -f "$directory/$artifact" ]; then
                    cp "$directory/$artifact" "$destination/$artifact" || capture_status=$?
                fi
            done
        done
    done
    return "$capture_status"
}
finish() {
    local result=$? capture_result=0
    trap - EXIT
    set +e
    capture_outputs > "$construction_out/capture.log" 2>&1
    capture_result=$?
    printf '%s\n' "$capture_result" > "$construction_out/capture.exit"
    if [ "$result" -eq 0 ] && [ "$capture_result" -ne 0 ]; then result=$capture_result; fi
    git -C "$construction_repo" rev-parse HEAD > "$construction_out/final-head.txt"
    git -C "$construction_repo" status --porcelain --untracked-files=all > "$construction_out/final-status.txt"
    git -C "$construction_repo" diff --binary > "$construction_out/final-tracked.diff"
    df -Pk "$construction_repo" > "$construction_out/df-final.log"
    date -u +%FT%TZ > "$construction_out/ended.txt"
    printf '%s\n' "$result" > "$construction_out/command.exit.tmp"
    (
        cd "$construction_out" || exit
        find . -type f ! -name SHA256SUMS ! -name command.exit.tmp -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS || exit
        # Hash the eventual completion file before publishing it atomically.
        sha256sum ./command.exit.tmp | sed 's@\./command.exit.tmp$@./command.exit@' >> SHA256SUMS
    )
    local manifest_result=$?
    if [ "$manifest_result" -ne 0 ]; then
        if [ "$result" -eq 0 ]; then result=$manifest_result; fi
        printf '%s\n' "$result" > "$construction_out/command.exit.tmp"
        # An incomplete checksum file is evidence, never a success manifest.
        if [ -f "$construction_out/SHA256SUMS" ]; then
            mv "$construction_out/SHA256SUMS" "$construction_out/SHA256SUMS.partial"
        fi
    fi
    mv "$construction_out/command.exit.tmp" "$construction_out/command.exit"
    printf 'CONSTRUCTION_EXIT=%s; partial evidence retained in %s\n' "$result" "$construction_out"
    exit "$result"
}
trap finish EXIT

gate() {
    local name=$1 code
    shift
    {
        printf 'UTC='; date -u +%FT%TZ
        printf 'PWD=%q COMMAND=' "$PWD"; printf '%q ' "$@"; printf '\n'
    } | tee -a "$construction_out/commands.log"
    if "$@" > "$construction_out/$name.log" 2>&1; then code=0; else code=$?; fi
    cat "$construction_out/$name.log"
    printf '%s\n' "$code" > "$construction_out/$name.exit"
    printf '%s_EXIT=%s\n' "$name" "$code"
    if [ "$code" -ne 0 ]; then exit "$code"; fi
}
disk_floor() {
    local available
    df -Pk "$construction_repo"
    available=$(df -Pk "$construction_repo" | awk 'NR==2 {print $4}')
    [[ "$available" =~ ^[0-9]+$ ]] && [ "$available" -ge 31457280 ]
}
clean_source() {
    # gate calls this function in an if-condition: propagate each check
    # explicitly rather than relying on errexit inside that conditional.
    test "$(git rev-parse HEAD)" = "$construction_head" || return 1
    git diff --exit-code || return 1
    git diff --cached --exit-code || return 1
    local entry path
    local job_pattern="^jobs/codex/[0-9]{8}T[0-9]{6}Z-${construction_session}-gpu0\\.(log|sh)$"
    # Operational receipts are created before this check. Only this run's
    # exact receipt locations are allowed; all other untracked source fails.
    git status --porcelain=v1 -z --untracked-files=all |
        while IFS= read -r -d '' entry; do
            test "${entry:0:3}" = '?? ' || return 1
            path=${entry:3}
            case "$path" in
                "evals/$construction_session/"*|pods/reaper.log) continue ;;
            esac
            if [[ "$path" =~ $job_pattern ]]; then continue; fi
            printf 'Unadmitted untracked path: %q\n' "$path" >&2
            return 1
        done
}
sha() { sha256sum "$1" | awk '{print $1}'; }

date -u +%FT%TZ > "$construction_out/started.txt"
cd "$construction_repo"
gate source-preflight clean_source
gate disk-preflight disk_floor
gate rustc-version rustc +1.97.1 --version
gate cargo-version cargo +1.97.1 --version
gate clippy-version cargo +1.97.1 clippy --version
gate nvcc-version "$construction_cuda" --version
gate cuobjdump-version cuobjdump --version
gate rust-version-pin grep -E '^rustc 1\.97\.1 ' "$construction_out/rustc-version.log"
gate cargo-version-pin grep -E '^cargo 1\.97\.1 ' "$construction_out/cargo-version.log"
gate cuda-version-pin grep -F 'release 12.8, V12.8.93' "$construction_out/nvcc-version.log"
gate gpu-metadata nvidia-smi --query-gpu=name,uuid,driver_version,memory.total,compute_cap --format=csv
printf 'UOR_NVCC_ARCH=%s\nCARGO_TARGET_DIR=%s\n' "$UOR_NVCC_ARCH" "$CARGO_TARGET_DIR" > "$construction_out/build-environment.txt"
printf '%s  %s\n' "$construction_ordinary_source" crates/uor-r4-discrete/src/discrete.cu > "$construction_out/ordinary-source.sha256"
gate ordinary-source-pin sha256sum -c "$construction_out/ordinary-source.sha256"
git rev-parse HEAD > "$construction_out/git-head.txt"
git archive --format=tar HEAD crates/uor-r4-discrete \
    docs/labs/k16-timing-2026-10-15/instrument \
    docs/labs/counterfactual-baseline-reuse-implementation-2026-10-16/construct.sh \
    scripts/check_native_cuda_vendor.rs > "$construction_out/reviewed-source.tar"

cat > "$construction_out/nvcc-audit" <<'NVCC_AUDIT'
#!/usr/bin/env bash
set -euo pipefail
{ printf 'PWD=%q COMMAND=' "$PWD"; printf '%q ' /usr/local/cuda/bin/nvcc "$@"; printf '\n'; } >> "$NVCC_AUDIT_LOG"
exec /usr/local/cuda/bin/nvcc "$@"
NVCC_AUDIT
chmod 755 "$construction_out/nvcc-audit"
export NVCC="$construction_out/nvcc-audit"

# The direct discrete check compiles CUDA-gated targets without executing them.
# Strict Clippy is on the standalone instrument, as in gate2556-r1. Generated
# FRAME_REACHABILITY/ROOT_REACHABILITY dependency warnings remain in full logs;
# this script does not claim direct-crate strict-Clippy validation.
gate discrete-check cargo +1.97.1 check --locked -p uor-r4-discrete --features cuda --all-targets
gate disk-after-discrete disk_floor
cd "$construction_repo/docs/labs/k16-timing-2026-10-15/instrument"
gate instrument-check cargo +1.97.1 check --locked --features cuda
gate disk-after-check disk_floor
gate instrument-clippy cargo +1.97.1 clippy --locked --features cuda --all-targets -- -D warnings
gate disk-after-clippy disk_floor
gate instrument-release cargo +1.97.1 build --locked --release --features cuda --bin k16-timing
gate disk-after-release disk_floor
cp "$construction_target/release/k16-timing" "$construction_out/k16-timing"
gate reuse-identity "$construction_out/k16-timing" reuse-identity
cp "$construction_out/reuse-identity.log" "$construction_out/compiled-identity.json"
for phase in finite timing; do
    gate "$phase-template" "$construction_out/k16-timing" reuse-template --phase "$phase" --out-dir "$construction_out/never-run-$phase"
    cp "$construction_out/$phase-template.log" "$construction_out/$phase-template.json"
    gate "$phase-plan" "$construction_out/k16-timing" reuse-check --pin "$construction_out/$phase-template.json"
done

# Identity/plan commands hash and parse metadata only. They do not release or
# run the finite controls or timing dose; source-template pins stay unbound.
gate no-model-identity jq -e '.cuda_context_created == false and .model_compute_executed == false' "$construction_out/compiled-identity.json"
capture_outputs
construction_selected=0
for directory in "$construction_out/compiled"/*; do
    [ -d "$directory" ] || continue
    # A completed check build is also evidence; compare every complete U image.
    if [ -f "$directory/discrete.ptx" ]; then
        test "$(sha "$directory/discrete.ptx")" = "$construction_ordinary_ptx"
    fi
    if [ -f "$directory/ordinary.cubin" ]; then
        test "$(sha "$directory/ordinary.cubin")" = "$construction_ordinary_cubin"
    fi
    [[ $(basename "$directory") == release-* ]] || continue
    [ -f "$directory/baseline-reuse.cubin" ] || continue
    if jq -e \
        --arg up "$(sha "$directory/discrete.ptx")" \
        --arg dp "$(sha "$directory/decomposition.ptx")" \
        --arg u "$(sha "$directory/ordinary.cubin")" \
        --arg d "$(sha "$directory/decomposition.cubin")" \
        --arg ur "$(sha "$directory/ordinary-resources.txt")" \
        --arg dr "$(sha "$directory/decomposition-resources.txt")" \
        --arg rs "$(sha "$directory/baseline-reuse.cu")" \
        --arg rp "$(sha "$directory/baseline-reuse.ptx")" \
        --arg rc "$(sha "$directory/baseline-reuse.cubin")" \
        --arg rr "$(sha "$directory/baseline-reuse-resources.txt")" \
        '.compiled.kernel_sha256 == $up and .compiled.decomposition_kernel_sha256 == $dp
        and .compiled.cuda_images.ordinary_sha256 == $u and .compiled.cuda_images.decomposition_sha256 == $d
        and .compiled.cuda_images.ordinary_report_sha256 == $ur and .compiled.cuda_images.decomposition_report_sha256 == $dr
        and .compiled.baseline_reuse_image.source_sha256 == $rs and .compiled.baseline_reuse_image.ptx_sha256 == $rp
        and .compiled.baseline_reuse_image.cubin_sha256 == $rc and .compiled.baseline_reuse_image.report_sha256 == $rr
        and .compiled.cuda_images.target == "sm_120" and .compiled.baseline_reuse_image.target == "sm_120"' \
        "$construction_out/compiled-identity.json" > "$directory/binary-binding.json"; then
        construction_selected=$((construction_selected+1))
        printf '%s\n' "compiled/$(basename "$directory")" >> "$construction_out/selected-image-directory.txt"
        for image in ordinary decomposition baseline-reuse; do
            gate "$image-cuobjdump-resources" cuobjdump --dump-resource-usage "$directory/$image.cubin"
            gate "$image-cuobjdump-elf" cuobjdump --dump-elf "$directory/$image.cubin"
        done
    fi
done
test "$construction_selected" -eq 1
test "$(sha "$construction_out/k16-timing")" = "$(jq -r .binary_sha256 "$construction_out/compiled-identity.json")"
cd "$construction_repo"
gate source-final clean_source
gate disk-final disk_floor
printf '%s\n' 'Construction only: no runtime resource observation, CUDA context, model/control/timing execution, apply or training.' > "$construction_out/scope.txt"
# EXIT trap retains outputs and writes final SHA256SUMS, including on failure.
