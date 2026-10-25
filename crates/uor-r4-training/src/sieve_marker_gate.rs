//! A discourse-marker gate over the prime-sieve frame compiler
//! (`m3-memory-turn-discriminator-gate`).
//!
//! A marker is the cofactor one training frame leaves when it is sieved by
//! another training frame of the same relation: `f = m ++ g` on the words
//! before the slot, with the same words after it, so `sig(f) = sig(m) ·
//! sig(g)`. "Actually, my hometown is {v}." sieved by "My hometown is {v}."
//! leaves the marker "actually". Markers are found only in the training
//! phrasings, and they are relation-independent: a marker found on one
//! relation's frames may precede any relation's slot frame.
//!
//! A turn compiles through the gate when it is at most [`MAX_MARKERS`]
//! markers followed by an exact training slot frame and its value, or an
//! exact frame alone (the sieve compiler's own reading). The margin gate is
//! exact explanation: every word outside the value is a marker word or a
//! frame word, and every surviving reading names one expert and one span.
//! The decision runs on the GPU (`sieve_frame.cu`, `sieve_frame_compile_marked`);
//! this module prepares data and decodes stored decisions.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::sieve_frame_compiler::{Act, DeviceFrames, FrameTable, Word};
use crate::turn_action::{CompiledAction, SourceSpan};
use crate::{invalid, sha256_bytes, Result};

/// The marker table's schema.
pub const MARKER_TABLE_SCHEMA: &str = "uor-r4.sieve-marker-table/1";
/// The core-cofactor gate table's schema.
pub const CORE_COFACTOR_TABLE_SCHEMA: &str = "uor-r4.sieve-core-cofactor-table/1";
/// The most markers a turn may carry before its frame.
pub const MAX_MARKERS: u32 = 2;
/// The SHA-256 of the world=v2 marker table's bytes, registered before the
/// gated run; its test fails if the frames, the rule or the layout change.
pub const WORLD_V2_MARKER_TABLE_SHA256: &str =
    "cd79399490c0abcfc14253b5680a282fc840672c5932e925d62a3b95375a125d";
/// The SHA-256 of the world=v2 core-cofactor gate table's bytes.
pub const WORLD_V2_CORE_COFACTOR_TABLE_SHA256: &str =
    "b5bd388410a82f14ca6a855f0e420b159f99ad1dd77467e46f33b390ead829d4";

/// One discourse marker: lexicon word IDs, and whether it turns an assert
/// frame into a correction.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Marker {
    pub words: Vec<u32>,
    pub update: bool,
}

/// The markers found in a frame table, bound to that table by digest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkerTable {
    pub schema: String,
    pub frame_table_sha256: String,
    pub max_markers: u32,
    pub markers: Vec<Marker>,
}

impl MarkerTable {
    /// Every marker the training frames imply. A sequence is an update
    /// marker when it only ever turns an assert frame into an update frame
    /// (or sits between two update frames), and neutral when it never
    /// changes the act. A sequence that would erase a correction, or that
    /// plays both roles, is left out.
    pub fn derive(table: &FrameTable) -> Result<Self> {
        let mut pairs: BTreeMap<Vec<u32>, BTreeSet<(Act, Act)>> = BTreeMap::new();
        let slots = || table.frames.iter().filter(|f| f.slot);
        for f in slots() {
            for g in slots().filter(|g| g.relation == f.relation && g.suffix == f.suffix) {
                if f.prefix.len() <= g.prefix.len() {
                    continue;
                }
                let cut = f.prefix.len() - g.prefix.len();
                if f.prefix[cut..] == g.prefix[..] {
                    pairs
                        .entry(f.prefix[..cut].to_vec())
                        .or_default()
                        .insert((f.act, g.act));
                }
            }
        }
        let mut markers = Vec::new();
        for (words, kinds) in pairs {
            let erases = kinds.contains(&(Act::Assert, Act::Update));
            let converts = kinds.contains(&(Act::Update, Act::Assert));
            let keeps_assert = kinds.contains(&(Act::Assert, Act::Assert));
            if erases || (converts && keeps_assert) {
                continue;
            }
            markers.push(Marker {
                words,
                update: converts,
            });
        }
        Ok(Self {
            schema: MARKER_TABLE_SCHEMA.into(),
            frame_table_sha256: table.identity()?.artifact_sha256,
            max_markers: MAX_MARKERS,
            markers,
        })
    }

    /// The marker table's bytes: its JSON, in field and table order.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| invalid(format!("marker table JSON: {e}")))
    }

    pub fn sha256(&self) -> Result<String> {
        Ok(sha256_bytes(&self.canonical_bytes()?))
    }

    /// The markers in the flat layout the kernel reads.
    pub fn device_markers(&self) -> Result<DeviceMarkers> {
        device_markers_from_slice(&self.markers)
    }
}

fn device_markers_from_slice(markers: &[Marker]) -> Result<DeviceMarkers> {
    let mut out = DeviceMarkers::default();
    for marker in markers {
        out.offset.push(len_u32(out.tokens.len())?);
        out.length.push(len_u32(marker.words.len())?);
        out.update.push(u32::from(marker.update));
        out.tokens.extend(&marker.words);
    }
    Ok(out)
}

/// One frame derived from [`FrameTable`] by cofactor division, contraction
/// symmetry, or fourth-proportional core/shell completion, mapped back to its
/// canonical `base_frame` index in `table.frames`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerivedFrame {
    pub base_frame: u32,
    pub relation: u32,
    pub act: Act,
    pub slot: bool,
    pub prefix: Vec<u32>,
    pub suffix: Vec<u32>,
}

/// Combined device frame tables for the gated kernel, including `frame_base`
/// mapping each frame index back to its canonical `0..table.frames.len()` index.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GateFrameSet {
    pub frames: DeviceFrames,
    pub frame_act: Vec<u32>,
    pub frame_expert_update: Vec<u64>,
    pub frame_base: Vec<u32>,
}

/// Two-sided cofactor markers and fourth-proportional core/shell frame
/// completions derived strictly from [`FrameTable`] (`Split::Train` only).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreCofactorTable {
    pub schema: String,
    pub frame_table_sha256: String,
    pub max_markers: u32,
    pub markers: Vec<Marker>,
    pub derived_frames: Vec<DerivedFrame>,
}

