"""External, one-envelope resource supervision for the frozen #1102 bridge.

This module has no model, exporter, fixture, or project imports. Merely importing
it does not start work. ``supervise`` is called by the external parent; the
coordinator calls ``phase`` and ``complete`` over inherited, acknowledged pipes.
All coordinator and worker output must remain below the declared run root.
The filesystem ledger is a sampled high-water ledger, not an OS write sandbox:
the independently reviewed coordinator must use exclusive retained outputs.

An accepted result requires supervisor-completion.json AND supervisor-wall.json,
and the absence of supervisor-stop.json. A stop always overrides completion.
No invocation can reuse an existing supervisor-consumed.json envelope.
"""

from __future__ import annotations

import json
import os
from pathlib import Path
import selectors
import signal
import stat
import subprocess
import time
from typing import Any


PHASES = ("export_integrity", "execution", "replay")
TERMINALS = frozenset({
    "NATIVE_REFERENCE_PRESERVED",
    "NATIVE_REFERENCE_MISMATCH",
    "UNAVAILABLE_NATIVE_REFERENCE",
    "ABORTED_NATIVE_REFERENCE_BUDGET",
})
AUTHORING_BYTES = 672_846
RUN_BYTES = 128 * 1024 * 1024
RSS_BYTES = 3 * 1024 * 1024 * 1024
BUILD_BYTES = 2 * 1024 * 1024 * 1024
SAMPLE_SECONDS = 0.1
MAX_EVENT_BYTES = 8192
SCHEMA = "uor-r4.native-reference-supervisor/1"


class SupervisionFailure(RuntimeError):
    """A measured cap, interruption, protocol, or monitoring failure."""


class AdmissionUnavailable(RuntimeError):
    """The coordinator executable could not start; no child work began."""


def _encoded(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False) + "\n").encode("ascii")


def _directory_sync(path: Path) -> None:
    fd = os.open(path, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def _exclusive(path: Path, value: Any) -> None:
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0)
    fd = os.open(path, flags, 0o600)
    try:
        data = _encoded(value)
        with os.fdopen(fd, "wb", closefd=False) as stream:
            stream.write(data)
            stream.flush()
            os.fsync(fd)
        os.fchmod(fd, 0o444)
    finally:
        os.close(fd)
    _directory_sync(path.parent)


def _checked_root(path: Path) -> Path:
    path = Path(os.path.abspath(path))
    # Reject a symlink at any existing component; resolve() would hide it.
    for item in reversed((path, *path.parents)):
        if item.exists() or item.is_symlink():
            if stat.S_ISLNK(item.lstat().st_mode):
                raise ValueError(f"symlink in supervised root: {item}")
    path.mkdir(parents=True, exist_ok=True)
    if not path.is_dir():
        raise ValueError("supervised root is not a directory")
    return path


class _Ledger:
    def __init__(self, roots: tuple[Path, ...], fixed_bytes: int, immutable: bool):
        self.roots = roots
        self.fixed_bytes = fixed_bytes
        self.immutable = immutable
        self.seen: dict[str, tuple[int, int, int]] = {}
        self.generations: dict[tuple[str, int, int], int] = {}
        self.high_water = fixed_bytes
        self.file_count = 0

    def sample(self) -> int:
        present: set[str] = set()
        for root in self.roots:
            pending = [root]
            while pending:
                directory = pending.pop()
                try:
                    entries = os.scandir(directory)
                except FileNotFoundError:
                    if self.immutable or directory == root:
                        raise
                    continue  # Cargo removed an already observed temp directory.
                with entries:
                    for entry in entries:
                        try:
                            info = entry.stat(follow_symlinks=False)
                        except FileNotFoundError:
                            if self.immutable:
                                raise
                            continue  # Cargo rename/unlink raced this sample.
                        if stat.S_ISDIR(info.st_mode):
                            pending.append(Path(entry.path))
                            continue
                        if not stat.S_ISREG(info.st_mode):
                            raise SupervisionFailure(
                                f"nonregular ledger entry: {entry.path}")
                        key = entry.path
                        present.add(key)
                        current = (info.st_dev, info.st_ino, info.st_size)
                        old = self.seen.get(key)
                        if self.immutable and old is not None and current[:2] != old[:2]:
                            raise SupervisionFailure(f"ledger file replaced: {key}")
                        if self.immutable and old is not None and current[2] < old[2]:
                            raise SupervisionFailure(f"ledger file shrank: {key}")
                        self.seen[key] = current
                        generation = (key, current[0], current[1])
                        self.generations[generation] = max(
                            current[2], self.generations.get(generation, 0))
        missing = set(self.seen) - present
        if missing and self.immutable:
            raise SupervisionFailure(f"ledger file deleted: {sorted(missing)[0]}")
        # Build tools legitimately rename and remove their scratch outputs.
        # Their observed prior generations remain charged, never subtracted.
        total = self.fixed_bytes + sum(self.generations.values())
        self.high_water = max(self.high_water, total)
        self.file_count = len(present)
        return total


