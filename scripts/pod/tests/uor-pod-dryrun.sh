#!/usr/bin/env bash
# Dry-run test of scripts/pod/uor-pod (leases per lab+session, up placement and
# caps, down, reap, register/keep, prune-stopped, status) against a fake
# runpodctl, vastai, ssh and gh on PATH (no network, no cost):
#
#   scripts/pod/tests/uor-pod-dryrun.sh
#
# The fake account holds pod A (2 x 5090), pod B (1 x 4090, idle, unleased,
# reaper idle for 30 min), pod C (1 x 4090, busy, unleased, non-canonical
# volume), pod D and pod E (stopped; E for 3 days). Stock: 5090 none in
# EUR-NO-1 but Low in EU-RO-1 (no uor-shared volume there yet); 4090 Low in EUR-NO-1.
# Also covered: lease timestamps in every RFC 3339 form (and the fail-safe for
# unparseable ones, in uor-pod and the pod-side reaper), the helper preflight of
# `up`, datacenter fallback on create-time stock errors (non-dry, against the
# fake runpodctl only), and the bootstrap's bounded build-lock wait and atomic
# cache publish (its cache functions sourced with a stub build and fake flock),
# the argument preflight of `up` (--ref against a local stand-in origin, option
# checks, the bootstrap's --check-args), the circuit breaker, the saved bootstrap
# log of a failed `up` (non-dry, fake pod), and the non-fatal Ollama install.
set -euo pipefail

HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
TOOL=$HERE/../uor-pod
W=$(mktemp -d "${TMPDIR:-/tmp}/uor-pod-test.XXXXXX")
trap 'rm -rf "$W"' EXIT
mkdir -p "$W/bin" "$W/fake"
export UOR_POD_STATE=$W/state UOR_POD_DRY_RUN=1 FAKE=$W/fake PATH="$W/bin:$PATH"
# Every Vast call uses the fixture adapter, including non-dry mocked placement;
# never inherit an owned/live Rust provider binary from the invoking session.
export UOR_POD_VAST_BIN=$W/bin/vastai
unset UOR_POD_SESSION UOR_POD_MAX_PODS UOR_POD_MAX_RATE
export UOR_POD_CONFIG=$W/no-config  # a real user config never loads
unset UOR_POD_REPO_SLUG UOR_POD_CANONICAL_DC UOR_POD_VOLUME_ID UOR_POD_NEW_VOLUME_GB UOR_POD_IMAGE UOR_POD_BOARD UOR_POD_TEMPLATE UOR_HF_STORE
unset UOR_POD_VAST_DISK_GB UOR_POD_VAST_MIN_RELIABILITY UOR_POD_VAST_MIN_INET_MBPS UOR_POD_VAST_ARCHIVE_MAX_MB
unset UOR_POD_VAST_DRIVER_VERSION
# Vast.ai became the default provider for new pods on 2026-10-13. Every test
# before the Vast section at the end exercises the Runpod path, so it is chosen
# here; the Vast section sets its own provider. (The fake vastai below lists no
# instances, so those tests see exactly the Runpod pods they did before.)
export UOR_POD_PROVIDER=runpod
# (the project uses Vast only since 13 October, Runpod as a backup; until the Vast
# section these tests are a Runpod-only account, as config.example sets one up)
export UOR_POD_PROVIDERS=runpod
# A local stand-in for the GitHub repository `--ref` is checked against (no
# network): main (2 commits), branch feature (2 commits), and a clone holding
# one commit that was never pushed.
G() { git -c user.name=t -c user.email=t@example.invalid -c init.defaultBranch=main -c advice.detachedHead=false "$@"; }
G init -q --bare "$W/origin.git"
G clone -q "$W/origin.git" "$W/clone" 2>/dev/null
G -C "$W/clone" commit -q --allow-empty -m one
G -C "$W/clone" commit -q --allow-empty -m two
G -C "$W/clone" push -q origin HEAD:refs/heads/main
MAIN_SHA=$(G -C "$W/clone" rev-parse HEAD)
G -C "$W/clone" checkout -q -b feature
G -C "$W/clone" commit -q --allow-empty -m f1
FEAT1_SHA=$(G -C "$W/clone" rev-parse HEAD)
G -C "$W/clone" commit -q --allow-empty -m f2
G -C "$W/clone" push -q origin feature
FEAT_TIP=$(G -C "$W/clone" rev-parse HEAD)
G -C "$W/clone" commit -q --allow-empty -m local-only
LOCAL_SHA=$(G -C "$W/clone" rev-parse HEAD)
export UOR_POD_REPO_URL=$W/origin.git UOR_POD_GIT_DIR=$W/clone
NOW=$(date -u +%s)
OLD=$(date -u -r $((NOW - 3 * 86400)) '+%a %b %d %Y %H:%M:%S' 2>/dev/null || date -u -d "@$((NOW - 3 * 86400))" '+%a %b %d %Y %H:%M:%S')

cat > "$FAKE/pods.json" <<J
[
 {"id":"poda","name":"a","desiredStatus":"RUNNING","gpuCount":2,"costPerHr":1.98,"uptimeSeconds":3600,"env":{"SECRET":"must-not-print"}},
 {"id":"podb","name":"b","desiredStatus":"RUNNING","gpuCount":1,"costPerHr":0.74,"uptimeSeconds":7200},
 {"id":"podc","name":"c","desiredStatus":"RUNNING","gpuCount":1,"costPerHr":0.74,"uptimeSeconds":7200,"networkVolumeId":"volro"},
 {"id":"podd","name":"d","desiredStatus":"EXITED","gpuCount":2,"costPerHr":1.48,"uptimeSeconds":0,"lastStatusChange":"Exited by user: $OLD GMT+0000 (UTC)"},
 {"id":"pode","name":"e","desiredStatus":"EXITED","gpuCount":1,"costPerHr":0.74,"uptimeSeconds":0,"lastStatusChange":"Exited by user: $OLD GMT+0000 (UTC)"}
]
J
cat > "$FAKE/gpus.json" <<'J'
[
 {"gpuId":"NVIDIA GeForce RTX 5090","securePricePerHr":0.99,"dataCenterAvailability":[{"dataCenterId":"EUR-NO-1","stockStatus":"none"},{"dataCenterId":"EU-RO-1","stockStatus":"Low"}]},
 {"gpuId":"NVIDIA GeForce RTX 4090","securePricePerHr":0.74,"dataCenterAvailability":[{"dataCenterId":"EUR-NO-1","stockStatus":"Low"}]},
 {"gpuId":"NVIDIA A100 80GB PCIe","securePricePerHr":1.59,"dataCenterAvailability":[{"dataCenterId":"EUR-NO-1","stockStatus":"High"}]}
]
J
# ssh ports: poda 1001, podb 1002, podc 1003
printf 'gpu=0, NVIDIA GeForce RTX 5090, 0, 1\ngpu=1, NVIDIA GeForce RTX 5090, 0, 1\nreaper=1\nprobe_ok=1\n' > "$FAKE/probe-1001"
printf 'gpu=0, NVIDIA GeForce RTX 4090, 0, 1\nidle_since=%s\nreaper=1\nprobe_ok=1\n' $((NOW - 1800)) > "$FAKE/probe-1002"
printf 'gpu=0, NVIDIA GeForce RTX 4090, 97, 20000\napp=4242\nreaper=0\nprobe_ok=1\n' > "$FAKE/probe-1003"

cat > "$W/bin/runpodctl" <<'S'
#!/usr/bin/env bash
echo "runpodctl $*" >> "$FAKE/calls"
case "$1 $2" in
  "pod list")
    # $FAKE/rp-fail-once: this listing fails, the next one succeeds
    if [ -f "$FAKE/rp-fail-once" ]; then rm -f "$FAKE/rp-fail-once"; echo "Error: 502" >&2; exit 1; fi
    cat "$FAKE/pods.json";;
  "pod get") jq --arg i "$3" '.[] | select(.id == $i) | {networkVolumeId: "lmd1pfah3y"} + .
     + (if .desiredStatus == "RUNNING" then {ssh: {ip: "10.0.0.1", port: ({"poda":1001,"podb":1002,"podc":1003,"podnew":1004}[$i])}} else {} end)' "$FAKE/pods.json";;
  "gpu list") cat "$FAKE/gpus.json";;
  "network-volume list") echo '[{"id":"lmd1pfah3y","name":"uor-shared-EUR-NO-1","dataCenterId":"EUR-NO-1","size":200}]';;
  # FAKE_ALLOW_CREATE=1 (only the non-dry placement tests at the end): pod create
  # succeeds unless the datacenter is listed in $FAKE/nostock (Runpod's
  # out-of-stock error) or $FAKE/create-error exists (any other error).
  "pod create")
    [ "${FAKE_ALLOW_CREATE:-0}" = 1 ] || { echo "REAL MUTATION CALLED IN DRY RUN" >&2; exit 9; }
    dc='' n='' prev=''; for a in "$@"; do [ "$prev" = --data-center-ids ] && dc=$a; [ "$prev" = --gpu-count ] && n=$a; prev=$a; done
    # $FAKE/max-count: no host has more than that many free GPUs
    if grep -qx "$dc" "$FAKE/nostock" 2>/dev/null || { [ -f "$FAKE/max-count" ] && [ "$n" -gt "$(cat "$FAKE/max-count")" ]; }; then
      echo "Error: There are no longer any instances available with the requested specifications. Please refresh and try again." >&2; exit 1
    fi
    [ ! -f "$FAKE/create-error" ] || { echo "Error: template h15vb984sw not found" >&2; exit 1; }
    jq '. + [{"id":"podnew","name":"new","desiredStatus":"RUNNING","gpuCount":2,"costPerHr":1.98,"uptimeSeconds":0}]' "$FAKE/pods.json" > "$FAKE/pods.json.new"
    mv "$FAKE/pods.json.new" "$FAKE/pods.json"
    echo '{"id":"podnew"}';;
  "network-volume create")
    [ "${FAKE_ALLOW_CREATE:-0}" = 1 ] || { echo "REAL MUTATION CALLED IN DRY RUN" >&2; exit 9; }
    echo '{"id":"volnew"}';;
  "network-volume delete")
    [ "${FAKE_ALLOW_CREATE:-0}" = 1 ] || { echo "REAL MUTATION CALLED IN DRY RUN" >&2; exit 9; }
    echo '{}';;
  "pod delete")
    [ "${FAKE_ALLOW_CREATE:-0}" = 1 ] || { echo "REAL MUTATION CALLED IN DRY RUN" >&2; exit 9; }
    echo '{}';;
  *) echo "fake runpodctl: unhandled $*" >&2; exit 1;;
esac
S
# fake vastai: $FAKE/vast-instances.json is the account; offers come from
# $FAKE/vast-offers-<gpu_name>-<num_gpus>.json; an offer id listed in
# $FAKE/vast-gone was taken after the search. Mutations need FAKE_ALLOW_CREATE=1.
# Like the real CLI, `destroy instance` asks [y/N] and aborts unless stdin says y,
# and `create instance` returns an instance-scoped key (a sentinel here).
cat > "$W/bin/vastai" <<'S'
#!/usr/bin/env bash
echo "vastai $*" >> "$FAKE/calls"
inst=$FAKE/vast-instances.json; [ -f "$inst" ] || echo '[]' > "$inst"
case "$1 $2" in
  "show instances")
    [ "${FAKE_VAST_LIST_FAIL:-0}" != 1 ] || { echo "failed with error 502" >&2; exit 1; }
    # nothing at all with exit 0 (FAKE_VAST_LIST_EMPTY=1, or =after-create once a rental happened)
    case ${FAKE_VAST_LIST_EMPTY:-0} in 1) exit 0;; after-create) [ ! -f "$FAKE/vast-created" ] || exit 0;; esac
    # an API error printed as an object with exit 0 (an expired key, a 5xx page)
    [ "${FAKE_VAST_LIST_OBJ:-0}" != 1 ] || { echo '{"error": "invalid_api_key"}'; exit 0; }
    cat "$inst";;
  "show instance") jq --argjson i "$3" 'map(select(.id == $i)) | if length > 0 then .[0] else {instances: null} end' "$inst";;
  "search offers")
    g=$(printf '%s' "$3" | sed -n 's/.*gpu_name=\([^ ]*\).*/\1/p'); n=$(printf '%s' "$3" | sed -n 's/.*num_gpus=\([0-9]*\).*/\1/p')
    echo "$*" > "$FAKE/vast-query"
    if [ "${FAKE_VAST_LEDGER_BLOCK:-0}" = 1 ]; then
      mv "$UOR_POD_STATE/ledger.jsonl" "$UOR_POD_STATE/ledger-before-block.jsonl"
      mkdir "$UOR_POD_STATE/ledger.jsonl"
    fi
    cat "$FAKE/vast-offers-$g-$n.json" 2>/dev/null || echo '[]';;
  "create instance")
    [ "${FAKE_ALLOW_CREATE:-0}" = 1 ] || { echo "REAL MUTATION CALLED IN DRY RUN" >&2; exit 9; }
    if [ -f "$UOR_POD_STATE/ledger.jsonl" ]; then tail -1 "$UOR_POD_STATE/ledger.jsonl" > "$FAKE/ledger-at-create.json"; fi
    if grep -qx "$3" "$FAKE/vast-gone" 2>/dev/null; then echo "{\"success\": false, \"msg\": \"offer $3 is no longer available\"}"; exit 0; fi
    label=''; prev=''; for a in "$@"; do [ "$prev" = --label ] && label=$a; prev=$a; done
    touch "$FAKE/vast-created"
    c=$((9001 + $(jq length "$inst")))
    jq --argjson c "$c" --arg l "$label" --argjson now "$(date +%s)" '. + [{id: $c, label: $l, num_gpus: 1, dph_total: 0.469,
      gpu_name: "RTX 5090", geolocation: "South Korea, KR", intended_status: "running", actual_status: "running",
      public_ipaddr: "10.0.0.2", ports: {"22/tcp": [{"HostIp": "0.0.0.0", "HostPort": "2001"}]}, start_date: $now,
      jupyter_token: "SENTINEL-JUPYTER-TOKEN"}]' "$inst" > "$inst.new" && mv "$inst.new" "$inst"
    # FAKE_VAST_CREATE_GARBLED=1: rented, but the reply does not parse
    [ "${FAKE_VAST_CREATE_GARBLED:-0}" != 1 ] || { echo "Warning: client out of date"; echo "{\"success\": true, \"new_contract\": $c"; exit 0; }
    echo "{\"success\": true, \"new_contract\": $c, \"instance_api_key\": \"SENTINEL-INSTANCE-KEY\"}";;
  "destroy instance")
    [ "${FAKE_ALLOW_CREATE:-0}" = 1 ] || { echo "REAL MUTATION CALLED IN DRY RUN" >&2; exit 9; }
    ans=''; read -r ans || true
    printf 'Are you sure you want to destroy instance %s? [y/N] ' "$3"
    [ "$ans" = y ] || { echo "Aborted."; exit 0; }
    echo "destroying instance $3."
    jq --argjson i "$3" 'map(select(.id != $i))' "$inst" > "$inst.new" && mv "$inst.new" "$inst";;
  *) echo "fake vastai: unhandled $*" >&2; exit 1;;
