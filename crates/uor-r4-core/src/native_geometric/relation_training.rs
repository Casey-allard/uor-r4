//! Offline construction supervision for relation writes and exact reads.
//! Source byte labels are fit-only; the serving path receives raw tokens.
use super::relation::*;
use super::value_types::{ValueFeature, ValueRow, ValueState, ValueWork};
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationLabel {
    pub owner_end_byte: u64,
    pub value_end_byte: u64,
    pub action: u8,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationExample {
    pub id: String,
    pub prompt: String,
    pub response: String,
    pub writes: Vec<RelationLabel>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Alternative {
    keys: Vec<ValueFeature>,
    correct: bool,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Frame {
    alternatives: Vec<Alternative>,
}

fn fit(frames: &[Frame], epochs: usize) -> Result<(Vec<ValueRow>, usize)> {
    let mut weights = BTreeMap::<ValueFeature, i32>::new();
    for f in frames {
        for a in &f.alternatives {
            for k in &a.keys {
                weights.entry(*k).or_default();
            }
        }
    }
    if weights.len() > RELATION_ROWS {
        return Err(Error("relation rows exceed bound".into()));
    }
    let mut best = weights.clone();
    let mut best_correct = 0;
    for _ in 0..epochs {
        for f in frames {
            let scores: Vec<i64> = f
                .alternatives
                .iter()
                .map(|a| a.keys.iter().map(|k| i64::from(weights[k])).sum())
                .collect();
            let mut winner = 0;
            let mut wanted = None;
            for (i, a) in f.alternatives.iter().enumerate() {
                if scores[i] > scores[winner] {
                    winner = i;
                }
                if a.correct && wanted.is_none_or(|j: usize| scores[i] > scores[j]) {
                    wanted = Some(i);
                }
            }
            let wanted = wanted.ok_or_else(|| Error("relation target unreachable".into()))?;
            if !f.alternatives[winner].correct || scores[wanted] < scores[winner] + 8 {
                for k in &f.alternatives[winner].keys {
                    if let Some(w) = weights.get_mut(k) {
                        *w = (*w - 1).max(-1000000);
                    }
                }
                for k in &f.alternatives[wanted].keys {
                    if let Some(w) = weights.get_mut(k) {
                        *w = (*w + 1).min(1000000);
                    }
                }
            }
        }
        let correct = frames
            .iter()
            .filter(|f| {
                let mut winner = 0;
                let mut best_score = i64::MIN;
                for (i, a) in f.alternatives.iter().enumerate() {
                    let s: i64 = a.keys.iter().map(|k| i64::from(weights[k])).sum();
                    if s > best_score {
                        best_score = s;
                        winner = i;
                    }
                }
                f.alternatives[winner].correct
            })
            .count();
        if correct > best_correct {
            best_correct = correct;
            best = weights.clone();
        }
        if correct == frames.len() {
            break;
        }
    }
    Ok((
        best.into_iter()
            .map(|(feature, weight)| ValueRow { feature, weight })
            .collect(),
        best_correct,
    ))
}

fn writer_frame(
    model: &Model,
    words: &super::value_lexemes::LexemeState,
    label: Option<&RelationLabel>,
    context: Option<&[TokenGeometry]>,
) -> Result<Frame> {
    let words = &words.recent[..words.recent_len.min(8)];
    let mut work = ValueWork::default();
    let addr = addresses(model, words, &mut work);
    let mut alternatives = vec![Alternative {
        keys: Vec::new(),
        correct: label.is_none(),
    }];
    for o in 0..words.len() {
        for v in 0..words.len() {
            if o == v || (o != 0 && v != 0) {
                continue;
            }
            let (f, n) = write_features_with_context(model, words, &addr, o, v, context, &mut work);
            for a in 1..=3 {
                alternatives.push(Alternative {
                    keys: f[..n].iter().map(|f| key(*f, a)).collect(),
                    correct: label.is_some_and(|l| {
                        l.action == a
                            && l.owner_end_byte == words[o].byte_end
                            && l.value_end_byte == words[v].byte_end
                    }),
                });
            }
        }
    }
    if !alternatives.iter().any(|a| a.correct) {
        return Err(Error(
            "relation labeled source outside local write candidates".into(),
        ));
    }
    Ok(Frame { alternatives })
}

impl RelationModel {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        let valid = |rows: &[ValueRow]| {
            !rows.is_empty()
                && rows.len() <= RELATION_ROWS
                && rows.windows(2).all(|p| p[0].feature < p[1].feature)
                && rows.iter().all(|r| {
                    (-1000000..=1000000).contains(&r.weight)
                        && (r.feature.kind & 31) < 20
                        && (1..=3).contains(&(r.feature.kind >> 5))
                })
        };
        if !matches!(
            self.schema.as_str(),
            "uor-r4.exact-relation/1" | "uor-r4.exact-relation/2"
        ) || !valid(&self.writer)
            || !valid(&self.reader)
            || !(1..=64).contains(&self.epochs)
            || self.training.is_empty()
            || self.training.len() > 256
            || self.training.iter().any(|r| r.id.is_empty())
            || self
                .training
                .iter()
                .map(|r| &r.id)
                .collect::<BTreeSet<_>>()
                .len()
                != self.training.len()
        {
            return Err(Error("invalid learned relation model".into()));
        }
        if (self.schema == "uor-r4.exact-relation/1" && !self.role_context.is_empty())
            || (self.schema == "uor-r4.exact-relation/2"
                && self.role_context != compile_role_context(model)?)
        {
            return Err(Error(
                "relation role transport differs from parent tokenizer/geometry".into(),
            ));
        }
        let mut parent = model.clone();
        let role = parent
            .response_entry
            .as_mut()
            .and_then(|e| e.copy.as_mut())
            .and_then(|c| c.role_read.as_mut())
            .ok_or_else(|| Error("relation role parent missing".into()))?;
        if role.actions.len() != 3 {
            return Err(Error(
                "relation reader requires three parent entry actions".into(),
            ));
        }
        role.relations = None;
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent {
            return Err(Error("relation parent identity differs".into()));
        }
        Ok(())
    }
}

fn compile_role_context(model: &Model) -> Result<Vec<TokenGeometry>> {
    let role = super::role_read::head(model)
        .ok_or_else(|| Error("relation role parent missing".into()))?;
    let mut rows = Vec::with_capacity(role.dictionary.len());
    for word in &role.dictionary {
        let text = std::str::from_utf8(&word.bytes[..usize::from(word.len)])
            .map_err(|e| Error(e.to_string()))?;
        let mut row = TokenGeometry {
            prime: word.prime,
            leaf: model.geometry.identity,
            phases: [0; PHASE_CHANNELS],
        };
        for token in model.encode(text)? {
            let g = &model.geometry.tokens[token as usize];
            row.leaf = model.geometry.products
                [model.geometry.row_bases[usize::from(row.leaf)] + usize::from(g.leaf)];
            for (p, d) in row.phases.iter_mut().zip(g.phases) {
                *p = p.wrapping_add(d);
            }
        }
        rows.push(row);
    }
    rows.sort_by_key(|r| r.prime);
    Ok(rows)
}

impl Model {
    pub fn relation_training(&self) -> &[DocumentReceipt] {
        head(self).map_or(&[], |h| h.training.as_slice())
    }
    pub fn fit_relations(
        &self,
        documents: &[RelationExample],
        epochs: usize,
    ) -> Result<(Model, serde_json::Value)> {
        self.fit_relations_mode(documents, epochs, false)
    }

    /// Learn participant-independent role context while preserving exact values.
    pub fn fit_relations_with_role_paths(
        &self,
        documents: &[RelationExample],
        epochs: usize,
    ) -> Result<(Model, serde_json::Value)> {
        self.fit_relations_mode(documents, epochs, true)
    }

    fn fit_relations_mode(
        &self,
        documents: &[RelationExample],
        epochs: usize,
        role_paths: bool,
    ) -> Result<(Model, serde_json::Value)> {
        self.validate()?;
        if head(self).is_some()
            || super::role_read::head(self).is_none()
            || documents.is_empty()
            || documents.len() > 256
            || !(1..=64).contains(&epochs)
            || documents
                .iter()
                .map(|d| d.prompt.len() + d.response.len())
                .sum::<usize>()
                > 1024 * 1024
        {
            return Err(Error("invalid relation fitting source/config".into()));
        }
        let role_context = if role_paths {
            compile_role_context(self)?
        } else {
            Vec::new()
        };
        let context = role_paths.then_some(role_context.as_slice());
        let mut frames = BTreeSet::new();
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        let mut examples = Vec::new();
        for d in documents {
            if d.id.is_empty()
                || !ids.insert(&d.id)
                || d.writes.len() > 16
                || d.writes.iter().any(|l| !(1..=3).contains(&l.action))
            {
                return Err(Error("invalid relation supervision".into()));
            }
            receipts.push(super::training::receipt(&Document {
                id: d.id.clone(),
                text: serde_json::to_string(d).map_err(|e| Error(e.to_string()))?,
            }));
            let mut words = super::value_lexemes::LexemeState::default();
            let mut last = None;
            let mut consumed = BTreeSet::new();
            let mut expected = RelationState::default();
            let mut pose = self.geometry.identity;
            let mut phases = [0_u16; PHASE_CHANNELS];
            let mut seen = 0;
            // Match the actual tokenizer and per-token geometry while exposing
            // every completed word inside a lexical piece, as serving does.
            for (sequence, token) in std::iter::once(BOS)
                .chain(self.encode(&d.prompt)?)
                .enumerate()
            {
                let g = &self.geometry.tokens[token as usize];
                pose = self.geometry.products
                    [self.geometry.row_bases[usize::from(pose)] + usize::from(g.leaf)];
                for (p, d) in phases.iter_mut().zip(g.phases) {
                    *p = p.wrapping_add(d);
                }
                seen = sequence as u64 + 1;
                if token == BOS || token == EOS {
                    continue;
                }
                let single;
                let bytes = if token < LEXICAL_BASE {
                    single = [(token - 2) as u8];
                    &single[..]
                } else {
                    &self.lexical_pieces[(token - LEXICAL_BASE) as usize][..]
                };
                for &b in bytes {
                    words.feed(
                        b,
                        super::value_types::ValueEntry {
                            sequence: sequence as u64,
                            token,
                            cue: g.prime,
                            pose,
                            phases,
                        },
                        &mut ValueWork::default(),
                    );
                    if words.recent_len == 0 || last == Some(words.recent[0].byte_end) {
                        continue;
                    }
                    last = Some(words.recent[0].byte_end);
                    let labels: Vec<_> = d
                        .writes
                        .iter()
                        .enumerate()
                        .filter(|(_, l)| {
                            l.owner_end_byte.max(l.value_end_byte) == words.recent[0].byte_end
                        })
                        .collect();
                    if labels.len() > 1 {
                        return Err(Error("multiple labels at one relation boundary".into()));
                    }
                    let label = labels.first().map(|(_, l)| *l);
                    frames.insert(writer_frame(self, &words, label, context)?);
                    if let Some((i, l)) = labels.first() {
                        consumed.insert(*i);
                        let o = words
                            .recent
                            .iter()
                            .find(|w| w.len != 0 && w.byte_end == l.owner_end_byte)
                            .ok_or_else(|| Error("owner label absent".into()))?;
                        let v = words
                            .recent
                            .iter()
                            .find(|w| w.len != 0 && w.byte_end == l.value_end_byte)
                            .ok_or_else(|| Error("value label absent".into()))?;
                        expected.commit(*o, *v, l.action, &mut ValueWork::default());
                    }
                }
            }
            if consumed.len() != d.writes.len() {
                return Err(Error("relation labels were not observed".into()));
            }
            words.finish(&mut ValueWork::default());
            words.begin();
            let mut values = ValueState::new(self);
            values.lexemes = Some(words);
            values.seen = seen;
            values.pose = pose;
            values.phases = phases;
            values.relations = Some(expected);
            examples.push((values, d));
        }
        let frames: Vec<_> = frames.into_iter().collect();
        if frames.len() > 8192 {
            return Err(Error("relation frame bound exceeded".into()));
        }
        let (writer, write_correct) = fit(&frames, epochs)?;
        let mut reads = Vec::new();
        for (values, d) in &examples {
            let words = values
                .lexemes
                .as_ref()
                .ok_or_else(|| Error("read words absent".into()))?;
            let state = values
                .relations
                .as_ref()
                .ok_or_else(|| Error("read records absent".into()))?;
            let mut work = ValueWork::default();
            let addr = addresses(self, &words.queries[..words.query_len], &mut work);
            let wanted = d
                .response
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches('.');
            let offset = d
                .response
                .bytes()
                .position(|b| b.is_ascii_alphabetic())
                .unwrap_or(0);
            let prefix = self.encode(&d.response[..offset])?.first().copied();
            let mut alternatives = vec![Alternative {
                keys: Vec::new(),
                correct: false,
            }];
            let role =
                super::role_read::head(self).ok_or_else(|| Error("role parent absent".into()))?;
            for id in state.directory {
                let Some(record) = state.record(id) else {
                    continue;
                };
                let (f, n) = read_features(self, record, words, &addr, &mut work);
                let owner_mentioned = words.queries[..words.query_len.min(8)]
                    .iter()
                    .any(|w| record.owner.matches(w, &mut work));
                for (ai, a) in role.actions.iter().enumerate() {
                    let correct = owner_mentioned
                        && if record.conflict {
                            !a.copy && wanted == "Unknown"
                        } else {
                            a.copy
                                && a.prefix == prefix
                                && record.value.bytes[..usize::from(record.value.len)]
                                    == *wanted.as_bytes()
                        };
                    alternatives.push(Alternative {
                        keys: f[..n].iter().map(|f| key(*f, (ai + 1) as u8)).collect(),
                        correct,
                    });
                }
            }
            if !alternatives.iter().any(|a| a.correct) {
                alternatives[0].correct = true;
            }
            reads.push(Frame { alternatives });
        }
        let (reader, read_correct) = fit(&reads, epochs)?;
        let mut model = self.clone();
        model
            .response_entry
            .as_mut()
            .and_then(|e| e.copy.as_mut())
            .and_then(|c| c.role_read.as_mut())
            .ok_or_else(|| Error("role parent absent".into()))?
            .relations = Some(RelationModel {
            schema: if role_paths {
                "uor-r4.exact-relation/2"
            } else {
                "uor-r4.exact-relation/1"
            }
            .into(),
            role_context,
            parent: self.artifact_cid.clone(),
            writer,
            reader,
            training: receipts,
            epochs,
        });
        model.refresh_identity()?;
        model.validate()?;
        let h = head(&model).ok_or_else(|| Error("relation fit absent".into()))?;
        let report = serde_json::json!({"schema":"uor-r4.relation-fit/1","parent":self.artifact_cid(),"artifact":model.artifact_cid(),"documents":documents.len(),"writer_frames":frames.len(),"writer_correct":write_correct,"reader_frames":reads.len(),"reader_correct":read_correct,"writer_rows":h.writer.len(),"reader_rows":h.reader.len(),"epochs":epochs,"scope":"Construction-supervised margin updates export sparse integer tables. Writer labels are exact source byte endpoints and typed actions offline only. Read fitting uses labeled construction writes; generated evaluation must test the assembled learned writer/reader. One association family; no general semantic memory claim."});
        let mut report = report;
        report["operator_schema"] = serde_json::json!(h.schema);
        report["role_context_rows"] = serde_json::json!(h.role_context.len());
        Ok((model, report))
    }
}
