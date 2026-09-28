#!/usr/bin/env python3
"""Release-bundle manifest packager for the width-576 integer dialogue bundle (#965).

The manifest records only what this script reads or runs:
- every bundle file with its size and SHA-256 digest (and a BLAKE3 digest when
  the optional `blake3` Python module is installed; otherwise no BLAKE3 field);
- the model shape from `bundle.json`;
- the native `uor-chat` binary and the WebAssembly helper module, each with the
  command, exit status and TAP lines of its instruction audit as actually run;
- the full-path cost report, read from its `metrics` and `parity_gate`
  sections with no defaults: a missing key is an error, never a fallback.

The numerical contract is recorded as a declaration, not a certification. The
manifest is not sealed and is not a qualification; a cost report counts as
sealed only when its directory holds a `report_output` manifest listing it.

Build identities (commit, compiler, flags) are inputs that default to
UNRECORDED; nothing is inferred from the checkout that runs the packager.

Usage:
    python3 scripts/package_release_bundle_integer.py [--bundle-dir DIR]
        [--arm64-bin PATH] [--arm64-build-commit SHA] [--arm64-compiler TEXT]
        [--arm64-flags TEXT] [--wasm-bin PATH] [--wasm-build-commit SHA]
        [--m1-cost PATH] [--output PATH]
"""

import argparse
import hashlib
import json
import os
import subprocess
import sys
import time

try:
    import blake3

    HAS_BLAKE3 = True
except ImportError:
    HAS_BLAKE3 = False

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DEFAULT_BUNDLE_DIR = "/Users/casey.allard/uor-r4/.uor-models/investigations/fourth-research-lab-20260926/dialogue-child-bundle-1"
DEFAULT_BIN_DIR = "/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-bins/698bdda481de57c2257b158bb53f7942568a3be7"
DEFAULT_ARM64_BIN = os.path.join(DEFAULT_BIN_DIR, "uor-chat")
DEFAULT_WASM_BIN = os.path.join(DEFAULT_BIN_DIR, "uor_r4_integer.wasm")
DEFAULT_M1_COST = os.path.join(REPO_ROOT, "docs/evidence/full-path-m1-cost-dialogue576-2026-09-28.json")
DEFAULT_OUTPUT = os.path.join(REPO_ROOT, "docs/evidence/release-bundle-manifest-dialogue576-2026-09-28.json")
UNRECORDED = "UNRECORDED"
M1_COST_SCHEMAS = {"uor-r4.m1-cost-profile/1", "uor-r4.m1-cost-profile/2"}
DECLARED_CEILINGS = {
    "cold_load_ms": ("metrics.cold_load_ms", 250.0),
    "step_mean_ms": ("metrics.step_latency.mean_ms", 4.0),
    "step_p90_ms": ("metrics.step_latency.p90_ms", 4.0),
}


class ManifestError(Exception):
    """An input the manifest needs is missing or malformed."""


