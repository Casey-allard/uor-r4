//! Whole selected phrases enter the same learned query-update and completion
//! transition. Parent parameters and geometric encoding are unchanged.
use super::{completion, runtime::PayloadWindow, scheduling, span};
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    relational_attention::runtime::{Error, Result},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Control {
    Full,
    LegacyWordOnly,
    UpdateFirstWord,
    StalePayload,
    UpdateDisabled,
    ReadDisabled,
    ExactIdentity,
}
pub fn generate(
    a: &span::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    prompt: &[u8],
    control: Control,
) -> Result<completion::Generated> {
    if control == Control::StalePayload {
        return Err(Error::State);
    }
    generate_inner(a, g, m, records, prompt, control, None)
}
/// Diagnostic intervention only: payloads come from actual matched baseline
/// trajectories and are indexed by the current clause, not by expected answers.
pub fn generate_with_stale(
    a: &span::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    prompt: &[u8],
    stale_payloads: &[Vec<u8>],
) -> Result<completion::Generated> {
    generate_inner(
        a,
        g,
        m,
        records,
        prompt,
        Control::StalePayload,
        Some(stale_payloads),
    )
}
fn generate_inner(
    a: &span::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    prompt: &[u8],
    control: Control,
    stale_payloads: Option<&[Vec<u8>]>,
) -> Result<completion::Generated> {
    a.validate(g)?;
    let reader = match control {
        Control::ReadDisabled => span::Control::ReadDisabled,
        Control::ExactIdentity => span::Control::ExactIdentity,
        _ => span::Control::Full,
    };
    let execution = if control == Control::UpdateDisabled {
        scheduling::Control::UpdateDisabled
    } else {
        scheduling::Control::Full
    };
    let window = if control == Control::LegacyWordOnly {
        PayloadWindow::Word
    } else {
        PayloadWindow::Phrase
    };
    completion::generate_routed_payload(
        &a.parent,
        g,
        m,
        records,
        prompt,
        completion::Control::Full,
        execution,
        window,
        |bytes, clause| {
            let payload = match control {
                Control::UpdateFirstWord => bytes
                    .split(|b| *b == b' ')
                    .next()
                    .ok_or(Error::Shape)?
                    .to_vec(),
                Control::StalePayload => stale_payloads
                    .and_then(|p| p.get(clause))
                    .ok_or(Error::State)?
                    .clone(),
                _ => bytes.to_vec(),
            };
            Ok(payload)
        },
        |query, _| span::route(a, g, m, records, query, reader),
    )
}
#[cfg(test)]
mod tests {
    use super::super::runtime::{splice, splice_with_window};
    use super::*;
    #[test]
    fn phrase_splice_keeps_exact_bytes_and_old_word_contract() -> Result<()> {
        let q = b"who did they help?";
        assert!(splice(q, 8, 12, b"ruby amber").is_err());
        assert_eq!(
            splice_with_window(q, 8, 12, b"ruby amber", PayloadWindow::Phrase)?,
            b"who did ruby amber help?"
        );
        assert_eq!(
            splice(q, 8, 12, b"ruby")?,
            splice_with_window(q, 8, 12, b"ruby", PayloadWindow::Phrase)?
        );
        for bad in [
            b"".as_slice(),
            b"ruby  amber",
            b" ruby",
            b"ruby ",
            b"ruby\tamber",
            b"Ruby amber",
            b"a b c d",
            b"abcdefghijklmnopq",
        ] {
            assert!(splice_with_window(q, 8, 12, bad, PayloadWindow::Phrase).is_err());
        }
        let max = b"abcdefghijklmnop abcdefghijklmnop abcdefghijklmnop";
        assert_eq!(max.len(), 50);
        assert!(splice_with_window(q, 8, 12, max, PayloadWindow::Phrase).is_ok());
        assert!(splice_with_window(q, 9, 12, b"ruby amber", PayloadWindow::Phrase).is_err());
        let mut long = b"they".to_vec();
        long.resize(128, b'?');
        assert!(splice_with_window(&long, 0, 4, max, PayloadWindow::Phrase).is_err());
        Ok(())
    }
    #[test]
    fn later_phrase_component_and_order_rebind_canonical_query() -> Result<()> {
        use crate::native_geometric::ordered_state::runtime::{Query, CANONICAL};
        let g = BoundGeometry::canonical().map_err(|_| Error::Geometry)?;
        let q = b"who did they help?";
        let encode = |payload: &[u8]| -> Result<Query> {
            Query::encode(
                &g,
                &splice_with_window(q, 8, 12, payload, PayloadWindow::Phrase)?,
                CANONICAL,
            )
        };
        let a = encode(b"ruby amber")?;
        let b = encode(b"ruby birch")?;
        let c = encode(b"amber ruby")?;
        assert_ne!(a.occurrences, b.occurrences);
        assert_ne!(a.occurrences, c.occurrences);
        assert_eq!(&a.prefixes[..13], &b.prefixes[..13]);
        assert_ne!(a.prefixes, b.prefixes);
        assert_ne!(a.prefixes, c.prefixes);
        Ok(())
    }
}
