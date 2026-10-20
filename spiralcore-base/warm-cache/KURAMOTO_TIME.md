# Kuramoto Time — serialization, witness hashing and Merkle roots

Date: 2026-10-19. Status: design addition. This replaces the earlier use of
Kuramoto as a pacing/congestion heuristic. Kuramoto now defines the logical
time base for warm-cache commitment, witness serialization and Merkle roots,
and it synchronizes time across architecture levels.

## 1. Why a phase clock, not wall time

A cache commitment needs a time value both peers can agree on, with a
monotonic order and a local freshness signal. Wall clocks drift and are not
synchronized at each level. Kuramoto oscillators give:

- a local phase vector theta_i(t);
- an order parameter R e^{i psi}, where R is coherence and psi is the mean
  phase;
- a monotonic epoch when psi crosses 2*pi under coherence;
- a phase bin q that describes where inside the epoch the commitment occurred;
- a natural way to couple a child level to a parent level.

The clock is logical: (level, epoch, phase bin, coherence), not a timestamp
from an external server. Wall time is only a coarse bootstrap/holdover aid.

## 2. Hierarchical time

Levels (defaults; counts must be chosen):

    L0 core:    4 Kuramoto oscillators per node
    L1 node:    oscillators = core phases in that node
    L2 edge:    oscillators = node phases in that edge
    L3 region/global: oscillators = edge phases

Each level l has:

- phases theta_i^l(t);
- order parameter R_l e^{i psi_l};
- coupling K_l and natural frequencies omega_i^l;
- epoch counter e_l, incremented when unwrapped psi_l crosses 2*pi;
- phase bin q_l = floor(((psi_l mod 2*pi) / 2*pi) * 2^B_l);
- coherence code c_l = quantize(R_l);
- drift estimate d_l.

A level commits only when R_l >= R_min for W consecutive cycles and the phase
difference to its parent is within tolerance. If not, the level holds its last
committed epoch and requests a sync beacon; it does not commit a new time tag.

The time header is:

    T = ( (level 0: e0, q0, c0), (level 1: e1, q1, c1), ..., parent_epoch )

Peers exchange T in a small TIME_SYNC beacon. A child level phase-locks to the
parent phase reference, so e_l and q_l stay aligned. A global time_id is

    time_id = H( canonical_binary(T) )

but the actual witness includes T explicitly, not only time_id.

## 3. Serialization order

Operations, pages and Merkle leaves are serialized in a deterministic order:

    (e_L, q_L, e_{L-1}, q_{L-1}, ..., e_0, q_0, level, node_id, page_id, seq)

If two peers are synchronized, this order is identical. If they are not, the
time header differs and they do not commit; they exchange TIME_SYNC first.
This is the serialization role of Kuramoto time: it turns concurrent updates
into a total order without a central timestamp server.

## 4. Witness hashing

Two roots are kept:

1. Content root C (time-independent)
   - leaf = H(0x00 || page_id || page_version || digest(page_bytes))
   - node = H(0x01 || left || right)
   - C = root. Used for cache identity, dedup and seed_id.

2. Temporal root W (Kuramoto-time dependent)
   - temporal leaf = H(0x10 || level || e_l || q_l || c_l || content_leaf)
   - temporal node = H(0x11 || level || e_l || q_l || c_l || left || right)
   - W = root. Used for commitment, freshness, replay rejection and repair
     ordering.

The witness for a warm transfer is:

    witness = H(0x20 || W || canonical_binary(T) || parent_time_id)
    capsule = first 8 bytes of witness   (fast accidental check)
    full witness = 32 bytes              (repair/commit check)

