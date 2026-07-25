#!/usr/bin/env python3
"""
Compiler GPU/Tensor/BLAS Dependency and Default Feature Audit Script

Specification: docs/compiler_dependency_audit.md (Issue #174).
Enforces zero GPU, tensor, or BLAS accelerator dependencies across the
compiler dependency graph (Cargo.lock and Cargo.toml manifests).
"""

import os
import sys

EXACT_FORBIDDEN_CRATES = [
    "ash",
    "ort",
    "torch",
    "metal",
    "rocm",
    "cuda",
    "opencl",
]

SUBSTRING_FORBIDDEN_PATTERNS = [
    "cuda-",
    "cuda_",
    "cust",
    "cudnn",
    "nvml",
    "chainer-cuda",
    "hip-sys",
    "metal-sys",
    "opencl-",
    "cl-sys",
    "vulkan",
    "wgpu",
    "directml",
    "sycl",
    "tch",
    "candle-core",
    "onnxruntime",
    "openblas-sys",
    "intel-mkl-sys",
    "accelerate-src",
]

FORBIDDEN_FEATURE_PATTERNS = [
    "gpu",
    "cuda",
    "metal",
    "opencl",
    "vulkan",
    "wgpu",
    "sycl",
]

def is_forbidden_crate(pkg_name):
    pkg = pkg_name.lower()
    for exact in EXACT_FORBIDDEN_CRATES:
        if pkg == exact:
            return exact
    for pattern in SUBSTRING_FORBIDDEN_PATTERNS:
        if pattern in pkg:
            return pattern
    return None

def audit_lockfile(lockfile_path):
    if not os.path.exists(lockfile_path):
        print(f"Warning: {lockfile_path} not found.")
        return 0

    violations = []
    packages_scanned = 0

    with open(lockfile_path, "r", encoding="utf-8") as f:
        for line in f:
            line_str = line.strip()
            if line_str.startswith('name = '):
                packages_scanned += 1
                pkg_name = line_str.split('name = ')[1].strip('"').lower()
                matched_pat = is_forbidden_crate(pkg_name)
                if matched_pat:
                    violations.append((pkg_name, matched_pat))

    if violations:
        print(f"FAIL: {len(violations)} forbidden dependency violations detected in Cargo.lock:")
        for pkg, pat in violations:
            print(f"  - Package '{pkg}' matched denylist pattern '{pat}'")
        return -1

    print(f"Pass: Audited {packages_scanned} lockfile packages; 0 forbidden GPU/tensor/BLAS dependencies.")
    return packages_scanned

def audit_cargo_toml_default_features(manifest_path):
    if not os.path.exists(manifest_path):
        return 0

    violations = []
    in_features = False

    with open(manifest_path, "r", encoding="utf-8") as f:
        for line in f:
            line_str = line.strip()
            if line_str.startswith("[features]"):
                in_features = True
                continue
            if line_str.startswith("[") and in_features:
                in_features = False

            if in_features and line_str.startswith("default ="):
                lower = line_str.lower()
                for pattern in FORBIDDEN_FEATURE_PATTERNS:
                    if pattern in lower:
                        violations.append((manifest_path, pattern))

    if violations:
        print(f"FAIL: Default GPU feature violation detected in {manifest_path}:")
        for path, pat in violations:
            print(f"  - Default features match forbidden pattern '{pat}'")
        return -1

    return 0

def main():
    repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    lockfile_path = os.path.join(repo_root, "Cargo.lock")

    lock_res = audit_lockfile(lockfile_path)
    if lock_res < 0:
        sys.exit(1)

    manifest_paths = [os.path.join(repo_root, "Cargo.toml")]
    for root, _, files in os.walk(os.path.join(repo_root, "crates")):
        for f in files:
            if f == "Cargo.toml":
                manifest_paths.append(os.path.join(root, f))

    for mp in manifest_paths:
        if audit_cargo_toml_default_features(mp) < 0:
            sys.exit(1)

    print("Compiler dependency and feature audit passed cleanly: 100% CPU-native.")
    sys.exit(0)

if __name__ == "__main__":
    main()
