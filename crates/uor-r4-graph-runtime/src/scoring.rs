use uor_r4_graph_format::ScoreQ;

/// Typed fixed-point residual classes consumed by runtime scoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ResidualKind {
    Transition = 0,
    Emission = 1,
    Goal = 2,
    Constraint = 3,
    Uncertainty = 4,
}

/// Sentinel scores used by selection/ranking paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ScoreSentinel {
    NoScore = 0,
    SaturatedLow = 1,
    SaturatedHigh = 2,
}

/// Canonical ordered score domain used by deterministic ranking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderedScore {
    Sentinel(ScoreSentinel),
    Real(ScoreQ),
}

impl Ord for OrderedScore {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        use OrderedScore::{Real, Sentinel};
        use ScoreSentinel::{NoScore, SaturatedHigh, SaturatedLow};
        match (*self, *other) {
            (Sentinel(NoScore), Sentinel(NoScore)) => core::cmp::Ordering::Equal,
            (Sentinel(NoScore), _) => core::cmp::Ordering::Less,
            (_, Sentinel(NoScore)) => core::cmp::Ordering::Greater,
            (Sentinel(SaturatedLow), Sentinel(SaturatedLow)) => core::cmp::Ordering::Equal,
            (Sentinel(SaturatedLow), Sentinel(SaturatedHigh)) => core::cmp::Ordering::Less,
            (Sentinel(SaturatedLow), Real(_)) => core::cmp::Ordering::Less,
            (Sentinel(SaturatedHigh), Sentinel(SaturatedHigh)) => core::cmp::Ordering::Equal,
            (Sentinel(SaturatedHigh), _) => core::cmp::Ordering::Greater,
            (Real(_), Sentinel(SaturatedLow)) => core::cmp::Ordering::Greater,
            (Real(_), Sentinel(SaturatedHigh)) => core::cmp::Ordering::Less,
            (Real(a), Real(b)) => a.cmp(&b),
        }
    }
}

impl PartialOrd for OrderedScore {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// One typed residual contribution with canonical evidence id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypedContribution {
    pub evidence_id: u32,
    pub kind: ResidualKind,
    pub value: ScoreQ,
}

/// Sort contributions into canonical accumulation order.
pub fn sort_contributions_canonical(contributions: &mut [TypedContribution]) {
    contributions.sort_by_key(|c| (c.kind as u8, c.evidence_id));
}

/// Reference fixed-point accumulator.
///
/// The order is the provided slice order; callers should canonicalize first via
/// [`sort_contributions_canonical`]. Total: `Some(total)` for a well-formed
/// contribution set, `None` when any evidence id is duplicated (each canonical
/// evidence may contribute at most once).
pub fn accumulate_reference(base: ScoreQ, contributions: &[TypedContribution]) -> Option<ScoreQ> {
    let mut total = base;
    for (index, contribution) in contributions.iter().enumerate() {
        if contributions[..index]
            .iter()
            .any(|prior| prior.evidence_id == contribution.evidence_id)
        {
            return None;
        }
        total = total.saturating_add(contribution.value);
    }
    Some(total)
}

/// Canonical deterministic top-1 selector:
/// higher score wins; ties break to the lowest token id.
pub fn select_best(candidates: &[(u32, OrderedScore)]) -> Option<(u32, OrderedScore)> {
    let (&(mut best_token, mut best_score), rest) = candidates.split_first()?;
    for &(token, score) in rest {
        if score > best_score || (score == best_score && token < best_token) {
            best_token = token;
            best_score = score;
        }
    }
    Some((best_token, best_score))
}

#[cfg(test)]
mod tests {
    use super::{
        OrderedScore, ResidualKind, ScoreSentinel, TypedContribution, accumulate_reference,
        select_best, sort_contributions_canonical,
    };
    use uor_r4_graph_format::ScoreQ;

