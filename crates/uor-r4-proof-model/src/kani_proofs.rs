#![allow(unexpected_cfgs)]

#[cfg(all(kani, feature = "full-model"))]
compile_error!("Kani proofs must run with --no-default-features");

#[cfg(kani)]
mod kani_proofs {
    use kani::any;
    use uor_r4_graph_format::ScoreQ;
    use uor_r4_graph_runtime::runtime_state::{RuntimeState, SemanticStateSlot};

    #[kani::proof]
    fn proof_score_q_saturating_add_safety() {
        let a = ScoreQ::from_raw(any::<i32>());
        let b = ScoreQ::from_raw(any::<i32>());

        let sum = a.saturating_add(b);
        assert_eq!(sum.raw(), a.raw().saturating_add(b.raw()));
    }

    #[kani::proof]
    fn proof_runtime_state_slot_update_safety() {
        let mut state = RuntimeState::<32, 8, 8, 8>::default();

        let token = any::<u32>();
        state.record_token(token);

        let slot = SemanticStateSlot {
            region_id: any::<u32>(),
            token: any::<u32>(),
            score_q: ScoreQ::from_raw(any::<i32>()),
            age: any::<u16>(),
        };

        // This should not panic or cause OOB
        state.local_mut().update_slot(slot);
        state.segment_mut().update_slot(slot);
        state.session_mut().update_slot(slot);
    }
}
