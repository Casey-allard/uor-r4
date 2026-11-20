#!/usr/bin/env bash
# Continue a finished, reviewed Wick GPU spool into the registered count cap.
set -euo pipefail
umask 077
test "$(hostname -s)" = wick
root=$(realpath "$1")
go=$(realpath "$2")
test "$(sha256sum "$go" | cut -d' ' -f1)" = "$3"
test "$(jq -er '.decision + "/" + .dose' "$go")" = GO/c
spool=$(jq -er .output "$go")
census=$(jq -er .census "$go")
train=cd3adfe949ee34a62bc8d5e48592e091fcece796618d7910d1667d014a4bae2a
test "$(jq -er .corpus_sha256 "$go")" = "$train"
cap=3912936331
pruner="$root/pruner/target/release/prune-compact-count"
verifier=$(realpath "${5:?hardened verifier executable}")
verifier_sha=${6:?hardened verifier SHA-256}
source="$spool/compact.counts"
receipt="$spool/receipt.json"
printf '%s  %s\n' \
  28d06be01c22439fb715d6ba516845c8ee5a0530c0b0c09ec9d062461d4a4a53 "$census" \
  18dc458be3e6dc95ff0d963220d08d3dfd028a245a1783f93f4f7273be447897 "$pruner" \
  "$verifier_sha" "$verifier" | sha256sum --strict -c -
test "$(stat -c %s "$source")" = 148096362864
# The archive gives the adjusted hash domain for the highest raw order.
# Require both domain-specific raw hashes for all other orders.
jq -e --slurpfile prior "$census" --arg train "$train" '
  def shape: [.order,.contexts,.grams,.targets,.count_of_counts];
  def digests: [.context_sha256,.triple_sha256];
  def words: [shape,digests];
  . as $r | $prior[0] as $p |
  .plan.train_sha256 == $train and .artifact_bytes == 148096362864 and
  (.raw|length) == 11 and (.adjusted|length) == 11 and
  all(range(11); . as $i |
    ($r.adjusted[$i]|words) == ($p.orders[$i]|words) and
    ($r.raw[$i]|shape) == ($p.raw_orders[$i]|shape) and
    ($i == 10 or ($r.raw[$i]|digests) == ($p.raw_orders[$i]|digests))) and
  ($p.raw_orders[10]|words) == ($p.orders[10]|words)
' "$receipt" >/dev/null
source_hash=$(jq -er '.artifact_sha256 | select(test("^[0-9a-f]{64}$"))' "$receipt")
out="$4"
mkdir "$out"
candidate="$out/compact.counts"
/usr/bin/time -v -o "$out/prune.time" "$pruner" "$source" "$source_hash" \
  "$train" "$cap" "$candidate" > "$out/prune.json"
threshold=$(jq -er .threshold "$out/prune.json")
hash=$(jq -er .artifact_sha256 "$out/prune.json")
/usr/bin/time -v -o "$out/verify.time" "$verifier" "$source" "$candidate" \
  "$threshold" "$cap" "$train" "$source_hash" "$hash" > "$out/verify.json"
crosscheck=$(jq -e --slurpfile w "$out/prune.json" '
  def same($g;$s): ($g | del(.groups) + {contexts:.groups}) ==
                     ($s | del(.order,.data_bytes));
  . as $v | $w[0] as $p |
  .native_cold_load == "PASS" and .candidate_sha256 == $p.artifact_sha256 and
  .candidate_bytes == $p.artifact_bytes and .candidate_bytes <= .compact_cap and
  .threshold == $p.threshold and .prior_threshold_bytes > .compact_cap and
  .floor_bytes == $p.mandatory_floor_bytes and (.sections|length) == 11 and
  (.paired_native_lookups|length) > 0 and
  all(range(11); . as $i | $v.sections[$i] as $s |
    $s.order == $i and same($s.original;$p.original[$i]) and
    same($s.retained;$p.retained[$i]) and $s.section_data_bytes == $p.retained[$i].data_bytes)
' "$out/verify.json")
test "$crosscheck" = true
printf '%s\n' "$crosscheck" > "$out/crosscheck.partial"
mv "$out/crosscheck.partial" "$out/crosscheck"
(cd "$out" && sha256sum prune.json verify.json crosscheck > receipts.sha256)
