#!/usr/bin/env bash
# drive.sh STEP POD: the laptop side of claude-400 (D87).
#   note POD RATE_CENTS OFFER  record a lease (time now, $/h in whole cents, Vast offer id) in lease.env;
#                              a re-lease carries the earlier pods' cents and the first lease time, so the
#                              guard's $1,100 and 60 h stops count every pod of the run
#   put POD                    copy the instrument, pod.sh, bank and receipt inputs to the pod, and restore
#                              every finished file of $LOCAL/out (a re-leased pod resumes from them); it
#                              refuses a pod whose out/ is not empty, and checks each restored file by sha256
#   offer POD                  right after up: check the rented Vast instance (4 or 8 GPUs, >= 1000 GB RAM per replica, NVLink
#                              >= 400 GB/s, datacenter) and save it to offer.json; exit 1 on a mismatch
#   machine POD                write machine.json from the lease and the pod's gpus.csv, and push it
#   sync POD [final]           copy finished shards/counts off the pod, checked by sha256 (a finished shard
#                              already on the laptop is never overwritten); "final" checks the logs too
#   hold POD                   tell the guard a vLLM restart is under way (counts as busy for 2 h)
#   wait-segment POD           block until the current run segment has stopped (tells the driver)
#   watch POD                  the guard (L-004), started detached right after the lease; see guard_help
#   pod POD STEP...            run a pod.sh step on the pod
guard_help='The guard syncs every 10 min. It stops the run and downs the pod at $1,100 or 60 h (all pods).
It downs the pod after a verified final sync once no run has been alive for 30 min (3 h before the
first segment). It downs it after 30 min unreachable. It never exits while the pod is up: a failed
sync or down is retried every 5 min with an alert (stdout, ALERTS file, desktop notification).'
set -euo pipefail
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT=$(cd "$HERE/../../.." && pwd)
UP=${UOR_POD:-$ROOT/scripts/pod/uor-pod}
# uor-pod reaches Vast pods (ssh, renew, down) only through the Rust adapter, which is built in the
# main checkout's local/ (a worktree has none): without it every pod call dies (16 Nov, pod 1).
if [ -z "${UOR_POD_VAST_BIN:-}" ]; then
  _main=$(cd "$(git -C "$ROOT" rev-parse --git-common-dir 2>/dev/null)/.." 2>/dev/null && pwd || true)
  export UOR_POD_VAST_BIN=${_main:-$ROOT}/local/pod-provider-target/debug/uor-r4-pod-provider
fi
LOCAL=${M2_LOCAL:-$ROOT/local/m2-run}
W=${M2_POD_W:-/workspace/m2}
CAP_CENTS=${M2_CAP_CENTS:-110000}
CAP_SECONDS=${M2_CAP_SECONDS:-216000}
GRACE=${M2_GRACE:-1800}
GRACE_FIRST=${M2_GRACE_FIRST:-10800}
EVERY=${M2_EVERY:-600}
RETRY=${M2_RETRY:-300}
UNREACH_MAX=${M2_UNREACH_MAX:-1800}
STOP_WAIT=${M2_STOP_WAIT:-1800}
KILL_WAIT=${M2_KILL_WAIT:-60}
POLL=${M2_POLL:-60}
LOGS='rate.jsonl segments.jsonl dropped.jsonl metrics.json'
step=${1:?usage: drive.sh note|put|machine|sync|hold|wait-segment|watch|pod POD ...}; POD=${2:?POD}; shift 2
ssh_pod() { "$UP" ssh "$POD" "$@"; }
# log and alert never fail: under set -e a failed write (a full disk) must not kill the guard
# while the pod is up.
log() { echo "$(date -u +%FT%TZ) $*" || true; }
alert() {
  log "ALERT: $*"
  { mkdir -p "$LOCAL" && echo "$(date -u +%FT%TZ) $POD $*" >> "$LOCAL/ALERTS"; } 2>/dev/null || true
  command -v osascript >/dev/null && osascript -e "display notification \"$POD: $*\" with title \"claude-400 guard\"" >/dev/null 2>&1 || true
}

