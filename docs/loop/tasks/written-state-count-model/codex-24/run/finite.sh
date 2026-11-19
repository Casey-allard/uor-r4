#!/usr/bin/env bash
set -euo pipefail
cd /workspace/d24/stage
sha256sum -c --status inputs.sha256
mkdir /workspace/d24/f1
out=/workspace/d24/f1
./d24-role finite "$out/planted.bank" > "$out/planted.txt"
./invalid-inputs.sh "$PWD/d24-role" "$PWD/d24-project" "$out/invalid" > "$out/invalid.txt"
# Projection is a finite parity check of four original ungraded TRAIN candidates per artifact.
for seed in seed1 seed2; do
 head -4 "$seed/train.candidates.txt" > "$out/$seed.tape"
 for mode in positional identity; do
  ./d24-project "$mode" "$seed/tables.bin" "$out/$seed.tape" "$out/$seed.$mode"
  head -4 "$seed/train.$mode.txt" | cmp - "$out/$seed.$mode"
 done
done
cd "$out"
sha256sum planted.bank planted.txt invalid.txt seed{1,2}.{tape,positional,identity} > finite.sha256
touch complete
