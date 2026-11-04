//! The served point key in the integer artifact (claude-222, `point-key-integer`): TRAIN's
//! opening points under the integer trunk, at the writer query's Q12, with the first token of
//! each opening's reply, as a `uor-r4.lut-stack/8` container stores them in
//! `memory.point.points` and `memory.point.targets`.
//!
//! Where the turn memory's exact key misses a reply's opening, the session names the opening's
//! first token by [`ServedPointKey::name`]. The original rule counts every target at the least L1
//! distance and ties to the smaller id. A voted table ranks rows by exact `(L1 distance, row)`,
//! counts the targets in the first `vote_k`, and again ties to the smaller id. Both use the integer
//! trunk's own points (the item's condition (b)); the default remains byte-compatible with the
//! original artifacts.

use super::format::{Container, StackPointKey, POINT_KEY_EXP};
use super::memory::MEMORY_BOS_ID;
use super::StackError;

/// A stack's point table: `rows × width` Q12 points, row-major, and each row's target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PointKeyImage {
    pub width: usize,
    pub points: Vec<i16>,
    pub targets: Vec<u32>,
    /// Rows voted over after exact `(L1 distance, row)` ordering.
    pub vote_k: usize,
}

impl PointKeyImage {
    /// The rows the table holds.
    pub fn rows(&self) -> usize {
        self.targets.len()
    }

    /// The header record naming this table.
    pub fn record(&self) -> StackPointKey {
        StackPointKey {
            rows: self.rows(),
            width: self.width,
            exp: POINT_KEY_EXP,
            vote_k: self.vote_k,
        }
    }

    /// Refuse a table the session cannot serve: no rows, points that are not `rows × width`, or a
    /// target that is BOS or past `vocab`.
    pub fn check(&self, vocab: usize) -> Result<(), StackError> {
        if self.targets.is_empty() || self.width == 0 {
            return Err(StackError::Numerics("the point table has no rows".to_owned()));
        }
        if self.vote_k == 0 || self.vote_k > self.rows() {
            return Err(StackError::Numerics(
                "the point table's vote width is outside its row count".to_owned(),
            ));
        }
        if self.rows().checked_mul(self.width) != Some(self.points.len()) {
            return Err(StackError::Numerics("the point table's points are not rows × width".to_owned()));
        }
        if let Some(&bad) = self.targets.iter().find(|&&t| t == MEMORY_BOS_ID || t as usize >= vocab) {
            return Err(StackError::Numerics(format!("the point table names {bad}, BOS or past the vocabulary")));
        }
        Ok(())
    }
}

/// The point table of a parsed container, every section checked against its record, or `None`
/// for a stack without one.
pub(crate) fn point_image(artifact: &Container<'_>) -> Result<Option<PointKeyImage>, StackError> {
    let Some(record) = &artifact.point else {
        return Ok(None);
    };
    let image = PointKeyImage {
        width: record.width,
        points: artifact.table_i16("memory.point.points")?,
        targets: artifact.table_u32("memory.point.targets")?,
        vote_k: record.vote_k,
    };
    if image.rows() != record.rows {
        return Err(StackError::Numerics(format!(
            "memory.point.targets holds {} rows, not the record's {}",
            image.rows(),
            record.rows
        )));
    }
    image.check(artifact.shape.vocab)?;
    Ok(Some(image))
}

/// The point table as a session serves it.
pub(crate) struct ServedPointKey {
    width: usize,
    points: Vec<i16>,
    targets: Vec<u32>,
    vote_k: usize,
}

impl ServedPointKey {
    /// The served table of a checked image.
    pub(crate) fn new(image: PointKeyImage) -> Self {
        Self {
            width: image.width,
            points: image.points,
            targets: image.targets,
            vote_k: image.vote_k,
        }
    }

    /// The number of exactly ranked rows voted over at a point-key miss.
    pub(crate) fn vote_k(&self) -> usize {
        self.vote_k
    }

