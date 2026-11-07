# Prep sheet: centroid refit on the current human-text checkpoint (one-model-end-to-end, deepseek-19, 6 Nov 2026)

Lab: deepseek. Item: one-model-end-to-end (#3407). The current input and execution
package are under [deepseek-19](../../loop/tasks/one-model-end-to-end/deepseek-19/ORDER.md);
DeepSeek-16 through DeepSeek-18 remain the historical n3S preparation and delivery.

TEST FITNESS: PENDING - this retarget is complete only when its exact input manifest and
job have received a fresh non-author review. No capture or evaluation is authorized until
that review writes `TEST FITNESS: FIT`.

## Question

On the current human-text champion, Claude-301's D3/16,384-step (b64L) FBS checkpoint at
seeds 1 and 2, does replacing eligible output-table rows with the mean TRAIN query for
each target improve the whole-table nearest top-1, beyond the original table and a
target-count-preserving matched-label control, without lowering top-8?

This tests the output readout only. The trunk, input query path, table cells and writer
admission remain frozen.

## Why now

Claude-301's (b64L) beat D3's T2 and unigram beyond the paired band at both seeds and on
unseen rows. Claude-302's width-1,024 attempt then stopped at registered instrument gates
before training, and its REPORT keeps (b64L) at width 256 as the current recipe. The
centroid package's previous n3S checkpoint predates that result. DeepSeek-237 rejected
median and mean-centroid rows on two older public artifacts, so the prior is negative.
This run asks whether that negative survives on the current champion and the active
scorer.

## Base artifact

The base is the current pinned FBS implementation plus the two Claude-301 (b64L)
step-016384 freeze archives listed below. The implementation pin is
`3f41847a30eb8a6020109620e525047b89c279ea`; it contains the verified D84 cache
evaluator, the DEV-omission repair and the query-cache writer, with no runtime-code
change from the previous `3a9c2f11d` pin. The input specification, capture binary and CPU
evaluator binary are all hashed in the local run record.

## Raw data

| Input | Store path | Identity |
|---|---|---|
| data root | `w2-inputs/2026-10-25/driver-data` | tokenizer `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`; DEV tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`; DEV mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| D3 TRAIN | `results/claude/scale-probe-2026-11-05/inputs/d3` | tokens `ed8f25374e99bd65cd509b5cfcdffb6a8476401a973ea02aaaaa0ea76e35ee52`; mask `ec6e520c7a67aaf6efc7aaf42bf7513b3e697239fb18b73a1e89f130eacd4875`; 27,416,554 scored |
| oasst2 heldout | `results/claude/real-text-corpus-2026-11-05/oasst2/heldout` | tokens `02a04e97bbb8936c993bf5eff0f29c171ebeae88988217a4100fdc77701f169e`; mask `9c5bb6567c6a4cded2dfb2280ae4acdc798e283a3beae8aaebc695656e1b4833` |
| wiki-00000 heldout | `results/claude/real-text-corpus-2026-11-05/wiki-00000/heldout` | tokens `82b3a40c3ffe2cabd9164f636d9c2710fb054e171913eca944ebf819b70209f7`; mask `34fe6b68e0d039b914bb1cde789223ef463b37ea77af7f3d77ab90eb0977386f` |
| wiki-00020 heldout | `results/claude/real-text-corpus-2026-11-05/wiki-00020/heldout` | tokens `9b7f1432b105f161ed8b54b01900d8fa791efc6dbfd5ad1d62a8d4d94a86d640`; mask `a6803ab3de3fae60c382416639ffa583c26ac822db78a37afd2c2d718767e00d` |
| wiki-00040 heldout | `results/claude/real-text-corpus-2026-11-05/wiki-00040/heldout` | tokens `c927243b540bccdda8af8fb5a11aafa8b9041d4af30dd614f23d2ebb9aa5962d`; mask `9d95ea6de0eba07d4cdbc258b5fa6ae88def952f062d166ba417d783724fa625` |

The four held-out roots contain exactly 1,460,697 scored rows. Their pooled identity is
`5f88a5b3295ba5d684a8c75206787642fd65a2377d2dbb268992ec5c71a621eb`.

## Data split

Training uses D3 only. Evaluation uses the four complete held-out roots. No panel row is
used to fit a centroid, and no D3 row is scored as a panel row.

### Checkpoints

The archives are:

- seed 1:
  `results/claude/one-model-end-to-end-d3-accum-2026-11-06/runs-2/s1/d3-accum-s1.tar.gz`,
  SHA-256 `f7852f2540e58c17b2311d39fb47f8e53471e2cda8d6136f81b034cdf6bf5bba`;
- seed 2:
  `results/claude/one-model-end-to-end-d3-accum-2026-11-06/runs-2/s2/d3-accum-s2.tar.gz`,
  SHA-256 `1f24f03a4b638f7ad6fafd184f980bca324de758085a62e5b5d0dfb3a4f6ab32`.

The machine-readable authority for those paths and hashes is
[`checkpoint-inputs.json`](../../labs/one-model-end-to-end/centroid-cache-2026-11-06/checkpoint-inputs.json).
It is a separate reviewed input: changing checkpoints requires changing that file, not
editing seed cases inside the capture job. Before capture, the job verifies the archive
hash, the run config, all three checkpoint files and each selected line in the archive's
inner `SHA256SUMS.files`.

| Seed | Checkpoint | File | SHA-256 |
|---|---|---|---|
| 1 | `oh/run/b64L-s1/models/step-016384` | run config | `6b2b84640e37d1f6322a5b4ab6db0d783469d7adf4db1c7e67b532d4310056f2` |
| 1 | | model config | `1045acb61d92d390f0f61617ffa4c1847437473c9694c09787b48bf46e761ed2` |
| 1 | | model | `6fd2d764171cc416d5ff272e19e3ebe0644183ab9c43ca0662706756ac75f5de` |
| 1 | | Hamming read | `672de97fcc329b10c3cde591f66047352851f9fe1d02df85a168190f5ba12ddd` |
| 2 | `oh/run/b64L-s2/models/step-016384` | run config | `f21567d0b43ba5fcc444bda840ad77bf8a5e9b46315ed8e515d223619f15e593` |
| 2 | | model config | `778145167a18ff3b1006d4f609b7f608cc528b0dd753279b3ff38c8c63a88b95` |
| 2 | | model | `13f3ae6230831ad9797224c0d20ecb6d15463fc28a3fcbea0ddfd9f0046c3306` |
| 2 | | Hamming read | `3de5f52daa5d48504c3bba757e6ede95dd6a31bcba42653c5d12a6c1c5c35c7d` |

The run and model configs together name D3, seed 1 or 2, objective `nearest`,
`train_as_served: true`, wall batch 64 as four micro-batches of 16, 16,384 steps, width
256, six layers, four Hamming heads and context 192. An archive with a different selected
checkpoint or config is a stop.

The fixed D84 evaluator reports the historical profile identifier
`n3s-registered-v1`. The input specification explicitly defines that string as a frozen
evaluation-protocol identifier, not as the checkpoint name, and binds the current target
as `b64l-centroid-v1`. The CPU result block checks the evaluator profile and writes a
result-identity sidecar carrying both the protocol identifier and target identifier.

## Steps

The reviewed pod script is run once per seed on a fresh DeepSeek-owned RTX 5090. Check
`scripts/pod/uor-pod status` first. Do not lease, stop or inspect either existing busy pod.
Two fresh one-card pods keep the run parallel and at most four total:

```sh
set -euo pipefail
export UOR_POD_MAX_RATE=4.50
JOB_SHA256=a451145ae5b508d7c79aa6e17f11cc1e301665692ab411752575595db6c57cd5
SPEC_SHA256=865497d8702d34d75182e592c16de247f3d653ad5bf52a8c9d4aa1822ad6cf71
[ "$(sha256sum docs/labs/one-model-end-to-end/centroid-cache-2026-11-06/job.sh |
  cut -d' ' -f1)" = "$JOB_SHA256" ] || {
  echo "local capture script does not match the reviewed hash" >&2
  exit 1
}
[ "$(sha256sum docs/labs/one-model-end-to-end/centroid-cache-2026-11-06/checkpoint-inputs.json |
  cut -d' ' -f1)" = "$SPEC_SHA256" ] || {
  echo "local checkpoint specification does not match the reviewed hash" >&2
  exit 1
}
SOURCE_PIN=$(jq -er .source_pin \
  docs/labs/one-model-end-to-end/centroid-cache-2026-11-06/checkpoint-inputs.json)

