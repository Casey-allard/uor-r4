//! Pure receipt lifecycle for the registered whole-interval admission audit.
//! No model execution or filesystem durability is implemented by this module.

use crate::{corpus::sha256, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const AUDIT_SCHEMA: &str = "uor-r4.attention-whole-interval-audit/v1";
pub const CADENCE: u64 = 64;
pub const MAX_BATCHES: u64 = 16_404;
pub const Q1_BATCHES: u64 = 4_096;
pub const ANCHOR_ROWS: u64 = 3_847;

fn invalid(s: &str) -> Error {
    Error::Invalid(format!("attention audit: {s}"))
}
fn digest<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_vec(value)
        .map(|v| sha256(&v))
        .map_err(|_| invalid("receipt encoding failed"))
}
fn hash_valid(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn require_hash(s: &str) -> Result<()> {
    if hash_valid(s) {
        Ok(())
    } else {
        Err(invalid("invalid lowercase SHA256"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunBinding {
    pub plan_sha256: String,
    pub release_sha256: String,
    pub lineage_sha256: String,
    pub sheet_sha256: String,
    pub anchor_sha256: String,
    /// Bound schedule inventory; numerical family membership is not inferred
    /// from a caller-provided family label.
    pub combine_gates: u32,
    pub relation_gates: u32,
}
impl RunBinding {
    fn validate(&self) -> Result<()> {
        for h in [
            &self.plan_sha256,
            &self.release_sha256,
            &self.lineage_sha256,
            &self.sheet_sha256,
            &self.anchor_sha256,
        ] {
            require_hash(h)?;
        }
        if self.combine_gates == 0
            || self.relation_gates == 0
            || self
                .combine_gates
                .checked_add(self.relation_gates)
                .is_none()
        {
            return Err(invalid("invalid bound gate inventory"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotPin {
    pub checkpoint_sha256: String,
    pub storage_sha256: String,
    pub snapshot_sha256: String,
}
impl SnapshotPin {
    fn validate(&self) -> Result<()> {
        require_hash(&self.checkpoint_sha256)?;
        require_hash(&self.storage_sha256)?;
        require_hash(&self.snapshot_sha256)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Family {
    Vector,
    Combine,
    Relation,
}
impl Family {
    fn index(self) -> usize {
        match self {
            Self::Vector => 0,
            Self::Combine => 1,
            Self::Relation => 2,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FamilyCounters {
    pub attempted: u64,
    pub sensitive: u64,
    pub policy_refused: u64,
    pub provisional_applied: u64,
    pub committed_applied: u64,
    pub rejected_applied: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateStamp {
    pub run_sha256: String,
    pub counters_sha256: String,
    pub completed: u64,
    pub epoch: u64,
    pub generation: u64,
    pub runtime_model: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepEvidence {
    pub before: StateStamp,
    pub family: Family,
    pub applied: bool,
    pub sensitive: bool,
    pub policy_refused: bool,
    pub artifact_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditEvidence {
    pub stamp: StateStamp,
    pub snapshot: SnapshotPin,
    pub admitted: u64,
    pub artifact_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PassingState {
    pub snapshot: SnapshotPin,
    pub epoch: u64,
    pub at_completed: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestoreRequirement {
    pub failed: StateStamp,
    pub failed_snapshot: SnapshotPin,
    pub passing: PassingState,
    pub next_generation: u64,
    pub audit_artifact_sha256: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestoreEvidence {
    pub required: RestoreRequirement,
    pub restored_snapshot: SnapshotPin,
    pub restored_epoch: u64,
    pub advanced_completed: u64,
    pub new_runtime_model: u64,
    pub artifact_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepDisposition {
    pub evidence: StepEvidence,
    pub rejected_interval: bool,
    pub rejected_applied: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntervalRecord {
    pub start: StateStamp,
    pub inspected: AuditEvidence,
    pub failed_snapshot: SnapshotPin,
    pub after: StateStamp,
    pub rejected: bool,
    pub steps: Vec<StepDisposition>,
    pub restoration: Option<RestoreEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    InstrumentUnreachable,
    RejectionRate,
    ConsecutiveRejections,
    ParentQ1,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "record", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum RecordKind {
    Baseline(AuditEvidence),
    Interval(Box<IntervalRecord>),
    ParentQ1 {
        stamp: StateStamp,
        passed: bool,
        artifact_sha256: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DurableRecord {
    pub schema: String,
    pub run_sha256: String,
    pub kind: RecordKind,
    pub stop_reasons: Vec<StopReason>,
}
impl DurableRecord {
    pub fn sha256(&self) -> Result<String> {
        digest(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditAction {
    Commit(DurableRecord),
    Restore(RestoreRequirement),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    AwaitBaseline,
    Ready,
    Open,
    AwaitBoundary,
    AwaitAudit,
    AwaitRestore,
    AwaitDurable,
    AwaitQ1,
    Complete,
    Stopped,
}

#[derive(Debug, Clone, Serialize)]
struct Progress {
    completed: u64,
    epoch: u64,
    generation: u64,
    runtime_model: u64,
    counters: [FamilyCounters; 3],
    baseline: Option<u64>,
    passing: Option<PassingState>,
    audited_intervals: u64,
    rejected_intervals: u64,
    consecutive_full_rejections: u64,
    reasons: Vec<StopReason>,
    q1_passed: bool,
}
impl Progress {
    fn stamp(&self, run: &str) -> Result<StateStamp> {
        Ok(StateStamp {
            run_sha256: run.into(),
            counters_sha256: digest(&self.counters)?,
            completed: self.completed,
            epoch: self.epoch,
            generation: self.generation,
            runtime_model: self.runtime_model,
        })
    }
    fn committed(&self) -> u64 {
        self.counters.iter().map(|c| c.committed_applied).sum()
    }
    fn reason(&mut self, r: StopReason) {
        if !self.reasons.contains(&r) {
            self.reasons.push(r);
        }
    }
    fn no_op_checks(&mut self, full_interval: bool) {
        if full_interval && self.consecutive_full_rejections >= 3 {
            self.reason(StopReason::ConsecutiveRejections);
        }
        if (self.completed == Q1_BATCHES
            || self.completed == MAX_BATCHES
            || !self.reasons.is_empty())
            && self.rejected_intervals > self.audited_intervals / 2
        {
            self.reason(StopReason::RejectionRate);
        }
    }
}

/// Runtime-only state: deliberately NOT Deserialize. Loading arbitrary serialized
/// state cannot assert that a pending receipt was durably written or a Device
/// was restored. A future controller must replay authenticated settled records;
/// this module grants no crash resume or retry permission.
#[derive(Debug, Serialize)]
pub struct AuditMachine {
    binding: RunBinding,
    run_sha256: String,
    frozen_zero: bool,
    state: Progress,
    phase: Phase,
    interval_start: Option<StateStamp>,
    steps: Vec<StepEvidence>,
    boundary: Option<SnapshotPin>,
    restore: Option<(RestoreRequirement, AuditEvidence)>,
    pending: Option<(DurableRecord, Progress)>,
    used_artifacts: BTreeSet<String>,
}

impl AuditMachine {
    pub fn new(binding: RunBinding, frozen_zero: bool, runtime_model: u64) -> Result<Self> {
        binding.validate()?;
        if runtime_model == 0 {
            return Err(invalid("runtime model identity is zero"));
        }
        let run_sha256 = digest(&(AUDIT_SCHEMA, &binding, frozen_zero))?;
        Ok(Self {
            binding,
            run_sha256,
            frozen_zero,
            state: Progress {
                completed: 0,
                epoch: 0,
                generation: 0,
                runtime_model,
                counters: [FamilyCounters::default(); 3],
                baseline: None,
                passing: None,
                audited_intervals: 0,
                rejected_intervals: 0,
                consecutive_full_rejections: 0,
                reasons: Vec::new(),
                q1_passed: false,
            },
            phase: Phase::AwaitBaseline,
            interval_start: None,
            steps: Vec::new(),
            boundary: None,
            restore: None,
            pending: None,
            used_artifacts: BTreeSet::new(),
        })
    }
    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn completed(&self) -> u64 {
        self.state.completed
    }
    pub fn runtime_model(&self) -> u64 {
        self.state.runtime_model
    }
    pub fn counters(&self) -> &[FamilyCounters; 3] {
        &self.state.counters
    }
    pub fn committed_updates(&self) -> u64 {
        self.state.committed()
    }
    pub fn stop_reasons(&self) -> &[StopReason] {
        &self.state.reasons
    }
    pub fn last_passing(&self) -> Option<&PassingState> {
        self.state.passing.as_ref()
    }
    pub fn stamp(&self) -> Result<StateStamp> {
        self.state.stamp(&self.run_sha256)
    }
    pub fn pending_record(&self) -> Option<&DurableRecord> {
        self.pending.as_ref().map(|p| &p.0)
    }
    pub fn expected_family(&self) -> Family {
        if !(self.state.completed + 1).is_multiple_of(8) {
            return Family::Vector;
        }
        let inventory =
            u64::from(self.binding.combine_gates) + u64::from(self.binding.relation_gates);
        if (self.state.completed / 8) % inventory < u64::from(self.binding.combine_gates) {
            Family::Combine
        } else {
            Family::Relation
        }
    }
    fn fresh_artifact(&self, h: &str) -> Result<()> {
        require_hash(h)?;
        if self.used_artifacts.contains(h) {
            return Err(invalid("artifact replay"));
        }
        Ok(())
    }
    fn validate_audit(&self, e: &AuditEvidence) -> Result<()> {
        if e.stamp != self.stamp()? || e.admitted > ANCHOR_ROWS {
            return Err(invalid("audit state/count mismatch"));
        }
        e.snapshot.validate()?;
        self.fresh_artifact(&e.artifact_sha256)
    }
    fn stage(&mut self, kind: RecordKind, next: Progress) -> Result<DurableRecord> {
        let record = DurableRecord {
            schema: AUDIT_SCHEMA.into(),
            run_sha256: self.run_sha256.clone(),
            kind,
            stop_reasons: next.reasons.clone(),
        };
        // Ensure serialization succeeds before changing any state.
        record.sha256()?;
        self.pending = Some((record.clone(), next));
        self.phase = Phase::AwaitDurable;
        Ok(record)
    }
    pub fn baseline(&mut self, e: AuditEvidence) -> Result<DurableRecord> {
        if self.phase != Phase::AwaitBaseline {
            return Err(invalid("baseline out of phase"));
        }
        self.validate_audit(&e)?;
        let mut next = self.state.clone();
        next.baseline = Some(e.admitted);
        next.passing = Some(PassingState {
            snapshot: e.snapshot.clone(),
            epoch: 0,
            at_completed: 0,
        });
        if e.admitted < 500 {
            next.reason(StopReason::InstrumentUnreachable);
        }
        self.stage(RecordKind::Baseline(e), next)
    }
    pub fn record_step(&mut self, e: StepEvidence) -> Result<()> {
        if !matches!(self.phase, Phase::Ready | Phase::Open) {
            return Err(invalid("transition blocked by lifecycle barrier"));
        }
        if e.before != self.stamp()? || e.family != self.expected_family() {
            return Err(invalid("transition state/family mismatch"));
        }
        self.fresh_artifact(&e.artifact_sha256)?;
        if (e.applied && (!e.sensitive || e.policy_refused))
            || e.policy_refused != (self.frozen_zero && e.family == Family::Relation)
        {
            return Err(invalid("proposal sensitivity/policy/apply mismatch"));
        }
        if self.state.completed >= MAX_BATCHES || self.steps.len() >= CADENCE as usize {
            return Err(invalid("attempt budget exceeded"));
        }
        let mut next = self.state.clone();
        let c = &mut next.counters[e.family.index()];
        c.attempted += 1;
        c.sensitive += u64::from(e.sensitive);
        c.policy_refused += u64::from(e.policy_refused);
        c.provisional_applied += u64::from(e.applied);
        next.completed += 1;
        next.epoch += u64::from(e.applied);
        // All arithmetic is bounded by the fixed attempted dose, validated above.
        let start = if self.phase == Phase::Ready {
            Some(e.before.clone())
        } else {
            self.interval_start.clone()
        };
        self.used_artifacts.insert(e.artifact_sha256.clone());
        self.steps.push(e);
        self.interval_start = start;
        self.state = next;
        self.phase = if self.state.completed.is_multiple_of(CADENCE)
            || self.state.completed == MAX_BATCHES
        {
            Phase::AwaitBoundary
        } else {
            Phase::Open
        };
        Ok(())
    }
    /// Call only after the actual device synchronization/status barrier. This
    /// records the supplied pin; the pure state machine cannot prove GPU sync.
    pub fn seal_boundary(&mut self, snapshot: SnapshotPin) -> Result<StateStamp> {
        if self.phase != Phase::AwaitBoundary {
            return Err(invalid("boundary out of phase"));
        }
        snapshot.validate()?;
        let stamp = self.stamp()?;
        self.boundary = Some(snapshot);
        self.phase = Phase::AwaitAudit;
        Ok(stamp)
    }
    pub fn audit(&mut self, e: AuditEvidence) -> Result<AuditAction> {
        if self.phase != Phase::AwaitAudit {
            return Err(invalid("audit out of phase"));
        }
        self.validate_audit(&e)?;
        if self.boundary.as_ref() != Some(&e.snapshot) {
            return Err(invalid("inspected checkpoint differs from sealed boundary"));
        }
        let baseline = self
            .state
            .baseline
            .ok_or_else(|| invalid("missing baseline"))?;
        if e.admitted < baseline - 24 {
            let required = RestoreRequirement {
                failed: e.stamp.clone(),
                failed_snapshot: e.snapshot.clone(),
                passing: self
                    .state
                    .passing
                    .clone()
                    .ok_or_else(|| invalid("missing passing checkpoint"))?,
                next_generation: self
                    .state
                    .generation
                    .checked_add(1)
                    .ok_or_else(|| invalid("restoration generation overflow"))?,
                audit_artifact_sha256: e.artifact_sha256.clone(),
            };
            self.restore = Some((required.clone(), e));
            self.phase = Phase::AwaitRestore;
            Ok(AuditAction::Restore(required))
        } else {
            let (kind, next) = self.interval_record(e, None)?;
            self.stage(kind, next).map(AuditAction::Commit)
        }
    }
    fn interval_record(
        &self,
        audit: AuditEvidence,
        restoration: Option<RestoreEvidence>,
    ) -> Result<(RecordKind, Progress)> {
        let rejected = restoration.is_some();
        let mut next = self.state.clone();
        for c in &mut next.counters {
            if rejected {
                c.rejected_applied += c.provisional_applied;
            } else {
                c.committed_applied += c.provisional_applied;
            }
            c.provisional_applied = 0;
        }
        next.epoch = next.committed();
        next.audited_intervals += 1;
        if rejected {
            next.rejected_intervals += 1;
            if self.steps.len() == CADENCE as usize {
                next.consecutive_full_rejections += 1;
            }
            let r = restoration
                .as_ref()
                .ok_or_else(|| invalid("missing restoration"))?;
            next.generation = r.required.next_generation;
            next.runtime_model = r.new_runtime_model;
        } else {
            next.consecutive_full_rejections = 0;
            next.passing = Some(PassingState {
                snapshot: audit.snapshot.clone(),
                epoch: next.epoch,
                at_completed: next.completed,
            });
        }
        next.no_op_checks(self.steps.len() == CADENCE as usize);
        let record = IntervalRecord {
            start: self
                .interval_start
                .clone()
                .ok_or_else(|| invalid("missing interval start"))?,
            failed_snapshot: audit.snapshot.clone(),
            inspected: audit,
            after: next.stamp(&self.run_sha256)?,
            rejected,
            steps: self
                .steps
                .iter()
                .cloned()
                .map(|e| {
                    let applied = e.applied;
                    StepDisposition {
                        evidence: e,
                        rejected_interval: rejected,
                        rejected_applied: rejected && applied,
                    }
                })
                .collect(),
            restoration,
        };
        Ok((RecordKind::Interval(Box::new(record)), next))
    }
    /// The controller must restore a fresh Device from the exact passing pin,
    /// drop stale scratch/proposals and provide its new process-local identity.
    /// This method validates receipt identity, not physical Device behavior.
    pub fn restored(&mut self, e: RestoreEvidence) -> Result<DurableRecord> {
        if self.phase != Phase::AwaitRestore {
            return Err(invalid("restore out of phase"));
        }
        let (required, audit) = self
            .restore
            .as_ref()
            .ok_or_else(|| invalid("missing restore requirement"))?;
        if &e.required != required
            || e.new_runtime_model <= self.state.runtime_model
            || e.artifact_sha256 == required.audit_artifact_sha256
            || e.restored_snapshot != required.passing.snapshot
            || e.restored_epoch != required.passing.epoch
            || e.advanced_completed != self.state.completed
        {
            return Err(invalid("stale or mismatched restored identity"));
        }
        self.fresh_artifact(&e.artifact_sha256)?;
        let (kind, next) = self.interval_record(audit.clone(), Some(e))?;
        self.stage(kind, next)
    }
    /// Supply the inherited Q1 decision for the committed post-audit state.
    /// This method does not compute DEV scores or replace the parent's gates.
    pub fn parent_q1(
        &mut self,
        stamp: StateStamp,
        passed: bool,
        artifact_sha256: String,
    ) -> Result<DurableRecord> {
        if self.phase != Phase::AwaitQ1 || stamp != self.stamp()? {
            return Err(invalid("Q1 out of phase or wrong committed state"));
        }
        self.fresh_artifact(&artifact_sha256)?;
        let mut next = self.state.clone();
        next.q1_passed = passed;
        if !passed {
            next.reason(StopReason::ParentQ1);
        }
        self.stage(
            RecordKind::ParentQ1 {
                stamp,
                passed,
                artifact_sha256,
            },
            next,
        )
    }
    /// The caller acknowledges an exact record hash only AFTER append-only
    /// durable storage. Acknowledgement is an interface contract, not an fsync
    /// implementation or a crash-recovery proof.
    pub fn acknowledge(&mut self, persisted_sha256: &str) -> Result<()> {
        if self.phase != Phase::AwaitDurable {
            return Err(invalid("no pending durable record"));
        }
        let (record, next) = self
            .pending
            .as_ref()
            .ok_or_else(|| invalid("missing pending record"))?;
        if persisted_sha256 != record.sha256()? {
            return Err(invalid("durable acknowledgement hash differs"));
        }
        let q1_record = matches!(record.kind, RecordKind::ParentQ1 { .. });
        match &record.kind {
            RecordKind::Baseline(e) => {
                self.used_artifacts.insert(e.artifact_sha256.clone());
            }
            RecordKind::Interval(i) => {
                self.used_artifacts
                    .insert(i.inspected.artifact_sha256.clone());
                if let Some(r) = &i.restoration {
                    self.used_artifacts.insert(r.artifact_sha256.clone());
                }
            }
            RecordKind::ParentQ1 {
                artifact_sha256, ..
            } => {
                self.used_artifacts.insert(artifact_sha256.clone());
            }
        }
        self.state = next.clone();
        self.pending = None;
        self.restore = None;
        self.steps.clear();
        self.interval_start = None;
        self.boundary = None;
        self.phase = if self.state.completed == Q1_BATCHES && !q1_record {
            Phase::AwaitQ1
        } else if !self.state.reasons.is_empty() {
            Phase::Stopped
        } else if self.state.completed == MAX_BATCHES {
            Phase::Complete
        } else {
            Phase::Ready
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(n: u64) -> String {
        crate::corpus::sha256(&n.to_le_bytes())
    }

    fn pins(n: u64) -> SnapshotPin {
        SnapshotPin {
            checkpoint_sha256: hash(n),
            storage_sha256: hash(n + 1),
            snapshot_sha256: hash(n + 2),
        }
    }

    fn machine(frozen: bool) -> AuditMachine {
        AuditMachine::new(
            RunBinding {
                plan_sha256: hash(1),
                release_sha256: hash(2),
                lineage_sha256: hash(3),
                sheet_sha256: hash(4),
                anchor_sha256: hash(5),
                combine_gates: 1,
                relation_gates: 1,
            },
            frozen,
            1,
        )
        .unwrap()
    }

    fn baseline(m: &mut AuditMachine, admitted: u64) {
        let e = AuditEvidence {
            stamp: m.stamp().unwrap(),
            snapshot: pins(10),
            admitted,
            artifact_sha256: hash(11),
        };
        let r = m.baseline(e).unwrap();
        assert!(m.record_step(step(m, Family::Vector, false, 500)).is_err());
        m.acknowledge(&r.sha256().unwrap()).unwrap();
    }

    fn step(m: &AuditMachine, family: Family, applied: bool, id: u64) -> StepEvidence {
        StepEvidence {
            before: m.stamp().unwrap(),
            family,
            applied,
            sensitive: applied,
            policy_refused: false,
            artifact_sha256: hash(id),
        }
    }

    fn fill(m: &mut AuditMachine, applied: bool) {
        while m.phase() == Phase::Ready || m.phase() == Phase::Open {
            let n = m.completed();
            let family = m.expected_family();
            let mut e = step(
                m,
                family,
                applied && !(m.frozen_zero && family == Family::Relation),
                1000 + n,
            );
            e.policy_refused = m.frozen_zero && family == Family::Relation;
            m.record_step(e).unwrap();
        }
        assert_eq!(m.phase(), Phase::AwaitBoundary);
    }

    fn close(m: &mut AuditMachine, admitted: u64) -> DurableRecord {
        let p = pins(100_000 + m.completed());
        let s = m.seal_boundary(p.clone()).unwrap();
        let r = m
            .audit(AuditEvidence {
                stamp: s,
                snapshot: p,
                admitted,
                artifact_sha256: hash(200_000 + m.completed()),
            })
            .unwrap();
        match r {
            AuditAction::Commit(r) => r,
            AuditAction::Restore(required) => m
                .restored(RestoreEvidence {
                    restored_snapshot: required.passing.snapshot.clone(),
                    restored_epoch: required.passing.epoch,
                    advanced_completed: required.failed.completed,
                    required,
                    new_runtime_model: m.runtime_model() + 1,
                    artifact_sha256: hash(300_000 + m.completed()),
                })
                .unwrap(),
        }
    }

    #[test]
    fn baseline_is_required_and_unreachable_is_terminal() {
        let mut m = machine(false);
        assert!(m.record_step(step(&m, Family::Vector, false, 10)).is_err());
        baseline(&mut m, 499);
        assert_eq!(m.phase(), Phase::Stopped);
        assert_eq!(m.stop_reasons(), &[StopReason::InstrumentUnreachable]);
    }

    #[test]
    fn commit_requires_exact_durable_ack_and_replay_cannot_double_commit() {
        let mut m = machine(false);
        baseline(&mut m, 600);
        fill(&mut m, true);
        assert_eq!(m.committed_updates(), 0);
        assert_eq!(m.stamp().unwrap().epoch, 64);
        let r = close(&mut m, 576);
        assert!(m.record_step(step(&m, Family::Vector, false, 99)).is_err());
        let before = serde_json::to_vec(&m).unwrap();
        assert!(m.acknowledge(&hash(900)).is_err());
        assert_eq!(before, serde_json::to_vec(&m).unwrap());
        let sha = r.sha256().unwrap();
        m.acknowledge(&sha).unwrap();
        assert_eq!(m.committed_updates(), 64);
        assert_eq!(m.phase(), Phase::Ready);
        assert!(m.acknowledge(&sha).is_err());
    }

    #[test]
    fn rejection_restores_epoch_advances_attempts_and_preserves_failed_evidence() {
        let mut m = machine(false);
        baseline(&mut m, 600);
        let passing = m.last_passing().unwrap().clone();
        fill(&mut m, true);
        let r = close(&mut m, 575);
        let RecordKind::Interval(ref interval) = r.kind else {
            panic!()
        };
        assert!(interval.rejected);
        assert_eq!(interval.steps.len(), 64);
        assert!(interval.steps.iter().all(|s| s.rejected_applied));
        assert_ne!(interval.failed_snapshot, passing.snapshot);
        m.acknowledge(&r.sha256().unwrap()).unwrap();
        assert_eq!(m.completed(), 64);
        assert_eq!(m.committed_updates(), 0);
        assert_eq!(m.stamp().unwrap().epoch, 0);
        assert_eq!(m.stamp().unwrap().generation, 1);
        assert_eq!(m.last_passing().unwrap(), &passing);
        assert_eq!(
            m.counters().iter().map(|c| c.rejected_applied).sum::<u64>(),
            64
        );
    }

    #[test]
    fn malformed_identity_wrong_schedule_and_replay_refuse_without_mutation() {
        let mut m = machine(false);
        baseline(&mut m, 600);
        for field in ["run_sha256", "counters_sha256"] {
            let mut e = step(&m, Family::Vector, false, 66);
            let mut json = serde_json::to_value(&e.before).unwrap();
            json[field] = serde_json::json!(hash(999));
            e.before = serde_json::from_value(json).unwrap();
            let before = serde_json::to_vec(&m).unwrap();
            assert!(m.record_step(e).is_err());
            assert_eq!(before, serde_json::to_vec(&m).unwrap());
        }
        assert!(m.record_step(step(&m, Family::Combine, false, 77)).is_err());
        let e = step(&m, Family::Vector, false, 88);
        m.record_step(e.clone()).unwrap();
        let mut replay = e;
        replay.before = m.stamp().unwrap();
        assert!(m.record_step(replay).is_err());
        let mut unknown = serde_json::to_value(step(&m, Family::Vector, false, 99)).unwrap();
        unknown["extra"] = serde_json::json!(true);
        assert!(serde_json::from_value::<StepEvidence>(unknown).is_err());
        let mut b = m.binding.clone();
        b.anchor_sha256 = "NOT_SHA".into();
        assert!(AuditMachine::new(b.clone(), false, 1).is_err());
        b.anchor_sha256 = hash(5);
        b.combine_gates = u32::MAX;
        assert!(AuditMachine::new(b, false, 1).is_err());
    }

    #[test]
    fn audit_barrier_binds_snapshot_epoch_cursor_and_anchor() {
        let mut m = machine(false);
        baseline(&mut m, 600);
        assert!(m.seal_boundary(pins(88)).is_err());
        fill(&mut m, false);
        let stamp = m.seal_boundary(pins(88)).unwrap();
        let valid = AuditEvidence {
            stamp,
            snapshot: pins(88),
            admitted: 600,
            artifact_sha256: hash(99),
        };
        for which in 0..5 {
            let mut bad = valid.clone();
            match which {
                0 => bad.stamp.epoch += 1,
                1 => bad.stamp.completed -= 1,
                2 => bad.stamp.generation += 1,
                3 => bad.snapshot = pins(99),
                _ => bad.admitted = ANCHOR_ROWS + 1,
            }
            let before = serde_json::to_vec(&m).unwrap();
            assert!(m.audit(bad).is_err());
            assert_eq!(before, serde_json::to_vec(&m).unwrap());
        }
        assert!(m.record_step(step(&m, Family::Vector, false, 111)).is_err());
        assert!(matches!(m.audit(valid).unwrap(), AuditAction::Commit(_)));
    }

    #[test]
    fn stale_restore_and_forged_passing_checkpoint_refuse() {
        let mut m = machine(false);
        baseline(&mut m, 600);
        fill(&mut m, true);
        let stamp = m.seal_boundary(pins(88)).unwrap();
        let AuditAction::Restore(required) = m
            .audit(AuditEvidence {
                stamp,
                snapshot: pins(88),
                admitted: 575,
                artifact_sha256: hash(99),
            })
            .unwrap()
        else {
            panic!()
        };
        for which in 0..8 {
            let mut e = RestoreEvidence {
                required: required.clone(),
                restored_snapshot: required.passing.snapshot.clone(),
                restored_epoch: required.passing.epoch,
                advanced_completed: required.failed.completed,
                new_runtime_model: 2,
                artifact_sha256: hash(100),
            };
            match which {
                0 => e.new_runtime_model = 1,
                1 => e.required.next_generation += 1,
                2 => e.required.passing.snapshot = pins(987),
                3 => e.required.failed.completed -= 1,
                4 => e.artifact_sha256 = required.audit_artifact_sha256.clone(),
                5 => e.restored_snapshot = pins(123456),
                6 => e.restored_epoch += 1,
                _ => e.advanced_completed -= 1,
            }
            let before = serde_json::to_vec(&m).unwrap();
            assert!(m.restored(e).is_err());
            assert_eq!(before, serde_json::to_vec(&m).unwrap());
        }
        assert!(m.record_step(step(&m, Family::Vector, false, 111)).is_err());
        let r = m
            .restored(RestoreEvidence {
                restored_snapshot: required.passing.snapshot.clone(),
                restored_epoch: required.passing.epoch,
                advanced_completed: required.failed.completed,
                required,
                new_runtime_model: 2,
                artifact_sha256: hash(100),
            })
            .unwrap();
        m.acknowledge(&r.sha256().unwrap()).unwrap();
        let mut stale = step(&m, Family::Vector, false, 111);
        stale.before.runtime_model = 1;
        stale.before.generation = 0;
        assert!(m.record_step(stale).is_err());
    }

    #[test]
    fn frozen_refusals_survive_rejection_and_nonapplied_attempts_are_not_rejected_applies() {
        let mut m = machine(true);
        baseline(&mut m, 600);
        fill(&mut m, true);
        let r = close(&mut m, 575);
        let RecordKind::Interval(i) = &r.kind else {
            panic!()
        };
        assert_eq!(
            i.steps.iter().filter(|s| s.evidence.policy_refused).count(),
            4
        );
        assert_eq!(i.steps.iter().filter(|s| s.rejected_applied).count(), 60);
        m.acknowledge(&r.sha256().unwrap()).unwrap();
        assert_eq!(m.counters()[2].policy_refused, 4);
        assert_eq!(m.counters()[2].attempted, 4);
        assert_eq!(m.counters()[2].provisional_applied, 0);
        assert_eq!(m.counters()[2].committed_applied, 0);
        assert_eq!(m.counters()[2].rejected_applied, 0);
    }

    #[test]
    fn third_full_rejection_stops_immediately_and_pass_resets_streak() {
        let mut m = machine(false);
        baseline(&mut m, 600);
        for rejected in [true, true, false, true, true, true] {
            fill(&mut m, false);
            let r = close(&mut m, if rejected { 575 } else { 600 });
            m.acknowledge(&r.sha256().unwrap()).unwrap();
            if m.completed() < 384 {
                assert_eq!(m.phase(), Phase::Ready);
            }
        }
        assert_eq!(m.completed(), 384);
        assert_eq!(m.phase(), Phase::Stopped);
        assert!(m
            .stop_reasons()
            .contains(&StopReason::ConsecutiveRejections));
        assert!(m.stop_reasons().contains(&StopReason::RejectionRate));
        assert!(m.record_step(step(&m, Family::Vector, false, 999)).is_err());
    }

    fn to_q1(rejected: impl Fn(u64) -> bool) -> AuditMachine {
        let mut m = machine(false);
        baseline(&mut m, 600);
        for interval in 0..64 {
            fill(&mut m, false);
            let r = close(&mut m, if rejected(interval) { 575 } else { 600 });
            m.acknowledge(&r.sha256().unwrap()).unwrap();
        }
        assert_eq!(m.phase(), Phase::AwaitQ1);
        m
    }

    #[test]
    fn q1_is_postaudit_committed_state_and_strict_half_rate() {
        let mut half = to_q1(|i| i % 2 == 0);
        assert!(half.stop_reasons().is_empty());
        assert!(half
            .record_step(step(&half, Family::Vector, false, 999))
            .is_err());
        let r = half
            .parent_q1(half.stamp().unwrap(), true, hash(900_001))
            .unwrap();
        half.acknowledge(&r.sha256().unwrap()).unwrap();
        assert_eq!(half.phase(), Phase::Ready);
        let mut over = to_q1(|i| i % 2 == 0 || i == 63);
        assert_eq!(over.stop_reasons(), &[StopReason::RejectionRate]);
        let r = over
            .parent_q1(over.stamp().unwrap(), false, hash(900_002))
            .unwrap();
        assert!(r.stop_reasons.contains(&StopReason::RejectionRate));
        assert!(r.stop_reasons.contains(&StopReason::ParentQ1));
        over.acknowledge(&r.sha256().unwrap()).unwrap();
        assert_eq!(over.phase(), Phase::Stopped);
    }

    #[test]
    fn full_dose_requires_258_audits_with_final_twenty_before_completion() {
        let mut m = machine(false);
        baseline(&mut m, 600);
        let mut count = 1;
        while m.completed() < MAX_BATCHES {
            let start = m.completed();
            fill(&mut m, false);
            assert_ne!(m.phase(), Phase::Complete);
            let r = close(&mut m, 600);
            if m.completed() == MAX_BATCHES {
                let RecordKind::Interval(i) = &r.kind else {
                    panic!()
                };
                assert_eq!(i.steps.len(), 20);
                assert_eq!(m.completed() - start, 20);
            }
            m.acknowledge(&r.sha256().unwrap()).unwrap();
            count += 1;
            if m.phase() == Phase::AwaitQ1 {
                let q = m
                    .parent_q1(m.stamp().unwrap(), true, hash(900_003))
                    .unwrap();
                m.acknowledge(&q.sha256().unwrap()).unwrap();
            }
        }
        assert_eq!(count, 258);
        assert_eq!(m.phase(), Phase::Complete);
        assert_eq!(m.committed_updates(), 0);
        assert!(m.record_step(step(&m, Family::Vector, false, 999)).is_err());
    }
}
