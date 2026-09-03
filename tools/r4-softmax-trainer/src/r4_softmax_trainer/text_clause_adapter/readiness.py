"""One separately reviewed, zero-forward #1096 runtime-readiness attempt.

Freeze constructs only identities and harmless probe files. Run requires the
independent review's exact manifest hash, uses the actual comparison worker,
and never imports campaign.prepare or opens a research population.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import resource
import selectors
import signal
import subprocess
import time

from . import contract

LIMITS = {"seconds": 60, "new_bytes": 16 * 1024**2,
          "peak_rss_bytes": 3 * 1024**3, "model_forwards": 0}
PROBE_BYTES = b"Harmless runtime isolation sentinel. No research payload or answer.\n"
STOP_SHA256 = "87bd3082ce9b4da5e5227a3b82f6515773cf5f113de689ff6251a2a45340fad5"


class CaptureStop(Exception):
    def __init__(self, error: Exception, data: dict) -> None:
        super().__init__(str(error))
        self.stdout = bytes(data["stdout"])
        self.stderr = bytes(data["stderr"])


def deadline_alarm(_number, _frame) -> None:
    raise TimeoutError("60-second readiness budget exhausted")


def new_bytes(root: Path) -> int:
    return sum(path.stat().st_size for path in root.rglob("*") if path.is_file())


def write_bytes(path: Path, payload: bytes) -> dict:
    with path.open("xb") as stream:
        stream.write(payload)
        stream.flush()
        os.fsync(stream.fileno())
    return contract.record(path)


def freeze(repo: Path, python: Path, output: Path) -> dict:
    started = time.monotonic()
    # A new directory prevents overwriting a prior freeze, result or user file.
    output.mkdir(mode=0o700)
    binding = contract.make_bindings(repo)
    previous = contract.record(repo / "docs/r4_text_clause_adapter_1094_evidence/prepare-stopped.json")
    if previous["sha256"] != STOP_SHA256:
        raise ValueError("original unavailable terminal changed")
    binding_record = contract.exclusive(output / "bindings.json", binding)
    resolved = python.resolve(strict=True)
    venv, base = python.parent.parent, resolved.parent.parent
    site = venv / "lib/python3.12/site-packages"
    runtime_paths = [resolved, venv / "pyvenv.cfg", base / "lib/python3.12/pathlib.py",
                     base / "lib/python3.12/posixpath.py", site / "torch/__init__.py",
                     site / "torch-2.7.1.dist-info/METADATA"]
    runtime_paths += sorted((site / "torch").glob("_C*.so"))
    runtime_paths += sorted((site / "torch/lib").glob("*.dylib"))
    runtime_paths += sorted(site.glob("*.pth")) + [site / "_virtualenv.py"]
    runtime_files = [contract.record(path) for path in runtime_paths]
    probes = {}
    for name in ("corpus", "reference", "history", "results"):
        directory = output / "probes" / name
        directory.mkdir(parents=True)
        probes[name] = write_bytes(directory / "sentinel.txt", PROBE_BYTES)
    manifest_path, profile_path = output / "manifest.json", output / "worker.sb"
    profile = contract.sandbox_profile(repo, python, output / "bindings.json", binding["assets"],
                                       extra_read_files=(manifest_path, profile_path))
    profile_record = write_bytes(profile_path, profile.encode())
    manifest = {
        "schema": "uor-r4.isolated-runtime-readiness/1", "issue": 1096,
        "source_commit": binding["source_commit"], "repo": str(repo),
        "previous_stop": previous, "bindings": binding_record,
        "profile": profile_record, "limits": LIMITS,
        "interpreter": {"launcher": str(python), "resolved": str(resolved),
                        "venv": str(venv.resolve()), "base": str(base),
                        "links": contract.interpreter_links(python)},
        "runtime_files": runtime_files, "torch_file": str(site / "torch/__init__.py"),
        "hardware": binding["hardware"], "runtime": contract.RUNTIME,
        "probes": probes,
        "probe_scope": "New harmless stand-ins under denied home paths; no actual corpus payload is opened.",
        "preparation_seconds": time.monotonic() - started,
        "model_loads": 0, "model_forwards": 0, "optimizer_updates": 0,
    }
    if new_bytes(output) + len(contract.canonical(manifest)) + 8 * 1024**2 > LIMITS["new_bytes"]:
        raise ValueError("freeze exceeds receipt budget")
    return contract.exclusive(manifest_path, manifest)


def receive(command: list[str], env: dict, deadline: float) -> tuple[int, bytes, bytes]:
    """Bound combined output and elapsed time; reap the single process group."""
    process = None
    data = {"stdout": bytearray(), "stderr": bytearray()}
    selector = selectors.DefaultSelector()
    try:
        process = subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, cwd="/", env=env, start_new_session=True)
        for stream, name in ((process.stdout, "stdout"), (process.stderr, "stderr")):
            selector.register(stream, selectors.EVENT_READ, name)
        while selector.get_map():
            left = deadline - time.monotonic()
            if left <= 0:
                raise TimeoutError("60-second readiness budget exhausted")
            for key, _ in selector.select(min(left, 0.25)):
                block = os.read(key.fd, 65536)
                if not block:
                    selector.unregister(key.fileobj)
                else:
                    data[key.data].extend(block)
                    if sum(map(len, data.values())) > 1024**2:
                        raise ValueError("worker output exceeded the frozen 1-MiB sublimit")
        return process.wait(timeout=max(0.001, deadline-time.monotonic())), bytes(data["stdout"]), bytes(data["stderr"])
    except (TimeoutError, ValueError, subprocess.TimeoutExpired) as error:
        raise CaptureStop(error, data) from error
    finally:
        selector.close()
        if process is not None:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait()
            process.stdout.close()
            process.stderr.close()
        # Execution has ended and the child is reaped. Do not let the same
        # timer discard captured bytes while the coordinator persists evidence.
        signal.setitimer(signal.ITIMER_REAL, 0)


def run(output: Path, review_path: Path) -> dict:
    started = time.monotonic()
    manifest_path = output / "manifest.json"
    manifest_record = contract.record(manifest_path)
    manifest = json.loads(manifest_path.read_bytes())
    review = json.loads(review_path.read_bytes())
    if (review.get("status") != "APPROVED_FOR_SINGLE_READINESS_ATTEMPT"
            or review.get("manifest_sha256") != manifest_record["sha256"]):
        raise ValueError("independent review does not admit these exact bindings")
    # Exclusive creation is the attempt lock. No failure removes or resets it.
    contract.exclusive(output / "started.json", {
        "schema": "uor-r4.isolated-runtime-started/1", "issue": 1096,
        "manifest": manifest_record, "review": contract.record(review_path), "limits": LIMITS,
        "model_loads": 0, "model_forwards": 0, "optimizer_updates": 0,
    })
    events, command = [], []
    status, reason = "UNAVAILABLE_ISOLATED_RUNTIME", None
    previous_alarm = signal.signal(signal.SIGALRM, deadline_alarm)
    deadline = started + LIMITS["seconds"]
    signal.setitimer(signal.ITIMER_REAL, max(0.001, deadline-time.monotonic()))
    try:
        if manifest["limits"] != LIMITS or manifest["runtime"] != contract.RUNTIME:
            raise ValueError("frozen limits or runtime changed")
        # Raw capture is <=1 MiB (+one 64-KiB read); reserve 8 MiB for its
        # retained bytes, JSON event expansion, terminal and resource receipts.
        if new_bytes(output) + review_path.stat().st_size + 8 * 1024**2 > LIMITS["new_bytes"]:
            raise TimeoutError("insufficient space within the frozen receipt byte budget")
        for item in [manifest["bindings"], manifest["profile"], manifest["previous_stop"]] + manifest["runtime_files"]:
            contract.verify_record(item)
        binding = json.loads(Path(manifest["bindings"]["path"]).read_bytes())
        for item in binding["source_files"] + list(binding["assets"].values()):
            contract.verify_record(item)
        for item in manifest["probes"].values():
            contract.verify_record(item)
            if Path(item["path"]).read_bytes() != PROBE_BYTES:
                raise ValueError("probe is not the harmless frozen payload")
        interpreter = manifest["interpreter"]
        if contract.interpreter_links(Path(interpreter["launcher"])) != interpreter["links"]:
            raise ValueError("interpreter aliases drifted")
        if contract.hardware_identity() != manifest["hardware"]:
            raise ValueError("hardware changed")
        repo = Path(manifest["repo"])
        expected_profile = contract.sandbox_profile(repo, Path(interpreter["launcher"]),
            Path(manifest["bindings"]["path"]), binding["assets"],
            extra_read_files=(manifest_path, Path(manifest["profile"]["path"])))
        if expected_profile.encode() != Path(manifest["profile"]["path"]).read_bytes():
            raise ValueError("profile differs from the reviewed generator")
        # A small explicit environment excludes PYTHONHOME, old PYTHONPATH,
        # dynamic-loader injection and user-site imports.
        env = {"PATH": "/usr/bin:/bin:/usr/sbin:/sbin", "HOME": str(Path.home()),
               "PYTHONPATH": str(repo / "tools/r4-softmax-trainer/src"),
               "PYTHONNOUSERSITE": "1", "PYTHONDONTWRITEBYTECODE": "1", "PYTHONUNBUFFERED": "1",
               "OMP_NUM_THREADS": "4", "VECLIB_MAXIMUM_THREADS": "4"}
        command = ["/usr/bin/sandbox-exec", "-f", manifest["profile"]["path"],
                   interpreter["launcher"], "-m", "r4_softmax_trainer.text_clause_adapter.worker",
                   "--bindings", manifest["bindings"]["path"], "--arm", "adapter", "--readiness-only",
                   "--readiness-manifest", str(manifest_path), "--readiness-sha256", manifest_record["sha256"]]
        if deadline <= time.monotonic():
            raise TimeoutError("binding verification exhausted the readiness budget")
        code, stdout, stderr = receive(command, env, deadline)
        write_bytes(output / "worker.stdout.jsonl", stdout)
        write_bytes(output / "worker.stderr.txt", stderr)
        parsed = [json.loads(line) for line in stdout.splitlines() if line]
        if any(type(event) is not dict for event in parsed):
            raise ValueError("worker emitted a non-object event")
        events = parsed
        if code != 0 or len(events) != 2 or [e.get("event") for e in events] != ["ready", "done"]:
            raise ValueError(f"worker did not complete readiness: exit={code}; stderr={stderr[:4096].decode(errors='replace')}")
        for event in events:
            identity = event.get("readiness_identity", {})
            if (event.get("bindings_sha256") != manifest["bindings"]["sha256"]
                    or event.get("runtime") != contract.RUNTIME
                    or event.get("deterministic_algorithms") is not True
                    or any(event.get(k) != 0 for k in ("model_loads", "row_forwards", "batch_forwards"))
                    or identity.get("manifest_sha256") != manifest_record["sha256"]
                    or identity.get("profile_sha256") != manifest["profile"]["sha256"]
                    or identity.get("denied_probes") != {name: True for name in manifest["probes"]}):
                raise ValueError("worker identity, probes or zero-forward counts differ")
        first, last = events
        if (first.get("status") != "ARTIFACTS_READY" or first.get("states") is not None
                or first.get("isolation_denied") is not True
                or last.get("states_before") is not None or last.get("states_after") is not None
                or last.get("audit", {}).get("optimizer_updates") != 0
                or last.get("audit", {}).get("isolation_denied") is not True):
            raise ValueError("worker states, isolation or optimizer audit differ")
        status = "ISOLATED_RUNTIME_READY"
    except Exception as error:
        signal.setitimer(signal.ITIMER_REAL, 0)
        reason = f"{type(error).__name__}: {error}"
        if isinstance(error, CaptureStop):
            write_bytes(output / "worker.stdout.jsonl", error.stdout)
            write_bytes(output / "worker.stderr.txt", error.stderr)
            for line in error.stdout.splitlines():
                try:
                    event = json.loads(line)
                    if isinstance(event, dict):
                        events.append(event)
                except ValueError:
                    pass  # The exact partial bytes remain in worker.stdout.jsonl.
        if isinstance(error, (TimeoutError, CaptureStop)):
            status = "INCOMPLETE_RESOURCE"
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
        signal.signal(signal.SIGALRM, previous_alarm)
    elapsed = time.monotonic()-started
    parent_rss = int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)
    child_rss = int(resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss)
    if elapsed > LIMITS["seconds"] or parent_rss + child_rss > LIMITS["peak_rss_bytes"]:
        status, reason = "INCOMPLETE_RESOURCE", reason or "time or combined peak RSS cap exceeded"
    result = {"schema": "uor-r4.isolated-runtime-result/1", "issue": 1096, "status": status,
              "reason": reason, "manifest_sha256": manifest_record["sha256"], "command": command,
              "worker_events": events, "elapsed_seconds": elapsed,
              "combined_peak_rss_bound_bytes": parent_rss + child_rss,
              "model_loads": max([0] + [e.get("model_loads", 0) for e in events]),
              "model_forwards": max([0] + [e.get("row_forwards", 0) for e in events]),
              "optimizer_updates": max([0] + [e.get("audit", {}).get("optimizer_updates", 0) for e in events]),
              "withheld_comparison": "NOT_RUN", "model_replay": "NOT_RUN"}
    # Leave ample room for both the terminal and its resource receipt.
    if new_bytes(output) + len(contract.canonical(result)) + 4096 > LIMITS["new_bytes"]:
        result.update(status="INCOMPLETE_RESOURCE", reason="receipt byte cap exceeded")
    result_record = contract.exclusive(output / "result.json", result)
    contract.exclusive(output / "resources.json", {"result": result_record,
        "bytes_before_resource_receipt": new_bytes(output), "limits": LIMITS})
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("phase", choices=("freeze", "run"))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--repo", type=Path)
    parser.add_argument("--python", type=Path)
    parser.add_argument("--review", type=Path)
    args = parser.parse_args()
    if args.phase == "freeze":
        if not args.repo or not args.python:
            parser.error("freeze requires repo and python")
        result = freeze(args.repo.resolve(), args.python.absolute(), args.output.absolute())
    else:
        if not args.review:
            parser.error("run requires independent review")
        result = run(args.output.absolute(), args.review.resolve())
    print(json.dumps(result, indent=2))
    return 0 if args.phase == "freeze" or result["status"] == "ISOLATED_RUNTIME_READY" else 1


if __name__ == "__main__":
    raise SystemExit(main())
