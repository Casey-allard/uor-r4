//! Host validation of bounded typed state. A checkpoint carries user-provided
//! old state, not authenticated source truth. Retained token evidence is
//! checked where available; evicted literal payloads remain explicit state.
use super::*;
use crate::native_geometric::numeral::{Numeral, Scanner};
use crate::native_geometric::value_lexemes::{LexemeState, WordAtom, WordScanner};
use crate::native_geometric::value_types::{
    ValueAction, ValueDerivation, ValueEntry, ValueRecord, ValueWork, LEXEME_VALUE_SCHEMA, QUERY,
    VALUES,
};
use crate::prime_route_attention::ZPhi;

fn invalid(message: &str) -> Error {
    Error(format!("session typed value {message}"))
}

pub(super) fn validate_lexeme_field_presence(
    model: &Model,
    wire: &serde_json::Value,
) -> Result<()> {
    let Some(head) = &model.values else {
        return Ok(());
    };
    let Some(values) = wire.get("values").and_then(serde_json::Value::as_object) else {
        return Ok(());
    };
    let lexical = head.schema == LEXEME_VALUE_SCHEMA;
    if if lexical {
        !values
            .get("lexemes")
            .is_some_and(serde_json::Value::is_object)
    } else {
        values.contains_key("lexemes")
    } {
        return Err(invalid(
            "word state fields do not match the artifact schema",
        ));
    }
    let validate_record = |record: &serde_json::Value| -> Result<()> {
        if if lexical {
            !record
                .get("lexical")
                .is_some_and(serde_json::Value::is_array)
        } else {
            record.get("lexical").is_some()
        } {
            return Err(invalid(
                "record word fields do not match the artifact schema",
            ));
        }
        Ok(())
    };
    for field in ["records", "sources"] {
        if let Some(records) = values.get(field).and_then(serde_json::Value::as_array) {
            for record in records {
                validate_record(record)?;
            }
        }
    }
    if let Some(operands) = values
        .get("emission")
        .and_then(|emission| emission.get("decision"))
        .and_then(|decision| decision.get("operands"))
        .and_then(serde_json::Value::as_array)
    {
        for record in operands {
            validate_record(record)?;
        }
    }
    Ok(())
}

fn bytes(model: &Model, token: u32, mut consume: impl FnMut(u8)) {
    if token == BOS || token == EOS {
        return;
    }
    if token < LEXICAL_BASE {
        consume((token - 2) as u8);
    } else {
        for &byte in &model.lexical_pieces[(token - LEXICAL_BASE) as usize] {
            consume(byte);
        }
    }
}

