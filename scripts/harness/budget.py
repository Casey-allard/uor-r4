"""Cumulative cost, live compute and the disk floor - all read-only.

The harness cannot spend money and must never mutate pod state. What it can do is make
three things visible, because a session that cannot see them cannot be held to them:

1. **Cumulative cost.** Receipts in ``docs/harness/ledger/*.jsonl`` that carry cost
   fields, summed by month and by milestone; plus any tracked cumulative ledger the repo
   already keeps (the requested ``.uor-models/native-joint-learning-2026-09-04/model-time.json``
   and the ``resource-ledger.json`` chain under ``docs/labs/``), and the local pod event
   ledger under ``$UOR_POD_STATE`` when it is present. When a ledger is not there the
   answer is ``ABSENT`` and the path that was checked, never a guessed number.
2. **Live compute.** ``budget scan`` runs only the read-only status subcommands of
   ``scripts/pod/uor-pod``, parses the running pods, GPU specs and hourly cost, and prints
   the total against the owner caps (<= 4 running pods, <= $8.00/h across all labs).
   If ``uor-pod`` is absent or errors, the answer is ``UNAVAILABLE`` with the exact
   command and exit code; nothing is fabricated and no pod state is touched.
3. **Disk.** Free space against the owner's 30-70 GB working band and the 128 MiB storage
   stop margin, with the hygiene command that clears it.
"""

from __future__ import annotations

import json
import os
import re
import shutil
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Iterable

from core import HarnessPaths, emit, run
from checkrules import Rule, RuleResult, register as register_rule
from registries import Registry
from util import ABSENT, MISSING, truncate

# --------------------------------------------------------------------------- constants

#: owner caps (AGENTS.md "GPU pods"; docs/labs/compute.md section "Caps").
POD_CAP = 4
RATE_CAP_USD_PER_H = 8.00

#: the owner's local working band and the retained-evidence stop margin (AGENTS.md).
DISK_BAND_GB = (30, 70)
STORAGE_MARGIN_MIB = 128

#: the cumulative ledger the task names, plus the per-experiment chain the repo keeps.
MODEL_TIME_LEDGER = ".uor-models/native-joint-learning-2026-09-04/model-time.json"
RESOURCE_LEDGER_GLOB = "*/resource-ledger.json"

#: receipts may record a cost under any of these names.
COST_KEYS = ("cost_usd", "usd", "usd_total", "spend_usd", "cost")
#: rates are not totals - they are reported separately so they cannot be summed as spend.
RATE_KEYS = ("usd_per_h", "rate_usd_per_h", "usd_per_hour")

#: the pod script is read-only here; these are its status subcommands.
POD_SCRIPT_REL = "scripts/pod/uor-pod"
POD_READONLY = ("status", "gpus", "log")
POD_TIMEOUT_S = 25

HYGIENE_COMMAND = "scripts/storage/uor-hygiene --apply"


# ----------------------------------------------------------------------------- helpers


def _rel(path: Path, root: Path) -> str:
    try:
        return str(Path(path).resolve().relative_to(Path(root).resolve()))
    except (ValueError, OSError):
        return str(path)


def _num(value: Any) -> float | None:
    if isinstance(value, bool):
        return None
    if isinstance(value, (int, float)):
        return float(value)
    if isinstance(value, str):
        m = re.search(r"-?[0-9]+(?:\.[0-9]+)?", value.replace(",", ""))
        if m:
            try:
                return float(m.group(0))
            except ValueError:  # pragma: no cover - regex guarantees a number
                return None
    return None


def _read_jsonl_tolerant(path: Path) -> list[dict[str, Any]]:
    """Read a JSONL file, skipping the lines that do not parse.

    ``core.read_jsonl`` opens in strict UTF-8, so one undecodable byte anywhere in the
    file loses every record after it. A ledger is append-only evidence, so a corrupt
    line must cost that line only. The strict reader still owns schema questions; this
    one only decides what can be read.
    """
    if not path.is_file():
        return []
    out: list[dict[str, Any]] = []
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return []
    for line in text.splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(record, dict):
            out.append(record)
    return out


def _gb(num_bytes: int) -> float:
    return num_bytes / (1024.0**3)


