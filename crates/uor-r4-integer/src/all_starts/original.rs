use super::{l1, root_at, Bank, Margin, OnsetDecision, NIL, NO_RADIUS};
use crate::{
    assertion_value::{assertion_fence, AssertionValue},
    discrete_d11::ROLE_COPY_STOPS,
    invalid,
    paired_role::{validate_role_ids, ROLE_CONTEXT, ROLE_VOCAB},
    span_selector::{CONT, START},
    Result,
};

const HEADER: usize = 6;
const RECORD: usize = 10;
const TRACE: usize = 10;
pub const ORIGINAL_CANDIDATE_WORDS: usize = HEADER + RECORD * ROLE_CONTEXT;
pub const ORIGINAL_DECISION_WORDS: usize = 2 + TRACE * ROLE_CONTEXT;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OriginalStartGeometry {
    /// Number of adjacent pre-source pieces represented, capped at three.
    pub depth: usize,
    pub signature: u32,
    pub roots: [u16; 10],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OriginalStart {
    pub source: u32,
    pub word_end: u32,
    pub fence: u32,
    /// Exact original distance after the factual head, excluding the source.
    pub connector_pieces: u32,
    pub copy_pieces: u32,
    /// None when the incumbent eight-token cap fires before the terminator.
    pub descriptor: Option<OriginalStartGeometry>,
}

pub struct OriginalCandidates<'a> {
    ids: &'a [u32],
    roots: &'a [u32],
    classes: &'a [u32],
    fact_end: usize,
    next: usize,
    fence: Option<usize>,
}

impl OriginalCandidates<'_> {
    pub fn fence(&self) -> Option<u32> {
        self.fence.map(|position| position as u32)
    }
}

impl Iterator for OriginalCandidates<'_> {
    type Item = OriginalStart;

    fn next(&mut self) -> Option<Self::Item> {
        let fence = self.fence?;
        let class = |position: usize| self.classes[self.ids[position] as usize];
        let source = (self.next..fence).find(|&position| class(position) == START)?;
        let word_end = (source + 1..fence)
            .find(|&position| class(position) != CONT)
            .unwrap_or(fence);
        self.next = word_end;
        let connector_pieces = source - self.fact_end - 1;
        let copy_pieces = fence - source;
        let descriptor = (copy_pieces < 8).then(|| {
            let depth = connector_pieces.min(3);
            let mut signature = 0;
            let mut roots = [0; 10];
            for (slot, position) in (source - depth..word_end).enumerate() {
                signature |= class(position) << (2 * slot);
                roots[slot] = self.roots[self.ids[position] as usize] as u16;
            }
            OriginalStartGeometry {
                depth,
                signature,
                roots,
            }
        });
        Some(OriginalStart {
            source: source as u32,
            word_end: word_end as u32,
            fence: fence as u32,
            connector_pieces: connector_pieces as u32,
            copy_pieces: copy_pieces as u32,
            descriptor,
        })
    }
}

/// Enumerate complete word starts after at least one connector piece and before
/// the first actual copy-stop ID. Head end is inclusive. The descriptor views
/// original adjacent pieces while retaining the untruncated address and depth.
pub fn original_candidates<'a>(
    ids: &'a [u32],
    roots: &'a [u32],
    classes: &'a [u32],
    fact_end: usize,
    query_start: usize,
    stops: [u32; 4],
) -> Result<OriginalCandidates<'a>> {
    validate_role_ids(ids)?;
    if fact_end == 0
        || fact_end >= query_start
        || query_start >= ids.len()
        || roots.len() != ROLE_VOCAB
        || classes.len() != ROLE_VOCAB
        || ids
            .iter()
            .any(|&id| roots[id as usize] > u16::MAX as u32 || classes[id as usize] > CONT)
    {
        return Err(invalid("invalid original onset tape, tables or endpoints"));
    }
    Ok(OriginalCandidates {
        ids,
        roots,
        classes,
        fact_end,
        next: fact_end + 2,
        fence: assertion_fence(ids, fact_end + 1, query_start, stops)
            .map(|position| position as usize),
    })
}

