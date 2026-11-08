# REPORT: one-model-end-to-end, deepseek-25

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Laptop only; no pod, model step or
spend.

## What did not work, first

- **The controller waited on the wrong process.** `uor-pod run` starts a
  detached remote job, prints its log path and returns after submission. The
  old controller waited only on that submission process, so it could check the
  store and tear down the pod while training was still running.
- **The first parser expected the wrong output shape.** `uor-pod` puts
  `Log: PATH` at the end of its one-line “Started on…” message, not on a
  separate line.
- **Directory appearance was not checkpoint completion.** The trainer creates
  `step-N/` before writing its tensor and sidecar files. Freezing on the
  directory could copy a partial checkpoint and fail or seal invalid evidence.
- **An existing pin was not a launch-pin check.** The controller could upload a
  job from a different or dirty local checkout while the remote source still
  checked out the named pin.
- **A log file alone did not prove the job was alive.** If the remote wrapper
  died without writing `# exit=N`, polling the existing log could continue
  indefinitely while the pod billed.
- **The store preflight treated failure as an empty output root.** A failed
  listing could therefore rent a pod that could never upload its seal.
- **A missing expected checkpoint was silently skipped.** A trainer that
  exited zero after writing only some of the four checkpoints could pass the
  post-run loop.
- **The registered export command could not export the train-as-served base.**
  `cpu-fbs-export` requires both `--raw-gates` and `--raw-transition`; without
  them the job would fail before the first base control.
- **The generated pod session failed uor-pod's lowercase validator.** The UTC
  timestamp introduced uppercase `T` and `Z`.
- **The launcher's `$1.15/h` cap was global, not per-pod.** With other labs
  already running, it could reject a valid one-GPU launch; it also left the
  owner's actual four-pod/$8-per-hour caps dependent on ambient environment.
- **A live hung job could renew forever, and cleanup could not delete it while
  its GPU lock remained held.** There was no file-progress watchdog or active
  process-group termination.
- **A failed panel or copy could be reported as a completed checkpoint.** The
  training function ran in a negated conditional, which suppresses Bash
  `errexit`; panel failures were hidden by the function's final `printf`, and
  the checkpoint could get its scored marker anyway.
- **The seal could race its own asynchronous `tee`.** The log writer was not
  joined before the hash manifest was made.
- **The seal budget was false.** It retained duplicate checkpoints and raw
  snapshots, while the packet promised an under-100-MiB seal.
- **An interrupt could archive a failed run while returning success.** The
  signal trap preserved a zero status when failure cleanup itself succeeded.
- **The fresh FIT review found four more launch-path defects.** A missing run
  root was treated as a stall from epoch zero; unreachable SSH was counted as a
  stopped job; non-finite loss was checked only after the full schedule; and the
  merged pin was not tied to the reviewed tree.
- **The fresh D67 review found three evidence defects.** Signal cleanup could
  lose the pod ID during rental or job submission, the base continuation grade
  did not assert the registered sample fingerprint, and the sheet still carried
  a path-dependent overlap hash plus a stale M2 bar.
- **The final D67 review found one missing finite check.** The per-checkpoint
  predicate covered training loss but not the DEV loss and margins serialized
  in the same progress row.
- **The post-repair FIT review found the step-0 exception was unconditional.**
  After the DEV checks were added, the predicate still accepted any step-0
  training-loss value instead of only the null value emitted before training.
- **The merged-tree FIT review found two incomplete protections.** The positive
  train-loss branch had no permanent gate or mutation, and failure cleanup
  attempted `down --to-store` even after `stop_remote_job` failed.
- **The final FIT review found four deeper launch-path defects.** A detached
  job could start before its log path was returned and then evade cleanup, the
  continuation guard stopped before the registered best-checkpoint outcome,
  the exactly-one-step-0 clause had no mutation, and initialization was checked
  only after training.
- **The next FIT review found the retry reused an initialization approval.**
  Attempt 1's approval marker remained present when the OOM retry launched,
  allowing attempt 2's trainer to pass the handshake before its own record was
  validated.
- **D67 found the pre-registration still ambiguous.** The deepseek-23 ORDER
  retained the retired early continuation stop, and neither it nor the sheet
  defined how "best checkpoint" would be selected before the results.
- **The final FIT review found three selector-contract gaps.** One stop-rule
  sentence still required the whole outcome map before step 512, the free
  generation rendering was not named, and the guard-safe/first-result selector
  semantics had no permanent mutations.
- **D67 found the controller-repair ORDER did not authorize execution.** The
  active deepseek-25 ORDER explicitly prohibited pod launch, so the reviewed
  runtime had no active run authority even though its run design passed.
