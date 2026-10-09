# Shared GPU pods — one tool, one cadence

Every lab session (Claude, Codex/GPT, OpenCode/DeepSeek and any lab that joins
later; several sessions of one lab may run at once) uses the same tool,
`scripts/pod/uor-pod`, and the same cadence to see, lease, create, share and
delete the owner's Runpod GPU pods. The goals: no pod idles, no session is
surprised by another's pod, and any session can take over a pod whose lease
expired. The source of truth is the laptop state directory below. The
[Compute board #2037](https://github.com/UOR-Foundation/uor-r4/issues/2037) is
a **log for the owner, not a communication channel**: the tool posts one line
per event; sessions coordinate through leases and the tool's output, and do not
read or write the board to ask for anything. The rules every session must
follow without reading this file are inline in [AGENTS.md](../../AGENTS.md)
("GPU pods") and printed by `uor-pod` with no arguments and at the top of
`uor-pod status`.

## Identity: lab + session

Every mutating command takes `--lab L --session S` (or `UOR_POD_SESSION=S`).
Leases are files `leases/<pod>/<lab>-<session>.json`; every ledger line and
board post carries both. Sessions of the same lab are separate holders: they
may hold different GPUs on the same or different pods, and one session never
uses, renews, releases or deletes another session's lease or files (including a
`/root/KEEP_ALIVE`).

## The cadence

1. **Look first.** `scripts/pod/uor-pod status` prints the rules, then every
   pod (running and stopped; GPUs, $/h, uptime), its leases, each GPU's live
   utilisation and memory next to the session that leases it, the reaper, any
   running pod with **NO LIVE LEASE**, every stopped pod as a **CLEANUP** item,
   stray temporary files, the live RTX 5090/4090 stock per datacenter, and ends
   with **"Free GPUs you can lease now"**. `status --du` adds volume usage.
2. **Idle is not free.** A GPU under another session's live lease is shown
   "leased but idle N min (not free)": only its holder (`release`) or expiry
   frees it. **Only an EXPIRED lease** (past `expires`, no renewal) may be
   taken over: `lease` re-probes the pod, refuses if a job still runs on those
   GPUs, shrinks the expired lease and logs a `takeover`.
3. **Share before you create.** Lease free GPUs:
   `uor-pod lease POD --lab L --session S --gpus 0,1 --purpose "…" --hours H [--card URL]`.
   `up` refuses (exit 3) and prints the free slot when one of the requested
   type exists. A refused lease prints the free-GPU summary and the spin-up
   guidance.
4. **No free GPU → spin up your own pod** within the caps (all running pods of
   all labs count): `uor-pod up --lab L --session S --purpose "…" --hours H`
   (defaults: 2 × RTX 5090). It applies the GPU policy, placement order and
   caps below, waits for SSH, seeds a non-canonical volume, runs the bootstrap
   (toolchain, cached binaries + parity, reaper) and writes your lease. When the
   cap is reached it names any free GPUs to lease (exit 4). **Never fall back
   to the laptop CPU because no GPU is free or the cap is reached; if no GPU is
   free and the cap is reached, ask the owner or wait for a lease to expire.**
5. **Renew every ≤ 30 minutes while working:**
   `uor-pod renew POD --lab L --session S [--hours H]`. A lease is a promise
   that the session is actively using the GPUs; expiry is automatic release.
   Renewal is not posted to the board (the ledger has it).
6. **One job per GPU, on the GPU.** `uor-pod run POD --lab L --session S --gpu K -- CMD…`
   starts a detached job under `flock /root/gpuK.lock` (a second job on the
   same GPU queues behind it) with the bootstrap environment, logging to
   `/workspace/uor-r4/jobs/<lab>/<UTC>-<session>-gpuK.log`, ending with an
   `# exit=N` line. `--gpu 0,1` holds both locks for a data-parallel run. GPU
   evaluation uses `device=cuda`.
