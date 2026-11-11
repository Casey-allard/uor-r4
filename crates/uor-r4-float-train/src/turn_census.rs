//! Turn census (claude-349): what a read that admits one earlier user turn, and copies from that
//! turn alone, could do on conversations written without memory lines. No model and no GPU.
//!
//! `train-in-space --data-root DIR --turn-census TRAIN PANEL LINED` reads three corpus splits,
//! each a folder with `tokens.u16` and `response_mask.u8`: TRAIN and the panel written
//! `recall=off`, and the panel's conversations written `recall=oracle`. The lined panel's
//! `System: Memory:` lines say which replies recall and the value each must state; nothing else
//! of it is read, and the two panels must hold the same turns once the lines are taken out.
//!
//! A reply's **content tokens** are its tokens other than punctuation and the `K` most frequent
//! tokens of TRAIN (64 is the registered `K`; 0, 16 and 256 are printed beside it). Its **label**
//! is the user turn, before the one being answered, that holds the most of them, if it holds at
//! least one; ties go to the later turn. The census prints how often recall replies have a label
//! and how often the labelled turn holds the value; how often the copy read (latest exact match
//! on the token before, copy key 1) names every value token when it looks only inside the
//! admitted turn; the turns that could compete; and how many turns a reply has to choose from.

use std::collections::{HashMap, HashSet};
use std::ops::Range;

/// Errors are sentences; the binary wraps them.
pub type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
}

/// One turn: who speaks, and the corpus positions of its content (no marker, no end mark).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Turn {
    pub role: Role,
    pub content: Range<usize>,
}

/// The ids that frame a conversation: the document start and end marks, the turn separator and
/// the three role markers, each as the tokenizer encodes it alone.
pub struct Marks {
    pub bos: u32,
    pub eos: u32,
    pub sep: Vec<u32>,
    pub system: Vec<u32>,
    pub user: Vec<u32>,
    pub assistant: Vec<u32>,
}

/// Text to ids and back, as the corpus's tokenizer does it.
pub struct Codec<'a> {
    pub decode: &'a dyn Fn(&[u32]) -> String,
    pub encode: &'a dyn Fn(&str) -> Vec<u32>,
}

/// The documents of a corpus, each as its turns. Refused when a document does not open with the
/// start mark, a turn has no role marker, or the response mask is not 1 on exactly the assistant
/// turns' content and end mark (0 on the start mark, every role marker, every separator and
/// every other turn).
pub fn documents(tokens: &[u32], mask: &[u8], marks: &Marks) -> Result<Vec<Vec<Turn>>> {
    if tokens.len() != mask.len() {
        return Err("the tokens and the response mask differ in length".to_owned());
    }
    let role_at = |i: usize| {
        [
            (Role::System, &marks.system),
            (Role::User, &marks.user),
            (Role::Assistant, &marks.assistant),
        ]
        .into_iter()
        .find(|(_, marker)| i <= tokens.len() && tokens[i..].starts_with(marker))
        .map(|(role, marker)| (role, marker.len()))
    };
    let mut docs = Vec::new();
    let mut start = 0;
    while start < tokens.len() {
        if tokens[start] != marks.bos {
            return Err(format!("no document start mark at token {start}"));
        }
        let end = tokens[start + 1..]
            .iter()
            .position(|&t| t == marks.bos)
            .map_or(tokens.len(), |p| start + 1 + p);
        if mask[start] != 0 {
            return Err(format!(
                "the response mask is set on the start mark at token {start}"
            ));
        }
        let mut turns = Vec::new();
        let mut i = start + 1;
        while i < end {
            let (role, len) = role_at(i).ok_or_else(|| format!("no role marker at token {i}"))?;
            let a = i + len;
            let next = (a..end).find(|&j| {
                tokens[j..end].starts_with(&marks.sep) && role_at(j + marks.sep.len()).is_some()
            });
            let stop = next.unwrap_or(end);
            let mut b = stop;
            if b > a && tokens[b - 1] == marks.eos {
                b -= 1;
            }
            let want = u8::from(role == Role::Assistant);
            let separator = next.map_or(stop..stop, |j| j..j + marks.sep.len());
            let marks_clear = mask[i..a].iter().chain(&mask[separator]).all(|&m| m == 0);
            if !marks_clear || mask[a..stop].iter().any(|&m| m != want) {
                return Err(format!(
                    "the response mask disagrees with the {role:?} turn at token {a}"
                ));
            }
            turns.push(Turn {
                role,
                content: a..b,
            });
            i = next.map_or(end, |j| j + marks.sep.len());
        }
        docs.push(turns);
        start = end;
    }
    Ok(docs)
}