impl Session {
    pub(super) fn restore_value_state(
        &mut self,
        model: &Model,
        mut saved: ValueState,
        retained: &[u32],
        observed: u64,
        source_bytes: u64,
    ) -> Result<()> {
        let lexical = model
            .values
            .as_ref()
            .is_some_and(|head| head.schema == LEXEME_VALUE_SCHEMA);
        if saved.seen != observed
            || saved.lexemes.is_some() != lexical
            || saved.recent_len != observed.min(32) as usize
            || saved.recent_cursor != (observed & 31) as usize
            || saved.records.len() != saved.next_id.min(VALUES as u64) as usize
            || saved.sources.len() > VALUES
            || saved.query_len > QUERY
            || saved.started_at > observed
            || usize::from(saved.pose) >= model.geometry.inverses.len()
            || !saved.scanner.snapshot_valid(observed)
            || (saved.active && saved.scanner != Scanner::default())
            || (saved.active && saved.query_len != saved.started_at.min(QUERY as u64) as usize)
            || (saved.query_len != 0
                && saved.query_len != saved.started_at.min(QUERY as u64) as usize)
            || saved.pending.is_some()
        {
            return Err(invalid("shape, scanner or counters are invalid"));
        }
        let oldest = observed - saved.recent_len as u64;
        let ring_oldest = observed - retained.len() as u64;
        let recent = |sequence: u64| -> Option<ValueEntry> {
            if sequence < oldest || sequence >= observed {
                return None;
            }
            Some(saved.recent[(sequence & 31) as usize])
        };
        let cue = |token: u32| {
            let alias = model
                .memory_read
                .as_ref()
                .and_then(|memory| memory.cue_aliases.as_ref())
                .map_or(token, |aliases| aliases.representatives[token as usize]);
            model.geometry.tokens[alias as usize].prime
        };
        let valid_entry = |entry: ValueEntry| -> bool {
            (entry.token as usize) < model.vocabulary_size()
                && usize::from(entry.pose) < model.geometry.inverses.len()
                && entry.cue == cue(entry.token)
        };
        let follows = |older: ValueEntry, newer: ValueEntry| {
            let geometry = &model.geometry.tokens[newer.token as usize];
            model.geometry.products
                [model.geometry.row_bases[usize::from(older.pose)] + usize::from(geometry.leaf)]
                == newer.pose
                && older
                    .phases
                    .iter()
                    .zip(geometry.phases)
                    .zip(newer.phases)
                    .all(|((&old, delta), new)| old.wrapping_add(delta) == new)
        };
        let mut previous = None;
        for sequence in oldest..observed {
            let entry = saved.recent[(sequence & 31) as usize];
            if entry.sequence != sequence
                || !valid_entry(entry)
                || (sequence >= ring_oldest
                    && entry.token != retained[(sequence - ring_oldest) as usize])
                || previous.is_some_and(|older| !follows(older, entry))
            {
                return Err(invalid("recent token or geometric path is invalid"));
            }
            if sequence == 0 {
                let geometry = &model.geometry.tokens[entry.token as usize];
                if entry.pose != geometry.leaf || entry.phases != geometry.phases {
                    return Err(invalid("initial token geometry is invalid"));
                }
            }
            previous = Some(entry);
        }
        if let Some(last) = previous {
            if last.pose != saved.pose || last.phases != saved.phases {
                return Err(invalid("current geometry differs from recent history"));
            }
        } else if saved.pose != model.geometry.identity || saved.phases != [0; PHASE_CHANNELS] {
            return Err(invalid("empty state geometry is invalid"));
        }
        let validate_atom = |atom: &WordAtom| -> Result<()> {
            if !atom.snapshot_valid(observed, source_bytes, model.geometry.inverses.len()) {
                return Err(invalid(
                    "word bytes, sequence or geometric endpoint are invalid",
                ));
            }
            if atom.len == 0 {
                return Ok(());
            }
            if recent(atom.end)
                .is_some_and(|entry| entry.pose != atom.pose || entry.phases != atom.phases)
            {
                return Err(invalid("word endpoint differs from retained geometry"));
            }
            // A word spans at most one token per source byte. With complete
            // history, even a long word within the first lexical token has
            // exact retained evidence; partial windows need the span bound.
            let span = u64::from(atom.len) - 1;
            if atom.end >= oldest && (oldest == 0 || atom.end - oldest >= span) {
                let mut found = false;
                let matches = |word: WordAtom| {
                    word.len == atom.len
                        && word.bytes == atom.bytes
                        && word.end == atom.end
                        && word.pose == atom.pose
                        && word.phases == atom.phases
                };
                for start in atom.end.saturating_sub(span).max(oldest)..=atom.end {
                    let mut replay = LexemeState::default();
                    let mut work = ValueWork::default();
                    for sequence in start..=atom.end {
                        let entry = saved.recent[(sequence & 31) as usize];
                        bytes(model, entry.token, |byte| {
                            let before = work.lexical_writes;
                            replay.feed(byte, entry, &mut work);
                            if work.lexical_writes != before {
                                found |= matches(replay.recent[0]);
                            }
                        });
                    }
                    replay.finish(&mut work);
                    found |= replay.recent_len != 0 && matches(replay.recent[0]);
                    if found {
                        break;
                    }
                }
                if !found {
                    return Err(invalid("word payload differs from retained source bytes"));
                }
            }
            Ok(())
        };
        let validate_word_array = |atoms: &[WordAtom]| -> Result<()> {
            let len = atoms.iter().take_while(|atom| atom.len != 0).count();
            for atom in atoms {
                validate_atom(atom)?;
            }
            if atoms[len..].iter().any(|atom| *atom != WordAtom::default())
                || atoms[..len].windows(2).any(|pair| {
                    pair[0].byte_end < u64::from(pair[0].len)
                        || pair[1].byte_end > pair[0].byte_end - u64::from(pair[0].len)
                        || pair[1].end > pair[0].end
                })
            {
                return Err(invalid("word order or unused padding is invalid"));
            }
            Ok(())
        };
        if let Some(words) = &saved.lexemes {
            if !words.snapshot_valid(observed, model.geometry.inverses.len())
                || words.source_bytes_seen != source_bytes
                || (saved.active
                    && (words.scanner != WordScanner::default()
                        || words.queries != words.recent
                        || words.query_len != words.recent_len))
                || (saved.query_len == 0 && words.query_len != 0)
                || (words.scanner.pending.len != 0
                    && words.recent_len != 0
                    && words
                        .scanner
                        .pending
                        .byte_end
                        .checked_sub(u64::from(words.scanner.pending.len))
                        .is_none_or(|prior_limit| words.recent[0].byte_end > prior_limit))
                || words.queries[..words.query_len]
                    .iter()
                    .any(|word| word.end >= saved.started_at)
                || (saved.scanner.snapshot_needs_suffix()
                    && words.literal_cues.as_slice() != &words.recent[..4])
                || (!saved.scanner.snapshot_needs_suffix()
                    && words.literal_cues != [WordAtom::default(); 4])
            {
                return Err(invalid(
                    "word scanner, source counters or captured query are invalid",
                ));
            }
            validate_word_array(&words.recent)?;
            validate_word_array(&words.queries)?;
            validate_word_array(&words.literal_cues)?;
            validate_atom(&words.scanner.pending)?;
            if words.scanner.pending.len != 0 {
                let pending = words.scanner.pending;
                // Unlike completed atoms, a pending word must end at the
                // current observation and agree with an unclosed byte suffix.
                if pending.end.checked_add(1) != Some(observed) {
                    return Err(invalid(
                        "pending word is not at the current source boundary",
                    ));
                }
                let mut found = false;
                for start in oldest..observed {
                    let mut replay = LexemeState::default();
                    for sequence in start..observed {
                        let entry = saved.recent[(sequence & 31) as usize];
                        bytes(model, entry.token, |byte| {
                            replay.feed(byte, entry, &mut ValueWork::default())
                        });
                    }
                    let actual = replay.scanner.pending;
                    found |= pending.len == actual.len
                        && pending.bytes == actual.bytes
                        && pending.end == actual.end
                        && pending.pose == actual.pose
                        && pending.phases == actual.phases;
                    if found {
                        break;
                    }
                }
                if !found {
                    return Err(invalid("pending word differs from retained source suffix"));
                }
            }
        }
        // An explicit end_response may reset the scanner at any token
        // boundary. Require an open numeric prefix to be reachable from one
        // such boundary within the retained raw-token metadata. Long rejected
        // or identifier runs can predate that metadata and cannot emit values.
        if saved.scanner.snapshot_needs_suffix() {
            let mut matched = false;
            for start in oldest..observed {
                let mut scanner = Scanner::default();
                for sequence in start..observed {
                    let entry = saved.recent[(sequence & 31) as usize];
                    bytes(model, entry.token, |byte| {
                        scanner.feed(byte, sequence);
                    });
                }
                if scanner == saved.scanner {
                    matched = true;
                    break;
                }
            }
            if !matched {
                return Err(invalid(
                    "scanner is inconsistent with retained source bytes",
                ));
            }
        }
        for offset in 0..saved.query_len {
            let entry = saved.queries[offset];
            if !valid_entry(entry)
                || entry.sequence.checked_add(offset as u64 + 1) != Some(saved.started_at)
                || recent(entry.sequence).is_some_and(|actual| actual != entry)
                || (offset != 0 && !follows(entry, saved.queries[offset - 1]))
            {
                return Err(invalid("captured query is invalid"));
            }
        }
        let valid_prime = |prime: u32| {
            prime == 0
                || model
                    .geometry
                    .tokens
                    .iter()
                    .any(|token| token.prime == prime)
        };
        let validate_record = |record: &ValueRecord| -> Result<()> {
            if record.id >= saved.next_id
                || record.start > record.end
                || record.end >= observed
                || usize::from(record.pose) >= model.geometry.inverses.len()
                || record.cue.iter().any(|&prime| !valid_prime(prime))
                || record.derived != record.derivation.is_some()
                || record.lexical.is_some() != lexical
                || (record.derived && record.start != record.end)
                || (!record.derived && record.end - record.start > 19)
            {
                return Err(invalid("record identity, interval or geometry is invalid"));
            }
            if let Some(words) = &record.lexical {
                validate_word_array(words)?;
                if words.iter().any(|word| {
                    word.len != 0
                        && (word.end > record.start || (record.derived && word.end == record.start))
                }) {
                    return Err(invalid("record word cue follows its value source boundary"));
                }
                if !record.derived {
                    let state = saved
                        .lexemes
                        .as_ref()
                        .ok_or_else(|| invalid("record has words without word state"))?;
                    // A shared lexical token can contain words on both sides
                    // of the number. Without a byte interval for that number,
                    // its token endpoint cannot order those same-token words.
                    if !state.recent[..state.recent_len]
                        .iter()
                        .any(|word| word.end == record.start)
                    {
                        let mut compared = 0;
                        for (index, actual) in state.recent[..state.recent_len]
                            .iter()
                            .filter(|word| word.end < record.start)
                            .take(4)
                            .enumerate()
                        {
                            if words[index] != *actual {
                                return Err(invalid(
                                    "literal word cues differ from retained source order",
                                ));
                            }
                            compared += 1;
                        }
                        if state.recent_len < state.recent.len()
                            && words[compared..]
                                .iter()
                                .any(|word| *word != WordAtom::default())
                        {
                            return Err(invalid(
                                "literal word cues invent unavailable earlier words",
                            ));
                        }
                    }
                }
            }
            if let Some(derivation) = record.derivation {
                let [left_id, right_id] = derivation.operand_ids;
                let [left, right] = derivation.operand_values;
                let computed = match derivation.action {
                    ValueAction::Copy if left_id == right_id && left == right => Some(left),
                    ValueAction::Add if left_id != right_id => ZPhi::new(left, 0)
                        .checked_add(ZPhi::new(right, 0))
                        .ok()
                        .map(|value| value.a),
                    _ => None,
                };
                if left_id >= record.id || right_id >= record.id || computed != Some(record.value) {
                    return Err(invalid(
                        "retained derivation identity or arithmetic is invalid",
                    ));
                }
                for (id, value) in derivation
                    .operand_ids
                    .into_iter()
                    .zip(derivation.operand_values)
                {
                    if saved
                        .records
                        .iter()
                        .chain(&saved.sources)
                        .any(|operand| operand.id == id && operand.value != value)
                    {
                        return Err(invalid("derivation operand differs from retained value"));
                    }
                }
            }
            if !record.derived {
                if recent(record.end)
                    .is_some_and(|entry| entry.pose != record.pose || entry.phases != record.phases)
                {
                    return Err(invalid("literal endpoint differs from retained geometry"));
                }
                for (offset, &prime) in record.cue.iter().enumerate() {
                    if let Some(sequence) = record.start.checked_sub(offset as u64 + 1) {
                        if recent(sequence).is_some_and(|entry| entry.cue != prime) {
                            return Err(invalid("literal cue differs from retained tokens"));
                        }
                    } else if prime != 0 {
                        return Err(invalid("literal cue precedes observation history"));
                    }
                }
                if record.start >= oldest {
                    let mut scanner = Scanner::default();
                    let mut found = false;
                    for sequence in record.start..=record.end {
                        let entry = saved.recent[(sequence & 31) as usize];
                        bytes(model, entry.token, |byte| {
                            if let Some(literal) = scanner.feed(byte, sequence) {
                                found |= literal.start == record.start
                                    && literal.end == record.end
                                    && literal.value == record.value;
                            }
                        });
                    }
                    if let Some(literal) = scanner.finish() {
                        found |= literal.start == record.start
                            && literal.end == record.end
                            && literal.value == record.value;
                    }
                    if !found {
                        return Err(invalid(
                            "literal payload differs from retained source bytes",
                        ));
                    }
                }
            }
            Ok(())
        };
        for (offset, record) in saved.records.iter().enumerate() {
            validate_record(record)?;
            if record.id != saved.next_id - saved.records.len() as u64 + offset as u64 {
                return Err(invalid("record IDs are not the retained write suffix"));
            }
        }
        let mut previous_id = None;
        for record in &saved.sources {
            validate_record(record)?;
            if record.end >= saved.started_at
                || previous_id.is_some_and(|id| record.id != id + 1)
                || saved
                    .records
                    .iter()
                    .any(|current| current.id == record.id && current != record)
            {
                return Err(invalid("captured source identity is invalid"));
            }
            previous_id = Some(record.id);
        }
        if let Some(emission) = saved.emission {
            let decision = emission.decision;
            let [a, b] = decision.operands;
            let computed = match decision.action {
                ValueAction::Copy if a == b => Some(a.value),
                ValueAction::Add if a.id != b.id => ZPhi::new(a.value, 0)
                    .checked_add(ZPhi::new(b.value, 0))
                    .ok()
                    .map(|value| value.a),
                _ => None,
            };
            let numeral = Numeral::from_zphi(ZPhi::new(decision.value, 0));
            if !saved.active
                || !saved.consumed
                || saved.query_len == 0
                || decision.cursor != 0
                || emission.cursor == 0
                || emission.cursor >= emission.numeral.len
                || decision.at_seen < saved.started_at
                || decision.at_seen.checked_add(u64::from(emission.cursor)) != Some(observed)
                || decision.write_id.checked_add(1) != Some(saved.next_id)
                || computed != Some(decision.value)
                || numeral != Some(emission.numeral)
                || decision.token != emission.numeral.tokens[0]
                || !saved.sources.contains(&a)
                || !saved.sources.contains(&b)
            {
                return Err(invalid("committed derivation or numeral cursor is invalid"));
            }
            let record = saved
                .records
                .last()
                .ok_or_else(|| invalid("emission has no committed write"))?;
            let mut expected_cue = [0; 4];
            for (prime, query) in expected_cue
                .iter_mut()
                .zip(&saved.queries[..saved.query_len])
            {
                *prime = query.cue;
            }
            if !record.derived
                || record.id != decision.write_id
                || record.value != decision.value
                || record.derivation
                    != Some(ValueDerivation {
                        action: decision.action,
                        operand_ids: [a.id, b.id],
                        operand_values: [a.value, b.value],
                    })
                || record.start != decision.at_seen
                || record.end != decision.at_seen
                || record.cue != expected_cue
                || record.lexical
                    != saved.lexemes.as_ref().map(|words| {
                        let mut expected = [WordAtom::default(); 4];
                        expected.copy_from_slice(&words.queries[..4]);
                        expected
                    })
                || decision
                    .at_seen
                    .checked_sub(1)
                    .and_then(recent)
                    .is_some_and(|entry| entry.pose != record.pose || entry.phases != record.phases)
            {
                return Err(invalid("derived record differs from committed emission"));
            }
            for offset in 0..emission.cursor {
                let sequence = decision.at_seen + u64::from(offset);
                if recent(sequence)
                    .is_none_or(|entry| entry.token != emission.numeral.tokens[usize::from(offset)])
                {
                    return Err(invalid(
                        "emission cursor differs from observed numeral bytes",
                    ));
                }
            }
        }
        // Deserialization allocates Vec capacity from serialized length;
        // restore the fixed serving capacity before any model observation.
        let mut records = Vec::with_capacity(VALUES);
        records.extend_from_slice(&saved.records);
        saved.records = records;
        let mut sources = Vec::with_capacity(VALUES);
        sources.extend_from_slice(&saved.sources);
        saved.sources = sources;
        saved.pending = None;
        self.values = Some(saved);
        Ok(())
    }
}
