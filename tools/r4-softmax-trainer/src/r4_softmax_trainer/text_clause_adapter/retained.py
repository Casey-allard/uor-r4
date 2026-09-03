"""Metadata-only assembly and admission for the retained #1094 preparation.

Assembly never imports the adapter, a project dependency or a model package. It
reads pinned public receipts, committed executable source and the sealed
directory's own metadata. Runtime/asset reads belong exclusively to the timed
``verify_runtime`` call. Assembly is not a repeat of ``campaign.prepare``.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess


CONTRACT_REVISION = "6008e3527cb119d12af4abd99fe86a3d4ebe5a53"
CONTRACT_PATH = "docs/r4_text_clause_preparation_1094_contract.json"
CONTRACT_SHA256 = "fa0fbde6fda045bfa770837fd3eda612329bf34d2abaee34a5b143c751c65780"
SCHEMA = "uor-r4.text-clause-retained-preparation/1"
STATUS = "PREPARATION_ASSEMBLED_FROM_RETAINED_EVIDENCE"
RELEASE_SCHEMA = "uor-r4.text-clause-retained-release/1"
RELEASE_STATUS = "ACCEPTED_FOR_RETAINED_EVIDENCE_COMPARISON"
PACKAGE = "tools/r4-softmax-trainer/src/r4_softmax_trainer"
SOURCE_EXTRAS = (PACKAGE + "/text_clause_adapter/policy.json",
                 "tools/r4-softmax-trainer/pyproject.toml",
                 "tools/r4-softmax-trainer/uv.lock")
PROBE_BYTES = b"Harmless #1094 worker isolation denial sentinel.\n"
LIMITS = {"phase_seconds": 120, "cumulative_seconds": 360,
          "peak_rss_bytes": 3221225472, "new_bytes": 134217728,
          "batch_size": 128, "logical_row_forwards": 6400}
DEBIT = {"seconds": 120, "logical_row_forwards": 0, "retained_bytes": 3465401,
         "new_preparation_attempts": 0,
         "byte_components": {"frozen_corpus_selection_policy": 3397265,
                             "population_manifest": 1565,
                             "original_harmless_probe": 48,
                             "original_comparison_receipts": 66523},
         "kind": "full original preparation allocation quarantined; not measured elapsed"}


def canonical(value: object) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False) + "\n").encode()


def digest(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def _json(payload: bytes) -> dict:
    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ValueError("duplicate JSON object key")
            result[key] = value
        return result
    value = json.loads(payload, object_pairs_hook=pairs,
                       parse_constant=lambda _: (_ for _ in ()).throw(ValueError("nonfinite JSON")))
    if type(value) is not dict:
        raise ValueError("expected JSON object")
    return value


def _absolute(path: Path) -> Path:
    path = Path(path)
    if not path.is_absolute() or ".." in path.parts or str(path) != os.path.normpath(str(path)):
        raise ValueError("path must be absolute and normalized")
    return path


def _executing_repo(repo: Path) -> None:
    if Path(__file__).resolve() != repo / PACKAGE / "text_clause_adapter/retained.py":
        raise ValueError("executing retained module is outside the bound coordinator source")


def _no_symlinks(path: Path) -> None:
    """Inspect components only; never enumerate a sealed directory."""
    path = _absolute(path)
    for item in reversed((path,) + tuple(path.parents)):
        try:
            mode = item.lstat().st_mode
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(mode):
            raise ValueError(f"symlink in metadata/source/output path: {item}")


def _read_metadata(path: Path) -> bytes:
    """Read only a unique regular output file, never a hardlinked payload alias."""
    _no_symlinks(path)
    initial = path.lstat()
    if not stat.S_ISREG(initial.st_mode) or initial.st_nlink != 1:
        raise ValueError("output metadata must be a unique regular file")
    descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        actual = os.fstat(descriptor)
        if (not stat.S_ISREG(actual.st_mode) or actual.st_nlink != 1
                or (actual.st_dev, actual.st_ino) != (initial.st_dev, initial.st_ino)):
            raise ValueError("output metadata identity changed before reading")
        with os.fdopen(descriptor, "rb", closefd=False) as stream:
            return stream.read()
    finally:
        os.close(descriptor)


def record(path: Path) -> dict:
    """Hash a named unsealed metadata/source file; callers choose the scope."""
    path = _absolute(path)
    _no_symlinks(path)
    if not stat.S_ISREG(path.stat().st_mode):
        raise ValueError(f"not a regular file: {path}")
    hasher, size = hashlib.sha256(), 0
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            hasher.update(block)
            size += len(block)
    return {"path": str(path), "bytes": size, "sha256": hasher.hexdigest()}


def _verify_record(item: dict) -> dict:
    actual = record(Path(item["path"]))
    if any(actual[key] != item[key] for key in ("bytes", "sha256")):
        raise ValueError(f"file identity changed: {item['path']}")
    return actual


def _write(path: Path, payload: bytes) -> dict:
    with path.open("xb") as stream:
        stream.write(payload)
        stream.flush()
        os.fsync(stream.fileno())
    return record(path)


def _git(repo: Path, *args: str, input: bytes | None = None) -> bytes:
    return subprocess.check_output(["/usr/bin/git", "-C", str(repo), *args],
                                   input=input,
                                   env={"PATH": "/usr/bin:/bin:/usr/sbin:/sbin",
                                        "HOME": str(Path.home()), "LC_ALL": "C"})


def _source_paths(repo: Path, revision: str) -> list[str]:
    paths = _git(repo, "ls-tree", "-r", "--name-only", "-z", revision,
                 "--", PACKAGE, *SOURCE_EXTRAS).decode().split("\0")
    return sorted(path for path in paths if path and
                  ((path.startswith(PACKAGE + "/") and path.endswith(".py"))
                   or path in SOURCE_EXTRAS))


def _committed_files(repo: Path, revision: str, paths: list[str]) -> list[bytes]:
    requests = "".join(revision + ":" + path + "\n" for path in paths).encode()
    payload = _git(repo, "cat-file", "--batch", input=requests)
    result, offset = [], 0
    for _ in paths:
        end = payload.index(b"\n", offset)
        header = payload[offset:end].split()
        if len(header) != 3 or header[1] != b"blob":
            raise ValueError("source Git object is not a blob")
        size, start = int(header[2]), end + 1
        stop = start + size
        if size < 0 or payload[stop:stop + 1] != b"\n":
            raise ValueError("invalid source Git batch framing")
        result.append(payload[start:stop])
        offset = stop + 1
    if offset != len(payload):
        raise ValueError("unexpected source Git batch tail")
    return result


def source_identity(repo: Path, revision: str) -> dict:
    """Compare the complete executable source closure with committed bytes."""
    repo = _absolute(repo)
    _no_symlinks(repo)
    expected = _source_paths(repo, revision)
    package = repo / PACKAGE
    actual = []
    for root, directories, files in os.walk(package, followlinks=False):
        for name in directories + files:
            item = Path(root) / name
            if item.is_symlink():
                raise ValueError(f"source closure contains symlink: {item}")
        actual.extend(str((Path(root) / name).relative_to(repo))
                      for name in files if name.endswith(".py"))
    if sorted(actual + list(SOURCE_EXTRAS)) != expected:
        raise ValueError("executable source closure has missing or uncommitted files")
    files = []
    for path, committed in zip(expected, _committed_files(repo, revision, expected)):
        actual_record = record(repo / path)
        if (actual_record["bytes"], actual_record["sha256"]) != (len(committed), digest(committed)):
            raise ValueError(f"source differs from committed bytes: {path}")
        files.append(actual_record)
    return {"repo": str(repo), "commit": revision, "files": files}


def _historical(repo: Path) -> tuple[dict, dict, dict]:
    raw = _git(repo, "show", CONTRACT_REVISION + ":" + CONTRACT_PATH)
    if digest(raw) != CONTRACT_SHA256:
        raise ValueError("retained preparation contract changed")
    frozen = _json(raw)
    evidence, content = {}, {}
    for key, item in frozen["sources"].items():
        payload = _git(repo, "show", item["revision"] + ":" + item["path"])
        if (len(payload), digest(payload)) != (item["bytes"], item["sha256"]):
            raise ValueError(f"historical evidence changed: {key}")
        evidence[key] = item
        content[key] = _json(payload) if item["path"].endswith(".json") else payload
    return frozen, evidence, content


def _metadata_record(item: dict) -> dict:
    payload = canonical(item["content"])
    if (len(payload), digest(payload)) != (item["bytes"], item["sha256"]):
        raise ValueError("curator metadata serialization/commitment mismatch")
    return item["content"]


def _sealed(corpus: Path, *, require_sealed: bool) -> dict:
    path = corpus / "withheld"
    _no_symlinks(path)
    value = path.lstat()
    mode = stat.S_IMODE(value.st_mode)
    if not stat.S_ISDIR(value.st_mode) or mode not in ((0,) if require_sealed else (0, 0o500)):
        raise ValueError("withheld directory must be sealed, or explicitly owner read/execute only at release")
    return {"path": str(path), "device": value.st_dev, "inode": value.st_ino,
            "uid": value.st_uid, "gid": value.st_gid, "assembled_mode": 0,
            "metadata_only": True, "payload_access": "NOT_RUN"}


def _disjoint_output(output: Path, roots: list[Path]) -> None:
    # Reject lexical containment before lstat/resolve could touch any component
    # beneath a sealed directory.
    for root in roots:
        if output == root or output in root.parents or root in output.parents:
            raise ValueError(f"assembly output overlaps protected source/corpus/runtime tree: {root}")
    _no_symlinks(output)
    for root in roots:
        left, right = output.resolve(), root.resolve()
        if left == right or left in right.parents or right in left.parents:
            raise ValueError(f"assembly output overlaps protected source/corpus/runtime tree: {root}")
    if Path.home() not in output.parents:
        raise ValueError("assembly/probe directory must be under the denied home tree")


def _protected_roots(repo: Path, history: dict) -> list[Path]:
    ready = history["ready_manifest"]
    roots = [repo, Path(ready["repo"]),
             Path(history["old_start"]["corpus_manifest"]["path"]).parent,
             Path(ready["interpreter"]["venv"]), Path(ready["interpreter"]["base"]),
             Path(ready["bindings"]["path"]).parent,
             Path(history["old_stop"]["partial_progress"][0]["record"]["path"]).parent]
    roots += [Path(item["path"]).parent for item in history["ready_bindings"]["assets"].values()]
    return roots


def _environment(worker_repo: Path, output: Path) -> dict:
    return {"PATH": "/usr/bin:/bin:/usr/sbin:/sbin", "HOME": str(Path.home()),
            "PYTHONPATH": str(worker_repo / "tools/r4-softmax-trainer/src"),
            "PYTHONNOUSERSITE": "1", "PYTHONDONTWRITEBYTECODE": "1", "PYTHONUNBUFFERED": "1",
            "OMP_NUM_THREADS": "4", "VECLIB_MAXIMUM_THREADS": "4",
            "UOR_ISOLATION_PROBE": str(output / "isolation-probe.txt")}


def profile_from_metadata(repo: Path, binding_path: Path, ready: dict,
                          assets: dict, *, extras: tuple[str, ...] = ()) -> str:
    """The admitted generator, with interpreter resolution supplied as metadata.

    This does not call resolve/readlink on an interpreter or load contract.py.
    An exact reconstruction of #1096's prior profile is checked before rebind.
    """
    home = str(Path.home())
    interpreter = ready["interpreter"]
    allowed = [str(repo / "tools/r4-softmax-trainer/src"),
               str(Path(interpreter["launcher"]).parent.parent),
               str(Path(interpreter["resolved"]).parent.parent)]
    exact = [str(binding_path), str(repo / "tools/r4-softmax-trainer/pyproject.toml"),
             str(repo / "tools/r4-softmax-trainer/uv.lock")]
    exact += [assets[name]["path"] for name in sorted(assets)]
    exact += [item["path"] for item in interpreter["links"]]
    exact += list(extras)
    ancestors = sorted({str(parent) for path in allowed + exact for parent in Path(path).parents
                        if parent == Path(home) or Path(home) in parent.parents})
    exclusions = "\n".join([f"  (require-not (subpath {json.dumps(path)}))" for path in allowed]
                           + [f"  (require-not (literal {json.dumps(path)}))" for path in exact])
    metadata = "\n".join(f"  (require-not (literal {json.dumps(path)}))" for path in ancestors)
    return ("(version 1)\n(allow default)\n(deny network*)\n"
            f"(deny file-write* (subpath {json.dumps(home)}))\n"
            f"(deny file-read-data (require-all (subpath {json.dumps(home)})\n{exclusions}\n))\n"
            f"(deny file-read-metadata (require-all (subpath {json.dumps(home)})\n{exclusions}\n{metadata}\n))\n"
            f"(deny file-read-xattr (require-all (subpath {json.dumps(home)})\n{exclusions}\n))\n")


def _profile_delta(old: str, new: str, ready: dict, output: Path) -> dict:
    old_lines, new_lines = old.splitlines(), new.splitlines()
    removed = sorted(set(old_lines) - set(new_lines))
    added = sorted(set(new_lines) - set(old_lines))
    # Preserve line multiplicity: the same literal can occur in all three rules.
    counts = lambda lines, values: [{"line": line, "occurrences": lines.count(line)} for line in values]
    return {"previous_profile_sha256": digest(old.encode()), "profile_sha256": digest(new.encode()),
            "removed": counts(old_lines, removed), "added": counts(new_lines, added),
            "binding_literal_rebind": {"from": ready["bindings"]["path"],
                                       "to": str(output / "bindings.json")},
            "removed_readiness_literals": [str(Path(ready["profile"]["path"]).parent / "manifest.json"),
                                           ready["profile"]["path"]],
            "worker_source_tree_rebound": False,
            "new_runtime_or_home_tree_grants": False,
            "new_corpus_reference_history_result_content_grants": False,
            "network_and_home_write_denials_unchanged": True,
            "scope": "new binding literal and ancestor metadata; remove old readiness literals; no new measurement"}


def _build(repo: Path, output: Path, revision: str, *, require_sealed: bool) -> tuple[dict, bytes, bytes]:
    frozen, evidence, history = _historical(repo)
    ready, binding = history["ready_manifest"], history["ready_bindings"]
    old_start, old_stop = history["old_start"], history["old_stop"]
    if (old_stop["status"] != "UNAVAILABLE_REFERENCE_REPLAY"
            or history["old_authoring"]["status"] != "AUTHORING_INPUT_EXACT"
            or history["ready_result"]["status"] != "ISOLATED_RUNTIME_READY"
            or len(ready["runtime_files"]) != 18 or len(ready["interpreter"]["links"]) != 2
            or len(binding["assets"]) != 5 or binding["limits"] != LIMITS):
        raise ValueError("historical admission contract mismatch")
    worker_repo = _absolute(Path(ready["repo"]))
    corpus = _absolute(Path(old_start["corpus_manifest"]["path"]).parent)
    _disjoint_output(output, _protected_roots(repo, history))
    sealed = _sealed(corpus, require_sealed=require_sealed)
    curation = {item["original_path_label"]: item for item in history["old_curation"]["records"]}
    population = _metadata_record(curation["population.json"])
    selection = _metadata_record(curation["selection.json"])
    if population["selection_sha256"] != old_start["selection"]["sha256"]:
        raise ValueError("selection/population chain differs")
    # Only these two original public metadata files are opened. The commitments
    # inside them are not traversed or evaluated during assembly.
    for item in (old_start["corpus_manifest"], old_start["selection"]):
        _verify_record(item)
    coordinator = source_identity(repo, revision)
    worker = source_identity(worker_repo, ready["source_commit"])
    coordinator_paths = {str(Path(item["path"]).relative_to(repo)) for item in coordinator["files"]}
    worker_paths = {str(Path(item["path"]).relative_to(worker_repo)) for item in worker["files"]}
    if coordinator_paths - worker_paths != {PACKAGE + "/text_clause_adapter/retained.py"}:
        raise ValueError("only the new retained protocol may extend the accepted source closure")
    if [{k: item[k] for k in ("path", "bytes", "sha256")} for item in binding["source_files"]] != worker["files"]:
        # The historic generator ordered extras after Python files.
        if sorted((i["path"], i["bytes"], i["sha256"]) for i in binding["source_files"]) != sorted(
                (i["path"], i["bytes"], i["sha256"]) for i in worker["files"]):
            raise ValueError("accepted worker source closure differs")
    for path in worker["files"]:
        relative = Path(path["path"]).relative_to(worker_repo)
        # Only the new coordinator, its retained protocol and synthetic tests
        # may change; the imported model/adapter/worker/curator stay identical.
        if str(relative).endswith("/text_clause_adapter/campaign.py"):
            continue
        actual = record(repo / relative)
        if (actual["bytes"], actual["sha256"]) != (path["bytes"], path["sha256"]):
            raise ValueError(f"accepted computational source changed: {relative}")
    old_profile = history["ready_profile"].decode()
    old_extras = (str(Path(ready["profile"]["path"]).parent / "manifest.json"), ready["profile"]["path"])
    if profile_from_metadata(worker_repo, Path(ready["bindings"]["path"]), ready,
                             binding["assets"], extras=old_extras) != old_profile:
        raise ValueError("metadata profile generator does not reproduce accepted profile")
    profile = profile_from_metadata(worker_repo, output / "bindings.json", ready, binding["assets"])
    binding_bytes = _git(repo, "show", evidence["ready_bindings"]["revision"] + ":" + evidence["ready_bindings"]["path"])
    paths = {"output": str(output), "corpus": str(corpus), "worker_repo": str(worker_repo),
             "coordinator_repo": str(repo), "bindings": str(output / "bindings.json"),
             "profile": str(output / "worker.sb"), "probe": str(output / "isolation-probe.txt"),
             "assembly": str(output / "retained-preparation.json")}
    env = _environment(worker_repo, output)
    item = lambda path, payload: {"path": path, "bytes": len(payload), "sha256": digest(payload)}
    envelope = {
        "schema": SCHEMA, "issue": 1094, "status": STATUS, "release": "NOT_ADMITTED",
        "contract": {"revision": CONTRACT_REVISION, "path": CONTRACT_PATH, "sha256": CONTRACT_SHA256},
        "historical_evidence": evidence, "historical_debit": DEBIT,
        "original_terminal": old_stop["status"], "recorded_debit": frozen["recorded_debit"],
        "separate_1096_ledger": frozen["separate_1096_ledger"],
        "reused_observations": {"authoring": "320/320 valid; 16/16 refusals; two exact schema probes",
                                "runtime": "accepted #1096 event only; not rerun",
                                "model_loads": 0, "model_forwards": 0, "optimizer_updates": 0},
        "limits": LIMITS, "comparison": {"valid_rows": 1600, "refusal_rows": 80,
                                          "boundary_controls": 16, "roles_per_valid_row": 14,
                                          "soft_outputs_logits_decisions": "byte-identical",
                                          "fresh_process_replay": "exact", "tolerance_changes": False},
        "coordinator_source": coordinator, "worker_source": worker, "worker_repo": str(worker_repo),
        "bindings": item(paths["bindings"], binding_bytes),
        "sandbox": item(paths["profile"], profile.encode()),
        "probe": item(paths["probe"], PROBE_BYTES), "output_paths": paths,
        "profile_delta": _profile_delta(old_profile, profile, ready, output),
        "clean_environment": env,
        "runtime_identity": {"interpreter": ready["interpreter"], "runtime_files": ready["runtime_files"],
                             "runtime": ready["runtime"], "hardware": ready["hardware"],
                             "assets": binding["assets"]},
        "selection": old_start["selection"], "corpus_manifest": old_start["corpus_manifest"],
        "corpus_commitments": population, "selection_metadata": selection,
        "sealed_directory": sealed,
        "execution_identity_rule": "fresh runtime/assets/source/hardware verification before and after each worker under phase clock",
        "worker_identity_scope": "unchanged worker source/asset/runtime checks and one harmless denial; not four-probe readiness",
        "not_run": ["prepare", "authoring", "readiness", "runtime_or_asset_verification",
                    "model_load", "model_forward", "withheld_payload_access", "comparison", "replay"],
    }
    return envelope, binding_bytes, profile.encode()


def assemble(repo: Path, output: Path) -> dict:
    repo, output = _absolute(repo), _absolute(output)
    _executing_repo(repo)
    revision = _git(repo, "rev-parse", "HEAD").decode().strip()
    envelope, bindings, profile = _build(repo, output, revision, require_sealed=True)
    output.mkdir(mode=0o700)  # exclusive; partial evidence survives a later failure
    for name, payload in (("bindings.json", bindings), ("worker.sb", profile),
                          ("isolation-probe.txt", PROBE_BYTES)):
        _write(output / name, payload)
    if DEBIT["retained_bytes"] + sum(len(p) for p in (bindings, profile, PROBE_BYTES, canonical(envelope))) > LIMITS["new_bytes"]:
        raise ValueError("assembly exceeds retained campaign byte budget")
    envelope["assembly_record"] = _write(output / "retained-preparation.json", canonical(envelope))
    return envelope


def validate_assembly(path: Path, *, repo: Path | None = None, output: Path | None = None,
                      require_sealed: bool = True) -> dict:
    path = _absolute(path)
    selected_repo = _absolute(repo or Path(__file__).resolve().parents[5])
    selected_output = _absolute(output or path.parent)
    value = load_for_release(path, repo=selected_repo, output=selected_output)
    assembly_record = value.pop("assembly_record")
    expected, _, _ = _build(selected_repo, selected_output, value["coordinator_source"]["commit"],
                             require_sealed=require_sealed)
    if value != expected:
        raise ValueError("retained envelope differs from its pinned evidence/source/path contract")
    for key in ("bindings", "sandbox", "probe"):
        _verify_record(value[key])
    payload = _read_metadata(path)
    if (len(payload), digest(payload)) != (assembly_record["bytes"], assembly_record["sha256"]):
        raise ValueError("assembly metadata changed during validation")
    value["assembly_record"] = assembly_record
    return value


def load_for_release(path: Path, *, repo: Path, output: Path) -> dict:
    """Bind public metadata for approval before the durable attempt marker.

    No fresh executable, runtime, model or sealed-directory identity is read.
    After exact review matching the caller must first create the exclusive
    admission marker, then call ``validate_assembly`` under its execution clock.
    """
    path, repo, output = _absolute(path), _absolute(repo), _absolute(output)
    _executing_repo(repo)
    frozen, evidence, history = _historical(repo)
    _disjoint_output(output, _protected_roots(repo, history))
    if path != output / "retained-preparation.json":
        raise ValueError("assembly path differs from exclusive output")
    payload = _read_metadata(path)
    value = _json(payload)
    ready = history["ready_manifest"]
    worker_repo = Path(ready["repo"])
    corpus = Path(history["old_start"]["corpus_manifest"]["path"]).parent
    expected_paths = {"output": str(output), "corpus": str(corpus), "worker_repo": str(worker_repo),
                      "coordinator_repo": str(repo), "bindings": str(output / "bindings.json"),
                      "profile": str(output / "worker.sb"), "probe": str(output / "isolation-probe.txt"),
                      "assembly": str(path)}
    runtime = {"interpreter": ready["interpreter"], "runtime_files": ready["runtime_files"],
               "runtime": ready["runtime"], "hardware": ready["hardware"],
               "assets": history["ready_bindings"]["assets"]}
    expected = {"schema": SCHEMA, "issue": 1094, "status": STATUS, "release": "NOT_ADMITTED",
                "historical_evidence": evidence, "historical_debit": DEBIT, "limits": LIMITS,
                "recorded_debit": frozen["recorded_debit"], "worker_repo": str(worker_repo),
                "original_terminal": "UNAVAILABLE_REFERENCE_REPLAY", "output_paths": expected_paths,
                "clean_environment": _environment(worker_repo, output), "runtime_identity": runtime,
                "corpus_manifest": history["old_start"]["corpus_manifest"],
                "selection": history["old_start"]["selection"]}
    if "assembly_record" in value or any(value.get(key) != item for key, item in expected.items()):
        raise ValueError("release envelope metadata differs from the frozen protocol")
    if value.get("coordinator_source", {}).get("repo") != str(repo):
        raise ValueError("release envelope coordinator path differs")
    for name, filename in (("bindings", "bindings.json"), ("sandbox", "worker.sb"),
                           ("probe", "isolation-probe.txt")):
        if value.get(name, {}).get("path") != str(output / filename):
            raise ValueError("release envelope artifact path differs")
    value["assembly_record"] = {"path": str(path), "bytes": len(payload), "sha256": digest(payload)}
    return value


def release_bindings(assembly: dict) -> dict:
    """Exact reviewer bindings; computing this does not create an approval."""
    return {"assembly_sha256": assembly["assembly_record"]["sha256"],
            "bindings_sha256": assembly["bindings"]["sha256"],
            "profile_sha256": assembly["sandbox"]["sha256"],
            "coordinator_source_sha256": digest(canonical(assembly["coordinator_source"])),
            "worker_source_sha256": digest(canonical(assembly["worker_source"])),
            "environment_sha256": digest(canonical(assembly["clean_environment"])),
            "runtime_identity_sha256": digest(canonical(assembly["runtime_identity"])),
            "profile_delta_sha256": digest(canonical(assembly["profile_delta"])),
            "corpus_manifest_sha256": assembly["corpus_manifest"]["sha256"],
            "selection_sha256": assembly["selection"]["sha256"],
            "historical_debit_seconds": 120, "historical_retained_bytes": 3465401}


def verify_release(assembly: dict, review_path: Path) -> dict:
    if review_path != Path(assembly["output_paths"]["output"]) / "release.json":
        raise ValueError("release receipt must be the named file inside the assembled output")
    payload = _read_metadata(review_path)
    review = _json(payload)
    if (review.get("schema") != RELEASE_SCHEMA or review.get("status") != RELEASE_STATUS
            or review.get("issue") != 1094 or not isinstance(review.get("reviewer"), str)
            or not review["reviewer"].strip()):
        raise ValueError("independent retained-evidence release is missing")
    if any(review.get(key) != value for key, value in release_bindings(assembly).items()):
        raise ValueError("independent release does not bind this exact assembled envelope")
    review["review_record"] = {"path": str(review_path), "bytes": len(payload), "sha256": digest(payload)}
    return review


def _interpreter_links(python: Path) -> list[dict]:
    pending, current, result = list(python.parts[1:]), Path("/"), []
    while pending:
        current /= pending.pop(0)
        if current.is_symlink():
            if len(result) >= 40:
                raise ValueError("interpreter symlink chain exceeds bound")
            target = os.readlink(current)
            result.append({"path": str(current), "target": target})
            destination = Path(target)
            if not destination.is_absolute():
                destination = current.parent / destination
            pending, current = list(destination.absolute().parts[1:]) + pending, Path("/")
    return result


def verify_runtime(assembly: dict) -> dict:
    """Fresh hashes under the caller's execution/replay clock; never assembly."""
    _executing_repo(Path(assembly["coordinator_source"]["repo"]))
    identity = assembly["runtime_identity"]
    interpreter = identity["interpreter"]
    launcher = Path(interpreter["launcher"])
    if (str(launcher.resolve(strict=True)) != interpreter["resolved"]
            or _interpreter_links(launcher) != interpreter["links"]):
        raise ValueError("accepted interpreter or aliases changed")
    for item in identity["runtime_files"] + list(identity["assets"].values()):
        _verify_record(item)
    hardware = {key: subprocess.check_output(["/usr/sbin/sysctl", "-n", key], text=True).strip()
                for key in identity["hardware"]}
    if hardware != identity["hardware"]:
        raise ValueError("accepted hardware changed")
    for key in ("coordinator_source", "worker_source"):
        source = assembly[key]
        if source_identity(Path(source["repo"]), source["commit"]) != source:
            raise ValueError("committed source identity changed")
    for key in ("bindings", "sandbox", "probe"):
        _verify_record(assembly[key])
    return {"runtime_files_verified": len(identity["runtime_files"]),
            "interpreter_links_verified": len(interpreter["links"]),
            "assets_verified": len(identity["assets"]), "hardware": hardware,
            "coordinator_source_sha256": digest(canonical(assembly["coordinator_source"])),
            "worker_source_sha256": digest(canonical(assembly["worker_source"])),
            "model_loads": 0, "model_forwards": 0, "optimizer_updates": 0,
            "scope": "parent file/alias/hardware identity; no worker readiness measurement"}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    value = assemble(args.repo, args.output)
    print(canonical({"status": value["status"], "assembly": value["assembly_record"],
                     "release": value["release"]}).decode(), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
