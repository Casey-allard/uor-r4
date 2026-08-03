//! Server-side quantization for the persistent session manifold.
//!
//! The graph runtime consumes only the resulting 288-bit value. Floating
//! point state construction stays on the serving side, outside the deployed
//! integer-only prediction contract.

pub const SESSION_SIGNATURE_BITS: usize = 288;
pub const SESSION_SIGNATURE_BYTES: usize = SESSION_SIGNATURE_BITS / 8;

/// Quantize a centered session-manifold state into the graph's 288-bit
/// signature space.
///
/// Each bit uses a deterministic signed three-coordinate projection. The
/// fixed index schedule is part of the signature definition: changing it is
/// a representation change and must be versioned with the graph artifacts.
pub fn from_state(state: &[f64]) -> [u8; SESSION_SIGNATURE_BYTES] {
    let mut signature = [0u8; SESSION_SIGNATURE_BYTES];
    if state.is_empty() {
        return signature;
    }

    let mean = state.iter().sum::<f64>() / state.len() as f64;
    for bit in 0..SESSION_SIGNATURE_BITS {
        let first = state[(bit * 37 + 11) % state.len()] - mean;
        let second = state[(bit * 97 + 7) % state.len()] - mean;
        let third = state[(bit * 193 + 3) % state.len()] - mean;
        let projection = first + 0.5 * second - 0.25 * third;
        if projection >= 0.0 {
            signature[bit >> 3] |= 1 << (bit & 7);
        }
    }
    signature
}

/// Deterministic fallback for clients that have token history but no
/// persistent f64 manifold. This preserves the same 288-bit lane and makes
/// the direct chat client session-sensitive; the HTTP server uses [`from_state`]
/// from its persistent manifold instead.
pub fn from_tokens(tokens: &[u32]) -> [u8; SESSION_SIGNATURE_BYTES] {
    let mut lanes = [0u64; 5];
    for (position, &token) in tokens.iter().enumerate() {
        let mut value =
            u64::from(token).wrapping_add((position as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
        for (lane, slot) in lanes.iter_mut().enumerate() {
            value = splitmix64(value.wrapping_add(lane as u64));
            *slot = slot.rotate_left((lane * 11 + position % 17) as u32) ^ value;
        }
    }

    let mut signature = [0u8; SESSION_SIGNATURE_BYTES];
    for (index, byte) in signature.iter_mut().enumerate() {
        let lane = index % lanes.len();
        let shift = (index / lanes.len()) * 8;
        *byte = (lanes[lane] >> shift) as u8;
    }
    signature
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_projection_is_deterministic_and_sensitive() {
        let first: Vec<f64> = (0..512).map(|index| (index as f64 * 0.13).sin()).collect();
        let second: Vec<f64> = (0..512)
            .map(|index| (index as f64 * 0.17 + 1.3).cos())
            .collect();
        assert_eq!(from_state(&first), from_state(&first));
        assert_ne!(from_state(&first), from_state(&second));
    }

    #[test]
    fn token_history_projection_is_order_sensitive() {
        assert_ne!(from_tokens(&[1, 2, 3]), from_tokens(&[3, 2, 1]));
        assert_eq!(from_tokens(&[]), [0; SESSION_SIGNATURE_BYTES]);
    }
}

/// Pinned multi-turn fixture (issue #247): six deterministic conversations,
/// four turns each. The text is original to this fixture. Determinism is
/// guarded by `session_signature::tests::fixture_evolution_is_deterministic`
/// (two independently constructed routers produce identical signatures), so
/// the fixture is pinned by construction rather than by golden bytes that
/// would drift with deliberate vocabulary-representation changes.
pub const MULTI_TURN_FIXTURE: [[&str; 4]; 6] = [
    [
        "Where does the water table sit under the northern ridge this season?",
        "The survey said the aquifer runs deeper past the second borehole.",
        "Then route the drilling plan around the ridge and log the depth readings.",
        "Agreed. Schedule the crew for the dry months and archive the coordinates.",
    ],
    [
        "Explain how the router picks an expert window for a new prompt.",
        "So the deficit angle decides whether the wave stays trapped or scatters?",
        "Show me a prompt whose orbit stays symmetric across all scale windows.",
        "Now perturb one word and tell me which resonance changes first.",
    ],
    [
        "I want to compare two compression runs from last week.",
        "The second run kept more probability mass in the emission lists.",
        "Plot the contrast distribution for both and mark the coarse regions.",
        "Keep the finer cover and re-run the certification overnight.",
    ],
    [
        "Draft a short note about the meeting tomorrow morning.",
        "Add a reminder about the budget review and the travel forms.",
        "Actually move the meeting to the afternoon and copy the field team.",
        "Send it and file the old draft under the project archive.",
    ],
    [
        "What does the coherence measure tell us about a noisy prompt?",
        "If kappa drops, does the state vector drift off the hypersphere?",
        "Give an example where two sessions diverge from the same prompt.",
        "Summarize the divergence in one sentence for the log.",
    ],
    [
        "List the steps to index a new corpus into the manifold.",
        "How long until the new vocabulary settles into stable coordinates?",
        "Index the field manual next and keep the identities separate.",
        "Verify the retrieval quality before switching the default corpus.",
    ],
];

/// Deterministic evolution gamma for the fixture. The server derives gamma
/// dynamically from kappa (src/server.rs); the fixture pins the midpoint so
/// the measurement is reproducible.
const FIXTURE_GAMMA: f64 = 0.5;

/// Session signatures from the pinned fixture through the SHIPPED path:
/// `UorR4Router::new` -> `index_corpus` -> `evolve_state` per turn ->
/// `session_signature_from_state`. One signature per turn per transcript.
pub fn fixture_session_signatures() -> Vec<[u8; SESSION_SIGNATURE_BYTES]> {
    let mut router = crate::UorR4Router::new(0.85);
    let mut signatures = Vec::with_capacity(MULTI_TURN_FIXTURE.len() * 4);
    for (index, transcript) in MULTI_TURN_FIXTURE.iter().enumerate() {
        let identity = format!("fixture-{index}");
        let joined = transcript.join(" ");
        router.index_corpus(&joined, &identity);
        for turn in transcript {
            router.evolve_state(&identity, turn, FIXTURE_GAMMA);
            signatures.push(from_state(&router.get_brain_state_native(&identity)));
        }
    }
    signatures
}

#[cfg(test)]
mod fixture_tests {
    use super::*;

    /// The #247 pinned fixture is deterministic by construction: two
    /// independently constructed routers, evolved over the same pinned
    /// transcripts, produce byte-identical session signatures. This is the
    /// pin — golden bytes would silently invalidate on any deliberate
    /// representation change, while double-construction equality catches
    /// exactly the nondeterminism that would corrupt the #247 measurement.
    #[test]
    fn fixture_evolution_is_deterministic() {
        let first = fixture_session_signatures();
        let second = fixture_session_signatures();
        assert_eq!(first, second, "fixture evolution must be deterministic");
        assert_eq!(first.len(), MULTI_TURN_FIXTURE.len() * 4);
        // The signatures must carry information: not all-zero, and turns
        // must move the state (adjacent turns of one transcript differ).
        assert!(first.iter().any(|sig| sig.iter().any(|&b| b != 0)));
        assert!(
            first
                .chunks(4)
                .all(|turns| turns.windows(2).any(|w| w[0] != w[1])),
            "every transcript must produce at least one state change across turns"
        );
    }
}