7. **Everything durable goes to `/workspace`** (layout below). The container
   disk (`/root`) dies with the pod. **Do not keep stopped pods as storage.**
   Anything durable lives on the network volume; a stopped pod whose files are
   not on the volume is either copied now or accepted as lost (regenerable) and
   deleted — never left for someone to remember. `uor-pod prune-stopped
   [--older-than 24h]` lists them; `--yes` deletes those not marked KEEP after
   you have confirmed nothing unique is on them (owner approval for pods with
   data).
8. **Release when done:** `uor-pod release POD --lab L --session S`, then, if
   no other session holds a lease, `uor-pod down POD --lab L --session S`
   (refused while another session's lease is live). Do not leave a pod for the
   reaper to find; it is the safety net, not the plan.
9. **Anyone may reap:** `uor-pod reap` deletes running pods that have no
   unexpired lease and whose GPUs have been idle ≥ 20 minutes (no compute
   process, ≤ 2 % utilisation, no held GPU lock). It never touches a pod with a
   live lease or KEEP mark and never deletes a pod it cannot probe.

`uor-pod log [-n N]` tails the ledger; `uor-pod ssh POD [CMD]` opens a shell;
`uor-pod gpus` prints the policy table and live stock.

**Pods made outside `up`** are backfilled with
`uor-pod register POD --lab L --session S --purpose "…" [--gpu 5090] [--keep REASON]`:
it probes the pod and leases **only the GPUs that are busy now, for 1 hour**
(the session renews if it is really working); idle GPUs stay free. A stopped pod
gets no lease. `--keep REASON` (or `uor-pod keep POD --purpose REASON`) marks a
pod that holds unique data: `down`, `reap` and `prune-stopped` refuse it until
`uor-pod unkeep POD`.

**Validation/test pods** are created with `up --test` (name prefix
`uor-test-`); they do not count toward the pod cap (they still count toward
$/h), free GPUs elsewhere do not block them, and the session that made them
deletes them as soon as the test ends.

**Lease files are written only by `uor-pod`** (atomic temporary file + rename
under one lock). Editing them by hand races other sessions; `status` warns about
stray `*.tmp*` files.

## GPU policy

Measured for **our** code (FP32 CUDA-core trainer with TF32 matmul; the
bf16/tensor-core rewrite is in progress). Owner decision, 5 October 2026.

| Key | GPU | VRAM | `CUDA_COMPUTE_CAP` | Secure $/GPU/h | Tier | Use |
| --- | --- | --- | --- | --- | --- | --- |
| `5090` | RTX 5090 | 32 GB | 120 | 0.99 | **default** | The only automatic choice. 96M fine-tune step ≈ 0.09–0.10 s/GPU (2 × 5090: 2,000 steps, batch 16, ctx 384 in ≈ 3.5 min); parity 22/22. ≈ 1.7–2 × a 4090 and ≈ 25 % cheaper per job. |
| `4090` | RTX 4090 | 24 GB | 89 | 0.74 | explicit | Only with `--gpu 4090` (≈ 0.17–0.20 s/step). |
| `l40s`, `6000ada` | L40S / RTX 6000 Ada | 48 GB | 89 | 1.09 / 0.84 | explicit | Only when a job needs more than 32 GB. |
| `a100`, `h100`, `h200`, `b200`, … | A100/H100/H200/B200 | 80+ GB | 80/90/100 | 1.59+ | **forbidden** | Measured A100 = 0.6–0.75 × a 4090 for this trainer. Never automatic; refused without `--owner-approved`. Revisit after the bf16 Phase 2 parity gate. |

Default shape: **2 GPUs per pod** (`data_parallel=2`, or two seeds in
parallel); 1 GPU for evaluation-only or grading jobs (`--count 1`).

"Low" stock usually means one free card per host. When `--count` is not given
and no host has 2 free, `up` falls back to 1 GPU (on 10-06, 2-GPU creates failed
15 of 15 times while 1-GPU creates succeeded first try). Run a second arm as its
own pod. Pass `--count 2` to insist on two GPUs on one host.

## Placement

`up` tries the requested type (5090 unless `--gpu` says otherwise) on secure
cloud in the network-volume datacenters, in this order:

| Order | Datacenter | Volume | Role |
| --- | --- | --- | --- |
| 1 | EUR-NO-1 | `lmd1pfah3y` `uor-shared-EUR-NO-1` (200 GB) | **canonical store** |
| 2 | EU-RO-1 | `uor-shared-EU-RO-1` (100 GB, created on first placement) | non-canonical |
| 3 | EUR-IS-1 | `uor-shared-EUR-IS-1` (100 GB, created on first placement) | non-canonical |

These are the only network-volume datacenters that ever list 5090 stock
(EU-CZ-1 and EUR-IS-2 have 5090s but no network volumes). The two
non-canonical volumes are owner-approved and created lazily, only when a pod
is first placed there (posted to the board).

Stock can change between the listing and the request. When `runpodctl pod
create` in one datacenter fails with Runpod's out-of-stock error ("no longer
any instances available"), `up` moves on to the next listed datacenter in this
order; any other create error stops at once. Before anything is listed or
created, `up` checks every helper file it will upload (`uor-pod-bootstrap.sh`,
`uor-reaper.sh`, `hot-set.txt`: present, readable, non-empty, `bash -n` clean),
so a broken checkout costs nothing. It also checks every argument first: an
option `up` does not take or a stray word is refused; `--ref` (branch, tag or a
7-40 character SHA) is resolved to the full commit, which must be on a branch of
the public repository (the pod fetches from GitHub); and the exact bootstrap
argument vector is run through the bootstrap's own parser
(`uor-pod-bootstrap.sh --check-args`). A bad argument never creates a pod.

**Failed `up` and the circuit breaker.** The bootstrap output is saved in full
on the laptop (`$STATE/logs/bootstrap-<pod>-<UTC>.log`) and teed onto the pod
volume (`/workspace/uor-r4/pods/bootstrap-<pod>-<UTC>.log`; lost with an
off-volume pod). When the bootstrap fails, `up` prints its last 30 lines,
records both log paths and the last output line in the `up-failed` ledger
event, and deletes the pod. After two `up-failed` events of one lab+session
within 60 minutes, `up` refuses (exit 5, `up-refused` in the ledger) and shows
those reasons and logs until the cause is fixed; `--force-retry` overrides it
(`up-force-retry`). Test a fix with `uor-pod bootstrap POD` on a running pod
rather than creating new ones (the incident of 5 October: 14 pods were created,
billed and deleted in 2 hours for the same bootstrap failure).

If none of the three has stock, or all of them refuse at create time, `up`
**does not fall back silently**: it prints the stock table and stops. Then:

* `--wait [--wait-hours H]` retries every minute for up to H hours (5090 stock often shows as "Low" for only a few minutes, so a 5-minute poll missed it)
  (default 2), posting "waiting" and "gave up" to the board;
* `--gpu 4090` is the explicit alternative (same datacenter order);
* `--allow-off-volume` places the pod in any stocked datacenter with a local
  150 GB `/workspace` and no network volume (seeded like a non-canonical one).

### Non-canonical and off-volume pods

A non-canonical `/workspace` starts empty. `up` seeds it with the **hot set**
([`scripts/pod/hot-set.txt`](../../scripts/pod/hot-set.txt)) before bootstrap,
copying from a running pod that mounts the canonical volume, through the
laptop (`uor-pod seed POD`, or `UOR_POD_HOT_MIRROR=DIR` for a laptop copy of
the same paths). If no canonical pod is running, seeding is skipped with a
warning and bootstrap installs the toolchain and builds cold.

| Hot-set path (under `/workspace`) | What | Size (2026-10-05) |
| --- | --- | --- |
| `toolchain/rust-1.97.1-x86_64.tar`, `toolchain/cargo-registry.tar` | Rust 1.97.1 (rustup tree) and the cargo registry cache | ~1.5 GB |
| `bin/<sha>-sm<cap>/` | cached release binaries for the commit being bootstrapped | ~0.1 GB |
| `uor-r4/data/` | tokenizer, fine-tune stores (`ft-*.tar`), the 100M base being fine-tuned (`geo-100m-*`), paraphrases, step5 inputs (corpora, sieve compiler/trunk, open panels), `MD5SUMS` | 2.6 GB |
| `toolchain/ollama`, `ollama/` | Ollama binary and qwen2.5:7b (only with `--with-ollama`) | ~5 GB |

Results produced off-canonical must be **archived back before `down`**:
`uor-pod push POD uor-r4/runs/… uor-r4/evals/…` (relays to the canonical
volume through the laptop) or fetched to the laptop and recorded in the task
issue. `down` refuses a non-canonical or volume-less pod until you pass
`--confirm-archived`. `uor-pod pull POD PATH…` copies extra inputs in.

## Any region

Owner-approved 7 October 2026 (#1750).

- **GPU ladder.** Without `--gpu`, `up` tries 2 × RTX 5090, then 2 × RTX 4090,
  then 2 × RTX PRO 6000, then 1 × each in the same order. `--gpu KEY` asks for
  that type only.
- **Placement.** The three volume datacenters come first: EUR-NO-1, EU-RO-1,
  EUR-IS-1. When `UOR_POD_VOLUME_DCS` is unset, the allowlisted extra
  datacenters (`EXTRA_VOLUME_DCS`) follow, ordered by stock. Every pod outside
  EUR-NO-1 is non-canonical: it starts with an empty `/workspace`, and its
  results must be archived before `down` (see Placement above).
- **HF store.** A private Hugging Face dataset, `caseyallard/uor-r4-store`,
  holds `data/*` and the base `bases/geo-214m-e448de86/{model/,report.json}`.
  On a non-canonical pod the bootstrap (`--non-canonical`) downloads the
  missing `data/*` files and checks them against `MD5SUMS`. This step only
  downloads, and a failure in it is a warning. To fetch a base:
  `hf download caseyallard/uor-r4-store --repo-type dataset --include 'bases/geo-214m-e448de86/*' --local-dir DIR`.
- **Token.** `up` copies `~/.cache/huggingface/token` from the laptop to the
  pod (`/root/.cache/huggingface/token`, mode 600) over stdin and logs only
  "hf token copied", never the value.
- **One-off pod-to-pod copies.** `runpodctl send FILE` on the source pod
  prints a code; run `runpodctl receive CODE` on the destination.

## Layout of `/workspace` (canonical volume)

| Path | Contents | Owner |
| --- | --- | --- |
| `uor-r4/data/` | shared inputs (tar + `MD5SUMS`) | all labs |
| `uor-r4/runs/`, `uor-r4/evals/` | training runs and evaluations, one sealed report root each | the run's lab |
| `uor-r4/step*/` | per-step bundles (`MD5SUMS`, `.done` markers) | the step's lab |
| `uor-r4/jobs/<lab>/` | `uor-pod run` logs | the lab |
| `uor-r4/pods/reaper.log` | pod-side reaper log (all pods) | tool |
| `bin/<sha>-sm<cap>/` | release binaries, `BUILD.json`, `parity.log`, `SHA256SUMS` | tool |
| `toolchain/` | `rust-<ver>-x86_64.tar`, `cargo-registry.tar` (unpacked to `/root` by bootstrap), `ollama/` | tool |
| `ollama/` | Ollama models (`OLLAMA_MODELS`) | tool |
| `codex-uor-r4-20261004/`, `pearlfortune/` | preserved lab material | its lab |

Never delete another lab's material on the volume; it is the shared archive.

## Standard template and bootstrap

* **Template** `uor-r4-cuda128` (`h15vb984sw`): the official
  `runpod/pytorch:1.0.2-cu1281-torch280-ubuntu2404` image (CUDA 12.8.1
  devel, `nvcc` present), 60 GB container disk, port 22/tcp only, volume at
  `/workspace`. SSH keys come from the account (`PUBLIC_KEY`); Jupyter is off
  (the image starts it only when `JUPYTER_PASSWORD` is set, and no HTTP port
  is exposed; the image's internal nginx listens on a few ports that are not
  mapped publicly). It replaces `vishva123/nvidia-cuda-13.2-devel-runpod`, which
  ignored `PUBLIC_KEY` and ran an unauthenticated Jupyter.
* **Bootstrap** (`scripts/pod/uor-pod-bootstrap.sh`, run by `up` or
  `uor-pod bootstrap POD`): checks `nvcc` 12.8 (apt install only as a
  fallback), maps the GPU to `CUDA_COMPUTE_CAP` (5090 → 120, 4090/L40S/6000
  Ada → 89, A100 → 80, H100/H200 → 90), unpacks Rust 1.97.1 from
  `/workspace/toolchain/rust-1.97.1-x86_64.tar` to the container disk
  (installing it and writing the archive on the first pod of a volume;
  compiling with the toolchain directly on the network filesystem was
  IO-bound, rustc at ~20 % of one core), limits cargo to the pod's CPU quota
  (`RUNPOD_CPU_COUNT`; `nproc` reports the host), and builds only when
  `/workspace/bin/<sha>-sm<cap>/BUILD.json` with parity PASS is missing (a
  cached build whose parity failed is moved to `…failed-<UTC>-<pod>` and rebuilt):

  ```
  CUDA_COMPUTE_CAP=<cap> cargo build --release -p uor-r4-training --features cuda \
    --example geometric-stack --example m-world --example mqar-bench \
    --bin chat-grade --bin dialogue-recall-corpus --bin mix-chat-corpus
  UOR_REQUIRE_CUDA=1 cargo test --release -p uor-r4-training --features cuda \
    --test cuda_stack_ops_parity -- --test-threads=1
  ```

  The source is a shallow fetch of the public repository at the exact commit
  (`--ref`, default `main`), built on the container disk; the binaries,
  `parity.log` and `BUILD.json` (commit, cap, rustc, nvcc, timings, parity
  result) are moved into the cache atomically. `--with-ollama` adds the
  judge: the current `ollama-linux-amd64.tar.zst` asset (then the legacy
  `.tgz`, then the official `install.sh`) unpacked onto the volume, models in
  `/workspace/ollama`. A failed Ollama install or model pull is **not fatal**:
  the pod is kept for training, `BOOTSTRAP_RESULT` reports `"ollama":
  "FAILED"`/`"pull-failed"`, and `up` warns and records it in the `bootstrap`
  ledger event. Only build/parity failures fail the bootstrap. Shells and `uor-pod run` jobs source `/root/.uor-pod-env`
  (`UOR_BIN`, `CUDA_COMPUTE_CAP`, PATH, LD_LIBRARY_PATH, cargo, Ollama).
* **Pod-side reaper** (`/root/uor-reaper.sh`, started by bootstrap only on
  pods made by `up`): every minute, if no `/root/leases/*.json` is unexpired
  and the GPUs have been idle ≥ 20 minutes, it deletes its own pod through the
  Runpod GraphQL API (`podTerminate`) with the pod's own `RUNPOD_API_KEY` (read
  from `/proc/1/environ` at use, passed to curl on stdin, never logged; that
  pod-scoped key is refused by REST v1 with HTTP 403, so the reaper uses
  GraphQL). Bootstrap runs `uor-reaper.sh --check` and reports `api-ok`. Log:
  `/workspace/uor-r4/pods/reaper.log`. Leases reach the pod as
  `/root/leases/<lab>.json` on every lease/renew; **if a renew cannot reach the
  pod the reaper will not see it** — the tool warns, retry.

## Leases, state and the board

* A lease is `{lab, session, id, pod, gpus, purpose, card, started, expires, renewed, hours}`
  in `~/.local/share/uor-r4/compute/leases/<pod>/<lab>-<session>.json` on the
  laptop all sessions share. Writes take one global `flock` and replace files by rename.
  `uor-pod` writes times only as `YYYY-MM-DDTHH:MM:SSZ`. It reads any RFC 3339 /
  ISO 8601 time with an explicit offset (fractional seconds, `Z`, `+HH:MM`,
  `+HHMM`) or an epoch number, and so does the pod-side reaper. A lease whose
  `expires` cannot be parsed (or has no offset), whose `gpus` is not a list of
  indices, or that is not JSON is treated as **live** on the GPUs it names (all
  of them when unknown), never as expired; `status` prints a
  `MALFORMED LEASE FILE` warning naming it. Its holder fixes it with `renew`,
  which rewrites the canonical form.
  `ledger.jsonl` (append-only) records every event; `pods.json` caches what the
  API does not say (GPU type, datacenter, volume, whether `up` made it).
* `up`, `down`, `lease`, `release`, `takeover`, `reap`, `prune-stopped`,
  volume creation and `up --wait` outcomes post one log line (owner's view, not
  a channel) to [#2037](https://github.com/UOR-Foundation/uor-r4/issues/2037).
* `UOR_POD_DRY_RUN=1` prints every mutation instead of doing it;
  `scripts/pod/tests/uor-pod-dryrun.sh` exercises the logic against fake
  `runpodctl`/`ssh`/`gh`.

## Caps and what needs the owner

Defaults: **≤ 4 running pods and ≤ $8/h in total** (owner decision,
5 October 2026), enforced by `up`. The cap is the owner's spending rule, not a
Runpod limit; within it each lab may run its own 2 × 5090 pod ($1.98/h) in
parallel. All running pods of all labs and sessions count; only `uor-test-`
validation pods are left out of the pod count (not out of $/h).
`--owner-approved` overrides a cap or the GPU policy and is only used when the
owner's decision is cited in the card. The owner decides: raising the caps,
anything above $8/h, GPU types outside the policy table (and any use of the
forbidden tier), new network volumes beyond `uor-shared-EU-RO-1` and
`uor-shared-EUR-IS-1`, other providers and community cloud.

## Takeover

A lease past `expires` with no renewal is **abandoned**; a lease that is live is
never taken, however idle its GPUs look. Before reusing or deleting an
abandoned lease's pod, the next session checks `uor-pod status` (GPU
utilisation, compute processes, held GPU locks) and the job logs under
`/workspace/uor-r4/jobs/<lab>/`, leaves any live job running, and cites the
ledger line (`uor-pod log`) on its own task issue. `lease` performs the probe
itself and refuses GPUs that still run a job; with no live job it shrinks the
expired lease and logs a `takeover`. An abandoned pod with no live job may then
be deleted with `down`. Results already on `/workspace` stay where they are and
keep their owner.

## Security

* Never read, print or copy `~/.runpod/config.toml` or any API key. Use
  `runpodctl` and `gh`. The tool strips `env` from every pod record it shows.
* No unauthenticated Jupyter. Use the standard template; never set a weak
  `JUPYTER_PASSWORD` or expose 8888 publicly.
* The account SSH key for pods is `~/.ssh/uor_compute`
  (`UOR_POD_SSH_KEY` overrides). Pods are ephemeral, so host keys are not
  pinned (`StrictHostKeyChecking=no`); do not reuse a pod for secrets.

## Recovering pod access when SSH keys fail

1. `runpodctl ssh list-keys` must show the key you use
   (`ssh-keygen -lf ~/.ssh/uor_compute.pub` gives its fingerprint). If it is
   missing, add the **public** key with `runpodctl ssh add-key`; it reaches only
   pods created afterwards.
2. `runpodctl pod get POD` shows the SSH address and whether the pod is
   running; `runpodctl pod logs POD` shows boot errors.
3. On a standard-template pod, a key missing from the pod means it was added
   after creation: create a new pod with `up` (the volume keeps the work) and
   `down` the old one once its jobs are done.
4. On a legacy pod (image that ignores `PUBLIC_KEY`), do not open
   Jupyter to the internet to inject keys. Prefer replacing the pod; if a
   running job must be rescued, use `runpodctl send`/`receive` or the Runpod
   web terminal, then append the public key to `/root/.ssh/authorized_keys`.
5. SSH is often refused for 1–2 minutes after a pod reports ready (keys still
   being installed). `up` retries for up to 15 minutes and `bootstrap` for 5
   before declaring failure; do the same by hand. The API's runtime status can
   also flap to "initializing" on a healthy pod; the tool keeps the last SSH
   address it saw.

## Remote-command pitfalls

* `pkill -f PATTERN` inside `ssh … 'pkill -f PATTERN; …'` kills the remote
  shell itself, whose argv contains the pattern: use `pkill -x NAME`, a PID,
  or a bracketed pattern (`pkill -f 'jupyter[-]lab'`).
* On the legacy `vishva123/…` image Jupyter is the container's main process:
  killing it restarts the container (the disk survives, running jobs die). Do
  not kill it there; use the standard template, which runs no Jupyter.
* Start remote work detached so it survives the SSH session:
  `setsid nohup CMD > LOG 2>&1 < /dev/null &` — `uor-pod run` does this.
* A `flock` on the network volume can outlive a deleted pod, and a paid pod
  must not idle behind another pod's build. The bootstrap's build lock only
  avoids duplicate work: the holder refreshes `…lock.holder` with its phase
  and progress every 20 s, and a waiter gives up after at most 3 minutes
  (`UOR_BUILD_WAIT_S`), sooner when the heartbeat is older than 90 s or shows
  no progress for 90 s. It then builds on its own container disk into a
  private staging directory and publishes by a single `rename(2)` onto
  `bin/<sha>-sm<cap>/`, which never merges into an existing directory: the
  first parity-PASS build wins and a later one is discarded. A cached build
  without parity PASS is moved to `…failed-<UTC>-<pod>` under a short publish
  lock; without that lock the new build stays private (`…private-<pod>-<UTC>`).

## Validation record (5 October 2026)

Real runs against the account, template `h15vb984sw`, commit `6a918a43`,
1 × RTX 5090 in EUR-NO-1 on the canonical volume:

| Step | Result |
| --- | --- |
| `up` create → SSH with `~/.ssh/uor_compute` (`PUBLIC_KEY`) | 30–51 s on four pods |
| `nvcc` | 12.8 present in the image (no apt install); no Jupyter process; only port 22 mapped |
| Cold bootstrap (Rust from the volume archive, shallow fetch, build, parity) | 609 s: fetch 326 s (GitHub fetch of the commit), build 231 s, parity 42 s |
| Parity `cuda_stack_ops_parity` on sm_120 | PASS, 31 passed, 0 failed, 1 ignored |
| Warm bootstrap on a new pod (cache hit) | 2 s after SSH; `up` total ≈ 1 min |
| First-ever Rust install on the volume | 31 s (then archived) |
| `uor-pod run` | job ran on GPU 0 with `UOR_BIN`, `CUDA_COMPUTE_CAP=120`, log + `# exit=0` on the volume |
| `release` + `down` | pod deleted |
| Pod-side reaper (`--check` api-ok; idle limit set to 1 min for the test) | pod deleted itself through GraphQL `podTerminate` 61 s after its lease was released |

Not yet observed: a cold build on a 2 × 5090 pod, the non-canonical volume
path (EU-RO-1/EUR-IS-1 seeding) and `--allow-off-volume`, which are exercised
only by the dry-run test. The very first validation pod reported 30 of 31
parity tests failing with `CUDA_ERROR_NO_DEVICE` while `nvidia-smi` saw the
GPU; the failed cache is kept as `bin/6a918a43…-sm120.failed-*` and the next
pod passed. The cause was not identified; a cached build counts only with
parity PASS.
