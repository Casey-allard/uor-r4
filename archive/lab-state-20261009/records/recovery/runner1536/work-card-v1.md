# Runner stable boot and UNKNOWN path repair

Issue #1536; parent #1520; programme #820. Source base61bbd85a0a49e1bb72c71b4e02752d575e3bee4b; adopted policyba266ad44f21c9df2dafa9d56a1e2051c1565d91.
Observed blocker: clock-adjusted kern.boottime falsely triggered identity loss/reboot; live UNKNOWN finalization moved IPC paths, losing actual wrapper exit. Independent reconciliation is retained beside this card.
Deliverable: validated stable boot-session identity with fail-closed legacy handling; preserve UNKNOWN runtime paths and child handles until positive stop; immutable outcome and exactly-once accounting; focused regressions; protected reviewed delivery/deployment; reconcile existing Claude attempt and bounded continuity pilot.
Scope: tools/lab-runner/src/process.rs, jobs.rs, daemon.rs and focused existing tests/README. No model source, gates, resource floors or sealed evaluation content.
Decision: regressions and independent review must resolve identity/stop/path/accounting risks before deployment. UNKNOWN stays UNKNOWN. A new model run cannot substitute for a host recovery test.
Projection: <=90min complete preparation/implementation/review/delivery; any build/tests <=15min, CARGO_BUILD_JOBS=2, one Cargo process, <=2GiB declared RSS, <=2GiB new internal cache plus64MiB receipts. Reuse verified existing runner cache. No model/GPU/provider request, deletion, security override or paid compute.
Admission: source work proceeds while production admissions remain held. Defective production observation cannot safely supervise its own repair. Before an exceptional bounded local repair build, publish exact argv/source/cache/process/host observations and obtain prospective council review of the narrow recovery execution packet; do not silently bypass the hold. All other jobs stay held.
Tests: stable UUID same/different/invalid and legacy mutable strings; live legacy process cannot prove absence or authorize a signal; UNKNOWN persistence/replay leave live payload path/FIFO intact; later explicit stop/reconciliation preserves one receipt and charges once; existing affected runner suite.
Review: independent systems and second council passes; author/integrator Codex. Delegated source implementation must keep these exact boundaries.
Next: implement source/tests; freeze candidate and recovery-execution packet; independent review before any repair build.