/// For every turn of the plain corpus, the content of the memory line that stands before its
/// twin in the lined corpus (assistant turns only). Refused unless the two corpora hold the same
/// documents and, with the system turns taken out, the same turns token for token; and refused
/// when a system turn of the lined corpus does not stand directly before a reply (so a reply has
/// at most one line, and no line is left over).
pub fn pair_lines(
    plain: &[u32],
    plain_docs: &[Vec<Turn>],
    lined: &[u32],
    lined_docs: &[Vec<Turn>],
) -> Result<Vec<Vec<Option<Range<usize>>>>> {
    if plain_docs.len() != lined_docs.len() {
        return Err(format!(
            "the plain corpus has {} documents and the lined corpus {}",
            plain_docs.len(),
            lined_docs.len()
        ));
    }
    let mut all = Vec::with_capacity(plain_docs.len());
    for (d, (p, l)) in plain_docs.iter().zip(lined_docs).enumerate() {
        let mut lines = Vec::with_capacity(p.len());
        let mut pending = None;
        let mut twins = p.iter();
        for (n, turn) in l.iter().enumerate() {
            if turn.role == Role::System {
                if l.get(n + 1).is_none_or(|next| next.role != Role::Assistant) {
                    return Err(format!(
                        "document {d}: a system line that does not stand directly before a reply"
                    ));
                }
                pending = Some(turn.content.clone());
                continue;
            }
            let twin = twins
                .next()
                .ok_or_else(|| format!("document {d}: more lined turns"))?;
            if twin.role != turn.role || plain[twin.content.clone()] != lined[turn.content.clone()]
            {
                return Err(format!(
                    "document {d}: the two corpora differ beyond their lines"
                ));
            }
            lines.push(if turn.role == Role::Assistant {
                pending.take()
            } else {
                None
            });
        }
        if twins.next().is_some() {
            return Err(format!("document {d}: more plain turns"));
        }
        all.push(lines);
    }
    Ok(all)
}

/// The label of the reply at turn `reply`: of the turns `candidates` (ascending), the one that
/// holds the most of the reply's tokens outside `stop`, if it holds at least one; ties go to the
/// later turn.
pub fn label(
    tokens: &[u32],
    doc: &[Turn],
    reply: usize,
    stop: &HashSet<u32>,
    candidates: &[usize],
) -> Option<usize> {
    let words: HashSet<u32> = tokens[doc[reply].content.clone()]
        .iter()
        .copied()
        .filter(|t| !stop.contains(t))
        .collect();
    let mut best = None;
    let mut most = 1;
    for &c in candidates {
        let turn: HashSet<u32> = tokens[doc[c].content.clone()].iter().copied().collect();
        let held = words.iter().filter(|w| turn.contains(w)).count();
        if held >= most {
            best = Some(c);
            most = held;
        }
    }
    best
}

/// Whether the copy read names token `t + 1` from position `t`: the latest position `s` of
/// `range` whose key equals the query, read for the token it holds. The key at `s` is the token
/// before it (copy key 1), or the two tokens before it when `two` (copy key 2).
pub fn names_next(x: &[u32], range: Range<usize>, t: usize, two: bool) -> bool {
    range
        .rev()
        .find(|&s| {
            s >= 1 && x[s - 1] == x[t] && (!two || (s >= 2 && t >= 1 && x[s - 2] == x[t - 1]))
        })
        .is_some_and(|s| x[s] == x[t + 1])
}

/// Whether `text` holds `value` as whole words.
pub fn holds(text: &str, value: &str) -> bool {
    !value.is_empty()
        && text.match_indices(value).any(|(i, _)| {
            let before = text[..i].chars().next_back();
            let after = text[i + value.len()..].chars().next();
            !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
        })
}

/// Where the value's tokens stand in the reply, if the reply states it in the same pieces and as
/// whole words (a run that is the start or the middle of a longer word is passed over).
fn value_span(
    x: &[u32],
    reply: &Range<usize>,
    value: &str,
    codec: &Codec<'_>,
) -> Option<Range<usize>> {
    [format!(" {value}"), value.to_owned()]
        .iter()
        .find_map(|text| {
            let needle = (codec.encode)(text);
            if needle.is_empty() || needle.len() > reply.len() {
                return None;
            }
            (reply.start..=reply.end - needle.len())
                .find(|&p| {
                    let end = p + needle.len();
                    let around = p.saturating_sub(1).max(reply.start)..(end + 1).min(reply.end);
                    x[p..end] == needle[..] && holds(&(codec.decode)(&x[around]), value)
                })
                .map(|p| p..p + needle.len())
        })
}

/// How the copy read does on one value with one turn admitted: every value token named, the
/// first named, and the ones after the first named (true when there are none).
#[derive(Clone, Copy, Default)]
struct Copied {
    n: usize,
    not_in_reply: usize,
    all: [usize; 3],
    first: [usize; 3],
    rest: [usize; 3],
}