esac
S
cat > "$W/bin/ssh" <<'S'
#!/usr/bin/env bash
port=''; while [ $# -gt 0 ]; do case $1 in -p) port=$2; shift 2;; -i|-o) shift 2;; root@*) shift; break;; *) shift;; esac; done
echo "ssh $port $*" >> "$FAKE/calls"
case "$*" in *probe_ok*) cat "$FAKE/probe-$port" 2>/dev/null;; esac
# a Vast pod's archive at `down` runs for real against $FAKE/vast-ws (its
# /workspace), so the exit status is what a pod would return; FAKE_VAST_KB
# stands in for a large result set
c=$*
case "$c" in
  *"du -sk uor-r4/"*) if [ -n "${FAKE_VAST_KB:-}" ]; then echo "$FAKE_VAST_KB"; exit 0; fi; bash -c "${c//\/workspace/$FAKE/vast-ws}"; exit;;
  *"for d in uor-r4/jobs"*|*"tar -cf - uor-r4/"*) bash -c "${c//\/workspace/$FAKE/vast-ws}"; exit;;
  *"uor-store ls "*)
    d=$(printf '%s' "$c" | sed -n 's/.*vast-archive\/[^/ ]*\/\([^ ]*\).*/\1/p')
    l=$(printf '%s' "$c" | sed -n 's/.*> \([^ ]*uor-store-list[^ ]*\).*/\1/p')
    n=$(find "$FAKE/vast-ws/$d" -type f 2>/dev/null | wc -l | tr -d " ")
    [ "${FAKE_STORE_TRUNC:-0}" = 1 ] && n=$((n > 0 ? n - 1 : 0))
    i=0; while [ "$i" -lt "$n" ]; do echo "0.0 MB  vast-archive/$d/f$i"; i=$((i+1)); done > "$l"
    [ "${FAKE_STORE_LATE_FAIL:-0}" = 1 ] && exit 23
    exit 0;;
  *"uor-store get "*) echo MATCH; exit 0;;
  *"sha256sum /tmp/uor-store-manifest"*)
    echo "FAKEMANIFESTHASH  $(printf '%s' "$c" | sed -n 's/.*sha256sum \([^ ]*\).*/\1/p')"; exit 0;;
  *"wc -l < /tmp/uor-store-"*) bash -c "$c"; exit;;
  *"find . -type f -print0"*)
    d=$(printf '%s' "$c" | sed -n 's|.*/workspace/\([^ ]*\) .*|\1|p')
    f=$(printf '%s' "$c" | sed -n 's|.*-print0 > \([^ ]*\).*|\1|p')
    m=$(printf '%s' "$c" | sed -n 's|.*> \([^ ]*\.sha256\).*|\1|p')
    n=$(find "$FAKE/vast-ws/$d" -type f 2>/dev/null | wc -l | tr -d " ")
    : > "$f"; : > "$m"; i=0; while [ "$i" -lt "$n" ]; do echo "./f$i" >> "$f"; echo "HASH  ./f$i" >> "$m"; i=$((i+1)); done; exit 0;;
  *"/root/uor-store put "*) exit 0;;
esac
# the bootstrap run (non-dry tests): 40 lines of output, then either the failure
# of 2026-10-05 (Ollama's .tgz 404 -> tar, rc 2) or a BOOTSTRAP_RESULT with
# "ollama": $FAKE_OLLAMA; the exit status line the real pod-side wrapper appends
case "$*" in *"uor-pod-bootstrap.sh --sha"*)
  echo "$*" > "$FAKE/bootcmd"
  i=1; while [ $i -le 40 ]; do echo "bootstrap line $i"; i=$((i + 1)); done
  if [ "${FAKE_BOOT_RC:-0}" != 0 ]; then
    echo "curl: (22) The requested URL returned error: 404"; echo "tar: Error is not recoverable: exiting now"
  else
    echo "BOOTSTRAP_RESULT {\"pod\":\"podnew\",\"parity\":\"PASS\",\"ollama\":\"${FAKE_OLLAMA:-off}\"}"
  fi
  echo "UOR_POD_BOOTSTRAP_RC=${FAKE_BOOT_RC:-0}";;
esac
exit 0
S
cat > "$W/bin/gh" <<'S'
#!/usr/bin/env bash
echo "gh $*" >> "$FAKE/calls"; [ "${FAKE_ALLOW_CREATE:-0}" = 1 ] && exit 0; echo "REAL GH CALLED IN DRY RUN" >&2; exit 9
S
# fake flock for the bootstrap build-lock tests only (macOS ships none; they
# need its exit status): a descriptor listed in FAKE_FLOCK_HELD is held by
# "another pod". Kept off PATH elsewhere so uor-pod's perl lock path is tested.
mkdir -p "$W/fbin"
cat > "$W/fbin/flock" <<'S'
#!/usr/bin/env bash
fd=${!#}
case " ${FAKE_FLOCK_HELD:-} " in *" $fd "*) exit 1;; esac
exit 0
S
chmod +x "$W/bin/"* "$W/fbin/flock"

pass=0 fail=0
ok()  { pass=$((pass + 1)); echo "ok   $1"; }
bad() { fail=$((fail + 1)); echo "FAIL $1"; sed 's/^/     | /' "$W/out"; }
expect() {  # expect NAME RC PATTERN -- uor-pod args...
  local name=$1 want=$2 pat=$3 rc=0; shift 4
  "$TOOL" "$@" > "$W/out" 2>&1 || rc=$?
  if [ "$rc" = "$want" ] && grep -qE -e "$pat" "$W/out"; then ok "$name"; else echo "     rc=$rc want=$want pattern=$pat"; bad "$name"; fi
}
has() { if grep -qE -e "$2" "$W/out"; then ok "$1"; else bad "$1"; fi; }
hasnt() { if grep -qE -e "$2" "$W/out"; then bad "$1"; else ok "$1"; fi; }
C1=(--lab claude --session c1)
D1=(--lab deepseek --session d1)
D2=(--lab deepseek --session d2)
X1=(--lab codex --session x1)

expect "no arguments print the rules" 0 "Never fall back to the laptop CPU" --
expect "lease without a session refused" 1 "--session NAME is required" -- lease poda --lab claude --gpus 0 --purpose x --hours 1
expect "bad lab refused" 1 "lab must match" -- lease poda --lab Claude! --session c1 --gpus 0 --purpose x --hours 1
expect "claude/c1 leases A gpu 0" 0 "Leased poda gpus 0 to claude/c1" -- lease poda "${C1[@]}" --gpus 0 --purpose "test A" --hours 2
if [ -f "$UOR_POD_STATE/leases/poda/claude-c1.json" ] && [ "$(jq -r .session "$UOR_POD_STATE/leases/poda/claude-c1.json")" = c1 ]; then
  ok "lease file is per lab+session"; else bad "lease file is per lab+session"; fi
expect "deepseek/d1 leases A gpu 1 (two labs share a pod)" 0 "Leased poda gpus 1 to deepseek/d1" -- lease poda "${D1[@]}" --gpus 1 --purpose bf16 --hours 1
expect "deepseek/d2 cannot take d1's GPU (same lab, other session)" 1 "another session's live lease" -- lease poda "${D2[@]}" --gpus 1 --purpose x --hours 1
has "refusal prints spin-up guidance" "spin up your own pod: uor-pod up"
expect "codex cannot take claude's idle GPU" 1 "An idle leased GPU is not free" -- lease poda "${X1[@]}" --gpus 0 --purpose x --hours 1
expect "lease out-of-range refused" 1 "out of range" -- lease podb "${X1[@]}" --gpus 3 --purpose x --hours 1
expect "renew own lease" 0 "Renewed claude/c1 on poda" -- renew poda "${C1[@]}" --hours 3
expect "renew without lease refused" 1 "holds no lease" -- renew poda "${X1[@]}"
expect "run on own leased gpu (dry)" 0 "DRY-RUN: on poda" -- run poda "${C1[@]}" --gpu 0 -- echo hello
expect "run on another session's gpu refused" 1 "are not in claude/c1's lease" -- run poda "${C1[@]}" --gpu 1 -- echo hi
expect "status shows leased-but-idle as not free" 0 "gpu 1 0% 1 MiB  leased by deepseek/d1  leased but idle .*not free" -- status
has "status names free GPUs (B is unleased)" "podb gpus 0: free -> uor-pod lease podb"
hasnt "status: no FREE for leased A" "poda gpus .*: free"
has "status flags stopped pods as cleanup" "podd .*CLEANUP: stopped"
hasnt "status hides pod env" "must-not-print"
# an expired lease on C (busy) and on B (idle)
mkdir -p "$UOR_POD_STATE/leases/podb" "$UOR_POD_STATE/leases/podc"
echo '{"lab":"codex","session":"old","id":"codex-old","pod":"podb","gpus":[0],"purpose":"x","card":"","started":"2026-01-01T00:00:00Z","expires":"2026-01-01T01:00:00Z","renewed":"2026-01-01T00:00:00Z","hours":1}' > "$UOR_POD_STATE/leases/podb/codex-old.json"
sed 's/podb/podc/' "$UOR_POD_STATE/leases/podb/codex-old.json" > "$UOR_POD_STATE/leases/podc/codex-old.json"
expect "expired lease with a live job is not taken over" 1 "EXPIRED lease but still running a job" -- lease podc "${D2[@]}" --gpus 0 --purpose x --hours 1
expect "expired lease on an idle GPU is taken over" 0 "Leased podb gpus 0 to deepseek/d2" -- lease podb "${D2[@]}" --gpus 0 --purpose bf16 --hours 1
if [ ! -f "$UOR_POD_STATE/leases/podb/codex-old.json" ]; then ok "expired lease file removed on takeover"; else bad "expired lease file removed on takeover"; fi
"$TOOL" log -n 5 > "$W/out" 2>&1; has "takeover is in the ledger with the session" "takeover deepseek podb .*\"session\":\"d2\""
expect "release deepseek/d2 on B" 0 "Released deepseek/d2" -- release podb "${D2[@]}"
rm -f "$UOR_POD_STATE/leases/podc/codex-old.json"
expect "forbidden A100 refused" 1 "not used for this trainer" -- up "${X1[@]}" --gpu a100 --purpose x --hours 1
expect "up without session refused" 1 "--session NAME is required" -- up --lab codex --purpose x --hours 1
expect "up refuses when a free GPU of that type exists" 3 "Free GPUs already exist: pod podb" -- up "${X1[@]}" --gpu 4090 --purpose x --hours 1 --count 1
# up must distinguish unleased from proven idle, just as status does.
cp "$FAKE/probe-1002" "$FAKE/probe-1002.idle"
for condition in util app lock unreachable empty unknown; do
  case $condition in
    util) printf 'gpu=0, NVIDIA GeForce RTX 4090, 99, 16039, GPU-B\nprobe_ok=1\n' > "$FAKE/probe-1002";;
    app) printf 'gpu=0, NVIDIA GeForce RTX 4090, 0, 16039, GPU-B\napp=GPU-B\nprobe_ok=1\n' > "$FAKE/probe-1002";;
    lock) printf 'gpu=0, NVIDIA GeForce RTX 4090, 0, 1, GPU-B\nbusy_lock=/root/gpu0.lock\nprobe_ok=1\n' > "$FAKE/probe-1002";;
    unreachable) rm "$FAKE/probe-1002";;
    unknown) printf 'gpu=0, NVIDIA GeForce RTX 4090, N/A, 1, GPU-B\nprobe_ok=1\n' > "$FAKE/probe-1002";;
    empty) printf 'probe_ok=1\n' > "$FAKE/probe-1002";;
  esac
  expect "up does not offer $condition GPU as free" 0 "Creating 1 x 4090" -- up "${X1[@]}" --gpu 4090 --purpose x --hours 1 --count 1
  hasnt "up $condition has no false free claim" "Free GPUs already exist"
done
mv "$FAKE/probe-1002.idle" "$FAKE/probe-1002"
expect "up still offers a probed idle GPU" 3 "Free GPUs already exist: pod podb" -- up "${X1[@]}" --gpu 4090 --purpose x --hours 1 --count 1
expect "default caps are 4 pods / \$8/h" 0 "caps: <= 4 running pods, <= \\\$8.00/h" -- status
UOR_POD_MAX_PODS=3 expect "cap reached names the free GPUs to lease" 4 "cap reached \\(3 of 3 pods\\) — free GPUs exist" -- up "${X1[@]}" --purpose x --hours 1 --count 2
has "cap message lists podb gpu 0" "podb gpus 0: free"
UOR_POD_MAX_PODS=3 UOR_POD_MAX_RATE=99 expect "up --test pods bypass the pod cap" 0 "--name uor-test-codex-" -- up "${X1[@]}" --purpose x --hours 1 --count 2 --test --gpu 4090
export UOR_POD_MAX_PODS=9 UOR_POD_MAX_RATE=99
expect "up: no 5090 in EUR-NO-1 -> EU-RO-1, never 4090" 0 "Creating 2 x 5090 .* in EU-RO-1 volume dryrunvolume \\(non-canonical\\) for codex/x1" -- up "${X1[@]}" --purpose x --hours 1 --count 2
expect "up: creates uor-shared-EU-RO-1 lazily" 0 "network-volume create --name uor-shared-EU-RO-1 --size 100 --data-center-id EU-RO-1" -- up "${X1[@]}" --purpose x --hours 1 --count 2
expect "up --gpu 4090 explicit -> EUR-NO-1 canonical" 0 "Creating 2 x 4090 .* in EUR-NO-1 for codex/x1" -- up "${X1[@]}" --gpu 4090 --purpose x --hours 1 --count 2
# ---- portability (up): config file, environment over config, UOR_POD_TEMPLATE=none, a bad --hf-store
cat > "$W/user-config" <<'C'
: "${UOR_POD_VOLUME_ID:=testvol123}"
: "${UOR_POD_CANONICAL_DC:=EUR-NO-1}"
C
UOR_POD_CONFIG=$W/user-config expect "config file sets the canonical volume" 0 "--network-volume-id testvol123" -- up "${X1[@]}" --gpu 4090 --purpose x --hours 1 --count 2
UOR_POD_CONFIG=$W/user-config UOR_POD_VOLUME_ID=envvol456 expect "environment beats the config file" 0 "--network-volume-id envvol456" -- up "${X1[@]}" --gpu 4090 --purpose x --hours 1 --count 2
UOR_POD_TEMPLATE=none expect "UOR_POD_TEMPLATE=none creates from the public image" 0 "runpodctl pod create --image runpod/pytorch:1.0.2-cu1281-torch280-ubuntu2404 --name" -- up "${X1[@]}" --gpu 4090 --purpose x --hours 1 --count 2
hasnt "UOR_POD_TEMPLATE=none passes no --template-id" "--template-id"
UOR_POD_TEMPLATE= expect "empty UOR_POD_TEMPLATE also creates from the public image" 0 "runpodctl pod create --image runpod/pytorch:" -- up "${X1[@]}" --gpu 4090 --purpose x --hours 1 --count 2
hasnt "empty UOR_POD_TEMPLATE passes no --template-id" "--template-id"
UOR_POD_CONFIG=$W/user-config expect "a loaded config file is reported" 0 "uor-pod: config $W/user-config loaded" -- status
UOR_POD_CANONICAL_DC=US-CO-1 expect "canonical DC outside UOR_POD_VOLUME_DCS warns" 0 "warning: canonical DC US-CO-1 is not in UOR_POD_VOLUME_DCS" -- status
UOR_HF_STORE='a;b' expect "up refuses a bad --hf-store before any API call" 1 "the bootstrap rejects the arguments up would pass .*--hf-store may contain only" -- up "${X1[@]}" --gpu 4090 --purpose x --hours 1 --count 2
UOR_POD_VOLUME_DCS="EUR-NO-1 EUR-IS-1" expect "up --gpu 5090: no 5090 in volume DCs refuses (no silent fallback)" 1 "no 5090 stock in EUR-NO-1 EUR-IS-1.*--wait" -- up "${X1[@]}" --gpu 5090 --purpose x --hours 1 --count 2
has "refusal prints the stock table" "4090 \\\$0.74: EUR-NO-1\\*=Low"
UOR_POD_VOLUME_DCS="EUR-NO-1 EUR-IS-1" expect "up --wait retries every minute" 0 "would retry every 1 min" -- up "${X1[@]}" --gpu 5090 --purpose x --hours 1 --count 2 --wait --wait-hours 1
UOR_POD_VOLUME_DCS="EUR-NO-1 EUR-IS-1" expect "up --allow-off-volume places 5090 in EU-RO-1 without volume" 0 "Creating 2 x 5090 .* in EU-RO-1 OFF-VOLUME" -- up "${X1[@]}" --purpose x --hours 1 --count 2 --allow-off-volume
UOR_POD_MAX_RATE=4 expect "up over rate cap refused" 1 "exceeds \\\$4/h" -- up "${X1[@]}" --purpose x --hours 1 --count 3
expect "up (dry) names the next datacenter to try on a stock error" 0 "if EU-RO-1 reports no instances available, try next" -- up "${X1[@]}" --purpose x --hours 1 --count 2
# GPU ladder (owner 2026-10-07): without --gpu, 5090 -> 4090 -> RTX PRO 6000.
cp "$FAKE/gpus.json" "$FAKE/gpus.json.orig"
cat > "$FAKE/gpus.json" <<'J'
[
 {"gpuId":"NVIDIA GeForce RTX 5090","securePricePerHr":0.99,"dataCenterAvailability":[{"dataCenterId":"EUR-NO-1","stockStatus":"none"},{"dataCenterId":"EU-RO-1","stockStatus":"none"}]},
 {"gpuId":"NVIDIA GeForce RTX 4090","securePricePerHr":0.74,"dataCenterAvailability":[{"dataCenterId":"EUR-NO-1","stockStatus":"Low"}]},
 {"gpuId":"NVIDIA RTX PRO 6000 Blackwell Server Edition","securePricePerHr":2.09,"dataCenterAvailability":[{"dataCenterId":"EU-RO-1","stockStatus":"Low"}]}
]
J
expect "ladder: no 5090 anywhere -> 4090" 0 "Creating 2 x 4090 \\(ladder\\) .* in EUR-NO-1 for codex/x1" -- up "${X1[@]}" --purpose x --hours 1 --count 2
expect "explicit --gpu 5090 with no stock still refuses" 1 "no 5090 stock in .*--wait" -- up "${X1[@]}" --gpu 5090 --purpose x --hours 1 --count 2
jq 'map(if .gpuId == "NVIDIA GeForce RTX 4090" then .dataCenterAvailability = [{"dataCenterId":"EUR-NO-1","stockStatus":"none"}] else . end)' "$FAKE/gpus.json" > "$FAKE/gpus.json.new" && mv "$FAKE/gpus.json.new" "$FAKE/gpus.json"
expect "ladder: no 5090 or 4090 -> RTX PRO 6000" 0 "Creating 2 x pro6000 \\(ladder\\) .* in EU-RO-1 volume" -- up "${X1[@]}" --purpose x --hours 1 --count 2
jq 'map(.dataCenterAvailability |= map(.stockStatus = "none"))' "$FAKE/gpus.json" > "$FAKE/gpus.json.new" && mv "$FAKE/gpus.json.new" "$FAKE/gpus.json"
expect "ladder: no stock on any rung refuses, naming the ladder" 1 "no ladder 5090 -> 4090 -> pro6000 stock" -- up "${X1[@]}" --purpose x --hours 1 --count 2
mv "$FAKE/gpus.json.orig" "$FAKE/gpus.json"
# helper files are checked before anything is created: a copy of the tool with
# one helper missing, empty or broken must stop before any create/volume call
PF=$W/pf; mkdir -p "$PF"; cp "$HERE/../uor-pod" "$HERE/../uor-pod-bootstrap.sh" "$HERE/../hot-set.txt" "$PF/"
cp "$HERE/../uor-reaper.sh" "$PF/"
pf_up() {  # NAME PATTERN [extra up args]
  local name=$1 pat=$2 rc=0 n0; shift 2
  n0=$(wc -l < "$FAKE/calls")
  "$PF/uor-pod" up "${X1[@]}" --purpose x --hours 1 --count 2 "$@" > "$W/out" 2>&1 || rc=$?
  if [ "$rc" = 1 ] && grep -qE -e "$pat" "$W/out" && ! grep -qE 'Creating|pod create|network-volume create' "$W/out" &&
     ! tail -n "+$((n0 + 1))" "$FAKE/calls" | grep -qE 'pod create|network-volume|gpu list|pod list'; then ok "$name"
  else echo "     rc=$rc pattern=$pat"; bad "$name"; fi
}
pf_up "up refuses before creating when uor-store is missing" "helper file check failed before creating anything:.*uor-store \\(missing\\)"
cp "$HERE/../uor-store" "$PF/"; rm "$PF/uor-reaper.sh"
pf_up "up refuses before creating when uor-reaper.sh is missing" "helper file check failed before creating anything:.*uor-reaper.sh \\(missing\\)"
cp "$HERE/../uor-reaper.sh" "$PF/"; : > "$PF/hot-set.txt"
pf_up "up refuses before creating when hot-set.txt is empty" "hot-set.txt \\(empty\\)"
cp "$HERE/../hot-set.txt" "$PF/"; printf 'if then fi (\n' > "$PF/uor-pod-bootstrap.sh"
pf_up "up refuses before creating when the bootstrap has a syntax error" "uor-pod-bootstrap.sh \\(bash syntax error\\)"
if "$PF/uor-pod" up "${X1[@]}" --purpose x --hours 1 --count 2 --no-bootstrap > "$W/out" 2>&1; then
  has "--no-bootstrap needs only the seed helpers" "Creating 2 x 5090"