# ALIVE, HOLD, DEAD or UNREACH; an SSH failure is retried three times, 30 s apart, before UNREACH.
probe() {
  local s
  for _ in 1 2 3; do
    s=$(ssh_pod "if kill -0 \$(cat $W/run.pid 2>/dev/null) 2>/dev/null; then echo ALIVE; elif [ -n \"\$(find $W/HOLD -mmin -120 2>/dev/null)\" ]; then echo HOLD; else echo DEAD; fi" 2>/dev/null | tail -1 || true)
    case $s in ALIVE|HOLD|DEAD) echo "$s"; return;; esac
    sleep 30
  done
  echo UNREACH
}

# 0: synced and verified. 1: a mismatch. 2: no trustworthy listing (transport, or an empty final list
# after a run started). 3: a restore is in progress on the pod (nothing is copied).
do_sync() {
  local final=${1:-} pat='shard-*.jsonl counts-*.jsonl' raw list
  mkdir -p "$LOCAL/out"
  [ "$final" = final ] && pat="$pat $LOGS"
  # The remote prints its flags, the sums of final files only (.part files are still being written),
  # then END; a listing without END is not trusted.
  raw=$(ssh_pod "cd $W/out 2>/dev/null || { echo NOOUT; echo END; exit 0; }; [ -e .restoring ] && echo RESTORING; [ -e $W/run.pid ] && echo RAN; sha256sum $pat 2>/dev/null | grep -v '\.part' || true; echo END" 2>/dev/null) || { log "sync: listing failed (ssh)"; return 2; }
  [ "$(echo "$raw" | tail -1)" = END ] || { log "sync: listing has no END"; return 2; }
  echo "$raw" | grep -qx RESTORING && { log "sync: restore in progress, skipped"; return 3; }
  list=$(echo "$raw" | grep -v -x -E 'END|NOOUT|RAN|RESTORING' || true)
  if [ "$final" = final ] && [ -z "$list" ] && echo "$raw" | grep -qx RAN; then log "sync: empty final listing after a run started"; return 2; fi
  local need=() bad=0 sum name
  while read -r sum name; do
    [ -n "$name" ] || continue
    case $name in
    shard-*|counts-*)
      # Finished shards are immutable: copy a missing one; a differing local copy is a mismatch, never overwritten.
      if [ ! -f "$LOCAL/out/$name" ]; then need+=("$name")
      elif [ "$(shasum -a 256 "$LOCAL/out/$name" | cut -d' ' -f1)" != "$sum" ]; then log "MISMATCH (kept local) $name"; bad=1; fi ;;
    *) need+=("$name") ;;
    esac
  done <<< "$list"
  # Copies land in .incoming/ and move into out/ only once their sha256 matches the listing, so a
  # stream cut mid-file never leaves a file under its final name. Each sync has its own staging
  # folder, so overlapping syncs (guard, wait-segment, by hand) never swap each other's files.
  local inc copyfail=0
  inc=$(mktemp -d "$LOCAL/out/.incoming.XXXXXX")
  if [ ${#need[@]} -gt 0 ]; then
    ssh_pod "cd $W/out && tar cf - ${need[*]}" | tar xf - -C "$inc" || { log "sync: copy failed"; copyfail=1; }
  fi
  for name in "${need[@]+"${need[@]}"}"; do
    sum=$(echo "$list" | awk -v n="$name" '$2 == n {print $1}')
    if [ -f "$inc/$name" ] && [ "$(shasum -a 256 "$inc/$name" | cut -d' ' -f1)" = "$sum" ]; then
      mv -f "$inc/$name" "$LOCAL/out/$name"
    else
      log "not verified, not kept: $name"; bad=1
    fi
  done
  # Between final syncs the logs are still growing, so no listed sum fits them: each copy replaces the
  # local one only when it is whole (a .jsonl ends in a newline, metrics.json parses).
  if [ "$final" != final ]; then
    ssh_pod "cd $W/out && tar cf - $LOGS 2>/dev/null || true" | tar xf - -C "$inc" 2>/dev/null || true
    for name in $LOGS; do
      [ -s "$inc/$name" ] || continue
      case $name in
      *.jsonl) [ -z "$(tail -c 1 "$inc/$name")" ] || { log "log copy cut short, old kept: $name"; continue; } ;;
      *.json) jq -e . "$inc/$name" > /dev/null 2>&1 || { log "log copy unreadable, old kept: $name"; continue; } ;;
      esac
      mv -f "$inc/$name" "$LOCAL/out/$name"
    done
  fi
  rm -rf "$inc"
  [ "$copyfail" = 1 ] && return 2
  log "synced${final:+ ($final)} $(echo "$list" | grep -c . || true) files (${#need[@]} copied), mismatches=$bad; $(tail -1 "$LOCAL/out/rate.jsonl" 2>/dev/null || true)"
  return $bad
}