def _process_rss(root_pid: int) -> tuple[int, set[int]]:
    # Inspect only PID, PPID, and resident KiB; never print command arguments.
    probe = subprocess.run(
        ["/bin/ps", "-axo", "pid=,ppid=,pgid=,rss="], capture_output=True,
        text=True, timeout=1.0, check=True,
    )
    rows: dict[int, tuple[int, int, int]] = {}
    for line in probe.stdout.splitlines():
        parts = line.split()
        if len(parts) != 4:
            raise SupervisionFailure("unexpected ps resource row")
        pid, parent, group, rss_kib = map(int, parts)
        if min(pid, parent, group, rss_kib) < 0:
            raise SupervisionFailure("negative ps resource field")
        rows[pid] = (parent, group, rss_kib * 1024)
    # Same-group descendants remain visible after the coordinator exits.
    descendants = {pid for pid, (_, group, _) in rows.items() if group == root_pid}
    if root_pid in rows:
        descendants.add(root_pid)
    while True:
        enlarged = descendants | {
            pid for pid, (parent, _, _) in rows.items() if parent in descendants
        }
        if enlarged == descendants:
            break
        descendants = enlarged
    return sum(rows[pid][2] for pid in descendants if pid in rows), descendants


def _event(kind: str, **fields: Any) -> None:
    """Send one small event and wait for external acknowledgement."""
    try:
        output_fd = int(os.environ["R4_NATIVE_BRIDGE_EVENTS_FD"])
        ack_fd = int(os.environ["R4_NATIVE_BRIDGE_ACK_FD"])
    except (KeyError, ValueError) as error:
        raise RuntimeError("an external #1102 supervisor is required") from error
    data = _encoded({"kind": kind, **fields})
    if len(data) > MAX_EVENT_BYTES:
        raise ValueError("supervisor event exceeds its byte bound")
    view = memoryview(data)
    while view:
        written = os.write(output_fd, view)
        if written <= 0:
            raise RuntimeError("supervisor event pipe closed")
        view = view[written:]
    reply = bytearray()
    while not reply.endswith(b"\n") and len(reply) < 16:
        chunk = os.read(ack_fd, 1)
        if not chunk:
            raise RuntimeError("supervisor acknowledgement pipe closed")
        reply.extend(chunk)
    if reply != b"OK\n":
        raise RuntimeError("supervisor refused the event")


def phase(name: str) -> None:
    """Advance to execution, then replay. No event resets an existing clock."""
    if name not in PHASES[1:]:
        raise ValueError("only execution and replay are explicit transitions")
    _event("phase", phase=name)


def progress(details: dict[str, Any]) -> None:
    """Attach bounded coordinator progress without changing any clock."""
    _event("progress", details=details)


def complete(terminal: str, details: dict[str, Any] | None = None) -> None:
    """Request completion; subsequent writes and process exit remain charged."""
    if terminal not in TERMINALS:
        raise ValueError("invalid completed comparison terminal")
    _event("complete", terminal=terminal, details=details or {})


