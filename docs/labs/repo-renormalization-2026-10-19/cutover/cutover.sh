#!/bin/sh
# Docs cutover, as one reproducible sequence. Run at the root of a clean checkout of the
# cutover branch. XT = path to a built xtask binary (built from this branch).
# Each stage is its own commit so review can check them separately.
set -eu
XT=${XT:?set XT to the built xtask binary}
CATALOG_SRC=${CATALOG_SRC:?set CATALOG_SRC to the merged catalog.jsonl}
MAPSCRIPT=${MAPSCRIPT:?set MAPSCRIPT to make-map.sh}
W=local/cutover; mkdir -p "$W"
unset CARGO_MANIFEST_DIR
GIT="git"

echo "== map"
sh "$MAPSCRIPT" > "$W/map.tsv"; wc -l < "$W/map.tsv"
echo "== baselines"
$XT links --json "$W/links-base.json" | tail -1
$XT archive-verify --json "$W/av-base.json" | tail -1
BASE=$(git rev-parse HEAD)

echo "== 1. relink live docs + rewrite live code paths"
$XT docs-relink --map "$W/map.tsv" --apply | tail -1
$XT docs-rewrite-refs --map "$W/map.tsv" --exclude crates/uor-r4-harness --exclude scripts/harness --exclude tests/harness --exclude crates/uor-r4-claim-wording --apply | tail -1
$GIT commit -qam "docs(cutover): point live links and code paths at archive/ (pre-move rewrite)

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"

echo "== 2. move (renames only)"
$XT docs-move --map "$W/map.tsv" --apply | tail -1
$GIT commit -qm "docs(cutover): move dated records to archive/ at equal depth (renames only)

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"

echo "== 3. pointer stubs for archived links into live files"
$XT links --baseline "$W/links-base.json" --renames-from "$BASE" > "$W/after-move.txt" 2>&1 || true
grep '^  NEW' "$W/after-move.txt" | awk '{split($2,a,":"); if (a[1] ~ /^archive\//) print $NF}' | sort -u \
  | while read -r t; do p=${t#archive/}; [ "$p" != "$t" ] || continue
      if git ls-files --error-unmatch "docs/$p" >/dev/null 2>&1 || [ -n "$(git ls-files "docs/$p/" | head -1)" ]; then echo "$t"; fi
    done > "$W/stubs.txt"
wc -l < "$W/stubs.txt"
$XT docs-stubs --list "$W/stubs.txt" --apply | tail -1
git add archive && $GIT commit -qm "docs(cutover): pointer stubs and figure snapshots where archived records link to live files

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"

echo "== 4. catalog"
cp "$CATALOG_SRC" archive/catalog.jsonl
$XT docs-catalog | tail -1
$XT docs-catalog --check | tail -1
git add archive && $GIT commit -qm "docs(cutover): archive catalog (1,173 records) generated from archive/catalog.jsonl

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"

echo "== gates"
echo "-- R100 only in the move commit:"; git diff -M100% --name-status HEAD~3 HEAD~2 | grep -vc '^R100' || true
echo "-- archive-verify vs base:"; $XT archive-verify --baseline "$W/av-base.json" --renames-from "$BASE" --base "$BASE" | tail -2
echo "-- links vs base:"; $XT links --baseline "$W/links-base.json" --renames-from "$BASE" > "$W/after-all.txt" 2>&1 || true
sed -n 1,3p "$W/after-all.txt"
grep '^  NEW' "$W/after-all.txt" | awk '{split($2,a,":"); s=a[1]; c="other"; if (s ~ /^archive\//) c="archive"; else if (s ~ /^docs\/harness\/prep\//) c="pinned-prep"; else if (s ~ /^docs\/labs\//) c="labs-record"; print c}' | sort | uniq -c
echo "-- other-class new breaks (must be none):"; grep '^  NEW' "$W/after-all.txt" | awk '{split($2,a,":"); s=a[1]; if (s !~ /^archive\// && s !~ /^docs\/harness\/prep\// && s !~ /^docs\/labs\//) print}' | head -20
echo DONE