def disk_free(path: Path) -> dict[str, Any]:
    """Free space where the work happens, measured - never assumed."""
    try:
        usage = shutil.disk_usage(str(path))
    except OSError as exc:  # pragma: no cover - environment
        return {"ok": False, "error": f"{type(exc).__name__}: {exc}", "path": str(path)}
    free_gb = _gb(usage.free)
    low, high = DISK_BAND_GB
    return {
        "ok": True,
        "path": str(path),
        "free_bytes": usage.free,
        "total_bytes": usage.total,
        "free_gb": round(free_gb, 2),
        "band_gb": [low, high],
        "in_band": low <= free_gb <= high,
        "below_floor": free_gb < low,
        "above_band": free_gb > high,
        "margin_mib": STORAGE_MARGIN_MIB,
        "hygiene": HYGIENE_COMMAND,
    }


def pod_state_dir() -> Path:
    env = os.environ.get("UOR_POD_STATE")
    if env:
        return Path(env).expanduser()
    return Path.home() / ".local" / "share" / "uor-r4" / "compute"


# ------------------------------------------------------------------- receipts / cost


def _receipt_costs(recs: Iterable[dict[str, Any]]) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    """Split receipts into (cost entries, rate-only entries).

    A rate is not a charge: counting ``usd_per_h`` as money would overstate the ledger.
    """
    costs: list[dict[str, Any]] = []
    rates: list[dict[str, Any]] = []
    for rec in recs:
        month = str(rec.get("ts", ""))[:7] or "?"
        amount = None
        key_used = ""
        for key in COST_KEYS:
            if key in rec:
                amount = _num(rec.get(key))
                key_used = key
                if amount is not None:
                    break
        if amount is not None:
            costs.append(
                {
                    "ts": rec.get("ts", ""),
                    "month": month,
                    "kind": rec.get("kind", ""),
                    "milestone": rec.get("milestone", "") or "",
                    "field": key_used,
                    "usd": round(amount, 4),
                    "file": rec.get("_file", ""),
                }
            )
            continue
        for key in RATE_KEYS:
            if key in rec:
                rate = _num(rec.get(key))
                if rate is not None:
                    rates.append(
                        {
                            "ts": rec.get("ts", ""),
                            "month": month,
                            "milestone": rec.get("milestone", "") or "",
                            "field": key,
                            "usd_per_h": rate,
                        }
                    )
                break
    return costs, rates


def _sum_by(rows: list[dict[str, Any]], key: str) -> dict[str, float]:
    out: dict[str, float] = {}
    for row in rows:
        k = str(row.get(key) or "(unset)")
        out[k] = round(out.get(k, 0.0) + float(row["usd"]), 4)
    return dict(sorted(out.items()))