- **The seal included the panel build target and duplicate downloads.** That
  contradicted the sheet's retained-storage projection and risked uploading
  build products as research evidence.

## What changed

- `run-s1.sh` submits once and parses the log path from the actual `uor-pod`
  success message.
- It polls the remote log for the terminal `# exit=N` footer while the existing
  lease renewal loop remains active.
- A missing log, nonzero remote exit or missing/failed seal fails the
  controller before a successful teardown path.
- `gate.sh` pins the detached-job rules and rejects any return of the old
  submission-wrapper wait.
- `test-mutants.sh` exercises the actual `uor-pod` log-line shape and proves
  submission-only waiting, ignored remote exits and missing log paths are
  rejected.
- The trainer now writes `.checkpoint-ready` only after the model, sidecars and
  served restore complete. The job waits for that marker before freezing and
  refuses an unmarked checkpoint after trainer exit.
- `train_and_score` is no longer called in a negated conditional. It records a
  failed trainer in `TRAIN_FAILURE`; every panel, export, grade and reading is
  explicitly checked, and a checkpoint is marked scored only after all of them
  succeed.
- `gate.sh` pins both sides of the completion protocol and its mutation suite
  rejects a return to directory-only checkpoint detection.
- The base control now runs teacher-forced ranks under both renderings and
  checks the registered one-user 18/138/873, per-turn 5/7/13, per-turn top-8
  198/256, free-generation 0/256, and continuation 132/512 readings exactly
  before training.
- The launcher now requires local `HEAD == UOR_SFT_PIN` and no tracked
  modifications before it may upload the controller or rent a pod.
- The remote probe reports the terminal exit when present, the GPU lock while
  the job runs, or a stopped state when the log exists but the job no longer
  holds its lock. Three consecutive stopped probes fail the controller.
- Store access is checked against the existing topic before rental. A listing
  failure aborts; only a successful listing can prove the new run root is
  absent.
- The job now requires the exact 512-row sample fingerprint on every grade,
  verifies `init.json` against the registered model/config/run hashes, and
  reproduces the registered corpus-panel overlap audit before training. The
  overlap report is checked by content because its JSON embeds input paths and
  therefore has a run-root-dependent file hash.
- A missing expected checkpoint fails unless a registered stop already ended
  the schedule.
- Base export and every checkpoint export now pass both raw served-snapshot
  flags, and the base artifact must reproduce `9a370566…`.
- Run IDs are lowercased before the uor-pod session is formed. The global caps
  are pinned to four pods and $8/hour, the provider is pinned to Vast, and the
  launch remains one RTX 5090.
- The exact pin must be contained in freshly fetched `origin/main`.
- The launcher also requires `UOR_SFT_REVIEWED_RUNTIME`; the SHA-256 of the
  reviewed tracked build/runtime tree at `PIN`, with only the FIT sheet's two
  verdict lines normalized and this report excluded as review evidence, must
  equal the identity recorded by review.
- Launch records free space, force-reads memory state, and authenticates or
  records absence of `model-time.json` and the storage receipt before rental.
- The controller fails after 30 minutes without a newer file under the run
  root. Failure cleanup terminates the exact remote process group, waits for
  GPU 0's lock to clear, then seals the pod through `down --to-store`.
- Progress is measured from submission time until the first run file appears;
  epoch zero is never a valid last-progress timestamp. Confirmed stopped jobs and
  SSH-transport failures have separate thresholds, so a locked startup or a
  short network fault cannot masquerade as a dead job.
- Cleanup discovers the owned pod from the live lease and partial `up.log`, and
  discovers the submitted job path from `run.out`, when a signal arrives before
  the normal assignments complete.
- The base archive is extracted only for the checkpoint and run config; the
  archive and temporary download root are removed before training. The panel
  build uses `/workspace/uor-r4/build`, outside the sealed run.
- The sealed job log is written directly, not through asynchronous `tee`.
  Duplicate checkpoint copies and raw snapshot files are outside or removed,
  and the job fails before upload if the completed seal exceeds 100 MiB.
- The first trainer attempt is micro-batch 16. Only a classified OOM permits
  one fresh micro-batch-8 attempt; both attempts keep separate trainer,
  artifact, grade, panel, scored-marker and reading roots.
- Every checkpoint and the base continuation grade must use the registered
  512-row sample fingerprint. Finiteness is checked at each scored checkpoint,
  including DEV loss and both DEV margins, so a non-finite loss stops the seed
  before later checkpoints are exported. Step 0 must have a null training loss;
  every positive step must have a finite numeric training loss.
