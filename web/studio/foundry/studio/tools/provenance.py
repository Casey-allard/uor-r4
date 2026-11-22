"""The evidence a published application carries, written by the lane beside the core.

PrismPM's archive composer takes a closed ArchiveProvenance record (crates/prismpm/src/holo/archive.rs)
whose every field it validates by shape and embeds in the archive's Prism extension. This script fills
it from the same sources PrismPM's own application build reads (application_build.rs): the LexLean
attestation and build manifest, PrismPM's dependency register and stdlib release, its pinned
hologram-live and uor-hologram commits, the lean4-prod export, the generated core, the packaged
crate (a deterministic tar; see below), and the shell projection. Every value is derived; nothing is typed in. Fields the lane cannot
derive are not invented: the script refuses.

Usage (from scripts/lane.sh): python3 tools/provenance.py <PrismPM dir> <attestation id> <build id> <export dir>
Writes shell/provenance.json, which the shell precaches beside manifest.json and the host reads when
a visitor publishes. It is outside the shell closure because the closure digest is one of its fields.
"""
import hashlib, io, json, pathlib, re, subprocess, sys

root = pathlib.Path(__file__).resolve().parent.parent
prismpm, attestation_id, build_id, export = (pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3], pathlib.Path(sys.argv[4]))


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def canonical(value):
    # PrismPM's canonical JSON: sorted keys, no spaces, no trailing newline.
    return json.dumps(value, separators=(",", ":"), sort_keys=True, ensure_ascii=False).encode("utf-8")


attestation = json.loads((root / ".lexlean" / "verified" / attestation_id / "attestation.json").read_text(encoding="utf-8"))
build_manifest_path = root / ".lexlean" / "build" / build_id / "manifest.json"
build_manifest = json.loads(build_manifest_path.read_text(encoding="utf-8"))
register = (prismpm / "model" / "dependencies.toml").read_text(encoding="utf-8")


def dependency(name):
    block = register.split('id = "' + name + '"', 1)[1].split("[[dependency]]", 1)[0]
    revision = re.search(r'revision = "([0-9a-f]{40})"', block).group(1)
    crate = re.search(r'path = "([^"]+\.crate)"\s*\nsha256 = "([0-9a-f]{64})"', block)
    return revision, (crate.group(2) if crate else None)


lexlean_commit, lexlean_crate = dependency("lexlean")
lean4_prod_commit, _ = dependency("lean4-prod")
stdlib = json.loads((prismpm / "stdlib" / "release.json").read_text(encoding="utf-8"))
build_rs = (prismpm / "crates" / "prismpm" / "src" / "application_build.rs").read_text(encoding="utf-8")
hologram_live_commit = re.search(r'HOLOGRAM_LIVE_COMMIT: &str = "([0-9a-f]{40})"', build_rs).group(1)
uor_hologram_commit = re.search(r'UOR_HOLOGRAM_COMMIT: &str = "([0-9a-f]{40})"', build_rs).group(1)

kernel_sha, coverage_sha, roots_sha = sha(export / "kernel.ir"), sha(export / "coverage.json"), sha(export / "roots.json")
lcnf_manifest = canonical({"coverage_sha256": coverage_sha, "kernel_ir_sha256": kernel_sha, "roots_sha256": roots_sha, "schema": "prismpm/lcnf-manifest/1"})
# The Core-Wasm target profile of this core: the ABI in core/src/abi.rs (holo_run, core-wasm@1), with
# the request and response ceilings the browser boundary allows a published archive to cross.
target_profile = canonical({"contract": "hologram:guest/core-wasm@1", "export": "holo_run", "input_allocation_cap": 268435456, "maximum_pages": 4096, "response_maximum": 268435456, "schema": "lean4-prod/core-wasm-target/1"})

package = re.search(r'\[package\]\s*\nname = "([^"]+)"\s*\nversion = "([^"]+)"', (root / "core" / "Cargo.toml").read_text(encoding="utf-8"))
cargo_name, cargo_version = package.group(1), package.group(2)
# The crate bytes: `cargo package` refuses a git dependency that is not on crates.io (uor-hologram is
# not), so the package is a deterministic tar of what `cargo package` would take (Cargo.toml, Cargo.lock,
# src/**) plus the generated core the crate includes by path: sorted paths, zero mtimes, fixed owner.
import tarfile, time
crate_path = root / "core" / "target" / "package" / f"{cargo_name}-{cargo_version}.crate"
crate_path.parent.mkdir(parents=True, exist_ok=True)
members = sorted([root / "core" / "Cargo.toml", root / "core" / "Cargo.lock", root / "generated" / "hologram_forge_core.rs"] + [p for p in (root / "core" / "src").rglob("*") if p.is_file()])
with tarfile.open(crate_path, "w:gz", compresslevel=9, format=tarfile.PAX_FORMAT) as tar:
    for path in members:
        info = tarfile.TarInfo(str(path.relative_to(root)).replace("\\", "/"))
        data = path.read_bytes().replace(b"\r\n", b"\n")
        info.size, info.mtime, info.uid, info.gid, info.uname, info.gname, info.mode = len(data), 0, 0, 0, "", "", 0o644
        tar.addfile(info, io.BytesIO(data))
closure = json.loads((root / "shell" / "manifest.json").read_text(encoding="utf-8"))["closure"]
provenance = {
    "schema": "hologram-forge/provenance/1",
    "attestation_id": attestation_id,
    "build_id": build_id,
    "source_id": attestation["source_id"],
    "semantic_id": attestation["semantic_id"],
    "compiler_semantics_id": build_manifest["compiler"]["semantics_id"],
    "snapshot_id": build_id,
    "stdlib_semantics_id": stdlib["semantic_id"],
    "prism_stdlib_crate_sha256": stdlib["crate_sha256"],
    "lexlean_commit": lexlean_commit,
    "lexlean_package_sha256": lexlean_crate,
    "lean4_prod_commit": lean4_prod_commit,
    "hologram_live_commit": hologram_live_commit,
    "uor_hologram_commit": uor_hologram_commit,
    "target_profile_id": hashlib.sha256(target_profile).hexdigest(),
    "lean_manifest_sha256": sha(build_manifest_path),
    "lcnf_manifest_sha256": hashlib.sha256(lcnf_manifest).hexdigest(),
    "kernel_ir_sha256": kernel_sha,
    "coverage_sha256": coverage_sha,
    "roots_sha256": roots_sha,
    "generated_core_sha256": sha(root / "generated" / "hologram_forge_core.rs"),
    "cargo_name": cargo_name,
    "cargo_version": cargo_version,
    "cargo_crate_sha256": sha(crate_path),
    "cargo_crate_kind": "deterministic tar of Cargo.toml, Cargo.lock, src/** and generated/hologram_forge_core.rs (cargo package refuses the git dependency)",
    "view_model_id": sha(root / "src" / "Forge.lex.tex"),
    "browser_projection_sha256": closure,
    "prismpm_commit": (root / "PRISMPM_REV").read_text(encoding="utf-8").strip(),
}
missing = [k for k, v in provenance.items() if not v]
if missing:
    sys.exit("provenance fields could not be derived: " + ", ".join(missing))
out = root / "shell" / "provenance.json"
io.open(out, "w", encoding="utf-8", newline="\n").write(json.dumps(provenance, indent=1) + "\n")
print(f"wrote {out}: attestation {attestation_id[:12]}, closure {closure[:12]}, crate {provenance['cargo_crate_sha256'][:12]}")
