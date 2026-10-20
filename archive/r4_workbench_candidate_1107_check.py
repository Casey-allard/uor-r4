#!/usr/bin/env python3
"""Source-only #1107 decision checker; never imports or executes model code."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CRATE = ROOT / "crates/uor-r4-workbench"


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def main() -> None:
    service_path = ROOT / "docs/r4_service_contract_1105.json"
    private_path = ROOT / "docs/r4_workbench_private_release_1107.json"
    service = json.loads(service_path.read_text(encoding="utf-8"))
    private = json.loads(private_path.read_text(encoding="utf-8"))
    require(
        digest(service_path)
        == "337d66d025fc9ec3a1e8c21befc25198b015061235fefc98f9208f99412e7a7f",
        "accepted #1105 contract identity changed",
    )
    require(service["status"] == "SERVICE_API_CONTRACT_SPECIFIED", "wrong authority status")
    require(private["status"] == "SOURCE_PROTOCOL_FROZEN_BEHAVIOR_NOT_RUN", "private protocol status changed")
    require(private["counters"] == {
        "model_loads": 0,
        "forwards": 0,
        "comparisons": 0,
        "qualification_calls": 0,
        "service_or_browser_runs": 0,
    }, "private protocol claims runtime work")

    workspace = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    manifest = (CRATE / "Cargo.toml").read_text(encoding="utf-8")
    require('"crates/uor-r4-workbench"' in workspace, "workbench is not a workspace member")
    default_block = workspace.split("default-members = [", 1)[1].split("]", 1)[0]
    require("uor-r4-workbench" not in default_block, "workbench must remain opt-in")
    compact_manifest = re.sub(r"\s+", " ", manifest)
    require('name = "uor-r4-workbench"' in manifest, "wrong package name")
    require('name = "r4-workbench"' in manifest, "wrong binary name")
    require('default-features = false' in compact_manifest, "API default features are enabled")
    require('features = ["learned-reference"]' in compact_manifest, "wrong API feature")
    for forbidden in ["tokio", "axum", "hyper", "reqwest", "clap", "base64 ="]:
        require(forbidden not in manifest, f"unadmitted dependency present: {forbidden}")

    required_sources = {
        "authority.rs", "base64.rs", "comparison.rs", "host.rs", "http.rs", "intake.rs",
        "ipc.rs", "launch.rs", "lib.rs", "lifecycle.rs", "main.rs", "strict_json.rs",
        "wire.rs", "worker.rs",
    }
    actual_sources = {path.name for path in (CRATE / "src").glob("*.rs")}
    require(actual_sources == required_sources, f"source set mismatch: {actual_sources ^ required_sources}")

    main_source = (CRATE / "src/main.rs").read_text(encoding="utf-8")
    for mode in ["--config", "--internal-worker", "--private-compare-host", "--private-metadata"]:
        require(mode in main_source, f"missing final executable mode {mode}")
    require("TcpListener" not in (CRATE / "src/worker.rs").read_text(encoding="utf-8"), "worker owns a listener")
    require("TcpListener" not in (CRATE / "src/comparison.rs").read_text(encoding="utf-8"), "comparison owns a listener")
    for public_name in ["host.rs", "http.rs", "worker.rs"]:
        source = (CRATE / "src" / public_name).read_text(encoding="utf-8")
        require("ComparisonAdmission" not in source, f"comparison admission leaked into {public_name}")
        require(".compare(" not in source, f"comparison call leaked into {public_name}")
    comparison = (CRATE / "src/comparison.rs").read_text(encoding="utf-8")
    require("ComparisonAdmission::from_trusted_release" in comparison, "missing original provenance admission")
    require("hash_inherited_executable" in comparison, "private mode does not bind executing image")

    wire = (CRATE / "src/wire.rs").read_text(encoding="utf-8")
    ipc_commands = service["enums"]["IPCCommand"]
    require(ipc_commands == ["load", "answer", "unload"], "accepted IPC commands changed")
    command_body = wire.split("pub enum IpcCommand", 1)[1].split("}", 1)[0]
    require("Compare" not in command_body, "comparison became a worker command")
    rust_names = {
        "IPCRequest": "IpcRequest",
        "IPCLoad": "IpcLoad",
        "IPCResponse": "IpcResponse",
    }
    for definition in service["types"]["definitions"]:
        if definition == "ExpectedBinding":
            require("ExpectedBinding" in wire, "missing imported wire type ExpectedBinding")
        else:
            rust_name = rust_names.get(definition, definition)
            require(f"struct {rust_name}" in wire or f"enum {rust_name}" in wire, f"missing wire type {definition}")

    http = (CRATE / "src/http.rs").read_text(encoding="utf-8")
    for route in service["routes"]:
        fragments = [fragment for fragment in route["path"].split("{job_id}") if fragment]
        require(all(fragment in http for fragment in fragments), f"missing route source {route['path']}")
    http_production = http.split("#[cfg(test)]", 1)[0]
    require("compare" not in http_production.lower(), "public HTTP source contains comparison route")
    require("Access-Control-Allow-Origin" not in http, "CORS header was introduced")

    asset_manifest_path = CRATE / "assets/assets.json"
    assets = json.loads(asset_manifest_path.read_text(encoding="utf-8"))
    require(assets["schema"] == "uor-r4.workbench-assets/1", "wrong asset schema")
    require(len(assets["files"]) <= 128, "asset count cap exceeded")
    require({row["path"] for row in assets["files"]} == {"index.html", "app.js", "styles.css", "NOTICE.txt"}, "unexpected asset set")
    total = 0
    for row in assets["files"]:
        path = CRATE / "assets" / row["path"]
        body = path.read_bytes()
        require(len(body) == row["bytes"], f"asset length mismatch: {row['path']}")
        require(hashlib.sha256(body).hexdigest() == row["sha256"], f"asset digest mismatch: {row['path']}")
        require(0 < len(body) <= 4_194_304, f"asset cap mismatch: {row['path']}")
        total += len(body)
    require(total <= 16_777_216, "asset total cap exceeded")
    frontend = (CRATE / "assets/app.js").read_text(encoding="utf-8")
    html = (CRATE / "assets/index.html").read_text(encoding="utf-8")
    require("innerHTML" not in frontend, "shell uses executable HTML insertion")
    require("textContent" in frontend, "shell lacks text-only result rendering")
    require("http://" not in frontend + html and "https://" not in frontend + html, "shell has remote dependency or provider")
    for forbidden in ["fallback", "canned", "Qwen", "GLM", "markdown", "WebGPU"]:
        require(forbidden.lower() not in frontend.lower(), f"shell includes forbidden behavior: {forbidden}")

    print(json.dumps({
        "schema": "uor-r4.workbench-source-check/1",
        "issue": 1107,
        "status": "SOURCE_CHECKS_PASSED",
        "service_contract_sha256": digest(service_path),
        "private_release_contract_sha256": digest(private_path),
        "source_files": len(required_sources),
        "wire_definitions": len(service["types"]["definitions"]),
        "routes": len(service["routes"]),
        "assets": len(assets["files"]),
        "asset_bytes": total,
        "runtime_work": "NOT_RUN",
    }, sort_keys=True, separators=(",", ":")))


if __name__ == "__main__":
    main()