impl Bank {
    pub(crate) fn choose_original(
        &self,
        ids: &[u32],
        roots: &[u32],
        classes: &[u32],
        fact_end: usize,
        query_start: usize,
        frozen: AssertionValue,
    ) -> Result<OnsetDecision> {
        let row = original_packet(ids, roots, classes, fact_end, query_start, frozen)?;
        let output = self.score_original_row(&row)?;
        Ok(OnsetDecision {
            status: output[0],
            source: (output[1] != NIL).then_some(output[1]),
        })
    }

    /// Canonical ordered score, shared by ordinary selection and the device
    /// differential. An explicit calibration may clear an unresolved rival.
    pub fn score_original_row(
        &self,
        row: &[u32; ORIGINAL_CANDIDATE_WORDS],
    ) -> Result<[u32; ORIGINAL_DECISION_WORDS]> {
        let [length, fact, query, fence, mask, count] =
            [row[0], row[1], row[2], row[3], row[4], row[5]];
        if self.mode.fitted_mode() != super::OnsetMode::Ordered
            || length as usize > ROLE_CONTEXT
            || !(0 < fact && fact < query && query < length)
            || (fence != NIL && !(fact < fence && fence < query))
            || (fence == NIL && (count != 0 || mask != 0))
            || mask > 7
            || count as usize > ROLE_CONTEXT
        {
            return Err(invalid("invalid original onset packet bounds"));
        }
        let mut output = [NIL; ORIGINAL_DECISION_WORDS];
        let mut best = Margin::ABSENT;
        let mut tied = false;
        let mut unseen = false;
        let mut previous_word_end = fact + 2;
        let mut seen_mask = 0;
        for (index, c) in row[HEADER..].chunks_exact(RECORD).enumerate() {
            if index >= count as usize {
                if c.iter().any(|&v| v != 0) {
                    return Err(invalid("nonzero original onset packet padding"));
                }
                continue;
            }
            let [source, end, connector, copy, signature] = [c[0], c[1], c[2], c[3], c[4]];
            if !(previous_word_end <= source && source < end && end <= fence)
                || connector != source - fact - 1
                || copy != fence - source
                || (connector <= 3 && mask & (1 << (connector - 1)) == 0)
            {
                return Err(invalid("invalid original onset candidate addresses"));
            }
            if connector <= 3 {
                seen_mask |= 1 << (connector - 1);
            }
            previous_word_end = end;
            let trace = &mut output[2 + index * TRACE..2 + (index + 1) * TRACE];
            trace[..5].copy_from_slice(&[source, end, connector, copy, 1]);
            if copy >= 8 {
                if c[4..].iter().any(|&v| v != 0) {
                    return Err(invalid("inaccessible original onset carries geometry"));
                }
                continue;
            }
            let depth = connector.min(3) as usize;
            let slots = depth + (end - source) as usize;
            if signature >> (2 * slots) != 0 || (signature >> (2 * depth)) & 3 != START {
                return Err(invalid("invalid original onset class signature"));
            }
            let geometry = &c[5..];
            for index in 0..10 {
                let class = (signature >> (2 * index)) & 3;
                if (index < slots && (class > CONT || index > depth && class != CONT))
                    || (index >= slots && root_at(geometry, index) != 0)
                {
                    return Err(invalid("invalid original onset geometry or padding"));
                }
            }
            let shape = depth as u32 | ((end - source) << 8);
            let mut closest = [NIL; 2];
            let mut radius = NO_RADIUS;
            let mut negative_radius = NO_RADIUS;
            for (index, record) in self
                .records()
                .enumerate()
                .filter(|(_, record)| record[0] & 65535 == shape && record[1] == signature)
            {
                let distance = (0..slots)
                    .map(|slot| l1(root_at(geometry, slot), root_at(&record[3..], slot)))
                    .sum();
                let label = (record[0] >> 16) as usize;
                closest[label] = closest[label].min(distance);
                radius = record[2];
                if let Some(calibration) = &self.negative_calibration {
                    negative_radius = calibration.radius(index);
                }
            }
            trace[4..8].copy_from_slice(&[2, closest[0], closest[1], radius]);
            if closest.contains(&NIL) || radius == NO_RADIUS || closest[0] > radius {
                if self.negative_calibration.is_some() {
                    trace[8] = negative_radius;
                    if negative_radius != NO_RADIUS
                        && closest[1] <= negative_radius
                        && closest[1] < closest[0]
                    {
                        trace[4] = 4;
                        continue;
                    }
                }
                unseen = true;
                continue;
            }
            let margin = Margin {
                num: closest[1] as i32 - closest[0] as i32,
                den: slots as i32,
                source,
            };
            trace[4] = 3;
            trace[8..].copy_from_slice(&[margin.num as u32, margin.den as u32]);
            match margin.compare(best) {
                std::cmp::Ordering::Greater => {
                    best = margin;
                    tied = false;
                }
                std::cmp::Ordering::Equal => tied = true,
                std::cmp::Ordering::Less => {}
            }
        }
        if seen_mask != mask {
            return Err(invalid(
                "original onset packet omits an incumbent candidate",
            ));
        }
        output[0] = if unseen {
            2
        } else if best.den == 0 {
            1
        } else if best.num <= 0 {
            4
        } else if tied {
            3
        } else {
            0
        };
        if output[0] == 0 {
            output[1] = best.source;
        }
        Ok(output)
    }
}

