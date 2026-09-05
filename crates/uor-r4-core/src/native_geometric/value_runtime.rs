//! Bounded typed operand selection, exact addition and causal byte emission.
//! Source bytes are lexed without expression/assignment/answer parsing. At an
//! explicit response boundary, at most sixteen typed records are captured.
use super::numeral::{Literal, Numeral};
use super::value_types::*;
use super::*;
use crate::prime_route_attention::ZPhi;

impl ValueState {
    fn recent_at(&self, distance: usize) -> Option<ValueEntry> {
        if distance == 0 || distance > self.recent_len {
            return None;
        }
        let index = if self.recent_cursor >= distance {
            self.recent_cursor - distance
        } else {
            32 - (distance - self.recent_cursor)
        };
        Some(self.recent[index])
    }
    fn append_record(&mut self, record: ValueRecord, work: &mut ValueWork) {
        if self.records.len() == VALUES {
            self.records.remove(0);
            work.record_evictions = work.record_evictions.saturating_add(1);
        }
        self.records.push(record);
    }
    fn literal(&mut self, literal: Literal, work: &mut ValueWork) {
        let mut record = ValueRecord {
            id: self.next_id,
            value: literal.value,
            start: literal.start,
            end: literal.end,
            lexical: self.lexemes.as_ref().map(|words| words.literal_cues),
            ..ValueRecord::default()
        };
        // At most 32 recent token metadata records; oversized split fragments
        // cannot borrow unrelated geometry or cues after metadata eviction.
        let Some(endpoint) = self
            .recent
            .iter()
            .find(|entry| entry.sequence == literal.end && entry.sequence < self.seen)
        else {
            return;
        };
        record.pose = endpoint.pose;
        record.phases = endpoint.phases;
        for offset in 0..4 {
            if let Some(sequence) = literal.start.checked_sub(offset as u64 + 1) {
                if let Some(entry) = self
                    .recent
                    .iter()
                    .find(|entry| entry.sequence == sequence && entry.sequence < self.seen)
                {
                    record.cue[offset] = entry.cue;
                }
            }
        }
        let Some(next) = self.next_id.checked_add(1) else {
            return;
        };
        self.next_id = next;
        self.append_record(record, work);
        work.literal_writes = work.literal_writes.saturating_add(1);
    }
    pub(super) fn observe(&mut self, model: &Model, token: u32, work: &mut ValueWork) {
        let sequence = self.seen;
        if self.active {
            self.commit(token, work);
        }
        let geometry = &model.geometry.tokens[token as usize];
        self.pose = model.geometry.products
            [model.geometry.row_bases[usize::from(self.pose)] + usize::from(geometry.leaf)];
        work.h4_reads = work.h4_reads.saturating_add(1);
        for (phase, delta) in self.phases.iter_mut().zip(geometry.phases) {
            *phase = phase.wrapping_add(delta);
        }
        work.phase_updates = work.phase_updates.saturating_add(PHASE_CHANNELS as u64);
        let cue_token = model
            .memory_read
            .as_ref()
            .and_then(|m| m.cue_aliases.as_ref())
            .map_or(token, |a| a.representatives[token as usize]);
        self.recent[self.recent_cursor] = ValueEntry {
            sequence,
            token,
            cue: model.geometry.tokens[cue_token as usize].prime,
            pose: self.pose,
            phases: self.phases,
        };
        self.recent_cursor = (self.recent_cursor + 1) & 31;
        self.recent_len = (self.recent_len + 1).min(32);
        self.seen = self.seen.saturating_add(1);
        // Generated response bytes do not become a second literal copy of a
        // derived write. They still update ordinary token/geometric memory.
        if self.active {
            return;
        }
        if token == BOS || token == EOS {
            if let Some(words) = &mut self.lexemes {
                words.finish(work);
            }
            if let Some(literal) = self.scanner.finish() {
                self.literal(literal, work);
            }
            if let Some(words) = &mut self.lexemes {
                words.clear_literal();
            }
            return;
        }
        let byte;
        let bytes = if token < LEXICAL_BASE {
            byte = [(token - 2) as u8];
            &byte[..]
        } else {
            &model.lexical_pieces[(token - LEXICAL_BASE) as usize][..]
        };
        for &value in bytes {
            work.input_bytes = work.input_bytes.saturating_add(1);
            if let Some(words) = &mut self.lexemes {
                words.feed(value, self.recent[(self.recent_cursor + 31) & 31], work);
            }
            let was_open = self.scanner.snapshot_needs_suffix();
            let literal = self.scanner.feed(value, sequence);
            if let Some(literal) = literal {
                self.literal(literal, work);
            }
            if let Some(words) = &mut self.lexemes {
                if self.scanner.snapshot_needs_suffix() {
                    // A delimiter can finish one numeral and start a signed
                    // fragment in the same byte. Preserve the former receipt
                    // before capturing the new source context.
                    if !was_open || literal.is_some() {
                        words.capture_literal();
                    }
                } else {
                    words.clear_literal();
                }
            }
        }
    }
    pub(super) fn begin(&mut self, work: &mut ValueWork) {
        if let Some(words) = &mut self.lexemes {
            words.finish(work);
        }
        if let Some(literal) = self.scanner.finish() {
            self.literal(literal, work);
        }
        if let Some(words) = &mut self.lexemes {
            words.begin();
        }
        self.pending = None;
        self.emission = None;
        self.consumed = false;
        self.active = true;
        self.started_at = self.seen;
        self.sources.clear();
        self.sources.extend_from_slice(&self.records);
        self.query_len = self.recent_len.min(QUERY);
        for index in 0..self.query_len {
            if let Some(entry) = self.recent_at(index + 1) {
                self.queries[index] = entry;
            }
        }
    }
    pub(super) fn end(&mut self) {
        self.active = false;
        self.pending = None;
        self.emission = None;
        self.consumed = false;
        self.sources.clear();
        self.query_len = 0;
        self.scanner = super::numeral::Scanner::default();
        if let Some(words) = &mut self.lexemes {
            words.end();
        }
    }
    pub(super) fn proposal(&self, index: usize) -> Option<(ValueAction, ValueRecord, ValueRecord)> {
        // Fixed address space: low four bits address the first operand;
        // upper bits select Copy (0), or one of sixteen second operands.
        let left = index & 15;
        let right = index >> 4;
        let a = *self.sources.get(left)?;
        if right == 0 {
            return Some((ValueAction::Copy, a, a));
        }
        let b = *self.sources.get(right - 1)?;
        (a.id != b.id).then_some((ValueAction::Add, a, b))
    }
    pub(super) fn features(
        &self,
        model: &Model,
        action: ValueAction,
        a: ValueRecord,
        b: ValueRecord,
        control: Control,
        work: &mut ValueWork,
    ) -> ([ValueFeature; VALUE_FEATURES], usize) {
        let mut features = [ValueFeature::default(); VALUE_FEATURES];
        let mut len = 0;
        let op = match action {
            ValueAction::Copy => 0,
            ValueAction::Add => 1,
        };
        let rank_a = self
            .sources
            .iter()
            .rev()
            .position(|r| r.id == a.id)
            .unwrap_or(VALUES) as u64;
        let rank_b = self
            .sources
            .iter()
            .rev()
            .position(|r| r.id == b.id)
            .unwrap_or(VALUES) as u64;
        let mut add = |kind, a, b| {
            if len < VALUE_FEATURES {
                features[len] = ValueFeature { kind, a, b };
                len += 1;
            }
        };
        add(0, op, 0);
        add(1, op, (rank_a << 8) | rank_b);
        add(2, op, u64::from(a.derived) | (u64::from(b.derived) << 1));
        for query in &self.queries[..self.query_len] {
            add(3, op, u64::from(query.cue));
            add(4, (op << 8) | rank_a, u64::from(query.cue));
            add(5, (op << 8) | rank_b, u64::from(query.cue));
            let mut matches = 0;
            for offset in 0..4 {
                work.cue_comparisons = work.cue_comparisons.saturating_add(2);
                if query.cue == a.cue[offset] {
                    matches |= 1 << offset;
                }
                if query.cue == b.cue[offset] {
                    matches |= 1 << (offset + 4);
                }
            }
            add(6, op, matches);
        }
        let endpoint = self.queries[0];
        if !matches!(control, Control::GeometryDisabled | Control::H4Disabled) {
            let inverse = model.geometry.inverses[usize::from(a.pose)];
            let relative = model.geometry.products
                [model.geometry.row_bases[usize::from(inverse)] + usize::from(endpoint.pose)];
            let inverse_b = model.geometry.inverses[usize::from(b.pose)];
            let pair = model.geometry.products
                [model.geometry.row_bases[usize::from(inverse_b)] + usize::from(a.pose)];
            work.h4_reads = work.h4_reads.saturating_add(4);
            add(7, op, (u64::from(relative) << 8) | u64::from(pair));
        }
        if !matches!(control, Control::GeometryDisabled | Control::ZetaDisabled) {
            for channel in 0..PHASE_CHANNELS {
                let phase = endpoint.phases[channel].wrapping_sub(a.phases[channel]);
                let pair = a.phases[channel].wrapping_sub(b.phases[channel]);
                work.phase_updates = work.phase_updates.saturating_add(2);
                add(
                    8 + channel as u8,
                    op,
                    (u64::from(phase >> 12) << 4) | u64::from(pair >> 12),
                );
            }
        }
        if control != Control::ValueLexemesDisabled
            && model
                .values
                .as_ref()
                .is_some_and(|head| head.schema == LEXEME_VALUE_SCHEMA)
        {
            if let (Some(words), Some(cues_a), Some(cues_b)) = (&self.lexemes, a.lexical, b.lexical)
            {
                for (index, query) in words.queries[..words.query_len].iter().enumerate() {
                    let mut matches = 0;
                    for offset in 0..4 {
                        if query.matches(&cues_a[offset], work) {
                            matches |= 1 << offset;
                        }
                        if query.matches(&cues_b[offset], work) {
                            matches |= 1 << (offset + 4);
                        }
                    }
                    add(16, (op << 8) | index as u64, matches);
                }
            }
        }
        (features, len)
    }
    pub(super) fn offer(
        &mut self,
        model: &Model,
        baseline: Candidate,
        control: Control,
        work: &mut ValueWork,
    ) -> Option<Candidate> {
        self.pending = None;
        if !self.active || control == Control::ValuesDisabled || control == Control::MemoryDisabled
        {
            return None;
        }
        let head = model.values.as_ref()?;
        if let Some(emission) = self.emission {
            if usize::from(emission.cursor) >= usize::from(emission.numeral.len)
                || head.continuation_score <= 0
            {
                return None;
            }
            let token = emission.numeral.tokens[usize::from(emission.cursor)];
            let score = baseline.score + i64::from(head.continuation_score);
            self.pending = Some(ValueDecision {
                token,
                cursor: emission.cursor,
                score,
                at_seen: self.seen,
                ..emission.decision
            });
            return Some(Candidate { token, score });
        }
        if self.consumed || self.query_len == 0 || self.next_id == u64::MAX {
            return None;
        }
        let mut selected = None;
        let mut best = 0_i64;
        for index in 0..272 {
            let Some((action, a, b)) = self.proposal(index) else {
                continue;
            };
            work.proposals = work.proposals.saturating_add(1);
            let Some(value) = execute(action, a.value, b.value, work) else {
                continue;
            };
            let (features, len) = self.features(model, action, a, b, control, work);
            let mut score = 0_i64;
            for feature in &features[..len] {
                work.feature_lookups = work.feature_lookups.saturating_add(1);
                if let Ok(index) = head.rows.binary_search_by(|row| {
                    work.feature_comparisons = work.feature_comparisons.saturating_add(1);
                    row.feature.cmp(feature)
                }) {
                    score += i64::from(head.rows[index].weight);
                }
            }
            if score > best {
                best = score;
                selected = Some((action, a, b, value));
            }
        }
        let (action, a, b, value) = selected?;
        let numeral = Numeral::from_zphi(ZPhi::new(value, 0))?;
        // Exact spelling has nineteen place visits plus one subtraction per
        // digit value; this counter is the fixed worst-case visit bound.
        work.numeral_steps = work.numeral_steps.saturating_add(190);
        let token = numeral.tokens[0];
        let score = baseline.score + best;
        self.pending = Some(ValueDecision {
            action,
            operands: [a, b],
            value,
            write_id: self.next_id,
            token,
            cursor: 0,
            score,
            at_seen: self.seen,
        });
        Some(Candidate { token, score })
    }
    pub(super) fn selected(&mut self, best: Candidate) {
        if self
            .pending
            .is_some_and(|p| p.token != best.token || p.score != best.score)
        {
            self.pending = None;
        }
    }
    fn commit(&mut self, token: u32, work: &mut ValueWork) {
        let pending = self.pending.take();
        if token == EOS {
            self.active = false;
            self.emission = None;
            return;
        }
        let Some(decision) = pending.filter(|p| p.token == token && p.at_seen == self.seen) else {
            let interrupted = self.emission.take().is_some();
            if interrupted {
                self.consumed = true;
            }
            if pending.is_some() || interrupted {
                work.emission_mismatches = work.emission_mismatches.saturating_add(1);
            }
            return;
        };
        if decision.cursor == 0 {
            let Some(next) = self.next_id.checked_add(1) else {
                return;
            };
            let Some(numeral) = Numeral::from_zphi(ZPhi::new(decision.value, 0)) else {
                return;
            };
            work.numeral_steps = work.numeral_steps.saturating_add(190);
            self.next_id = next;
            let mut cue = [0; 4];
            for (dst, entry) in cue.iter_mut().zip(&self.queries[..self.query_len]) {
                *dst = entry.cue;
            }
            self.append_record(
                ValueRecord {
                    id: decision.write_id,
                    value: decision.value,
                    start: self.seen,
                    end: self.seen,
                    derived: true,
                    derivation: Some(ValueDerivation {
                        action: decision.action,
                        operand_ids: decision.operands.map(|record| record.id),
                        operand_values: decision.operands.map(|record| record.value),
                    }),
                    cue,
                    pose: self.pose,
                    phases: self.phases,
                    lexical: self.lexemes.as_ref().map(|words| {
                        let mut cue = [super::value_lexemes::WordAtom::default(); 4];
                        cue.copy_from_slice(&words.queries[..4]);
                        cue
                    }),
                },
                work,
            );
            self.emission = Some(ValueEmission {
                decision,
                numeral,
                cursor: 1,
            });
            self.consumed = true;
            work.derived_writes = work.derived_writes.saturating_add(1);
        } else if let Some(emission) = &mut self.emission {
            emission.cursor = emission.cursor.saturating_add(1);
        }
        work.emission_commits = work.emission_commits.saturating_add(1);
        if self.emission.is_some_and(|e| e.cursor >= e.numeral.len) {
            self.emission = None;
        }
    }
}
pub(super) fn execute(action: ValueAction, a: i64, b: i64, work: &mut ValueWork) -> Option<i64> {
    match action {
        ValueAction::Copy => Some(a),
        ValueAction::Add => {
            work.additions = work.additions.saturating_add(1);
            match ZPhi::new(a, 0).checked_add(ZPhi::new(b, 0)) {
                Ok(value) => Some(value.a),
                Err(_) => {
                    work.overflow_rejections = work.overflow_rejections.saturating_add(1);
                    None
                }
            }
        }
    }
}
