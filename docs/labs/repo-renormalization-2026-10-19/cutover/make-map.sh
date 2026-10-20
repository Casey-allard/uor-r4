#!/bin/sh
# Regenerate the cutover move map (v3 rules) from the current tree. Run at the repo root.
# Rules (owner-approved 19 Oct 2026): docs/X -> archive/X at equal depth for
#   history, evidence, r4_*_evidence; docs/integration loose files and subdirs except the live hubs;
#   loose docs/*.{md,json,...} except the live references; dated docs/labs records older than CUTOFF.
# Live and never moved: docs/harness, docs/integration/{current-state,DECISIONS,project-track,NOW}.md,
#   docs/labs governance files and records dated CUTOFF or later, curated docs (mechanisms, figures,
#   geometry.md, adr, ...) which Phase 3 replaces.
set -eu
CUTOFF=2026-10-18   # records dated on or after this stay live
# Lab records named by harness code or tests stay live until the harness path registry lands.
PINNED_LABS="d30-no-python-2026-10-13 k16-timing-2026-10-15 pointer-gate-finetune-2026-10-10 prlint-fixtures-2026-10-14 prlint-linking-gates-2026-10-14 prlint-registry-gates-2026-10-14 rung0-pilot-release-2026-10-14"
KEEP_INT='^(current-state|DECISIONS|project-track|NOW)\.md$'
KEEP_LOOSE='^(README|CONFIGURATION|inference_contract|RELEASE_PIPELINE|formal_vocabulary|minimal_client|reproducibility|geometry|architecture|PROJECT_MAP|scaling_law|RESEARCH|geometric_intelligence_programme|hologram_r4_formal_monograph)\.md$'
{
  for d in history evidence $(git ls-files 'docs/r4_*_evidence/*' | awk -F/ '{print $2}' | sort -u); do
    printf 'docs/%s\tarchive/%s\n' "$d" "$d"
  done
  git ls-files 'docs/integration/*' | awk -F/ 'NF==3{print $3}' | grep -vE "$KEEP_INT" | while read -r f; do
    printf 'docs/integration/%s\tarchive/integration/%s\n' "$f" "$f"
  done
  git ls-files 'docs/integration/*' | awk -F/ 'NF>=4{print $3}' | sort -u | while read -r d; do
    printf 'docs/integration/%s\tarchive/integration/%s\n' "$d" "$d"
  done
  git ls-files 'docs/*' | awk -F/ 'NF==2{print $2}' | grep -vE "$KEEP_LOOSE" | while read -r f; do
    printf 'docs/%s\tarchive/%s\n' "$f" "$f"
  done
  git ls-files docs/labs | awk -F/ 'NF>=4{print $3}' | sort -u \
    | grep -E -- '-[0-9]{4}-[0-9]{2}-[0-9]{2}$|[0-9]{8}$' | awk -v c="$CUTOFF" '{ d=$0; if (match(d, /[0-9]{4}-[0-9]{2}-[0-9]{2}$/)) { if (substr(d, RSTART) < c) print d } else print d }' | while read -r d; do
    case " $PINNED_LABS " in *" $d "*) continue;; esac
    printf 'docs/labs/%s\tarchive/labs/%s\n' "$d" "$d"
  done
} | sort -u
