#!/usr/bin/env bash
# Structural rehearsal only. Contains no model, grade or panel data.
set -euo pipefail
umask 077
[[ $# == 1 && "$1" == /* ]]
out="$1"
here=$(cd "$(dirname "$0")" && pwd -P)
mkdir "$out"
cd "$out"
for split in fit held; do
    printf '%s\n' "$split" > "$split-spec.json"
    for run in primary repeat; do
        root="$out/$run-$split"
        mkdir "$root"
        cp "$split-spec.json" "$root/stage-spec.json"
        printf 'same canonical bytes\n' > "$root/candidates.json"
        printf 'allowed raw receipt path difference %s\n' "$run" > "$root/audit.log"
        (cd "$root" && sha256sum stage-spec.json candidates.json > cold.sha256)
        (cd "$root" && find . -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum) > "$root.all.sha256"
    done
done
pin() { jq -n --arg path "$1" --arg sha "$(sha256sum "$1" | cut -d' ' -f1)" '{path:$path,sha256:$sha}'; }
capture() { jq -n --arg root "$1" --argjson raw "$(pin "$1.all.sha256")" '{root:$root,raw:$raw}'; }
jq -n --argjson fp "$(capture "$out/primary-fit")" --argjson fr "$(capture "$out/repeat-fit")" \
      --argjson hp "$(capture "$out/primary-held")" --argjson hr "$(capture "$out/repeat-held")" \
      --argjson fs "$(pin "$out/fit-spec.json")" --argjson hs "$(pin "$out/held-spec.json")" '
  {captures:{fit:{primary:$fp,repeat:$fr},held:{primary:$hp,repeat:$hr}},files:{fit_stage:$fs,held_stage:$hs}}
' > spec.json
mkdir primary-study repeat-study refuse-study
"$here/reuse-capture.sh" "$out/spec.json" primary "$out/primary-study"
"$here/reuse-capture.sh" "$out/spec.json" repeat "$out/repeat-study"
[[ "$(readlink primary-study/fit)" == "$out/primary-fit" && "$(readlink repeat-study/fit)" == "$out/repeat-fit" ]]
cmp primary-study/reused-fit.json repeat-study/reused-fit.json
printf 'forged, outside the cold subset\n' >> repeat-fit/audit.log
if "$here/reuse-capture.sh" "$out/spec.json" primary "$out/refuse-study" > refuse.log 2>&1; then exit 1; fi
[[ ! -e refuse-study/fit ]]
printf '%s\n' 'Both pinned selections agreed. A changed non-cold raw file was refused before linking.'