- The launcher builds the Vast provider adapter from the reviewed checkout
  before calling `uor-pod status` or `uor-pod up`.
- Failure cleanup forces a nonzero status for every non-sealed run, including
  SIGINT and SIGTERM paths. It archives and tears down only after the active job
  has stopped; a failed stop leaves the pod and reports failure. If post-start
  submission fails before the log path is returned, cleanup finds the exact
  session's unique job script over SSH; an unidentifiable attempted submission
  fails without teardown.
- The continuation guard is recorded at every checkpoint but does not stop the
  run. Only row 2's registered no-ranking-movement stop fires during training;
  row 1 is applied after all checkpoints through 1,024, when the best checkpoint
  can be selected.
- The best checkpoint is now selected mechanically: guard-safe checkpoints are
  preferred when any exist, then teacher-forced EOS top-1, free EOS stops,
  complete accepted rows, EOS median rank, continuation guard and earlier step
  break ties. The job writes the ranked list and selection to
  `best-checkpoint.json`. The decision and reversal are recorded in
  `DECISION.md`. Only `per-turn/v1` free-generation readings select or classify
  a checkpoint; `one-user-turn/v1` is an audit control. The sheet's stop rule
  now applies only row 2 before step 512 and applies the remaining rows to the
  selected checkpoint after all available checkpoints through 1,024.
- The trainer publishes `.init-ready` only after writing `init.json`, then waits
  for the controller's approval file before optimization. The controller
  validates the exact initialization record and releases that wait; an invalid
  record stops the trainer while it is still blocked at initialization. Each
  micro-batch attempt has its own approval marker and refuses a pre-existing one.
- The outcome map's partial-movement row now covers every guard-passing case
  with any EOS/generation/content movement that does not meet rows 3–5,
  removing the prior logical gap before the catch-all.

## Measurements

No model training or GPU measurement ran. Local preflight downloaded and
re-hashed all 18 runtime data/base files; every hash matched the registered
value. The exact base checkpoint exported to
`9a370566cd9da163191c0196763c68ddb395927fcf61fdecc1736b04941a4bd9`.
The 512-row grade reproduced fingerprint `b724656f…` and `132/512`.

The current overlap instrument reproduced 25,071 documents, 79,679 assistant
replies and zero findings in all four overlap categories. Its path-bearing JSON
hashed to `587a24c1…`, demonstrating why the controller checks its content
rather than the historical path-dependent file hash. The base per-turn teacher
ranks were 5/7/13 with top-8 198; one-user ranks were 18/138/873 with top-8 0.

The repair leaves the planned workload unchanged: one RTX 5090, 36–39 minutes
normally, with the registered one-retry projection at about $1.50. The launch
now enforces a 100 MiB seal cap and keeps the panel build outside that seal.

## Checks run at the head

- Post-repair FIT review: **NO-GO** at
  `3d7b0130b670574f5468d3396b806e9a3fbea059`, reviewer session
  `01a1b83b-0416-7ba0-85bb-d8371645cd69`, because step-0 training loss was
  still accepted without validation.
- Merged-tree FIT review: **NO-GO** at
  `b640dcdc7b78b19965c3961ac4d0601243fb894e`, reviewed-runtime
  `fe816bc244d526d591d2caadd27efaade461c81efd64991f354bd0738f17ac08`,
  reviewer session `01a1b84a-8abf-7a62-89cd-7e1e98b8fa36`, because the
  positive-step training-loss rule lacked permanent coverage and failed-stop
  cleanup still attempted teardown.
- Final FIT review: **NO-GO** at
  `8f1e54aaf87bbaee018a4986592114ea9f6178d5`, reviewed-runtime
  `eb47a400271fd879c537634e14f7047891354659e3f573bc3a2f0ccdc40ac6fc`,
  reviewer session `01a1b854-3cf8-7781-9a8d-658b19fffd58`, because detached
  submission could lose its job identity, the guard stopped early, one progress
  mutation was missing, and initialization was validated after training.
- Next FIT review: **NO-GO** at
  `1f280b4cbb73784b7be06db35512e62bc4295a5d`, reviewed-runtime
  `4125e86c9b99915e9f932140e228f3890a14f5644f034bcb897f91776f586894`,
  reviewer session `01a1b85e-a292-7363-b1bb-019a0ab91bfd`, because the
  micro-batch-8 retry reused attempt 1's initialization approval.