CONTROLLER_RUN_ID=$(date -u +%Y%m%dT%H%M%SZ)-$(od -An -N8 -tx1 /dev/urandom | tr -d ' \n')
CONTROLLER_RUN_DIR="local/deepseek/centroid-cache/2026-11-06/$CONTROLLER_RUN_ID"
mkdir -p "$(dirname "$CONTROLLER_RUN_DIR")"
mkdir "$CONTROLLER_RUN_DIR"
DISCOVERY_INDEX="$CONTROLLER_RUN_DIR/pods.tsv"
: > "$DISCOVERY_INDEX"
RUN_ID1="$CONTROLLER_RUN_ID-s1"
RUN_ID2="$CONTROLLER_RUN_ID-s2"
RUN_DIR1="/workspace/uor-r4/runs/$CONTROLLER_RUN_ID-s1"
RUN_DIR2="/workspace/uor-r4/runs/$CONTROLLER_RUN_ID-s2"
OUT_STORE_S1="results/deepseek/one-model-end-to-end-centroid-2026-11-06/s1-$RUN_ID1"
OUT_STORE_S2="results/deepseek/one-model-end-to-end-centroid-2026-11-06/s2-$RUN_ID2"
{
  printf 'export CONTROLLER_RUN_DIR=%q\n' "$CONTROLLER_RUN_DIR"
  printf 'export OUT_STORE_S1=%q\n' "$OUT_STORE_S1"
  printf 'export OUT_STORE_S2=%q\n' "$OUT_STORE_S2"
  printf 'export RUN_DIR1=%q\nexport RUN_DIR2=%q\n' "$RUN_DIR1" "$RUN_DIR2"
} > "$CONTROLLER_RUN_DIR/outputs.env"
printf 'controller_run_dir=%s\n' "$CONTROLLER_RUN_DIR"
PODS=()
SESSIONS=()
RUN_DIRS=()
STARTED=()
FINISHED=()
RESOLVED=()

teardown_pod() {
  local pod=$1 session=$2
  scripts/pod/uor-pod release "$pod" --lab deepseek --session "$session" || true
  scripts/pod/uor-pod down "$pod" --lab deepseek --session "$session" --confirm-archived
}

retain_pod() {
  local pod=$1 session=$2 reason=$3
  echo "retaining $pod: $reason" >&2
  echo "inspect it, preserve unique evidence, then unkeep and down it explicitly" >&2
  scripts/pod/uor-pod renew "$pod" --lab deepseek --session "$session" --hours 7 || true
  if ! scripts/pod/uor-pod keep "$pod" --lab deepseek \
      --purpose "centroid failure retained: $reason"; then
    echo "ERROR: could not mark $pod KEEP; its evidence is protected only until the current lease expires" >&2
    return 1
  fi
}

