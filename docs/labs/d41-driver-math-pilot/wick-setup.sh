#!/usr/bin/env bash
# d41 donor pilot (claude-390), wick step 1 of 3: the external engine and the donor files.
# Runs inside Claude's wick container (local/claude/wick/wick.sh job d41-setup < this file).
# It runs no model. Nothing here is built into, or copied into, the repository.
set -euo pipefail
LLAMA_TAG=b12000
LLAMA_SHA=68d20b884c3878217de79eade0a249efde96ab38
DONOR=openai/gpt-oss-20b
REV=6cee5e81ee83917806bbde320786a8fb61efebee
W=/workspace/d41
mkdir -p "$W/donor-hf" "$W/engine"

export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq cmake python3-venv build-essential >/dev/null

# The engine: llama.cpp at a pinned tag, built with nvcc for sm_89 (the RTX 4090).
cd "$W/engine"
[ -d llama.cpp ] || git clone -q --branch "$LLAMA_TAG" --depth 1 https://github.com/ggml-org/llama.cpp
cd llama.cpp
test "$(git rev-parse HEAD)" = "$LLAMA_SHA"
cmake -B build -DGGML_CUDA=ON -DCMAKE_CUDA_ARCHITECTURES=89 -DLLAMA_CURL=OFF -DCMAKE_BUILD_TYPE=Release >/dev/null
cmake --build build --target llama-server -j 12 2>&1 | tail -2
./build/bin/llama-server --version 2>&1 | tail -2

# The donor: the released files at the pinned revision (no token: the repo is public).
cd "$W/donor-hf"
for f in LICENSE USAGE_POLICY config.json generation_config.json model.safetensors.index.json \
         model-00000-of-00002.safetensors model-00001-of-00002.safetensors model-00002-of-00002.safetensors \
         tokenizer.json tokenizer_config.json special_tokens_map.json chat_template.jinja; do
  [ -s "$f" ] || { curl -sfL -o "$f.part" "https://huggingface.co/$DONOR/resolve/$REV/$f" && mv "$f.part" "$f"; }
done
sha256sum ./* > "$W/donor-hf.sha256"
# The LFS files against the sha256 Hugging Face publishes for them at $REV (tree API, 15 Nov 2026).
sha256sum -c - <<SUMS
16d0f997dcfc4462089d536bffe51b4bcea2f872f5c430be09ef8ed392312427  model-00000-of-00002.safetensors
4fbe328ab445455d6f58dc73852b85873bd626986310abd91cd4d2ce3245eaea  model-00001-of-00002.safetensors
a18106b209e9ab35c3406db4f6f12a927364a058b21e9d1373d682e20674b303  model-00002-of-00002.safetensors
0614fe83cadab421296e664e1f48f4261fa8fef6e03e63bb75c20f38e37d07d3  tokenizer.json
SUMS
cat "$W/donor-hf.sha256"

# The converter's environment (llama.cpp's own requirements, outside the repository).
[ -x "$W/venv/bin/python" ] || python3 -m venv "$W/venv"
"$W/venv/bin/pip" install -q -r "$W/engine/llama.cpp/requirements/requirements-convert_hf_to_gguf.txt"
grep -n "repack_mxfp4\|MXFP4" "$W/engine/llama.cpp/conversion/gpt_oss.py" | head -20 || true
echo setup-done