    /// The rows the table holds: the capacity [`Self::name`]'s scratch needs.
    pub(crate) fn rows(&self) -> usize {
        self.targets.len()
    }

    /// The target `query` names, and its least distance. With `vote_k = 1`, preserve the original
    /// commonest-target rule among every row tied at the least distance. With `vote_k > 1`, rank
    /// rows exactly by `(L1 distance, row index)` and choose the commonest target in the first
    /// `vote_k`, ties to the smaller id. `ranked` is the caller's scratch, of at least
    /// [`Self::rows`] capacity, so the search allocates nothing.
    #[inline(never)]
    pub(crate) fn name(
        &self,
        query: &[i32],
        ranked: &mut Vec<(i64, usize, u32)>,
    ) -> Result<(u32, i64), StackError> {
        if query.len() != self.width {
            return Err(StackError::Numerics("the point key's query is not the table's width".to_owned()));
        }
        ranked.clear();
        let (least, considered) = if self.vote_k == 1 {
            let mut least = i64::MAX;
            for (index, (&target, row)) in self
                .targets
                .iter()
                .zip(self.points.chunks_exact(self.width))
                .enumerate()
            {
                let sum = l1_distance(query, row);
                if sum < least {
                    least = sum;
                    ranked.clear();
                }
                if sum == least {
                    ranked.push((sum, index, target));
                }
            }
            ranked.sort_unstable_by_key(|&(_, _, target)| target);
            (least, &ranked[..])
        } else {
            let mut least = i64::MAX;
            for (index, (&target, row)) in self
                .targets
                .iter()
                .zip(self.points.chunks_exact(self.width))
                .enumerate()
            {
                let distance = l1_distance(query, row);
                least = least.min(distance);
                ranked.push((distance, index, target));
            }
            // Select before sorting so a small vote width stays O(rows + vote_k log vote_k).
            let _ = ranked.select_nth_unstable(self.vote_k - 1);
            ranked.truncate(self.vote_k);
            ranked.sort_unstable();
            ranked.sort_unstable_by_key(|&(_, _, target)| target);
            (least, &ranked[..])
        };
        // The commonest target: the longest run of sorted ids, a longer run replacing the held
        // one only when strictly longer, so ties go to the smaller id.
        let (mut named, mut best) = (u32::MAX, 0usize);
        let mut at = 0;
        while at < considered.len() {
            let id = considered[at].2;
            let run = considered[at..]
                .iter()
                .take_while(|(_, _, target)| *target == id)
                .count();
            if run > best {
                (named, best) = (id, run);
            }
            at += run;
        }
        if best == 0 || named == MEMORY_BOS_ID {
            return Err(StackError::Numerics("the point key named no token, or BOS".to_owned()));
        }
        Ok((named, least))
    }

    /// The least L1 distance at Q12 from `query` to a row naming `target` (claude-224), or `None`
    /// when no row names it or the query is not the table's width: the input to parity's rounding
    /// bound. It is off the step's path.
    pub(crate) fn least_distance_to(&self, query: &[i32], target: u32) -> Option<i64> {
        if query.len() != self.width {
            return None;
        }
        self.points
            .chunks_exact(self.width)
            .zip(&self.targets)
            .filter(|&(_, &t)| t == target)
            .map(|(row, _)| query.iter().zip(row).map(|(&x, &p)| i64::from(x.abs_diff(i32::from(p)))).sum())
            .min()
    }