pod_down() {
  "$UP" down "$POD" --lab claude --session claude-400 --confirm-archived || return 1
  echo "M2_DOWN_EPOCH=$(date +%s)" >> "$LOCAL/lease.env"
}

# Stop the run if it is still alive: STOP, then TERM and KILL after STOP_WAIT.
stop_run() {
  ssh_pod "touch $W/out/STOP" || true
  local t=0
  while [ "$(probe)" = ALIVE ] && [ "$t" -lt "$STOP_WAIT" ]; do sleep "$POLL"; t=$((t + POLL)); done
  if [ "$(probe)" = ALIVE ]; then
    alert "run still alive ${t}s after STOP: TERM, then KILL"
    ssh_pod "kill -TERM \$(cat $W/run.pid); sleep $KILL_WAIT; kill -KILL \$(cat $W/run.pid) 2>/dev/null; true" || true
  fi
}

# Keep the uor-pod lease live (leases last at most 24 h), so the reaper never takes the pod during a
# GPU-idle stretch such as an engine restart. A failure is alerted and retried on the next pass.
RENEW_EVERY=${M2_RENEW_EVERY:-3600}; last_renew=0
renew_lease() {
  local now; now=$(date +%s)
  [ $((now - last_renew)) -ge "$RENEW_EVERY" ] || return 0
  if "$UP" renew "$POD" --lab claude --session claude-400 --hours 24 > /dev/null 2>&1; then last_renew=$now
  else alert "lease renew failed; retrying next pass"; fi
}

# A verified final sync, then down. It never returns while the pod is up. Two bounded exits skip
# the final sync: the pod unreachable for UNREACH_MAX in a row (the last verified sync holds the
# shards), and a restore marker that stays with no run (out/ then holds only laptop copies, since
# pod.sh run refuses to start while .restoring exists). Codes 1 and 2 on a reachable pod keep it up
# with an alert every RETRY, past the caps: the sheet's stop rules name this state.
finish() {
  local why=$1 rc unreach=0 stale=0
  while :; do
    renew_lease
    rc=0; do_sync final || rc=$?
    if [ "$rc" = 0 ]; then
      log "final sync verified ($why); down"
      if pod_down; then log "pod down ($why)"; exit 0; fi
      alert "down failed after a verified sync ($why); retrying every ${RETRY}s"
    elif [ "$rc" = 2 ] && [ "$(probe)" = UNREACH ]; then
      unreach=$((unreach + RETRY))
      if [ "$unreach" -ge "$UNREACH_MAX" ]; then
        alert "unreachable ${unreach}s during finish: down without a final sync (the last verified sync holds the shards)"
        if pod_down; then exit 3; fi
      fi
    elif [ "$rc" = 3 ] && [ "$(probe)" != ALIVE ]; then
      unreach=0; stale=$((stale + 1))
      if [ "$stale" -ge 3 ]; then
        alert "restore marker stayed with no run ($stale tries): out/ holds only laptop copies; down"
        if pod_down; then exit 4; fi
      fi
    else
      unreach=0
      alert "final sync not verified (code $rc, $why); pod kept up past the caps; retrying every ${RETRY}s"
    fi
    sleep "$RETRY"
  done
}