impl Copied {
    /// Rules, in order: copy key 1 inside the turn, copy key 2 inside the turn, copy key 1 over
    /// everything from the document's start (`lo`) up to the token being read, the reply's own
    /// earlier tokens included (today's causal read, with no turn admitted).
    fn add(&mut self, x: &[u32], turn: &Range<usize>, lo: usize, span: Option<&Range<usize>>) {
        self.n += 1;
        let Some(span) = span else {
            self.not_in_reply += 1;
            return;
        };
        for rule in 0..3 {
            let named = |p: usize| match rule {
                0 => names_next(x, turn.clone(), p - 1, false),
                1 => names_next(x, turn.clone(), p - 1, true),
                _ => names_next(x, lo..p - 1, p - 1, false),
            };
            let first = named(span.start);
            let rest = (span.start + 1..span.end).all(named);
            self.first[rule] += usize::from(first);
            self.rest[rule] += usize::from(rest);
            self.all[rule] += usize::from(first && rest);
        }
    }

    fn line(&self, admitted: &str) -> String {
        let [k1, k2, any] = self.all;
        format!(
            "turn-census ceiling admitted={admitted} n={} not_in_reply={} k1_all={k1} k1_first={} k1_rest={} k2_all={k2} k2_first={} unrestricted_all={any} unrestricted_first={}",
            self.n, self.not_in_reply, self.first[0], self.rest[0], self.first[1], self.first[2]
        )
    }
}

/// The clauses of a turn: its content cut at punctuation tokens, empty pieces dropped.
pub fn clauses(x: &[u32], content: &Range<usize>, punct: &HashSet<u32>) -> Vec<Range<usize>> {
    let mut parts = Vec::new();
    let mut start = content.start;
    for i in content.clone() {
        if punct.contains(&x[i]) {
            if i > start {
                parts.push(start..i);
            }
            start = i + 1;
        }
    }
    if content.end > start {
        parts.push(start..content.end);
    }
    parts
}

fn user_turns_before(doc: &[Turn], reply: usize) -> Option<(usize, Vec<usize>)> {
    let asked = (0..reply).rev().find(|&i| doc[i].role == Role::User)?;
    Some((
        asked,
        (0..asked).filter(|&i| doc[i].role == Role::User).collect(),
    ))
}

fn reach_line(split: &str, docs: &[Vec<Turn>]) -> String {
    let mut hist = [0usize; 8];
    let (mut replies, mut most) = (0usize, 0usize);
    for doc in docs {
        for reply in (0..doc.len()).filter(|&i| doc[i].role == Role::Assistant) {
            let Some((_, prior)) = user_turns_before(doc, reply) else {
                continue;
            };
            replies += 1;
            most = most.max(prior.len());
            hist[prior.len().min(7)] += 1;
        }
    }
    let cells: Vec<String> = hist
        .iter()
        .enumerate()
        .map(|(n, c)| format!("{n}={c}"))
        .collect();
    format!(
        "turn-census reach split={split} replies={replies} prior_user_turns {} (7 is 7 or more) most={most}",
        cells.join(" ")
    )
}

/// Which frequent-token lists to count (each the size of the list), and which one is registered.
pub struct Stops<'a> {
    pub tops: &'a [usize],
    pub registered: usize,
}

