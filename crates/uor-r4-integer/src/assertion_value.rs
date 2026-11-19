//! Resolve the value following a frozen fact word within the exact writer's assertion.
use crate::{
    content_selector::{BOUNDARY, WIDTH},
    continuation_selector::{self, Continuation},
    invalid,
    span_selector::{Bank, SpanChoice},
    value_selector::{self, Decisions, ExactBank},
    Result,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssertionValue {
    /// Exclusive original prompt position of a delimiter the exact writer recognizes.
    pub fence: Option<u32>,
    pub value: Decisions,
    pub continuation: Continuation,
    /// The candidate after checking that the unchanged writer can reach the delimiter.
    pub source: Option<u32>,
}

impl AssertionValue {
    pub fn words(self) -> [u32; 13] {
        let mut words = [BOUNDARY; 13];
        words[0] = self.fence.unwrap_or(BOUNDARY);
        words[1..8].copy_from_slice(&self.value.words());
        words[8..12].copy_from_slice(&self.continuation.words());
        words[12] = self.source.unwrap_or(BOUNDARY);
        words
    }
}

/// Check the incoming whole-word coordinates before testing a value.
pub fn endpoints(ids: &[u32], frozen: SpanChoice) -> Result<Option<[u32; 2]>> {
    let (Some(fact_start), Some(fact_end), Some(query_start), Some(query_end)) = (
        frozen.fact_start,
        frozen.fact_end,
        frozen.query_start,
        frozen.query_end,
    ) else {
        if frozen.fact_start.is_none()
            && frozen.fact_end.is_none()
            && frozen.query_start.is_none()
            && frozen.query_end.is_none()
        {
            return Ok(None);
        }
        return Err(invalid("partial assertion bounds"));
    };
    if ids.len() > WIDTH
        || fact_start > fact_end
        || fact_end >= query_start
        || query_start > query_end
        || query_end as usize >= ids.len()
    {
        return Err(invalid("invalid assertion bounds"));
    }
    Ok((ids[fact_start as usize..=fact_end as usize]
        == ids[query_start as usize..=query_end as usize])
        .then_some([fact_end, query_start]))
}

pub fn decide(
    ids: &[u32],
    roots: &[u32],
    bank: &Bank,
    exact: &ExactBank,
    frozen: SpanChoice,
    stops: [u32; 4],
) -> Result<AssertionValue> {
    let Some([fact_end, query_start]) = endpoints(ids, frozen)? else {
        return Ok(AssertionValue::default());
    };
    let start = fact_end as usize + 1;
    let query = query_start as usize;
    let Some(fence) = ids[start..query]
        .iter()
        .position(|id| stops.contains(id))
        .map(|offset| (start + offset) as u32)
    else {
        return Ok(AssertionValue::default());
    };
    let value =
        value_selector::decide_before(ids, roots, bank, exact, fact_end as usize, fence as usize)?;
    let continuation =
        continuation_selector::decide_before(ids, roots, bank, Some(fact_end), Some(fence), value)?;
    // ExactCopyState checks its eight-token cap before checking the next delimiter.
    let source = continuation
        .source
        .filter(|&source| source < fence && fence - source < 8);
    Ok(AssertionValue {
        fence: Some(fence),
        value,
        continuation,
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reranks_before_stop_and_keeps_exact_writer_release() {
        let mut classes = vec![1; 4096];
        classes[16] = 0;
        let roots = vec![0; 4096];
        let bank = Bank {
            classes,
            relations: vec![[1, 0, BOUNDARY, BOUNDARY, 0], [3, 0, 0, 0, 0]],
        };
        let exact = ExactBank {
            connectors: vec![[1, 30, BOUNDARY, BOUNDARY]],
        };
        let frozen = SpanChoice {
            fact_start: Some(0),
            fact_end: Some(0),
            query_start: Some(5),
            query_end: Some(5),
            source: Some(4),
            ..Default::default()
        };
        let prompt = [29, 30, 31, 16, 44, 29];
        let old = value_selector::decide(&prompt, &roots, &bank, &exact, frozen).unwrap();
        assert_eq!(old.word_start, Some(4));
        let without_old_source = SpanChoice {
            source: None,
            ..frozen
        };
        let choice = decide(
            &prompt,
            &roots,
            &bank,
            &exact,
            without_old_source,
            [16, 1, 0, 201],
        )
        .unwrap();
        assert_eq!(choice.source, Some(2));
        assert_eq!(choice.fence, Some(3));
        assert_eq!(choice.value.eligible_mask, 1);

        let missing = decide(
            &[29, 30, 31, 77, 44, 29],
            &roots,
            &bank,
            &exact,
            frozen,
            [16, 1, 0, 201],
        )
        .unwrap();
        assert_eq!(missing.source, None);
        let short = decide(
            &[29, 16, 31, 77, 44, 29],
            &roots,
            &bank,
            &exact,
            frozen,
            [16, 1, 0, 201],
        )
        .unwrap();
        assert_eq!(short.source, None);
        assert_eq!(short.fence, Some(1));

        let mismatch = decide(
            &[29, 30, 31, 16, 44, 70],
            &roots,
            &bank,
            &exact,
            frozen,
            [16, 1, 0, 201],
        )
        .unwrap();
        assert_eq!(mismatch, AssertionValue::default());
    }
}