    #[test]
    fn canonical_accumulator_saturates_at_bounds() {
        let contributions = [TypedContribution {
            evidence_id: 1,
            kind: ResidualKind::Emission,
            value: ScoreQ::from_raw(1),
        }];
        let max_total = accumulate_reference(ScoreQ::MAX, &contributions).expect("accumulate");
        assert_eq!(max_total, ScoreQ::MAX);
        // (Option::expect on the total accumulator: None == duplicate id.)

        let negative = [TypedContribution {
            evidence_id: 1,
            kind: ResidualKind::Constraint,
            value: ScoreQ::from_raw(-1),
        }];
        let min_total = accumulate_reference(ScoreQ::MIN, &negative).expect("accumulate");
        assert_eq!(min_total, ScoreQ::MIN);
    }

    #[test]
    fn ordered_score_places_sentinels_consistently() {
        assert!(
            OrderedScore::Sentinel(ScoreSentinel::NoScore)
                < OrderedScore::Sentinel(ScoreSentinel::SaturatedLow)
        );
        assert!(
            OrderedScore::Sentinel(ScoreSentinel::SaturatedLow)
                < OrderedScore::Real(ScoreQ::from_raw(0))
        );
        assert!(
            OrderedScore::Real(ScoreQ::from_raw(0))
                < OrderedScore::Sentinel(ScoreSentinel::SaturatedHigh)
        );
    }

    #[test]
    fn deterministic_tie_break_prefers_lowest_token() {
        let best = select_best(&[
            (42, OrderedScore::Real(ScoreQ::from_raw(7))),
            (7, OrderedScore::Real(ScoreQ::from_raw(7))),
            (99, OrderedScore::Real(ScoreQ::from_raw(6))),
        ])
        .expect("best");
        assert_eq!(best.0, 7);
    }

    #[test]
    fn duplicate_evidence_is_rejected() {
        let contributions = [
            TypedContribution {
                evidence_id: 3,
                kind: ResidualKind::Emission,
                value: ScoreQ::from_raw(4),
            },
            TypedContribution {
                evidence_id: 3,
                kind: ResidualKind::Transition,
                value: ScoreQ::from_raw(5),
            },
        ];
        // Duplicate evidence id 3 → the total accumulator rejects with None.
        assert_eq!(accumulate_reference(ScoreQ::ZERO, &contributions), None);
    }

    #[test]
    fn non_adjacent_duplicate_evidence_is_rejected_after_canonical_sort() {
        let mut contributions = [
            TypedContribution {
                evidence_id: 3,
                kind: ResidualKind::Uncertainty,
                value: ScoreQ::from_raw(4),
            },
            TypedContribution {
                evidence_id: 2,
                kind: ResidualKind::Emission,
                value: ScoreQ::from_raw(2),
            },
            TypedContribution {
                evidence_id: 3,
                kind: ResidualKind::Transition,
                value: ScoreQ::from_raw(5),
            },
        ];
        sort_contributions_canonical(&mut contributions);
        // The canonical sort surfaces the non-adjacent duplicate id 3 → None.
        assert_eq!(accumulate_reference(ScoreQ::ZERO, &contributions), None);
    }

    #[test]
    fn canonical_sort_orders_kind_then_id() {
        let mut contributions = [
            TypedContribution {
                evidence_id: 9,
                kind: ResidualKind::Uncertainty,
                value: ScoreQ::from_raw(1),
            },
            TypedContribution {
                evidence_id: 2,
                kind: ResidualKind::Transition,
                value: ScoreQ::from_raw(1),
            },
            TypedContribution {
                evidence_id: 1,
                kind: ResidualKind::Transition,
                value: ScoreQ::from_raw(1),
            },
        ];
        sort_contributions_canonical(&mut contributions);
        assert_eq!(contributions[0].evidence_id, 1);
        assert_eq!(contributions[1].evidence_id, 2);
        assert_eq!(contributions[2].kind, ResidualKind::Uncertainty);
    }
}
