"""Independent adversarial fixtures for the locked editor package boundary."""
import base64
import gzip
import hashlib
import importlib.util
import io
import json
import pathlib
import subprocess
import sys
import tarfile
import tempfile
from unittest.mock import patch

module = importlib.util.spec_from_file_location("editor_assets", pathlib.Path(__file__).resolve().parents[1] / "scripts/editor_assets.py")
a = importlib.util.module_from_spec(module)
module.loader.exec_module(a)
MIB = 1024 * 1024

def fixture(extra=(), omit=(), metadata=None):
    stream = io.BytesIO()
    rows = [("package/package.json", json.dumps(metadata or {"name": "monaco-editor", "version": "0.52.2"}).encode(), None)]
    rows += [("package/" + ("min/" if name.startswith("vs/") else "") + name, b"independent fixture bytes", None) for name in sorted(a.REQUIRED) if name not in omit]
    rows += list(extra)
    with tarfile.open(fileobj=stream, mode="w") as archive:
        for name, body, kind in rows:
            item = tarfile.TarInfo(name)
            item.size = len(body)
            if kind:
                item.type = kind
                item.linkname = "../../escape"
                item.size = 0
            archive.addfile(item, io.BytesIO(body) if not kind else None)
    return gzip.compress(stream.getvalue())

def lock(blob):
    return {"packages": {"node_modules/monaco-editor": {"version": "0.52.2", "resolved": a.URL, "integrity": "sha512-" + base64.b64encode(hashlib.sha512(blob).digest()).decode()}}}

def refuses(fn, code):
    try:
        fn()
    except a.EditorAssetError as error:
        assert error.code == code, (error.code, code)
        assert str(error) == f"Editor assets refused: {code}"
        return
    raise AssertionError(f"expected {code}")

def schema():
    blob = fixture()
    for mutation in [{}, {"packages": []}]:
        refuses(lambda: a.decode_package(blob, mutation), "invalidEditorLock")
    for key, value in [("version", "0.52.3"), ("resolved", "https://secret.test/package.tgz"), ("integrity", "sha256-weak"), ("dev", True)]:
        data = lock(blob)
        data["packages"]["node_modules/monaco-editor"][key] = value
        refuses(lambda: a.decode_package(blob, data), "invalidEditorLock")

def valid():
    blob = fixture(extra=[("package/ThirdPartyNotices.txt", b"copyright fixture", None), ("package/min/vs/extra.js", b"\xef\xbb\xbfexact\r\n", None)])
    files, receipt = a.decode_package(blob, lock(blob))
    assert a.REQUIRED.issubset(files) and files["ThirdPartyNotices.txt"] == b"copyright fixture"
    assert files["vs/extra.js"] == b"\xef\xbb\xbfexact\r\n"
    assert receipt["profile"] == "foundry/editor-assets/1"
    assert receipt["source"]["integrity"] == lock(blob)["packages"]["node_modules/monaco-editor"]["integrity"]
    assert receipt["files"] == [{"path": name, "sha256": hashlib.sha256(body).hexdigest(), "bytes": len(body)} for name, body in sorted(files.items())]

def corrupt():
    blob = fixture()
    refuses(lambda: a.decode_package(blob + b"changed", lock(blob)), "editorPackageIntegrityMismatch")
    for bad in [b"invalid gzip", blob[:-5], fixture(omit=["vs/loader.js"]), fixture(metadata={"name": "wrong", "version": "0.52.2"})]:
        refuses(lambda: a.decode_package(bad, lock(bad)), "invalidEditorArchive")
    for name, kind in [("package/min/vs/../escape", None), ("/package/absolute", None), ("package/min/vs/link", tarfile.SYMTYPE), ("package/min/vs/hard", tarfile.LNKTYPE), ("package/min/vs/device", tarfile.CHRTYPE), ("package/min/vs/loader.js", None)]:
        bad = fixture(extra=[(name, b"x", kind)])
        refuses(lambda: a.decode_package(bad, lock(bad)), "invalidEditorArchive")

def limits():
    blob = b"x" * (32 * MIB + 1)
    refuses(lambda: a.decode_package(blob, lock(blob)), "editorAssetLimitExceeded")
    del blob
    blob = gzip.compress(b"x" * (256 * MIB + 1), compresslevel=1)
    refuses(lambda: a.decode_package(blob, lock(blob)), "editorAssetLimitExceeded")
    del blob
    item = tarfile.TarInfo("package/huge")
    item.size = 32 * MIB + 1
    blob = gzip.compress(item.tobuf() + b"\0" * 1024)
    refuses(lambda: a.decode_package(blob, lock(blob)), "editorAssetLimitExceeded")
    blob = fixture(extra=[(f"package/min/vs/extra/{i}", b"", None) for i in range(4097)])
    refuses(lambda: a.decode_package(blob, lock(blob)), "editorAssetLimitExceeded")
    blob = fixture(extra=[(f"package/other/{i}", b"", None) for i in range(16385)])
    refuses(lambda: a.decode_package(blob, lock(blob)), "editorAssetLimitExceeded")
    blob = fixture(extra=[(f"package/min/vs/large{i}", b"x" * (32 * MIB), None) for i in range(3)])
    refuses(lambda: a.decode_package(blob, lock(blob)), "editorAssetLimitExceeded")

def staging():
    blob = fixture()
    with tempfile.TemporaryDirectory() as temporary:
        root = pathlib.Path(temporary)
        (root / "vendor/prism").mkdir(parents=True)
        (root / "shell").mkdir()
        (root / "vendor/prism/package-lock.json").write_text(json.dumps(lock(blob)))
        def transport(command, **options):
            assert command[:5] == ["npm", "pack", "--offline", "--ignore-scripts", "--json"]
            assert command[-1] == a.URL and options["timeout"] == 60 and options["check"] and options["capture_output"]
            (pathlib.Path(command[command.index("--pack-destination") + 1]) / "monaco-editor-0.52.2.tgz").write_bytes(blob)
        with patch.object(a.subprocess, "run", side_effect=transport):
            receipt = a.stage_editor_assets(root)
            assert a.check_editor_assets(root) == receipt
            target = root / "shell/editor/vs/loader.js"
            target.write_bytes(b"changed")
            refuses(lambda: a.check_editor_assets(root), "editorPackageIntegrityMismatch")
            # Changing the receipt together with the bytes still disagrees with the locked archive.
            changed = json.loads((root / "shell/editor/package.json").read_text())
            for row in changed["files"]:
                if row["path"] == "vs/loader.js":
                    row.update(bytes=7, sha256=hashlib.sha256(b"changed").hexdigest())
            (root / "shell/editor/package.json").write_text(json.dumps(changed))
            refuses(lambda: a.check_editor_assets(root), "editorPackageIntegrityMismatch")
            a.stage_editor_assets(root)
            (root / "shell/editor/extra.js").write_bytes(b"extra")
            refuses(lambda: a.check_editor_assets(root), "editorPackageIntegrityMismatch")
            with patch.object(a.shutil, "copytree", side_effect=OSError("secret path")):
                refuses(lambda: a.stage_editor_assets(root), "editorAssetStorageFailed")
        with patch.object(a.subprocess, "run", side_effect=subprocess.TimeoutExpired("npm", 60)):
            refuses(lambda: a.stage_editor_assets(root), "editorPackageUnavailable")

cases = {"schema": schema, "valid": valid, "corrupt": corrupt, "limits": limits, "staging": staging}
cases[sys.argv[1]]()
print(json.dumps({"case": sys.argv[1], "passed": True}))
