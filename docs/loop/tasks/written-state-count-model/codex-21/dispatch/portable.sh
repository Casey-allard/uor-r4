#!/usr/bin/env bash
# Package only the new unit. The frozen predecessor has its own verified portable archive.
set -euo pipefail
umask 077
stage=${1:?sealed stage}
out=${2:?new archive}
[[ "$stage" == /home/callard/uor-codex/20261119-clause-veto && "$out" == /* && ! -e "$out" ]]
cd "$stage"
sha256sum --status --check inputs.sha256
sha256sum --status --check build-receipt/source.sha256
sha256sum --status --check build-receipt/executables.sha256
cd ..
name=${stage##*/}
tar --exclude="$name/build-draft*" --exclude="$name/target" --exclude="$name/ops" --exclude="$name/held*" -cf - "$name" | zstd -q -T2 -3 --long=28 -o "$out"
sha256sum "$out"