else bad "--no-bootstrap needs only the seed helpers"; fi
cp "$HERE/../uor-pod-bootstrap.sh" "$PF/"
expect "up with every helper present proceeds (dry)" 0 "Creating 2 x 5090" -- up "${X1[@]}" --purpose x --hours 1 --count 2
# ---- preflight of every bootstrap argument (incident 2026-10-05: 14 pods were
# created, billed and deleted because the bootstrap failed): a bad argument must
# stop `up` before any Runpod API call. noapi also proves no API call happened.
noapi() {  # NAME RC PATTERN [ENV=VALUE...] -- uor-pod args...
  local name=$1 want=$2 pat=$3 rc=0 n0 envs=(); shift 3
  while [ "$1" != -- ]; do envs+=("$1"); shift; done; shift
  n0=$(wc -l < "$FAKE/calls")
  env ${envs[@]+"${envs[@]}"} "$TOOL" "$@" > "$W/out" 2>&1 || rc=$?
  if [ "$rc" = "$want" ] && grep -qE -e "$pat" "$W/out" &&
     ! tail -n "+$((n0 + 1))" "$FAKE/calls" | grep -qE 'runpodctl'; then ok "$name"
  else echo "     rc=$rc want=$want pattern=$pat"; tail -n "+$((n0 + 1))" "$FAKE/calls" | sed 's/^/     calls: /'; bad "$name"; fi
}
UPX=(up "${X1[@]}" --purpose x --hours 1 --count 2)
noapi "unknown short SHA refused before any API call" 1 "--ref deadbee: unknown or ambiguous commit" -- "${UPX[@]}" --ref deadbee
noapi "40-char SHA absent from origin refused" 1 "unknown or ambiguous commit" -- "${UPX[@]}" --ref 0123456789abcdef0123456789abcdef01234567
noapi "commit only in the local checkout (never pushed) refused" 1 "is on no branch of .*push it first" -- "${UPX[@]}" --ref "${LOCAL_SHA:0:10}"
noapi "unknown branch refused" 1 "--ref 'no-such-branch' is not a branch or tag" -- "${UPX[@]}" --ref no-such-branch
noapi "shell metacharacters in --ref refused" 1 "is not a branch, tag or commit" -- "${UPX[@]}" --ref 'main;rm'
noapi "too-short SHA refused" 1 "7 to 40 hex characters" -- "${UPX[@]}" --ref abc12
expect "short SHA of a pushed (non-tip) commit expands to the full commit" 0 "--ref ${FEAT1_SHA:0:9} -> $FEAT1_SHA" -- "${UPX[@]}" --ref "${FEAT1_SHA:0:9}"
has "the expanded SHA proceeds to placement (dry)" "Creating 2 x 5090"
expect "upper-case short SHA is accepted" 0 "-> $FEAT1_SHA" -- "${UPX[@]}" --ref "$(printf '%s' "${FEAT1_SHA:0:12}" | tr 'a-f' 'A-F')"
expect "branch name resolves to its tip" 0 "--ref feature -> $FEAT_TIP" -- "${UPX[@]}" --ref feature
expect "default ref main resolves" 0 "--ref main -> $MAIN_SHA" -- "${UPX[@]}"
noapi "option of another subcommand refused" 1 "up does not take --gpus" -- "${UPX[@]}" --gpus 0
noapi "stray positional argument refused" 1 "up takes no positional arguments \\(got 'extra'" -- "${UPX[@]}" extra
noapi "--count 0 refused" 1 "--count must be 1..8" -- up "${X1[@]}" --purpose x --hours 1 --count 0
noapi "--count two refused" 1 "--count must be 1..8" -- up "${X1[@]}" --purpose x --hours 1 --count two
noapi "--hours 48 refused" 1 "--hours must be a number in \\(0, 24\\]" -- up "${X1[@]}" --purpose x --hours 48
noapi "unknown --gpu refused" 1 "unknown --gpu 'rtx9000'" -- "${UPX[@]}" --gpu rtx9000
noapi "--wait-hours out of range refused" 1 "--wait-hours must be in" -- "${UPX[@]}" --wait --wait-hours 99
# the bootstrap's own parser judges the exact vector: a bootstrap that no longer
# knows --with-ollama stops `up` on the laptop
sed 's/--with-ollama) OLLAMA=1; shift;;//' "$HERE/../uor-pod-bootstrap.sh" > "$PF/uor-pod-bootstrap.sh"
n0=$(wc -l < "$FAKE/calls"); rc=0
"$PF/uor-pod" "${UPX[@]}" --with-ollama > "$W/out" 2>&1 || rc=$?
if [ "$rc" = 1 ] && grep -q "the bootstrap rejects the arguments up would pass (--sha $MAIN_SHA --pod preflightpod --with-ollama): unknown argument --with-ollama" "$W/out" &&
   ! tail -n "+$((n0 + 1))" "$FAKE/calls" | grep -q runpodctl; then ok "bootstrap rejecting an argument stops up before any API call"
else bad "bootstrap rejecting an argument stops up before any API call"; fi
cp "$HERE/../uor-pod-bootstrap.sh" "$PF/"
BS=$HERE/../uor-pod-bootstrap.sh
chk() {  # NAME RC PATTERN bootstrap-args...
  local name=$1 want=$2 pat=$3 rc=0; shift 3
  bash "$BS" --check-args "$@" > "$W/out" 2>&1 || rc=$?
  if [ "$rc" = "$want" ] && grep -qE -e "$pat" "$W/out"; then ok "$name"; else echo "     rc=$rc"; bad "$name"; fi
}
chk "bootstrap --check-args accepts a full SHA" 0 "bootstrap arguments OK" --sha "$MAIN_SHA" --pod podx --with-ollama --off-volume
chk "bootstrap --check-args accepts --non-canonical" 0 "bootstrap arguments OK" --sha "$MAIN_SHA" --pod podx --non-canonical
chk "bootstrap --check-args rejects a short SHA" 2 "--sha must be a full 40-character commit \\(got 7" --sha "${MAIN_SHA:0:7}" --pod podx
chk "bootstrap --check-args rejects upper-case hex" 2 "lower-case hex" --sha "$(printf '%s' "$MAIN_SHA" | tr 'a-f' 'A-F')"
chk "bootstrap --check-args rejects an unknown argument" 2 "unknown argument --bogus" --sha "$MAIN_SHA" --bogus
chk "bootstrap --check-args rejects --sha without a value" 2 "--sha needs a value" --sha
# ---- circuit breaker: 2 up-failed of one lab+session within 60 min refuse the next up
fail_event() {  # SESSION AGE-SECONDS
  jq -cn --arg s "$1" --arg ts "$(jq -rn --argjson a "$2" 'now - $a | floor | todate')" \
    '{ts: $ts, event: "up-failed", lab: "codex", session: $s, pod: "podold", by: "t", reason: "bootstrap failed",
      log: "/workspace/uor-r4/pods/bootstrap-podold-x.log", local_log: "/state/logs/bootstrap-podold-x.log",
      last: "tar: Error is not recoverable: exiting now"}' >> "$UOR_POD_STATE/ledger.jsonl"
}
BRK=(up --lab codex --session brk --purpose x --hours 1 --count 2)
fail_event brk 600
expect "one recent failure does not trip the breaker" 0 "Creating 2 x 5090" -- "${BRK[@]}"
fail_event brk 300
noapi "two failures within 60 min trip the breaker (no API call)" 5 "up refused \\(circuit breaker\\): codex/brk had 2 failed 'up' attempts in the last 60 min" -- "${BRK[@]}"
has "breaker shows the failure reasons" "pod podold: bootstrap failed"
has "breaker shows the last bootstrap output" "last output: tar: Error is not recoverable"
has "breaker shows the log paths" "log: /state/logs/bootstrap-podold-x.log"
has "breaker names --force-retry" "retry with --force-retry"
if tail -1 "$UOR_POD_STATE/ledger.jsonl" | jq -e '.event == "up-refused" and .reason == "circuit-breaker" and .failures == 2 and .session == "brk"' >/dev/null; then
  ok "breaker refusal recorded in the ledger"; else bad "breaker refusal recorded in the ledger"; fi