    /// A second, plain implementation of [`Self::name`]'s rule, for the run's instrument
    /// (claude-228, the item's done-when 2): every row's L1 distance at Q12 into a list ordered
    /// exactly by `(distance, row)`; `vote_k = 1` counts every target at the least distance, while
    /// a wider vote counts the first `vote_k` rows. Both choose the largest count with ties to the
    /// smaller id. It shares no code with `name` and is off the step's path. `None` for a query of
    /// another width.
    pub(crate) fn reference_name(&self, query: &[i32]) -> Option<(u32, i64)> {
        if query.len() != self.width {
            return None;
        }
        let mut distances: Vec<(i64, usize, u32)> = self
            .points
            .chunks_exact(self.width)
            .zip(&self.targets)
            .enumerate()
            .map(|(row, (point, &target))| {
                let distance = query
                    .iter()
                    .zip(point)
                    .map(|(&x, &p)| (i64::from(x) - i64::from(p)).abs())
                    .sum();
                (distance, row, target)
            })
            .collect();
        distances.sort_unstable();
        let least = distances.first()?.0;
        let considered: &[(i64, usize, u32)] = if self.vote_k == 1 {
            &distances[..distances.partition_point(|candidate| candidate.0 == least)]
        } else {
            &distances[..self.vote_k]
        };
        let mut counts = std::collections::BTreeMap::<u32, usize>::new();
        for &(_, _, target) in considered {
            *counts.entry(target).or_default() += 1;
        }
        // The map iterates ids upward, so a later id replaces the held one only on a larger count.
        let mut named: Option<(u32, usize)> = None;
        for (&target, &count) in &counts {
            if named.is_none_or(|(_, best)| count > best) {
                named = Some((target, count));
            }
        }
        named.map(|(target, _)| (target, least))
    }
}

/// Exact Q12 L1 distance. `black_box` keeps the sum scalar, as
/// `stack_whole_table_nearest`'s: a vectorized sum leaves through a float register move, which the
/// serving audit refuses.
#[inline]
fn l1_distance(query: &[i32], row: &[i16]) -> i64 {
    let mut sum = 0i64;
    for (&x, &p) in query.iter().zip(row) {
        sum += i64::from(core::hint::black_box(x).abs_diff(i32::from(p)));
    }
    sum
}

/// The run's instrument at a served opening (claude-228, the item's done-when 2): the name and least
/// distance the point key served must equal the reference scan's of the same query against the
/// same table, or the decode stops.
pub(crate) fn check_served_point(
    model: &super::IntegerStackModel,
    query: &[i32],
    served: (u32, i64),
) -> Result<(), StackError> {
    let reference = model.point_key_reference(query);
    if reference != Some(served) {
        return Err(StackError::Numerics(format!(
            "the point key served {served:?}, where the reference scan names {reference:?}"
        )));
    }
    Ok(())
}

/// The run's instrument at export (claude-228, the item's done-when 2): the table `stack` reads back
/// as, from its own bytes, must equal the one built, byte for byte.
pub fn check_point_key_read_back(stack: &[u8], built: &PointKeyImage) -> Result<(), StackError> {
    let read = super::memory::read_memory_uorlut01(stack)?.and_then(|memory| memory.point);
    if read.as_ref() != Some(built) {
        return Err(StackError::Numerics("the point table does not read back as built".to_owned()));
    }
    Ok(())
}

/// Why a panel reply ended (claude-224).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplyEnd {
    /// The decode named BOS, which ends a greedy reply unkept, as on the float side.
    Bos,
    /// The reply holds its `reply_tokens`.
    Length,
    /// Prompt and reply fill the artifact's context. The float side slides its window there; the
    /// integer session cannot, so both integer arms are capped the same way.
    Context,
}

/// Whether a greedy reply of `reply` tokens after a `prompt`-token prompt ends before keeping
/// `token`: at BOS, or keeps it and then ends, at its length or a full context.
/// `None` keeps `token` and decodes on.
pub fn reply_stop(token: u32, reply: usize, prompt: usize, reply_tokens: usize, context: usize) -> Option<ReplyEnd> {
    if token == MEMORY_BOS_ID {
        Some(ReplyEnd::Bos)
    } else if reply + 1 >= reply_tokens {
        Some(ReplyEnd::Length)
    } else if prompt + reply + 1 >= context {
        Some(ReplyEnd::Context)
    } else {
        None
    }
}