case $step in
note)
  RATE=${1:?RATE_CENTS}; OFFER=${2:?OFFER}
  [[ $RATE =~ ^[0-9]{3,5}$ ]] || { echo "RATE_CENTS must be whole cents per hour, e.g. 2308" >&2; exit 1; }
  mkdir -p "$LOCAL"
  now=$(date +%s); prior=0; first=$now
  if [ -f "$LOCAL/lease.env" ]; then
    # shellcheck disable=SC1091
    . "$LOCAL/lease.env"
    end=${M2_DOWN_EPOCH:-$now}
    prior=$(( ${M2_PRIOR_CENTS:-0} + (end - M2_LEASE_EPOCH) * M2_RATE_CENTS / 3600 ))
    first=${M2_FIRST_LEASE_EPOCH:-$M2_LEASE_EPOCH}
    mv "$LOCAL/lease.env" "$LOCAL/lease-$M2_POD.env"
  fi
  printf 'M2_POD=%s\nM2_LEASE_EPOCH=%s\nM2_RATE_CENTS=%s\nM2_OFFER=%s\nM2_PRIOR_CENTS=%s\nM2_FIRST_LEASE_EPOCH=%s\n' \
    "$POD" "$now" "$RATE" "$OFFER" "$prior" "$first" > "$LOCAL/lease.env"
  cat "$LOCAL/lease.env"
  ;;