def _kill_tree(process: subprocess.Popen[bytes], known: set[int]) -> None:
    # Refresh before killing individual PIDs, so historical exited worker PIDs
    # cannot accidentally target a later unrelated process after PID reuse.
    try:
        _, live = _process_rss(process.pid)
    except (OSError, ValueError, subprocess.SubprocessError, SupervisionFailure):
        live = set()
    for sig in (signal.SIGTERM, signal.SIGKILL):
        try:
            os.killpg(process.pid, sig)
        except ProcessLookupError:
            pass
        for pid in (known & live) - {process.pid}:
            try:
                os.kill(pid, sig)
            except ProcessLookupError:
                pass
        if sig == signal.SIGTERM:
            try:
                process.wait(timeout=0.25)
            except subprocess.TimeoutExpired:
                pass
    process.wait(timeout=2.0)


def _run(
    command: list[str], cwd: str, env: dict[str, str], receipt_root: Path,
    release_sha256: str, ledger_roots: tuple[Path, ...], fixed_bytes: int,
    byte_cap: int, total_seconds: float, comparison: bool,
) -> int:
    if not command or not all(isinstance(item, str) for item in command):
        raise ValueError("a nonempty explicit argv is required")
    if len(release_sha256) != 64 or any(c not in "0123456789abcdef"
                                        for c in release_sha256):
        raise ValueError("release_sha256 must be a complete lowercase SHA256")
    started = time.monotonic()
    active = "export_integrity" if comparison else "native_build"
    phase_started = started
    phase_elapsed: dict[str, float] = {}
    receipt_root = _checked_root(receipt_root)
    consumed = receipt_root / "supervisor-consumed.json"
    # This exclusive file is the one-way admission latch, not a retry marker.
    _exclusive(consumed, {"schema": SCHEMA, "release_sha256": release_sha256,
                          "mode": "comparison" if comparison else "build",
                          "status": "CONSUMED", "pid": os.getpid(),
                          "started_monotonic": started})
    ledger = _Ledger(ledger_roots, fixed_bytes, immutable=comparison)
    journal_path = receipt_root / "supervisor-progress.jsonl"
    journal = journal_path.open("xb", buffering=0)
    event_read, event_write = os.pipe()
    ack_read, ack_write = os.pipe()
    process: subprocess.Popen[bytes] | None = None
    stdout = stderr = None
    known: set[int] = set()
    rss_peak = 0
    sampled = 0
    last_progress = started
    requested: dict[str, Any] | None = None
    interrupted: list[int] = []
    old_handlers: dict[int, Any] = {}
    selector = selectors.DefaultSelector()

    def record(kind: str, **fields: Any) -> None:
        journal.write(_encoded({"schema": SCHEMA, "kind": kind,
                               "elapsed_seconds": time.monotonic() - started,
                               "phase": active, **fields}))
        os.fsync(journal.fileno())

    def check_clocks() -> float:
        now = time.monotonic()
        if interrupted:
            raise SupervisionFailure(f"external interruption signal {interrupted[0]}")
        if now - started > total_seconds:
            raise SupervisionFailure("cumulative wall budget exceeded")
        if comparison and now - phase_started > 120.0:
            raise SupervisionFailure(f"{active} wall budget exceeded")
        return now

    def sample() -> None:
        nonlocal rss_peak, sampled, known
        check_clocks()
        if process is not None:
            rss, descendants = _process_rss(process.pid)
            known |= descendants
            rss_peak = max(rss_peak, rss)
            if comparison and rss > RSS_BYTES:
                raise SupervisionFailure("combined coordinator/worker RSS cap exceeded")
            if process.poll() is not None and descendants - {process.pid}:
                raise SupervisionFailure("worker remained alive after coordinator exit")
        measured_bytes = ledger.sample()
        sampled += 1
        if measured_bytes > byte_cap:
            raise SupervisionFailure("complete retained byte ledger cap exceeded")
        check_clocks()

    try:
        for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
            old_handlers[sig] = signal.getsignal(sig)
            signal.signal(sig, lambda received, _frame: interrupted.append(received))
        _exclusive(receipt_root / "supervisor-start.json", {
            "schema": SCHEMA, "release_sha256": release_sha256,
            "started_monotonic": started, "initial_phase": active,
            "phase_seconds": 120 if comparison else 900,
            "cumulative_seconds": total_seconds,
            "rss_bytes": RSS_BYTES if comparison else None,
            "byte_cap": byte_cap, "fixed_original_authoring_bytes": fixed_bytes,
            "ledger_roots": [str(path) for path in ledger_roots],
            "sample_interval_seconds": SAMPLE_SECONDS,
            "rss_scope": "coordinator and all discovered descendants",
            "ledger_policy": ("retained paths; no deletion, shrinking, or replacement"
                              if comparison else "observed file generations remain charged after build cleanup"),
        })
        sample()
        stdout = (receipt_root / "coordinator.stdout.log").open("xb", buffering=0)
        stderr = (receipt_root / "coordinator.stderr.log").open("xb", buffering=0)
        child_env = dict(env)
        child_env.update({"R4_NATIVE_BRIDGE_EVENTS_FD": str(event_write),
                          "R4_NATIVE_BRIDGE_ACK_FD": str(ack_read),
                          "R4_NATIVE_BRIDGE_RUN_ROOT": str(receipt_root),
                          "R4_NATIVE_BRIDGE_RELEASE_SHA256": release_sha256})
        try:
            process = subprocess.Popen(command, cwd=cwd, env=child_env, stdin=subprocess.DEVNULL,
                                       stdout=stdout, stderr=stderr, start_new_session=True,
                                       pass_fds=(event_write, ack_read), close_fds=True)
        except OSError as error:
            raise AdmissionUnavailable(f"coordinator launch unavailable: {error}") from error
        known.add(process.pid)
        os.close(event_write)
        event_write = -1
        os.close(ack_read)
        ack_read = -1
        os.set_blocking(event_read, False)
        selector.register(event_read, selectors.EVENT_READ)
        record("started", child_pid=process.pid)
        buffer = bytearray()
        pipe_open = True
        while True:
            sample()
            now = time.monotonic()
            if now - last_progress >= 1.0:
                record("sample", rss_peak_bytes=rss_peak,
                       retained_ledger_bytes=ledger.high_water,
                       sampled_file_count=ledger.file_count, samples=sampled)
                last_progress = now
            remaining = total_seconds - (now - started)
            if comparison:
                remaining = min(remaining, 120.0 - (now - phase_started))
            ready = selector.select(timeout=max(0.0, min(SAMPLE_SECONDS, remaining)))
            if ready:
                chunk = os.read(event_read, MAX_EVENT_BYTES + 1)
                if not chunk:
                    selector.unregister(event_read)
                    pipe_open = False
                else:
                    buffer.extend(chunk)
                if len(buffer) > MAX_EVENT_BYTES:
                    raise SupervisionFailure("coordinator event exceeds byte bound")
                while b"\n" in buffer:
                    line, _, rest = buffer.partition(b"\n")
                    buffer = bytearray(rest)
                    message = json.loads(line)
                    if not comparison or not isinstance(message, dict):
                        raise SupervisionFailure("unexpected coordinator event")
                    kind = message.get("kind")
                    if requested is not None:
                        raise SupervisionFailure("event follows completion request")
                    if kind == "phase" and set(message) == {"kind", "phase"}:
                        index = PHASES.index(active)
                        if index + 1 >= len(PHASES) or message["phase"] != PHASES[index + 1]:
                            raise SupervisionFailure("out-of-order or repeated phase")
                        switched = check_clocks()
                        phase_elapsed[active] = switched - phase_started
                        record("phase_finished", phase_seconds=phase_elapsed[active])
                        active = message["phase"]
                        # Journal/ack latency is charged to the newly active phase.
                        phase_started = switched
                        record("phase_started")
                    elif kind == "progress" and set(message) == {"kind", "details"}:
                        if not isinstance(message["details"], dict):
                            raise SupervisionFailure("invalid progress details")
                        record("coordinator_progress", details=message["details"])
                    elif kind == "complete" and set(message) == {"kind", "terminal", "details"}:
                        if message["terminal"] not in TERMINALS or not isinstance(message["details"], dict):
                            raise SupervisionFailure("invalid completion request")
                        if message["terminal"] == "NATIVE_REFERENCE_PRESERVED" and active != "replay":
                            raise SupervisionFailure("preserved terminal before replay")
                        if message["terminal"] == "ABORTED_NATIVE_REFERENCE_BUDGET":
                            record("coordinator_stop", details=message["details"])
                            raise SupervisionFailure("coordinator reported ABORTED_NATIVE_REFERENCE_BUDGET")
                        requested = message
                        record("completion_requested", terminal=message["terminal"])
                    else:
                        raise SupervisionFailure("unknown or malformed event")
                    sample()
                    os.write(ack_write, b"OK\n")
            if process.poll() is not None and not pipe_open:
                if buffer:
                    raise SupervisionFailure("incomplete final protocol event")
                break
        exit_code = process.wait()
        if exit_code != 0:
            raise SupervisionFailure(f"child exited with status {exit_code}")
        if comparison and requested is None:
            raise SupervisionFailure("child exited without a final completion request")
        if not comparison:
            requested = {"terminal": "NATIVE_BUILD_COMPLETED", "details": {}}
        stdout.flush()
        stderr.flush()
        os.fsync(stdout.fileno())
        os.fsync(stderr.fileno())
        sample()
        phase_elapsed[active] = time.monotonic() - phase_started
        record("child_exit_verified", exit_code=exit_code, phase_seconds=phase_elapsed[active])
        _exclusive(receipt_root / "supervisor-completion.json", {
            "schema": SCHEMA, "release_sha256": release_sha256,
            "terminal": requested["terminal"], "details": requested["details"],
            "child_exit_code": exit_code, "phase_seconds": phase_elapsed,
            "elapsed_seconds": time.monotonic() - started,
            "rss_peak_bytes": rss_peak, "samples": sampled,
            "ledger_bytes_before_final_receipts": ledger.high_water,
            "stop_overrides_completion": True,
        })
        sample()
        wall = {
            "schema": SCHEMA, "release_sha256": release_sha256,
            "elapsed_seconds_after_completion_write": time.monotonic() - started,
            "active_phase_seconds_after_completion_write": time.monotonic() - phase_started,
            "ledger_bytes_after_completion_write": ledger.high_water,
            "ledger_bytes_including_wall_receipt": ledger.high_water,
            "coverage": "startup, identity work, child output/fsync/exit, completion write",
            "wall_receipt_write_checked_before_return": True,
            "stop_overrides_completion": True,
        }
        while True:
            inclusive = ledger.high_water + len(_encoded(wall))
            if inclusive == wall["ledger_bytes_including_wall_receipt"]:
                break
            wall["ledger_bytes_including_wall_receipt"] = inclusive
        _exclusive(receipt_root / "supervisor-wall.json", wall)
        sample()  # Charges this final wall receipt; any overrun writes a stop.
        os.fchmod(journal.fileno(), 0o444)
        os.fsync(journal.fileno())
        check_clocks()
        return 0
    except (Exception, KeyboardInterrupt) as error:
        if comparison and process is None and isinstance(error, AdmissionUnavailable):
            # The frozen missing-runtime branch is unavailable. In contrast,
            # interruption/protocol failure after launch lacks a final worker
            # receipt and therefore uses the frozen ABORTED terminal below.
            try:
                sample()
                record("admission_unavailable", reason=str(error))
                _exclusive(receipt_root / "supervisor-completion.json", {
                    "schema": SCHEMA, "release_sha256": release_sha256,
                    "terminal": "UNAVAILABLE_NATIVE_REFERENCE",
                    "details": {"reason": str(error), "child_started": False},
                    "child_exit_code": None,
                    "phase_seconds": {active: time.monotonic() - phase_started},
                    "elapsed_seconds": time.monotonic() - started,
                    "rss_peak_bytes": 0, "samples": sampled,
                    "ledger_bytes_before_final_receipts": ledger.high_water,
                    "stop_overrides_completion": True,
                })
                sample()
                _exclusive(receipt_root / "supervisor-wall.json", {
                    "schema": SCHEMA, "release_sha256": release_sha256,
                    "elapsed_seconds_after_completion_write": time.monotonic() - started,
                    "active_phase_seconds_after_completion_write": time.monotonic() - phase_started,
                    "ledger_bytes_after_completion_write": ledger.high_water,
                    "coverage": "admission attempt and completion write; child never started",
                    "wall_receipt_write_checked_before_return": True,
                    "stop_overrides_completion": True,
                })
                sample()
                return 0
            except Exception as tail_error:
                error = SupervisionFailure(f"unavailable-admission receipt incomplete: {tail_error}")
        if process is not None:
            try:
                _kill_tree(process, known)
            except (OSError, subprocess.SubprocessError) as kill_error:
                error = SupervisionFailure(f"{error}; kill failure: {kill_error}")
        try:
            ledger.sample()
        except Exception:
            pass  # Keep the observed high water; never replace it with zero.
        phase_elapsed[active] = time.monotonic() - phase_started
        _exclusive(receipt_root / "supervisor-stop.json", {
            "schema": SCHEMA, "release_sha256": release_sha256,
            "terminal": "ABORTED_NATIVE_REFERENCE_BUDGET" if comparison else "NATIVE_BUILD_STOPPED",
            "reason": str(error), "phase_seconds": phase_elapsed,
            "elapsed_seconds": time.monotonic() - started,
            "rss_peak_bytes": rss_peak, "retained_ledger_high_water_bytes": ledger.high_water,
            "samples": sampled, "child_exit_code": process.poll() if process else None,
            "stop_overrides_completion": True,
        })
        return 124
    finally:
        for sig, handler in old_handlers.items():
            signal.signal(sig, handler)
        selector.close()
        for fd in (event_read, event_write, ack_read, ack_write):
            if fd >= 0:
                os.close(fd)
        journal.close()
        if stdout is not None:
            stdout.close()
        if stderr is not None:
            stderr.close()