expect "--force-retry overrides the breaker" 0 "Creating 2 x 5090" -- "${BRK[@]}" --force-retry
has "--force-retry warns" "retrying because of --force-retry"
if grep -q '"event":"up-force-retry".*"session":"brk"' "$UOR_POD_STATE/ledger.jsonl"; then ok "--force-retry recorded in the ledger"; else bad "--force-retry recorded in the ledger"; fi
expect "another session of the same lab is not blocked" 0 "Creating 2 x 5090" -- up --lab codex --session brk2 --purpose x --hours 1 --count 2
fail_event old1 7200; fail_event old1 5400
expect "failures older than 60 min do not count" 0 "Creating 2 x 5090" -- up --lab codex --session old1 --purpose x --hours 1 --count 2
unset UOR_POD_MAX_PODS UOR_POD_MAX_RATE
expect "register: idle pod gets no lease" 0 "podb: no busy GPU; nothing leased" -- register podb "${X1[@]}" --purpose backfill
expect "register: busy GPU leased for 1 h" 0 "Leased podc gpus 0 to codex/x1" -- register podc "${X1[@]}" --purpose backfill
expect "register a stopped pod with --keep" 0 "no lease written \\(marked KEEP" -- register podd "${D1[@]}" --purpose "bf16 data" --keep "unique data to copy"
expect "down refuses a KEEP pod" 1 "marked KEEP" -- down podd "${D1[@]}"
expect "down of a non-canonical pod needs --confirm-archived" 1 "non-canonical volume volro" -- down podc "${X1[@]}"
expect "down non-canonical after archiving (dry)" 0 "DRY-RUN: runpodctl pod delete podc" -- down podc "${X1[@]}" --confirm-archived
expect "down refused while another session leases" 1 "another session holds a live lease" -- down poda "${C1[@]}"
expect "release deepseek/d1" 0 "Released deepseek/d1" -- release poda "${D1[@]}"
expect "down by the last lease holder (dry)" 0 "DRY-RUN: runpodctl pod delete poda" -- down poda "${C1[@]}"
expect "release" 0 "Released claude/c1's lease on poda" -- release poda "${C1[@]}"
expect "release codex on C" 0 "Released codex/x1" -- release podc "${X1[@]}"
# B's idle clock is relative to the suite's start (NOW); restart it at 30 min here so the
# exact "idle 30 min" text does not depend on how long the suite took to get this far.
sed -i.bak "s/^idle_since=.*/idle_since=$(( $(date -u +%s) - 1800 ))/" "$FAKE/probe-1002" && rm -f "$FAKE/probe-1002.bak"
expect "reap: deletes idle unleased B, keeps busy C" 0 "podb: no live lease, idle 30 min — deleting" -- reap
has "reap keeps busy C" "podc: no live lease, GPUs busy — kept"
has "reap starts A's idle clock" "poda: no live lease, idle 0 min"
# A lease hand-written with Python's datetime.isoformat() (fractional seconds
# and an explicit +00:00 offset) must not abort the tool, and must be read as
# LIVE: a timestamp the tool cannot parse must never look expired and hand
# another session's working GPU away. Labs still write leases this way.
FRAC=$(jq -rn 'now + 7200 | floor | todate | sub("Z$"; ".245421+00:00")')
printf '{"lab":"codex","session":"hand","id":"codex-hand","pod":"poda","gpus":[0],"purpose":"hand-written","card":"","started":"%s","expires":"%s","renewed":"%s","hours":2}' "$FRAC" "$FRAC" "$FRAC" > "$UOR_POD_STATE/leases/poda/codex-hand.json"
expect "hand-written fractional timestamp does not abort the tool" 0 "Pods \\(caps" -- status
has "hand-written lease is listed" "codex/hand"
expect "hand-written live lease still holds its GPU" 1 "another session's live lease" -- lease poda "${D2[@]}" --gpus 0 --purpose x --hours 1
rm -f "$UOR_POD_STATE/leases/poda/codex-hand.json"
# Timestamps in every RFC 3339 / ISO 8601 form with an offset are parsed, not
# just tolerated: an EXPIRED fractional/offset lease is expired (takeover works),
# offsets are applied, and only an unparseable one falls back to LIVE.
hand_lease() {  # POD ID EXPIRES-JSON [GPUS-JSON]
  printf '{"lab":"%s","session":"%s","id":"%s","pod":"%s","gpus":%s,"purpose":"hand","card":"","started":%s,"expires":%s,"renewed":%s,"hours":2}' \
    "${2%%-*}" "${2#*-}" "$2" "$1" "${4:-[0]}" "$3" "$3" "$3" > "$UOR_POD_STATE/leases/$1/$2.json"
}
ts_at() { jq -rn --argjson d "$1" --arg z "$2" --argjson o "${3:-0}" '(now + $d | floor) + $o | todate | sub("Z$"; $z)'; }
mkdir -p "$UOR_POD_STATE/leases/podb"
hand_lease podb codex-frac "\"$(ts_at -3600 .875296+00:00)\""
expect "expired fractional +00:00 lease is EXPIRED (not stuck live)" 0 "lease codex/frac gpus=0 until .* EXPIRED" -- status
hasnt "status does not warn when no lease is malformed" "MALFORMED LEASE FILE"
expect "expired fractional lease can be taken over" 0 "Leased podb gpus 0 to deepseek/d2" -- lease podb "${D2[@]}" --gpus 0 --purpose ts --hours 1
expect "release after fractional takeover" 0 "Released deepseek/d2" -- release podb "${D2[@]}"
# now+1h written in -05:00 local time: ignoring the offset would read it as expired
hand_lease podb codex-west "\"$(ts_at 3600 -05:00 -18000)\""
expect "-05:00 offset applied: live lease holds its GPU" 1 "another session's live lease" -- lease podb "${D2[@]}" --gpus 0 --purpose x --hours 1
rm -f "$UOR_POD_STATE/leases/podb/codex-west.json"
# now-1h written in +05:30 local time: ignoring the offset would read it as live
hand_lease podb codex-east "\"$(ts_at -3600 .5+05:30 19800)\""
expect "+05:30 offset applied: expired lease is taken over" 0 "Leased podb gpus 0 to deepseek/d2" -- lease podb "${D2[@]}" --gpus 0 --purpose ts --hours 1
expect "release after offset takeover" 0 "Released deepseek/d2" -- release podb "${D2[@]}"
hand_lease podb codex-epoch "$(jq -n 'now + 3600 | floor')"
expect "epoch-number expires is understood (live)" 1 "another session's live lease" -- lease podb "${D2[@]}" --gpus 0 --purpose x --hours 1
rm -f "$UOR_POD_STATE/leases/podb/codex-epoch.json"
# unparseable expires (garbage, or a past time with no offset) -> LIVE, loudly
hand_lease podb codex-bad '"tomorrow-ish"'
hand_lease podb codex-nozone '"2020-01-01T00:00:00"'
expect "unparseable expires is treated as LIVE (fail safe)" 1 "another session's live lease" -- lease podb "${D2[@]}" --gpus 0 --purpose x --hours 1
expect "status still runs with malformed leases" 0 "Running: " -- status
has "status warns loudly about the malformed lease" "WARNING: MALFORMED LEASE FILE\\(S\\).*treated as LIVE"
has "status names the unparseable expires" "codex-bad.json: unparseable expires \"tomorrow-ish\""
has "status names the offset-less expires" "codex-nozone.json: unparseable expires \"2020-01-01T00:00:00\""
has "status marks the lease line" "codex/bad gpus=0 until tomorrow-ish \\[MALFORMED: treated as LIVE\\]"
has "status warns on stderr too" "uor-pod: warning: malformed lease"
hasnt "podb is not offered as free" "podb gpus 0: free"
rm -f "$UOR_POD_STATE/leases/podb/codex-bad.json" "$UOR_POD_STATE/leases/podb/codex-nozone.json"
echo 'not json {' > "$UOR_POD_STATE/leases/podb/codex-broken.json"
expect "a non-JSON lease file is LIVE on every GPU (fail safe)" 1 "another session's live lease" -- lease podb "${D2[@]}" --gpus 0 --purpose x --hours 1
expect "status survives a non-JSON lease file" 0 "codex-broken.json: not a JSON lease object" -- status
rm -f "$UOR_POD_STATE/leases/podb/codex-broken.json"
# what uor-pod writes is canonical, and renew rewrites a hand-written lease canonically
mkdir -p "$UOR_POD_STATE/leases/podb"
hand_lease podb claude-c9 "\"$(ts_at 1800 .123456+00:00)\""
expect "renew of a hand-written lease" 0 "Renewed claude/c9 on podb" -- renew podb --lab claude --session c9 --hours 1
CANON='^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$'
if jq -e --arg re "$CANON" '[.started, .expires, .renewed] | all(test($re))' "$UOR_POD_STATE/leases/podb/claude-c9.json" >/dev/null; then
  ok "renew rewrites started/expires/renewed in the canonical ...Z form"; else bad "renew rewrites started/expires/renewed in the canonical ...Z form"; fi
expect "release c9" 0 "Released claude/c9" -- release podb --lab claude --session c9
# the pod-side reaper reads the mirrored leases with the same rules
RL=$W/reaper-leases; mkdir -p "$RL"
reaper_live() { UOR_REAPER_LEASES=$RL bash "$HERE/../uor-reaper.sh" --live-lease 2>/dev/null; }
echo "{\"expires\": \"$(ts_at -60 .875296+00:00)\"}" > "$RL/a.json"
if reaper_live; then bad "reaper: expired fractional lease is not live"; else ok "reaper: expired fractional lease is not live"; fi
echo "{\"expires\": \"$(ts_at 600 .875296+00:00)\"}" > "$RL/a.json"
if reaper_live; then ok "reaper: future fractional lease is live"; else bad "reaper: future fractional lease is live"; fi
echo "{\"expires\": \"$(ts_at 3600 -05:00 -18000)\"}" > "$RL/a.json"
if reaper_live; then ok "reaper: -05:00 offset applied (live)"; else bad "reaper: -05:00 offset applied (live)"; fi
echo '{"expires": "2020-01-01T00:00:00"}' > "$RL/a.json"
if reaper_live; then ok "reaper: unparseable lease is live (never delete on doubt)"; else bad "reaper: unparseable lease is live (never delete on doubt)"; fi
echo '{"expires": "2020-01-01T00:00:00Z"}' > "$RL/a.json"
if reaper_live; then bad "reaper: expired canonical lease is not live"; else ok "reaper: expired canonical lease is not live"; fi
expect "prune-stopped lists stopped pods" 0 "pode .*stopped .*\\(72 h\\)" -- prune-stopped
has "prune-stopped without --yes deletes nothing" "Listed only"
expect "prune-stopped --yes deletes old unprotected pods (dry)" 0 "DRY-RUN: runpodctl pod delete pode" -- prune-stopped --older-than 24h --yes
has "prune-stopped skips KEEP pods" "skipped: marked KEEP"
mkdir -p "$UOR_POD_STATE/leases/podz"; cp "$UOR_POD_STATE/leases/podb/../poda/"*.json "$UOR_POD_STATE/leases/podz/" 2>/dev/null || echo '{}' > "$UOR_POD_STATE/leases/podz/x-y.json"
touch -t 202601010000 "$UOR_POD_STATE/pods.json.tmp.123"
expect "status runs" 0 "Running: 3 pod" -- status
has "status warns about stray tmp files" "stray temporary files"
expect "log shows sessions" 0 "lease claude poda .*\"session\":\"c1\"" -- log -n 80
if grep -q "REAL" "$W/out" "$FAKE/calls" 2>/dev/null; then bad "no real mutation in dry run"; else ok "no real mutation in dry run"; fi

# ---- placement fallback: NOT a dry run, against the fake runpodctl only
# (FAKE_ALLOW_CREATE=1 lets the fake "create" podnew; nothing leaves this machine).
# Stock changes between the listing and `pod create`: an out-of-stock error in
# one datacenter moves on to the next in policy order.
export UOR_POD_STATE=$W/state2 UOR_POD_DRY_RUN=0 FAKE_ALLOW_CREATE=1 UOR_POD_MAX_PODS=9 UOR_POD_MAX_RATE=99 UOR_POD_SSH_WAIT=30
SHA40=0123456789abcdef0123456789abcdef01234567
cp "$FAKE/pods.json" "$FAKE/pods.orig.json"
echo '[]' > "$FAKE/pods.json"
cat > "$FAKE/gpus.json" <<'J'
[{"gpuId":"NVIDIA GeForce RTX 5090","securePricePerHr":0.99,"dataCenterAvailability":[{"dataCenterId":"EUR-NO-1","stockStatus":"Low"},{"dataCenterId":"EU-RO-1","stockStatus":"Low"},{"dataCenterId":"EUR-IS-1","stockStatus":"Low"}]}]
J
UP=(up "${X1[@]}" --gpu 5090 --purpose fallback --hours 1 --count 2 --no-bootstrap --ref "$SHA40")
creates() { grep -c "pod create .*--data-center-ids $1" "$FAKE/calls" || true; }
echo EUR-NO-1 > "$FAKE/nostock"; : > "$FAKE/calls"
expect "create out of stock in EUR-NO-1 -> next datacenter EU-RO-1" 0 "Pod podnew created" -- "${UP[@]}"
has "fallback names the stock error" "pod create in EUR-NO-1: no instances available any more"
has "fallback creates in EU-RO-1 on its volume" "Creating 2 x 5090 .* in EU-RO-1 volume volnew \\(non-canonical\\)"
if [ "$(creates EUR-NO-1)" = 1 ] && [ "$(creates EU-RO-1)" = 1 ] && [ "$(creates EUR-IS-1)" = 0 ]; then
  ok "one create per datacenter, in policy order, stopping at the first success"; else bad "one create per datacenter, in policy order, stopping at the first success"; fi
if [ "$(jq -r '.podnew.dc' "$UOR_POD_STATE/pods.json")" = EU-RO-1 ] && [ -f "$UOR_POD_STATE/leases/podnew/codex-x1.json" ]; then
  ok "the pod is recorded where it landed, under the creator's lease"; else bad "the pod is recorded where it landed, under the creator's lease"; fi
rm -rf "$UOR_POD_STATE"; echo '[]' > "$FAKE/pods.json"
printf 'EUR-NO-1\nEU-RO-1\nEUR-IS-1\n' > "$FAKE/nostock"; : > "$FAKE/calls"
expect "every datacenter out of stock at create -> reported as no stock" 1 "no 5090 stock in EUR-NO-1 EU-RO-1 EUR-IS-1.*--wait" -- "${UP[@]}"
has "the report lists the datacenters that refused" "pod create found no instances in: EUR-NO-1 EU-RO-1 EUR-IS-1"
if [ "$(creates EUR-IS-1)" = 1 ]; then ok "all three datacenters were tried"; else bad "all three datacenters were tried"; fi
: > "$FAKE/calls"
UOR_POD_WAIT_INTERVAL=1 expect "--wait retries the whole placement order after create-time stock errors" 1 "no 5090 stock after waiting" -- "${UP[@]}" --wait --wait-hours 0.0008
if [ "$(creates EUR-NO-1)" -ge 2 ] && [ "$(creates EUR-IS-1)" -ge 2 ]; then ok "--wait made repeated full passes"; else bad "--wait made repeated full passes"; fi
rm -f "$FAKE/nostock"; touch "$FAKE/create-error"; : > "$FAKE/calls"
expect "a non-stock create error stops at once" 1 "pod create failed in EUR-NO-1: Error: template" -- "${UP[@]}"
if [ "$(creates EU-RO-1)" = 0 ]; then ok "no fallback on a non-stock error"; else bad "no fallback on a non-stock error"; fi
rm -f "$FAKE/create-error"
# ---- "Low" stock is usually one free card per host: without --count, up falls
# back from 2 GPUs to 1; an explicit --count 2 insists
rm -rf "$UOR_POD_STATE"; echo '[]' > "$FAKE/pods.json"; echo 1 > "$FAKE/max-count"; : > "$FAKE/calls"
UP1=(up "${X1[@]}" --gpu 5090 --purpose fallback --hours 1 --no-bootstrap --ref "$SHA40")
expect "no host with 2 free and no --count -> falls back to 1 GPU" 0 "Pod podnew created" -- "${UP1[@]}"
has "the fallback is announced" "no 2 x 5090 on one host; falling back to 1 x 5090"
if grep -q -- "--gpu-count 1 " "$FAKE/calls"; then ok "the second attempt asks for one GPU"; else bad "the second attempt asks for one GPU"; fi
rm -rf "$UOR_POD_STATE"; echo '[]' > "$FAKE/pods.json"; : > "$FAKE/calls"
expect "an explicit --count 2 never falls back" 1 "no 5090 stock in EUR-NO-1 EU-RO-1 EUR-IS-1" -- "${UP[@]}"
if ! grep -q -- "--gpu-count 1 " "$FAKE/calls"; then ok "no 1-GPU create with an explicit --count"; else bad "no 1-GPU create with an explicit --count"; fi
rm -f "$FAKE/max-count"
# ---- a failed bootstrap (non-dry, fake pod): the last 30 lines are printed, the
# full log is kept locally and teed onto the pod volume, the ledger names both;
# then the circuit breaker stops the third attempt. A failed Ollama is not fatal.
BOOT=(up --lab claude --session boot --purpose boot --hours 1 --count 2 --ref "$MAIN_SHA")
fresh_pods() { echo '[]' > "$FAKE/pods.json"; : > "$FAKE/calls"; }
fresh_pods
FAKE_BOOT_RC=2 expect "bootstrap failure: up fails and deletes the pod" 1 "bootstrap failed: deleting pod podnew" -- "${BOOT[@]}"
has "the last 30 lines are printed with the log path" "---- last 30 lines of the bootstrap output \\(full log: $UOR_POD_STATE/logs/bootstrap-podnew-[0-9TZ]+\\.log; on the pod volume: /workspace/uor-r4/pods/bootstrap-podnew-[0-9TZ]+\\.log\\)"
has "the tail shows the failing line" "^  \\| tar: Error is not recoverable"
has "the tail shows line 40" "^  \\| bootstrap line 40$"
hasnt "the tail stops at 30 lines (line 11 is outside it)" "^  \\| bootstrap line 11$"
hasnt "the tail hides the exit-status marker" "^  \\| UOR_POD_BOOTSTRAP_RC"
EV=$(grep '"event":"up-failed"' "$UOR_POD_STATE/ledger.jsonl" | tail -1)
LLOG=$(printf '%s' "$EV" | jq -r '.local_log // empty')
if [ -n "$LLOG" ] && [ -f "$LLOG" ] && grep -q '^bootstrap line 1$' "$LLOG" && grep -q '^tar: Error is not recoverable' "$LLOG"; then
  ok "the full bootstrap log is kept on the laptop"; else bad "the full bootstrap log is kept on the laptop"; fi
if printf '%s' "$EV" | jq -e '(.log | test("^/workspace/uor-r4/pods/bootstrap-podnew-[0-9]{8}T[0-9]{6}Z\\.log$")) and .log_on_volume == true
     and .last == "tar: Error is not recoverable: exiting now" and .reason == "bootstrap failed"' >/dev/null; then
  ok "up-failed ledger event names the logs and the last output line"; else echo "     $EV"; bad "up-failed ledger event names the logs and the last output line"; fi
if grep -q "uor-pod-bootstrap.sh --sha $MAIN_SHA --pod podnew 2>&1; echo \"UOR_POD_BOOTSTRAP_RC=\$?\"; } | tee /workspace/uor-r4/pods/bootstrap-podnew-" "$FAKE/bootcmd"; then
  ok "the pod tees the bootstrap output onto its volume"; else sed 's/^/     /' "$FAKE/bootcmd"; bad "the pod tees the bootstrap output onto its volume"; fi
