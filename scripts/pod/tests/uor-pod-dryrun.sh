#!/usr/bin/env bash
# Dry-run test of scripts/pod/uor-pod (leases per lab+session, up placement and
# caps, down, reap, register/keep, prune-stopped, status) against a fake
# runpodctl, ssh and gh on PATH (no network, no cost):
#
#   scripts/pod/tests/uor-pod-dryrun.sh
#
# The fake account holds pod A (2 x 5090), pod B (1 x 4090, idle, unleased,
# reaper idle for 30 min), pod C (1 x 4090, busy, unleased, non-canonical
# volume), pod D and pod E (stopped; E for 3 days). Stock: 5090 none in
# EUR-NO-1 but Low in EU-RO-1 (no uor-shared volume there yet); 4090 Low in EUR-NO-1.
set -euo pipefail

HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
TOOL=$HERE/../uor-pod
W=$(mktemp -d "${TMPDIR:-/tmp}/uor-pod-test.XXXXXX")
trap 'rm -rf "$W"' EXIT
mkdir -p "$W/bin" "$W/fake"
export UOR_POD_STATE=$W/state UOR_POD_DRY_RUN=1 FAKE=$W/fake PATH="$W/bin:$PATH"
unset UOR_POD_SESSION UOR_POD_MAX_PODS UOR_POD_MAX_RATE
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
  "pod list") cat "$FAKE/pods.json";;
  "pod get") jq --arg i "$3" '.[] | select(.id == $i) | {networkVolumeId: "lmd1pfah3y"} + .
     + (if .desiredStatus == "RUNNING" then {ssh: {ip: "10.0.0.1", port: ({"poda":1001,"podb":1002,"podc":1003}[$i])}} else {} end)' "$FAKE/pods.json";;
  "gpu list") cat "$FAKE/gpus.json";;
  "network-volume list") echo '[{"id":"lmd1pfah3y","name":"uor-shared-EUR-NO-1","dataCenterId":"EUR-NO-1","size":200}]';;
  "network-volume create"|"pod delete"|"pod create") echo "REAL MUTATION CALLED IN DRY RUN" >&2; exit 9;;
  *) echo "fake runpodctl: unhandled $*" >&2; exit 1;;
esac
S
cat > "$W/bin/ssh" <<'S'
#!/usr/bin/env bash
port=''; while [ $# -gt 0 ]; do case $1 in -p) port=$2; shift 2;; -i|-o) shift 2;; root@*) shift; break;; *) shift;; esac; done
echo "ssh $port $*" >> "$FAKE/calls"
case "$*" in *probe_ok*) cat "$FAKE/probe-$port" 2>/dev/null;; esac
exit 0
S
cat > "$W/bin/gh" <<'S'
#!/usr/bin/env bash
echo "gh $*" >> "$FAKE/calls"; echo "REAL GH CALLED IN DRY RUN" >&2; exit 9
S
chmod +x "$W/bin/"*

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
expect "default caps are 4 pods / \$8/h" 0 "caps: <= 4 running pods, <= \\\$8.00/h" -- status
UOR_POD_MAX_PODS=3 expect "cap reached names the free GPUs to lease" 4 "cap reached \\(3 of 3 pods\\) — free GPUs exist" -- up "${X1[@]}" --purpose x --hours 1 --count 2
has "cap message lists podb gpu 0" "podb gpus 0: free"
UOR_POD_MAX_PODS=3 UOR_POD_MAX_RATE=99 expect "up --test pods bypass the pod cap" 0 "--name uor-test-codex-" -- up "${X1[@]}" --purpose x --hours 1 --count 2 --test --gpu 4090
export UOR_POD_MAX_PODS=9 UOR_POD_MAX_RATE=99
expect "up: no 5090 in EUR-NO-1 -> EU-RO-1, never 4090" 0 "Creating 2 x 5090 .* in EU-RO-1 volume dryrunvolume \\(non-canonical\\) for codex/x1" -- up "${X1[@]}" --purpose x --hours 1 --count 2
expect "up: creates uor-shared-EU-RO-1 lazily" 0 "network-volume create --name uor-shared-EU-RO-1 --size 100 --data-center-id EU-RO-1" -- up "${X1[@]}" --purpose x --hours 1 --count 2
expect "up --gpu 4090 explicit -> EUR-NO-1 canonical" 0 "Creating 2 x 4090 .* in EUR-NO-1 for codex/x1" -- up "${X1[@]}" --gpu 4090 --purpose x --hours 1 --count 2
UOR_POD_VOLUME_DCS="EUR-NO-1 EUR-IS-1" expect "up: no 5090 in volume DCs refuses (no silent fallback)" 1 "no 5090 stock in EUR-NO-1 EUR-IS-1.*--wait" -- up "${X1[@]}" --purpose x --hours 1 --count 2
has "refusal prints the stock table" "4090 \\\$0.74: EUR-NO-1\\*=Low"
UOR_POD_VOLUME_DCS="EUR-NO-1 EUR-IS-1" expect "up --wait retries every 5 min" 0 "would retry every 5 min" -- up "${X1[@]}" --purpose x --hours 1 --count 2 --wait --wait-hours 1
UOR_POD_VOLUME_DCS="EUR-NO-1 EUR-IS-1" expect "up --allow-off-volume places 5090 in EU-RO-1 without volume" 0 "Creating 2 x 5090 .* in EU-RO-1 OFF-VOLUME" -- up "${X1[@]}" --purpose x --hours 1 --count 2 --allow-off-volume
UOR_POD_MAX_RATE=4 expect "up over rate cap refused" 1 "exceeds \\\$4/h" -- up "${X1[@]}" --purpose x --hours 1 --count 3
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

echo "passed $pass, failed $fail"
[ "$fail" = 0 ]