/// One panel prompt's greedy reply on the integer engine (claude-224).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PanelReply {
    pub reply: Vec<u32>,
    /// The token the served point key named at the opening, and its least distance at Q12, if it
    /// served there.
    pub keyed: Option<(u32, i64)>,
    /// With a float name given, the least distance at Q12 to a row naming it, if the table has one.
    pub float_distance: Option<i64>,
    pub end: ReplyEnd,
}

/// `prompt`'s greedy reply, in a fresh session: the prompt stepped with the reply start marked at
/// its last token, so the turn memory keys and moves as served, then each chosen token fed back,
/// until [`reply_stop`]. A `/8` stack's point key names the opening where the exact key misses.
/// `float_keyed` is the float side's served name for this prompt, whose least distance in this
/// table is recorded.
pub fn panel_reply(
    model: &super::IntegerStackModel,
    prompt: &[u32],
    reply_tokens: usize,
    float_keyed: Option<u32>,
) -> Result<PanelReply, StackError> {
    let context = model.shape().context;
    if prompt.is_empty() || prompt.len() > context || reply_tokens == 0 {
        return Err(StackError::Numerics("a panel prompt must be 1 to context tokens, with a reply of 1 or more".to_owned()));
    }
    let mut session = model.session();
    let mut step = None;
    for (row, &token) in prompt.iter().enumerate() {
        if row + 1 == prompt.len() {
            session.mark_reply_start();
        }
        step = Some(session.step_native(token)?);
    }
    let step = step.ok_or(StackError::MissingData)?;
    let keyed = session.last_point_key();
    // The step's `query_q12` is the buffer the served search read (claude-228).
    if let Some(served) = keyed {
        check_served_point(model, &step.query_q12, served)?;
    }
    let float_distance = float_keyed.and_then(|f| model.point_key_distance(&step.query_q12, f));
    let (mut reply, mut token) = (Vec::new(), step.token);
    let end = loop {
        let stop = reply_stop(token, reply.len(), prompt.len(), reply_tokens, context);
        if stop != Some(ReplyEnd::Bos) {
            reply.push(token);
        }
        if let Some(end) = stop {
            break end;
        }
        token = session.step_native(token)?.token;
    };
    Ok(PanelReply { reply, keyed, float_distance, end })
}

/// The float sheet's window length (`SHEET_CONTEXT`), which [`point_key_windows`] cuts TRAIN at.
pub const POINT_KEY_WINDOW: usize = 192;
/// The `<|bos|>` id the windows split documents at.
pub const POINT_KEY_BOS: u32 = MEMORY_BOS_ID;

/// One TRAIN window holding openings (claude-222): the window's tokens up to its last opening row,
/// and each opening's row in it and its target, the reply's first token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PointKeyWindow {
    pub input: Vec<u32>,
    pub openings: Vec<(usize, u32)>,
}

/// TRAIN's openings in the float trunk's windows (claude-218's `opening_points`).
/// - **The windows** are `build_document_windows`: each `bos`-bounded document's targets cut into
///   runs of at most `context`, each read from a causal prefix of at most `context` tokens that never
///   crosses the document's start.
/// - **An opening** is a scored target (`mask[p] == 1`, not `bos`) whose previous token is a prompt
///   token (`mask[p - 1] == 0`, `opens_reply`), read at row `p - 1` of its window.
///
/// Positions only, so the integer crate needs no float code.
pub fn point_key_windows(
    tokens: &[u32],
    mask: &[u8],
    context: usize,
    bos: u32,
) -> Result<Vec<PointKeyWindow>, StackError> {
    if tokens.len() != mask.len() || context == 0 {
        return Err(StackError::Numerics("the split's mask is not its tokens' length, or the context is 0".to_owned()));
    }
    // A mask byte past 1 is neither prompt nor reply, and would drop or invent openings silently.
    if mask.iter().any(|&m| m > 1) {
        return Err(StackError::Numerics("the split's response mask holds a byte other than 0 or 1".to_owned()));
    }
    let starts: Vec<usize> = (0..tokens.len()).filter(|&p| tokens[p] == bos).collect();
    let mut windows = Vec::new();
    for (doc, &start) in starts.iter().enumerate() {
        let end = starts.get(doc + 1).copied().unwrap_or(tokens.len());
        let mut cursor = start + 1;
        while cursor < end {
            let target_end = (cursor + context).min(end);
            let from = (target_end - 1).saturating_sub(context).max(start);
            let openings: Vec<(usize, u32)> = (cursor..target_end)
                .filter(|&p| tokens[p] != bos && mask[p] == 1 && mask[p - 1] == 0)
                .map(|p| (p - 1 - from, tokens[p]))
                .collect();
            if let Some(&(last, _)) = openings.last() {
                windows.push(PointKeyWindow {
                    input: tokens[from..=from + last].to_vec(),
                    openings,
                });
            }
            cursor = target_end;
        }
    }
    Ok(windows)
}