if grep -q 'runpodctl pod delete podnew' "$FAKE/calls"; then ok "the failed pod is deleted"; else bad "the failed pod is deleted"; fi
if grep '"event":"bootstrap"' "$UOR_POD_STATE/ledger.jsonl" | tail -1 | jq -e '.rc == 2 and (.local_log | length > 0)' >/dev/null; then
  ok "bootstrap ledger event carries rc and log"; else bad "bootstrap ledger event carries rc and log"; fi
fresh_pods
FAKE_BOOT_RC=2 expect "second bootstrap failure" 1 "bootstrap failed: deleting pod podnew" -- "${BOOT[@]}"
fresh_pods
FAKE_BOOT_RC=2 expect "third up of the session is refused by the breaker" 5 "circuit breaker.*claude/boot had 2 failed" -- "${BOOT[@]}"
has "the refusal shows the saved log" "log: $UOR_POD_STATE/logs/bootstrap-podnew-"
has "the refusal shows the 404/tar line" "last output: tar: Error is not recoverable"
if ! grep -q 'runpodctl' "$FAKE/calls"; then ok "the breaker made no API call and created no pod"; else bad "the breaker made no API call and created no pod"; fi
fresh_pods
FAKE_OLLAMA=FAILED expect "--force-retry with a failed Ollama: pod is kept and ready" 0 "Ready: ssh" -- "${BOOT[@]}" --with-ollama --force-retry
has "a failed Ollama is a warning, not a failure" "Ollama on podnew: FAILED — the pod is kept"
if ! grep -q 'runpodctl pod delete' "$FAKE/calls"; then ok "a failed Ollama does not delete the pod"; else bad "a failed Ollama does not delete the pod"; fi
if grep '"event":"bootstrap"' "$UOR_POD_STATE/ledger.jsonl" | tail -1 | jq -e '.rc == 0 and .ollama == "FAILED"' >/dev/null; then
  ok "the Ollama failure is recorded in the bootstrap ledger event"; else bad "the Ollama failure is recorded in the bootstrap ledger event"; fi
if grep -q -- "--with-ollama | tee" "$FAKE/bootcmd" || grep -q -- "--with-ollama 2>&1" "$FAKE/bootcmd"; then ok "--with-ollama reaches the bootstrap"; else bad "--with-ollama reaches the bootstrap"; fi
# ---- the Hugging Face token: `up` copies ~/.cache/huggingface/token to the pod
# over stdin; the value never appears in the output or on a command line
HH=$W/hfhome; mkdir -p "$HH/.cache/huggingface"; printf 'hf_FAKETOKEN123' > "$HH/.cache/huggingface/token"
rm -rf "$UOR_POD_STATE"; fresh_pods
# Token setup belongs to the bootstrap-enabled path; the bootstrap itself is
# the fake above, and the ref resolves only against the local stand-in origin.
HOME=$HH expect "up with an HF token on the laptop" 0 "hf token copied" -- up --lab claude --session hftok --purpose hf --hours 1 --count 2 --ref "$MAIN_SHA"
if grep -q 'cat > /root/.cache/huggingface/token' "$FAKE/calls"; then ok "the token is copied to the pod over ssh"; else bad "the token is copied to the pod over ssh"; fi
if ! grep -q 'HF_HOME' "$FAKE/calls"; then ok "the token is never written to the image's HF_HOME (a shared volume on Runpod)"; else bad "the token is never written to the image's HF_HOME (a shared volume on Runpod)"; fi
# the bootstrap points hf at that private copy (the image's HF_HOME is elsewhere)
if grep -q '^if \[ -f /root/.cache/huggingface/token \]; then export HF_TOKEN_PATH=/root/.cache/huggingface/token; fi$' "$HERE/../uor-pod-bootstrap.sh"; then
  ok "the bootstrap's store calls use HF_TOKEN_PATH=/root/.cache/huggingface/token"; else bad "the bootstrap's store calls use HF_TOKEN_PATH=/root/.cache/huggingface/token"; fi
if ! grep -q 'hf_FAKETOKEN123' "$W/out" "$FAKE/calls"; then ok "the token value is in neither the output nor any command line"; else bad "the token value is in neither the output nor any command line"; fi
# ---- any-region placement: no ladder GPU in the volume datacenters, stock in
# two allowlisted extra datacenters -> the one with more stock, non-canonical;
# an explicit UOR_POD_VOLUME_DCS keeps placement to that list only
cat > "$FAKE/gpus.json" <<'J'
[{"gpuId":"NVIDIA GeForce RTX 5090","securePricePerHr":0.99,"dataCenterAvailability":[{"dataCenterId":"EUR-NO-1","stockStatus":"none"},{"dataCenterId":"US-TX-3","stockStatus":"Low"},{"dataCenterId":"CA-MTL-1","stockStatus":"High"},{"dataCenterId":"AP-JP-1","stockStatus":"High"}]},
 {"gpuId":"NVIDIA GeForce RTX 4090","securePricePerHr":0.74,"dataCenterAvailability":[{"dataCenterId":"EU-RO-1","stockStatus":"none"}]}]
J
ANY=(up "${X1[@]}" --purpose anyregion --hours 1 --count 2 --no-bootstrap --ref "$SHA40")
rm -rf "$UOR_POD_STATE"; fresh_pods
expect "no ladder stock in the volume datacenters -> placed in an extra datacenter" 0 "Pod podnew created" -- "${ANY[@]}"
has "the extra datacenter with more stock is tried first, labelled non-canonical" "Creating 2 x 5090 \(ladder\) .* in CA-MTL-1 volume volnew \(non-canonical\)"
if [ "$(creates CA-MTL-1)" = 1 ] && [ "$(creates US-TX-3)" = 0 ] && [ "$(creates AP-JP-1)" = 0 ]; then
  ok "one create, in CA-MTL-1; a datacenter outside the allowlist is never used"; else bad "one create, in CA-MTL-1; a datacenter outside the allowlist is never used"; fi
if jq -e '.podnew.dc == "CA-MTL-1" and .podnew.canonical == false' "$UOR_POD_STATE/pods.json" >/dev/null; then
  ok "the extra-datacenter pod is cached as non-canonical"; else bad "the extra-datacenter pod is cached as non-canonical"; fi
rm -rf "$UOR_POD_STATE"; fresh_pods
UOR_POD_VOLUME_DCS="EUR-NO-1 EU-RO-1 EUR-IS-1" expect "UOR_POD_VOLUME_DCS set -> extra datacenters are not used" 1 "no ladder 5090 -> 4090 -> pro6000 stock in EUR-NO-1 EU-RO-1 EUR-IS-1" -- "${ANY[@]}"
if ! grep -q 'pod create' "$FAKE/calls"; then ok "no create anywhere with an explicit datacenter list"; else bad "no create anywhere with an explicit datacenter list"; fi
# ladder + rate cap: 2 x 5090 busts the cap but 2 x 4090 fits -> the ladder goes on
# to the cheaper rung instead of ending `up` on the first rung's cap miss
cat > "$FAKE/gpus.json" <<'J'
[{"gpuId":"NVIDIA GeForce RTX 5090","securePricePerHr":0.99,"dataCenterAvailability":[{"dataCenterId":"EUR-NO-1","stockStatus":"Low"}]},
 {"gpuId":"NVIDIA GeForce RTX 4090","securePricePerHr":0.74,"dataCenterAvailability":[{"dataCenterId":"EUR-NO-1","stockStatus":"Low"}]}]
J
rm -rf "$UOR_POD_STATE"; rm -f "$FAKE/nostock"
echo '[{"id":"podx","name":"x","desiredStatus":"RUNNING","gpuCount":4,"costPerHr":3.5,"uptimeSeconds":10}]' > "$FAKE/pods.json"
: > "$FAKE/calls"
UOR_POD_MAX_RATE=5 expect "ladder: a rate-cap miss on 5090 continues to the cheaper 4090 rung" 0 "Pod podnew created" -- "${ANY[@]}"
has "the cheaper rung is the one created" "Creating 2 x 4090 \(ladder\)"
rm -rf "$UOR_POD_STATE"; echo '[{"id":"podx","name":"x","desiredStatus":"RUNNING","gpuCount":4,"costPerHr":4.9,"uptimeSeconds":10}]' > "$FAKE/pods.json"
UOR_POD_MAX_RATE=5 expect "ladder: every rung over the cap ends with the cap message" 1 "cap: running .* exceeds .*5/h; needs --owner-approved" -- "${ANY[@]}"
# a new volume made for a datacenter whose create then has no stock is deleted;
# a pre-existing volume (the canonical one) never is
cat > "$FAKE/gpus.json" <<'J'
[{"gpuId":"NVIDIA GeForce RTX 5090","securePricePerHr":0.99,"dataCenterAvailability":[{"dataCenterId":"EUR-NO-1","stockStatus":"Low"},{"dataCenterId":"CA-MTL-1","stockStatus":"Low"}]}]
J
rm -rf "$UOR_POD_STATE"; echo '[]' > "$FAKE/pods.json"; printf 'EUR-NO-1\nCA-MTL-1\n' > "$FAKE/nostock"; : > "$FAKE/calls"
expect "stock error in a new extra datacenter" 1 "no instances" -- "${UP[@]}"
if grep -q 'network-volume create --name uor-shared-CA-MTL-1' "$FAKE/calls" && [ "$(grep -c 'network-volume delete' "$FAKE/calls")" = 1 ] && grep -q 'network-volume delete volnew' "$FAKE/calls"; then
  ok "the volume created for CA-MTL-1 is deleted; the canonical volume is not"; else bad "the volume created for CA-MTL-1 is deleted; the canonical volume is not"; fi
rm -f "$FAKE/nostock"
cp "$FAKE/pods.orig.json" "$FAKE/pods.json"
unset FAKE_ALLOW_CREATE UOR_POD_MAX_PODS UOR_POD_MAX_RATE UOR_POD_SSH_WAIT
export UOR_POD_DRY_RUN=1

# ---- bootstrap build lock: the cache section of uor-pod-bootstrap.sh, sourced
# (UOR_BOOTSTRAP_LIB=1) with a stub build and a fake flock. A pod never waits
# long for another pod's build, and two finished builds never corrupt the cache.
BR=$W/binroot
boot() {  # NAME PATTERN [ENV=VALUE...] -> runs obtain_bin once in a subshell, output in $W/out
  local name=$1 pat=$2 rc=0; shift 2
  # shellcheck disable=SC2016  # the script runs in the child bash
  env PATH="$W/fbin:$PATH" UOR_BOOTSTRAP_LIB=1 UOR_BOOTSTRAP_BINROOT="$BR" UOR_BUILD_POLL_S=1 \
    UOR_BUILD_HEARTBEAT_EVERY=1 "$@" bash -c '
    boot=$1 sha=$2
    set -- --sha "$sha" --pod podx
    # shellcheck disable=SC1090
    . "$boot" "$@"
    CAP=120
    # the HF store: STUB_STORE_TAR is what bin/<sha>-sm<cap>.tar holds (none: a miss);
    # an upload is copied to $BINROOT/pushed.tar and logged
    store_get() { [ -n "${STUB_STORE_TAR:-}" ] && [ "$1" = "bin/$SHA-sm$CAP.tar" ] || return 1; cp "$STUB_STORE_TAR" "$2"; }
    store_put() { echo "put $2" >> "$BINROOT/puts"; cp "$1" "$BINROOT/pushed.tar"; }
    build_into() {  # stub: a parity-PASS build after STUB_BUILD_S seconds
      echo build >> "$BINROOT/builds"
      if [ -n "${REG_TAR_STUB:-}" ]; then REG_TAR=$REG_TAR_STUB; REG_PUSH=1; fi
      [ -z "${STUB_DURING_BUILD:-}" ] || eval "$STUB_DURING_BUILD"
      sleep "${STUB_BUILD_S:-0}"
      printf "{\"parity\": \"PASS\", \"built_on_pod\": \"podx\"}\n" > "$1/BUILD.json"
    }
    t0=$(date +%s); obtain_bin; echo "RESULT BIN=$BIN BUILT=$BUILT waited=$(( $(date +%s) - t0 ))"
  ' _ "$HERE/../uor-pod-bootstrap.sh" "$SHA40" > "$W/out" 2>&1 || rc=$?
  if [ "$rc" = 0 ] && grep -qE -e "$pat" "$W/out"; then ok "$name"; else echo "     rc=$rc pattern=$pat"; bad "$name"; fi
}
CB=$BR/$SHA40-sm120
fresh_binroot() { rm -rf "$BR"; mkdir -p "$BR"; }
other_pass() { mkdir -p "$CB"; printf '{"parity": "PASS", "built_on_pod": "other"}\n' > "$CB/BUILD.json"; }
waited() { sed -n 's/.*waited=\([0-9]*\).*/\1/p' "$W/out"; }
fresh_binroot
boot "free lock: build, publish atomically" "cached $CB" STUB_BUILD_S=0
if grep -q '"built_on_pod": "podx"' "$CB/BUILD.json" && ! ls -d "$BR"/.*.tmp.* >/dev/null 2>&1; then ok "published build is complete, no staging left"; else bad "published build is complete, no staging left"; fi
boot "second pod: cache hit, no build" "cache hit: $CB"
fresh_binroot; echo "otherpod 2026-10-05T00:00:00Z phase=build files=10" > "$BR/.$SHA40-sm120.lock.holder"
# Date the heartbeat an hour ahead so its age never passes the 2 s stale limit: only the
# no-progress exit can fire. Otherwise a slow start (busy machine) lets the stale exit win.
FUTURE=$(( $(date -u +%s) + 3600 ))
touch -d "$(date -u -r "$FUTURE" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u -d "@$FUTURE" +%Y-%m-%dT%H:%M:%SZ)" "$BR/.$SHA40-sm120.lock.holder"
boot "lock held, fresh heartbeat without progress: build here after the stall limit" "no visible build progress .*building here" \
  FAKE_FLOCK_HELD=8 UOR_BUILD_WAIT_S=60 UOR_BUILD_HEARTBEAT_STALE_S=2
if [ "$(waited)" -le 10 ] && grep -q '"built_on_pod": "podx"' "$CB/BUILD.json"; then ok "stalled holder costs seconds, not 45 min"; else bad "stalled holder costs seconds, not 45 min"; fi
fresh_binroot
( i=0; while [ $i -lt 40 ]; do echo "otherpod t phase=build files=$i" > "$BR/.$SHA40-sm120.lock.holder"; i=$((i + 1)); sleep 0.25; done ) &
HBW=$!
boot "lock held, progressing heartbeat: wait is bounded by UOR_BUILD_WAIT_S" "waited [0-9]+s \\(limit 3s\\) for another pod's build; building here" \
  FAKE_FLOCK_HELD=8 UOR_BUILD_WAIT_S=3 UOR_BUILD_HEARTBEAT_STALE_S=30
kill "$HBW" 2>/dev/null || true; wait "$HBW" 2>/dev/null || true
if [ "$(waited)" -le 8 ]; then ok "bounded wait ended on time"; else bad "bounded wait ended on time"; fi
fresh_binroot; echo "otherpod t phase=parity files=99" > "$BR/.$SHA40-sm120.lock.holder"
( sleep 1; other_pass ) &
boot "lock held, other pod publishes during the wait: use it, no build" "cache filled by another pod meanwhile" \
  FAKE_FLOCK_HELD=8 UOR_BUILD_WAIT_S=20 UOR_BUILD_HEARTBEAT_STALE_S=20
wait
if [ ! -f "$BR/builds" ]; then ok "no duplicate build when the other pod finished"; else bad "no duplicate build when the other pod finished"; fi
fresh_binroot; echo "gone 2026-01-01T00:00:00Z" > "$BR/.$SHA40-sm120.lock.holder"; touch -t 202601010000 "$BR/.$SHA40-sm120.lock.holder"
boot "lock held by a vanished pod (stale heartbeat): build at once" "heartbeat is [0-9]+s old \\(stale" \
  FAKE_FLOCK_HELD=8 UOR_BUILD_WAIT_S=60 UOR_BUILD_HEARTBEAT_STALE_S=30