put)
  # The pod says EMPTY, RESTORING (an earlier put failed: out/ holds only laptop copies) or NONEMPTY;
  # anything else (an SSH failure) refuses, so put never runs blind against a pod with real work.
  st=$(ssh_pod "if [ -z \"\$(ls -A $W/out 2>/dev/null)\" ]; then echo EMPTY; elif [ -e $W/out/.restoring ]; then echo RESTORING; else echo NONEMPTY; fi" 2>/dev/null | tail -1 || true)
  case $st in
  EMPTY) ;;
  RESTORING) echo "pod $W/out holds a failed restore: clearing it" >&2; ssh_pod "rm -rf $W/out" ;;
  NONEMPTY) echo "pod $W/out is not empty: refusing" >&2; exit 1 ;;
  *) echo "could not read pod $W/out (got '$st'): refusing" >&2; exit 1 ;;
  esac
  T=$(mktemp -d "${TMPDIR:-/tmp}/m2put.XXXX")
  mkdir -p "$T/instrument"
  cp -R "$HERE/instrument/Cargo.toml" "$HERE/instrument/Cargo.lock" "$HERE/instrument/src" "$T/instrument/"
  cp "$HERE/pod.sh" "$T/"
  for f in bank.jsonl bank.jsonl.manifest.json model.json machine.json engine.json; do
    if [ -f "$LOCAL/$f" ]; then cp "$LOCAL/$f" "$T/"; fi
  done
  tar cf - -C "$T" . | ssh_pod "mkdir -p $W && tar xf - -C $W && chmod +x $W/pod.sh && ls $W"
  rm -rf "$T"
  # Restore finished work so a re-leased pod resumes instead of starting the bank again. The pod's
  # .restoring marker keeps the guard from syncing a half-extracted file meanwhile.
  files=()
  for f in "$LOCAL"/out/shard-*.jsonl "$LOCAL"/out/counts-*.jsonl; do [ -f "$f" ] && files+=("$(basename "$f")"); done
  for f in $LOGS; do [ -f "$LOCAL/out/$f" ] && files+=("$f"); done
  if [ ${#files[@]} -gt 0 ]; then
    (cd "$LOCAL/out" && shasum -a 256 "${files[@]}") > "$LOCAL/restore.sha256"
    # On any failure from here the pod's out/ (copies of laptop files only) is removed, marker and
    # all, so the guard is never blocked by .restoring and put can simply be run again.
    trap 'ssh_pod "rm -rf $W/out" || true; echo "restore failed: pod out/ removed; run put again" >&2' EXIT
    ssh_pod "mkdir -p $W/out && touch $W/out/.restoring"
    (cd "$LOCAL/out" && tar cf - "${files[@]}") | ssh_pod "tar xf - -C $W/out"
    # Every restored file must read back "OK" on the pod; anything else stops before a run starts.
    got=$(ssh_pod "cd $W/out && sha256sum -c -" < "$LOCAL/restore.sha256" 2>&1 || true)
    ok=$(echo "$got" | grep -c ': OK$' || true)
    [ "$ok" = "${#files[@]}" ] || { echo "$got" | grep -v ': OK$' >&2; echo "restore check FAILED: $ok of ${#files[@]} OK" >&2; exit 1; }
    ssh_pod "rm -f $W/out/.restoring"
    trap - EXIT
    log "restored ${#files[@]} files to $W/out, sha256 OK"
  fi
  ;;
offer)
  # The sheet's lease filters that `uor-pod up` does not apply itself: RAM for one Engram host copy
  # and an NVLink bridge across all four GPUs for TP4. Vast reports cpu_ram in MB.
  mkdir -p "$LOCAL"
  "${VASTAI:-vastai}" show instance "${POD#vast-}" --raw > "$LOCAL/offer.json"
  jq -c '{id, machine_id, host_id, gpu_name, num_gpus, cpu_ram, bw_nvlink, gpu_mem_bw, hosting_type, dph_total, geolocation, driver_version}' "$LOCAL/offer.json"
  if jq -e '(.num_gpus == 4 or .num_gpus == 8) and (.gpu_name | test("H200|B200")) and .cpu_ram >= 1000000 * (.num_gpus / 4) and (.bw_nvlink // 0) >= 400' "$LOCAL/offer.json" > /dev/null; then
    log "offer metadata matches the sheet"
  else
    alert "rented instance does not match the sheet's filters: down it now"; exit 1
  fi
  # A lease approved for one named host at one price (amendment 2): M2_EXPECT_MACHINE and M2_MAX_DPH.
  if [ -n "${M2_EXPECT_MACHINE:-}" ] && ! jq -e --argjson m "$M2_EXPECT_MACHINE" '.machine_id == $m' "$LOCAL/offer.json" > /dev/null; then
    alert "rented machine is not the approved machine $M2_EXPECT_MACHINE: down it now"; exit 1
  fi
  if [ -n "${M2_MAX_DPH:-}" ] && ! jq -e --argjson p "$M2_MAX_DPH" '(.dph_total // 1e9) <= $p' "$LOCAL/offer.json" > /dev/null; then
    alert "rented price is above the approved \$$M2_MAX_DPH/h: down it now"; exit 1
  fi
  # Vast's bw_nvlink is not proof: pod 1 (16 Nov, 4xH200 NVL, listed 478 GB/s) had no NVLink at
  # all. Every GPU pair in the pod's own topology must be NV#, or TP4 runs over PCIe.
  ssh_pod "nvidia-smi topo -m" > "$LOCAL/offer-topo.txt" 2>&1 || { alert "cannot read the pod's GPU topology: down it now"; exit 1; }
  # Strip colour codes; GPU rows only; any non-NV entry other than the diagonal X fails.
  if sed 's/\x1b\[[0-9;]*m//g' "$LOCAL/offer-topo.txt" | awk -v NG="$(jq -r .num_gpus "$LOCAL/offer.json")" '/^GPU[0-9]/ { for (i = 2; i <= NG + 1; i++) if ($i != "X" && $i !~ /^NV[0-9]+$/) bad = 1; n++ } END { exit (n == NG && !bad) ? 0 : 1 }'; then
    log "GPU topology: NVLink between every pair"
  else
    alert "GPU topology is not NVLink between every pair (see offer-topo.txt): down it now"; exit 1
  fi
  ;;
machine)
  # shellcheck disable=SC1091
  . "$LOCAL/lease.env"
  gpus=$(ssh_pod "cat $W/gpus.csv")
  mem=$(ssh_pod "cat $W/host-mem.txt")
  vv=$(ssh_pod "cat $W/vllm-version.txt")
  jq -n --arg pod "$M2_POD" --arg offer "$M2_OFFER" --argjson cents "$M2_RATE_CENTS" \
    --argjson lease "$M2_LEASE_EPOCH" --arg gpus "$gpus" --arg mem "$mem" --arg vv "$vv" \
    '{provider: "vast", pod: $pod, offer: $offer, rate_cents_per_hour: $cents, lease_epoch: $lease,
      gpus_csv: ($gpus | split("\n")), host_mem: ($mem | split("\n")), vllm_version: $vv}' > "$LOCAL/machine.json"
  ssh_pod "cat > $W/machine.json" < "$LOCAL/machine.json"
  jq -c '{pod, offer, gpus: (.gpus_csv | length)}' "$LOCAL/machine.json"
  ;;