/// The point table of `model` over `windows` (claude-222). Each window is stepped in a fresh
/// session from its first token, with no reply-start signal, so the turn memory keys and moves
/// nothing. Each opening's row gives the writer's own Q12 query, which is the integer trunk's own
/// point (the item's condition (b)), and its target.
pub fn build_point_key(
    model: &super::IntegerStackModel,
    windows: &[PointKeyWindow],
) -> Result<PointKeyImage, StackError> {
    let width = model.shape().width;
    let (mut points, mut targets) = (Vec::new(), Vec::new());
    for window in windows {
        let mut session = model.session();
        let mut next = window.openings.iter().peekable();
        for (row, &token) in window.input.iter().enumerate() {
            let step = session.step_native(token)?;
            while let Some(&&(at, target)) = next.peek() {
                if at != row {
                    break;
                }
                if step.query_q12.len() != width {
                    return Err(StackError::Numerics("the stack's query is not its width".to_owned()));
                }
                points.extend(step.query_q12.iter().map(|&q| q.clamp(-32768, 32767) as i16));
                targets.push(target);
                next.next();
            }
        }
    }
    let image = PointKeyImage {
        width,
        points,
        targets,
        vote_k: 1,
    };
    image.check(model.shape().vocab)?;
    Ok(image)
}

#[cfg(test)]
mod point_key_tests {
    use super::*;

    fn served(width: usize, points: Vec<i16>, targets: Vec<u32>) -> ServedPointKey {
        served_vote(width, points, targets, 1)
    }

    fn served_vote(
        width: usize,
        points: Vec<i16>,
        targets: Vec<u32>,
        vote_k: usize,
    ) -> ServedPointKey {
        let image = PointKeyImage {
            width,
            points,
            targets,
            vote_k,
        };
        image.check(100).unwrap();
        ServedPointKey::new(image)
    }

    /// claude-222: the least L1 distance names the target, over every row.
    #[test]
    fn the_point_key_names_the_least_l1_rows_target() {
        let key = served(2, vec![10, 10, 0, 0, -5, 3], vec![7, 8, 9]);
        let mut ranked = Vec::with_capacity(key.rows());
        assert_eq!(key.name(&[1, 1], &mut ranked).unwrap(), (8, 2));
        assert_eq!(key.name(&[9, 12], &mut ranked).unwrap(), (7, 3));
        assert_eq!(key.name(&[-6, 3], &mut ranked).unwrap(), (9, 1));
    }