/// The census: one line a reading, then up to `show` examples of each kind of miss.
pub fn turn_census(
    train: (&[u32], &[u8]),
    panel: (&[u32], &[u8]),
    lined: (&[u32], &[u8]),
    marks: &Marks,
    codec: &Codec<'_>,
    lists: &Stops<'_>,
    show: usize,
) -> Result<Vec<String>> {
    let (tops, registered_list) = (lists.tops, lists.registered);
    const WINDOWS: [usize; 3] = [192, 256, 384];
    let train_docs = documents(train.0, train.1, marks)?;
    let docs = documents(panel.0, panel.1, marks)?;
    let lined_docs = documents(lined.0, lined.1, marks)?;
    let lines = pair_lines(panel.0, &docs, lined.0, &lined_docs)?;
    let x = panel.0;
    let text = |range: &Range<usize>| (codec.decode)(&x[range.clone()]).replace('\n', " ");

    let mut freq: HashMap<u32, usize> = HashMap::new();
    for &t in train.0 {
        *freq.entry(t).or_default() += 1;
    }
    let mut ranked: Vec<(u32, usize)> = freq.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let seen: HashSet<u32> = ranked
        .iter()
        .map(|&(t, _)| t)
        .chain(x.iter().copied())
        .collect();
    let punct: HashSet<u32> = seen
        .into_iter()
        .filter(|&t| !(codec.decode)(&[t]).chars().any(char::is_alphanumeric))
        .collect();
    let stops: Vec<HashSet<u32>> = tops
        .iter()
        .map(|&k| {
            punct
                .iter()
                .copied()
                .chain(ranked.iter().take(k).map(|&(t, _)| t))
                .collect()
        })
        .collect();
    let stop = &stops[registered_list];

    let (mut general, mut abstain) = (0usize, 0usize);
    let (mut general_prior, mut abstain_prior) = (0usize, 0usize);
    let per_list = || vec![0usize; tops.len()];
    let (mut general_labelled, mut abstain_labelled) = (per_list(), per_list());
    let (mut clause_shares, mut clause_pick, mut turn_pick) = (per_list(), per_list(), per_list());
    let (mut contiguous, mut holder_clauses) = (0usize, [0usize; 4]);
    let mut values = 0usize;
    let (mut labelled, mut label_holds) = (vec![0usize; tops.len()], vec![0usize; tops.len()]);
    let (mut stated, mut in_asked, mut in_model, mut nowhere, mut label_latest) = (0, 0, 0, 0, 0);
    let (mut ext_model, mut ext_asked) = (0usize, 0usize);
    let (mut by_label, mut by_holder) = (Copied::default(), Copied::default());
    let mut forms: HashMap<u32, [usize; 5]> = HashMap::new();
    let (mut rival, mut rival_later, mut no_subject) = (0usize, 0usize, 0usize);
    let (mut equal, mut equal_later, mut holes_of) = (0usize, 0usize, 0usize);
    let mut within = [0usize; 3];
    let mut examples: [Vec<String>; 3] = Default::default();
    let kinds = ["unlabelled", "wrong-turn", "copy-miss"];

    for (doc, doc_lines) in docs.iter().zip(&lines) {
        let Some(lo) = doc.first().map(|t| t.content.start) else {
            continue;
        };
        for reply in (0..doc.len()).filter(|&i| doc[i].role == Role::Assistant) {
            let Some((asked, prior)) = user_turns_before(doc, reply) else {
                continue;
            };
            let labels: Vec<Option<usize>> = stops
                .iter()
                .map(|s| label(x, doc, reply, s, &prior))
                .collect();
            let registered = labels[registered_list];
            let line = match doc_lines[reply].as_ref() {
                None => None,
                Some(range) => {
                    let line = (codec.decode)(&lined.0[range.clone()]);
                    let Some(rest) = line.trim().strip_prefix("Memory:") else {
                        return Err(format!("a system line that is not a memory line: {line:?}"));
                    };
                    let rest = rest.trim();
                    Some(rest.strip_suffix('.').unwrap_or(rest).to_owned())
                }
            };
            let value = match line.as_deref() {
                None => {
                    general += 1;
                    general_prior += usize::from(!prior.is_empty());
                    for (count, found) in general_labelled.iter_mut().zip(&labels) {
                        *count += usize::from(found.is_some());
                    }
                    continue;
                }
                Some("none") => {
                    abstain += 1;
                    abstain_prior += usize::from(!prior.is_empty());
                    for (count, found) in abstain_labelled.iter_mut().zip(&labels) {
                        *count += usize::from(found.is_some());
                    }
                    continue;
                }
                Some(value) => value,
            };
            values += 1;
            let has = |i: usize| holds(&text(&doc[i].content), value);
            for (k, found) in labels.iter().enumerate() {
                if let Some(turn) = *found {
                    labelled[k] += 1;
                    label_holds[k] += usize::from(has(turn));
                }
            }
            let holder = prior.iter().copied().rfind(|&c| has(c));
            if holder.is_some() {
                stated += 1;
                label_latest += usize::from(registered == holder);
            } else if has(asked) {
                in_asked += 1;
            } else if (0..reply).any(|i| doc[i].role == Role::Assistant && has(i)) {
                in_model += 1;
            } else {
                nowhere += 1;
            }
            // What the question's own words could pick with nothing learned: a clause, or a turn.
            if let Some(turn) = holder {
                let spot = value_span(x, &doc[turn].content, value, codec);
                contiguous += usize::from(spot.is_some());
                let parts = clauses(x, &doc[turn].content, &punct);
                holder_clauses[parts.len().clamp(1, 4) - 1] += 1;
                let home = spot.and_then(|s| parts.iter().find(|c| c.contains(&s.start)).cloned());
                for (k, list) in stops.iter().enumerate() {
                    let words: HashSet<u32> = x[doc[asked].content.clone()]
                        .iter()
                        .copied()
                        .filter(|t| !list.contains(t))
                        .collect();
                    let shared = |range: &Range<usize>| {
                        let held: HashSet<u32> = x[range.clone()].iter().copied().collect();
                        words.iter().filter(|w| held.contains(w)).count()
                    };
                    let latest_most = |ranges: Vec<Range<usize>>| {
                        let mut best = None;
                        let mut most = 1;
                        for range in ranges {
                            let n = shared(&range);
                            if n >= most {
                                most = n;
                                best = Some(range);
                            }
                        }
                        best
                    };
                    let every_clause: Vec<Range<usize>> = prior
                        .iter()
                        .flat_map(|&c| clauses(x, &doc[c].content, &punct))
                        .collect();
                    let every_turn: Vec<Range<usize>> =
                        prior.iter().map(|&c| doc[c].content.clone()).collect();
                    if let Some(home) = &home {
                        clause_shares[k] += usize::from(shared(home) >= 1);
                        clause_pick[k] +=
                            usize::from(latest_most(every_clause).as_ref() == Some(home));
                    }
                    turn_pick[k] +=
                        usize::from(latest_most(every_turn).as_ref() == Some(&doc[turn].content));
                }
            }
            let wider: Vec<usize> = (0..reply)
                .filter(|&i| doc[i].role != Role::System)
                .collect();
            match label(x, doc, reply, stop, &wider) {
                Some(w) if w == asked => ext_asked += 1,
                Some(w) if doc[w].role == Role::Assistant => ext_model += 1,
                _ => {}
            }
            let span = value_span(x, &doc[reply].content, value, codec);
            if let Some(turn) = holder {
                by_holder.add(x, &doc[turn].content, lo, span.as_ref());
            }
            let show_it = |kind: usize, examples: &mut [Vec<String>; 3]| {
                if examples[kind].len() < show {
                    examples[kind].push(format!(
                        "turn-census example {} | value={value} | asked={} | label={} | reply={}",
                        kinds[kind],
                        text(&doc[asked].content),
                        registered.map_or("(none)".to_owned(), |l| text(&doc[l].content)),
                        text(&doc[reply].content)
                    ));
                }
            };
            let Some(turn) = registered else {
                show_it(0, &mut examples);
                continue;
            };
            let reach = doc[reply].content.end + 1 - doc[turn].content.start;
            for (count, window) in within.iter_mut().zip(WINDOWS) {
                *count += usize::from(reach <= window);
            }
            if !has(turn) {
                show_it(1, &mut examples);
                continue;
            }
            let before = by_label.all[0];
            by_label.add(x, &doc[turn].content, lo, span.as_ref());
            let copied = by_label.all[0] > before;
            if let Some(span) = &span {
                // A reply repeats the statement's wording at the value when the token before the
                // value is the same in the reply and in the turn.
                let said = &doc[turn].content;
                let same = value_span(x, said, value, codec).is_some_and(|home| {
                    home.start > said.start && x[home.start - 1] == x[span.start - 1]
                });
                let first = names_next(x, said.clone(), span.start - 1, false);
                let form = forms.entry(x[span.start - 1]).or_default();
                for (count, add) in form
                    .iter_mut()
                    .zip([true, copied, first, same, same && copied])
                {
                    *count += usize::from(add);
                }
            }
            if !copied {
                show_it(2, &mut examples);
            }
            // The turns that could compete with the labelled one.
            holes_of += 1;
            let value_ids: HashSet<u32> = (codec.encode)(&format!(" {value}"))
                .into_iter()
                .chain((codec.encode)(value))
                .collect();
            let bag = |i: usize| -> HashSet<u32> {
                x[doc[i].content.clone()]
                    .iter()
                    .copied()
                    .filter(|t| !stop.contains(t) && !value_ids.contains(t))
                    .collect()
            };
            let subject = bag(turn);
            if subject.is_empty() {
                no_subject += 1;
            } else {
                let rivals: Vec<usize> = prior
                    .iter()
                    .copied()
                    .filter(|&c| c != turn && !has(c) && subject.is_subset(&bag(c)))
                    .collect();
                rival += usize::from(!rivals.is_empty());
                rival_later += usize::from(rivals.iter().any(|&c| c > turn));
            }
            let sorted = |i: usize| {
                let mut ids = x[doc[i].content.clone()].to_vec();
                ids.sort_unstable();
                ids
            };
            let mine = sorted(turn);
            let twins: Vec<usize> = prior
                .iter()
                .copied()
                .filter(|&c| c != turn && sorted(c) == mine)
                .collect();
            equal += usize::from(!twins.is_empty());
            equal_later += usize::from(twins.iter().any(|&c| c > turn));
        }
    }

    let mut out = vec![format!(
        "turn-census panel documents={} general={general} abstain={abstain} value={values} punctuation_ids={} train_tokens={} general_with_a_prior_user_turn={general_prior} abstain_with_a_prior_user_turn={abstain_prior}",
        docs.len(),
        punct.len(),
        train.0.len()
    )];
    for (k, top) in tops.iter().enumerate() {
        out.push(format!(
            "turn-census label k={top}{} labelled={} of={values} label_holds_value={}",
            if k == registered_list {
                " registered"
            } else {
                ""
            },
            labelled[k],
            label_holds[k]
        ));
    }
    out.push(format!(
        "turn-census value-source of={values} in_a_prior_user_turn={stated} only_in_the_asked_turn={in_asked} only_in_a_model_turn={in_model} nowhere={nowhere} label_is_the_latest_holder={label_latest}"
    ));
    for (k, top) in tops.iter().enumerate() {
        let mark = if k == registered_list {
            " registered"
        } else {
            ""
        };
        out.push(format!(
            "turn-census leak k={top}{mark} general_labelled={} of={general} abstain_labelled={} of={abstain}",
            general_labelled[k], abstain_labelled[k]
        ));
        out.push(format!(
            "turn-census question-pick k={top}{mark} of={stated} clause_shares_a_question_token={} clause_is_the_pick={} turn_is_the_pick={}",
            clause_shares[k], clause_pick[k], turn_pick[k]
        ));
    }
    out.push(format!(
        "turn-census holder-turn of={stated} value_contiguous={contiguous} clauses 1={} 2={} 3={} 4_or_more={}",
        holder_clauses[0], holder_clauses[1], holder_clauses[2], holder_clauses[3]
    ));
    out.push(format!(
        "turn-census wider-candidates of={values} a_model_turn_wins={ext_model} the_asked_turn_wins={ext_asked}"
    ));
    out.push(by_label.line("label"));
    out.push(by_holder.line("latest-holder"));
    let mut forms: Vec<(u32, [usize; 5])> = forms.into_iter().collect();
    forms.sort_by(|a, b| b.1[0].cmp(&a.1[0]).then(a.0.cmp(&b.0)));
    for (before, [n, all, first, same, same_all]) in forms.into_iter().take(12) {
        out.push(format!(
            "turn-census form token_before_value={:?} n={n} k1_all={all} k1_first={first} turn_has_same_token_before={same} k1_all_of_those={same_all}",
            (codec.decode)(&[before])
        ));
    }
    out.push(format!(
        "turn-census holes of={holes_of} same_subject_rival={rival} rival_is_later={rival_later} no_subject_tokens={no_subject} equal_bag={equal} equal_bag_is_later={equal_later}"
    ));
    out.push(reach_line("train", &train_docs));
    out.push(reach_line("panel", &docs));
    out.push(format!(
        "turn-census window labelled={} within_192={} within_256={} within_384={}",
        labelled[registered_list], within[0], within[1], within[2]
    ));
    out.extend(examples.into_iter().flatten());
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORDS: [&str; 16] = [
        "<bos>",
        "<eos>",
        "\n",
        "System",
        "User",
        "Assistant",
        ":",
        "Memory",
        "my",
        "friend",
        "is",
        "maya",
        "who",
        "it's",
        ".",
        "ok",
    ];

    fn ids(text: &str) -> Vec<u32> {
        text.split(' ')
            .filter(|w| !w.is_empty())
            .map(|w| {
                WORDS
                    .iter()
                    .position(|&known| known == w)
                    .expect("a known word") as u32
            })
            .collect()
    }

    fn words(ids: &[u32]) -> String {
        ids.iter()
            .map(|&i| WORDS[i as usize])
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn marks() -> Marks {
        Marks {
            bos: 0,
            eos: 1,
            sep: ids("\n"),
            system: ids("System :"),
            user: ids("User :"),
            assistant: ids("Assistant :"),
        }
    }

    /// A corpus from turns written `role|content`; assistant content and its end mark get mask 1.
    fn corpus(turns: &[&str]) -> (Vec<u32>, Vec<u8>) {
        let (mut tokens, mut mask) = (vec![0], vec![0]);
        for (i, turn) in turns.iter().enumerate() {
            let (role, content) = turn.split_once('|').expect("role|content");
            let mut put = |text: &str, m: u8| {
                let new = ids(text);
                mask.extend(std::iter::repeat_n(m, new.len()));
                tokens.extend(new);
            };
            if i > 0 {
                put("\n", 0);
            }
            put(&format!("{role} :"), 0);
            let reply = u8::from(role == "Assistant");
            put(content, reply);
            if reply == 1 {
                put("<eos>", 1);
            }
        }
        (tokens, mask)
    }

    const PLAIN: [&str; 4] = [
        "User|my friend is maya .",
        "Assistant|ok .",
        "User|who is my friend",
        "Assistant|it's maya .",
    ];

    #[test]
    fn documents_are_split_into_turns_and_a_wrong_mask_is_refused() {
        let (tokens, mut mask) = corpus(&PLAIN);
        let docs = documents(&tokens, &mask, &marks()).expect("a well formed corpus");
        assert_eq!(docs.len(), 1);
        let roles: Vec<Role> = docs[0].iter().map(|t| t.role).collect();
        assert_eq!(
            roles,
            [Role::User, Role::Assistant, Role::User, Role::Assistant]
        );
        assert_eq!(
            words(&tokens[docs[0][0].content.clone()]),
            "my friend is maya ."
        );
        assert_eq!(words(&tokens[docs[0][3].content.clone()]), "it's maya .");
        mask[docs[0][0].content.start] = 1;
        assert!(documents(&tokens, &mask, &marks()).is_err());
        // A mask set on a role marker, on a separator or on the start mark is refused too.
        mask[docs[0][0].content.start] = 0;
        assert!(documents(&tokens, &mask, &marks()).is_ok());
        for at in [0, docs[0][1].content.start - 1, docs[0][1].content.end + 1] {
            mask[at] = 1;
            assert!(documents(&tokens, &mask, &marks()).is_err(), "token {at}");
            mask[at] = 0;
        }
        // The reply's end mark belongs to the reply: a mask of 0 there is refused.
        let end_mark = docs[0][1].content.end;
        assert_eq!(tokens[end_mark], marks().eos);
        mask[end_mark] = 0;
        assert!(documents(&tokens, &mask, &marks()).is_err());
    }

    #[test]
    fn a_value_inside_a_longer_word_is_passed_over() {
        let piece = |id: &u32| match id {
            100 => " 47",
            101 => "47",
            102 => "00",
            103 => ".",
            _ => " is",
        };
        let decode = |ids: &[u32]| ids.iter().map(piece).collect::<String>();
        let encode = |text: &str| vec![if text.starts_with(' ') { 100 } else { 101 }];
        let codec = Codec {
            decode: &decode,
            encode: &encode,
        };
        assert_eq!(
            value_span(&[104, 100, 103], &(0..3), "47", &codec),
            Some(1..2)
        );
        assert_eq!(value_span(&[104, 100, 102], &(0..3), "47", &codec), None);
        assert_eq!(
            value_span(&[104, 100, 102, 100, 103], &(0..5), "47", &codec),
            Some(3..4)
        );
    }

    #[test]
    fn a_line_is_paired_with_its_reply_and_other_differences_are_refused() {
        let (plain, plain_mask) = corpus(&PLAIN);
        let lined_turns = [
            PLAIN[0],
            PLAIN[1],
            PLAIN[2],
            "System|Memory : maya .",
            PLAIN[3],
        ];
        let (lined, lined_mask) = corpus(&lined_turns);
        let p = documents(&plain, &plain_mask, &marks()).expect("plain");
        let l = documents(&lined, &lined_mask, &marks()).expect("lined");
        let lines = pair_lines(&plain, &p, &lined, &l).expect("the same conversation");
        assert_eq!(
            lines[0].iter().map(Option::is_some).collect::<Vec<_>>(),
            [false, false, false, true]
        );
        let line = lines[0][3].clone().expect("the reply's line");
        assert_eq!(words(&lined[line]), "Memory : maya .");
        let (other, other_mask) = corpus(&["User|my friend is ok .", PLAIN[1], PLAIN[2], PLAIN[3]]);
        let o = documents(&other, &other_mask, &marks()).expect("other");
        assert!(pair_lines(&other, &o, &lined, &l).is_err());
        // A system line anywhere but directly before a reply is refused: before a user turn, two
        // before one reply, and one left over at the end.
        let line = "System|Memory : maya .";
        let misplaced: [&[&str]; 3] = [
            &[PLAIN[0], PLAIN[1], line, PLAIN[2], PLAIN[3]],
            &[PLAIN[0], PLAIN[1], PLAIN[2], line, line, PLAIN[3]],
            &[PLAIN[0], PLAIN[1], PLAIN[2], PLAIN[3], line],
        ];
        for turns in misplaced {
            let (bad, bad_mask) = corpus(turns);
            let b = documents(&bad, &bad_mask, &marks()).expect("a well formed corpus");
            assert!(pair_lines(&plain, &p, &bad, &b).is_err(), "{turns:?}");
        }
    }

    #[test]
    fn the_label_skips_stop_tokens_and_takes_the_later_of_two_tied_turns() {
        let turns = [
            "User|my friend is maya .",
            "Assistant|ok .",
            "User|my friend is maya .",
            "Assistant|ok .",
            "User|who is my friend",
            "Assistant|it's maya .",
        ];
        let (tokens, mask) = corpus(&turns);
        let docs = documents(&tokens, &mask, &marks()).expect("corpus");
        let stop: HashSet<u32> = ids(".").into_iter().collect();
        assert_eq!(label(&tokens, &docs[0], 5, &stop, &[0, 2]), Some(2));
        assert_eq!(label(&tokens, &docs[0], 5, &stop, &[0]), Some(0));
        let all: HashSet<u32> = ids("it's maya .").into_iter().collect();
        assert_eq!(label(&tokens, &docs[0], 5, &all, &[0, 2]), None);
    }

    #[test]
    fn the_copy_read_names_a_value_only_after_a_token_the_turn_holds_before_it() {
        // "is maya" is in the turn, so after "is" the read names "maya"; after "it's" it cannot.
        let (tokens, mask) = corpus(&[
            "User|my friend is maya .",
            "Assistant|friend is maya .",
            "User|who is my friend",
            "Assistant|it's maya .",
        ]);
        let docs = documents(&tokens, &mask, &marks()).expect("corpus");
        let turn = docs[0][0].content.clone();
        let repeats = docs[0][1].content.start + 1;
        assert_eq!(words(&tokens[repeats..repeats + 2]), "is maya");
        assert!(names_next(&tokens, turn.clone(), repeats, false));
        let short = docs[0][3].content.start;
        assert_eq!(words(&tokens[short..short + 2]), "it's maya");
        assert!(!names_next(&tokens, turn, short, false));
    }

    #[test]
    fn the_census_counts_a_label_that_holds_the_value_and_a_copy_that_misses_it() {
        let (plain, plain_mask) = corpus(&PLAIN);
        let (lined, lined_mask) = corpus(&[
            PLAIN[0],
            PLAIN[1],
            PLAIN[2],
            "System|Memory : maya .",
            PLAIN[3],
        ]);
        let decode = |ids: &[u32]| words(ids).replace(" :", ":").replace(" .", ".");
        let encode = |text: &str| ids(text);
        let codec = Codec {
            decode: &decode,
            encode: &encode,
        };
        let out = turn_census(
            (&plain, &plain_mask),
            (&plain, &plain_mask),
            (&lined, &lined_mask),
            &marks(),
            &codec,
            &Stops {
                tops: &[0],
                registered: 0,
            },
            1,
        )
        .expect("a census");
        let has = |needle: &str| out.iter().any(|line| line.contains(needle));
        assert!(has("general=1 abstain=0 value=1"), "{out:#?}");
        assert!(
            has("k=0 registered labelled=1 of=1 label_holds_value=1"),
            "{out:#?}"
        );
        assert!(has("in_a_prior_user_turn=1"), "{out:#?}");
        assert!(
            has("admitted=label n=1 not_in_reply=0 k1_all=0 k1_first=0 k1_rest=1"),
            "{out:#?}"
        );
        assert!(has("example copy-miss | value=maya"), "{out:#?}");
        // "who is my friend" shares three words with the one clause that holds the value.
        assert!(
            has("clause_shares_a_question_token=1 clause_is_the_pick=1 turn_is_the_pick=1"),
            "{out:#?}"
        );
        assert!(has("value_contiguous=1 clauses 1=1 2=0"), "{out:#?}");
        // "it's maya" against "is maya": the reply does not repeat the statement's token.
        assert!(
            has("n=1 k1_all=0 k1_first=0 turn_has_same_token_before=0 k1_all_of_those=0"),
            "{out:#?}"
        );
        assert!(
            has("general_with_a_prior_user_turn=0 abstain_with_a_prior_user_turn=0"),
            "{out:#?}"
        );
        // A system line that is not a memory line is refused, never read as a value.
        let (stray, stray_mask) = corpus(&[PLAIN[0], PLAIN[1], PLAIN[2], "System|ok .", PLAIN[3]]);
        let refused = turn_census(
            (&plain, &plain_mask),
            (&plain, &plain_mask),
            (&stray, &stray_mask),
            &marks(),
            &codec,
            &Stops {
                tops: &[0],
                registered: 0,
            },
            1,
        );
        assert!(refused.is_err());
    }

    #[test]
    fn a_turn_is_cut_into_clauses_at_punctuation() {
        let (tokens, mask) = corpus(&["User|my friend is maya . who is ok"]);
        let docs = documents(&tokens, &mask, &marks()).expect("corpus");
        let punct: HashSet<u32> = ids(".").into_iter().collect();
        let parts = clauses(&tokens, &docs[0][0].content, &punct);
        let texts: Vec<String> = parts.iter().map(|p| words(&tokens[p.clone()])).collect();
        assert_eq!(texts, ["my friend is maya", "who is ok"]);
    }
}