The cryptographic hash remains SHA-256 (or the project's chosen strong hash).
Kuramoto supplies the synchronized time serialization; it is not collision
resistance and must not replace the hash.

Why both roots: content identity must survive a clock epoch change for dedup
and seed_id stability; freshness and ordering must change with time. The
content root answers "what is this"; the temporal root answers "when/ordered
as what".

## 5. Merkle tree shape

    page (4 KiB)              leaf: content digest + page version
      -> page group (e.g. 64 pages)
      -> node root (L1 time tag)
      -> edge root (L2 time tag)
      -> region/global root (L3 time tag)
      -> temporal root W
      -> witness

At each level the temporal node includes that level's epoch/phase/coherence.
A mismatch at a level localizes which level's clock or which page subtree
diverged. Repair can then be page-level (same epoch) or time-sync first
(different epoch).

## 6. Cold send / warm send with Kuramoto time

Cold send:
1. Exchange TIME_SYNC to establish T0 and the time domain at each level.
2. Send the seed bundle, plan, content root C and initial temporal root W0.
3. Receiver materializes the seed, verifies C and W0, stores seed_id and T0.

Warm send:
1. Sender includes T (epoch vector, phase bins, coherence) and plan_id.
2. Receiver checks T freshness and coherence; if stale or unsynchronized, it
   responds with TIME_SYNC, no data.
3. Receiver applies the plan to cached state.
4. Receiver computes the content result, then the temporal root W with T.
5. If W matches the sender's witness, commit with zero payload.
6. If W differs, compare temporal subtrees; first check whether the difference
   is time (clock) or content (data). Time mismatch -> sync; content mismatch
   -> page/leaf repair.
7. Assumption certificates expire by epoch: an assumption valid at epoch e is
   not silently reused at e+1.

This is how Kuramoto time reduces compute and resends: peers commit on a
shared logical time, so they know whether a disagreement is data divergence or
clock divergence before doing deep comparison.

## 7. Multi-level synchronization protocol

- Each level broadcasts a TIME_SYNC beacon:
  (level, epoch, phase_q, coherence_q, drift, parent_epoch, time_id).
- Child levels phase-lock to the parent reference:
  dtheta_i/dt = omega_i + (K_l/N_l) * sum_j sin(theta_j - theta_i)
               + epsilon_l * sin(psi_parent - theta_i)
- A level commits an epoch only when coherence and parent alignment pass; it
  holds otherwise.
- Beacons are tiny and piggyback on warm frames or repair frames when possible;
  a standalone beacon is sent only when drift exceeds tolerance.
- On partition, each side continues its local epoch but marks the parent epoch
  stale; on reconnect, exchange T, align epochs, then commit.
- The AURA gate is reused here as the **epoch commit gate**: R >= R_min and
  phase margin. It is not a data legality gate. The earlier empty-band result
  is fine: the gate wants high coherence, not a band around 0.82.

## 8. Failure modes and tests

Failure modes:
- frequency offset / drift -> phase error grows; fix with parent coupling and
  periodic beacon.
- low coherence -> no commit; hold and resync.
- epoch fork / partition -> parent_epoch mismatch; hold, exchange T, repair.
- replay / stale witness -> epoch/temporal root mismatch; reject.
- same content, new epoch -> content root matches, temporal root differs; this
  is expected; cache dedup uses C, freshness uses W.
- malicious clock -> not prevented by Kuramoto; authentication/signature of
  TIME_SYNC is a separate layer.

Tests to run:
1. Hierarchical Kuramoto simulation: 3 levels, 4 cores/node, 8-16 nodes/edge,
   frequency offsets and noise. Measure R_l, phase error, commit rate, epoch
   skew, time to resync.
2. Witness/Merkle vectors: same content in two epochs => C equal, W differs;
   same epoch different content => both differ; tampered leaf localized.
3. Serialization determinism: two peers with identical T produce identical
   leaf order and roots.
4. Repartition: hold, then reconnect; no commit before alignment; no data loss.
5. Wire overhead: TIME_SYNC beacon size and frequency; piggyback ratio.
6. Cold/warm: zero-payload warm on identical state; repair on content
   divergence; sync-only on clock divergence.

## 9. Defaults and open decisions

Defaults:
- levels L0 core (4), L1 node, L2 edge, L3 region;
- B_l phase bits: L0 10, L1 12, L2 14, L3 16;
- R_min: 0.95 for commit; W = 3 cycles; phase tolerance from B_l;
- content root for seed_id; temporal root for witness;
- AURA gate used only as epoch-commit coherence gate;
- SHA-256 for both roots; 8-byte capsule = truncated temporal witness.

Open decisions:
1. Exact level topology and oscillator counts; how many cores per node, nodes per
   edge, edges per region?
2. Phase bits B_l and commit frequency (epochs per second) per level; this sets
   time resolution and beacon rate.
3. R_min, W and phase tolerance; what is the acceptable false-commit/abort rate?
4. Bootstrap: is wall/PTP available for coarse initial epoch, or must Kuramoto
   cold-start from the seed bundle?
5. Should warm sends require exact epoch equality, or accept a parent epoch
   within tolerance?
6. Are TIME_SYNC beacons authenticated/signed, or is the network trusted?

## 10. Recommended counts and derived parameters (added 2026-10-19)

### Oscillator counts

Use four oscillators per time/decision cell, and a 4-ary hierarchy:

    L0: 4 oscillators per core cell
    L1: 4 core phases per node
    L2: 4 node phases per edge
    L3: 4 edge phases per region

Why four:
- Four phases have 4! = 24 strict orderings. The Bell field has 15 lawful
  codes plus one reserved code = 16; 24 orderings cover that and leave 8
  counterfactual slots. This is a natural four-oscillator decision cell.
- Four oscillators give six pairwise phase differences for timing and a
  three-of-four consistency check for one outlier, without a fifth oscillator.
- Three oscillators have too few orderings and no spare route; five add a
  redundant observer but not decision capacity. Add a fifth observer only if
  fault detection or Byzantine consensus is required; it is not needed for the
  deterministic time cell.

A four-oscillator cell is a time/decision cell, not a full ALU by itself. It
provides the phase order and clock that a four-phase virtual CPU uses; the
operation plan supplies the arithmetic/transform semantics.

### B_l from the inter-oscillator time difference

Let delta_l be the smallest phase difference that must be distinguished at
level l (the phase spread/offset envelope, or the Bell counterfactual margin if
that is smaller). Then

    B_l = ceil(log2(2*pi / delta_l)) + 1 safety bit
    tau_l = (2*pi / omega_l) / 2^B_l   (time-bin width)

Equivalently, if the observed time difference between oscillators is dt_l and
the level period is T_l:

    B_l = ceil(log2(T_l / dt_l)) + 1 safety bit

B_l is measured at runtime from the actual oscillator offsets; it is not a
fixed constant. The current 10/12/14/16 defaults are upper bounds, not derived
values.

### Bell counterfactual margin

If the Bell codes are mapped to equally spaced phase positions, the nearest
lawful-but-unselected route is at

    m = 2*pi/15 = 0.418879 rad
    half margin = 0.209440 rad
    third margin = 0.139626 rad

If the actual mapping is not equally spaced, replace m by the measured
minimum distance between the selected route and any lawful unselected route.
No statistical calibration is needed: the Bell counterfactual supplies the
decision margin.

### R_min and W from offsets, not from false-commit statistics

A commit is deterministic: the level may commit only when the observed phase
spread is inside the margin. Define sigma_allow = min(m/2, measured margin) and
sigma_max = measured worst-case spread for the current offsets. Then

    R_min = sin(sigma_allow)/sigma_allow      (uniform bounded spread)
    R_min = exp(-sigma_allow^2/2)             (von Mises approximation)
    use the smaller of the two for safety

and

    W = 1 in steady state, if the offset envelope is inside sigma_allow
    W = ceil(settling_cycles) at bootstrap/lock acquisition

W is not a probabilistic confidence knob. If sigma_max >= sigma_allow, the
cell does not commit at all; it tightens coupling or reduces the offset
envelope. That is the deterministic no-false-commit rule.

### Four-oscillator offset study

Normalized 4-oscillator Kuramoto cell, K=20, 60 s simulation, 10 s burn-in,
1 Hz nominal, offsets in cycles/s:

| offsets (Hz) | max phase spread (rad) | R_min measured | inside half Bell margin? |
|---|---:|---:|---|
| 0, 0, 0, 0 | 0.000 | 1.0000 | yes |
| -0.05, -0.02, 0.02, 0.05 | 0.0314 | 0.99993 | yes |
| -0.2, -0.07, 0.07, 0.2 | 0.1259 | 0.99889 | yes |
| -0.5, -0.17, 0.17, 0.5 | 0.3177 | 0.99298 | no (spread > 0.2094) |
| -1.0, -0.33, 0.33, 1.0 | 0.6595 | 0.97020 | no |
| -2.0, -0.67, 0.67, 2.0 | 1.9092 | 0.76992 | no |

Bell margin example: m=2*pi/15=0.4189 rad; half=0.2094. B=5 resolves the
half margin, B=6 resolves the third margin; add one safety bit. R_min at half
margin is 0.99271 (uniform) or 0.97831 (von Mises); at third margin 0.99675 or
0.99030.

So the answer to "what oscillator count": four per cell, four cells per parent.
The answer to "what B/R_min/W": derive them from the measured offset envelope
and the Bell counterfactual, not from a hand-set statistical threshold.

### Root decision (content vs temporal) deferred

The dual-root question is decided after measuring:

1. Root churn: how often the temporal root changes per epoch; cost of signing/
   storing it.
2. Cache identity: whether the same content must keep a stable seed_id across
   epochs (dedup/object-store requirement).
3. Mismatch classification: whether a failed witness is clock-only, content-only
   or both. A content root plus a temporal root separates these; one temporal
   root confounds them.
4. Repair localization: whether temporal subtrees localize clock vs data
   divergence faster than a single root.
5. Warm frame size and verification time for 1 vs 2 roots.

Experiment matrix:
- same content, next epoch: C stable, W changes; measure dedup hit and witness
  classification.
- same epoch, content change: C and W both change; measure repair localization.
- clock drift only: W changes, C stable; measure time-vs-data classification.
- partition/reconnect: hold both roots; measure resync and replay rejection.
- high epoch rate: measure root update cost and whether a single root is cheaper.

Default until the matrix is measured: keep C for seed_id/dedup and W for
witness; the temporal root is the commitment. The final choice follows the data.


## 11. Forward -> reverse -> forward -> reverse: test and layering

### What is mathematically true

For Kuramoto

    dtheta_i/dt = omega_i + (K/N) sum_j sin(theta_j - theta_i)

the reversed trajectory is

    phi_i(t) = -theta_i(T - t)

and it obeys the negative-coupling equation

    dphi_i/dt = omega_i - (K/N) sum_j sin(phi_j - phi_i)

with the same natural frequencies. If omega=0 this is an exact symmetry; if
omega is nonzero the reverse path needs the frequency signs reversed as well.
So the oscillator can represent forward and reverse time as two trajectories of
the same phase model, but not with the same stable coupling in both directions.

### The stability problem

A four-oscillator RK4 test (dt=1e-4, K=20, T=2 s, zero natural frequencies)
showed:

- the exact reversed trajectory satisfies the reverse ODE with max residual
  about 8e-6 (finite-difference/integration error);
- integrating the reverse ODE numerically from the exact reversed state gives
  phase error 1e-12 at 0.1 s, 1e-11 at 0.2 s, 4.8e-9 at 0.5 s, 1.05e-4 at
  1.0 s and 1.375 rad at 2.0 s;
- a 1e-6 perturbation of the reversed state grows to 1.375 rad after 2 s;
- replaying the stored forward trajectory backwards is exact (0.0 error);
- a forward-reverse-forward-reverse replay returns exactly to the start.

Negative coupling is unstable. The reverse branch is a valid mathematical
trajectory but not an attracting clock. A literal autonomous reverse-time
Kuramoto oscillator will not stay locked; it is a saddle/replay mode.

### Consequence

Use forward -> reverse -> forward -> reverse as a **calibration/verification
super-epoch**, not as a second autonomous time direction:

    F1 forward sweep, record phase history
    R1 replay the recorded phase backward, compare
    F2 forward sweep, compare against R1
    R2 replay backward, compare against F2
    commit at super-epoch end with the paired witness

This is a chopper-style bidirectional measurement: forward and reverse
residuals cancel symmetric offsets and expose delay asymmetry. It does not
replace the monotonic logical clock.

### Layering changes

1. Add an orientation/sweep layer above the oscillator:

       TimeHeader = (level, super_epoch, sweep_id, direction, epoch, q, c)

   The oscillator still supplies phase; the orientation layer maps a sweep to a
   logical tick. For a replay reverse sweep, map descending physical phase to
   ascending logical time:

       logical_tick = super_epoch*4 + sweep_id*2^B + (direction==F ? q : 2^B-1-q)

   This keeps causality and Merkle order monotonic even though the physical
   phase is replayed backward.

2. Temporal roots become paired:

       W_F = temporal root of the forward sweep
       W_R = temporal root of the reverse sweep
       W_pair = H(0x21 || W_F || W_R || super_epoch)
       witness = H(0x22 || W_pair || canonical(T) || parent_time_id)

   Content root C stays time-free. A warm commit can use W_pair for the
   calibration epoch and a single W for ordinary epochs. This adds a third
   root option to the root decision matrix: content, single temporal, paired.

3. Sync beacons carry both directions:

       TIME_SYNC { super_epoch, sweep, direction, phase_q, coherence_q,
                   drift, parent_epoch }

   The forward/reverse pair estimates phase offset and path-delay asymmetry.
   Commit only when the two directions agree inside the Bell margin.

4. Cold/warm path:
   - cold send: one forward sweep plus initial reverse verification;
   - periodic calibration: F-R-F-R super-epochs every N epochs or on drift
     alarm;
   - ordinary warm send: single-direction witness with a fresh calibration
     tag; no reverse sweep per transfer;
   - if reverse verification disagrees, hold commit and request repair/sync.

5. Cost: a reverse replay needs a phase-history buffer per level. For N
   oscillators, B bits, one super-epoch, the buffer is about N*B bits per
   level (plus samples if interpolating). A paired counter-rotating oscillator
   bank is the alternative, but it doubles oscillator count and needs a
   stability design. Replay is the cheaper first implementation.

6. Four-phase cell: four oscillators remain the cell. The F-R-F-R sequence is
   four sweeps of the same four oscillators, not four new oscillators. If true
   counter-rotating hardware is used, keep eight oscillators per cell (two
   banks) and treat the reverse bank as a witness, not as an independent time
   source.

### Does this change the root decision?

Yes. The candidates become:

- C only: stable seed identity, but no time freshness or clock/data
  classification.
- W only: temporal commitment, but cache identity churns and clock-vs-data
  mismatch is confounded.
- C + W: separates identity from freshness.
- C + W_pair: separates identity from freshness and also cancels sweep
  direction/offset; best for calibration epochs, more bytes and roots.

The experiment matrix from section 10 gains two rows:
- forward vs reverse witness on unchanged content: measure offset cancellation
  and false-commit rate;
- F-R-F-R super-epoch cost vs single sweep: added latency, buffer, root bytes.

Decision rule after measurement: if reverse calibration materially reduces
clock-only mismatches and the paired root fits the warm budget, use C + W_pair
at calibration boundaries and C + W for ordinary sends. If not, keep C + W.


## 12. Chiral tangent flip (proposed extension)

The pure forward/reverse flip is a symmetry, not a stable clock. The chiral
cell adds the missing degrees of freedom without adding oscillators:

    state per oscillator: (theta_i, v_i, chi_i)
    chi_i in {+1,-1}
    C: (theta_i, v_i, chi) -> (-theta_i, -v_i, -chi)
    reversibility condition: f(C x) = -C f(x)

A candidate cell:

    theta_i' = v_i
    v_i' = -gamma v_i + omega_i + (K/N) sum_j sin(theta_j - theta_i)
           + chi * tau_g * (J v)_i
           + chi * tau_c * (theta_{i+1} + theta_{i-1} - 2 theta_i)

where J is skew-symmetric (gyroscopic tangent term), tau_c is the
centrifugal/centripetal torsion, and the torsion terms are zero-sum over the
cell so they do not shift the mean phase. Under C the gamma/damping term must
flip sign or be balanced; otherwise the reverse branch is a saddle. That is
the exact place where a chiral cell can fail: a damped forward attractor does
not become a damped reverse attractor unless the loss/gain is balanced.

What this gives:
- exact reversibility if the torsion is odd under C and the damping is balanced;
- a possible stable reverse branch if the gyroscopic/torsion term supplies the
  missing stabilization (this is the experiment);
- four oscillators still: tangent and chirality are state registers, not extra
  oscillators; if only scalar phases exist, no stable autonomous reverse.

Canonical Kuramoto time is preserved only if the torsion does no net work on
psi and R. Keep R and psi as the clock; chirality is a branch tag. The temporal
header gains chi, the temporal root can be paired as W_pair = H(W_+, W_-), and
the content root stays time-free. The root decision matrix gains the paired
candidate; it is still decided by measurement.

Simulation to run: four-oscillator (theta, v, chi) cell; verify C symmetry
residual; perturb the reverse state by 1e-6; sweep tau_c (centrifugal and
centripetal signs), tau_g, and balanced gain/loss; measure reverse stability,
R, psi drift and phase error. If the reverse branch is stable and psi/R are
unchanged, the chiral tangent flip is viable and the oscillator count stays
four. If not, use replay or a paired bank.


## 13. Chiral-cell attempt: measured result (2026-10-19)

The proposed four-oscillator chiral cell was implemented as a second-order
Kuramoto system:

    theta' = v
    v' = -gamma v + omega + (K/N) sum_j sin(theta_j - theta_i)
         + tau_c (theta_{i+1}+theta_{i-1}-2 theta_i)
         + tau_g (theta_{i+1}-theta_{i-1})

Forward uses -gamma v; the reverse branch uses +gamma v (anti-damped) and the
same torsion terms. The reverse target was the exact time reverse of a stored
forward trajectory: theta_f(T-t), -v_f(T-t). The reverse was started with a
1e-6 perturbation, and the error was measured against the exact target for
T=0.5 s, dt=5e-5.

Grid: gamma in {0.0, 0.5}; tau_c and tau_g in {-5, 0, 5}.

Best result: no torsion, gamma=0.5, perturbed error 1.36e-4.
Torsion results: 3.09e-4 to 1.20e-2; every tested centrifugal/centripetal or
gyroscopic torsion made the reverse error worse. No grid point stabilized the
reverse branch.

Interpretation: in this model, chirality/tangent state plus simple
torsion does not remove the saddle. The reverse is exact as a stored-target
trajectory but not stable under perturbation. Four oscillators can carry
(theta, v, chi), but carrying the state is not the same as making the reverse
branch an attractor.

What still might work:
- balanced gain/loss (the gamma sign flip is explicit; simple +gamma in reverse
  is anti-damped);
- a phase-negating canonical flip with different torsion signs;
- a nonlinear saturating controller or a phase-conjugate pair;
- replay of the stored forward trajectory, which is exact (0.0 error) and is
  the fallback calibration mode.

So the current result is negative for the simple chiral-torsion cell, not a
proof that no chiral construction can work. The replay path remains exact and
can be used while the balanced-gain/loss and phase-conjugate variants are
tested.


## 14. Chiral-cell follow-up: balanced gain/loss, phase-negating flip, long window

Three follow-ups were tested. All are negative for an autonomous stable
reverse; replay remains exact.

### Balanced gain/loss (preserve mode)

Damping patterns: all +0.5; all -0.5; alternating +0.5/-0.5; half/half;
+0.5,0,0,-0.5. Torsion combinations in {-5,0,5} squared. T=0.4 s, dt=5e-5,
1e-6 perturbation.

Best result: all +0.5, no torsion, error 2.57e-4 (R 0.292 at 0.4 s).
The balanced patterns did not beat the uniform damped cell; every torsion
combination was worse. Balanced gain/loss alone did not make reverse stable.

### Phase-negating flip

The phase-negating reverse (theta_R = -theta_f(T-t), v_R = v_f(T-t), forces
negated) failed badly: errors 1.34 to 9 rad across the grid. The phase-conjugate
formulation is not a viable reverse clock in this model.

### Long window, no torsion

20 s forward, dt=2e-4, 1e-6 perturbation, three seeds, uniform gamma=0.5:

| seed | forward R | reverse R final | reverse error max |
|---|---:|---:|---:|
| 1 | 0.9999 | 0.3238 | 0.01115 |
| 2 | 1.0000 | 0.9292 | 0.01103 |
| 3 | 1.0000 | 0.6278 | 0.01109 |

The reverse error grows from about 1e-3 at 5 s to about 1.1e-2 at 20 s, and
the reverse coherence falls below the forward clock. The tangent/inertia delays
the divergence but does not remove it. Torsion variants at 5 s were worse:
tau_c=-5 and several tau_g signs blew up (vmax up to 1e9); tau_c=+5 gave a
small 5 s error but no evidence of long-window stability.

### Interpretation

- Four oscillators can carry (theta, v, chi), but the simple chiral/tangent/
torsion cell does not produce a stable autonomous reverse.
- The tangent/inertia is necessary but insufficient; the reverse is still a
time-reversed dissipative trajectory, not an attractor.
- Balanced gain/loss without a nonlinear saturating controller is not enough.
- Phase-negating flip is worse than phase-preserving reverse.
- Replay of stored (theta, v) is exact and remains the production-safe F-R-F-R
mechanism.

Next variants, if pursued: nonlinear saturating gain/loss, a phase-conjugate
oscillator pair with a controller, or a genuinely conservative reversible
cell (which gives exact reverse but may not synchronize as a clock).
