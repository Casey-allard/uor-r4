#!/usr/bin/env python3
"""Batch runner for bridge/transfer probes across kernels and zero limits."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import time
from datetime import datetime, timezone
from typing import Dict, List


def parse_list_int(text: str):
    return [int(x.strip()) for x in text.split(",") if x.strip()]


def parse_kernel_configs(text: str):
    out = []
    for tok in text.split(","):
        tok = tok.strip()
        if not tok:
            continue
        if ":" in tok:
            k, s = tok.split(":", 1)
            out.append((k.strip(), float(s.strip())))
        else:
            out.append((tok, 0.0))
    if not out:
        raise ValueError("no kernel configs")
    return out


def run_cmd(cmd):
    t0 = time.perf_counter()
    proc = subprocess.run(cmd, capture_output=True, text=True)
    dt = time.perf_counter() - t0
    return {
        "cmd": cmd,
        "returncode": proc.returncode,
        "elapsed_sec": dt,
        "stdout": proc.stdout[-4000:],
        "stderr": proc.stderr[-4000:],
    }


def skip_record(kind: str, output: str, kernel: str, scale: float, zeros: int) -> Dict[str, object]:
    return {
        "kind": kind,
        "output": output,
        "kernel": kernel,
        "kernel_scale": scale,
        "max_zeta_zeros": zeros,
        "status": "SKIPPED_EXISTS",
        "returncode": 0,
        "elapsed_sec": 0.0,
        "cmd": [],
        "stdout": "",
        "stderr": "",
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Batch orchestrator for H(x) probes")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--n-values", type=str, default="300000,600000,1000000")
    ap.add_argument("--x-step", type=int, default=2000)
    ap.add_argument("--kernel-configs", type=str, default="none:0,gaussian:100,lorentz:100")
    ap.add_argument("--zero-limits", type=str, default="64,128,256")
    ap.add_argument("--weights", type=str, default="-1,-1,0,-1")
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--jobs", type=int, default=4)
    ap.add_argument("--chunk-n", type=int, default=50000)
    ap.add_argument("--python-bin", type=str, default="python3")
    ap.add_argument("--run-sweep", action="store_true")
    ap.add_argument("--run-transfer", action="store_true")
    ap.add_argument("--skip-existing", action="store_true", default=True)
    ap.add_argument("--force-rerun", action="store_true")
    ap.add_argument("--output-manifest", type=str, default="research/output/hx_batch_manifest.json")
    args = ap.parse_args()

    run_sweep = args.run_sweep or (not args.run_sweep and not args.run_transfer)
    run_transfer = args.run_transfer or (not args.run_sweep and not args.run_transfer)

    kernels = parse_kernel_configs(args.kernel_configs)
    zero_limits = parse_list_int(args.zero_limits)
    if not zero_limits:
        raise ValueError("zero-limits must not be empty")

    out_dir = os.path.dirname(args.output_manifest) or "."
    os.makedirs(out_dir, exist_ok=True)

    runs: List[Dict[str, object]] = []
    skip_existing = bool(args.skip_existing and not args.force_rerun)
    for kernel, scale in kernels:
        for m in zero_limits:
            tag = f"{kernel}_s{int(scale) if scale.is_integer() else scale}_z{m}".replace(".", "p")
            if run_sweep:
                out_json = f"research/output/hx_bridge_sweep_batch_{tag}.json"
                if skip_existing and os.path.exists(out_json):
                    runs.append(skip_record("sweep", out_json, kernel, scale, m))
                else:
                    cmd = [
                        args.python_bin,
                        "research/hx_bridge_sweep.py",
                        "--bases",
                        args.bases,
                        "--n-values",
                        args.n_values,
                        "--x-step",
                        str(args.x_step),
                        f"--weights={args.weights}",
                        "--u-mode",
                        args.u_mode,
                        "--zero-kernel",
                        kernel,
                        "--kernel-scale",
                        str(scale),
                        "--max-zeta-zeros",
                        str(m),
                        "--streaming",
                        "--chunk-n",
                        str(args.chunk_n),
                        "--jobs",
                        str(args.jobs),
                        "--output",
                        out_json,
                    ]
                    res = run_cmd(cmd)
                    res["kind"] = "sweep"
                    res["output"] = out_json
                    res["kernel"] = kernel
                    res["kernel_scale"] = scale
                    res["max_zeta_zeros"] = m
                    res["status"] = "OK" if res["returncode"] == 0 else "FAIL"
                    runs.append(res)

            if run_transfer:
                out_json = f"research/output/hx_transfer_lemma_probe_batch_{tag}.json"
                if skip_existing and os.path.exists(out_json):
                    runs.append(skip_record("transfer", out_json, kernel, scale, m))
                else:
                    cmd = [
                        args.python_bin,
                        "research/hx_transfer_lemma_probe.py",
                        "--bases",
                        args.bases,
                        "--n-values",
                        args.n_values,
                        "--x-step",
                        str(args.x_step),
                        "--u-mode",
                        args.u_mode,
                        "--zero-kernel",
                        kernel,
                        "--kernel-scale",
                        str(scale),
                        "--max-zeta-zeros",
                        str(m),
                        "--streaming",
                        "--chunk-n",
                        str(args.chunk_n),
                        "--jobs",
                        str(args.jobs),
                        "--output",
                        out_json,
                    ]
                    res = run_cmd(cmd)
                    res["kind"] = "transfer"
                    res["output"] = out_json
                    res["kernel"] = kernel
                    res["kernel_scale"] = scale
                    res["max_zeta_zeros"] = m
                    res["status"] = "OK" if res["returncode"] == 0 else "FAIL"
                    runs.append(res)

    skipped = [r for r in runs if str(r.get("status")) == "SKIPPED_EXISTS"]
    executed = [r for r in runs if str(r.get("status")) != "SKIPPED_EXISTS"]
    ok = [r for r in executed if int(r["returncode"]) == 0]
    fail = [r for r in executed if int(r["returncode"]) != 0]
    manifest = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": args.bases,
            "n_values": args.n_values,
            "x_step": args.x_step,
            "kernel_configs": kernels,
            "zero_limits": zero_limits,
            "u_mode": args.u_mode,
            "jobs": args.jobs,
            "chunk_n": args.chunk_n,
            "run_sweep": run_sweep,
            "run_transfer": run_transfer,
            "skip_existing": skip_existing,
            "force_rerun": bool(args.force_rerun),
        },
        "summary": {
            "total_runs": len(runs),
            "executed_runs": len(executed),
            "skipped_runs": len(skipped),
            "ok_runs": len(ok),
            "failed_runs": len(fail),
            "total_elapsed_sec": sum(r["elapsed_sec"] for r in runs),
        },
        "runs": runs,
    }

    with open(args.output_manifest, "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2)

    md = args.output_manifest.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# H(x) Batch Orchestrator\n\n")
        f.write(f"Generated: {manifest['timestamp_utc']}\n\n")
        f.write(f"- total runs: {manifest['summary']['total_runs']}\n")
        f.write(f"- executed runs: {manifest['summary']['executed_runs']}\n")
        f.write(f"- skipped existing: {manifest['summary']['skipped_runs']}\n")
        f.write(f"- ok runs: {manifest['summary']['ok_runs']}\n")
        f.write(f"- failed runs: {manifest['summary']['failed_runs']}\n")
        f.write(f"- total elapsed (sum): {manifest['summary']['total_elapsed_sec']:.2f}s\n\n")
        for r in runs:
            status = str(r.get("status", "OK" if r["returncode"] == 0 else "FAIL"))
            f.write(
                f"- [{status}] {r['kind']} kernel={r['kernel']} scale={r['kernel_scale']} "
                f"zeros={r['max_zeta_zeros']} elapsed={r['elapsed_sec']:.2f}s output={r['output']}\n"
            )

    print(f"wrote: {args.output_manifest}")
    print(f"wrote: {md}")
    print(f"ok_runs: {manifest['summary']['ok_runs']}/{manifest['summary']['total_runs']}")


if __name__ == "__main__":
    main()
