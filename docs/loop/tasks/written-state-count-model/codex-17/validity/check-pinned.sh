#!/usr/bin/env bash
# Entry point for real captures. Pre-run review supplies a separate digest
# for the exact manifest, including the integrity tools.
set -euo pipefail
umask 077
[[ $# == 3 ]] || { echo "usage: check-pinned.sh MANIFEST MANIFEST_SHA256 NEW_RECEIPT" >&2; exit 2; }
manifest=$1
[[ "$2" =~ ^[0-9a-f]{64}$ && "$3" == /* && ! -e "$3" ]]
hash() { shasum -a 256 "$1" | cut -d' ' -f1; }
[[ "$(hash "$manifest")" == "$2" ]]
jq -e '.schema=="uor-r4.rank-validity-inputs/1" and
    (.files|keys)==["binder","checker","extra","fb","fc","fg","hb","hc","hg","query"] and
    all(.files[]; (.path|type=="string" and startswith("/") and (test("[\\r\\n]")|not)) and
                  (.sha256|test("^[0-9a-f]{64}$")))' "$manifest" >/dev/null
pin() {
    local path sha
    path=$(jq -er --arg key "$1" '.files[$key].path' "$manifest") || return 1
    sha=$(jq -er --arg key "$1" '.files[$key].sha256' "$manifest") || return 1
    [[ "$(hash "$path")" == "$sha" ]] || return 1
    printf '%s\n' "$path"
}
binder=$(pin binder); checker=$(pin checker); query=$(pin query)
fc=$(pin fc); hc=$(pin hc); fb=$(pin fb); hb=$(pin hb)
fg=$(pin fg); hg=$(pin hg); extra=$(pin extra)
[[ "$query" == "$(dirname "$checker")/input.jq" ]]
"$binder" verify-input-rows "$fb" "$fc" "$fg" "$extra"
"$binder" verify-input-rows "$hb" "$hc" "$hg" -
"$checker" "$fc" "$hc" "$fb" "$hb" "$fg" "$extra" "$hg" "$3"
