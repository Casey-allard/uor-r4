//! Explicit training-only interventions on emitted Copy feedback. Ownership at serving is unchanged.
use super::*;
impl TlTrainer {
    pub fn train_intervened_batch(
        &mut self,
        batch: &[TlExample],
        feedback: &[Option<Vec<u32>>],
    ) -> Result<TlBatchReport, String> {
        if feedback.len() != batch.len() {
            return Err("intervention batch length mismatch".into());
        }
        for (ex, value) in batch.iter().zip(feedback) {
            if !ex.weight.is_finite() || ex.weight <= 0.0 {
                return Err("invalid supervision weight".into());
            }
            if let Some(tokens) = value {
                if !ex.grounded
                    || tokens.len() != ex.sel.len()
                    || tokens.is_empty()
                    || tokens.iter().any(|&t| t as usize >= self.cfg.vocab)
                    || ex
                        .actions
                        .iter()
                        .filter(|a| matches!(a, TlAction::Copy))
                        .count()
                        > tokens.len()
                {
                    return Err("invalid training-only copied-feedback intervention".into());
                }
            }
        }
        Ok(self.train_batch_impl(batch, feedback))
    }
}
#[cfg(test)]
mod intervention_tests {
    use super::*;
    fn ex() -> TlExample {
        TlExample {
            sel: vec![1],
            res: vec![],
            facts: SlFacts {
                history: 2,
                ..SlFacts::default()
            },
            observed: vec![],
            actions: vec![TlAction::Copy, TlAction::Generate(3), TlAction::Stop],
            weight: 1.0,
            doc: 0,
            grounded: true,
            terminal_stop: true,
        }
    }
    fn make() -> TlTrainer {
        let mut cfg = TlConfig::new(8);
        cfg.h_dim = 8;
        let mut t = TlTrainer::new(cfg, TlTrainConfig::default()).unwrap();
        t.wo.fill(1.0);
        t
    }
    #[test]
    fn original_copy_feedback_is_exactly_the_default_training_path() {
        let mut a = make();
        let mut b = make();
        a.train_batch(&[ex()]);
        b.train_intervened_batch(&[ex()], &[Some(vec![1])]).unwrap();
        assert_eq!(a.e, b.e);
        assert_eq!(a.wo, b.wo);
        assert_eq!(a.me, b.me);
        assert_eq!(a.model().unwrap(), b.model().unwrap());
    }
    #[test]
    fn changing_only_feedback_changes_learning_with_selected_evidence_fixed() {
        let mut a = make();
        let mut b = make();
        a.train_intervened_batch(&[ex()], &[Some(vec![1])]).unwrap();
        b.train_intervened_batch(&[ex()], &[Some(vec![2])]).unwrap();
        assert_ne!(a.mo, b.mo);
    }
    #[test]
    fn invalid_intervention_does_not_update_trainer() {
        let mut a = make();
        let before = a.model().unwrap();
        assert!(a.train_intervened_batch(&[ex()], &[Some(vec![])]).is_err());
        assert_eq!(a.step, 0);
        assert_eq!(a.model().unwrap(), before);
    }
}