fresh_binroot
boot "both pods finish: the first complete build wins, the second is discarded" "another pod published $CB first \\(build complete\\); discarding" \
  STUB_DURING_BUILD="mkdir -p '$CB' && printf '{\"parity\": \"PASS\", \"built_on_pod\": \"other\"}\n' > '$CB/BUILD.json'"
if grep -q '"built_on_pod": "other"' "$CB/BUILD.json" && [ "$(find "$CB" -type f | wc -l | tr -d ' ')" = 1 ] && ! ls -d "$BR"/.*.tmp.* >/dev/null 2>&1; then
  ok "cache untouched by the losing build (no mixing, no staging left)"; else bad "cache untouched by the losing build (no mixing, no staging left)"; fi
fresh_binroot; mkdir -p "$CB"; printf '{"parity": "FAIL"}\n' > "$CB/BUILD.json"
boot "a cached build without parity PASS is moved aside and replaced" "moving it to $CB.failed-"
if grep -q '"built_on_pod": "podx"' "$CB/BUILD.json" && ls -d "$CB".failed-* >/dev/null 2>&1; then ok "failed build kept for inspection, PASS build published"; else bad "failed build kept for inspection, PASS build published"; fi
fresh_binroot; mkdir -p "$CB"; printf '{"parity": "FAIL"}\n' > "$CB/BUILD.json"; touch -t 202601010000 "$BR/.$SHA40-sm120.lock.holder"
boot "publish lock unavailable and cache occupied: keep the build private" "this pod uses $CB.private-podx-" \
  FAKE_FLOCK_HELD="7 8" UOR_BUILD_WAIT_S=60 UOR_BUILD_HEARTBEAT_STALE_S=30
if grep -q FAIL "$CB/BUILD.json" && grep -q "RESULT BIN=$CB.private-podx-" "$W/out"; then ok "occupied cache not overwritten without the publish lock"; else bad "occupied cache not overwritten without the publish lock"; fi

# ---- the HF store backs the binary cache for pods without the shared volume
# (Vast): a parity-PASS store build with matching checksums is a cache hit, any
# other store copy is ignored, and a build made here is uploaded
STOREBINS="geometric-stack chat-grade"
mkstoretar() {  # PARITY [corrupt|subset|wrongcap|symlink] -> $W/store.tar (a build of $SHA40 for sm120)
  local d=$W/storebuild b cap=120; rm -rf "$d"; mkdir -p "$d"
  for b in $STOREBINS; do echo "binary $b" > "$d/$b"; done
  # shellcheck disable=SC2086
  (cd "$d" && sha256sum $STOREBINS > SHA256SUMS)
  case ${2:-} in
    corrupt) echo tampered > "$d/geometric-stack";;
    subset) (cd "$d" && sha256sum geometric-stack > SHA256SUMS);;
    wrongcap) cap=89;;
    symlink) rm "$d/chat-grade"; ln -s /etc/passwd "$d/chat-grade";;
  esac
  printf '{"parity": "%s", "sha": "%s", "compute_cap": %s, "built_on_pod": "storepod"}\n' "$1" "$SHA40" "$cap" > "$d/BUILD.json"
  tar -C "$d" -cf "$W/store.tar" .
}
fresh_binroot; mkstoretar PASS
boot "the store's parity-PASS build is a cache hit (no build)" "cache hit from the HF store: $CB" STUB_STORE_TAR="$W/store.tar"
if [ ! -f "$BR/builds" ] && grep -q '"built_on_pod": "storepod"' "$CB/BUILD.json" && [ ! -f "$BR/puts" ] && ! ls -d "$BR"/.*store* >/dev/null 2>&1; then
  ok "store build published as the cache, nothing re-uploaded, no staging left"; else bad "store build published as the cache, nothing re-uploaded, no staging left"; fi
fresh_binroot; mkstoretar PASS corrupt
boot "a store build failing its checksums is ignored: build here" "not a complete build with matching checksums; building here" STUB_STORE_TAR="$W/store.tar"
if [ -f "$BR/builds" ] && grep -q '"built_on_pod": "podx"' "$CB/BUILD.json"; then ok "the tampered store build never reaches the cache"; else bad "the tampered store build never reaches the cache"; fi
fresh_binroot; mkstoretar FAIL
boot "a store build without parity PASS is ignored: build here" "not a complete build" STUB_STORE_TAR="$W/store.tar"
for bad in subset wrongcap symlink; do
  fresh_binroot; mkstoretar PASS "$bad"
  boot "a store build that is $bad is ignored: build here" "not a complete build" STUB_STORE_TAR="$W/store.tar"
  if [ -f "$BR/builds" ] && grep -q '"built_on_pod": "podx"' "$CB/BUILD.json"; then ok "the $bad store build never reaches the cache"; else bad "the $bad store build never reaches the cache"; fi
done
# the real build step never talks to the store: every upload waits for the publish
if ! sed -n '/^build_into() {/,/^}$/p' "$HERE/../uor-pod-bootstrap.sh" | grep -q 'store_'; then ok "build_into makes no store call (uploads follow the publish)"; else bad "build_into makes no store call (uploads follow the publish)"; fi
# the crate cache goes up only after the build is published (waiting pods see the cache first)
fresh_binroot; echo crates > "$W/reg.tar"
boot "the crate cache is uploaded after the build is published" "uploaded toolchain/cargo-registry.tar" REG_TAR_STUB="$W/reg.tar"
if [ "$(sed -n 1p "$BR/puts")" = "put bin/$SHA40-sm120.tar" ] && [ "$(sed -n 2p "$BR/puts")" = "put toolchain/cargo-registry.tar" ] &&
   [ "$(grep -n "cached $CB" "$W/out" | cut -d: -f1)" -lt "$(grep -n 'uploaded toolchain/cargo-registry.tar' "$W/out" | cut -d: -f1)" ]; then
  ok "order: publish, binaries, then the crate cache"; else sed 's/^/     /' "$BR/puts"; bad "order: publish, binaries, then the crate cache"; fi
fresh_binroot
boot "store miss: build here, then upload it for the next pod" "uploaded bin/$SHA40-sm120.tar to the HF store"
if grep -qx "put bin/$SHA40-sm120.tar" "$BR/puts" && tar -tf "$BR/pushed.tar" | grep -q 'BUILD.json'; then ok "the uploaded tar is this build"; else bad "the uploaded tar is this build"; fi
# uor-store (the pod's command for the HF store) against a fake hf whose store is a directory
HFS=$W/hfstore; mkdir -p "$HFS" "$W/hfbin"
cat > "$W/hfbin/hf" <<'S'
#!/usr/bin/env bash
echo "hf $*" >> "$FAKE/hf-calls"
cmd=$1; shift; repo=$1; shift
case $cmd in
  upload) src=$1 dst=$2; mkdir -p "$(dirname "$HFS/$dst")"; if [ -d "$src" ]; then mkdir -p "$HFS/$dst"; cp -R "$src/." "$HFS/$dst/"; else cp "$src" "$HFS/$dst"; fi;;
  download) path='' inc='' dir=.; while [ $# -gt 0 ]; do case $1 in --repo-type) shift 2;; --include) inc=$2; shift 2;; --local-dir) dir=$2; shift 2;; *) path=$1; shift;; esac; done
    if [ -n "$path" ]; then [ -e "$HFS/$path" ] || exit 0; mkdir -p "$(dirname "$dir/$path")"; cp -R "$HFS/$path" "$dir/$path"
    else p=${inc%\*}; [ -e "$HFS/$p" ] || exit 0; mkdir -p "$dir/$p"; cp -R "$HFS/$p." "$dir/$p"; fi;;
esac
S
chmod +x "$W/hfbin/hf"
us() { env PATH="$W/hfbin:$PATH" HFS="$HFS" HF_TOKEN_PATH="$W/notoken" bash "$HERE/../uor-store" "$@" > "$W/out" 2>&1; }
mkdir -p "$W/res/run1"; echo 0.42 > "$W/res/run1/metric.txt"
if us put "$W/res/run1" results/claude/demo-2026-10-13/run1 && [ "$(cat "$HFS/results/claude/demo-2026-10-13/run1/metric.txt")" = 0.42 ]; then ok "uor-store put uploads a directory"; else bad "uor-store put uploads a directory"; fi
if us get results/claude/demo-2026-10-13/run1/ "$W/back" && [ "$(cat "$W/back/results/claude/demo-2026-10-13/run1/metric.txt")" = 0.42 ]; then ok "uor-store get brings a folder back"; else bad "uor-store get brings a folder back"; fi
if ! us get results/nothing-here.txt "$W/back" && grep -q "nothing downloaded for results/nothing-here.txt" "$W/out"; then ok "uor-store get reports a missing path instead of a silent nothing"; else bad "uor-store get reports a missing path instead of a silent nothing"; fi
# uor-store ls follows the API's pages (a listing that stops at page one misses files)
cat > "$W/hfbin/curl" <<'S'
#!/usr/bin/env bash
hdr='' url=''; while [ $# -gt 0 ]; do case $1 in -D) hdr=$2; shift 2;; -H) shift 2;; -*) shift;; *) url=$1; shift;; esac; done
cat > /dev/null
case $url in
  *page=2*) : > "$hdr"; echo '[{"type":"file","path":"data/b.tar","size":2097152}]';;
  *) printf 'HTTP/2 200\r\nlink: <%s&page=2>; rel="next"\r\n\r\n' "$url" > "$hdr"; echo '[{"type":"directory","path":"data"},{"type":"file","path":"data/a.tar","size":1048576}]';;
esac
S
chmod +x "$W/hfbin/curl"; echo tok > "$W/notoken"
if us ls data && grep -q '^1 MB  data/a.tar$' "$W/out" && grep -q '^2 MB  data/b.tar$' "$W/out" && ! grep -q 'tok' "$W/out"; then ok "uor-store ls lists every page, without the token"; else bad "uor-store ls lists every page, without the token"; fi
# an old local copy is not a download: the path is gone from the store, DIR still has it
rm -rf "$HFS/results/claude/demo-2026-10-13"
if ! us get results/claude/demo-2026-10-13/run1/ "$W/back" && grep -q "nothing downloaded" "$W/out" && ! ls -d "$W/back"/.uor-store.* >/dev/null 2>&1; then
  ok "uor-store get never reports an old local copy as downloaded"; else bad "uor-store get never reports an old local copy as downloaded"; fi
if ! us put "$W/res/run1" ../escape && grep -q "without a leading / or '..'" "$W/out"; then ok "uor-store refuses a path outside the store"; else bad "uor-store refuses a path outside the store"; fi
if grep -q -- "upload caseyallard/uor-r4-store $W/res/run1 results/claude/demo-2026-10-13/run1 --repo-type dataset" "$FAKE/hf-calls"; then ok "uor-store uses the project store as a dataset"; else bad "uor-store uses the project store as a dataset"; fi

