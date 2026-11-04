#!/usr/bin/env bash
set -euo pipefail

RELEASE_TAG='studio-native-runner-v0.1.0'
RELEASE_ROOT="https://github.com/UOR-Foundation/uor-r4/releases/download/${RELEASE_TAG}"
RUNNER_HOME="${UOR_RUNNER_HOME:-${HOME}/.cache/uor-r4-studio/${RELEASE_TAG}}"
BINARY_NAME='uor-chat-arm64-apple-darwin'
STUDIO_ARCHIVE_NAME='studio-local.tar.gz'
UOR_CHAT_SHA256='f4f36c72baea6b0fb76c72ff511cf8e9615fd328022d3b08937660b28cc05f7e'
STUDIO_BUNDLE_SHA256='bf78084e09d03736d3de5cfbe98040e632cc102946f6396434f12314b9ede04d'
STACK_SHA256='d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df'
TOKENIZER_SHA256='6758318dbcad5925a550103a5c2ee41e35898c30a7daf066da09b0159a41c57b'
RESEARCH_MODEL='dual-c131-seed-1-raw_both.uorlut'
RESEARCH_TOKENIZER='tokenizer-seed-1.json'

if [ "${1:-}" = '--check' ]; then
    printf 'release: %s\n' "$RELEASE_TAG"
    printf 'binary: %s\n' "$BINARY_NAME"
    printf 'studio: %s\n' "$STUDIO_ARCHIVE_NAME"
    printf 'model: synthetic-stack.lut (%s)\n' "$STACK_SHA256"
    printf 'tokenizer: tokenizer.json (%s)\n' "$TOKENIZER_SHA256"
    printf 'not downloaded (research only): %s and %s\n' "$RESEARCH_MODEL" "$RESEARCH_TOKENIZER"
    printf 'serve: 127.0.0.1:8000 (loopback only)\n'
    exit 0
fi
if [ "$#" -ne 0 ]; then
    printf 'usage: bash native-runner.sh [--check]\n' >&2
    exit 2
fi
if [ "$(uname -s)" != 'Darwin' ] || [ "$(uname -m)" != 'arm64' ]; then
    printf 'This prerelease supports only macOS on Apple silicon (Darwin arm64).\n' >&2
    exit 1
fi
for command_name in curl shasum openssl tar; do
    if ! command -v "$command_name" >/dev/null 2>&1; then
        printf 'Required command is unavailable: %s\n' "$command_name" >&2
        exit 1
    fi
done

verify_hash() {
    local path="$1"
    local expected="$2"
    local actual
    actual="$(shasum -a 256 "$path")"
    actual="${actual%% *}"
    if [ "$actual" != "$expected" ]; then
        printf 'SHA-256 mismatch for %s\nexpected: %s\nactual:   %s\n' "$path" "$expected" "$actual" >&2
        return 1
    fi
}

fetch_verified() {
    local url="$1"
    local destination="$2"
    local expected="$3"
    local partial="${destination}.part.$$"
    if [ -f "$destination" ] && verify_hash "$destination" "$expected" 2>/dev/null; then
        return 0
    fi
    if ! curl --fail --location --silent --show-error --proto '=https' --tlsv1.2 "$url" --output "$partial"; then
        rm -f "$partial"
        return 1
    fi
    if ! verify_hash "$partial" "$expected"; then
        rm -f "$partial"
        return 1
    fi
    mv "$partial" "$destination"
}

mkdir -p "$RUNNER_HOME"
fetch_verified "${RELEASE_ROOT}/${BINARY_NAME}" "${RUNNER_HOME}/${BINARY_NAME}" "$UOR_CHAT_SHA256"
fetch_verified "${RELEASE_ROOT}/${STUDIO_ARCHIVE_NAME}" "${RUNNER_HOME}/${STUDIO_ARCHIVE_NAME}" "$STUDIO_BUNDLE_SHA256"
chmod 755 "${RUNNER_HOME}/${BINARY_NAME}"

tar -tzf "${RUNNER_HOME}/${STUDIO_ARCHIVE_NAME}" | while IFS= read -r entry; do
    case "$entry" in
        /*|../*|*/../*|*/..)
            printf 'Unsafe path in Studio bundle: %s\n' "$entry" >&2
            exit 1
            ;;
    esac
done
tar -xzf "${RUNNER_HOME}/${STUDIO_ARCHIVE_NAME}" -C "$RUNNER_HOME"
STUDIO_ROOT="${RUNNER_HOME}/studio"
verify_hash "${STUDIO_ROOT}/pkg/synthetic-stack.lut" "$STACK_SHA256"
verify_hash "${STUDIO_ROOT}/pkg/tokenizer.json" "$TOKENIZER_SHA256"

UOR_RUNNER_TOKEN="$(openssl rand -hex 32)"
trap 'unset UOR_RUNNER_TOKEN' EXIT
printf 'Native synthetic protocol demo: http://127.0.0.1:8000/\n'
printf 'Press Ctrl+C to stop. No useful-conversation or visual-understanding capability is claimed.\n'
printf '%s\n' "$UOR_RUNNER_TOKEN" | "${RUNNER_HOME}/${BINARY_NAME}" \
    --stack "${STUDIO_ROOT}/pkg/synthetic-stack.lut" \
    --tokenizer "${STUDIO_ROOT}/pkg/tokenizer.json" \
    --serve 127.0.0.1:8000 \
    --studio-root "$STUDIO_ROOT" \
    --token-stdin
