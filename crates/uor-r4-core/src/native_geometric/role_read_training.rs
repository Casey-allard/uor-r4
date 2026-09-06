//! Offline Rust learning of a shared, quantized source/entry decision.
use super::role_read::*;
use super::value_types::{ValueFeature, ValueRow};
use super::*;
use std::collections::{BTreeMap, BTreeSet};

impl RoleReadModel {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        let valid = self.schema == "uor-r4.role-read/1"
            && !self.actions.is_empty()
            && self.actions.len() <= READ_ACTIONS
            && self.actions.iter().all(|a| {
                (a.copy || a.prefix.is_some())
                    && a.prefix
                        .is_none_or(|t| t != BOS && (t as usize) < model.geometry.tokens.len())
            })
            && !self.rows.is_empty()
            && self.rows.len() <= READ_ROWS
            && self.rows.windows(2).all(|p| p[0].feature < p[1].feature)
            && self.rows.iter().all(|r| {
                (r.feature.kind >> 5) < self.actions.len() as u8
                    && (r.feature.kind & 31) < 30
                    && (-1_000_000..=1_000_000).contains(&r.weight)
            })
            && !self.training.is_empty()
            && self.training.len() <= 1024
            && self.training.iter().all(|r| !r.id.trim().is_empty())
            && self
                .training
                .iter()
                .map(|r| &r.id)
                .collect::<BTreeSet<_>>()
                .len()
                == self.training.len()
            && (1..=64).contains(&self.epochs)
            && (0.0001..=1.0).contains(&f64::from_bits(self.learning_rate_bits))
            && self.dictionary.len() <= super::word_copy_types::WORD_COPY_DICTIONARY
            && self.dictionary.iter().all(|w| {
                w.len > 0
                    && w.len <= 32
                    && w.bytes[usize::from(w.len)..].iter().all(|b| *b == 0)
                    && w.bytes[..usize::from(w.len)]
                        .iter()
                        .all(|b| b.is_ascii_alphanumeric() || *b == b'_')
            })
            && self
                .dictionary
                .windows(2)
                .all(|p| p[0].bytes[..usize::from(p[0].len)] < p[1].bytes[..usize::from(p[1].len)]);
        if !valid {
            return Err(Error("invalid role-read artifact".into()));
        }
        let primes = crate::corpus_induced_spin_placement::first_primes(self.dictionary.len())
            .map_err(|e| Error(e.to_string()))?;
        if self
            .dictionary
            .iter()
            .zip(primes)
            .any(|(w, p)| u64::from(w.prime) != p)
        {
            return Err(Error("role-read prime dictionary mismatch".into()));
        }
        if let Some(relations) = &self.relations {
            relations.validate(model)?;
        }
        let mut parent = model.clone();
        let copy = parent
            .response_entry
            .as_mut()
            .and_then(|e| e.copy.as_mut())
            .ok_or_else(|| Error("role read requires the copy parent".into()))?;
        if !copy.composed_entry || !copy.completed_word_suffix {
            return Err(Error("role read requires composed entry/completion".into()));
        }
        copy.role_read = None;
        parent.refresh_identity()?;
        if parent.artifact_cid != self.baseline_artifact {
            return Err(Error("role-read parent differs".into()));
        }
        Ok(())
    }
}

struct Alternative {
    keys: Vec<ValueFeature>,
    correct: bool,
}
struct Frame {
    alternatives: Vec<Alternative>,
}

fn session(model: &Model, prompt: &str) -> Result<Session> {
    let mut s = model.session(Control::Full)?;
    s.observe(model, BOS)?;
    for token in model.encode(prompt)? {
        s.observe(model, token)?;
    }
    s.begin_response(model)?;
    Ok(s)
}