# ---- Ollama on the pod is optional and never fatal (the bootstrap's Ollama
# functions, sourced with a fake curl that serves files from $W/serve by name and
# answers 404 otherwise, like ollama.com's retired .tgz on 2026-10-05).
mkdir -p "$W/cbin" "$W/serve" "$W/opkg/bin"
cat > "$W/cbin/curl" <<'S'
#!/usr/bin/env bash
out='' url=''; while [ $# -gt 0 ]; do case $1 in -o) out=$2; shift 2;; --retry) shift 2;; -*) shift;; *) url=$1; shift;; esac; done
echo "$url" >> "$FAKE/curl-calls"
f=$FAKE_SERVE/$(basename "$url")
[ -f "$f" ] || { echo "curl: (22) The requested URL returned error: 404" >&2; exit 22; }
if [ -n "$out" ]; then cp "$f" "$out"; else cat "$f"; fi
S
printf '#!/bin/sh\nexit 0\n' > "$W/cbin/setsid"
# shellcheck disable=SC2016  # the fake ollama's own variables
printf '#!/bin/sh\n[ "$1" = pull ] && exit "${FAKE_PULL_RC:-0}"\nexit 0\n' > "$W/opkg/bin/ollama"
chmod +x "$W/cbin/"* "$W/opkg/bin/ollama"
olla() {  # NAME PATTERN [ENV=VALUE...] -> runs ollama_step under set -e, then a later step
  local name=$1 pat=$2 rc=0; shift 2
  rm -rf "$W/odir" "$W/omodels"; : > "$FAKE/curl-calls"; mkdir -p "$W/ologs"
  # shellcheck disable=SC2016  # the script runs in the child bash
  env PATH="$W/cbin:$PATH" FAKE_SERVE="$W/serve" UOR_BOOTSTRAP_LIB=1 UOR_OLLAMA_LOGDIR="$W/ologs" UOR_OLLAMA_DIR="$W/odir" UOR_OLLAMA_MODELS="$W/omodels" \
    UOR_OLLAMA_INSTALL_SH=https://ollama.invalid/install.sh "$@" bash -c '
    set -euo pipefail
    # shellcheck disable=SC1090
    . "$1" --sha "$2" --pod podx
    ollama_step
    echo "AFTER OLLAMA: status=$OLLAMA_STATUS (the bootstrap continues)"
  ' _ "$HERE/../uor-pod-bootstrap.sh" "$SHA40" > "$W/out" 2>&1 || rc=$?
  if [ "$rc" = 0 ] && grep -qE -e "$pat" "$W/out"; then ok "$name"; else echo "     rc=$rc pattern=$pat"; bad "$name"; fi
}
olla "every Ollama source 404: status FAILED, bootstrap continues" "AFTER OLLAMA: status=FAILED"
has "the failure is logged as a warning" "WARNING: Ollama install FAILED; continuing without it"
if head -1 "$FAKE/curl-calls" | grep -q 'ollama-linux-amd64.tar.zst$' && sed -n 2p "$FAKE/curl-calls" | grep -q 'ollama-linux-amd64.tgz$' &&
   grep -q 'install.sh$' "$FAKE/curl-calls"; then ok "tries .tar.zst, then the legacy .tgz, then install.sh"; else sed 's/^/     /' "$FAKE/curl-calls"; bad "tries .tar.zst, then the legacy .tgz, then install.sh"; fi
if command -v zstd >/dev/null 2>&1; then
  tar -cf - -C "$W/opkg" bin | zstd -q -o "$W/serve/ollama-linux-amd64.tar.zst"
  olla "current .tar.zst asset installs Ollama" "AFTER OLLAMA: status=ready"
  if [ -x "$W/odir/bin/ollama" ] && ! grep -q '\.tgz$' "$FAKE/curl-calls"; then ok "installed onto the volume dir from .tar.zst"; else bad "installed onto the volume dir from .tar.zst"; fi
  olla "failed model pull is a warning (pull-failed), not fatal" "AFTER OLLAMA: status=pull-failed" FAKE_PULL_RC=1
  rm -f "$W/serve/ollama-linux-amd64.tar.zst"
else
  echo "skip .tar.zst install tests (no zstd on this machine)"
fi
tar -czf "$W/serve/ollama-linux-amd64.tgz" -C "$W/opkg" bin
olla "legacy .tgz still works when the .tar.zst is missing" "AFTER OLLAMA: status=ready"

# ---- portability: config file, BOARD/TEMPLATE none, --hf-store/--repo-url to the pod
expect "default board posts a gh issue comment" 0 "DRY-RUN: gh issue comment 2037" -- keep poda --purpose portability
if UOR_POD_BOARD=none "$TOOL" keep poda --purpose portability > "$W/out" 2>&1; then ok "UOR_POD_BOARD=none still runs"; else bad "UOR_POD_BOARD=none still runs"; fi
hasnt "UOR_POD_BOARD=none posts no gh issue comment" "gh issue comment"
if UOR_POD_BOARD= "$TOOL" keep poda --purpose portability > "$W/out" 2>&1; then ok "empty UOR_POD_BOARD still runs"; else bad "empty UOR_POD_BOARD still runs"; fi
hasnt "empty UOR_POD_BOARD posts no gh issue comment" "gh issue comment"
"$TOOL" log -n 1 > "$W/out" 2>&1; has "UOR_POD_BOARD=none still writes the ledger" "keep .*portability"
expect "default bootstrap args carry no --hf-store/--repo-url" 0 "run: bash /root/uor-pod-bootstrap.sh --sha $MAIN_SHA --pod poda \|" -- bootstrap poda --force --ref main
UOR_HF_STORE=caseyallard/uor-r4-data expect "UOR_HF_STORE reaches the bootstrap as --hf-store" 0 "uor-pod-bootstrap.sh --sha $MAIN_SHA --pod poda --hf-store caseyallard/uor-r4-data \|" -- bootstrap poda --force --ref main
UOR_POD_REPO_URL=file://$W/origin.git expect "a fork URL reaches the bootstrap as --repo-url" 0 "--pod poda --repo-url file://$W/origin.git \|" -- bootstrap poda --force --ref main
chk "bootstrap --check-args accepts --hf-store and --repo-url" 0 "bootstrap arguments OK" --sha "$MAIN_SHA" --pod podx --hf-store caseyallard/uor-r4-data --repo-url https://github.com/me/fork.git
chk "bootstrap --check-args rejects a ; in --hf-store" 2 "--hf-store may contain only" --sha "$MAIN_SHA" --hf-store 'a;b'
chk "bootstrap --check-args rejects a space in --repo-url" 2 "--repo-url may contain only" --sha "$MAIN_SHA" --repo-url 'https://x y'
chk "bootstrap --check-args rejects --hf-store without a value" 2 "--hf-store needs a value" --sha "$MAIN_SHA" --hf-store

# ---- Vast.ai, the default provider for new pods (owner 2026-10-13): placement,
# combined caps, both providers in status, archive-then-destroy at down, and the
# instance key from `create` never reaching output, ledger or state.
VO() {  # VO GPU_NAME COUNT 'JSON' -> the fake's offers for that search
  printf '%s\n' "$3" > "$FAKE/vast-offers-$1-$2.json"
}
VSEEN=$W/vast-seen; : > "$VSEEN"
vexpect() { expect "$@"; cat "$W/out" >> "$VSEEN"; }
vfresh() {  # empty accounts on both providers, fresh state
  rm -rf "$UOR_POD_STATE"; echo '[]' > "$FAKE/pods.json"; echo '[]' > "$FAKE/vast-instances.json"
  rm -f "$FAKE"/vast-offers-*.json "$FAKE/vast-gone"; : > "$FAKE/calls"
}
VH=$W/vasthome; mkdir -p "$VH"
export HOME=$VH UOR_POD_STATE=$W/state-vast UOR_POD_DRY_RUN=1 UOR_POD_MAX_PODS=4 UOR_POD_MAX_RATE=8.00
unset UOR_POD_PROVIDER FAKE_ALLOW_CREATE
export UOR_POD_PROVIDERS="vast runpod"   # a Vast account that still lists its Runpod pods
cat > "$FAKE/gpus.json" <<'J'
[{"gpuId":"NVIDIA GeForce RTX 5090","securePricePerHr":0.99,"dataCenterAvailability":[{"dataCenterId":"EU-RO-1","stockStatus":"Low"}]}]
J
OFF5='[{"id":7002,"dph_total":0.9,"num_gpus":1,"geolocation":"Hungary, HU","reliability2":0.995},{"id":7001,"dph_total":0.469,"num_gpus":1,"geolocation":"South Korea, KR","reliability2":0.998},{"id":7003,"dph_total":0.6,"num_gpus":1,"geolocation":"Maryland, US","reliability2":0.999}]'
VUP=(up "${X1[@]}" --gpu 5090 --count 1 --purpose vast --hours 1 --ref "$MAIN_SHA")
DRIVER_OFFERS='[{"id":7001,"dph_total":0.1,"num_gpus":1,"driver_version":"595.71.05"},{"id":7003,"dph_total":0.60049,"num_gpus":1,"geolocation":"Ireland, IE","reliability2":0.999,"driver_version":"580.95.05","machine_id":88,"host_id":99},{"id":7004,"dph_total":0.01,"num_gpus":1},{"id":7005,"dph_total":0.02,"num_gpus":1,"driver_version":580.9505}]'
vfresh; VO RTX_5090 1 "$DRIVER_OFFERS"
UOR_POD_VAST_DRIVER_VERSION=580.95.05 vexpect "exact driver excludes cheaper wrong, missing and numeric metadata" 0 "DRY-RUN: vastai create instance 7003" -- "${VUP[@]}"
if grep -q 'driver_version=580.95.05 --storage' "$FAKE/vast-query"; then ok "exact driver is sent as an equality query"; else bad "exact driver is sent as an equality query"; fi
if jq -e 'select(.event=="vast-offer-selected") | .offer_selection | .offer_id==7003 and .requested_driver=="580.95.05" and .driver_version=="580.95.05" and .machine_id==88 and .host_id==99 and .quoted_usd_per_h==0.60049 and (.query_started_at|test("^[0-9]{4}-[0-9]{2}-[0-9]{2}T"))' "$UOR_POD_STATE/ledger.jsonl" >/dev/null; then ok "precreate receipt binds offer, driver, host, raw quote and query time"; else bad "precreate receipt binds offer, driver, host, raw quote and query time"; fi
NO_DRIVER_MATCH=$(echo "$DRIVER_OFFERS" | jq 'map(select(.id!=7003))')
vfresh; VO RTX_5090 1 "$NO_DRIVER_MATCH"
UOR_POD_VAST_DRIVER_VERSION=580.95.05 vexpect "no driver match refuses without allocation" 1 "no Vast offer with exact driver 580.95.05; nothing allocated" -- "${VUP[@]}"
if ! grep -q '^vastai create instance' "$FAKE/calls" && ! grep -q 'DRY-RUN: vastai create' "$W/out"; then ok "missing or wrong driver metadata causes zero create calls"; else bad "missing or wrong driver metadata causes zero create calls"; fi
vfresh; VO RTX_5090 1 "$DRIVER_OFFERS"
UOR_POD_VAST_DRIVER_VERSION=580.95.05 UOR_POD_MAX_RATE=0.5 vexpect "matching driver still obeys price cap" 1 "exceeds.*0.5/h" -- "${VUP[@]}"
UOR_POD_VAST_DRIVER_VERSION='580.95.05*' vexpect "driver wildcard is refused before placement" 1 "requires an exact three-part decimal version" -- "${VUP[@]}"
UOR_POD_VAST_DRIVER_VERSION=580.95.05 vexpect "driver constraint cannot be sent to Runpod even with override" 1 "exact driver selection requires Vast" -- "${VUP[@]}" --provider runpod --owner-approved
vfresh; VO RTX_5090 1 "$(echo "$DRIVER_OFFERS" | jq '[.[]|select(.id==7003)] | . + .')"
UOR_POD_VAST_DRIVER_VERSION=580.95.05 vexpect "duplicate matching offer IDs refuse at mutation boundary" 1 "requires one matching metadata object before create" -- "${VUP[@]}"
if ! grep -q '^vastai create instance' "$FAKE/calls" && ! grep -q 'DRY-RUN: vastai create' "$W/out"; then ok "ambiguous duplicate metadata makes zero create calls"; else bad "ambiguous duplicate metadata makes zero create calls"; fi
vfresh; VO RTX_5090 1 "$DRIVER_OFFERS"
UOR_POD_VAST_DRIVER_VERSION=580.95.05 FAKE_VAST_LEDGER_BLOCK=1 vexpect "selection receipt append failure refuses before allocation" 1 "cannot record exact Vast offer selection; nothing allocated" -- "${VUP[@]}"
if ! grep -q '^vastai create instance' "$FAKE/calls" && ! grep -q 'DRY-RUN: vastai create' "$W/out"; then ok "failed precreate receipt makes zero create calls"; else bad "failed precreate receipt makes zero create calls"; fi
# Reset fixtures: the following existing tests prove unchanged unset behavior.
vfresh; VO RTX_5090 1 "$OFF5"
vexpect "Vast is the default provider and rents the cheapest offer" 0 "DRY-RUN: vastai create instance 7001 --image runpod/pytorch:1.0.2-cu1281-torch280-ubuntu2404 --disk 150 --ssh --direct --label uor-codex-" -- "${VUP[@]}"
has "the offer is announced with its price and place" "Creating 1 x 5090 \\(\\\$0.469/h\\) on Vast South Korea, KR, offer 7001"
if grep -q 'gpu_name=RTX_5090 num_gpus=1 datacenter=true verified=true rentable=true reliability>0.98 cuda_vers>=12.8 disk_space>=150 inet_down>=200 --storage 150 ' "$FAKE/vast-query"; then
  ok "the search asks for verified datacenter hosts, CUDA 12.8+, disk and bandwidth, priced with our disk"; else sed 's/^/     /' "$FAKE/vast-query"; bad "the search asks for verified datacenter hosts, CUDA 12.8+, disk and bandwidth, priced with our disk"; fi
if ! grep -q 'runpodctl pod create\|runpodctl gpu list' "$FAKE/calls"; then ok "a Vast up makes no Runpod placement call"; else bad "a Vast up makes no Runpod placement call"; fi
vexpect "on a Vast account, Runpod by hand is refused (backup only)" 1 "Runpod is the backup only .*--owner-approved" -- "${VUP[@]}" --provider runpod
if ! grep -q 'runpodctl pod create' "$FAKE/calls"; then ok "the refusal creates nothing"; else bad "the refusal creates nothing"; fi
vexpect "--provider runpod --owner-approved places on Runpod" 0 "DRY-RUN: runpodctl pod create" -- "${VUP[@]}" --provider runpod --owner-approved
UOR_POD_PROVIDER=runpod vexpect "UOR_POD_PROVIDER=runpod with --owner-approved places on Runpod" 0 "Creating 1 x 5090 .* in EU-RO-1" -- "${VUP[@]}" --owner-approved
vexpect "--provider takes vast or runpod only" 1 "--provider must be vast or runpod" -- "${VUP[@]}" --provider aws
UOR_POD_PROVIDER=aws vexpect "a bad UOR_POD_PROVIDER is refused" 1 "UOR_POD_PROVIDER must be vast or runpod" -- status
vexpect "a Vast pod id must be vast-<number>" 1 "vast-<contract number>" -- lease vast-abc "${X1[@]}" --gpus 0 --purpose x --hours 1
# ladder: no 5090 offer -> 4090; pro6000 is searched by Vast's own name
vfresh; VO RTX_4090 1 '[{"id":7101,"dph_total":0.4,"num_gpus":1,"geolocation":"Norway, NO","reliability2":0.99}]'
vexpect "ladder on Vast: no 5090 offer -> 4090" 0 "Creating 1 x 4090 \\(ladder\\) .* offer 7101" -- up "${X1[@]}" --count 1 --purpose vast --hours 1 --ref "$MAIN_SHA"
has "the 5090 miss is reported" "no Vast offer for 1 x 5090"
vfresh; VO RTX_PRO_6000_S 1 '[{"id":7201,"dph_total":1.2,"num_gpus":1,"geolocation":"Iceland, IS","reliability2":0.99}]'
vexpect "pro6000 maps to Vast's RTX PRO 6000 S" 0 "offer 7201" -- up "${X1[@]}" --gpu pro6000 --count 1 --purpose vast --hours 1 --ref "$MAIN_SHA"
vfresh
vexpect "no Vast offer at all names --wait and the Runpod backup after the stretch" 1 "no Vast offer for 5090 .*--wait.*after 30 min with no Vast offer, uses Runpod as the backup" -- "${VUP[@]}"
# caps are global: Runpod pods and Vast instances count together
vfresh; VO RTX_5090 1 "$OFF5"
echo '[{"id":"podr","name":"r","desiredStatus":"RUNNING","gpuCount":2,"costPerHr":7.8,"uptimeSeconds":60}]' > "$FAKE/pods.json"
vexpect "rate cap counts a Runpod pod against a Vast offer" 1 "cap: running \\\$7.8/h \\+ new \\\$0.469/h exceeds \\\$8.00/h" -- "${VUP[@]}"
echo '[{"id":"podr","name":"r","desiredStatus":"RUNNING","gpuCount":2,"costPerHr":1,"uptimeSeconds":60},{"id":"pods","name":"s","desiredStatus":"RUNNING","gpuCount":2,"costPerHr":1,"uptimeSeconds":60}]' > "$FAKE/pods.json"
echo '[{"id":8001,"label":"uor-claude-x","num_gpus":1,"dph_total":0.5,"intended_status":"running","actual_status":"running","public_ipaddr":"10.0.0.3","ports":{"22/tcp":[{"HostPort":"2002"}]}},
       {"id":8002,"label":"uor-deepseek-y","num_gpus":1,"dph_total":0.5,"intended_status":"running","actual_status":"loading"}]' > "$FAKE/vast-instances.json"
vexpect "pod cap counts Runpod pods and Vast instances (a loading one too)" 1 "cap reached \\(4 of 4 pods\\)" -- "${VUP[@]}"
FAKE_VAST_LIST_FAIL=1 vexpect "up refuses when a provider listing fails (caps unknowable)" 1 "Vast: the instance listing failed, so the caps cannot be checked" -- "${VUP[@]}"
vexpect "status lists both providers" 0 "vast-8001 +uor-claude-x +RUNNING +1x.* vol=vast-disk" -- status
has "status lists the Runpod pod too" "^podr +r +RUNNING"
has "status shows a loading Vast instance as running (it bills)" "vast-8002 +uor-deepseek-y +RUNNING"
has "status totals both providers" "Running: 4 pod\\(s\\) .*\\\$3/h"
has "status shows Vast offers" "^Vast offers \\(verified datacenter hosts"
has "status shows the cheapest Vast offer" "^  5090: 3 offers, cheapest \\\$0.469 \\(South Korea, KR\\)"
hasnt "status never shows the instance's tokens" "SENTINEL"
# a failed listing is not evidence that a pod is gone (non-dry: status archives
# the lease files of pods its provider no longer lists)
mkdir -p "$UOR_POD_STATE/leases/vast-8001"; echo '{"lab":"claude","session":"v","id":"claude-v","gpus":[0],"expires":"2099-01-01T00:00:00Z"}' > "$UOR_POD_STATE/leases/vast-8001/claude-v.json"
UOR_POD_DRY_RUN=0 FAKE_VAST_LIST_FAIL=1 vexpect "status survives a failed Vast listing" 0 "Vast: the instance listing failed; its pods are not shown" -- status
if [ -f "$UOR_POD_STATE/leases/vast-8001/claude-v.json" ]; then ok "a failed listing keeps the pod's live leases"; else bad "a failed listing keeps the pod's live leases"; fi
echo '[]' > "$FAKE/vast-instances.json"
UOR_POD_DRY_RUN=0 vexpect "a Vast pod that is really gone has its leases archived" 0 "vast-8001: gone from its provider" -- status
# ---- non-dry Vast up -> run -> down against the fake CLI (nothing leaves this machine)
vfresh; VO RTX_5090 1 "$OFF5"; echo 7001 > "$FAKE/vast-gone"
printf 'gpu=0, NVIDIA GeForce RTX 5090, 0, 1\nreaper=1\nprobe_ok=1\n' > "$FAKE/probe-2001"
mkdir -p "$VH/.cache/huggingface"; printf 'hf_FAKETOKEN123' > "$VH/.cache/huggingface/token"
export UOR_POD_DRY_RUN=0 FAKE_ALLOW_CREATE=1 UOR_POD_SSH_WAIT=30
vfresh; VO RTX_5090 1 "$DRIVER_OFFERS"
UOR_POD_VAST_DRIVER_VERSION=580.95.05 vexpect "mocked exact-driver create links selection to successful up" 0 "SSH ready after" -- "${VUP[@]}" --no-bootstrap
if jq -e '.event=="vast-offer-selected" and .offer_selection.offer_id==7003' "$FAKE/ledger-at-create.json" >/dev/null &&
   jq -e '."vast-9001".offer_selection | .offer_id==7003 and .driver_version=="580.95.05" and .quoted_usd_per_h==0.60049' "$UOR_POD_STATE/pods.json" >/dev/null &&
   grep '"event":"up"' "$UOR_POD_STATE/ledger.jsonl" | jq -e '.offer_selection.offer_id==7003' >/dev/null; then
  ok "selection is written before fake create and shared by up/cache receipts"
