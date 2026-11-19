#!/usr/bin/env bash
set -euo pipefail
if [[ $# -ne 3 ]]; then
    printf 'Usage: %s ROLE_BINARY PROJECT_BINARY NEW_OUTPUT_DIRECTORY\n' "$0" >&2
    exit 2
fi
role=$1 project=$2 out=$3
umask 077
mkdir "$out"
printf '%s\n' '1 2 3' > "$out/short.features"
printf '0' > "$out/good.features"
printf ' 0%.0s' {1..31} >> "$out/good.features"
printf '\n' >> "$out/good.features"
printf '3\n' > "$out/bad.labels"
: > "$out/empty.labels"
printf 'D24ROLE1' > "$out/truncated.bank"
{ printf 'D24ROLE1'; dd if=/dev/zero bs=792 count=1 status=none; } > "$out/zero-count.bank"
printf '0 1 0 1 0 5\n' > "$out/bad-span.candidates"
reject() {
    local name=$1 diagnostic=$2
    shift 2
    if "$@" > "$out/$name.stdout" 2> "$out/$name.stderr"; then
        printf '%s unexpectedly succeeded\n' "$name" >&2
        exit 1
    fi
    grep -Fx "$diagnostic" "$out/$name.stderr" >/dev/null
    test ! -e "$out/unexpected"
}
reject short-features 'Expected 32 signed i32 coordinates' \
    "$role" fit "$out/short.features" "$out/bad.labels" "$out/unexpected"
reject bad-label 'Expected role 0, 1 or 2' \
    "$role" fit "$out/good.features" "$out/bad.labels" "$out/unexpected"
reject absent-label 'Wrong training label count' \
    "$role" fit "$out/good.features" "$out/empty.labels" "$out/unexpected"
reject truncated-bank 'Truncated state' \
    "$role" predict "$out/good.features" "$out/truncated.bank" "$out/unexpected"
reject zero-count 'Each role requires 1 to 1000000 training candidates' \
    "$role" predict "$out/good.features" "$out/zero-count.bank" "$out/unexpected"
reject bad-arm 'Unknown projector arm' \
    "$project" bogus "$out/empty.labels" "$out/bad-span.candidates" "$out/unexpected"
dd if=/dev/zero of="$out/bad.tables" bs=33408 count=1 status=none
reject bad-tables 'invalid signed-axis code' \
    "$project" identity "$out/bad.tables" "$out/bad-span.candidates" "$out/unexpected"
{
    dd if=/dev/zero bs=32896 count=1 status=none
    printf '\000\001\002\003\004\005\006\007%.0s' {1..64}
} > "$out/good.tables"
reject bad-span 'Invalid half-open mention and clause bounds' \
    "$project" identity "$out/good.tables" "$out/bad-span.candidates" "$out/unexpected"
printf 'PASS eight exact CLI refusals with no output artifact\n'