impl CoreCofactorTable {
    /// Derive all prefix markers and completed frames strictly from `table`
    /// (`Split::Train` phrasings only).
    pub fn derive(table: &FrameTable) -> Result<Self> {
        let base_markers = MarkerTable::derive(table)?.markers;
        let mut marker_map: BTreeMap<Vec<u32>, bool> = base_markers
            .iter()
            .map(|m| (m.words.clone(), m.update))
            .collect();
        let mut suffix_markers: BTreeSet<Vec<u32>> = BTreeSet::new();
        let mut preslot_markers: BTreeSet<Vec<u32>> = BTreeSet::new();

        // 1. Two-sided cofactors (f.prefix = m_pre ++ g.prefix, f.suffix = g.suffix ++ m_suf)
        let slots = || table.frames.iter().filter(|f| f.slot);
        for f in slots() {
            for g in slots().filter(|g| g.relation == f.relation) {
                if f.prefix.len() > g.prefix.len()
                    && f.suffix.len() > g.suffix.len()
                    && f.prefix[f.prefix.len() - g.prefix.len()..] == g.prefix[..]
                    && f.suffix[..g.suffix.len()] == g.suffix[..]
                {
                    let m_pre = f.prefix[..f.prefix.len() - g.prefix.len()].to_vec();
                    let m_suf = f.suffix[g.suffix.len()..].to_vec();
                    let pre_is_neutral =
                        marker_map.iter().any(|(k, &u)| !u && k.starts_with(&m_pre));
                    marker_map.entry(m_pre).or_insert(!pre_is_neutral);
                    if f.act == Act::Update {
                        suffix_markers.insert(m_suf.clone());
                        marker_map.entry(m_suf).or_insert(true);
                    }
                }
            }
        }

        // 2. Left-division closure on prefix markers (m_long = m1 ++ m2)
        let snapshot: Vec<(Vec<u32>, bool)> =
            marker_map.iter().map(|(k, &v)| (k.clone(), v)).collect();
        for (m_long, u_long) in &snapshot {
            for (m1, _) in &snapshot {
                if m_long.len() > m1.len() && m_long.starts_with(m1) {
                    let rem = m_long[m1.len()..].to_vec();
                    marker_map.entry(rem).or_insert(*u_long);
                }
            }
        }

        // 3. Pre-slot cofactor (f.prefix = m_pre ++ g.prefix ++ m_slot, f.suffix == g.suffix)
        for f in slots().filter(|f| f.act == Act::Update) {
            for g in slots().filter(|g| g.relation == f.relation && g.suffix == f.suffix) {
                for (m_pre, _) in &snapshot {
                    let need = m_pre.len() + g.prefix.len();
                    if f.prefix.len() > need
                        && f.prefix.starts_with(m_pre)
                        && f.prefix[m_pre.len()..need] == g.prefix[..]
                    {
                        preslot_markers.insert(f.prefix[need..].to_vec());
                    }
                }
            }
        }

        // 4. Cross-relation shared Update prefix ([72] & [80] vs [70] & [78])
        let mut raw_derived: BTreeSet<(u32, Act, bool, Vec<u32>, Vec<u32>)> = BTreeSet::new();
        for f1 in slots().filter(|f| f.act == Act::Update) {
            for f2 in slots().filter(|f| f.act == Act::Update && f.relation != f1.relation) {
                let lcp_len = f1
                    .prefix
                    .iter()
                    .zip(&f2.prefix)
                    .take_while(|(a, b)| a == b)
                    .count();
                if lcp_len >= 2 {
                    let lcp = &f1.prefix[..lcp_len];
                    for a in slots().filter(|a| a.relation == f1.relation && a.act == Act::Assert) {
                        if lcp.len() > a.prefix.len() && lcp.ends_with(&a.prefix) {
                            let cut = lcp.len() - a.prefix.len();
                            marker_map.insert(lcp[..cut].to_vec(), true);
                            // Strip marker from f1 and f2 if suffix has a relation-specific word
                            for f in [f1, f2] {
                                let rem_pre = f.prefix[cut..].to_vec();
                                raw_derived.insert((
                                    f.relation,
                                    Act::Assert,
                                    true,
                                    rem_pre,
                                    f.suffix.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }

        // 5. Discover 2-word <-> 1-word contractions from table.frames:
        //    - Query contraction: [what, is] <-> [what's] (from same-relation query pair)
        //    - Assert contraction: [i, am] <-> [i'm] (from 1-word Assert [i'm] and [i, am, a] vs [..., i'm, a])
        let mut query_contraction: Option<([u32; 2], u32)> = None;
        for q1 in table.frames.iter().filter(|f| !f.slot) {
            for q2 in table
                .frames
                .iter()
                .filter(|f| !f.slot && f.relation == q1.relation)
            {
                if q1.prefix.len() == q2.prefix.len() + 1
                    && q2.prefix.len() >= 2
                    && q1.prefix[2..] == q2.prefix[1..]
                {
                    query_contraction = Some(([q1.prefix[0], q1.prefix[1]], q2.prefix[0]));
                }
            }
        }
        let mut assert_contraction: Option<([u32; 2], u32)> = None;
        if let Some(im_frame) = slots().find(|f| f.act == Act::Assert && f.prefix.len() == 1) {
            let im_tok = im_frame.prefix[0];
            for f_u in slots().filter(|f| f.act == Act::Update && f.prefix.len() >= 3) {
                let n = f_u.prefix.len();
                if f_u.prefix[n - 2] == im_tok {
                    let tail_tok = f_u.prefix[n - 1];
                    for f_a in slots().filter(|f| {
                        f.relation == f_u.relation
                            && f.act == Act::Assert
                            && f.prefix.len() == 3
                            && f.prefix[2] == tail_tok
                    }) {
                        assert_contraction = Some(([f_a.prefix[0], f_a.prefix[1]], im_tok));
                        let lead = f_u.prefix[..n - 2].to_vec();
                        let mut rel_leads = vec![lead.clone()];
                        for suf in &suffix_markers {
                            if lead.len() > suf.len() && lead.ends_with(suf) {
                                rel_leads.push(lead[..lead.len() - suf.len()].to_vec());
                            }
                        }
                        // Unmarked cofactor frames [i, am, a] and [i'm, a]
                        let a1 = f_a.prefix.clone();
                        let a2 = vec![im_tok, tail_tok];
                        raw_derived.insert((
                            f_a.relation,
                            Act::Assert,
                            true,
                            a1.clone(),
                            Vec::new(),
                        ));
                        raw_derived.insert((
                            f_a.relation,
                            Act::Assert,
                            true,
                            a2.clone(),
                            Vec::new(),
                        ));
                        for rl in &rel_leads {
                            for base_a in [&a1, &a2] {
                                let mut p = rl.clone();
                                p.extend(base_a);
                                raw_derived.insert((
                                    f_a.relation,
                                    Act::Update,
                                    true,
                                    p,
                                    Vec::new(),
                                ));
                            }
                            for g_a in slots().filter(|g| {
                                g.relation == f_a.relation
                                    && g.act == Act::Assert
                                    && g.suffix.is_empty()
                            }) {
                                let mut p = rl.clone();
                                p.extend(&g_a.prefix);
                                raw_derived.insert((
                                    f_a.relation,
                                    Act::Update,
                                    true,
                                    p,
                                    Vec::new(),
                                ));
                            }
                        }
                    }
                }
            }
        }

        // 6. Discover single-word Update infixes:
        //    Strip prefix markers from Update frames and compare to same-relation Assert frames.
        let mut noun_update_infix: Option<u32> = None;
        let mut verb_update_infix: Option<(u32, u32)> = None; // (subject_i, infix_just)
        let marker_keys: Vec<Vec<u32>> = marker_map.keys().cloned().collect();
        for f_u in slots().filter(|f| f.act == Act::Update && f.suffix.is_empty()) {
            let mut stripped_candidates = vec![f_u.prefix.clone()];
            for m in &marker_keys {
                if f_u.prefix.len() > m.len() && f_u.prefix.starts_with(m) {
                    stripped_candidates.push(f_u.prefix[m.len()..].to_vec());
                }
            }
            for cand in stripped_candidates {
                if cand != f_u.prefix {
                    raw_derived.insert((
                        f_u.relation,
                        Act::Update,
                        true,
                        cand.clone(),
                        f_u.suffix.clone(),
                    ));
                }
                for f_a in slots().filter(|f| {
                    f.relation == f_u.relation && f.act == Act::Assert && f.suffix.is_empty()
                }) {
                    if cand.len() == f_a.prefix.len() + 1 && f_a.prefix.len() >= 2 {
                        for pos in 1..cand.len() {
                            let mut without = cand[..pos].to_vec();
                            without.extend_from_slice(&cand[pos + 1..]);
                            if without == f_a.prefix {
                                // Distinguish determiner-followed noun infix (`the new code word is`)
                                // from subject-followed verb infix (`i just moved to`)
                                if f_a.prefix.len() >= 4 {
                                    noun_update_infix = Some(cand[pos]);
                                } else {
                                    verb_update_infix = Some((cand[0], cand[pos]));
                                }
                            }
                        }
                    }
                }
            }
        }

        // 7. Discover determiners {my, the}, copula `is`, and participle swap {called, named}
        let expand_query = |pre: &[u32]| -> Vec<u32> {
            if let Some(([w1, w2], whats)) = query_contraction {
                if pre.first() == Some(&whats) {
                    let mut out = vec![w1, w2];
                    out.extend_from_slice(&pre[1..]);
                    return out;
                }
            }
            pre.to_vec()
        };
        let mut det_pair: Option<(u32, u32)> = None;
        let mut copula_is: Option<u32> = None;
        for f1 in slots().filter(|f| f.act == Act::Assert && f.suffix.is_empty()) {
            for f2 in slots().filter(|f| {
                f.relation == f1.relation
                    && f.act == Act::Assert
                    && f.suffix.is_empty()
                    && f.prefix.len() == f1.prefix.len()
                    && f.prefix.len() >= 4
            }) {
                if f1.prefix[0] != f2.prefix[0] && f1.prefix[1..] == f2.prefix[1..] {
                    let a = f1.prefix[0].min(f2.prefix[0]);
                    let b = f1.prefix[0].max(f2.prefix[0]);
                    det_pair = Some((a, b));
                    copula_is = f1.prefix.last().copied();
                }
            }
        }
        let mut part_pair: Option<(u32, u32)> = None;
        for q1 in table.frames.iter().filter(|f| !f.slot) {
            let p1 = expand_query(&q1.prefix);
            for q2 in table
                .frames
                .iter()
                .filter(|f| !f.slot && f.relation == q1.relation)
            {
                let p2 = expand_query(&q2.prefix);
                if p1.len() == p2.len()
                    && p1.len() >= 4
                    && p1[..p1.len() - 1] == p2[..p2.len() - 1]
                    && p1.last() != p2.last()
                {
                    let a = (*p1.last().unwrap()).min(*p2.last().unwrap());
                    let b = (*p1.last().unwrap()).max(*p2.last().unwrap());
                    part_pair = Some((a, b));
                }
            }
        }

        // 8. Nominal Cores & Nominal Shells across all 10 relations
        if let (Some((det_my, det_the)), Some(tok_is)) = (det_pair, copula_is) {
            let dets = [det_my, det_the];
            let mut nominal_cores: BTreeSet<(u32, Vec<u32>)> = BTreeSet::new();
            for f in slots().filter(|f| f.act == Act::Assert && f.suffix.is_empty()) {
                if f.prefix.len() >= 3
                    && dets.contains(&f.prefix[0])
                    && f.prefix.last() == Some(&tok_is)
                {
                    nominal_cores.insert((f.relation, f.prefix[1..f.prefix.len() - 1].to_vec()));
                } else if f.prefix.len() >= 4
                    && dets.contains(&f.prefix[0])
                    && f.prefix[f.prefix.len() - 2] == tok_is
                {
                    // e.g. [61] `my [home] is in`
                    nominal_cores.insert((f.relation, f.prefix[1..f.prefix.len() - 2].to_vec()));
                }
            }
            // Combine [best, friend] + [friend's, name] -> [best, friend's, name] in rel=03,
            // and include sub-core [friend] when witnessed by a query ending in `[my, friend]` ([25])
            let mut loc_prep_in: Option<u32> = None;
            for f in slots().filter(|f| f.act == Act::Assert && f.suffix.is_empty()) {
                if f.prefix.len() >= 4
                    && dets.contains(&f.prefix[0])
                    && f.prefix[f.prefix.len() - 2] == tok_is
                {
                    loc_prep_in = f.prefix.last().copied();
                }
            }
            let core_snap: Vec<(u32, Vec<u32>)> = nominal_cores.iter().cloned().collect();
            for (r1, c1) in &core_snap {
                if c1.len() == 2
                    && table.frames.iter().any(|q| {
                        !q.slot
                            && q.relation == *r1
                            && q.prefix.len() >= 3
                            && q.prefix.ends_with(&[det_my, c1[1]])
                    })
                {
                    nominal_cores.insert((*r1, vec![c1[1]]));
                }
                for (r2, c2) in &core_snap {
                    if r1 == r2
                        && c1.len() == 2
                        && c2.len() == 2
                        && c1[0] != c2[0]
                        && c1[1] != c2[1]
                    {
                        nominal_cores.insert((*r1, vec![c1[0], c2[0], c2[1]]));
                    }
                }
            }

            // Collect nominal query shells from table.frames (`[left..., d] ++ C`)
            let mut query_leads: BTreeSet<Vec<u32>> = BTreeSet::new();
            for q in table.frames.iter().filter(|f| !f.slot) {
                for (r, c) in &nominal_cores {
                    if q.relation == *r
                        && q.prefix.len() > c.len() + 1
                        && q.prefix.ends_with(c)
                        && dets.contains(&q.prefix[q.prefix.len() - c.len() - 1])
                    {
                        query_leads.insert(q.prefix[..q.prefix.len() - c.len() - 1].to_vec());
                    }
                }
            }
            if let Some(([w1, w2], whats)) = query_contraction {
                let q_snap: Vec<Vec<u32>> = query_leads.iter().cloned().collect();
                for lead in q_snap {
                    if lead == [w1, w2] {
                        query_leads.insert(vec![whats]);
                    } else if lead == [whats] {
                        query_leads.insert(vec![w1, w2]);
                    }
                }
            }

            // Instantiate nominal shells for every (r, core)
            for (r, core) in &nominal_cores {
                let is_loc_rel = loc_prep_in.is_some_and(|tok_in| {
                    table.frames.iter().any(|g| {
                        g.relation == *r
                            && (g.prefix.ends_with(&[tok_is, tok_in])
                                || (!g.slot
                                    && table.frames.iter().any(|h| {
                                        !h.slot
                                            && h.prefix.first() == g.prefix.first()
                                            && table.frames.iter().any(|k| {
                                                k.relation == h.relation
                                                    && k.prefix.ends_with(&[tok_is, tok_in])
                                            })
                                    })))
                    })
                });
                for &d in &dets {
                    // Assert: `[d] ++ core ++ [is]`
                    let mut p_assert = vec![d];
                    p_assert.extend(core);
                    p_assert.push(tok_is);
                    raw_derived.insert((*r, Act::Assert, true, p_assert.clone(), Vec::new()));
                    if let Some(tok_in) = loc_prep_in.filter(|_| is_loc_rel) {
                        let mut p_in = p_assert;
                        p_in.push(tok_in);
                        raw_derived.insert((*r, Act::Assert, true, p_in, Vec::new()));
                    }

                    // Update with `new` infix: `[d, new] ++ core ++ [is]` and `[d, core[..k-1], new, core[k-1], is]`
                    if let Some(tok_new) = noun_update_infix {
                        let mut p_new1 = vec![d, tok_new];
                        p_new1.extend(core);
                        p_new1.push(tok_is);
                        raw_derived.insert((*r, Act::Update, true, p_new1.clone(), Vec::new()));
                        if let Some(tok_in) = loc_prep_in.filter(|_| is_loc_rel) {
                            let mut p_new_in = p_new1;
                            p_new_in.push(tok_in);
                            raw_derived.insert((*r, Act::Update, true, p_new_in, Vec::new()));
                        }
                        if core.len() >= 2 {
                            let mut p_new2 = vec![d];
                            p_new2.extend_from_slice(&core[..core.len() - 1]);
                            p_new2.push(tok_new);
                            p_new2.push(*core.last().unwrap());
                            p_new2.push(tok_is);
                            raw_derived.insert((*r, Act::Update, true, p_new2, Vec::new()));
                        }
                    }

                    // Query: `lead ++ [d] ++ core`
                    for lead in &query_leads {
                        let mut p_q = lead.clone();
                        p_q.push(d);
                        p_q.extend(core);
                        raw_derived.insert((*r, Act::Query, false, p_q, Vec::new()));
                    }
                }
            }
        }

        // 9. Entity Cores & Shells (rel=02 dog, rel=03 friend / best friend)
        if let (Some((det_my, _)), Some(tok_is), Some((tok_called, tok_named))) =
            (det_pair, copula_is, part_pair)
        {
            let parts = [tok_called, tok_named];
            let mut entity_cores: BTreeSet<(u32, Vec<u32>)> = BTreeSet::new();
            for f in slots().filter(|f| f.act == Act::Assert && f.suffix.is_empty()) {
                if f.prefix.len() >= 4
                    && f.prefix[0] == det_my
                    && f.prefix[f.prefix.len() - 2] == tok_is
                    && parts.contains(f.prefix.last().unwrap())
                {
                    entity_cores.insert((f.relation, f.prefix[1..f.prefix.len() - 2].to_vec()));
                }
            }
            // Also include `[best, friend]` from nominal_cores of the same entity relation
            for f in slots().filter(|f| f.act == Act::Assert && f.suffix.is_empty()) {
                if f.prefix.len() == 4
                    && f.prefix[0] == det_my
                    && f.prefix[3] == tok_is
                    && entity_cores
                        .iter()
                        .any(|(r, c)| *r == f.relation && c == &[f.prefix[2]])
                {
                    entity_cores.insert((f.relation, f.prefix[1..3].to_vec()));
                }
            }
            // Collect `i have a` prefix from `[11] i have a dog called`
            let mut have_a_lead: Option<Vec<u32>> = None;
            for f in slots().filter(|f| f.act == Act::Assert && f.suffix.is_empty()) {
                for (r, e) in &entity_cores {
                    if f.relation == *r
                        && f.prefix.len() > e.len() + 1
                        && parts.contains(f.prefix.last().unwrap())
                        && f.prefix[f.prefix.len() - e.len() - 1..f.prefix.len() - 1] == e[..]
                    {
                        let lead = &f.prefix[..f.prefix.len() - e.len() - 1];
                        if lead != [det_my] {
                            have_a_lead = Some(lead.to_vec());
                        }
                    }
                }
            }
            // Collect `we renamed my` from `[14] we renamed my dog`
            let mut renamed_verb: Option<(u32, u32)> = None; // (we, renamed)
            for f in slots().filter(|f| f.act == Act::Update && f.suffix.is_empty()) {
                for (r, e) in &entity_cores {
                    if f.relation == *r
                        && f.prefix.len() == e.len() + 3
                        && f.prefix[2] == det_my
                        && f.prefix[3..] == e[..]
                    {
                        renamed_verb = Some((f.prefix[0], f.prefix[1]));
                    }
                }
            }
            // Collect `who is my` from `[25] who is my friend`
            let mut who_is_my: Option<Vec<u32>> = None;
            for q in table.frames.iter().filter(|f| !f.slot) {
                for (r, e) in &entity_cores {
                    if q.relation == *r
                        && q.prefix.len() == e.len() + 3
                        && q.prefix[1] == tok_is
                        && q.prefix[2] == det_my
                        && q.prefix[3..] == e[..]
                    {
                        who_is_my = Some(q.prefix[..3].to_vec());
                    }
                }
            }

            for (r, e) in &entity_cores {
                for &pt in &parts {
                    // `my [E] is called/named`
                    let mut p1 = vec![det_my];
                    p1.extend(e);
                    p1.push(tok_is);
                    p1.push(pt);
                    raw_derived.insert((*r, Act::Assert, true, p1, Vec::new()));

                    if let Some(tok_new) = noun_update_infix {
                        let mut p1_new = vec![det_my, tok_new];
                        p1_new.extend(e);
                        p1_new.push(tok_is);
                        p1_new.push(pt);
                        raw_derived.insert((*r, Act::Update, true, p1_new, Vec::new()));
                    }

                    // `i have a [E] called/named`
                    if let Some(ref lead) = have_a_lead {
                        let mut p2 = lead.clone();
                        p2.extend(e);
                        p2.push(pt);
                        raw_derived.insert((*r, Act::Assert, true, p2, Vec::new()));
                        if let Some(tok_new) = noun_update_infix {
                            let mut p2_new = lead.clone();
                            p2_new.push(tok_new);
                            p2_new.extend(e);
                            p2_new.push(pt);
                            raw_derived.insert((*r, Act::Update, true, p2_new, Vec::new()));
                        }
                    }

                    // `what is my [E] called/named` and `what's my [E] called/named`
                    if let Some(([w1, w2], whats)) = query_contraction {
                        for q_lead in [vec![w1, w2, det_my], vec![whats, det_my]] {
                            let mut pq = q_lead;
                            pq.extend(e);
                            pq.push(pt);
                            raw_derived.insert((*r, Act::Query, false, pq, Vec::new()));
                        }
                    }
                }
                if let Some((tok_we, tok_renamed)) = renamed_verb {
                    let mut subj_list = vec![tok_we];
                    if let Some((tok_i, tok_just)) = verb_update_infix {
                        subj_list.push(tok_i);
                        for &s in &[tok_we, tok_i] {
                            let mut pj = vec![s, tok_just, tok_renamed, det_my];
                            pj.extend(e);
                            raw_derived.insert((*r, Act::Update, true, pj, Vec::new()));
                        }
                    }
                    for s in subj_list {
                        let mut pr = vec![s, tok_renamed, det_my];
                        pr.extend(e);
                        raw_derived.insert((*r, Act::Update, true, pr, Vec::new()));
                    }
                }
                if let Some(ref wim) = who_is_my {
                    let mut pw = wim.clone();
                    pw.extend(e);
                    raw_derived.insert((*r, Act::Query, false, pw, Vec::new()));
                }
            }
        }

        // 10. Preference Queries & Clauses (rel=09 food, rel=10 color)
        //     Discover category cores `[food]` and `[color]` from `[74]/[83]` and `[75]/[82]`
        let mut cat_cores: BTreeSet<(u32, u32)> = BTreeSet::new();
        let mut wh_words: BTreeSet<u32> = BTreeSet::new();
        let mut pref_tails: BTreeSet<Vec<u32>> = BTreeSet::new();
        for q1 in table
            .frames
            .iter()
            .filter(|f| !f.slot && f.prefix.len() >= 5)
        {
            for q2 in table.frames.iter().filter(|f| {
                !f.slot
                    && f.relation != q1.relation
                    && f.prefix.len() == q1.prefix.len()
                    && f.prefix[0] == q1.prefix[0]
                    && f.prefix[2..] == q1.prefix[2..]
            }) {
                cat_cores.insert((q1.relation, q1.prefix[1]));
                cat_cores.insert((q2.relation, q2.prefix[1]));
                wh_words.insert(q1.prefix[0]);
                pref_tails.insert(q1.prefix[2..].to_vec());
            }
        }
        // Close pref_tails under common prefix (`[do, i, like]`) and verb swap (`love <-> like`)
        let tails_snap: Vec<Vec<u32>> = pref_tails.iter().cloned().collect();
        let mut pref_verbs: BTreeSet<u32> = BTreeSet::new();
        for t in &tails_snap {
            if t.len() >= 3 {
                pref_verbs.insert(t[2]);
                pref_tails.insert(t[..3].to_vec());
            }
        }
        for t in &tails_snap {
            if t.len() == 4 {
                let best_tok = t[3];
                for &v in &pref_verbs {
                    pref_tails.insert(vec![t[0], t[1], v, best_tok]);
                }
            }
        }
        for &(r, cat_tok) in &cat_cores {
            for &wh in &wh_words {
                for tail in &pref_tails {
                    let mut pq = vec![wh, cat_tok];
                    pq.extend(tail);
                    raw_derived.insert((r, Act::Query, false, pq, Vec::new()));
                }
            }
            // Swap `love <-> like` and category core (`food <-> color`) in Assert/Update frames of rel=09, 10
            for f in slots().filter(|f| f.relation == r) {
                for &v in &pref_verbs {
                    let swapped_pre: Vec<u32> = f
                        .prefix
                        .iter()
                        .map(|&w| if pref_verbs.contains(&w) { v } else { w })
                        .collect();
                    if swapped_pre != f.prefix {
                        raw_derived.insert((r, f.act, true, swapped_pre.clone(), f.suffix.clone()));
                    }
                    // Also project category-prefixed frames (`[color, wise, ...]` <-> `[food, wise, ...]`)
                    for &(other_r, other_cat) in &cat_cores {
                        if other_r != r && f.prefix.first() == Some(&cat_tok) {
                            let mut cat_swapped = swapped_pre.clone();
                            cat_swapped[0] = other_cat;
                            raw_derived.insert((
                                other_r,
                                f.act,
                                true,
                                cat_swapped,
                                f.suffix.clone(),
                            ));
                        }
                    }
                }
            }
        }

        // 11. Personal, Location & Occupation Completions (rel=01, 04, 07, 08)
        //     `[01] people call me` + `[02] you can call me` + `[07] what do people call me` -> `what do you call me`
        for f1 in slots().filter(|f| f.act == Act::Assert && f.prefix.len() == 3) {
            for f2 in slots().filter(|f| {
                f.relation == f1.relation
                    && f.act == Act::Assert
                    && f.prefix.len() == 4
                    && f.prefix[2..4] == f1.prefix[1..3]
            }) {
                let tok_people = f1.prefix[0];
                let tok_you = f2.prefix[0];
                for q in table
                    .frames
                    .iter()
                    .filter(|f| !f.slot && f.relation == f1.relation)
                {
                    if q.prefix.contains(&tok_people) {
                        let swapped: Vec<u32> = q
                            .prefix
                            .iter()
                            .map(|&w| if w == tok_people { tok_you } else { w })
                            .collect();
                        raw_derived.insert((q.relation, Act::Query, false, swapped, Vec::new()));
                    }
                }
                if let (Some(([tok_i, tok_am], tok_im)), Some((tok_called, tok_named))) =
                    (assert_contraction, part_pair)
                {
                    for lead in [vec![tok_i, tok_am], vec![tok_im]] {
                        for pt in [tok_called, tok_named] {
                            let mut p = lead.clone();
                            p.push(pt);
                            raw_derived.insert((f1.relation, Act::Assert, true, p, Vec::new()));
                        }
                    }
                    for q in table
                        .frames
                        .iter()
                        .filter(|f| !f.slot && f.prefix.len() >= 3)
                    {
                        if let Some((det_my, _)) = det_pair {
                            if q.prefix[2] == det_my {
                                let tok_who = q.prefix[0];
                                raw_derived.insert((
                                    f1.relation,
                                    Act::Query,
                                    false,
                                    vec![tok_who, tok_am, tok_i],
                                    Vec::new(),
                                ));
                            }
                        }
                    }
                }
            }
        }
        if let Some(([tok_i, tok_am], tok_im)) = assert_contraction {
            for f in slots().filter(|f| f.act == Act::Assert) {
                if f.prefix.len() >= 3 && f.prefix[0] == tok_i && f.prefix[1] == tok_am {
                    let mut contracted = vec![tok_im];
                    contracted.extend_from_slice(&f.prefix[2..]);
                    raw_derived.insert((
                        f.relation,
                        Act::Assert,
                        true,
                        contracted,
                        f.suffix.clone(),
                    ));
                }
            }
        }
        // `[52] i work as a {v}` + `[59] what do i work as` -> `i work as {v}`
        for q in table
            .frames
            .iter()
            .filter(|f| !f.slot && f.prefix.len() >= 5)
        {
            let tail2 = &q.prefix[q.prefix.len() - 2..];
            for f in slots().filter(|f| {
                f.relation == q.relation
                    && f.act == Act::Assert
                    && f.suffix.is_empty()
                    && f.prefix.len() == 4
                    && f.prefix[1..3] == tail2[..]
            }) {
                raw_derived.insert((
                    f.relation,
                    Act::Assert,
                    true,
                    f.prefix[..3].to_vec(),
                    Vec::new(),
                ));
            }
        }
        // Location query `[where, ...]` <-> `[what, city, ...]` (from `[65]` vs `[66]`)
        for q1 in table
            .frames
            .iter()
            .filter(|f| !f.slot && f.prefix.len() == 4)
        {
            for q2 in table.frames.iter().filter(|f| {
                !f.slot
                    && f.relation == q1.relation
                    && f.prefix.len() == 6
                    && f.prefix[2..5] == q1.prefix[1..4]
            }) {
                let tok_where = q1.prefix[0];
                let what_city = [q2.prefix[0], q2.prefix[1]];
                let tok_in = q2.prefix[5];
                // Apply to all `[where, ...]` queries across relations
                for q in table
                    .frames
                    .iter()
                    .filter(|f| !f.slot && f.prefix.first() == Some(&tok_where))
                {
                    let mut wc = what_city.to_vec();
                    wc.extend_from_slice(&q.prefix[1..]);
                    raw_derived.insert((q.relation, Act::Query, false, wc.clone(), Vec::new()));
                    if !wc.ends_with(&[tok_in]) && q.prefix.len() == 5 {
                        let mut wc_in = wc;
                        wc_in.push(tok_in);
                        raw_derived.insert((q.relation, Act::Query, false, wc_in, Vec::new()));
                    }
                }
            }
        }
        // Location `[61] my home is in` vs `[27] my hometown is` + `[67] where is my home`:
        for q in table
            .frames
            .iter()
            .filter(|f| !f.slot && f.prefix.len() == 4)
        {
            if let Some((det_my, _)) = det_pair {
                if q.prefix[2] == det_my {
                    // `[where, is, my, home]` -> apply `[where, is, my]` to `[hometown]` (`rel=04`)
                    let where_is_my = &q.prefix[..3];
                    for f in slots().filter(|f| {
                        f.act == Act::Assert
                            && f.prefix.len() == 3
                            && f.prefix[0] == det_my
                            && table.frames.iter().any(|g| {
                                !g.slot && g.relation == f.relation && g.prefix[0] == q.prefix[0]
                            })
                    }) {
                        let mut pq = where_is_my.to_vec();
                        pq.push(f.prefix[1]);
                        raw_derived.insert((f.relation, Act::Query, false, pq, Vec::new()));
                    }
                }
            }
        }

        // 12. Suffix (`now`, `instead`), Pre-Slot (`really`), and `just` Infix Closure on Slot Frames
        let base_and_derived_slots: Vec<(u32, Act, Vec<u32>, Vec<u32>)> = table
            .frames
            .iter()
            .filter(|f| f.slot)
            .map(|f| (f.relation, f.act, f.prefix.clone(), f.suffix.clone()))
            .chain(
                raw_derived
                    .iter()
                    .filter(|(_, _, slot, _, _)| *slot)
                    .map(|(r, a, _, p, s)| (*r, *a, p.clone(), s.clone())),
            )
            .collect();
        for (r, _act, pre, suf) in &base_and_derived_slots {
            for m_suf in &suffix_markers {
                if !suf.ends_with(m_suf) {
                    let mut new_suf = suf.clone();
                    new_suf.extend(m_suf);
                    raw_derived.insert((*r, Act::Update, true, pre.clone(), new_suf));
                }
            }
            for m_slot in &preslot_markers {
                if !pre.ends_with(m_slot) {
                    let mut new_pre = pre.clone();
                    new_pre.extend(m_slot);
                    raw_derived.insert((*r, Act::Update, true, new_pre, suf.clone()));
                }
            }
            if let Some((tok_i, tok_just)) = verb_update_infix {
                if pre.len() >= 2
                    && pre[0] == tok_i
                    && pre[1] != tok_just
                    && assert_contraction.map_or(true, |([_, am], _)| pre[1] != am)
                {
                    let mut new_pre = vec![tok_i, tok_just];
                    new_pre.extend_from_slice(&pre[1..]);
                    raw_derived.insert((*r, Act::Update, true, new_pre, suf.clone()));
                }
            }
        }

        // Filter out any frame already in `table.frames` or lacking a canonical base_frame,
        // and filter out any exact `(slot, prefix, suffix)` duplicate across different relations.
        let mut pattern_relations: BTreeMap<(bool, Vec<u32>, Vec<u32>), BTreeSet<u32>> =
            BTreeMap::new();
        for f in &table.frames {
            pattern_relations
                .entry((f.slot, f.prefix.clone(), f.suffix.clone()))
                .or_default()
                .insert(f.relation);
        }
        for (r, _, slot, pre, suf) in &raw_derived {
            pattern_relations
                .entry((*slot, pre.clone(), suf.clone()))
                .or_default()
                .insert(*r);
        }

        let mut derived_frames = Vec::new();
        let mut seen: BTreeSet<(u32, bool, Vec<u32>, Vec<u32>)> = BTreeSet::new();
        for f in &table.frames {
            seen.insert((f.relation, f.slot, f.prefix.clone(), f.suffix.clone()));
        }
        for (relation, act, slot, prefix, suffix) in raw_derived {
            if seen.contains(&(relation, slot, prefix.clone(), suffix.clone())) {
                continue;
            }
            if pattern_relations
                .get(&(slot, prefix.clone(), suffix.clone()))
                .is_some_and(|rels| rels.len() > 1)
            {
                continue;
            }
            let Some(base_idx) = table
                .frames
                .iter()
                .position(|f| f.relation == relation && f.act == act && f.slot == slot)
            else {
                continue;
            };
            seen.insert((relation, slot, prefix.clone(), suffix.clone()));
            derived_frames.push(DerivedFrame {
                base_frame: len_u32(base_idx)?,
                relation,
                act,
                slot,
                prefix,
                suffix,
            });
        }

        let markers: Vec<Marker> = marker_map
            .into_iter()
            .map(|(words, update)| Marker { words, update })
            .collect();

        Ok(Self {
            schema: CORE_COFACTOR_TABLE_SCHEMA.into(),
            frame_table_sha256: table.identity()?.artifact_sha256,
            max_markers: MAX_MARKERS,
            markers,
            derived_frames,
        })
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| invalid(format!("core-cofactor table JSON: {e}")))
    }

    pub fn sha256(&self) -> Result<String> {
        Ok(sha256_bytes(&self.canonical_bytes()?))
    }

    pub fn device_markers(&self) -> Result<DeviceMarkers> {
        device_markers_from_slice(&self.markers)
    }

    /// Build the combined `DeviceFrames` (`table.frames` followed by
    /// `self.derived_frames`) and the `frame_base` index map back to `0..table.frames.len()`.
    pub fn device_frames(&self, table: &FrameTable) -> Result<GateFrameSet> {
        let mut frames = table.device_frames()?;
        let (mut frame_act, mut frame_expert_update) = gate_frames(table);
        let mut frame_base: Vec<u32> = (0..len_u32(table.frames.len())?).collect();
        let sig_words = frames.sig_words as usize;
        let update_prime = table.act_primes[1];

        for df in &self.derived_frames {
            let mut sig = vec![0u64; sig_words];
            for &id in df.prefix.iter().chain(&df.suffix) {
                if id == 0 || id as usize > table.lexicon.len() {
                    return Err(invalid("derived frame word lies outside the lexicon"));
                }
                let k = (id - 1) as usize;
                sig[k / 64] |= 1u64 << (k % 64);
            }
            frames.signature.extend(sig);
            frames.offset.push(len_u32(frames.tokens.len())?);
            frames.tokens.extend(df.prefix.iter().chain(&df.suffix));
            frames.prefix.push(len_u32(df.prefix.len())?);
            frames.suffix.push(len_u32(df.suffix.len())?);
            frames.slot.push(u32::from(df.slot));
            let act_code = match df.act {
                Act::Assert => ACT_ASSERT,
                Act::Update => ACT_UPDATE,
                Act::Query => ACT_QUERY,
            };
            let rel_prime = table.relation_primes[df.relation as usize - 1];
            let act_prime = table.act_primes[act_code as usize];
            frames.expert.push(rel_prime * act_prime);
            frame_act.push(act_code);
            frame_expert_update.push(if df.slot { rel_prime * update_prime } else { 0 });
            frame_base.push(df.base_frame);
        }
        Ok(GateFrameSet {
            frames,
            frame_act,
            frame_expert_update,
            frame_base,
        })
    }
}

fn len_u32(len: usize) -> Result<u32> {
    u32::try_from(len).map_err(|_| invalid("device array too large"))
}

/// Markers in the kernel's flat layout.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeviceMarkers {
    pub tokens: Vec<u32>,
    pub offset: Vec<u32>,
    pub length: Vec<u32>,
    pub update: Vec<u32>,
}

/// Act codes the gated kernel reports.
pub const ACT_ASSERT: u32 = 0;
pub const ACT_UPDATE: u32 = 1;
pub const ACT_QUERY: u32 = 2;

/// One turn's stored decision from the gated kernel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatedDecision {
    pub status: u32,
    pub frame: u32,
    pub act: u32,
    pub hole_start: u32,
    pub hole_len: u32,
    pub survivors: u32,
}

/// The action a stored gated decision stands for. The act comes from the
/// kernel (a marker can turn an assert frame into a correction); the
/// relation comes from the base frame.
pub fn decode_gated(
    table: &FrameTable,
    turn: &[Word],
    decision: GatedDecision,
) -> Result<CompiledAction> {
    use crate::sieve_frame_compiler::{STATUS_COMPILED, STATUS_DISAGREE, STATUS_NONE};
    match decision.status {
        STATUS_NONE => {
            return Ok(CompiledAction::Unresolved {
                reason: "no frame survives the markers, sieve and alignment".into(),
            })
        }
        STATUS_DISAGREE => {
            return Ok(CompiledAction::Unresolved {
                reason: "the surviving readings disagree".into(),
            })
        }
        STATUS_COMPILED => {}
        other => return Err(invalid(format!("unknown decision status {other}"))),
    }
    let frame = table
        .frames
        .get(decision.frame as usize)
        .ok_or_else(|| invalid("decision names a frame outside the table"))?;
    let relation = frame.relation;
    match (decision.act, frame.act) {
        (ACT_QUERY, Act::Query) => {
            if decision.hole_start != 0 || decision.hole_len != 0 {
                return Err(invalid(
                    "a compiled query decision has a non-empty value hole",
                ));
            }
            return Ok(CompiledAction::QueryCurrent { relation });
        }
        (ACT_ASSERT, Act::Assert) | (ACT_UPDATE, Act::Assert | Act::Update) => {}
        _ => return Err(invalid("decision act does not fit its frame")),
    }
    let first = decision.hole_start as usize;
    let last = (first + decision.hole_len as usize)
        .checked_sub(1)
        .filter(|_| decision.hole_len > 0)
        .ok_or_else(|| invalid("a compiled slot decision has an empty value"))?;
    let (Some(head), Some(tail)) = (turn.get(first), turn.get(last)) else {
        return Err(invalid("decision value lies outside the turn"));
    };
    let span = SourceSpan {
        start: head.start,
        end: tail.end,
    };
    Ok(if decision.act == ACT_UPDATE {
        CompiledAction::Correct { relation, span }
    } else {
        CompiledAction::Assert { relation, span }
    })
}

/// Per-frame act codes and correction experts (`p_relation · q_update`),
/// which the gated kernel reads beside the sieve's frame layout.
pub fn gate_frames(table: &FrameTable) -> (Vec<u32>, Vec<u64>) {
    let update_prime = table.act_primes[1];
    table
        .frames
        .iter()
        .map(|f| {
            let act = match f.act {
                Act::Assert => ACT_ASSERT,
                Act::Update => ACT_UPDATE,
                Act::Query => ACT_QUERY,
            };
            let corrected = if f.slot {
                table.relation_primes[f.relation as usize - 1] * update_prime
            } else {
                0
            };
            (act, corrected)
        })
        .unzip()
}

#[cfg(feature = "sieve-cuda")]
pub mod device {
    //! The gated kernel's launcher.

    use cudarc::driver::{CudaContext, CudaFunction, CudaStream, LaunchConfig, Ptx, PushKernelArg};
    use std::sync::Arc;
    use std::time::Instant;

    use super::{DeviceMarkers, GatedDecision};
    use crate::sieve_frame_compiler::device::PTX;
    use crate::sieve_frame_compiler::{DeviceFrames, DeviceTurns};
    use crate::{invalid, Result};

    fn cuda(error: impl std::fmt::Display) -> crate::TrainingError {
        invalid(format!("CUDA: {error}"))
    }

    fn len_u32(len: usize) -> Result<u32> {
        u32::try_from(len).map_err(|_| invalid("device array too large"))
    }

    /// A device allocation cannot be empty.
    fn nonempty<T: Copy + Default>(values: &[T]) -> Vec<T> {
        if values.is_empty() {
            vec![T::default()]
        } else {
            values.to_vec()
        }
    }

    pub struct GateDevice {
        stream: Arc<CudaStream>,
        kernel: CudaFunction,
    }

    /// Everything the gated launch reads besides the turns.
    pub struct GateTables<'a> {
        pub frames: &'a DeviceFrames,
        pub frame_act: &'a [u32],
        pub frame_expert_update: &'a [u64],
        pub frame_base: Option<&'a [u32]>,
        pub markers: &'a DeviceMarkers,
        pub max_value_words: u32,
        pub max_markers: u32,
    }

    impl GateDevice {
        pub fn new(ordinal: usize) -> Result<Self> {
            let context = CudaContext::new(ordinal).map_err(cuda)?;
            let stream = context.default_stream();
            let module = context.load_module(Ptx::from_src(PTX)).map_err(cuda)?;
            let kernel = module
                .load_function("sieve_frame_compile_marked")
                .map_err(cuda)?;
            Ok(Self { stream, kernel })
        }

        /// Compile every turn in one launch. Returns the decisions and the
        /// launch's wall time in microseconds (upload to download).
        #[allow(unsafe_code)]
        pub fn compile(
            &self,
            tables: &GateTables<'_>,
            turns: &DeviceTurns,
        ) -> Result<(Vec<GatedDecision>, u64)> {
            let f = tables.frames;
            let n_turns = len_u32(turns.length.len())?;
            let n_frames = len_u32(f.prefix.len())?;
            let n_markers = len_u32(tables.markers.length.len())?;
            if n_turns == 0
                || n_frames == 0
                || tables.frame_act.len() != f.prefix.len()
                || tables.frame_expert_update.len() != f.prefix.len()
                || tables.frame_base.is_some_and(|b| b.len() != f.prefix.len())
                || f.sig_words > 8
            {
                return Err(invalid(
                    "the gated launch needs turns, frames and matching tables",
                ));
            }
            let s = &self.stream;
            let started = Instant::now();
            let turn_tokens = s.clone_htod(&nonempty(&turns.tokens)).map_err(cuda)?;
            let turn_off = s.clone_htod(&turns.offset).map_err(cuda)?;
            let turn_len = s.clone_htod(&turns.length).map_err(cuda)?;
            let frame_sig = s.clone_htod(&f.signature).map_err(cuda)?;
            let frame_tokens = s.clone_htod(&nonempty(&f.tokens)).map_err(cuda)?;
            let frame_off = s.clone_htod(&f.offset).map_err(cuda)?;
            let frame_prefix = s.clone_htod(&f.prefix).map_err(cuda)?;
            let frame_suffix = s.clone_htod(&f.suffix).map_err(cuda)?;
            let frame_slot = s.clone_htod(&f.slot).map_err(cuda)?;
            let frame_act = s.clone_htod(tables.frame_act).map_err(cuda)?;
            let frame_expert = s.clone_htod(&f.expert).map_err(cuda)?;
            let frame_expert_update = s.clone_htod(tables.frame_expert_update).map_err(cuda)?;
            let m = tables.markers;
            let marker_tokens = s.clone_htod(&nonempty(&m.tokens)).map_err(cuda)?;
            let marker_off = s.clone_htod(&nonempty(&m.offset)).map_err(cuda)?;
            let marker_len = s.clone_htod(&nonempty(&m.length)).map_err(cuda)?;
            let marker_update = s.clone_htod(&nonempty(&m.update)).map_err(cuda)?;
            let n = n_turns as usize;
            let mut status = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut frame = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut act = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut hole_start = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut hole_len = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut survivors = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut launch = s.launch_builder(&self.kernel);
            launch
                .arg(&turn_tokens)
                .arg(&turn_off)
                .arg(&turn_len)
                .arg(&n_turns)
                .arg(&frame_sig)
                .arg(&frame_tokens)
                .arg(&frame_off)
                .arg(&frame_prefix)
                .arg(&frame_suffix)
                .arg(&frame_slot)
                .arg(&frame_act)
                .arg(&frame_expert)
                .arg(&frame_expert_update)
                .arg(&n_frames)
                .arg(&f.sig_words)
                .arg(&tables.max_value_words)
                .arg(&marker_tokens)
                .arg(&marker_off)
                .arg(&marker_len)
                .arg(&marker_update)
                .arg(&n_markers)
                .arg(&tables.max_markers)
                .arg(&mut status)
                .arg(&mut frame)
                .arg(&mut act)
                .arg(&mut hole_start)
                .arg(&mut hole_len)
                .arg(&mut survivors);
            let threads = n_turns.clamp(1, 128);
            let config = LaunchConfig {
                grid_dim: (n_turns.div_ceil(threads), 1, 1),
                block_dim: (threads, 1, 1),
                shared_mem_bytes: 0,
            };
            // SAFETY: the argument list is exactly sieve_frame_compile_marked's
            // ABI; every input is a device copy of a checked host array (marker
            // arrays padded to one element when empty, with n_markers = 0), and
            // every output holds one u32 per turn, the kernel's only writes.
            unsafe { launch.launch(config) }.map_err(cuda)?;
            let status = s.clone_dtoh(&status).map_err(cuda)?;
            let frame = s.clone_dtoh(&frame).map_err(cuda)?;
            let act = s.clone_dtoh(&act).map_err(cuda)?;
            let hole_start = s.clone_dtoh(&hole_start).map_err(cuda)?;
            let hole_len = s.clone_dtoh(&hole_len).map_err(cuda)?;
            let survivors = s.clone_dtoh(&survivors).map_err(cuda)?;
            let micros = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX);
            let decisions = (0..n)
                .map(|t| {
                    let raw_f = frame[t];
                    let mapped_f = match tables.frame_base {
                        Some(base) if survivors[t] > 0 => {
                            base.get(raw_f as usize).copied().unwrap_or(raw_f)
                        }
                        _ => raw_f,
                    };
                    GatedDecision {
                        status: status[t],
                        frame: mapped_f,
                        act: act[t],
                        hole_start: hole_start[t],
                        hole_len: hole_len[t],
                        survivors: survivors[t],
                    }
                })
                .collect();
            Ok((decisions, micros))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sieve_frame_compiler::words;

    fn texts(table: &FrameTable, marker: &Marker) -> String {
        marker
            .words
            .iter()
            .map(|&id| table.lexicon[id as usize - 1].as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn markers_are_cofactors_of_training_frames() -> Result<()> {
        let table = FrameTable::world_v2()?;
        let markers = MarkerTable::derive(&table)?;
        assert!(
            !markers.markers.is_empty(),
            "the training frames imply no marker"
        );
        for marker in &markers.markers {
            assert!(!marker.words.is_empty());
            // Some training frame is exactly this marker followed by another.
            let witnessed = table.frames.iter().filter(|f| f.slot).any(|f| {
                table.frames.iter().filter(|g| g.slot).any(|g| {
                    g.relation == f.relation
                        && g.suffix == f.suffix
                        && f.prefix.len() == marker.words.len() + g.prefix.len()
                        && f.prefix[..marker.words.len()] == marker.words[..]
                        && f.prefix[marker.words.len()..] == g.prefix[..]
                })
            });
            assert!(witnessed, "{}", texts(&table, marker));
        }
        let listed: Vec<String> = markers
            .markers
            .iter()
            .map(|m| {
                format!(
                    "{}{}",
                    texts(&table, m),
                    if m.update { " [update]" } else { "" }
                )
            })
            .collect();
        // The registered marker table, found by the rule above from the training
        // phrasings alone (no development phrasing or panel row was read).
        assert_eq!(
            listed,
            [
                "actually [update]",
                "people",
                "please note that",
                "remember",
                "sorry [update]",
                "sorry i meant [update]"
            ]
        );
        assert_eq!(markers.sha256()?, WORLD_V2_MARKER_TABLE_SHA256);
        Ok(())
    }

    #[test]
    fn core_cofactor_table_is_derived_from_training_frames() -> Result<()> {
        let table = FrameTable::world_v2()?;
        let cc = CoreCofactorTable::derive(&table)?;
        let listed: Vec<String> = cc
            .markers
            .iter()
            .map(|m| {
                format!(
                    "{}{}",
                    texts(&table, m),
                    if m.update { " [update]" } else { "" }
                )
            })
            .collect();
        assert_eq!(
            listed,
            [
                "actually [update]",
                "i changed my mind [update]",
                "i meant [update]",
                "instead [update]",
                "note that",
                "now [update]",
                "people",
                "please",
                "please note that",
                "remember",
                "sorry [update]",
                "sorry i meant [update]"
            ]
        );
        assert_eq!(cc.sha256()?, WORLD_V2_CORE_COFACTOR_TABLE_SHA256);
        let gf = cc.device_frames(&table)?;
        assert_eq!(
            gf.frames.prefix.len(),
            table.frames.len() + cc.derived_frames.len()
        );
        assert_eq!(gf.frame_act.len(), gf.frames.prefix.len());
        assert_eq!(gf.frame_expert_update.len(), gf.frames.prefix.len());
        assert_eq!(gf.frame_base.len(), gf.frames.prefix.len());
        for &base in &gf.frame_base {
            assert!((base as usize) < table.frames.len());
        }
        Ok(())
    }

    #[test]
    fn gate_frames_and_device_markers_match_table_layout() -> Result<()> {
        let table = FrameTable::world_v2()?;
        let (acts, updates) = gate_frames(&table);
        assert_eq!(acts.len(), table.frames.len());
        assert_eq!(updates.len(), table.frames.len());
        for (i, f) in table.frames.iter().enumerate() {
            if f.slot {
                assert_eq!(
                    updates[i],
                    table.relation_primes[f.relation as usize - 1] * table.act_primes[1]
                );
            } else {
                assert_eq!(acts[i], ACT_QUERY);
                assert_eq!(updates[i], 0);
            }
        }
        let markers = MarkerTable::derive(&table)?;
        let dm = markers.device_markers()?;
        assert_eq!(dm.length.len(), markers.markers.len());
        assert_eq!(dm.offset.len(), markers.markers.len());
        assert_eq!(dm.update.len(), markers.markers.len());
        let total_words: usize = markers.markers.iter().map(|m| m.words.len()).sum();
        assert_eq!(dm.tokens.len(), total_words);
        Ok(())
    }

    #[test]
    fn decode_gated_takes_the_act_from_the_kernel() -> Result<()> {
        let table = FrameTable::world_v2()?;
        let assert_frame = table
            .frames
            .iter()
            .position(|f| f.act == Act::Assert && f.relation == 4)
            .expect("frame") as u32;
        let query_frame = table
            .frames
            .iter()
            .position(|f| f.act == Act::Query && f.relation == 4)
            .expect("query frame") as u32;
        let turn = words("Actually, my hometown is Rivertown.");
        let d = |act, hole_start, hole_len| GatedDecision {
            status: 1,
            frame: assert_frame,
            act,
            hole_start,
            hole_len,
            survivors: 1,
        };
        let span = SourceSpan {
            start: turn[4].start,
            end: turn[4].end,
        };
        assert_eq!(
            decode_gated(&table, &turn, d(ACT_UPDATE, 4, 1))?,
            CompiledAction::Correct { relation: 4, span }
        );
        assert_eq!(
            decode_gated(&table, &turn, d(ACT_ASSERT, 4, 1))?,
            CompiledAction::Assert { relation: 4, span }
        );
        assert!(decode_gated(&table, &turn, d(ACT_QUERY, 0, 0)).is_err());
        assert!(decode_gated(&table, &turn, d(ACT_UPDATE, 4, 0)).is_err());
        assert_eq!(
            decode_gated(
                &table,
                &turn,
                GatedDecision {
                    status: 1,
                    frame: query_frame,
                    act: ACT_QUERY,
                    hole_start: 0,
                    hole_len: 0,
                    survivors: 1,
                }
            )?,
            CompiledAction::QueryCurrent { relation: 4 }
        );
        assert!(decode_gated(
            &table,
            &turn,
            GatedDecision {
                status: 1,
                frame: query_frame,
                act: ACT_QUERY,
                hole_start: 0,
                hole_len: 1,
                survivors: 1,
            }
        )
        .is_err());
        Ok(())
    }
}
