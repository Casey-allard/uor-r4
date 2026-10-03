#!/usr/bin/env bash
#
# Build the same pinned HF source in canonical D2 mode. The CI workflow runs
# this on Linux and macOS, then compares the source and output manifests.

set -euo pipefail

revision="7e27bd9f95328f0f3b08261d1252705110c806f8"
repository="HuggingFaceTB/SmolLM2-135M-Instruct"
source_dir="${D2_SOURCE_DIR:?D2_SOURCE_DIR must be set}"
output_dir="${D2_OUTPUT_DIR:?D2_OUTPUT_DIR must be set}"
manifest="${D2_MANIFEST:?D2_MANIFEST must be set}"
target="${D2_TARGET:-1000}"

mkdir -p "$source_dir" "$output_dir" "$(dirname "$manifest")"

download() {
    local filename="$1"
    curl --fail --silent --show-error --location --retry 5 --retry-all-errors \
        "https://huggingface.co/${repository}/resolve/${revision}/${filename}?download=true" \
        --output "$source_dir/$filename"
}

download config.json
download tokenizer.json
download model.safetensors

cargo run --release --bin r4 -- compile \
    --source "$source_dir" \
    --output "$output_dir" \
    --seconds 300 \
    --target "$target" \
    --sequence-length 128 \
    --canonical-deterministic

python3 - "$source_dir" "$output_dir" "$manifest" <<'PY'
import hashlib
import json
import pathlib
import platform
import sys


def digest_tree(root: pathlib.Path) -> dict[str, dict[str, int | str]]:
    files = {}
    for path in sorted(p for p in root.rglob("*") if p.is_file()):
        relative = path.relative_to(root).as_posix()
        data = path.read_bytes()
        files[relative] = {
            "bytes": len(data),
            "sha256": hashlib.sha256(data).hexdigest(),
        }
    return files


source, output, manifest = map(pathlib.Path, sys.argv[1:])
payload = {
    "revision": "7e27bd9f95328f0f3b08261d1252705110c806f8",
    "architecture": platform.machine(),
    "source": digest_tree(source),
    "output": digest_tree(output),
}
manifest.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
print(json.dumps(payload, indent=2, sort_keys=True))
PY