resolve_pod() {
  local i=$1 pod=${PODS[$1]} probe status incomplete cache_present cache_verified
  local status_present normal_present
  [ "${RESOLVED[$i]}" != 1 ] || return 0
  POD_STATUS=UNAVAILABLE
  if [ "${STARTED[$i]}" != 1 ]; then
    teardown_pod "$pod" "${SESSIONS[$i]}" || return 1
    RESOLVED[i]=1
    return 0
  fi
  if [ "${FINISHED[$i]}" != 1 ]; then
    retain_pod "$pod" "${SESSIONS[$i]}" \
      "job did not reach a verified terminal state" || return 1
    RESOLVED[i]=1
    return 0
  fi
  if ! probe=$(scripts/pod/uor-pod ssh "$pod" \
    "s=\$(cat '${RUN_DIRS[$i]}/EXIT' 2>/dev/null || printf missing); \
     e=0; test -e '${RUN_DIRS[$i]}/FAILURE_EVIDENCE_INCOMPLETE' && e=1; \
     c=0; { test -e '${RUN_DIRS[$i]}/results/cache' || \
       test -e '${RUN_DIRS[$i]}/results/cache.tar.zst'; } && c=1; \
     v=0; test -e '${RUN_DIRS[$i]}/CACHE_TRANSFER_VERIFIED' && v=1; \
     f=0; test -e '${RUN_DIRS[$i]}/status.json' && f=1; \
     n=0; test -e '${RUN_DIRS[$i]}/results/normal-outputs.tar.zst.sha256' && n=1; \
     printf '%s %s %s %s %s %s\n' \"\$s\" \"\$e\" \"\$c\" \"\$v\" \"\$f\" \"\$n\""); then
    retain_pod "$pod" "${SESSIONS[$i]}" "terminal evidence could not be read" ||
      return 1
    RESOLVED[i]=1
    return 0
  fi
  read -r status incomplete cache_present cache_verified status_present normal_present <<<"$probe"
  POD_STATUS=$status
  case $status in
    0)
      if [ "$incomplete" = 0 ] && [ "$cache_present" = 1 ] &&
        [ "$cache_verified" = 1 ] && [ "$status_present" = 1 ] &&
        [ "$normal_present" = 1 ]; then
        teardown_pod "$pod" "${SESSIONS[$i]}" || return 1
      else
        retain_pod "$pod" "${SESSIONS[$i]}" \
          "successful EXIT lacks verified cache, status or normal-output evidence" ||
          return 1
      fi
      ;;
    ''|*[!0-9]*)
      retain_pod "$pod" "${SESSIONS[$i]}" "EXIT is missing or malformed" || return 1
      ;;
    *)
      if [ "$incomplete" = 1 ]; then
        retain_pod "$pod" "${SESSIONS[$i]}" "failure evidence is incomplete" || return 1
      elif [ "$cache_present" = 1 ] && [ "$cache_verified" != 1 ]; then
        retain_pod "$pod" "${SESSIONS[$i]}" "cache transfer was not verified" || return 1
      else
        teardown_pod "$pod" "${SESSIONS[$i]}" || return 1
      fi
      ;;
  esac
  RESOLVED[i]=1
}

cleanup() {
  local rc=$? i
  trap - EXIT
  discover_up_pods || true
  for i in "${!PODS[@]}"; do
    resolve_pod "$i" || rc=1
  done
  exit "$rc"
}
trap cleanup EXIT

create_capture_pod() {
  local session=$1 seed=$2 purpose=$3 discovery="$CONTROLLER_RUN_DIR/$1.pod" pod
  printf '%s\t%s\t%s\n' "$discovery" "$session" "$seed" >> "$DISCOVERY_INDEX"
if ! scripts/pod/uor-pod up --lab deepseek --session "$session" --gpu 5090 \
    --count 1 --hours 7 --ref "$SOURCE_PIN" \
    --purpose "$purpose" 2>&1 |
    awk -v discovery="$discovery" '
      { print }
      $1 == "Pod" && $3 == "created;" {
        print $2 > discovery
        close(discovery)
      }
    '; then
    return 1
  fi
  discover_up_pods
  pod=$(sed -n '1p' "$discovery")
  [ -n "$pod" ] || {
    echo "could not parse the pod id from uor-pod up" >&2
    return 1
  }
  CREATED_POD=$pod
}

register_pod() {
  local existing
  for existing in "${PODS[@]}"; do
    [ "$existing" != "$1" ] || return 0
  done
  PODS+=("$1")
  SESSIONS+=("$2")
  case $3 in
    1) RUN_DIRS+=("$RUN_DIR1") ;;
    2) RUN_DIRS+=("$RUN_DIR2") ;;
  esac
  STARTED+=(0)
  FINISHED+=(0)
  RESOLVED+=(0)
}

discover_up_pods() {
  local discovery session seed pod
  [ -f "$DISCOVERY_INDEX" ] || return 0
  while IFS=$'\t' read -r discovery session seed; do
    [ -f "$discovery" ] || continue
    pod=$(sed -n '1p' "$discovery")
    [ -n "$pod" ] && register_pod "$pod" "$session" "$seed"
  done < "$DISCOVERY_INDEX"
}

start_capture() {
  local index=$1 pod=$2 session=$3 seed=$4 output log run_id run_dir
  case $seed in
    1) run_id=$RUN_ID1; run_dir=$RUN_DIR1 ;;
    2) run_id=$RUN_ID2; run_dir=$RUN_DIR2 ;;
  esac
  STARTED[index]=1
  output=$(scripts/pod/uor-pod run "$pod" --lab deepseek --session "$session" \
    --gpu 0 -- env UOR_CENTROID_RUN="$run_dir" UOR_CENTROID_RUN_ID="$run_id" \
    UOR_CENTROID_SPEC=/workspace/centroid-checkpoint-inputs.json \
    bash /workspace/centroid-cache-job.sh "$seed" 2>&1 |
    tee /dev/stderr) || return 1
  log=$(printf '%s\n' "$output" |
    sed -n 's/^Started on .* Log: //p' | tail -1)
  [ -n "$log" ] || {
    echo "could not parse the remote job log from uor-pod run" >&2
    return 1
  }
  CAPTURE_RUN_LOG=$log
  CAPTURE_RUN_SCRIPT=${log%.log}.sh
}

