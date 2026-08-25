#!/usr/bin/env python3
"""Run one deployed-quality command with durable host/resource evidence.

This wrapper is deliberately outside the deterministic production manifest:
wall time, host identity, free storage, and peak RSS are observations, not
semantic inputs. The wrapped command streams its own counters/rate/ETA to the
terminal while this process records the resource envelope even on failure.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import platform
import resource
import shutil
import stat
import subprocess
import sys
import tempfile
import time
from pathlib import Path


SCHEMA = "uor-r4-deployed-quality-resources/1"
SAMPLES_SCHEMA = "uor-r4-deployed-quality-resource-samples/1"
SAMPLE_INTERVAL_SECONDS = 5


def utc_now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat().replace("+00:00", "Z")


def sysctl(name: str) -> str | None:
    if sys.platform != "darwin":
        return None
    try:
        result = subprocess.run(
            ["/usr/sbin/sysctl", "-n", name],
            check=True,
            capture_output=True,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError):
        return None
    value = result.stdout.strip()
    return value or None


def integer(value: str | None) -> int | None:
    if value is None:
        return None
    try:
        return int(value)
    except ValueError:
        return None


def host_identity() -> dict[str, object | None]:
    uname = platform.uname()
    return {
        "system": uname.system,
        "release": uname.release,
        "version": uname.version,
        "machine": uname.machine,
        "processor": uname.processor or None,
        "model": sysctl("hw.model"),
        "cpu_brand": sysctl("machdep.cpu.brand_string"),
        "logical_cpus": os.cpu_count(),
        "physical_cpus": integer(sysctl("hw.physicalcpu")),
        "memory_bytes": integer(sysctl("hw.memsize")),
    }


def regular_file_sizes(root: Path) -> tuple[int, int, dict[str, int]]:
    total = 0
    count = 0
    evidence: dict[str, int] = {}
    if not root.is_dir():
        return total, count, evidence
    for directory, names, files in os.walk(root, followlinks=False):
        names.sort()
        files.sort()
        for name in files:
            path = Path(directory, name)
            try:
                metadata = path.lstat()
            except OSError:
                continue
            if not stat.S_ISREG(metadata.st_mode):
                continue
            size = metadata.st_size
            total += size
            count += 1
            relative = path.relative_to(root).as_posix()
            if (
                relative.startswith("graph/")
                or relative.startswith("evidence/")
                or relative == "release-bundle.json"
            ):
                evidence[relative] = size
    return total, count, evidence


def disk_snapshot(path: Path) -> dict[str, int]:
    usage = shutil.disk_usage(path)
    return {"total_bytes": usage.total, "used_bytes": usage.used, "free_bytes": usage.free}


def peak_rss_bytes() -> int:
    peak = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    # macOS reports bytes; Linux and the other common Unix implementations
    # report KiB. This repository's canonical host is macOS, but keep the
    # evidence unit truthful when the wrapper is used elsewhere.
    return int(peak if sys.platform == "darwin" else peak * 1024)


def process_tree_snapshot(root_pid: int) -> dict[str, object | None]:
    """Sample the wrapped process and all descendants without signalling them."""
    try:
        result = subprocess.run(
            ["/bin/ps", "-axo", "pid=,ppid=,%cpu=,rss=,vsz="],
            check=True,
            capture_output=True,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError):
        return {
            "process_count": None,
            "cpu_percent_sum": None,
            "cpu_capacity_percent": None,
            "cpu_capacity_fraction": None,
            "rss_bytes_sum": None,
            "virtual_bytes_sum": None,
        }
    rows: dict[int, tuple[int, float, int, int]] = {}
    for line in result.stdout.splitlines():
        fields = line.split()
        if len(fields) != 5:
            continue
        try:
            pid = int(fields[0])
            ppid = int(fields[1])
            cpu = float(fields[2])
            rss_kib = int(fields[3])
            virtual_kib = int(fields[4])
        except ValueError:
            continue
        rows[pid] = (ppid, cpu, rss_kib, virtual_kib)
    selected = {root_pid}
    changed = True
    while changed:
        changed = False
        for pid, (ppid, _, _, _) in rows.items():
            if ppid in selected and pid not in selected:
                selected.add(pid)
                changed = True
    live = [rows[pid] for pid in sorted(selected) if pid in rows]
    if not live:
        return {
            "process_count": 0,
            "cpu_percent_sum": 0.0,
            "cpu_capacity_percent": None if os.cpu_count() is None else os.cpu_count() * 100,
            "cpu_capacity_fraction": 0.0,
            "rss_bytes_sum": 0,
            "virtual_bytes_sum": 0,
        }
    logical_cpus = os.cpu_count()
    cpu_percent_sum = round(sum(row[1] for row in live), 3)
    return {
        "process_count": len(live),
        "cpu_percent_sum": cpu_percent_sum,
        "cpu_capacity_percent": None if logical_cpus is None else logical_cpus * 100,
        "cpu_capacity_fraction": (
            None if logical_cpus in (None, 0) else round(cpu_percent_sum / (logical_cpus * 100), 6)
        ),
        "rss_bytes_sum": sum(row[2] for row in live) * 1024,
        "virtual_bytes_sum": sum(row[3] for row in live) * 1024,
    }


def host_memory_snapshot() -> dict[str, int | None]:
    if sys.platform == "darwin":
        try:
            result = subprocess.run(
                ["/usr/bin/vm_stat"],
                check=True,
                capture_output=True,
                text=True,
            )
        except (OSError, subprocess.CalledProcessError):
            return {"available_bytes": None, "wired_bytes": None, "compressed_bytes": None}
        page_size = 4096
        values: dict[str, int] = {}
        for line in result.stdout.splitlines():
            if "page size of" in line:
                fields = line.split()
                try:
                    page_size = int(fields[7])
                except (IndexError, ValueError):
                    pass
                continue
            if ":" not in line:
                continue
            name, raw = line.split(":", 1)
            try:
                values[name.strip()] = int(raw.strip().rstrip("."))
            except ValueError:
                continue
        available_pages = sum(
            values.get(name, 0)
            for name in ("Pages free", "Pages inactive", "Pages speculative")
        )
        return {
            "available_bytes": available_pages * page_size,
            "wired_bytes": values.get("Pages wired down", 0) * page_size,
            "compressed_bytes": values.get("Pages occupied by compressor", 0) * page_size,
        }
    try:
        fields: dict[str, int] = {}
        for line in Path("/proc/meminfo").read_text(encoding="utf-8").splitlines():
            name, raw = line.split(":", 1)
            fields[name] = int(raw.strip().split()[0]) * 1024
        return {
            "available_bytes": fields.get("MemAvailable"),
            "wired_bytes": None,
            "compressed_bytes": None,
        }
    except (OSError, ValueError):
        return {"available_bytes": None, "wired_bytes": None, "compressed_bytes": None}


def write_synced_json_line(handle: object, payload: dict[str, object]) -> None:
    handle.write(json.dumps(payload, sort_keys=True, separators=(",", ":")))
    handle.write("\n")
    handle.flush()
    os.fsync(handle.fileno())


def command_option(command: list[str], name: str) -> str | None:
    for index, value in enumerate(command):
        if value == name and index + 1 < len(command):
            return command[index + 1]
        prefix = f"{name}="
        if value.startswith(prefix):
            return value[len(prefix) :]
    return None


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--bundle", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument(
        "--samples-output",
        type=Path,
        help="create-once live resource JSONL (default: <output>.samples.jsonl)",
    )
    parser.add_argument(
        "command",
        nargs=argparse.REMAINDER,
        help="wrapped command after -- (for example target/release/r4 deployed-quality ...)",
    )
    args = parser.parse_args()
    if args.command[:1] == ["--"]:
        args.command = args.command[1:]
    if not args.command:
        parser.error("a wrapped command is required after --")
    return args


def main() -> int:
    args = parse_args()
    bundle = args.bundle.resolve(strict=True)
    if not bundle.is_dir():
        raise SystemExit(f"bundle is not a directory: {bundle}")
    wrapped_bundle_option = "--bundle"
    wrapped_bundle = command_option(args.command, wrapped_bundle_option)
    if wrapped_bundle is None:
        wrapped_bundle_option = "--bundle-root"
        wrapped_bundle = command_option(args.command, wrapped_bundle_option)
    if wrapped_bundle is None:
        raise SystemExit("wrapped command must declare --bundle or --bundle-root")
    try:
        wrapped_bundle_path = Path(wrapped_bundle).resolve(strict=True)
    except OSError as error:
        raise SystemExit(f"wrapped --bundle is unavailable: {error}") from error
    if wrapped_bundle_path != bundle:
        raise SystemExit(
            f"wrapper bundle {bundle} does not match wrapped --bundle {wrapped_bundle_path}"
        )
    output = args.output.resolve()
    if output.exists():
        raise SystemExit(
            f"resource evidence already exists; choose a new append-only output path: {output}"
        )
    output.parent.mkdir(parents=True, exist_ok=True)
    samples_output = (
        args.samples_output.resolve()
        if args.samples_output is not None
        else output.with_name(f"{output.name}.samples.jsonl")
    )
    if samples_output.exists():
        raise SystemExit(
            "resource sample evidence already exists; choose a new append-only path: "
            f"{samples_output}"
        )
    samples_output.parent.mkdir(parents=True, exist_ok=True)

    before_bytes, before_files, before_evidence = regular_file_sizes(bundle)
    disk_before = disk_snapshot(bundle)
    started_utc = utc_now()
    started = time.monotonic_ns()
    exit_code: int
    interruption: str | None = None
    child: subprocess.Popen[bytes] | None = None
    sample_count = 0
    peak_sampled_rss = 0
    peak_sampled_processes = 0
    with samples_output.open("x", encoding="utf-8") as samples:
        write_synced_json_line(
            samples,
            {
                "schema": SAMPLES_SCHEMA,
                "event": "started",
                "semantic_admission_input": False,
                "bundle": str(bundle),
                "started_utc": started_utc,
                "sample_interval_seconds": SAMPLE_INTERVAL_SECONDS,
                "command": args.command,
            },
        )
        try:
            child = subprocess.Popen(args.command)
            while True:
                snapshot = process_tree_snapshot(child.pid)
                sampled_rss = snapshot.get("rss_bytes_sum")
                if isinstance(sampled_rss, int):
                    peak_sampled_rss = max(peak_sampled_rss, sampled_rss)
                sampled_processes = snapshot.get("process_count")
                if isinstance(sampled_processes, int):
                    peak_sampled_processes = max(peak_sampled_processes, sampled_processes)
                write_synced_json_line(
                    samples,
                    {
                        "schema": SAMPLES_SCHEMA,
                        "event": "sample",
                        "semantic_admission_input": False,
                        "sample_index": sample_count,
                        "elapsed_millis": (time.monotonic_ns() - started) // 1_000_000,
                        "process_tree": snapshot,
                        "host_memory": host_memory_snapshot(),
                        "storage": disk_snapshot(bundle),
                    },
                )
                sample_count += 1
                try:
                    exit_code = child.wait(timeout=SAMPLE_INTERVAL_SECONDS)
                    break
                except subprocess.TimeoutExpired:
                    continue
        except KeyboardInterrupt:
            interruption = "keyboard-interrupt"
            if child is not None and child.poll() is None:
                child.terminate()
                try:
                    child.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.wait()
            child_status = child.poll() if child is not None else None
            exit_code = 130 if child_status is None else child_status
        except OSError as error:
            exit_code = 127
            interruption = f"launch-error: {error}"
        write_synced_json_line(
            samples,
            {
                "schema": SAMPLES_SCHEMA,
                "event": "terminal",
                "semantic_admission_input": False,
                "status": "completed" if exit_code == 0 else "failed",
                "exit_code": exit_code,
                "interruption": interruption,
                "elapsed_millis": (time.monotonic_ns() - started) // 1_000_000,
                "samples": sample_count,
                "peak_sampled_rss_bytes": peak_sampled_rss,
                "peak_sampled_processes": peak_sampled_processes,
            },
        )
    elapsed_ns = time.monotonic_ns() - started
    finished_utc = utc_now()
    after_bytes, after_files, after_evidence = regular_file_sizes(bundle)
    disk_after = disk_snapshot(bundle)
    worker_option = "--workers"
    requested_workers = integer(command_option(args.command, worker_option))
    if requested_workers is None:
        worker_option = "--jobs"
        requested_workers = integer(command_option(args.command, worker_option))
    effective_workers = requested_workers if requested_workers is not None else os.cpu_count()

    artifact = {
        "schema": SCHEMA,
        "semantic_admission_input": False,
        "evidence_snapshot_scope": (
            "bundle files present at child exit; excludes this resource sidecar "
            "and the outer terminal transcript trailer"
        ),
        "bundle": str(bundle),
        "wrapped_bundle_option": wrapped_bundle_option,
        "command": args.command,
        "started_utc": started_utc,
        "finished_utc": finished_utc,
        "elapsed_millis": elapsed_ns // 1_000_000,
        "exit_code": exit_code,
        "status": "completed" if exit_code == 0 else "failed",
        "interruption": interruption,
        "peak_rss_bytes": peak_rss_bytes(),
        "peak_sampled_process_tree_rss_bytes": peak_sampled_rss,
        "peak_sampled_process_count": peak_sampled_processes,
        "resource_samples": str(samples_output),
        "resource_sample_count": sample_count,
        "worker_count": effective_workers,
        "worker_source": worker_option if requested_workers is not None else "available-parallelism",
        "host": host_identity(),
        "storage_before": {
            "bundle_bytes": before_bytes,
            "regular_files": before_files,
            "evidence_file_bytes": before_evidence,
            "filesystem": disk_before,
        },
        "storage_after": {
            "bundle_bytes": after_bytes,
            "regular_files": after_files,
            "evidence_file_bytes": after_evidence,
            "filesystem": disk_after,
        },
        "bundle_growth_bytes": after_bytes - before_bytes,
    }
    descriptor, temporary_name = tempfile.mkstemp(
        dir=output.parent,
        prefix=f".{output.name}.tmp-",
        text=True,
    )
    temporary = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as handle:
            json.dump(artifact, handle, indent=2, sort_keys=True)
            handle.write("\n")
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, output)
    finally:
        try:
            temporary.unlink()
        except FileNotFoundError:
            pass
    print(
        f"resource evidence: elapsed_ms={artifact['elapsed_millis']} "
        f"peak_rss_bytes={artifact['peak_rss_bytes']} "
        f"peak_process_tree_rss_bytes={artifact['peak_sampled_process_tree_rss_bytes']} "
        f"bundle_growth_bytes={artifact['bundle_growth_bytes']} path={output} "
        f"samples={samples_output}",
        flush=True,
    )
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