impl Model {
    pub fn role_read_training(&self) -> &[DocumentReceipt] {
        head(self).map_or(&[], |h| h.training.as_slice())
    }
    /// Fit only this source/entry selector. Parent copy/completion, typed
    /// arithmetic, occurrence memory and lexical continuation stay fixed.
    pub fn fit_role_read(
        &self,
        documents: &[ValueExample],
        config: ResponseEntryFitConfig,
    ) -> Result<(Model, serde_json::Value)> {
        self.validate()?;
        if head(self).is_some()
            || documents.is_empty()
            || documents.len() > 1024
            || documents
                .iter()
                .map(|d| d.prompt.len() + d.response.len())
                .sum::<usize>()
                > 4 * 1024 * 1024
            || !(1..=64).contains(&config.epochs)
            || !config.learning_rate.is_finite()
            || !(0.0001..=1.0).contains(&config.learning_rate)
        {
            return Err(Error("invalid role-read source/configuration".into()));
        }
        let (dictionary, omitted, _) = super::word_copy_training::dictionary(documents)?;
        let mut actions = Vec::<ReadAction>::new();
        let mut targets = Vec::new();
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        let mut prompts = BTreeMap::new();
        let mut skipped_numeric = 0;
        for doc in documents {
            if doc.id.trim().is_empty()
                || !ids.insert(&doc.id)
                || prompts
                    .insert(&doc.prompt, &doc.response)
                    .is_some_and(|r| r != &doc.response)
            {
                return Err(Error(
                    "role-read document identities or targets conflict".into(),
                ));
            }
            receipts.push(super::training::receipt(&Document {
                id: doc.id.clone(),
                text: serde_json::to_string(&(&doc.prompt, &doc.response))
                    .map_err(|e| Error(e.to_string()))?,
            }));
            let mut s = session(self, &doc.prompt)?;
            s.predict(self)?;
            let values = s
                .values
                .as_ref()
                .ok_or_else(|| Error("role-read typed words missing".into()))?;
            let entry = s
                .response_entry
                .as_ref()
                .ok_or_else(|| Error("role-read entry missing".into()))?;
            if !super::word_copy_runtime::eligible(self, entry, values, Control::Full) {
                if self.generate(&doc.prompt, 64, Control::Full)?.bytes != doc.response.as_bytes() {
                    return Err(Error(format!("upstream response differs for {}", doc.id)));
                }
                skipped_numeric += 1;
                targets.push(None);
                continue;
            }
            let words = values
                .lexemes
                .as_ref()
                .ok_or_else(|| Error("role-read words missing".into()))?;
            let offset = doc
                .response
                .bytes()
                .position(|b| b.is_ascii_alphabetic() || b == b'_')
                .unwrap_or(0);
            let mut correct = Vec::new();
            for (i, w) in words.queries[..words.query_len].iter().enumerate() {
                let len = usize::from(w.len);
                let tail = &doc.response.as_bytes()[offset..];
                if len > 0
                    && tail.starts_with(&w.bytes[..len])
                    && tail
                        .get(len)
                        .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_')
                    && len + 1 + usize::from(offset > 0)
                        <= usize::from(super::response_entry_types::RESPONSE_ENTRY_STEPS)
                {
                    correct.push(i as u8);
                }
            }
            let copy = !correct.is_empty();
            let tokens = if copy {
                self.encode(&doc.response[..offset])?
            } else {
                self.encode(&doc.response)?
            };
            if (copy && tokens.len() > 1) || (!copy && tokens.is_empty()) {
                return Err(Error("role-read entry exceeds one lexical prefix".into()));
            }
            let action = ReadAction {
                copy,
                prefix: tokens.first().copied(),
            };
            let ai = if let Some(i) = actions.iter().position(|a| *a == action) {
                i
            } else {
                let i = actions.len();
                actions.push(action);
                i
            };
            if actions.len() > READ_ACTIONS {
                return Err(Error("role-read action vocabulary exceeds8".into()));
            }
            if !copy {
                correct.push(NO_SOURCE);
            }
            targets.push(Some((ai, correct)));
        }
        let mut model = self.clone();
        model
            .response_entry
            .as_mut()
            .and_then(|e| e.copy.as_mut())
            .ok_or_else(|| Error("role-read copy parent missing".into()))?
            .role_read = Some(RoleReadModel {
            relations: None,
            schema: "uor-r4.role-read/1".into(),
            baseline_artifact: self.artifact_cid.clone(),
            dictionary,
            actions,
            rows: Vec::new(),
            training: receipts,
            epochs: config.epochs,
            learning_rate_bits: config.learning_rate.to_bits(),
            local_roles_only: true,
        });
        model.refresh_identity()?;
        let mut frames = Vec::new();
        let mut weights = BTreeMap::<ValueFeature, f64>::new();
        for (doc, target) in documents.iter().zip(&targets) {
            let Some((wanted, sources)) = target else {
                continue;
            };
            // Prepare from the frozen parent: prediction is used only for its
            // actual NoWrite eligibility, never to supply a source label.
            let mut s = session(self, &doc.prompt)?;
            s.predict(self)?;
            let values = s
                .values
                .as_ref()
                .ok_or_else(|| Error("role values absent".into()))?;
            let words = values
                .lexemes
                .as_ref()
                .ok_or_else(|| Error("role words absent".into()))?;
            let ctx = context(&model, values, Control::Full, &mut WordCopyWork::default());
            let read = head(&model).ok_or_else(|| Error("role head absent".into()))?;
            let mut alternatives = Vec::new();
            for i in 0..=words.query_len {
                let source = if i == words.query_len {
                    NO_SOURCE
                } else {
                    i as u8
                };
                let (feat, n) = features(
                    &model,
                    values,
                    &ctx,
                    i,
                    Control::Full,
                    &mut WordCopyWork::default(),
                );
                for (a, action) in read.actions.iter().enumerate() {
                    if action.copy != (source != NO_SOURCE) {
                        continue;
                    }
                    let keys: Vec<_> = feat[..n].iter().map(|f| key(*f, a)).collect();
                    for k in &keys {
                        weights.entry(*k).or_default();
                    }
                    alternatives.push(Alternative {
                        keys,
                        correct: a == *wanted && sources.contains(&source),
                    });
                }
            }
            if !alternatives.iter().any(|a| a.correct) {
                return Err(Error("role-read target unreachable".into()));
            }
            frames.push(Frame { alternatives });
        }
        if frames.is_empty() || frames.len() > config.max_positions || weights.len() > READ_ROWS {
            return Err(Error(format!(
                "role-read feature/position capacity exceeded: {} rows, {} frames",
                weights.len(),
                frames.len()
            )));
        }
        let mut best = weights.clone();
        let mut best_correct = 0;
        let mut best_loss = f64::INFINITY;
        let mut selected_epoch = 0;
        for epoch in 1..=config.epochs {
            for frame in &frames {
                let scores: Vec<f64> = frame
                    .alternatives
                    .iter()
                    .map(|a| a.keys.iter().map(|k| weights[k]).sum())
                    .collect();
                let max = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let z: f64 = scores.iter().map(|s| (s - max).exp()).sum();
                let positive_max = frame
                    .alternatives
                    .iter()
                    .zip(&scores)
                    .filter(|(a, _)| a.correct)
                    .map(|(_, s)| *s)
                    .fold(f64::NEG_INFINITY, f64::max);
                let pz: f64 = frame
                    .alternatives
                    .iter()
                    .zip(&scores)
                    .filter(|(a, _)| a.correct)
                    .map(|(_, s)| (s - positive_max).exp())
                    .sum();
                for (a, score) in frame.alternatives.iter().zip(scores) {
                    let desired = if a.correct {
                        (score - positive_max).exp() / pz
                    } else {
                        0.0
                    };
                    let delta = config.learning_rate * (desired - (score - max).exp() / z)
                        / (a.keys.len() as f64).sqrt();
                    if !delta.is_finite() {
                        return Err(Error("nonfinite role-read fit".into()));
                    }
                    for k in &a.keys {
                        if let Some(w) = weights.get_mut(k) {
                            *w = (*w + delta).clamp(-3900.0, 3900.0);
                        }
                    }
                }
            }
            // Select with the actual quantized table and the runtime's strict tie law.
            let mut correct = 0;
            let mut loss = 0.0;
            for frame in &frames {
                let scores: Vec<i64> = frame
                    .alternatives
                    .iter()
                    .map(|a| {
                        a.keys
                            .iter()
                            .map(|k| (weights[k] * 256.0).round() as i64)
                            .sum()
                    })
                    .collect();
                let mut winner = 0;
                for i in 1..scores.len() {
                    if scores[i] > scores[winner] {
                        winner = i;
                    }
                }
                correct += usize::from(frame.alternatives[winner].correct);
                let max = scores[winner] as f64 / 256.0;
                let z: f64 = scores.iter().map(|s| (*s as f64 / 256.0 - max).exp()).sum();
                let p: f64 = frame
                    .alternatives
                    .iter()
                    .zip(&scores)
                    .filter(|(a, _)| a.correct)
                    .map(|(_, s)| (*s as f64 / 256.0 - max).exp())
                    .sum();
                loss += z.ln() - p.ln();
            }
            if correct > best_correct || (correct == best_correct && loss < best_loss) {
                best_correct = correct;
                best_loss = loss;
                best = weights.clone();
                selected_epoch = epoch;
            }
        }
        let read = model
            .response_entry
            .as_mut()
            .and_then(|e| e.copy.as_mut())
            .and_then(|c| c.role_read.as_mut())
            .ok_or_else(|| Error("role read missing".into()))?;
        read.rows = best
            .into_iter()
            .map(|(feature, w)| ValueRow {
                feature,
                weight: (w * 256.0).round() as i32,
            })
            .collect();
        model.refresh_identity()?;
        model.validate()?;
        Ok((
            model.clone(),
            serde_json::json!({"schema":"uor-r4.role-read-fit/1","parent":self.artifact_cid(),"artifact":model.artifact_cid(),"frames":frames.len(),"numeric_preserved":skipped_numeric,"rows":weights.len(),"dictionary_omitted":omitted,"fit_correct":best_correct,"fit_loss":best_loss,"selected_epoch":selected_epoch,"epochs":config.epochs,"learning_rate":config.learning_rate,"scope":"Joint occurrence-or-NoRead and entry action. Relative local role features use exact equality and ordered prime context; weights learn their selection contribution. No separate semantic role classifier is claimed. Suffix and upstream parent remain fixed; construction accuracy is not transfer."}),
        ))
    }
}