    /// claude-222: among the rows at the least distance the commonest target wins, and between
    /// equally common targets the smaller id.
    #[test]
    fn the_point_key_takes_the_commonest_tied_target_then_the_smaller_id() {
        // Rows 0–3 are all at distance 2 from [0, 0]; 9 twice, 5 once, 4 once.
        let key = served(2, vec![1, 1, 2, 0, -1, -1, 0, 2, 50, 50], vec![9, 5, 9, 4, 3]);
        let mut ranked = Vec::with_capacity(key.rows());
        assert_eq!(key.name(&[0, 0], &mut ranked).unwrap(), (9, 2), "the commonest of the tied");
        let even = served(2, vec![1, 1, 2, 0, -1, -1, 0, 2], vec![9, 5, 5, 9]);
        assert_eq!(even.name(&[0, 0], &mut ranked).unwrap(), (5, 2), "a tie in counts goes to the smaller id");
        // A nearer single row beats a more common farther one.
        let near = served(2, vec![1, 1, 1, 1, 0, 1], vec![6, 6, 7]);
        assert_eq!(near.name(&[0, 0], &mut ranked).unwrap(), (7, 1));
    }

    /// deepseek-106: eight exact `(distance, row)` neighbours can override a wrong nearest row,
    /// and equal vote counts go to the smaller target id.
    #[test]
    fn the_eight_row_vote_orders_by_distance_then_row_and_breaks_target_ties() {
        let key = served_vote(
            1,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
            vec![9, 7, 7, 7, 7, 7, 7, 7, 8],
            8,
        );
        let mut ranked = Vec::with_capacity(key.rows());
        assert_eq!(key.name(&[0], &mut ranked).unwrap(), (7, 1));
        assert_eq!(key.reference_name(&[0]), Some((7, 1)));
        assert_eq!(
            ranked.iter().min().copied(),
            Some((1, 0, 9)),
            "the nearest row remains first under the exact ordering"
        );

        let tied = served_vote(1, vec![1, 1, 2, 2], vec![9, 5, 7, 8], 2);
        assert_eq!(tied.name(&[0], &mut ranked).unwrap(), (5, 1));
        assert_eq!(tied.reference_name(&[0]), Some((5, 1)));

        // Rows 1 and 2 are both at distance 1; the second vote slot goes to row 1.
        let row_tie = served_vote(1, vec![0, 1, 1, 9], vec![9, 3, 1, 4], 2);
        assert_eq!(row_tie.name(&[0], &mut ranked).unwrap(), (3, 0));
        assert_eq!(row_tie.reference_name(&[0]), Some((3, 0)));
    }

    /// claude-224: the float name's least distance is the exact L1 sum to the nearest row naming
    /// it, the least over every such row; `None` for a name no row holds or a query of another
    /// width.
    #[test]
    fn the_least_distance_to_a_name_is_its_nearest_rows_l1_sum() {
        // Rows naming 7 at L1 6 and 13 from [0, 0]; a row naming 8 at 1.
        let key = served(2, vec![2, -4, 6, 7, 1, 0], vec![7, 7, 8]);
        assert_eq!(key.least_distance_to(&[0, 0], 7), Some(6), "the least of the rows naming 7");
        assert_eq!(key.least_distance_to(&[6, 7], 7), Some(0));
        assert_eq!(key.least_distance_to(&[3, 3], 8), Some(5));
        assert_eq!(key.least_distance_to(&[0, 0], 9), None, "no row names 9");
        assert_eq!(key.least_distance_to(&[0], 7), None, "another width");
    }

    /// claude-222: a table holding BOS or an id past the vocabulary is refused, and so is a
    /// query of the wrong width or an empty table.
    #[test]
    fn the_point_key_refuses_bos_out_of_range_ids_and_a_wrong_width() {
        let image = |targets: Vec<u32>| PointKeyImage {
            width: 1,
            points: vec![0; targets.len()],
            targets,
            vote_k: 1,
        };
        assert!(image(vec![MEMORY_BOS_ID, 3]).check(100).is_err(), "BOS");
        assert!(image(vec![3, 100]).check(100).is_err(), "past the vocabulary");
        assert!(image(vec![]).check(100).is_err(), "no rows");
        assert!(PointKeyImage {
            width: 2,
            points: vec![0; 3],
            targets: vec![3, 4],
            vote_k: 1,
        }
        .check(100)
        .is_err(), "rows × width");
        assert!(PointKeyImage {
            width: 1,
            points: vec![0, 1],
            targets: vec![3, 4],
            vote_k: 0,
        }
        .check(100)
        .is_err(), "zero vote width");
        assert!(PointKeyImage {
            width: 1,
            points: vec![0, 1],
            targets: vec![3, 4],
            vote_k: 3,
        }
        .check(100)
        .is_err(), "vote width past the rows");
        let key = served(1, vec![0, 4], vec![3, 4]);
        assert!(key.name(&[0, 0], &mut Vec::new()).is_err(), "a wrong-width query");
    }