else bad "selection is written before fake create and shared by up/cache receipts"; fi
vfresh; VO RTX_5090 1 "$NO_DRIVER_MATCH"
UOR_POD_VAST_DRIVER_VERSION=580.95.05 UOR_POD_WAIT_INTERVAL=1 UOR_POD_RUNPOD_FALLBACK_MIN=0 vexpect "driver-selected wait cannot fall back to Runpod" 1 "no 5090 stock after waiting" -- "${VUP[@]}" --wait --wait-hours 0.00001 --no-bootstrap
if ! grep -Eq '^vastai create instance|^runpodctl pod create' "$FAKE/calls"; then ok "selected wait makes zero provider create calls"; else bad "selected wait makes zero provider create calls"; fi
# Continue the existing unselected non-dry fake-provider scenarios.
vfresh; VO RTX_5090 1 "$OFF5"; echo 7001 > "$FAKE/vast-gone"
vexpect "Vast up: a taken offer moves on to the next cheapest" 0 "Ready: ssh -i .* -p 2001 root@10.0.0.2" -- up "${X1[@]}" --gpu 5090 --count 1 --purpose vast --hours 1 --ref "$MAIN_SHA"
has "the taken offer is named with Vast's reason" "Vast offer 7001 was not rented \\(offer 7001 is no longer available\\); trying the next offer"
has "the next cheapest offer (7003, not 7002) is rented" "offer 7003, for codex/x1"
has "Vast pods skip the hot-set relay" "Vast pod: no hot-set seeding"
has "the HF token is copied" "hf token copied"
if grep -q -- "--pod vast-9001 --off-volume --non-canonical" "$FAKE/bootcmd"; then ok "the bootstrap fetches from the HF store (--off-volume --non-canonical)"; else sed 's/^/     /' "$FAKE/bootcmd"; bad "the bootstrap fetches from the HF store (--off-volume --non-canonical)"; fi
if jq -e '."vast-9001" | .provider == "vast" and .off_volume == true and .canonical == false and .gpu == "5090"' "$UOR_POD_STATE/pods.json" >/dev/null &&
   [ -f "$UOR_POD_STATE/leases/vast-9001/codex-x1.json" ]; then
  ok "the Vast pod is cached as vast-9001 under the creator's lease"; else bad "the Vast pod is cached as vast-9001 under the creator's lease"; fi
vexpect "run on the Vast pod" 0 "Started on vast-9001 gpu 0" -- run vast-9001 "${X1[@]}" --gpu 0 -- nvidia-smi -L
# only jobs/ exists (as on a fresh pod): the missing pods/runs/evals are skipped
mkdir -p "$FAKE/vast-ws/uor-r4/jobs/codex"; echo result > "$FAKE/vast-ws/uor-r4/jobs/codex/r.log"
FAKE_VAST_KB=$((6 * 1024 * 1024)) vexpect "down refuses when the results are too big for the laptop" 1 "results not archived .*nothing deleted" -- down vast-9001 "${X1[@]}"
has "the refusal says how big" "holds 6144 MB of results \\(over 5120 MB"
if jq -e 'any(.[]; .id == 9001)' "$FAKE/vast-instances.json" >/dev/null; then ok "nothing was destroyed"; else bad "nothing was destroyed"; fi
vexpect "down archives the results, then destroys" 0 "Deleted vast-9001" -- down vast-9001 "${X1[@]}"
has "the archive is reported" "Archived 0 MB of vast-9001 results to $UOR_POD_STATE/vast-archive/vast-9001"
if [ "$(cat "$UOR_POD_STATE/vast-archive/vast-9001/uor-r4/jobs/codex/r.log" 2>/dev/null)" = result ]; then ok "the results are on the laptop"; else bad "the results are on the laptop"; fi
if grep -q '^vastai destroy instance 9001$' "$FAKE/calls" && [ "$(jq length "$FAKE/vast-instances.json")" = 0 ]; then
  ok "destroy answered the [y/N] prompt and the instance is gone"; else bad "destroy answered the [y/N] prompt and the instance is gone"; fi
if grep '"event":"vast-archive"' "$UOR_POD_STATE/ledger.jsonl" | jq -e --arg d "$UOR_POD_STATE/vast-archive/vast-9001" '.dest == $d and .kb > 0' >/dev/null; then
  ok "the archive is in the ledger"; else bad "the archive is in the ledger"; fi
# a bootstrap failure destroys the new Vast pod as well
vfresh; VO RTX_5090 1 "$OFF5"
FAKE_BOOT_RC=2 vexpect "a failed Vast bootstrap destroys the instance" 1 "bootstrap failed: deleting pod vast-9001" -- up --lab claude --session vboot --gpu 5090 --count 1 --purpose vast --hours 1 --ref "$MAIN_SHA"
if [ "$(jq length "$FAKE/vast-instances.json")" = 0 ]; then ok "no instance is left billing"; else bad "no instance is left billing"; fi
# a reply that is not a list (exit 0) proves nothing: destroy is not confirmed, leases stay
vfresh; VO RTX_5090 1 "$OFF5"
vexpect "Vast up for the bad-listing cases" 0 "Ready: ssh" -- up --lab claude --session vbad --gpu 5090 --count 1 --purpose vast --hours 1 --ref "$MAIN_SHA"
rm -rf "$FAKE/vast-ws"
FAKE_VAST_LIST_OBJ=1 UOR_POD_VAST_POLL_S=0 vexpect "down: an error object from the listing is not 'gone'" 1 "delete of vast-9001 not confirmed" -- down vast-9001 --lab claude --session vbad
if [ -f "$UOR_POD_STATE/leases/vast-9001/claude-vbad.json" ]; then ok "an unconfirmed destroy keeps the leases"; else bad "an unconfirmed destroy keeps the leases"; fi
echo '[{"id": 9001, "label": "uor-claude-x", "num_gpus": 1, "dph_total": 0.5, "intended_status": "running", "actual_status": "running"}]' > "$FAKE/vast-instances.json"
FAKE_VAST_LIST_OBJ=1 vexpect "status: an error object from the listing is a failed listing" 0 "Vast: the instance listing failed; its pods are not shown" -- status
if [ -f "$UOR_POD_STATE/leases/vast-9001/claude-vbad.json" ]; then ok "an error-object listing keeps the leases"; else bad "an error-object listing keeps the leases"; fi
FAKE_VAST_LIST_OBJ=1 vexpect "up: an error-object listing stops up" 1 "Vast: the instance listing failed, so the caps cannot be checked" -- "${VUP[@]}"
# a Runpod listing that fails once and then works: status must judge "gone"
# from the listing it was built from, not from a second one
echo '[{"id":"podr","name":"r","desiredStatus":"RUNNING","gpuCount":1,"costPerHr":1,"uptimeSeconds":60}]' > "$FAKE/pods.json"
mkdir -p "$UOR_POD_STATE/leases/podr"; echo '{"lab":"deepseek","session":"r","id":"deepseek-r","gpus":[0],"expires":"2099-01-01T00:00:00Z"}' > "$UOR_POD_STATE/leases/podr/deepseek-r.json"
touch "$FAKE/rp-fail-once"
vexpect "status survives a Runpod listing that fails once" 0 "Runpod: the pod listing failed; its pods are not shown" -- status
if [ -f "$UOR_POD_STATE/leases/podr/deepseek-r.json" ]; then ok "a later successful listing does not archive the unlisted pod's leases"; else bad "a later successful listing does not archive the unlisted pod's leases"; fi
# a missing CLI hides a provider's pods from the caps: up refuses unless the
# account names only the providers it uses
mkdir -p "$W/novast"; for b in runpodctl ssh gh; do ln -sf "$W/bin/$b" "$W/novast/$b"; done
NOVAST="$W/novast:$(dirname "$(command -v jq)"):/usr/bin:/bin"
PATH=$NOVAST UOR_POD_VAST_BIN=$W/novast/no-provider vexpect "up without the Rust Vast adapter refuses (Vast pods would be uncounted)" 1 "Vast: Rust adapter missing.*caps cannot be checked" -- "${VUP[@]}" --provider runpod --owner-approved
PATH=$NOVAST UOR_POD_PROVIDERS=runpod vexpect "a Runpod-only account (UOR_POD_PROVIDERS=runpod) needs no vastai" 0 "Pod podnew created|Creating 1 x 5090 .* in EU-RO-1" -- "${VUP[@]}" --provider runpod --no-bootstrap
# no Vast offer for a sustained stretch: --wait falls back to Runpod (stock in
# EU-RO-1), ledgered; before that stretch it only retries Vast
vfresh
UOR_POD_DRY_RUN=0 FAKE_ALLOW_CREATE=1 UOR_POD_WAIT_INTERVAL=1 UOR_POD_RUNPOD_FALLBACK_MIN=0 vexpect "--wait: no Vast offer for the stretch -> Runpod backup" 0 "Pod podnew created" -- "${VUP[@]}" --wait --wait-hours 0.01 --no-bootstrap
has "the fallback is announced as the owner's backup rule" "no Vast offer for 5090 in 0 min: work is blocked, so trying Runpod as the backup"
if grep -q '"event":"up-fallback-runpod"' "$UOR_POD_STATE/ledger.jsonl" && grep -q 'runpodctl pod create' "$FAKE/calls"; then ok "the fallback is in the ledger and created on Runpod"; else bad "the fallback is in the ledger and created on Runpod"; fi
vfresh
UOR_POD_DRY_RUN=0 FAKE_ALLOW_CREATE=1 UOR_POD_WAIT_INTERVAL=1 vexpect "--wait shorter than the stretch never touches Runpod" 1 "no 5090 stock after waiting" -- "${VUP[@]}" --wait --wait-hours 0.001 --no-bootstrap
if ! grep -q 'runpodctl pod create' "$FAKE/calls" && ! grep -q 'up-fallback-runpod' "$UOR_POD_STATE/ledger.jsonl"; then ok "no Runpod pod before the stretch"; else bad "no Runpod pod before the stretch"; fi
# a rental whose reply does not parse is found by its label: no second rental
vfresh; VO RTX_5090 1 "$OFF5"
FAKE_VAST_CREATE_GARBLED=1 vexpect "an unparsed rental is found by its label, not repeated" 1 "no contract in the reply, but instance\\(s\\) 9001 labelled uor-claude-[0-9-]+ exist and bill" -- up --lab claude --session vgarb --gpu 5090 --count 1 --purpose vast --hours 1 --ref "$MAIN_SHA"
if [ "$(grep -c '^vastai create instance' "$FAKE/calls")" = 1 ]; then ok "exactly one create call"; else bad "exactly one create call"; fi
has "the refusal says how to destroy it" "echo y \\| vastai destroy instance 9001"
# an empty listing (exit 0) is a failed listing: up stops, and a garbled rental
# followed by an empty listing cannot be ruled out, so up stops before renting again
FAKE_VAST_LIST_EMPTY=1 vexpect "up: an empty listing stops up (caps unknowable)" 1 "Vast: the instance listing failed, so the caps cannot be checked" -- "${VUP[@]}"
vfresh; VO RTX_5090 1 "$OFF5"; rm -f "$FAKE/vast-created"
FAKE_VAST_CREATE_GARBLED=1 FAKE_VAST_LIST_EMPTY=after-create vexpect "an unparsed rental with an empty listing stops up" 1 "no contract in the reply and the instance listing failed, so a rental cannot be ruled out" -- up --lab claude --session vgarb2 --gpu 5090 --count 1 --purpose vast --hours 1 --ref "$MAIN_SHA"
if [ "$(grep -c '^vastai create instance' "$FAKE/calls")" = 1 ]; then ok "still exactly one create call"; else bad "still exactly one create call"; fi
# prune-stopped never destroys a stopped Vast pod's only copy of /workspace
echo '[{"id": 9001, "label": "uor-claude-x", "num_gpus": 1, "dph_total": 0.1, "intended_status": "stopped", "actual_status": "exited"}]' > "$FAKE/vast-instances.json"
vexpect "prune-stopped skips a stopped Vast pod" 0 "a Vast pod's disk holds its only copy of /workspace" -- prune-stopped --yes --older-than 0m
if jq -e 'length == 1' "$FAKE/vast-instances.json" >/dev/null; then ok "the stopped Vast pod is kept"; else bad "the stopped Vast pod is kept"; fi
# down --to-store: the pod uploads its results with /root/uor-store, no laptop copy
vfresh; VO RTX_5090 1 "$OFF5"
vexpect "Vast up for the --to-store case" 0 "Ready: ssh" -- up --lab claude --session vstore --gpu 5090 --count 1 --purpose vast --hours 1 --ref "$MAIN_SHA"
if grep -q 'cat > /root/uor-store && chmod 755 /root/uor-store' "$FAKE/calls"; then ok "up installs uor-store on the pod"; else bad "up installs uor-store on the pod"; fi
mkdir -p "$FAKE/vast-ws/uor-r4/jobs/claude"; echo r > "$FAKE/vast-ws/uor-r4/jobs/claude/r.log"
FAKE_STORE_TRUNC=1 FAKE_VAST_KB=$((9 * 1024 * 1024)) vexpect "down --to-store aborts when the store read-back is short" 1 "READ-BACK FAILED.*pod retained" -- down vast-9001 --lab claude --session vstore --to-store
if jq -e 'any(.[]; .id == 9001)' "$FAKE/vast-instances.json" >/dev/null; then ok "the pod is retained after a short listing"; else bad "the pod is retained after a short listing"; fi
FAKE_STORE_LATE_FAIL=1 FAKE_VAST_KB=$((9 * 1024 * 1024)) vexpect "down --to-store aborts when the store listing fails after rows" 1 "read-back listing .*failed.*pod retained" -- down vast-9001 --lab claude --session vstore --to-store
if jq -e 'any(.[]; .id == 9001)' "$FAKE/vast-instances.json" >/dev/null; then ok "the pod is retained after a late listing failure"; else bad "the pod is retained after a late listing failure"; fi
if jq -e 'any(.[]; .id == 9001)' "$FAKE/vast-instances.json" >/dev/null; then ok "the pod is retained after a failed read-back"; else bad "the pod is retained after a failed read-back"; fi
FAKE_VAST_KB=$((9 * 1024 * 1024)) vexpect "down --to-store archives even above the laptop cap" 0 "read-back verified 1 files" -- down vast-9001 --lab claude --session vstore --to-store
if grep -q '/root/uor-store put /workspace/uor-r4/jobs vast-archive/vast-9001/uor-r4/jobs' "$FAKE/calls" && [ ! -d "$UOR_POD_STATE/vast-archive/vast-9001" ]; then
  ok "the pod uploads; nothing is copied to the laptop"; else bad "the pod uploads; nothing is copied to the laptop"; fi
if grep -q 'uor-store get vast-archive/vast-9001/uor-r4/jobs.manifest.sha256' "$FAKE/calls"; then ok "the upload is read back before down"; else bad "the upload is read back before down"; fi
if grep '"event":"vast-archive"' "$UOR_POD_STATE/ledger.jsonl" | tail -1 | jq -e '.dest == "hf:vast-archive/vast-9001" and .verified == true and .files == 1' >/dev/null; then ok "the ledger records a verified archive"; else bad "the ledger records a verified archive"; fi
rm -rf "$FAKE/vast-ws"
# the instance key Vast returns on create never leaks
if ! grep -rq 'SENTINEL' "$VSEEN" "$UOR_POD_STATE" "$FAKE/calls"; then ok "the instance key and tokens never reach output, ledger, state or a command line"
else grep -rl 'SENTINEL' "$VSEEN" "$UOR_POD_STATE" "$FAKE/calls" | sed 's/^/     /'; bad "the instance key and tokens never reach output, ledger, state or a command line"; fi
if ! grep -q 'hf_FAKETOKEN123' "$VSEEN" "$FAKE/calls"; then ok "the HF token value is in neither the output nor a command line"; else bad "the HF token value is in neither the output nor a command line"; fi
unset FAKE_ALLOW_CREATE

echo "passed $pass, failed $fail"
[ "$fail" = 0 ]