sync) do_sync "${1:-}" ;;
hold) ssh_pod "touch $W/HOLD" && log "hold set: the guard counts the pod busy for 2 h (remove $W/HOLD to end it)" ;;
wait-segment)
  while :; do
    s=$(probe)
    [ "$s" = ALIVE ] || { log "segment ended ($s)"; do_sync || true; ssh_pod "tail -1 $W/out/segments.jsonl" || true; exit 0; }
    sleep 60
  done
  ;;
watch)
  echo "$guard_help"
  # shellcheck disable=SC1091
  [ -f "$LOCAL/lease.env" ] && . "$LOCAL/lease.env"
  if [ -z "${M2_LEASE_EPOCH:-}" ] || [ -z "${M2_RATE_CENTS:-}" ] || [ "${M2_POD:-}" != "$POD" ]; then
    alert "guard NOT started: $LOCAL/lease.env has no lease for $POD (run note first)"; exit 1
  fi
  if [ -z "${UOR_POD:-}" ] && [ ! -x "$UOR_POD_VAST_BIN" ]; then
    alert "guard NOT started: no Vast adapter at $UOR_POD_VAST_BIN (it could neither reach nor down the pod)"; exit 1
  fi
  first=${M2_FIRST_LEASE_EPOCH:-$M2_LEASE_EPOCH}; prior=${M2_PRIOR_CENTS:-0}
  seenf="$LOCAL/guard-seen-$POD"; seen=0; [ -f "$seenf" ] && seen=1
  idle=0; unreach=0
  while :; do
    now=$(date +%s); el=$((now - first)); spent=$((prior + (now - M2_LEASE_EPOCH) * M2_RATE_CENTS / 3600))
    renew_lease
    s=$(probe)
    log "state=$s elapsed=${el}s spent_cents=$spent idle=${idle}s"
    if [ "$spent" -ge "$CAP_CENTS" ] || [ "$el" -ge "$CAP_SECONDS" ]; then
      alert "HARD STOP: spent_cents=$spent elapsed=${el}s"
      stop_run
      finish "hard stop"
    fi
    case $s in
    ALIVE) seen=1; touch "$seenf" || true; idle=0; unreach=0; do_sync || true ;;
    HOLD) idle=0; unreach=0; do_sync || true ;;
    DEAD)
      unreach=0; do_sync || true
      g=$GRACE; [ "$seen" = 1 ] || g=$GRACE_FIRST
      [ "$idle" -ge "$g" ] && finish "no run for ${idle}s"
      idle=$((idle + EVERY)) ;;
    UNREACH)
      unreach=$((unreach + EVERY))
      if [ "$unreach" -ge "$UNREACH_MAX" ]; then
        alert "unreachable ${unreach}s: down without a final sync (the last verified sync holds the shards)"
        if pod_down; then exit 3; fi
        alert "down failed while unreachable; retrying"
      fi ;;
    esac
    sleep "$EVERY"
  done
  ;;
pod) ssh_pod "bash $W/pod.sh $*" ;;
*) echo "unknown step $step" >&2; exit 1 ;;
esac