    /// claude-228: the reference scan, a second implementation of the rule, names as the search
    /// does. The random tables hold few values and few ids, so ties at the least distance and ties
    /// in counts are common, and the hand cases pin each half of the rule.
    #[test]
    fn the_reference_scan_names_as_the_search_does() {
        let commonest = served(2, vec![1, 1, 2, 0, -1, -1, 0, 2, 50, 50], vec![9, 5, 9, 4, 3]);
        assert_eq!(commonest.reference_name(&[0, 0]), Some((9, 2)), "the commonest of the tied");
        let even = served(2, vec![1, 1, 2, 0, -1, -1, 0, 2], vec![9, 5, 5, 9]);
        assert_eq!(even.reference_name(&[0, 0]), Some((5, 2)), "a tie in counts goes to the smaller id");
        assert_eq!(even.reference_name(&[0]), None, "another width");
        let mut state = 0x2026_1031u64;
        let mut next = |m: u64| {
            state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
            (state >> 33) % m
        };
        let (mut cases, mut count_ties) = (0usize, 0usize);
        for _ in 0..400 {
            let (width, rows) = (1 + next(3) as usize, 1 + next(12) as usize);
            let points: Vec<i16> = (0..rows * width).map(|_| next(5) as i16 - 2).collect();
            let targets: Vec<u32> = (0..rows).map(|_| 1 + next(4) as u32).collect();
            for vote_k in 1..=rows {
                let key = served_vote(width, points.clone(), targets.clone(), vote_k);
                let mut ranked = Vec::with_capacity(key.rows());
                for _ in 0..8 {
                    let query: Vec<i32> = (0..width).map(|_| next(7) as i32 - 3).collect();
                    assert_eq!(key.name(&query, &mut ranked).ok(), key.reference_name(&query), "vote_k={vote_k}, query {query:?}");
                    // After naming, the considered rows are sorted by target id.
                    let considered = if key.vote_k == 1 {
                        let least = ranked.iter().map(|candidate| candidate.0).min().unwrap();
                        let tied = ranked.partition_point(|candidate| candidate.0 == least);
                        ranked[..tied].sort_unstable_by_key(|&(_, _, target)| target);
                        &ranked[..tied]
                    } else {
                        &ranked[..]
                    };
                    let mut runs: Vec<usize> = considered
                        .chunk_by(|left, right| left.2 == right.2)
                        .map(<[(i64, usize, u32)]>::len)
                        .collect();
                    runs.sort_unstable();
                    count_ties += usize::from(
                        runs.len() > 1 && runs[runs.len() - 1] == runs[runs.len() - 2],
                    );
                    cases += 1;
                }
            }
        }
        assert!(count_ties > 100, "only {count_ties} of {cases} cases tie in counts");
    }

    /// claude-228: a BOS name never serves. The search refuses it at the step, even from a table
    /// that skipped the image's check.
    #[test]
    fn a_bos_name_is_refused_at_the_step() {
        let key = ServedPointKey::new(PointKeyImage {
            width: 1,
            points: vec![0, 5],
            targets: vec![MEMORY_BOS_ID, 3],
            vote_k: 1,
        });
        assert!(key.name(&[0], &mut Vec::new()).is_err(), "BOS at the least distance");
        assert_eq!(key.name(&[5], &mut Vec::new()).unwrap(), (3, 0), "another row's name serves");
    }
}