watch_capture() {
  local pod=$1 run_dir=$2 session=$3 log=$4 script=$5 now last=0 failures=0 probe
  local exit_seen run_seen live_seen status_output
  while true; do
    if probe=$(scripts/pod/uor-pod ssh "$pod" \
      "e=0; g=0; p=0; test -f '$run_dir/EXIT' && e=1; test -f '$log' && grep -q '^# exit=' '$log' && g=1; pgrep -f \"^bash $script\" >/dev/null && p=1; printf '%s %s %s\n' \"\$e\" \"\$g\" \"\$p\""); then
      read -r exit_seen run_seen live_seen <<<"$probe"
      failures=0
      case "$exit_seen:$run_seen:$live_seen" in
        1:1:*) return 0 ;;
        0:1:*)
          echo "$pod ended without writing EXIT" >&2
          return 1
          ;;
        0:0:1|1:0:1) ;;
        0:0:0|1:0:0)
          echo "$pod has no live wrapper and no terminal record" >&2
          return 1
          ;;
        *)
          echo "$pod returned an invalid watch state: $probe" >&2
          return 1
          ;;
      esac
    else
      failures=$((failures + 1))
      if [ "$failures" -ge 20 ]; then
        if status_output=$(scripts/pod/uor-pod status 2>/dev/null); then
          if grep -Eq "^$pod .* RUNNING " <<<"$status_output"; then
            echo "$pod SSH was lost for 10 minutes but the provider still lists it; retaining the lease" >&2
            failures=0
          else
            echo "$pod is no longer listed by the provider" >&2
            return 1
          fi
        else
          echo "provider status for $pod is temporarily unavailable; retaining the lease" >&2
          failures=0
        fi
      fi
    fi
    now=$(date +%s)
    if [ $((now - last)) -ge 1500 ]; then
      if ! scripts/pod/uor-pod renew "$pod" --lab deepseek --session "$session" --hours 7; then
        echo "could not renew $pod" >&2
        return 1
      fi
      last=$now
    fi
    sleep 30
  done
}

scripts/pod/uor-pod status
create_capture_pod centroid-s1 1 "b64L centroid cache seed 1"
POD1=$CREATED_POD
create_capture_pod centroid-s2 2 "b64L centroid cache seed 2"
POD2=$CREATED_POD

# Copy the exact D67-reviewed script and checkpoint specification to each pod, then
# check both recorded SHA-256 values.
scripts/pod/uor-pod ssh "$POD1" 'cat > /workspace/centroid-cache-job.sh' \
  < docs/labs/one-model-end-to-end/centroid-cache-2026-11-06/job.sh
scripts/pod/uor-pod ssh "$POD2" 'cat > /workspace/centroid-cache-job.sh' \
  < docs/labs/one-model-end-to-end/centroid-cache-2026-11-06/job.sh
scripts/pod/uor-pod ssh "$POD1" 'cat > /workspace/centroid-checkpoint-inputs.json' \
  < docs/labs/one-model-end-to-end/centroid-cache-2026-11-06/checkpoint-inputs.json
scripts/pod/uor-pod ssh "$POD2" 'cat > /workspace/centroid-checkpoint-inputs.json' \
  < docs/labs/one-model-end-to-end/centroid-cache-2026-11-06/checkpoint-inputs.json
scripts/pod/uor-pod ssh "$POD1" "printf '%s  /workspace/centroid-cache-job.sh\n' '$JOB_SHA256' | sha256sum -c -"
scripts/pod/uor-pod ssh "$POD2" "printf '%s  /workspace/centroid-cache-job.sh\n' '$JOB_SHA256' | sha256sum -c -"
scripts/pod/uor-pod ssh "$POD1" "printf '%s  /workspace/centroid-checkpoint-inputs.json\n' '$SPEC_SHA256' | sha256sum -c -"
scripts/pod/uor-pod ssh "$POD2" "printf '%s  /workspace/centroid-checkpoint-inputs.json\n' '$SPEC_SHA256' | sha256sum -c -"
scripts/pod/uor-pod ssh "$POD1" 'command -v pgrep >/dev/null'
scripts/pod/uor-pod ssh "$POD2" 'command -v pgrep >/dev/null'

start_capture 0 "$POD1" centroid-s1 1
RUN_LOG1=$CAPTURE_RUN_LOG
RUN_SCRIPT1=$CAPTURE_RUN_SCRIPT

# Watch both jobs and renew each healthy lease every 25 minutes. Returning
# requires both the job's EXIT and the uor-pod wrapper's terminal record.
watch_capture "$POD1" "$RUN_DIR1" centroid-s1 "$RUN_LOG1" "$RUN_SCRIPT1" &
WATCH1=$!
ACTIVE_WATCHES=("$WATCH1")
WATCH2=
CAPTURE_STATUS1=
CAPTURE_STATUS2=
SECOND_START_FAILED=0
if start_capture 1 "$POD2" centroid-s2 2; then
  RUN_LOG2=$CAPTURE_RUN_LOG
  RUN_SCRIPT2=$CAPTURE_RUN_SCRIPT
  watch_capture "$POD2" "$RUN_DIR2" centroid-s2 "$RUN_LOG2" "$RUN_SCRIPT2" &
  WATCH2=$!
  ACTIVE_WATCHES+=("$WATCH2")
else
  SECOND_START_FAILED=1
  echo "seed 2 did not start; seed 1 remains watched and will finish" >&2
