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
unset UOR_POD_SESSION UOR_POD_MAX_PODS UOR_POD_MAX_RATE
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
  "pod list") cat "$FAKE/pods.json";;
  "pod get") jq --arg i "$3" '.[] | select(.id == $i) | {networkVolumeId: "lmd1pfah3y"} + .
     + (if .desiredStatus == "RUNNING" then {ssh: {ip: "10.0.0.1", port: ({"poda":1001,"podb":1002,"podc":1003,"podnew":1004}[$i])}} else {} end)' "$FAKE/pods.json";;
  "gpu list") cat "$FAKE/gpus.json";;
  "network-volume list") echo '[{"id":"lmd1pfah3y","name":"uor-shared-EUR-NO-1","dataCenterId":"EUR-NO-1","size":200}]';;
  # FAKE_ALLOW_CREATE=1 (only the non-dry placement tests at the end): pod create
  # succeeds unless the datacenter is listed in $FAKE/nostock (Runpod's
  # out-of-stock error) or $FAKE/create-error exists (any other error).
  "pod create")
    [ "${FAKE_ALLOW_CREATE:-0}" = 1 ] || { echo "REAL MUTATION CALLED IN DRY RUN" >&2; exit 9; }
    dc='' prev=''; for a in "$@"; do [ "$prev" = --data-center-ids ] && dc=$a; prev=$a; done
    if grep -qx "$dc" "$FAKE/nostock" 2>/dev/null; then
      echo "Error: There are no longer any instances available with the requested specifications. Please refresh and try again." >&2; exit 1
    fi
    [ ! -f "$FAKE/create-error" ] || { echo "Error: template h15vb984sw not found" >&2; exit 1; }
    jq '. + [{"id":"podnew","name":"new","desiredStatus":"RUNNING","gpuCount":2,"costPerHr":1.98,"uptimeSeconds":0}]' "$FAKE/pods.json" > "$FAKE/pods.json.new"
    mv "$FAKE/pods.json.new" "$FAKE/pods.json"
    echo '{"id":"podnew"}';;
  "network-volume create")
    [ "${FAKE_ALLOW_CREATE:-0}" = 1 ] || { echo "REAL MUTATION CALLED IN DRY RUN" >&2; exit 9; }
    echo '{"id":"volnew"}';;
  "pod delete")
    [ "${FAKE_ALLOW_CREATE:-0}" = 1 ] || { echo "REAL MUTATION CALLED IN DRY RUN" >&2; exit 9; }
    echo '{}';;
  *) echo "fake runpodctl: unhandled $*" >&2; exit 1;;
esac
S
cat > "$W/bin/ssh" <<'S'
#!/usr/bin/env bash
port=''; while [ $# -gt 0 ]; do case $1 in -p) port=$2; shift 2;; -i|-o) shift 2;; root@*) shift; break;; *) shift;; esac; done
echo "ssh $port $*" >> "$FAKE/calls"
case "$*" in *probe_ok*) cat "$FAKE/probe-$port" 2>/dev/null;; esac
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
expect "up (dry) names the next datacenter to try on a stock error" 0 "if EU-RO-1 reports no instances available, try next" -- up "${X1[@]}" --purpose x --hours 1 --count 2
# helper files are checked before anything is created: a copy of the tool with
# one helper missing, empty or broken must stop before any create/volume call
PF=$W/pf; mkdir -p "$PF"; cp "$HERE/../uor-pod" "$HERE/../uor-pod-bootstrap.sh" "$HERE/../hot-set.txt" "$PF/"
pf_up() {  # NAME PATTERN [extra up args]
  local name=$1 pat=$2 rc=0 n0; shift 2
  n0=$(wc -l < "$FAKE/calls")
  "$PF/uor-pod" up "${X1[@]}" --purpose x --hours 1 --count 2 "$@" > "$W/out" 2>&1 || rc=$?
  if [ "$rc" = 1 ] && grep -qE -e "$pat" "$W/out" && ! grep -qE 'Creating|pod create|network-volume create' "$W/out" &&
     ! tail -n "+$((n0 + 1))" "$FAKE/calls" | grep -qE 'pod create|network-volume|gpu list|pod list'; then ok "$name"
  else echo "     rc=$rc pattern=$pat"; bad "$name"; fi
}
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
UP=(up "${X1[@]}" --purpose fallback --hours 1 --count 2 --no-bootstrap --ref "$SHA40")
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
    build_into() {  # stub: a parity-PASS build after STUB_BUILD_S seconds
      echo build >> "$BINROOT/builds"
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
boot "both pods finish: the first PASS wins, the second is discarded" "another pod published $CB first \\(parity PASS\\); discarding" \
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

echo "passed $pass, failed $fail"
[ "$fail" = 0 ]