def sha256_file(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        while chunk := handle.read(65536):
            digest.update(chunk)
    return digest.hexdigest()


def blake3_file(path):
    digest = blake3.blake3()
    with open(path, "rb") as handle:
        while chunk := handle.read(65536):
            digest.update(chunk)
    return digest.hexdigest()


def require(document, dotted, source):
    """Value at `dotted` in `document`; a missing key raises ManifestError."""
    value = document
    for key in dotted.split("."):
        if not isinstance(value, dict) or key not in value:
            raise ManifestError(f"{source}: required key '{dotted}' is missing")
        value = value[key]
    return value


def run_audit(script, arguments):
    relative_script = os.path.join("scripts", script)
    command = [sys.executable, relative_script, *arguments]
    result = subprocess.run(command, capture_output=True, text=True, cwd=REPO_ROOT)
    lines = result.stdout.splitlines()
    return {
        "command": " ".join(["python3", relative_script, *arguments]),
        "exit_code": result.returncode,
        "tap_ok": sum(1 for line in lines if line.startswith("ok ")),
        "tap_not_ok": sum(1 for line in lines if line.startswith("not ok ")),
        "tap_lines": [
            line
            for line in lines
            if line.startswith(("1..", "# ", "ok ", "not ok "))
        ],
    }


def is_sealed_report(path):
    manifest_path = os.path.join(os.path.dirname(path), "manifest.json")
    if not os.path.isfile(manifest_path):
        return False
    try:
        with open(manifest_path, "r") as handle:
            manifest = json.load(handle)
    except (OSError, json.JSONDecodeError):
        return False
    files = manifest.get("files") if isinstance(manifest, dict) else None
    return (
        isinstance(manifest, dict)
        and manifest.get("schema") == "uor-r4.report-manifest/1"
        and isinstance(files, list)
        and any(isinstance(f, dict) and f.get("path") == os.path.basename(path) for f in files)
    )


def cost_record(path):
    with open(path, "r") as handle:
        report = json.load(handle)
    schema = require(report, "schema", path)
    if schema not in M1_COST_SCHEMAS:
        raise ManifestError(f"{path}: unsupported cost report schema '{schema}'")
    sealed = is_sealed_report(path)
    record = {
        "report_path": os.path.relpath(path, REPO_ROOT) if path.startswith(REPO_ROOT) else path,
        "report_sha256": sha256_file(path),
        "report_schema": schema,
        "sealed_report_root": sealed,
        "status": (
            "one run in a sealed report root; not a qualification"
            if sealed
            else "one run without a sealed report root; not a qualification"
        ),
        "run_conditions": (
            require(report, "run", path)
            if schema.endswith("/2")
            else "not recorded by this report schema (machine load, source commit and build are unknown)"
        ),
        "cold_load_ms": require(report, "metrics.cold_load_ms", path),
        "tokenizer_encode_us_per_tok": require(report, "metrics.tokenizer_encode_us_per_tok", path),
        "prompt_ingest_ms_per_tok": require(report, "metrics.prompt_ingest_ms_per_tok", path),
        "step_mean_ms": require(report, "metrics.step_latency.mean_ms", path),
        "step_p50_ms": require(report, "metrics.step_latency.p50_ms", path),
        "step_p90_ms": require(report, "metrics.step_latency.p90_ms", path),
        "throughput_tokens_per_sec": require(report, "metrics.step_latency.tok_per_sec", path),
        "session_save_ms": require(report, "metrics.session_save_ms", path),
        "session_restore_ms": require(report, "metrics.session_restore_ms", path),
        "peak_process_rss_mb": require(report, "metrics.peak_rss_mb", path),
        "live_repl_rss_mb": "UNAVAILABLE (not recorded by the harness)",
        "bytes_touched_per_token": require(report, "metrics.bytes_touched_per_token", path),
        "bytes_touched_basis": "analytic count in the harness source; not measured",
        "energy_soc_joules_per_token": require(report, "metrics.energy_soc_joules_per_token", path),
        "parity_requests": require(report, "parity_gate.requests", path),
        "parity_turns": require(report, "parity_gate.turns", path),
        "parity_decisions_verified": require(report, "parity_gate.verified_decisions", path),
        "parity_departures": require(report, "parity_gate.departures", path),
    }
    ceilings = {}
    for name, (dotted, limit) in DECLARED_CEILINGS.items():
        measured = require(report, dotted, path)
        ceilings[name] = {"ceiling": limit, "recorded": measured, "meets": measured <= limit}
    record["declared_ceilings"] = ceilings
    return record


def build_manifest(args):
    bundle_dir = os.path.abspath(args.bundle_dir)
    arm64_bin = os.path.abspath(args.arm64_bin)
    wasm_bin = os.path.abspath(args.wasm_bin)
    m1_cost = os.path.abspath(args.m1_cost)
    for label, path, check in (
        ("bundle directory", bundle_dir, os.path.isdir),
        ("ARM64 binary", arm64_bin, os.path.isfile),
        ("WASM module", wasm_bin, os.path.isfile),
        ("cost report", m1_cost, os.path.isfile),
    ):
        if not check(path):
            raise ManifestError(f"{label} not found: {path}")
    cost = cost_record(m1_cost)  # validated before any audit runs

    bundle_files = []
    total_bytes = 0
    for root, dirs, files in os.walk(bundle_dir):
        dirs.sort()
        for name in sorted(files):
            full_path = os.path.join(root, name)
            size = os.path.getsize(full_path)
            total_bytes += size
            entry = {
                "path": os.path.relpath(full_path, bundle_dir),
                "bytes": size,
                "sha256": sha256_file(full_path),
            }
            if HAS_BLAKE3:
                entry["blake3"] = blake3_file(full_path)
            bundle_files.append(entry)

    bundle_json = os.path.join(bundle_dir, "bundle.json")
    with open(bundle_json, "r") as handle:
        bundle_meta = json.load(handle)

    print("Running the ARM64 instruction audit...")
    # Pass the build identity explicitly: left to itself the native auditor
    # records the auditing checkout's HEAD, its rustc and a default flag string.
    arm64_audit = run_audit(
        "audit_zero_matmul_serving.py",
        [
            arm64_bin,
            "--strict-arm64",
            "--tap",
            "--git-commit",
            args.arm64_build_commit,
            "--compiler",
            args.arm64_compiler,
            "--flags",
            args.arm64_flags,
        ],
    )
    if arm64_audit["exit_code"] != 0:
        raise ManifestError(
            f"ARM64 instruction audit failed (exit {arm64_audit['exit_code']}); no manifest written"
        )
    print("Running the WebAssembly instruction audit (call graph followed)...")
    # The WASM audit's exit code is recorded, not gated: the WASM helper
    # module is documented as not D11-clean (#1474), so a failing WASM audit
    # is the expected, honestly recorded state rather than a packaging error.
    wasm_audit = run_audit(
        "audit_zero_matmul_wasm.py",
        [wasm_bin, "--tap", "--call-graph", "--build-commit", args.wasm_build_commit],
    )

    return {
        "schema": "uor-r4.release-bundle-manifest/2",
        "release_id": "dialogue576-v1-release-20260928",
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "status": "unsealed manifest; not a qualification",
        "owning_lab": "Lab 3 (Anti-Gravity)",
        "track": "T3 (Mission runtime and measured efficiency)",
        "issues_referenced": [965, 963, 1172, 1173, 964, 820],
        "digest_algorithms": ["sha256", "blake3"] if HAS_BLAKE3 else ["sha256"],
        "model_architecture": {
            "name": "UOR-R4 Geometric Language Model (Dialogue Child)",
            "width": require(bundle_meta, "model.width", bundle_json),
            "vocab_size": require(bundle_meta, "model.vocab_size", bundle_json),
            "context_capacity": require(bundle_meta, "model.context", bundle_json),
            "read_geometry": require(
                bundle_meta, "numerical_contract.native_integer_profile.read_geometry", bundle_json
            ),
            "declared_numerical_contract": "D11",
            "contract_qualification": "Declared, not certified by this manifest; see the audit records of the served binaries.",
        },
        "bundle_artifact": {
            "root_path": bundle_dir,
            "total_bytes": total_bytes,
            "file_count": len(bundle_files),
            "files": bundle_files,
        },
        "served_binaries": {
            "arm64_native": {
                "name": "uor-chat",
                "role": "native serving binary",
                "path": arm64_bin,
                "sha256": sha256_file(arm64_bin),
                "size_bytes": os.path.getsize(arm64_bin),
                "target": "aarch64-apple-darwin",
                "build_commit": args.arm64_build_commit,
                "compiler": args.arm64_compiler,
                "compiler_flags": args.arm64_flags,
                "audit": arm64_audit,
            },
            "webassembly": {
                "name": "uor_r4_integer.wasm",
                "role": "helper-export module; no load, session or step export; not a serving runtime",
                "path": wasm_bin,
                "sha256": sha256_file(wasm_bin),
                "size_bytes": os.path.getsize(wasm_bin),
                "target": "wasm32-unknown-unknown",
                "build_commit": args.wasm_build_commit,
                "audit": wasm_audit,
            },
        },
        "full_path_cost": cost,
        "capability_api": {
            "schema": "uor-r4.integer-capability-api/1",
            "container": "uor_r4_integer::capability_api::IntegerCapabilityApi",
            "safety": "crate root declares #![forbid(unsafe_code)]",
        },
    }


def main():
    parser = argparse.ArgumentParser(description="Write the integer release-bundle manifest.")
    parser.add_argument("--bundle-dir", default=DEFAULT_BUNDLE_DIR)
    parser.add_argument("--arm64-bin", default=DEFAULT_ARM64_BIN)
    parser.add_argument(
        "--arm64-build-commit",
        default=UNRECORDED,
        help=f"Commit that built uor-chat (default: {UNRECORDED})",
    )
    parser.add_argument(
        "--arm64-compiler",
        default=UNRECORDED,
        help=f"Compiler that built uor-chat (default: {UNRECORDED})",
    )
    parser.add_argument(
        "--arm64-flags",
        default=UNRECORDED,
        help=f"Compiler flags that built uor-chat (default: {UNRECORDED})",
    )
    parser.add_argument("--wasm-bin", default=DEFAULT_WASM_BIN)
    parser.add_argument(
        "--wasm-build-commit",
        default=UNRECORDED,
        help=f"Commit that built the WASM module (default: {UNRECORDED})",
    )
    parser.add_argument("--m1-cost", default=DEFAULT_M1_COST, help="Full-path cost report JSON")
    parser.add_argument("--output", default=DEFAULT_OUTPUT)
    args = parser.parse_args()

    try:
        manifest = build_manifest(args)
    except (ManifestError, OSError, json.JSONDecodeError) as error:
        print(f"Error: {error}", file=sys.stderr)
        sys.exit(1)

    out_path = os.path.abspath(args.output)
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w") as handle:
        json.dump(manifest, handle, indent=2)
        handle.write("\n")

    binaries = manifest["served_binaries"]
    cost = manifest["full_path_cost"]
    print(f"Manifest written: {out_path}")
    print(
        f"  bundle files: {manifest['bundle_artifact']['file_count']} "
        f"({manifest['bundle_artifact']['total_bytes']:,} bytes), digests: "
        f"{', '.join(manifest['digest_algorithms'])}"
    )
    for key in ("arm64_native", "webassembly"):
        audit = binaries[key]["audit"]
        print(
            f"  {binaries[key]['name']}: sha256 {binaries[key]['sha256']}, audit exit "
            f"{audit['exit_code']} ({audit['tap_ok']} ok, {audit['tap_not_ok']} not ok)"
        )
    print(f"  cost report: {cost['status']}")
    for name, ceiling in cost["declared_ceilings"].items():
        print(f"    {name}: recorded {ceiling['recorded']} vs ceiling {ceiling['ceiling']} (meets: {ceiling['meets']})")


if __name__ == "__main__":
    main()