fi
WATCH_FAILED=0
while [ "${#ACTIVE_WATCHES[@]}" -gt 0 ]; do
  FINISHED_WATCH=
  if wait -n -p FINISHED_WATCH "${ACTIVE_WATCHES[@]}"; then
    WATCH_RC=0
  else
    WATCH_RC=$?
  fi
  case $FINISHED_WATCH in
    "$WATCH1")
      if [ "$WATCH_RC" = 0 ]; then FINISHED[0]=1; else WATCH_FAILED=1; fi
      resolve_pod 0
      CAPTURE_STATUS1=$POD_STATUS
      ;;
    "$WATCH2")
      if [ "$WATCH_RC" = 0 ]; then FINISHED[1]=1; else WATCH_FAILED=1; fi
      resolve_pod 1
      CAPTURE_STATUS2=$POD_STATUS
      ;;
    *)
      echo "could not identify the completed capture watcher" >&2
      exit 1
      ;;
  esac
  remaining_watches=()
  for watch_pid in "${ACTIVE_WATCHES[@]}"; do
    [ "$watch_pid" = "$FINISHED_WATCH" ] || remaining_watches+=("$watch_pid")
  done
  ACTIVE_WATCHES=("${remaining_watches[@]}")
done
test "$SECOND_START_FAILED" = 0
test "$WATCH_FAILED" = 0
test "$CAPTURE_STATUS1" = 0
test "$CAPTURE_STATUS2" = 0
```

`JOB_SHA256` is replaced with the value recorded by the D67 review; `SPEC_SHA256` binds the
separate checkpoint input. The block copies and verifies both files on each pod, captures
both pod ids into the run record and owns teardown from the first successful `up`. A
watcher returns only after both the job's `EXIT` and the `uor-pod` wrapper's terminal
record exist; it treats a wrapper end without `EXIT` as a failure and stops renewing. On any failure,
the EXIT trap takes down pods with no started job. For a finished pod it applies this policy:
A successful result must have `EXIT 0`, `status.json`, the cache,
`CACHE_TRANSFER_VERIFIED` and the normal-output archive checksum; a failed result may
omit the cache only after complete diagnostics were uploaded and read back. Otherwise it
renews the lease, marks the pod `KEEP`, prints its id and leaves explicit
evidence-preservation instructions.
Each pod is resolved as soon as its own watcher finishes, so a completed seed does not
remain billed while its sibling runs. No self-set wall clock cancels either capture.

The script invokes `train-in-space` with `--query-cache-memory 0` and
`--query-cache-dev 0`. That yields exactly TRAIN plus the four registered panels; DEV is
still scored into the ordinary output files. The scorer may perform its normal memory-on
pass for those files, but no samples are decoded and no training occurs.

## Memory control

The cache captures:

- each query immediately before a scored target;
- target and corpus position;
- frozen whole-table nearest and writer decisions;
- opening/copy flags;
- the unchanged shared table.

TRAIN fitting uses only targets with at least eight observations. The control groups eligible
targets by exact observation count and deterministically cycles each non-singleton group once;
a singleton group maps to its own target in both arms. It therefore preserves every target's
total observation count and the global query multiset. A control that changes no target or
produces the candidate's table is `CONTROL_FAULT`.

## CPU evaluation

Build, restore both checkpoints and caches, and evaluate on the D84 host with the following
blocks. The restore loop reads the same reviewed checkpoint specification as the pod job;
there is no hand-substituted seed table.

Set `CONTROLLER_RUN_DIR` from the `controller_run_dir=` line printed by the capture block,
then load the immutable output identities it recorded before either pod started:

```sh
CONTROLLER_RUN_DIR=local/deepseek/centroid-cache/2026-11-06/<printed-id>
source "$CONTROLLER_RUN_DIR/outputs.env"
```

Do not reconstruct or reuse a timestamp-plus-random run id.

```sh
set -euo pipefail
CPU_RUN_ID=$(date -u +%Y%m%dT%H%M%SZ)-$(od -An -N8 -tx1 /dev/urandom | tr -d ' \n')
ROOT=local/deepseek/centroid-cache/2026-11-06/$CPU_RUN_ID
SPEC=docs/labs/one-model-end-to-end/centroid-cache-2026-11-06/checkpoint-inputs.json
CAPTURE_JOB_SHA256=a451145ae5b508d7c79aa6e17f11cc1e301665692ab411752575595db6c57cd5
CAPTURE_SPEC_SHA256=865497d8702d34d75182e592c16de247f3d653ad5bf52a8c9d4aa1822ad6cf71
[ "$(sha256sum "$SPEC" | cut -d' ' -f1)" = "$CAPTURE_SPEC_SHA256" ]
SOURCE_PIN=$(jq -er .source_pin "$SPEC")
EXPECTED=$(jq -c '.expected' "$SPEC")
EVAL_PROFILE=$(jq -er .evaluation.profile "$SPEC")
TARGET_ID=$(jq -er .evaluation.target_id "$SPEC")
mkdir -p "$(dirname "$ROOT")"
mkdir "$ROOT"
printf 'cpu_run_root=%s\n' "$ROOT"
available_gib=$(df -Pk "$ROOT" | awk 'NR==2 {print int($4 / 1048576)}')
[ "$available_gib" -ge 170 ] || { echo "need 170 GiB free, have $available_gib" >&2; exit 1; }
export CARGO_TARGET_DIR="$ROOT/target"
CPU_SOURCE_PATHS=(
  crates/uor-r4-cpu-train crates/uor-r4-core crates/uor-r4-discrete
  crates/uor-r4-integer crates/uor-r4-source-interface crates/uor-r4-tokenizer
  crates/uor-r4-vision third_party Cargo.toml Cargo.lock .cargo rust-toolchain.toml
)
test -z "$(git status --porcelain --untracked-files=all -- \
  "${CPU_SOURCE_PATHS[@]}")"