/// Serialize the bounded original-address input consumed by Rust and CUDA.
pub fn original_packet(
    ids: &[u32],
    roots: &[u32],
    classes: &[u32],
    fact_end: usize,
    query_start: usize,
    frozen: AssertionValue,
) -> Result<[u32; ORIGINAL_CANDIDATE_WORDS]> {
    let candidates =
        original_candidates(ids, roots, classes, fact_end, query_start, ROLE_COPY_STOPS)?;
    if candidates.fence() != frozen.fence
        || (frozen.value.eligible_mask | frozen.value.word_start_mask) > 7
    {
        return Err(invalid("original onset fence disagrees with incumbent"));
    }
    let mask = frozen.value.eligible_mask & frozen.value.word_start_mask;
    let mut row = [0; ORIGINAL_CANDIDATE_WORDS];
    row[..5].copy_from_slice(&[
        ids.len() as u32,
        fact_end as u32,
        query_start as u32,
        frozen.fence.unwrap_or(NIL),
        mask,
    ]);
    for candidate in candidates.filter(|candidate| {
        candidate.connector_pieces > 3 || mask & (1 << (candidate.connector_pieces - 1)) != 0
    }) {
        let offset = HEADER + RECORD * row[5] as usize;
        row[5] += 1;
        let record = &mut row[offset..offset + RECORD];
        record[..4].copy_from_slice(&[
            candidate.source,
            candidate.word_end,
            candidate.connector_pieces,
            candidate.copy_pieces,
        ]);
        if let Some(geometry) = candidate.descriptor {
            record[4] = geometry.signature;
            for (slot, root) in geometry.roots.into_iter().enumerate() {
                record[5 + slot / 2] |= u32::from(root) << (16 * (slot % 2));
            }
        }
    }
    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{all_starts::OnsetMode, discrete_d11::ExactCopyState};

    fn sites() -> (Vec<u32>, Vec<u32>) {
        let roots = (0..ROLE_VOCAB as u32).collect();
        let mut classes = vec![START; ROLE_VOCAB];
        for id in [0, 16, 201] {
            classes[id] = 0;
        }
        for id in [41, 43, 45, 47, 49, 51] {
            classes[id] = CONT;
        }
        (roots, classes)
    }

    #[test]
    fn original_long_connector_and_writer_stop_keep_exact_piece_addresses() {
        let (roots, classes) = sites();
        let ids = [
            0, 12, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 201, 53, 12,
        ];
        let candidates =
            original_candidates(&ids, &roots, &classes, 1, 15, ROLE_COPY_STOPS).unwrap();
        assert_eq!(candidates.fence(), Some(13));
        let starts: Vec<_> = candidates.collect();
        assert_eq!(
            starts
                .iter()
                .map(|c| (c.source, c.word_end, c.connector_pieces, c.copy_pieces))
                .collect::<Vec<_>>(),
            [
                (4, 6, 2, 9),
                (6, 8, 4, 7),
                (8, 10, 6, 5),
                (10, 12, 8, 3),
                (12, 13, 10, 1)
            ],
        );
        assert_eq!(starts[0].descriptor, None);
        let geometry = starts[1].descriptor.unwrap();
        assert_eq!(geometry.depth, 3);
        assert_eq!(geometry.signature, 2 | 1 << 2 | 2 << 4 | 1 << 6 | 2 << 8);
        assert_eq!(geometry.roots, [41, 42, 43, 44, 45, 0, 0, 0, 0, 0]);

        let mut state = ExactCopyState::new(Some(starts[1].source), ids.len() as u32);
        for &id in &ids[6..13] {
            let step = state.write(&ids, 999, ROLE_COPY_STOPS).unwrap();
            assert_eq!((step.copied, step.emitted), (1, id));
            state = step.state;
        }
        let stop = state
            .write(&ids, 999, ROLE_COPY_STOPS)
            .unwrap()
            .finish_with_eos(true, 1);
        assert_eq!((stop.state.cursor, stop.release, stop.emitted), (13, 1, 1));
    }

    #[test]
    fn exact_eight_piece_candidate_is_inaccessible_and_no_fence_abstains() {
        let (roots, classes) = sites();
        let ids = [0, 12, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 16, 12];
        let mut starts =
            original_candidates(&ids, &roots, &classes, 1, 13, ROLE_COPY_STOPS).unwrap();
        let early = starts.next().unwrap();
        assert_eq!(
            (early.source, early.copy_pieces, early.descriptor),
            (4, 8, None)
        );
        assert_eq!(starts.next().unwrap().source, 6);
        assert!(
            original_candidates(&ids, &roots, &classes, 1, 12, ROLE_COPY_STOPS)
                .unwrap()
                .next()
                .is_none()
        );
        assert!(original_candidates(&ids, &roots, &classes, 13, 13, ROLE_COPY_STOPS).is_err());
    }

    fn bank_with_candidate(row: &[u32; ORIGINAL_CANDIDATE_WORDS], index: usize) -> Bank {
        let record = &row[HEADER + index * RECORD..HEADER + (index + 1) * RECORD];
        let shape = record[2].min(3) | ((record[1] - record[0]) << 8);
        let mut words = vec![0; 4 + 16];
        words[2] = 2;
        for (label, exemplar) in words[4..].chunks_exact_mut(8).enumerate() {
            exemplar[..3].copy_from_slice(&[shape | (label as u32) << 16, record[4], 0]);
            exemplar[3..].copy_from_slice(&record[5..]);
        }
        words[15] ^= 1;
        Bank {
            words: words.into_boxed_slice(),
            mode: OnsetMode::OrderedWholeWord,
            negative_calibration: None,
        }
    }

    #[test]
    fn book_keeps_early_mask_and_unseen_far_rival_veto() {
        let (roots, classes) = sites();
        let ids = [0, 12, 40, 41, 42, 43, 44, 45, 46, 47, 16, 12];
        let frozen = AssertionValue {
            fence: Some(10),
            ..Default::default()
        };
        let row = original_packet(&ids, &roots, &classes, 1, 11, frozen).unwrap();
        assert_eq!(row[5], 2);
        assert_eq!(&row[HEADER..HEADER + 4], &[6, 8, 4, 4]);
        assert_eq!(&row[HEADER + RECORD..HEADER + RECORD + 4], &[8, 10, 6, 2]);
        let bank = bank_with_candidate(&row, 0);
        let trace = bank.score_original_row(&row).unwrap();
        assert_eq!(&trace[..2], &[2, NIL]);
        assert_eq!(&trace[2..8], &[6, 8, 4, 4, 3, 0]);
        assert_eq!(&trace[12..17], &[8, 10, 6, 2, 2]);
        assert!((1..NIL).contains(&trace[17]));

        let mut single = row;
        single[5] = 1;
        single[HEADER + RECORD..].fill(0);
        let result = bank.score_original_row(&single).unwrap();
        assert_eq!(&result[..2], &[0, 6]);
        single[HEADER + 2] = 3;
        assert!(bank.score_original_row(&single).is_err());

        let mut frozen = frozen;
        frozen.value.eligible_mask = 2;
        frozen.value.word_start_mask = 2;
        let included = original_packet(&ids, &roots, &classes, 1, 11, frozen).unwrap();
        assert_eq!(included[5], 3);
        assert_eq!(&included[HEADER..HEADER + 4], &[4, 6, 2, 6]);
    }
}
