"""FoundrySpecification.editorAssetBoundary: bounded locked-package staging."""
import base64
import gzip
import hashlib
import hmac
import io
import json
import pathlib
import re
import shutil
import subprocess
import tarfile
import tempfile
import zlib

MIB = 1024 * 1024
VERSION = "0.52.2"
URL = "https://registry.npmjs.org/monaco-editor/-/monaco-editor-0.52.2.tgz"
REQUIRED = {"LICENSE", "vs/loader.js", "vs/editor/editor.main.js", "vs/editor/editor.main.css", "vs/base/worker/workerMain.js", "vs/language/json/jsonWorker.js", "vs/language/typescript/tsWorker.js"}

EDITOR = {"name": "monaco-editor", "version": VERSION, "url": URL,
          "lock": "vendor/prism/package-lock.json", "destination": "editor",
          "profile": "foundry/editor-assets/1", "required": REQUIRED,
          "prefix": "package/min/vs/", "strip": "package/min/",
          "selected": {"package/LICENSE", "package/ThirdPartyNotices.txt"}}

class EditorAssetError(Exception):
    def __init__(self, code):
        self.code = code
        super().__init__(f"Editor assets refused: {code}")

def fail(code):
    raise EditorAssetError(code)

def locked_package(lock, policy=EDITOR):
    try:
        row = lock["packages"]["node_modules/" + policy["name"]]
        integrity = row["integrity"]
        if row["version"] != policy["version"] or row["resolved"] != policy["url"] or row.get("dev") or not isinstance(integrity, str) or not re.fullmatch(r"sha512-[A-Za-z0-9+/]{86}==", integrity):
            fail("invalidEditorLock")
        if len(base64.b64decode(integrity[7:], validate=True)) != 64:
            fail("invalidEditorLock")
        return {"name": policy["name"], "version": policy["version"], "url": policy["url"], "integrity": integrity}
    except (KeyError, TypeError, ValueError):
        fail("invalidEditorLock")

def decode_package(compressed, lock, policy=EDITOR):
    source = locked_package(lock, policy)
    if len(compressed) > 32 * MIB:
        fail("editorAssetLimitExceeded")
    actual = "sha512-" + base64.b64encode(hashlib.sha512(compressed).digest()).decode("ascii")
    if not hmac.compare_digest(actual, source["integrity"]):
        fail("editorPackageIntegrityMismatch")
    expanded = bytearray()
    try:
        with gzip.GzipFile(fileobj=io.BytesIO(compressed)) as stream:
            while chunk := stream.read(64 * 1024):
                if len(expanded) + len(chunk) > 256 * MIB:
                    fail("editorAssetLimitExceeded")
                expanded.extend(chunk)
        files, seen, total, package = {}, set(), 0, None
        with tarfile.open(fileobj=io.BytesIO(expanded), mode="r:") as archive:
            for index, member in enumerate(archive):
                if index >= 16384 or member.size > 32 * MIB:
                    fail("editorAssetLimitExceeded")
                name = member.name.rstrip("/") if member.isdir() else member.name
                if len(name) > 1024 or not re.fullmatch(r"[A-Za-z0-9_.@+ /-]+", name) or any(part in ("", ".", "..") for part in name.split("/")) or not (name == "package" or name.startswith("package/")) or name in seen:
                    fail("invalidEditorArchive")
                seen.add(name)
                if member.isdir():
                    continue
                if not member.isfile() or member.issparse() or member.size < 0:
                    fail("invalidEditorArchive")
                selected = (bool(policy["prefix"]) and name.startswith(policy["prefix"])) or name in policy["selected"]
                if selected or name == "package/package.json":
                    value = archive.extractfile(member).read(member.size + 1)
                    if len(value) != member.size:
                        fail("invalidEditorArchive")
                    if name == "package/package.json":
                        if len(value) > 65536:
                            fail("editorAssetLimitExceeded")
                        package = json.loads(value)
                    else:
                        target = name.removeprefix(policy["strip"]) if name.startswith(policy["strip"]) else name.removeprefix("package/")
                        total += len(value)
                        if len(files) >= 4096 or total > 64 * MIB:
                            fail("editorAssetLimitExceeded")
                        files[target] = value
        if not isinstance(package, dict) or package.get("name") != policy["name"] or package.get("version") != policy["version"] or not policy["required"].issubset(files) or any(not files[name] for name in policy["required"]):
            fail("invalidEditorArchive")
        rows = [{"path": name, "sha256": hashlib.sha256(body).hexdigest(), "bytes": len(body)} for name, body in sorted(files.items())]
        return files, {"profile": policy["profile"], "source": source, "files": rows}
    except EditorAssetError:
        raise
    except (OSError, EOFError, ValueError, TypeError, tarfile.TarError, zlib.error):
        fail("invalidEditorArchive")

def load_editor_package(root, policy=EDITOR):
    try:
        lock = json.loads((root / policy["lock"]).read_text())
    except (OSError, ValueError):
        fail("invalidEditorLock")
    source = locked_package(lock, policy)
    with tempfile.TemporaryDirectory(prefix="foundry-editor-package-") as temporary:
        try:
            subprocess.run(["npm", "pack", "--offline", "--ignore-scripts", "--json", "--pack-destination", temporary, source["url"]], check=True, timeout=60, capture_output=True)
            archive = pathlib.Path(temporary) / f"{policy['name']}-{policy['version']}.tgz"
            with archive.open("rb") as stream:
                compressed = stream.read(32 * MIB + 1)
        except (OSError, subprocess.SubprocessError):
            fail("editorPackageUnavailable")
        return decode_package(compressed, lock, policy)

def stage_editor_assets(root, policy=EDITOR):
    files, receipt = load_editor_package(root, policy)
    destination = root / "shell" / policy["destination"]
    try:
        with tempfile.TemporaryDirectory(prefix=".editor-stage-", dir=destination.parent) as temporary:
            staging = pathlib.Path(temporary)
            for name, body in files.items():
                target = staging / name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(body)
            (staging / "package.json").write_text(json.dumps(receipt, sort_keys=True, separators=(",", ":")) + "\n")
            if destination.exists():
                shutil.rmtree(destination)
            shutil.copytree(staging, destination)
    except OSError:
        fail("editorAssetStorageFailed")
    return receipt

def check_editor_assets(root, policy=EDITOR):
    files, receipt = load_editor_package(root, policy)
    destination = root / "shell" / policy["destination"]
    try:
        actual_receipt = json.loads((destination / "package.json").read_text())
        if actual_receipt != receipt:
            fail("editorPackageIntegrityMismatch")
        listed = []
        for item in destination.rglob("*"):
            if item.is_symlink():
                fail("invalidEditorArchive")
            if item.is_file():
                listed.append(item.relative_to(destination).as_posix())
        if sorted(listed) != sorted([*files, "package.json"]):
            fail("editorPackageIntegrityMismatch")
        for name, expected in files.items():
            with (destination / name).open("rb") as stream:
                actual = stream.read(len(expected) + 1)
            if actual != expected:
                fail("editorPackageIntegrityMismatch")
    except (OSError, ValueError):
        fail("editorAssetStorageFailed")
    return receipt

if __name__ == "__main__":
    root = pathlib.Path(__file__).resolve().parent.parent
    import sys
    receipt = check_editor_assets(root) if "--check" in sys.argv else stage_editor_assets(root)
    print(f"staged {len(receipt['files'])} locked Monaco files")