- Fresh non-author FIT: **GO** at
  `0ae6a64113dc2e81c25bbfc6cf5d7b027cba68a8`, reviewed-runtime
  `b1ac030988746b155a36007ef32e79649a0beaaf2d6932f9250124bb59b4bfbe`,
  reviewer session `01a1b864-d9e9-74b2-b224-de022d5d9de4`. No FIT blockers
  remained.
- D67 pre-run review: **NO-GO** at
  `5cf410bed630d926023181b9f7875bd8963a71cd`, reviewed-runtime
  `b1ac030988746b155a36007ef32e79649a0beaaf2d6932f9250124bb59b4bfbe`,
  reviewer session `01a1b86a-98d9-7483-9a6f-7da2d0da013c`, because the order
  retained the retired early guard stop and "best checkpoint" was undefined.
- Post-reconciliation FIT review: **NO-GO** at
  `a668947d4e6243967bf3e74fa928486aa7d89f1e`, reviewed-runtime
  `43791d32d7883925a2b48d5f0ae6e0a99ddbc3c011993449a1b583d91a049740`,
  reviewer session `01a1b872-d7dc-7930-a937-2cb0d1dc1215`, because one stop
  sentence, the free-generation rendering, and selector regression coverage
  were still ambiguous.
- Fresh main-integrated FIT: **GO** at
  `ec5eed0ac53960c60fe2ae72339bbd67289a0d6d`, reviewed-runtime
  `0f8f1952d260e69d49cb9c4375f5af1cddfb58a8fc392a176d06818c8526f8a8`,
  reviewer session `01a1b87e-f1d7-7552-a819-ddf861671a74`. No FIT blockers
  remained.
- D67: **NO-GO** at `6685b859b9e7d02132b8dd54a496fd5c177c42ea`,
  reviewed-runtime
  `0f8f1952d260e69d49cb9c4375f5af1cddfb58a8fc392a176d06818c8526f8a8`,
  reviewer session `01a1b88b-503a-7283-ae4f-15ab17056b60`, because the active
  controller-repair ORDER did not authorize pod launch.
- Fresh FIT after adding deepseek-26 and merging current main: **GO** at
  `d358ecb848adf8e84ef22ab36d5e6b6c7b59880e`, reviewed-runtime
  `044120d4bc348aec934028a7fc77f70831b41a745cf601f1639abc516541089a`,
  reviewer session `01a1b894-3057-7ce1-af5f-b0a9fc06e2ba`. No FIT blockers
  remained.
- Fresh D67 pre-run review: **GO** at
  `1902f0b2b2c4de57ecef59dc4bd1c1fee901857f`, reviewed-runtime
  `044120d4bc348aec934028a7fc77f70831b41a745cf601f1639abc516541089a`,
  reviewer session `01a1b89e-ade6-7b03-ba5f-d19cdbd091ef`. No blockers
  remained; live lease, pricing, merged-pin and on-pod hash checks remain
  mandatory dispatch rechecks.
- Added deepseek-26 as the active execution ORDER for seed 1. It carries no
  threshold or mechanism change and explicitly refuses seed 2, M2 substitution,
  another lab's pod, or any unrecorded input.
- `gate.sh`: 205 checks passed after pinning the reconciled stop sentence,
  per-turn source, guard-safe filter, first selected result, rendering label and
  full selector ordering.
- `test-mutants.sh`: 76 of 76 semantic mutations caught.
- Direct evaluation of the progress predicate accepted null and absent step-0
  training loss; it rejected finite, infinite and string step-0 training loss,
  non-finite positive-step training loss, and null or non-finite DEV loss,
  token margin and address margin.
- `CARGO_TARGET_DIR=target cargo test --offline --locked -p uor-r4-float-train
  --lib`: 308 passed, including `checkpoint_ready_marker_records_completion`
  and `initialization_record_is_ready_before_controller_approval`.
- The bare suite invocation omitted `CARGO_TARGET_DIR`; the existing save/load
  test requires an owned artifact directory and failed that precondition. The
  environment-qualified run above passed. The pod job exports the same target
  variable before building and testing.
- `bash -n` and ShellCheck on `job.sh` and `run-s1.sh`: passed.
- `git diff --check`: passed.

Not run: pod creation, remote build, model training or store write.

## Skills used

No system-provided Codex skill applied. The repository's loop, compute and
shell-controller rules were used directly.

## Still open

- Fresh FIT and D67 reviews of the execution-ORDER runtime, then the seed-1
  run.
- `m2-reply512` remains unavailable unless exact bytes are recovered.

## Plain summary

The SFT controller now waits for the actual remote training job, rather than
mistaking job submission for completion.