def resource_ledger(root: Path) -> dict[str, Any]:
    """The per-experiment ``resource-ledger.json`` chain the repo actually keeps."""
    if not root.is_dir():
        return {"present": False, "reason": ABSENT, "checked": str(root)}
    files = sorted(root.glob(RESOURCE_LEDGER_GLOB))
    entries: list[dict[str, Any]] = []
    for f in files:
        try:
            data = json.loads(f.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        if not isinstance(data, dict):
            continue
        entries.append(
            {
                "path": _rel(f, root),
                "schema": str(data.get("schema", "")),
                "status": data.get("status", "") or (data.get("orchestration", {}) or {}).get("status", ""),
                "updated_utc": data.get("updated_utc", "") or data.get("started_utc", "") or data.get("complete_accounting_start_utc", ""),
                "increment_seconds": data.get("increment_seconds", data.get("increment", data.get("increment_seconds"))),
                "projection_seconds": data.get(
                    "complete_projection_seconds",
                    data.get("increment_seconds", data.get("increment")),
                ),
                "cumulative_projected_seconds": data.get("cumulative_projected_seconds"),
                "lab_cumulative_projection_seconds": data.get("lab_cumulative_projection_seconds"),
                "previous_ledger": data.get("previous_ledger", ""),
                "new_pods": data.get("new_pods", ""),
                "approx_gpu_cost_usd": data.get("approximate_gpu_cost_usd"),
            }
        )
    chain = _chain(entries, root)
    return {
        "present": bool(entries),
        "reason": "" if entries else ABSENT,
        "checked": f"{root}/**/resource-ledger.json",
        "count": len(entries),
        "chain": chain,
        "latest_lab_cumulative_seconds": _latest_cumulative(entries),
        "gpu_cost_records": [
            {"path": e["path"], "usd": e["approx_gpu_cost_usd"]}
            for e in entries
            if _num(e.get("approx_gpu_cost_usd")) is not None
        ],
    }


def _latest_cumulative(entries: list[dict[str, Any]]) -> dict[str, Any]:
    """The highest cumulative projection any ledger states, with its path.

    Two schemas coexist: ``cumulative_projected_seconds`` (the pointer chain) and
    ``lab_cumulative_projection_seconds`` (the later complete-accounting ledgers).
    Neither is summed, because each is already a cumulative figure.
    """
    best: dict[str, Any] = {"seconds": None, "path": "", "field": ""}
    for entry in entries:
        for field in ("lab_cumulative_projection_seconds", "cumulative_projected_seconds"):
            value = _num(entry.get(field))
            if value is None:
                continue
            if best["seconds"] is None or value > best["seconds"]:
                best = {"seconds": value, "path": entry["path"], "field": field}
    if best["seconds"] is not None:
        best["hours"] = round(best["seconds"] / 3600.0, 2)
    return best


def _chain(entries: list[dict[str, Any]], root: Path) -> dict[str, Any]:
    """Walk ``previous_ledger`` back from the newest head to recover the cumulative total.

    Pointers are written in several shapes in the repo (a repo-relative path, an absolute
    pod path, or absent), so a link matches on the full path or on its tail. The head's own
    ``cumulative_projected_seconds`` is reported as the projection; the increments walked
    are reported separately as an internal-consistency check, not as a new total.
    """
    if not entries:
        return {
            "head": "",
            "links": 0,
            "files": 0,
            "increment_seconds": 0.0,
            "increment_hours": 0.0,
            "cumulative_projected_seconds": None,
            "cumulative_projected_hours": None,
            "note": "no resource-ledger.json found",
        }
    by_path = {e["path"]: e for e in entries}

    def follow(pointer: str) -> dict[str, Any] | None:
        pointer = pointer.strip()
        if not pointer:
            return None
        if pointer in by_path:
            return by_path[pointer]
        for path, entry in by_path.items():
            if pointer.endswith(path) or path.endswith(pointer) or pointer.endswith(path.split("/")[-1]):
                return entry
        return None

    heads = [
        e
        for e in entries
        if not any(str(o.get("previous_ledger") or "").strip() and follow(str(o["previous_ledger"])) is e for o in entries)
    ]
    pool = heads or entries
    head = sorted(pool, key=lambda e: str(e.get("updated_utc", "")), reverse=True)[0]

    node: dict[str, Any] | None = head
    links = 0
    increment = 0.0
    seen: set[str] = set()
    while node is not None and node["path"] not in seen:
        seen.add(node["path"])
        links += 1
        inc = _num(node.get("increment_seconds"))
        if inc is not None:
            increment += inc
        pointer = str(node.get("previous_ledger") or "").strip()
        if not pointer:
            break
        nxt = follow(pointer)
        if nxt is None or nxt["path"] in seen:
            break
        node = nxt
    cumulative = head.get("cumulative_projected_seconds")
    return {
        "head": head["path"],
        "links": links,
        "files": len(entries),
        "increment_seconds": round(increment, 1),
        "increment_hours": round(increment / 3600.0, 2),
        "cumulative_projected_seconds": cumulative,
        "cumulative_projected_hours": round(float(cumulative) / 3600.0, 2) if _num(cumulative) is not None else None,
        "note": (
            "cumulative_projected_seconds is the newest ledger's own projection; the walked increments "
            "are shown as an internal-consistency check, and a missing previous_ledger ends the walk"
        ),
    }


def model_time_ledger(root: Path) -> dict[str, Any]:
    """The ledger the task names explicitly. ABSENT rather than guessed when missing."""
    path = root / MODEL_TIME_LEDGER
    if not path.exists():
        return {"present": False, "reason": ABSENT, "checked": MODEL_TIME_LEDGER}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        return {"present": False, "reason": f"UNREADABLE ({type(exc).__name__})", "checked": MODEL_TIME_LEDGER}
    return {"present": True, "checked": MODEL_TIME_LEDGER, "data": data}


def pod_ledger_spend(state: Path | None = None) -> dict[str, Any]:
    """Actual pod spend from the local event ledger: rate x uptime per pod, by month.

    This is local machine state, outside the repo; it is read-only and clearly labelled.
    """
    state = state or pod_state_dir()
    path = state / "ledger.jsonl"
    checked = str(path)
    if not path.is_file():
        return {"present": False, "reason": ABSENT, "checked": checked, "usd_total": None}
    records = _read_jsonl_tolerant(path)
    rate_by_pod: dict[str, float] = {}
    months: dict[str, float] = {}
    labs: dict[str, float] = {}
    by_pod: dict[str, float] = {}
    events = 0
    for rec in records:
        if not isinstance(rec, dict):
            continue
        pod = str(rec.get("pod") or "")
        ts = str(rec.get("ts") or "")
        rate = _num(rec.get("usd_per_h"))
        if not pod or not ts:
            continue
        when = _parse_ts(ts)
        if when is None:
            continue
        if rate is not None:
            rate_by_pod[pod] = rate
        if rec.get("event") == "down":
            events += 1
            start = _parse_ts(str(rec.get("start_ts") or "")) or None
            rate_used = rate if rate is not None else rate_by_pod.get(pod)
            if rate_used is None:
                continue
            if start is not None:
                seconds = max(0.0, (when - start).total_seconds())
            else:
                uptime = _num(rec.get("uptime_s"))
                seconds = max(0.0, uptime) if uptime is not None else 0.0
            usd = rate_used * seconds / 3600.0
            month = ts[:7]
            months[month] = round(months.get(month, 0.0) + usd, 4)
            lab = str(rec.get("lab") or "(unset)")
            labs[lab] = round(labs.get(lab, 0.0) + usd, 4)
            by_pod[pod] = round(by_pod.get(pod, 0.0) + usd, 4)
    return {
        "present": True,
        "checked": checked,
        "records": len(records),
        "closed_events": events,
        "usd_total": round(sum(months.values()), 4),
        "by_month": dict(sorted(months.items())),
        "by_lab": dict(sorted(labs.items(), key=lambda kv: -kv[1])),
        "pods": dict(sorted(by_pod.items(), key=lambda kv: -kv[1])[:10]),
        "method": "sum over down events of usd_per_h x duration; pods still running are not charged here",
        "scope": "local machine state outside the repo (UOR_POD_STATE), not committed evidence",
    }


def _parse_ts(ts: str) -> datetime | None:
    if not ts:
        return None
    try:
        when = datetime.fromisoformat(ts.replace("Z", "+00:00"))
    except ValueError:
        return None
    return when if when.tzinfo else when.replace(tzinfo=timezone.utc)


# --------------------------------------------------------------------------- pod scan


def main_worktree(root: Path) -> Path | None:
    """The main working tree for ``root``, when ``root`` is a worktree of it.

    ``scripts/pod/uor-pod`` is gitignored, so a fresh worktree does not carry it: the
    owner's checkout is where it actually lives. Read-only git queries only.
    """
    rc, out = run(["git", "worktree", "list", "--porcelain"], cwd=root, timeout=10)
    if rc == 0:
        for line in out.splitlines():
            if line.startswith("worktree "):
                candidate = Path(line[len("worktree ") :].strip())
                if candidate != root:
                    return candidate
    rc, out = run(["git", "rev-parse", "--path-format=absolute", "--git-common-dir"], cwd=root, timeout=10)
    if rc == 0 and out.strip():
        common = Path(out.strip().splitlines()[0])
        if common.name == ".git":
            return common.parent
    return None


def pod_candidates(root: Path) -> list[tuple[str, Path]]:
    """Every place ``uor-pod`` may live, in resolution order.

    (a) this repo/worktree, (b) the main checkout when this is a worktree, (c) $UOR_POD.
    """
    out: list[tuple[str, Path]] = [("repo", root / POD_SCRIPT_REL)]
    main = main_worktree(root)
    if main is not None and main != root:
        out.append(("main checkout", main / POD_SCRIPT_REL))
    env = os.environ.get("UOR_POD")
    if env:
        out.append(("$UOR_POD", Path(env).expanduser()))
    return out


def resolve_pod_script(root: Path) -> dict[str, Any]:
    """Resolve the pod tool without running anything: which path, why, and what was checked."""
    checked: list[str] = []
    for source, path in pod_candidates(root):
        checked.append(f"{source}: {path}")
        if path.is_file():
            return {"path": path, "source": source, "checked": checked}
    return {"path": None, "source": "", "checked": checked}


def pod_script(root: Path) -> Path | None:
    """The pod script, or None. Never runs it and never mutates pod state."""
    return resolve_pod_script(root)["path"]


def parse_pod_status(text: str) -> dict[str, Any]:
    """Parse the headline numbers out of ``uor-pod status`` without trusting the layout."""
    out: dict[str, Any] = {
        "running": None,
        "usd_per_h": None,
        "free_summary": "",
        "raw_lines": len(text.splitlines()),
    }
    m = re.search(r"Running:\s*(\d+)\s*pod", text)
    if m:
        out["running"] = int(m.group(1))
    m = re.search(r"\$([0-9]+(?:\.[0-9]+)?)\s*/\s*h\s+of\s+\$([0-9]+(?:\.[0-9]+)?)\s*/\s*h", text)
    if m:
        out["usd_per_h"] = float(m.group(1))
    m = re.search(r"Free GPUs you can lease now:\s*(.+)", text)
    if m:
        out["free_summary"] = m.group(1).strip()
    return out


def parse_gpu_costs(text: str) -> dict[str, float]:
    """The secure $/GPU/h table from ``uor-pod gpus`` (or the stock block of status)."""
    costs: dict[str, float] = {}
    for m in re.finditer(r"^\s{2}([a-z0-9]+)\s+\$([0-9]+(?:\.[0-9]+)?)\s*:", text, re.MULTILINE):
        try:
            costs[m.group(1)] = float(m.group(2))
        except ValueError:  # pragma: no cover - regex guarantees a number
            continue
    return costs


def scan(args: Any, paths: HarnessPaths, reg: Registry) -> int:
    resolved = resolve_pod_script(paths.repo)
    script: Path | None = resolved["path"]
    attempts: list[dict[str, Any]] = []
    if script is None:
        payload = {
            "available": False,
            "reason": "UNAVAILABLE",
            "checked": resolved["checked"] or [POD_SCRIPT_REL],
            "detail": (
                "scripts/pod/uor-pod was not found in this repo, the main checkout or $UOR_POD "
                "(it is gitignored, so a fresh worktree does not carry it)"
            ),
            "attempts": [],
        }
        return _emit_scan(args, payload)
    if not os.access(script, os.X_OK):
        payload = {
            "available": False,
            "reason": "UNAVAILABLE",
            "checked": str(script),
            "script_source": resolved["source"],
            "detail": "uor-pod exists but is not executable; refusing to guess pod state",
            "attempts": [],
        }
        return _emit_scan(args, payload)

    # read-only subcommands only: status and gpus print state, they never mutate it
    status_rc, status_out = run(["bash", str(script), "status"], cwd=paths.repo, timeout=POD_TIMEOUT_S)
    attempts.append({"command": f"bash {script} status", "returncode": status_rc, "bytes": len(status_out)})
    gpus_rc, gpus_out = run(["bash", str(script), "gpus"], cwd=paths.repo, timeout=POD_TIMEOUT_S)
    attempts.append({"command": f"bash {script} gpus", "returncode": gpus_rc, "bytes": len(gpus_out)})

    if status_rc != 0 and gpus_rc != 0:
        payload = {
            "available": False,
            "reason": "UNAVAILABLE",
            "checked": str(script),
            "script_source": resolved["source"],
            "detail": "uor-pod status and gpus both failed; no pod numbers are reported",
            "attempts": attempts,
            "status_tail": truncate(status_out, 200),
        }
        return _emit_scan(args, payload)

    status = parse_pod_status(status_out)
    gpu_costs = parse_gpu_costs(gpus_out) or parse_gpu_costs(status_out)
    running = status["running"]
    rate = status["usd_per_h"]
    payload = {
        "available": True,
        "checked": str(script),
        "script_source": resolved["source"],
        "attempts": attempts,
        "running_pods": running,
        "usd_per_h": rate,
        "caps": {"pods": POD_CAP, "usd_per_h": RATE_CAP_USD_PER_H},
        "over_cap": {
            "pods": (running is not None and running > POD_CAP),
            "rate": (rate is not None and rate > RATE_CAP_USD_PER_H),
        },
        "free_gpus": status["free_summary"] or "not stated in the status output",
        "gpu_costs_usd_per_h": gpu_costs,
        "status_lines": status["raw_lines"],
    }
    return _emit_scan(args, payload)


def _emit_scan(args: Any, payload: dict[str, Any]) -> int:
    if getattr(args, "json", False):
        emit(json.dumps(payload, indent=2, ensure_ascii=False))
        return 0
    if not payload.get("available"):
        checked = payload.get("checked", POD_SCRIPT_REL)
        emit(f"pod scan: UNAVAILABLE ({payload.get('detail', 'no detail')})")
        if isinstance(checked, list):
            for entry in checked:
                emit(f"  checked {entry}")
        else:
            emit(f"  checked: {checked}")
        for attempt in payload.get("attempts", []):
            emit(f"  ran: {attempt['command']} -> exit {attempt['returncode']}")
        emit("  nothing is fabricated here and no pod state was touched; run the command yourself if you need live state.")
        return 0

    caps = payload["caps"]
    running = payload["running_pods"]
    rate = payload["usd_per_h"]
    emit("live compute (read-only via scripts/pod/uor-pod)")
    source = payload.get("script_source", "")
    if source and source != "repo":
        emit(f"  tool: {payload['checked']} (from the {source}; scripts/pod/ is gitignored)")
    emit(f"  running pods: {running if running is not None else MISSING} of {caps['pods']} allowed")
    emit(f"  cost: ${rate:.2f}/h" if rate is not None else "  cost: MISSING from the status output")
    if caps["usd_per_h"] is not None and rate is not None:
        emit(f"  cap: ${caps['usd_per_h']:.2f}/h across all labs, all sessions")
    if payload["over_cap"]["pods"]:
        emit(f"  OVER CAP: {running} running pods > {caps['pods']}")
    if payload["over_cap"]["rate"]:
        emit(f"  OVER CAP: ${rate:.2f}/h > ${caps['usd_per_h']:.2f}/h")
    emit(f"  free GPUs you can lease now: {payload['free_gpus']}")
    if payload["gpu_costs_usd_per_h"]:
        table = ", ".join(f"{k} ${v:.2f}" for k, v in sorted(payload["gpu_costs_usd_per_h"].items()))
        emit(f"  secure $/GPU/h: {table}")
    for attempt in payload["attempts"]:
        emit(f"  ran: {attempt['command']} -> exit {attempt['returncode']}")
    return 0


# ------------------------------------------------------------------------------- show


def collect(paths: HarnessPaths) -> dict[str, Any]:
    """Everything ``budget show`` prints, as data, so the rules and the view agree."""
    recs: list[dict[str, Any]] = []
    ledger_files: list[str] = []
    if paths.ledger_dir.is_dir():
        for f in sorted(paths.ledger_dir.glob("*.jsonl")):
            ledger_files.append(_rel(f, paths.repo))
            for rec in _read_jsonl_tolerant(f):
                if isinstance(rec, dict):
                    rec.setdefault("_file", f.name)
                    recs.append(rec)
    costs, rates = _receipt_costs(recs)
    return {
        "receipts": {
            "files": ledger_files,
            "records": len(recs),
            "cost_records": len(costs),
            "rate_only_records": len(rates),
            "usd_total": round(sum(c["usd"] for c in costs), 4),
            "by_month": _sum_by(costs, "month"),
            "by_milestone": _sum_by(costs, "milestone"),
            "entries": costs,
            "rate_entries": rates,
            "note": "receipts record orchestration; a cost field here is an estimate recorded by a session, not a charge",
        },
        "model_time_ledger": model_time_ledger(paths.repo),
        "resource_ledger": resource_ledger(paths.repo / "docs" / "labs"),
        "pod_ledger": pod_ledger_spend(),
        "disk": disk_free(paths.repo),
        "caps": {"pods": POD_CAP, "usd_per_h": RATE_CAP_USD_PER_H},
    }


def show(args: Any, paths: HarnessPaths, reg: Registry) -> int:
    data = collect(paths)
    if getattr(args, "json", False):
        emit(json.dumps(data, indent=2, ensure_ascii=False, default=str))
        return 0

    rec = data["receipts"]
    emit("cumulative cost")
    if not rec["files"]:
        emit(f"  receipts ledger: {ABSENT} (docs/harness/ledger/*.jsonl)")
    else:
        emit(f"  receipts: {rec['records']} record(s) in {len(rec['files'])} file(s); "
             f"{rec['cost_records']} carry a cost field")
        if rec["cost_records"]:
            emit(f"  receipts cost fields total: ${rec['usd_total']:.2f}")
            emit(f"    by month: {_fmt_map(rec['by_month'])}")
            emit(f"    by milestone: {_fmt_map(rec['by_milestone'])}")
        else:
            emit(f"  receipts cost fields: none recorded (by month: {ABSENT}; by milestone: {ABSENT})")
        if rec["rate_only_records"]:
            emit(f"  note: {rec['rate_only_records']} receipt(s) carry a $/h rate, not a charge - not summed")

    mt = data["model_time_ledger"]
    if mt["present"]:
        emit(f"  tracked model-time ledger: {mt['checked']} -> present")
        if isinstance(mt.get("data"), dict):
            for key in ("cumulative_ms", "limit_ms", "updated_utc"):
                if key in mt["data"]:
                    emit(f"    {key}: {mt['data'][key]}")
    else:
        emit(f"  tracked model-time ledger: {mt['reason']} at {mt['checked']} "
             "(the path the harness was told to check; no number is invented)")

    rl = data["resource_ledger"]
    if rl["present"]:
        chain = rl["chain"]
        emit(f"  repo cumulative ledger: {rl['count']} resource-ledger.json file(s) under {rl['checked']}")
        emit(f"    newest head of the pointer chain: {chain['head']} ({chain['links']} link(s) walked)")
        if chain["cumulative_projected_hours"] is not None:
            emit(f"    head projection: {chain['cumulative_projected_hours']} h "
                 f"({chain['cumulative_projected_seconds']:.0f} s, the newest ledger's own figure)")
        if chain["links"] == 1:
            emit("      the walk stopped at the head: it names no previous_ledger (a new accounting boundary), "
                 "so the chain does not reach back to the first ledger")
        latest = rl.get("latest_lab_cumulative_seconds") or {}
        if latest.get("seconds") is not None:
            emit(f"    latest stated lab cumulative: {latest['hours']} h ({latest['seconds']:.0f} s) "
                 f"in {latest['path']} [{latest['field']}]")
        if rl.get("gpu_cost_records"):
            for row in rl["gpu_cost_records"]:
                emit(f"    stated GPU cost: ${float(row['usd']):.2f} in {row['path']}")
    else:
        emit(f"  repo cumulative ledger: {ABSENT} under {rl['checked']}")

    pl = data["pod_ledger"]
    if pl["present"]:
        emit(f"  pod spend (actual, local): ${pl['usd_total']:.2f} over {pl['closed_events']} closed pod(s)")
        emit(f"    by month: {_fmt_map(pl['by_month'])}")
        emit(f"    by lab: {_fmt_map(pl['by_lab'])}")
        emit(f"    source: {pl['checked']} - {pl['scope']}")
    else:
        emit(f"  pod spend: {pl['reason']} at {pl['checked']}")

    d = data["disk"]
    if not d.get("ok"):
        emit(f"  disk: UNAVAILABLE ({d.get('error')})")
    else:
        low, high = d["band_gb"]
        state = "in band" if d["in_band"] else ("BELOW FLOOR" if d["below_floor"] else "above band")
        emit(f"  disk free: {d['free_gb']:.2f} GB {state} (owner band {low}-{high} GB; "
             f"retained-evidence margin {d['margin_mib']} MiB)")
        if d["below_floor"]:
            emit(f"    clear it: {HYGIENE_COMMAND}")
    emit(f"  owner caps: <= {POD_CAP} running pods, <= ${RATE_CAP_USD_PER_H:.2f}/h (all labs, all sessions)")
    return 0


def _fmt_map(mapping: dict[str, Any]) -> str:
    if not mapping:
        return ABSENT
    return ", ".join(f"{k} ${float(v):.2f}" for k, v in mapping.items())


# ------------------------------------------------------------------------------- rules


#: checkrules.rules() calls every module's register_rules() on every call, so this one
#: must be idempotent: without the cache, two runs in one process produce two rule
#: entries per id and a duplicate RuleResult for every violation.
_RULES_CACHE: list[Rule] | None = None


def register_rules() -> list[Rule]:
    global _RULES_CACHE
    if _RULES_CACHE is None:
        _RULES_CACHE = _build_rules()
    return _RULES_CACHE


def _build_rules() -> list[Rule]:
    return [
        register_rule(
            Rule(
                id="budget.receipts_parse",
                description="every ledger line parses as JSON and carries ts and kind",
                severity="error",
                check=_check_receipts_parse,
                rationale="a corrupt ledger silently erases the cumulative record of what ran and what it cost",
            )
        ),
        register_rule(
            Rule(
                id="budget.disk_floor",
                description=f"free disk stays inside the owner's {DISK_BAND_GB[0]}-{DISK_BAND_GB[1]} GB working band",
                severity="warn",
                check=_check_disk_floor,
                rationale="AGENTS.md resources: keep 30-70 GB free and retain the 128 MiB storage stop margin",
            )
        ),
        register_rule(
            Rule(
                id="budget.no_new_spending_class",
                description="the owner caps are printed so a session cannot silently exceed them",
                severity="info",
                check=_check_caps,
                rationale="AGENTS.md GPU pods: <= 4 running pods and <= $8/h all labs together; a new spending class needs the owner",
            )
        ),
    ]


def _check_receipts_parse(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    out: list[RuleResult] = []
    if not paths.ledger_dir.is_dir():
        return out
    for f in sorted(paths.ledger_dir.glob("*.jsonl")):
        rel = _rel(f, paths.repo)
        try:
            lines = f.read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError as exc:
            out.append(
                RuleResult(
                    rule_id="budget.receipts_parse",
                    severity="error",
                    ok=False,
                    message=f"cannot read ledger: {type(exc).__name__}: {exc}",
                    path=rel,
                    fix="check the file and its permissions",
                )
            )
            continue
        for i, line in enumerate(lines, start=1):
            if not line.strip():
                continue
            try:
                rec = json.loads(line)
            except json.JSONDecodeError as exc:
                out.append(
                    RuleResult(
                        rule_id="budget.receipts_parse",
                        severity="error",
                        ok=False,
                        message=f"line {i} is not JSON: {exc.msg}",
                        path=rel,
                        fix=f"repair or drop line {i} of {rel}",
                    )
                )
                continue
            if not isinstance(rec, dict):
                out.append(
                    RuleResult(
                        rule_id="budget.receipts_parse",
                        severity="error",
                        ok=False,
                        message=f"line {i} is JSON but not an object",
                        path=rel,
                        fix=f"a ledger line must be one JSON object; repair line {i}",
                    )
                )
                continue
            missing = [k for k in ("ts", "kind") if not rec.get(k)]
            if missing:
                out.append(
                    RuleResult(
                        rule_id="budget.receipts_parse",
                        severity="error",
                        ok=False,
                        message=f"line {i} is missing {', '.join(missing)}",
                        path=rel,
                        fix=f"add {', '.join(missing)} to line {i} of {rel}",
                    )
                )
    return out


def _check_disk_floor(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    d = disk_free(paths.repo)
    if not d.get("ok"):
        return [
            RuleResult(
                rule_id="budget.disk_floor",
                severity="warn",
                ok=False,
                message=f"cannot measure free disk ({d.get('error')})",
                path=str(paths.repo),
                fix="check the mount",
            )
        ]
    low, _high = DISK_BAND_GB
    if d["free_gb"] >= low:
        return []
    return [
        RuleResult(
            rule_id="budget.disk_floor",
            severity="warn",
            ok=False,
            message=(
                f"free disk is {d['free_gb']:.2f} GB, below the documented {low}-{DISK_BAND_GB[1]} GB working band "
                f"(measured on {d['path']})"
            ),
            path=str(paths.repo),
            fix=HYGIENE_COMMAND,
            meta={"free_gb": d["free_gb"], "band_gb": list(DISK_BAND_GB)},
        )
    ]


def _check_caps(paths: HarnessPaths, reg: Registry) -> list[RuleResult]:
    return [
        RuleResult(
            rule_id="budget.no_new_spending_class",
            severity="info",
            ok=False,
            message=(
                f"owner caps in force: <= {POD_CAP} running pods and <= ${RATE_CAP_USD_PER_H:.2f}/h "
                "across all labs and sessions, all external compute through scripts/pod/uor-pod; "
                "a new spending class needs an owner decision"
            ),
            path="AGENTS.md",
            fix="if a run would exceed a cap or need a new spending class, stop that line and ask the owner on the milestone",
            meta={"pods": POD_CAP, "usd_per_h": RATE_CAP_USD_PER_H},
        )
    ]


def main(args: Any, paths: HarnessPaths, reg: Registry) -> int:  # pragma: no cover - dispatcher parity
    if str(getattr(args, "action", "") or "") == "scan":
        return scan(args, paths, reg)
    return show(args, paths, reg)