git diff --quiet "$SOURCE_PIN" HEAD -- "${CPU_SOURCE_PATHS[@]}"
cargo build --release --locked -p uor-r4-cpu-train \
  --bin cpu-query-cache-evaluate --bin cpu-query-cache-aggregate
{
  printf 'base=%s\nhead=%s\n' \
    "$SOURCE_PIN" "$(git rev-parse HEAD)"
  sha256sum "$CARGO_TARGET_DIR/release/cpu-query-cache-evaluate" \
    "$CARGO_TARGET_DIR/release/cpu-query-cache-aggregate"
} > "$ROOT/evaluator-provenance.txt"
available_gib=$(df -Pk "$ROOT" | awk 'NR==2 {print int($4 / 1048576)}')
[ "$available_gib" -ge 146 ] || {
  echo "need 146 GiB free after build for two complete restores, have $available_gib" >&2
  exit 1
}

for seed in 1 2; do
  SEED_ROOT="$ROOT/s$seed"
  RUN_STORE=$(jq -er --arg seed "$seed" '.seeds[$seed].run_store' "$SPEC")
  ARCHIVE_NAME=$(jq -er --arg seed "$seed" '.seeds[$seed].archive' "$SPEC")
  ARCHIVE_SHA256=$(jq -er --arg seed "$seed" '.seeds[$seed].archive_sha256' "$SPEC")
  CHECKPOINT_REL=$(jq -er --arg seed "$seed" '.seeds[$seed].checkpoint' "$SPEC")
  CHECKPOINT_MANIFEST=$(jq -er --arg seed "$seed" '.seeds[$seed].checkpoint_manifest' "$SPEC")
  RUN_CONFIG_REL=$(jq -er --arg seed "$seed" '.seeds[$seed].run_config.path' "$SPEC")
  RUN_CONFIG_SHA256=$(jq -er --arg seed "$seed" '.seeds[$seed].run_config.sha256' "$SPEC")
  case $seed in
    1) OUT_STORE=$OUT_STORE_S1 ;;
    2) OUT_STORE=$OUT_STORE_S2 ;;
  esac

  CKPT_DOWN="$SEED_ROOT/checkpoint-download"
  scripts/pod/uor-store get "$RUN_STORE/$ARCHIVE_NAME" "$CKPT_DOWN"
  CKPT_ARCHIVE="$CKPT_DOWN/$RUN_STORE/$ARCHIVE_NAME"
  (cd "$(dirname "$CKPT_ARCHIVE")" &&
    printf '%s  %s\n' "$ARCHIVE_SHA256" "$ARCHIVE_NAME" | sha256sum -c -)

  mkdir -p "$SEED_ROOT/checkpoint"
  mapfile -t extract_paths < <(jq -r --arg seed "$seed" \
    '.seeds[$seed] |
      [.checkpoint_manifest, .run_config.path, (.checkpoint_files[].path)] | .[]' "$SPEC")
  tar -xzf "$CKPT_ARCHIVE" -C "$SEED_ROOT/checkpoint" "${extract_paths[@]}"
  : > "$SEED_ROOT/checkpoint/selected.sha256"
  {
    printf 'run-config\t%s\t%s\n' "$RUN_CONFIG_REL" "$RUN_CONFIG_SHA256"
    jq -r --arg seed "$seed" \
      '.seeds[$seed].checkpoint_files | to_entries[] |
        [.key, .value.path, .value.sha256] | @tsv' "$SPEC"
  } | while IFS=$'\t' read -r _ path sha; do
    got=$(sha256sum "$SEED_ROOT/checkpoint/$path" | cut -d' ' -f1)
    [ "$got" = "$sha" ] || {
      echo "$path hashes to $got, not $sha" >&2
      exit 1
    }
    line="$sha  ./${path#oh/run/}"
    grep -Fxq "$line" "$SEED_ROOT/checkpoint/$CHECKPOINT_MANIFEST" || {
      echo "$path differs from its inner SHA-256 manifest" >&2
      exit 1
    }
    printf '%s\n' "$line" >> "$SEED_ROOT/checkpoint/selected.sha256"
  done
  [ "$(wc -l < "$SEED_ROOT/checkpoint/selected.sha256")" -eq 4 ]
  (cd "$SEED_ROOT/checkpoint/oh/run" &&
    sha256sum -c "$SEED_ROOT/checkpoint/selected.sha256")
  CHECKPOINT="$SEED_ROOT/checkpoint/$CHECKPOINT_REL"
  jq -e --argjson seed "$seed" --argjson expected "$EXPECTED" '
    .seed == $seed and
    .objective == $expected.run.objective and
    .train_as_served == $expected.run.train_as_served and
    .width == $expected.run.width and
    .hamming_read == $expected.run.hamming_read and
    .batch == $expected.run.batch and
    .micro_batch == $expected.run.micro_batch and
    .steps == $expected.run.steps and
    .train_split.tokens_sha256 ==
      "ed8f25374e99bd65cd509b5cfcdffb6a8476401a973ea02aaaaa0ea76e35ee52" and
    .train_split.mask_sha256 ==
      "ec6e520c7a67aaf6efc7aaf42bf7513b3e697239fb18b73a1e89f130eacd4875"
  ' "$SEED_ROOT/checkpoint/$RUN_CONFIG_REL" >/dev/null
  jq -e --argjson expected "$EXPECTED" '
    .arch == $expected.model.arch and .gate == $expected.model.gate and
    .head == $expected.model.head and .pattern == $expected.model.pattern and
    .rotation == $expected.model.rotation and .width == $expected.model.width and
    .context == $expected.model.context and .heads == $expected.model.heads
  ' "$CHECKPOINT/config.json" >/dev/null

  CACHE_DOWN="$SEED_ROOT/cache-download"
  scripts/pod/uor-store get "$OUT_STORE/results/cache.tar.zst" "$CACHE_DOWN"
  scripts/pod/uor-store get "$OUT_STORE/results/cache.tar.zst.sha256" "$CACHE_DOWN"
  scripts/pod/uor-store get "$OUT_STORE/results/cache.SHA256SUMS" "$CACHE_DOWN"
  scripts/pod/uor-store get "$OUT_STORE/status.json" "$CACHE_DOWN"
  scripts/pod/uor-store get "$OUT_STORE/binaries.sha256" "$CACHE_DOWN"
  scripts/pod/uor-store get "$OUT_STORE/job.log" "$CACHE_DOWN"
  ARCHIVE_ROOT="$CACHE_DOWN/$OUT_STORE"
  CACHE_ARCHIVE_ROOT="$ARCHIVE_ROOT/results"
  (cd "$CACHE_ARCHIVE_ROOT" && sha256sum -c cache.tar.zst.sha256)
  CACHE_TAR="$CACHE_ARCHIVE_ROOT/cache.tar.zst"
  CACHE_MEMBERS="$SEED_ROOT/cache-members.txt"
  tar --zstd -tf "$CACHE_TAR" > "$CACHE_MEMBERS"
  [ -s "$CACHE_MEMBERS" ]
  while IFS= read -r member; do
    relative=${member%/}
    case "/$relative/" in
      *//*|*/../*) echo "unsafe cache path: $member" >&2; exit 1 ;;
    esac
    case $relative in
      cache|cache/*) ;;
      *) echo "cache path escapes cache/: $member" >&2; exit 1 ;;
    esac
  done < "$CACHE_MEMBERS"
  tar --zstd -tvf "$CACHE_TAR" |
    awk 'substr($0, 1, 1) != "-" && substr($0, 1, 1) != "d" {
      print "cache archive contains a non-regular entry: " $0 > "/dev/stderr";
      bad = 1
    } END { exit bad }' ||
    { echo "cache archive contains a link or special file" >&2; exit 1; }
  mkdir -p "$SEED_ROOT/cache-extracted"
  tar --zstd --no-same-owner --no-same-permissions \
    -xf "$CACHE_TAR" -C "$SEED_ROOT/cache-extracted"
  (cd "$SEED_ROOT/cache-extracted" &&
    sha256sum -c "$CACHE_ARCHIVE_ROOT/cache.SHA256SUMS")
  STATUS="$ARCHIVE_ROOT/status.json"
  BINARY_SHA=$(cut -d' ' -f1 "$ARCHIVE_ROOT/binaries.sha256")
  grep -Fxq "script_sha256=$CAPTURE_JOB_SHA256" "$ARCHIVE_ROOT/job.log"
  grep -Fxq "spec_sha256=$CAPTURE_SPEC_SHA256" "$ARCHIVE_ROOT/job.log"
  jq -e --arg binary "$BINARY_SHA" --arg spec "$CAPTURE_SPEC_SHA256" \
    --arg checkpoint "$CHECKPOINT_REL" --arg source "$SOURCE_PIN" \
    --arg run_config "$RUN_CONFIG_SHA256" --argjson seed "$seed" '
      .seed == $seed and .status == "CAPTURED" and
      .source_pin == $source and
      .checkpoint == $checkpoint and
      .checkpoint_inputs_sha256 == $spec and
      .run_config_sha256 == $run_config and
      .capture_binary_sha256 == $binary
    ' "$STATUS"
  TAR_SHA=$(cut -d' ' -f1 "$CACHE_ARCHIVE_ROOT/cache.tar.zst.sha256")
  jq -e --arg sha "$TAR_SHA" '.cache_tar_sha256 == $sha' "$STATUS"
  jq -r .cache_manifest_sha256 "$STATUS" > "$SEED_ROOT/manifest.sha256"
  available_gib=$(df -Pk "$ROOT" | awk 'NR==2 {print int($4 / 1048576)}')
  [ "$available_gib" -ge 31 ]
done
```

```sh
for seed in 1 2; do
  SEED_ROOT="$ROOT/s$seed"
  CHECKPOINT_REL=$(jq -er --arg seed "$seed" '.seeds[$seed].checkpoint' "$SPEC")
  RAYON_NUM_THREADS=32 /usr/bin/time -v -o "$ROOT/centroid-s$seed.time.txt" \
    "$CARGO_TARGET_DIR/release/cpu-query-cache-evaluate" \
    "$SEED_ROOT/cache-extracted/cache" \
    --seed "$seed" \
    --checkpoint-root "$SEED_ROOT/checkpoint/$CHECKPOINT_REL" \
    --train train \
    --panel oasst2 \
    --panel wiki-00000 \
    --panel wiki-00020 \
    --panel wiki-00040 \
    --memory 0 \
    --minimum 8 \
    --min-fitted-rows 100 \
    --min-fit-observations 10000 \
    --expect-train-tokens ed8f25374e99bd65cd509b5cfcdffb6a8476401a973ea02aaaaa0ea76e35ee52 \
    --expect-train-mask ec6e520c7a67aaf6efc7aaf42bf7513b3e697239fb18b73a1e89f130eacd4875 \
    --expect-rows 1460697 \
    --expect-identity 5f88a5b3295ba5d684a8c75206787642fd65a2377d2dbb268992ec5c71a621eb \
    --out "$ROOT/centroid-s$seed.json"
  jq -e --arg profile "$EVAL_PROFILE" \
    '.evaluation_profile == $profile' "$ROOT/centroid-s$seed.json"
done
```

This runs on the D84 host: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads and
59 GiB RAM. The TRAIN query stream is not materialized in memory.

After both reports exist, check that each names its captured manifest and pre-registered
checkpoint files:

```sh
for seed in 1 2; do
  manifest=$(cat "$ROOT/s$seed/manifest.sha256")
  run_config_sha=$(jq -er --arg seed "$seed" \
    '.seeds[$seed].run_config.sha256' "$SPEC")
  jq -e --arg manifest "$manifest" \
    '.cache.manifest_sha256 == $manifest' "$ROOT/centroid-s$seed.json"
  jq -e --slurpfile spec "$SPEC" --arg seed "$seed" \
    --arg run_config "$run_config_sha" --arg profile "$EVAL_PROFILE" '
      .cache.checkpoint_files ==
        ($spec[0].seeds[$seed].checkpoint_files |
          with_entries(.value = .value.sha256)) and
      .cache.run_config_sha256 == $run_config and
      .evaluation_profile == $profile
  ' "$ROOT/centroid-s$seed.json"
done
/usr/bin/time -v -o "$ROOT/aggregate.time.txt" \
  "$CARGO_TARGET_DIR/release/cpu-query-cache-aggregate" \
  "$ROOT/centroid-s1.json" "$ROOT/centroid-s2.json" --out "$ROOT/final.json"
FINAL_SHA=$(sha256sum "$ROOT/final.json" | cut -d' ' -f1)
jq -n --arg spec_sha "$CAPTURE_SPEC_SHA256" --arg profile "$EVAL_PROFILE" \
  --arg target "$TARGET_ID" --arg final_sha "$FINAL_SHA" '
    {
      schema: "uor-r4.centroid-result-identity/1",
      input_spec_sha256: $spec_sha,
      evaluation_profile: $profile,
      target_id: $target,
      final_sha256: $final_sha
    }
  ' > "$ROOT/result-identity.json"
```

## Instrument checks

- **I-0:** the float-training library tests pass at the capture pin.
- **I-1:** every input file matches the hashes above.
- **I-2:** the archive hash and inner checkpoint `SHA256SUMS` pass.
- **I-3:** each cache manifest names the expected checkpoint, D3 hashes, four panel hashes,
  query space and table hash, and contains exactly five no-memory splits: TRAIN plus the
  four panels.
- **I-3b:** the CPU report's `cache.manifest_sha256` and every checkpoint-file hash equal
  the uploaded status and the pre-registered seed table above. Its fixed
  `n3s-registered-v1` field is read as the frozen protocol identifier and paired with the
  `b64l-centroid-v1` target in `result-identity.json`.
- **I-4:** the CPU original table reproduces the frozen nearest decision at every panel
  row. Any difference is `CONTROL_FAULT`.
- **I-5:** the evaluator computes each split identity and the registered pooled identity
  `5f88a5b3...`; a mismatch is `CONTROL_FAULT`.
- **I-6:** the matched shuffle changes at least one eligible target and at least one
  resulting table coordinate while preserving every target's observation count.

## Minimum test power

- at least 100 fitted token rows;
- at least 10,000 TRAIN observations across fitted rows;
- all four panels and all 1,460,697 rows;
- both seeds.

The evaluator emits `INSUFFICIENT_POWER` from these checked counts. It stops that seed and
reports the census; it is neither a positive nor a mechanism negative.

## Bar

For each seed, pool the four panels:

- original decisions must match every frozen nearest decision exactly;
- centroid top-8 must be at least original top-8;
- centroid top-1 must exceed original and shuffled-control top-1;
- centroid-only wins against original must satisfy
  `candidate_only >= max(2, ceil(2 * sqrt(candidate_only + control_only)))`;
- the same paired rule must hold against the matched shuffle.

All conditions must hold at both seeds for a positive development result. An aggregate
`REJECT`, including a mixed `KEEP`/`REJECT`, closes mean-centroid refit for the tested
b64L configuration. `CONTROL_FAULT` and `INSUFFICIENT_POWER` are instrument outcomes, not
closures. Because this panel selected the current champion, even a positive can only
nominate the candidate for a fresh transfer panel; it cannot alone put the mechanism on
the serving path.

## Cost and stop

- Two parallel one-card captures, each reading 27.42M TRAIN rows and 1.46M panel rows.
- Estimate: 2-4 hours on each pod. The watcher renews each healthy lease; estimates are
  not cancellation rules. At the observed $1.15/h per card, a four-hour run on each of
  two leases projects about $9.20. The controller exports a **combined** running-rate cap
  of `$4.50/h`, not a `$1.15/h` per-pod cap; the D67 review must record the actual leases
  and rates before launch.
- The raw cache is 28.02 GiB. The job requires 128 GiB at admission and again requires
  118 GiB after downloads and build but before capture; that second gate covers three
  raw-size-equivalent copies plus the 31 GiB reserve. It retains the actual post-capture,
  both-archive readback-size and floor checks, including a 40 GiB minimum. Each produced
  archive is decompressed by `zstd -t` before hashing and upload.
- CPU evaluation: 32 Rayon workers, at most 32 GiB projected RSS and an estimated 2
  hours. It admits 170 GiB before the build (146 GiB for both retained restores plus a
  24 GiB build allowance), then requires 146 GiB after build and 31 GiB after each
  restore. Estimates are not cancellation rules. The point estimator streams TRAIN;
  only one panel's queries are resident.
- Each no-memory cache is about 29 GiB; the compressed archive is uploaded and read back.
- Stop on any input/checkpoint mismatch, `CONTROL_FAULT`, insufficient power, archive
  ambiguity, resource breach or missing credential.

## Outcome use

- **Positive at both seeds:** nominate centroid for a fresh transfer panel and the next
  active served-readout comparison. The current panel cannot retain a mechanism it helped
  select.
- **Negative:** close mean-centroid refit on this b64L configuration and move to a
  rank-aware or two-table
  output objective. Do not retune the threshold or shuffle after opening the panels.
- **Insufficient power:** report the observed fitted-row/observation census; do not call
  it a negative.
- **Control fault:** repair the capture/readout identity before interpreting any centroid
  number.