def supervise(command: list[str], cwd: str, env: dict[str, str], run_root: Path,
              release_sha256: str) -> int:
    """Run one admitted comparison; this call never retries or resets clocks."""
    root = _checked_root(run_root)
    return _run(command, cwd, env, root, release_sha256, (root,), AUTHORING_BYTES,
                RUN_BYTES, 360.0, True)


def supervise_build(command: list[str], cwd: str, env: dict[str, str],
                    build_root: Path, receipt_root: Path,
                    release_sha256: str) -> int:
    """One offline release-build invocation; requires an empty unique target.

    Callers must include the frozen release/offline/compiler flags in command.
    This function supplies only CARGO_TARGET_DIR and the independently enforced
    time/output limits. Compiler failure consumes this preparation invocation;
    it does not silently retry or permit a later invocation to reset its clock.
    """
    target = _checked_root(build_root)
    receipts = _checked_root(receipt_root)
    if target == receipts or target in receipts.parents or receipts in target.parents:
        raise ValueError("build target and receipt root must be disjoint")
    if any(target.iterdir()):
        raise ValueError("build target must be a new empty directory")
    build_env = dict(env)
    build_env["CARGO_TARGET_DIR"] = str(target)
    return _run(command, cwd, build_env, receipts, release_sha256,
                (target, receipts), 0, BUILD_BYTES, 900.0, False)
