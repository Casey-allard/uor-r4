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
//!
//! The place census (claude-350) is printed after it, from a fourth split, TRAIN's twin written
//! `recall=oracle`: what a read could do that has the question find the place where the value
//! starts, a register hold that place, and the reply read on from it (`place_census`).

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

// ---- The place census (claude-350) -------------------------------------------------------------

/// Where a recall reply's value stands: its first token in the user turn that states it (`s`),
/// its first token in the reply (`p`), and its length in tokens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Place {
    pub s: usize,
    pub p: usize,
    pub len: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Recall,
    General,
    Abstain,
}

/// One reply the place census counts. A general reply has a user turn before the one it answers;
/// a recall reply or an abstention is kept without one.
#[derive(Clone, Debug)]
pub struct Case {
    pub kind: Kind,
    /// The conversation's first content position.
    pub lo: usize,
    /// The contents of the user turns before the asked one, in order.
    pub users: Vec<Range<usize>>,
    pub asked: Range<usize>,
    pub reply: Range<usize>,
    pub place: Option<Place>,
}

/// The start of the latest run in `turn` equal to the reply's value run, as whole words.
fn same_run(
    x: &[u32],
    turn: &Range<usize>,
    span: &Range<usize>,
    value: &str,
    codec: &Codec<'_>,
) -> Option<usize> {
    let needle = &x[span.clone()];
    if needle.is_empty() || needle.len() > turn.len() {
        return None;
    }
    (turn.start..=turn.end - needle.len()).rev().find(|&q| {
        let end = q + needle.len();
        let around = q.saturating_sub(1).max(turn.start)..(end + 1).min(turn.end);
        x[q..end] == *needle && holds(&(codec.decode)(&x[around]), value)
    })
}

/// A recall reply with no place: its cause, and the line that lists it.
pub type Miss = (&'static str, String);

/// The replies the place census counts (every recall reply and abstention, and the general
/// replies with an earlier user turn), and each recall reply that has no place. A recall reply's
/// place is read from its memory line: the value's first whole-word run in the reply, and the
/// latest equal run in the latest earlier user turn that states the value.
pub fn cases(
    x: &[u32],
    docs: &[Vec<Turn>],
    lined: &[u32],
    lines: &[Vec<Option<Range<usize>>>],
    codec: &Codec<'_>,
) -> Result<(Vec<Case>, Vec<Miss>)> {
    let text = |range: &Range<usize>| (codec.decode)(&x[range.clone()]).replace('\n', " ");
    let (mut out, mut misses) = (Vec::new(), Vec::new());
    for (doc, doc_lines) in docs.iter().zip(lines) {
        let Some(lo) = doc.first().map(|t| t.content.start) else {
            continue;
        };
        for reply in (0..doc.len()).filter(|&i| doc[i].role == Role::Assistant) {
            let Some((asked, prior)) = user_turns_before(doc, reply) else {
                continue;
            };
            // Only a general reply is left out for having no earlier user turn: a recall reply
            // to a conversation's first question stays, as a miss.
            if prior.is_empty() && doc_lines[reply].is_none() {
                continue;
            }
            let value = match doc_lines[reply].as_ref() {
                None => None,
                Some(range) => {
                    let line = (codec.decode)(&lined[range.clone()]);
                    let Some(rest) = line.trim().strip_prefix("Memory:") else {
                        return Err(format!("a system line that is not a memory line: {line:?}"));
                    };
                    let rest = rest.trim();
                    Some(rest.strip_suffix('.').unwrap_or(rest).to_owned())
                }
            };
            let (kind, place) = match value.as_deref() {
                None => (Kind::General, None),
                Some("none") => (Kind::Abstain, None),
                Some(value) => {
                    let said = value_span(x, &doc[reply].content, value, codec);
                    let holder = prior
                        .iter()
                        .rev()
                        .find(|&&c| holds(&text(&doc[c].content), value));
                    let place = said.as_ref().zip(holder).and_then(|(span, &c)| {
                        same_run(x, &doc[c].content, span, value, codec).map(|s| Place {
                            s,
                            p: span.start,
                            len: span.len(),
                        })
                    });
                    if place.is_none() {
                        let cause = match (&said, holder) {
                            (None, _) => "the reply does not state the value as one run",
                            (_, None) => "no earlier user turn states the value",
                            _ => "the value is written in other tokens in its turn",
                        };
                        let line = format!(
                            "place-census miss | {cause} | value={value} | reply={}",
                            text(&doc[reply].content)
                        );
                        misses.push((cause, line));
                    }
                    (Kind::Recall, place)
                }
            };
            out.push(Case {
                kind,
                lo,
                users: prior.iter().map(|&c| doc[c].content.clone()).collect(),
                asked: doc[asked].content.clone(),
                reply: doc[reply].content.clone(),
                place,
            });
        }
    }
    Ok((out, misses))
}

/// What the learned question rows ask for: for each question token, the tokens that most often
/// stand before the true place over every row of every recall question (the most frequent first,
/// ties to the lower id; at most two).
pub fn targets(x: &[u32], cases: &[Case]) -> HashMap<u32, Vec<u32>> {
    let mut pairs: HashMap<u32, HashMap<u32, usize>> = HashMap::new();
    for case in cases {
        let Some(place) = case.place else {
            continue;
        };
        for t in case.asked.clone() {
            let before = pairs.entry(x[t]).or_default();
            *before.entry(x[place.s - 1]).or_default() += 1;
        }
    }
    pairs
        .into_iter()
        .map(|(token, counts)| {
            let mut ranked: Vec<(u32, usize)> = counts.into_iter().collect();
            ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            (token, ranked.into_iter().take(2).map(|(u, _)| u).collect())
        })
        .collect()
}

/// A question an ask head can form: learned rows or zero rows, one token or two, one head or two.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Form {
    pub name: &'static str,
    pub learned: bool,
    pub two: bool,
    pub heads: usize,
}

/// C1 to C3 of the ORDER, then the two widened forms.
pub const FORMS: [Form; 5] = [
    Form {
        name: "C1",
        learned: false,
        two: false,
        heads: 1,
    },
    Form {
        name: "C2",
        learned: true,
        two: false,
        heads: 1,
    },
    Form {
        name: "C3",
        learned: true,
        two: true,
        heads: 1,
    },
    Form {
        name: "C2w",
        learned: true,
        two: false,
        heads: 2,
    },
    Form {
        name: "C3w",
        learned: true,
        two: true,
        heads: 2,
    },
];

/// What one question row found: the latest position that fits, and how many fit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Find {
    pub t: usize,
    pub head: usize,
    pub pos: usize,
    pub fits: usize,
}

/// The finds of a reply's question, row by row. A position fits a row when the token before it
/// is the row's target (the row's own token for a zero row, or for a token with no learned
/// target) and, for a two-token question, the token two before it is the token before the row.
/// The candidates are every position of the conversation before the row, or the earlier user
/// turns only.
pub fn finds(
    x: &[u32],
    case: &Case,
    form: Form,
    user_only: bool,
    asks: &HashMap<u32, Vec<u32>>,
) -> Vec<Find> {
    let mut out = Vec::new();
    for t in case.asked.clone() {
        for head in 0..form.heads {
            let target = match asks.get(&x[t]).filter(|_| form.learned) {
                Some(list) => match list.get(head) {
                    Some(&target) => target,
                    None => continue,
                },
                None if head == 0 => x[t],
                None => continue,
            };
            let fit =
                |s: usize| s >= 2 && x[s - 1] == target && (!form.two || x[s - 2] == x[t - 1]);
            let whole = case.lo..t;
            let ranges: &[Range<usize>] = if user_only {
                &case.users
            } else {
                std::slice::from_ref(&whole)
            };
            let (mut pos, mut fits) = (None, 0usize);
            for s in ranges.iter().cloned().flatten().filter(|&s| fit(s)) {
                pos = Some(s);
                fits += 1;
            }
            if let Some(pos) = pos {
                out.push(Find { t, head, pos, fits });
            }
        }
    }
    out
}

/// One find a question row, as the ORDER words it: with two ask heads, the later of the heads'
/// positions, and every position that fits either head.
pub fn rows(found: &[Find]) -> Vec<Find> {
    let mut out: Vec<Find> = Vec::new();
    for f in found {
        match out.last_mut() {
            Some(row) if row.t == f.t => {
                row.pos = row.pos.max(f.pos);
                row.fits += f.fits;
            }
            _ => out.push(Find { head: 0, ..*f }),
        }
    }
    out
}

/// The keys a majority rule fires on: those with more right votes than wrong ones.
fn majority<K: Eq + std::hash::Hash>(votes: impl IntoIterator<Item = (K, bool)>) -> HashSet<K> {
    let mut tally: HashMap<K, (usize, usize)> = HashMap::new();
    for (key, right) in votes {
        let seat = tally.entry(key).or_default();
        if right {
            seat.0 += 1;
        } else {
            seat.1 += 1;
        }
    }
    tally
        .into_iter()
        .filter(|(_, (right, wrong))| right > wrong)
        .map(|(key, _)| key)
        .collect()
}

/// Which question row's find sets the register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arm {
    /// D1: the last row that finds anything.
    Last,
    /// D2: the row whose find fits the fewest positions, the later row on ties.
    Fewest,
    /// D3: the last row whose token fires under the majority rule.
    Voted,
}

pub const ARMS: [(Arm, &str); 3] = [(Arm::Last, "D1"), (Arm::Fewest, "D2"), (Arm::Voted, "D3")];

/// The place the register holds when the reply starts.
pub fn register(
    x: &[u32],
    found: &[Find],
    rule: Arm,
    fires: &HashSet<(u32, usize)>,
) -> Option<usize> {
    let pick = match rule {
        Arm::Last => found.last(),
        Arm::Fewest => found.iter().rev().min_by_key(|f| f.fits),
        Arm::Voted => found
            .iter()
            .rev()
            .find(|f| fires.contains(&(x[f.t], f.head))),
    };
    pick.map(|f| f.pos)
}

/// The reply rows the open rule is asked at while the register is set, each with whether opening
/// there is right. With the register at the true place only the rows up to the value-start row
/// count: the step opens the rest of the value, and the stop clears the register after it.
pub fn open_rows(case: &Case, reg: Option<usize>) -> Vec<(usize, bool)> {
    let Some(reg) = reg else {
        return Vec::new();
    };
    match case.place {
        Some(place) if place.s == reg => (case.reply.start - 1..place.p)
            .map(|t| (t, t + 1 == place.p))
            .collect(),
        _ => (case.reply.start - 1..case.reply.end)
            .map(|t| (t, false))
            .collect(),
    }
}

/// The label read from the tokens alone: the longest run of the reply equal to a run in an
/// earlier user turn, the latest on ties, starting at a token outside `stop`; a `space` token
/// directly before it in both joins it.
pub fn run_label(
    x: &[u32],
    case: &Case,
    stop: &HashSet<u32>,
    space: &HashSet<u32>,
) -> Option<Place> {
    let mut best: Option<(Place, usize)> = None;
    for p in case.reply.clone().filter(|&p| !stop.contains(&x[p])) {
        for user in &case.users {
            for s in user.clone().filter(|&s| x[s] == x[p]) {
                let room = (case.reply.end - p).min(user.end - s);
                let len = (0..room).take_while(|&i| x[p + i] == x[s + i]).count();
                if best.is_none_or(|(b, _)| len > b.len || (len == b.len && s > b.s)) {
                    best = Some((Place { s, p, len }, user.start));
                }
            }
        }
    }
    best.map(|(run, turn_start)| {
        let joins = run.s > turn_start
            && run.p > case.reply.start
            && x[run.s - 1] == x[run.p - 1]
            && space.contains(&x[run.s - 1]);
        if joins {
            Place {
                s: run.s - 1,
                p: run.p - 1,
                len: run.len + 1,
            }
        } else {
            run
        }
    })
}

/// The four corpus splits of the place census, each as its tokens and response mask: TRAIN and
/// the panel written `recall=off`, and each one's twin written `recall=oracle`.
pub struct Corpora<'a> {
    pub train: (&'a [u32], &'a [u8]),
    pub train_lined: (&'a [u32], &'a [u8]),
    pub panel: (&'a [u32], &'a [u8]),
    pub lined: (&'a [u32], &'a [u8]),
}

/// Whether `got` of `of` is at least `num / den`.
fn at_least(got: usize, of: usize, num: usize, den: usize) -> bool {
    of > 0 && got * den >= of * num
}

/// How the arming rules do on the panel.
#[derive(Clone, Copy, Default)]
struct Sets {
    reached: usize,
    right: usize,
    wrong_place: usize,
    general: usize,
    abstain: usize,
}

/// The place census (claude-350): counts A to G of the unit's ORDER and the choice each of its
/// rules makes, one line a reading. Every fitted part (the learned question rows' targets and
/// the majority rules) is fitted on TRAIN's places and on the panel's own; the rules read the
/// TRAIN fit (`fit=train`), and the panel fit is the ceiling. `lists.registered` names the stop
/// list of count F.
pub fn place_census(
    corpora: &Corpora<'_>,
    marks: &Marks,
    codec: &Codec<'_>,
    lists: &Stops<'_>,
    show: usize,
) -> Result<Vec<String>> {
    let load = |plain: (&[u32], &[u8]), twin: (&[u32], &[u8])| {
        let docs = documents(plain.0, plain.1, marks)?;
        let twin_docs = documents(twin.0, twin.1, marks)?;
        let lines = pair_lines(plain.0, &docs, twin.0, &twin_docs)?;
        cases(plain.0, &docs, twin.0, &lines, codec)
    };
    let (tx, x) = (corpora.train.0, corpora.panel.0);
    let (train_cases, _) = load(corpora.train, corpora.train_lined)?;
    let (panel_cases, misses) = load(corpora.panel, corpora.lined)?;
    let fits: [(&str, &[u32], &[Case]); 2] =
        [("train", tx, &train_cases), ("panel", x, &panel_cases)];

    let mut freq: HashMap<u32, usize> = HashMap::new();
    for &t in tx {
        *freq.entry(t).or_default() += 1;
    }
    let mut ranked: Vec<(u32, usize)> = freq.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let seen: HashSet<u32> = tx.iter().chain(x).copied().collect();
    let class = |test: &dyn Fn(&str) -> bool| -> HashSet<u32> {
        seen.iter()
            .copied()
            .filter(|&t| test(&(codec.decode)(&[t])))
            .collect()
    };
    let punct = class(&|s| !s.chars().any(char::is_alphanumeric));
    let space = class(&|s| !s.is_empty() && s.chars().all(char::is_whitespace));
    let new_word = class(&|s| s.starts_with(char::is_whitespace));
    let text = |range: &Range<usize>| (codec.decode)(&x[range.clone()]).replace('\n', " ");

    // A: the instrument.
    let of_kind = |cases: &[Case], kind: Kind| cases.iter().filter(|c| c.kind == kind).count();
    let recall: Vec<&Case> = panel_cases
        .iter()
        .filter(|c| c.kind == Kind::Recall)
        .collect();
    let n = recall.len();
    let general = of_kind(&panel_cases, Kind::General);
    let abstain = of_kind(&panel_cases, Kind::Abstain);
    let placed = recall.iter().filter(|c| c.place.is_some()).count();
    let mut out = vec![format!(
        "place-census A recall={n} read_on_reads_the_value={placed} general={general} abstain={abstain} train_recall={} train_placed={} train_general={} train_abstain={}",
        of_kind(&train_cases, Kind::Recall),
        train_cases.iter().filter(|c| c.place.is_some()).count(),
        of_kind(&train_cases, Kind::General),
        of_kind(&train_cases, Kind::Abstain)
    )];
    // The Instrument row: every miss is listed with its cause, and counted by cause.
    let mut causes: Vec<(&str, usize)> = Vec::new();
    for &(cause, _) in &misses {
        match causes.iter_mut().find(|seat| seat.0 == cause) {
            Some(seat) => seat.1 += 1,
            None => causes.push((cause, 1)),
        }
    }
    causes.sort();
    for (cause, count) in causes {
        out.push(format!(
            "place-census A-miss cause={cause:?} replies={count} of={n}"
        ));
    }
    out.extend(misses.into_iter().map(|(_, line)| line));
    let instrument = if placed == n {
        "every recall reply has a place".to_owned()
    } else {
        format!(
            "{} of {n} have no place: each is listed with its cause, and an instrument fault is repaired before anything else is read",
            n - placed
        )
    };
    out.push(format!("place-census decide instrument: {instrument}"));
    // A decision printed after a stopping clause is a reading, not a route.
    let unrouted = |stopped: bool| {
        if stopped {
            " (counted after a stopping clause: not routed)"
        } else {
            ""
        }
    };

    // B: the step and the stop.
    let turn_end = |case: &Case, s: usize| {
        let turn = case.users.iter().find(|u| u.contains(&s));
        turn.map_or(s, |u| u.end)
    };
    let stop_votes = |fx: &[u32], cases: &[Case]| {
        let mut votes = Vec::new();
        for case in cases {
            let Some(place) = case.place else {
                continue;
            };
            votes.extend((1..place.len).map(|i| (fx[place.s + i], false)));
            if place.s + place.len < turn_end(case, place.s) {
                votes.push((fx[place.s + place.len], true));
            }
        }
        votes
    };
    let exact = |stops: &dyn Fn(u32) -> bool| {
        let reads = |case: &Case, place: Place| {
            let after = place.s + place.len;
            (1..place.len).all(|i| !stops(x[place.s + i]))
                && (after == turn_end(case, place.s) || stops(x[after]))
        };
        recall
            .iter()
            .filter(|c| c.place.is_some_and(|place| reads(c, place)))
            .count()
    };
    let one_token = recall
        .iter()
        .filter(|c| c.place.is_some_and(|place| place.len == 1))
        .count();
    let b1 = exact(&|t| punct.contains(&t) || new_word.contains(&t));
    let b2: Vec<usize> = fits
        .iter()
        .map(|&(_, fx, fit_cases)| {
            let fires = majority(stop_votes(fx, fit_cases));
            exact(&|t| fires.contains(&t))
        })
        .collect();
    out.push(format!(
        "place-census B of={n} one_token={one_token} B1_exact={b1} B2_exact fit=train {} fit=panel {}",
        b2[0], b2[1]
    ));
    let stop = if one_token == n {
        "no step and no stop"
    } else if at_least(b1, n, 95, 100) {
        "B1"
    } else if at_least(b2[0], n, 95, 100) {
        "B2, a learned gain per source token"
    } else {
        "a plan change"
    };
    out.push(format!("place-census decide stop: {stop}"));

    // C: the find.
    let asks: Vec<HashMap<u32, Vec<u32>>> = fits
        .iter()
        .map(|&(_, fx, fit_cases)| targets(fx, fit_cases))
        .collect();
    // A reply passes when some row's find is the true place. A row has one find (`rows`); with
    // two heads the count that takes either head's own find is printed beside it.
    let reaches = |case: &Case, form: Form, user_only: bool, k: usize, by_row: bool| {
        case.place.is_some_and(|place| {
            let found = finds(x, case, form, user_only, &asks[k]);
            let found = if by_row { rows(&found) } else { found };
            found.iter().any(|f| f.pos == place.s)
        })
    };
    let mut read = Vec::new();
    for form in FORMS {
        for (k, &(name, ..)) in fits.iter().enumerate() {
            let count = |user_only: bool, by_row: bool| {
                recall
                    .iter()
                    .filter(|c| reaches(c, form, user_only, k, by_row))
                    .count()
            };
            let (every, users) = (count(false, true), count(true, true));
            let beside = if form.heads > 1 {
                format!(
                    " either_head_every_position={} either_head_user_turns_only={}",
                    count(false, false),
                    count(true, false)
                )
            } else {
                String::new()
            };
            out.push(format!(
                "place-census C form={} fit={name} of={n} every_position={every} user_turns_only={users}{beside}",
                form.name
            ));
            if k == 0 {
                read.push(if users > every {
                    (form, true, users)
                } else {
                    (form, false, every)
                });
            }
        }
    }
    let simple = read[..3]
        .iter()
        .copied()
        .find(|r| at_least(r.2, n, 80, 100));
    let best = read
        .iter()
        .copied()
        .fold(read[0], |best, r| if r.2 > best.2 { r } else { best });
    let ((form, user_only, reached), find) = match simple {
        Some(r) => (r, "built"),
        None if at_least(best.2, n, 50, 100) => (best, "built, with a share it cannot reach"),
        None => (best, "not built as written"),
    };
    out.push(format!(
        "place-census decide find: form={} user_turns_only={user_only} reaches={reached} of={n}: {find}",
        form.name
    ));
    // What the chosen form's rows ask for where they reach, and the replies it misses.
    let mut asked_for: HashMap<(u32, u32), usize> = HashMap::new();
    // The trace of a miss: no row finds anything; rows find, but the place fits none of them; or
    // the place fits a row and a later position fits that row too.
    let mut trace = [0usize; 3];
    let mut missed = Vec::new();
    for case in &recall {
        let Some(place) = case.place else {
            continue;
        };
        let found = rows(&finds(x, case, form, user_only, &asks[0]));
        if let Some(f) = found.iter().find(|f| f.pos == place.s) {
            *asked_for.entry((x[f.t], x[place.s - 1])).or_default() += 1;
            continue;
        }
        let only = Case {
            users: std::iter::once(place.s..place.s + 1).collect(),
            ..Case::clone(case)
        };
        let cause = if found.is_empty() {
            0
        } else if finds(x, &only, form, true, &asks[0]).is_empty() {
            1
        } else {
            2
        };
        trace[cause] += 1;
        if missed.len() < show {
            let stated = case.users.iter().find(|u| u.contains(&place.s));
            missed.push(format!(
                "place-census find-miss | asked={} | stated={} | value={}",
                text(&case.asked),
                stated.map_or(String::new(), &text),
                text(&(place.s..place.s + place.len))
            ));
        }
    }
    out.push(format!(
        "place-census find-trace missed={} no_row_finds_anything={} the_place_fits_no_row={} a_later_position_fits_too={}",
        trace.iter().sum::<usize>(),
        trace[0],
        trace[1],
        trace[2]
    ));
    out.extend(missed);
    let mut asked_for: Vec<((u32, u32), usize)> = asked_for.into_iter().collect();
    asked_for.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    for ((token, target), count) in asked_for.into_iter().take(12) {
        out.push(format!(
            "place-census find-row question_token={:?} asks_for={:?} replies={count}",
            (codec.decode)(&[token]),
            (codec.decode)(&[target])
        ));
    }

    // D: the arming rule. For each fit, the finds of the fit's own corpus (the majority rule is
    // fitted on them) and the finds of the panel.
    let found_in = |cx: &[u32], cases: &[Case], k: usize| -> Vec<Vec<Find>> {
        cases
            .iter()
            .map(|c| rows(&finds(cx, c, form, user_only, &asks[k])))
            .collect()
    };
    let mut armed = Vec::new();
    for (k, &(_, fx, fit_cases)) in fits.iter().enumerate() {
        let fit_found = found_in(fx, fit_cases, k);
        let fires = majority(fit_cases.iter().zip(&fit_found).flat_map(|(case, found)| {
            let right = move |f: &Find| case.place.is_some_and(|place| place.s == f.pos);
            found.iter().map(move |f| ((fx[f.t], f.head), right(f)))
        }));
        let panel_found = if k == 0 {
            found_in(x, &panel_cases, k)
        } else {
            fit_found.clone()
        };
        armed.push((fires, fit_found, panel_found));
    }
    let count_sets = |k: usize, rule: Arm| {
        let (fires, _, panel_found) = &armed[k];
        let mut sets = Sets::default();
        for (case, found) in panel_cases.iter().zip(panel_found) {
            let reg = register(x, found, rule, fires);
            match case.kind {
                Kind::Recall => {
                    let s = case.place.map(|place| place.s);
                    let reached = s.is_some_and(|s| found.iter().any(|f| f.pos == s));
                    sets.reached += usize::from(reached);
                    sets.right += usize::from(s.is_some() && reg == s);
                    sets.wrong_place += usize::from(reg.is_some() && reg != s);
                }
                Kind::General => sets.general += usize::from(reg.is_some()),
                Kind::Abstain => sets.abstain += usize::from(reg.is_some()),
            }
        }
        sets
    };
    let mut table = Vec::new();
    for (k, &(name, ..)) in fits.iter().enumerate() {
        for (rule, label) in ARMS {
            let s = count_sets(k, rule);
            out.push(format!(
                "place-census D fit={name} rule={label} reached={} true_place={} wrong_place={} of={n} general_set={} of={general} abstain_set={} of={abstain}",
                s.reached, s.right, s.wrong_place, s.general, s.abstain
            ));
            if k == 0 {
                table.push((rule, label, s));
            }
        }
    }
    let clean = |s: &Sets| {
        at_least(s.right, s.reached, 90, 100)
            && s.general * 100 <= general * 15
            && s.abstain * 100 <= abstain * 10
    };
    let wrong_sets = |s: &Sets| s.general + s.abstain + s.wrong_place;
    let half = table
        .iter()
        .copied()
        .filter(|(_, _, s)| at_least(s.right, s.reached, 50, 100));
    let ((rule, label, _), arm) = match table.iter().copied().find(|(_, _, s)| clean(s)) {
        Some(row) => (row, "built"),
        None => match half.min_by_key(|(_, _, s)| wrong_sets(s)) {
            Some(row) => (
                row,
                "built; stage 3 reads general replies and abstentions first",
            ),
            None => {
                let most = table.iter().copied().max_by_key(|(_, _, s)| s.right);
                (most.expect("three rules"), "a plan change")
            }
        },
    };
    let find_stops = find == "not built as written";
    out.push(format!(
        "place-census decide arm: {label}: {arm}{}",
        unrouted(find_stops)
    ));
    let arm_stops = find_stops || arm == "a plan change";
    let mut arming: Vec<String> = armed[0]
        .0
        .iter()
        .map(|&(token, _)| format!("{:?}", (codec.decode)(&[token])))
        .collect();
    arming.sort();
    out.push(format!(
        "place-census arm-tokens fit=train count={} first={:?}",
        arming.len(),
        &arming[..arming.len().min(24)]
    ));

    // E: the open rule, with the register as each arming rule sets it. The Open row reads the
    // chosen rule's line; the other two are printed beside it.
    for (k, &(name, fx, fit_cases)) in fits.iter().enumerate() {
        let (fires, fit_found, panel_found) = &armed[k];
        for (arm, arm_name) in ARMS {
            let regs = |cx: &[u32], found: &[Vec<Find>]| -> Vec<Option<usize>> {
                let reg = |f: &Vec<Find>| register(cx, f, arm, fires);
                found.iter().map(reg).collect()
            };
            let fit_regs = regs(fx, fit_found);
            let opens = majority(fit_cases.iter().zip(fit_regs).flat_map(|(case, reg)| {
                let rows = open_rows(case, reg).into_iter();
                rows.map(move |(t, right)| (fx[t], right))
            }));
            // `first`: recall replies where the value-start row is opened and no row before it is.
            let (mut right, mut right_of, mut first) = (0usize, 0usize, 0usize);
            let (mut wrong, mut rows) = ([0usize; 3], [0usize; 3]);
            for (case, reg) in panel_cases.iter().zip(regs(x, panel_found)) {
                let kind = case.kind as usize;
                rows[kind] += case.reply.len() + 1;
                right_of += usize::from(reg.is_some() && reg == case.place.map(|place| place.s));
                let (mut early, mut opened) = (false, false);
                for (t, is_right) in open_rows(case, reg) {
                    if opens.contains(&x[t]) {
                        if is_right {
                            right += 1;
                            opened = true;
                        } else {
                            wrong[kind] += 1;
                            early = true;
                        }
                    }
                }
                first += usize::from(opened && !early);
            }
            out.push(format!(
                "place-census E fit={name} arm={arm_name} right={right} of={right_of} wrong_recall={} of_rows={} wrong_general={} of_rows={} wrong_abstain={} of_rows={} value_start_is_the_first_open={first} of={n}",
                wrong[0], rows[0], wrong[1], rows[1], wrong[2], rows[2]
            ));
            if k == 0 && arm == rule {
                let fine = at_least(right, right_of, 95, 100)
                    && (0..3).all(|i| wrong[i] * 100 <= rows[i] * 2);
                let open = if fine {
                    "a learned gain per reply token"
                } else {
                    "a plan change"
                };
                out.push(format!(
                    "place-census decide open: arm={label}: {open}{}",
                    unrouted(arm_stops)
                ));
                let mut opening: Vec<String> =
                    opens.iter().map(|&t| (codec.decode)(&[t])).collect();
                opening.sort();
                out.push(format!(
                    "place-census open-tokens fit=train arm={label} count={} first={:?}",
                    opening.len(),
                    &opening[..opening.len().min(24)]
                ));
            }
        }
    }

    // F: the label read from the tokens alone.
    let label_counts = |cx: &[u32], cases: &[Case], stop: &HashSet<u32>| {
        // Recall replies, those labelled, labels equal to the true place, labels that hold the
        // value at the same offset, then general replies and abstentions the label fires on.
        let mut c = [0usize; 6];
        for case in cases {
            let label = run_label(cx, case, stop, &space);
            match case.kind {
                Kind::Recall => {
                    c[0] += 1;
                    c[1] += usize::from(label.is_some());
                    if let (Some(run), Some(place)) = (label, case.place) {
                        c[2] += usize::from(run.s == place.s);
                        let holds = run.s <= place.s
                            && place.s + place.len <= run.s + run.len
                            && run.p + (place.s - run.s) == place.p;
                        c[3] += usize::from(holds);
                    }
                }
                Kind::General => c[4] += usize::from(label.is_some()),
                Kind::Abstain => c[5] += usize::from(label.is_some()),
            }
        }
        c
    };
    let stop_list = |top: usize| -> HashSet<u32> {
        let frequent = ranked.iter().take(top).map(|&(t, _)| t);
        punct.iter().copied().chain(frequent).collect()
    };
    let mut equal = 0;
    for (k, &top) in lists.tops.iter().enumerate() {
        let registered = k == lists.registered;
        let mark = if registered { " registered" } else { "" };
        let mut line = |split: &str, c: [usize; 6], general: usize, abstain: usize| {
            out.push(format!(
                "place-census F list={top}{mark} split={split} labelled={} equal={} holds_the_value={} of={} general_fired={} of={general} abstain_fired={} of={abstain}",
                c[1], c[2], c[3], c[0], c[4], c[5]
            ));
        };
        let c = label_counts(x, &panel_cases, &stop_list(top));
        line("panel", c, general, abstain);
        if registered {
            equal = c[2];
            let c = label_counts(tx, &train_cases, &stop_list(top));
            let train_general = of_kind(&train_cases, Kind::General);
            line(
                "train",
                c,
                train_general,
                of_kind(&train_cases, Kind::Abstain),
            );
        }
    }
    let label = if at_least(equal, n, 90, 100) {
        "labels from the tokens"
    } else {
        "places from the oracle twin"
    };
    out.push(format!("place-census decide label: {label}"));

    // G: the window, as the tokens from the value-start row back to the place. The shares are of
    // the replies that have a place. The distance is in rows, `p - 1 - s`: a register that
    // reaches W rows back from the value-start row holds the place when the distance is at most
    // W, and stage 2's window is counted the same way.
    let reach = |cases: &[Case]| {
        let mut within = [0usize; 4];
        for place in cases.iter().filter_map(|c| c.place) {
            let bucket = match place.p - 1 - place.s {
                0..=192 => 0,
                193..=256 => 1,
                257..=512 => 2,
                _ => 3,
            };
            within[bucket] += 1;
        }
        within
    };
    let w = reach(&panel_cases);
    for (split, w) in [("panel", w), ("train", reach(&train_cases))] {
        out.push(format!(
            "place-census G split={split} within_192={} to_256={} to_512={} beyond={}",
            w[0], w[1], w[2], w[3]
        ));
    }
    let window = if w[3] * 100 > placed * 5 {
        "a plan change"
    } else if (w[1] + w[2] + w[3]) * 100 <= placed * 5 {
        "192"
    } else if at_least(w[0] + w[1], placed, 95, 100) {
        "256"
    } else {
        "512"
    };
    out.push(format!("place-census decide window: {window}"));
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

    /// The plain corpus of `turns` and its twin with a memory line before the last reply.
    fn twins(turns: &[&str], line: &str) -> [(Vec<u32>, Vec<u8>); 2] {
        let mut lined = turns.to_vec();
        lined.insert(turns.len() - 1, line);
        [corpus(turns), corpus(&lined)]
    }

    #[test]
    fn the_place_census_reads_one_recall_reply_from_find_to_stop() {
        let [(plain, plain_mask), (lined, lined_mask)] = twins(&PLAIN, "System|Memory : maya .");
        let decode = |ids: &[u32]| words(ids).replace(" :", ":").replace(" .", ".");
        let encode = |text: &str| ids(text);
        let codec = Codec {
            decode: &decode,
            encode: &encode,
        };
        let corpora = Corpora {
            train: (&plain, &plain_mask),
            train_lined: (&lined, &lined_mask),
            panel: (&plain, &plain_mask),
            lined: (&lined, &lined_mask),
        };
        let lists = Stops {
            tops: &[0],
            registered: 0,
        };
        let out = place_census(&corpora, &marks(), &codec, &lists, 4).expect("a census");
        let has = |needle: &str| out.iter().any(|line| line.contains(needle));
        assert!(
            has("A recall=1 read_on_reads_the_value=1 general=0 abstain=0"),
            "{out:#?}"
        );
        assert!(
            has("decide instrument: every recall reply has a place"),
            "{out:#?}"
        );
        // "it's" opens the value-start row and the role marker's ":" before it stays shut.
        assert!(has("value_start_is_the_first_open=1 of=1"), "{out:#?}");
        // "maya" is one token, and "." follows it in its turn.
        assert!(has("B of=1 one_token=1 B1_exact=1"), "{out:#?}");
        assert!(has("decide stop: no step and no stop"), "{out:#?}");
        // "is" stands before "maya" and the question holds "is": a zero row finds the place.
        assert!(
            has("C form=C1 fit=train of=1 every_position=1 user_turns_only=1"),
            "{out:#?}"
        );
        // No row of the question follows "friend", the token two before the place.
        assert!(
            has("C form=C3 fit=train of=1 every_position=0 user_turns_only=0"),
            "{out:#?}"
        );
        assert!(
            has("decide find: form=C1 user_turns_only=false reaches=1 of=1: built"),
            "{out:#?}"
        );
        assert!(
            has("find-row question_token=\"is\" asks_for=\"is\" replies=1"),
            "{out:#?}"
        );
        // The last row, "friend", finds "is" in the statement: only the majority rule keeps the
        // row whose find is the place.
        assert!(
            has("D fit=train rule=D1 reached=1 true_place=0 wrong_place=1"),
            "{out:#?}"
        );
        assert!(
            has("D fit=train rule=D2 reached=1 true_place=0 wrong_place=1"),
            "{out:#?}"
        );
        assert!(
            has("D fit=train rule=D3 reached=1 true_place=1 wrong_place=0"),
            "{out:#?}"
        );
        assert!(has("decide arm: D3: built"), "{out:#?}");
        // The reply's rows before the value: the role marker's ":" (wrong) and "it's" (right).
        assert!(
            has("E fit=train arm=D3 right=1 of=1 wrong_recall=0 of_rows=4"),
            "{out:#?}"
        );
        assert!(
            has("decide open: arm=D3: a learned gain per reply token"),
            "{out:#?}"
        );
        assert!(
            has("F list=0 registered split=panel labelled=1 equal=1 holds_the_value=1 of=1"),
            "{out:#?}"
        );
        assert!(has("decide label: labels from the tokens"), "{out:#?}");
        assert!(has("G split=panel within_192=1 to_256=0"), "{out:#?}");
        assert!(has("decide window: 192"), "{out:#?}");
    }

    #[test]
    fn a_find_is_the_latest_fit_and_user_turns_only_passes_over_a_model_turn() {
        let turns = [
            "User|my friend is maya .",
            "Assistant|friend is ok .",
            "User|who is my friend",
            "Assistant|it's maya .",
        ];
        let [(plain, plain_mask), (lined, lined_mask)] = twins(&turns, "System|Memory : maya .");
        let decode = |ids: &[u32]| words(ids).replace(" :", ":").replace(" .", ".");
        let encode = |text: &str| ids(text);
        let codec = Codec {
            decode: &decode,
            encode: &encode,
        };
        let docs = documents(&plain, &plain_mask, &marks()).expect("corpus");
        let lined_docs = documents(&lined, &lined_mask, &marks()).expect("corpus");
        let lines = pair_lines(&plain, &docs, &lined, &lined_docs).expect("twins");
        let (replies, misses) = cases(&plain, &docs, &lined, &lines, &codec).expect("cases");
        assert!(misses.is_empty(), "{misses:#?}");
        // The first reply has no earlier user turn, so only the recall reply is a case.
        assert_eq!(replies.len(), 1);
        assert_eq!(replies[0].kind, Kind::Recall);
        let place = replies[0].place.expect("a place");
        assert_eq!(words(&plain[place.s - 1..=place.s]), "is maya");
        assert_eq!(words(&plain[place.p - 1..=place.p]), "it's maya");
        let none = HashMap::new();
        let row = |user_only: bool| {
            finds(&plain, &replies[0], FORMS[0], user_only, &none)
                .into_iter()
                .find(|f| words(&plain[f.t..=f.t]) == "is")
                .expect("the row finds a position")
        };
        // Over every position the model's "is ok" is later than the statement's "is maya".
        let every = row(false);
        assert_eq!(words(&plain[every.pos..=every.pos]), "ok");
        assert_eq!(every.fits, 2);
        let users = row(true);
        assert_eq!((users.pos, users.fits), (place.s, 1));
        // A learned row asks for its target, not for its own token: "who" asks for "is".
        let asks = targets(&plain, &replies);
        let who = finds(&plain, &replies[0], FORMS[1], true, &asks);
        assert_eq!(who[0].pos, place.s);
        assert_eq!(words(&plain[who[0].t..=who[0].t]), "who");
    }

    #[test]
    fn the_token_label_takes_the_longest_run_and_the_later_of_two_equal_ones() {
        let label_of = |last: &str| {
            let turns = [
                "User|my friend is maya .",
                "Assistant|ok .",
                "User|my friend is ok .",
                "Assistant|ok .",
                "User|who is my friend",
                last,
            ];
            let (plain, mask) = corpus(&turns);
            let docs = documents(&plain, &mask, &marks()).expect("corpus");
            let doc = &docs[0];
            let case = Case {
                kind: Kind::General,
                lo: doc[0].content.start,
                users: vec![doc[0].content.clone(), doc[2].content.clone()],
                asked: doc[4].content.clone(),
                reply: doc[5].content.clone(),
                place: None,
            };
            let stop: HashSet<u32> = ids(".").into_iter().collect();
            let label = run_label(&plain, &case, &stop, &HashSet::new()).expect("a label");
            (label.s - doc[2].content.start, label.len)
        };
        // "my friend is ok ." is whole in the second statement, and only three tokens of the
        // first.
        assert_eq!(label_of("Assistant|my friend is ok ."), (0, 5));
        // "my friend" is in both statements: the later one is the label.
        assert_eq!(label_of("Assistant|my friend ."), (0, 2));
    }

    #[test]
    fn a_space_token_before_the_run_in_both_joins_the_label() {
        // "is" stands in for the tokenizer's lone space token: it is on the stop list, so a run
        // cannot start at it, and it joins a run it stands before in the turn and in the reply.
        let label_of = |last: &str| {
            let turns = [
                "User|my friend is maya .",
                "Assistant|ok .",
                "User|who is my friend",
                last,
            ];
            let (plain, mask) = corpus(&turns);
            let docs = documents(&plain, &mask, &marks()).expect("corpus");
            let doc = &docs[0];
            let case = Case {
                kind: Kind::Recall,
                lo: doc[0].content.start,
                users: vec![doc[0].content.clone()],
                asked: doc[2].content.clone(),
                reply: doc[3].content.clone(),
                place: None,
            };
            let stop: HashSet<u32> = ids(". is").into_iter().collect();
            let space: HashSet<u32> = ids("is").into_iter().collect();
            let label = run_label(&plain, &case, &stop, &space).expect("a label");
            let run = words(&plain[label.s..label.s + label.len]);
            (run, label.p - doc[3].content.start)
        };
        assert_eq!(
            label_of("Assistant|it's is maya ."),
            ("is maya .".to_owned(), 1)
        );
        // "it's" stands before the run in the reply: nothing joins.
        assert_eq!(label_of("Assistant|it's maya ."), ("maya .".to_owned(), 1));
    }

    #[test]
    fn two_heads_on_one_row_give_one_find_at_the_later_position() {
        let find = |t, head, pos, fits| Find { t, head, pos, fits };
        let found = [find(5, 0, 40, 1), find(5, 1, 12, 2), find(6, 1, 7, 1)];
        assert_eq!(rows(&found), [find(5, 0, 40, 3), find(6, 0, 7, 1)]);
        // The register takes the row's one find, whichever head held it.
        let last = register(&[], &rows(&found[..2]), Arm::Last, &HashSet::new());
        assert_eq!(last, Some(40));
    }

    #[test]
    fn the_census_counts_one_find_a_row_and_a_first_open_only_with_no_row_open_before_it() {
        // Four recall replies to the one-token question "who". Over them its targets are "my"
        // (twice; the lower id, so the first head's) and "is" (twice; the second head's).
        let reply = "Assistant|it's maya .";
        let mine = ["User|my maya .", "Assistant|ok .", "User|who", reply];
        // The first "it's" of this reply opens before the value-start row does.
        let twice = [mine[0], mine[1], mine[2], "Assistant|it's it's maya ."];
        // Only the second head finds the place.
        let is = ["User|friend is maya .", mine[1], mine[2], reply];
        // The second head finds the place and the first a later position, the one after "my".
        let both = ["User|friend is maya . my ok .", mine[1], mine[2], reply];
        let mut joined = [(Vec::new(), Vec::new()), (Vec::new(), Vec::new())];
        for turns in [mine, twice, is, both] {
            let pair = twins(&turns, "System|Memory : maya .");
            for (whole, (tokens, mask)) in joined.iter_mut().zip(pair) {
                whole.0.extend(tokens);
                whole.1.extend(mask);
            }
        }
        let [(plain, plain_mask), (lined, lined_mask)] = joined;
        let decode = |ids: &[u32]| words(ids).replace(" :", ":").replace(" .", ".");
        let encode = |text: &str| ids(text);
        let codec = Codec {
            decode: &decode,
            encode: &encode,
        };
        let corpora = Corpora {
            train: (&plain, &plain_mask),
            train_lined: (&lined, &lined_mask),
            panel: (&plain, &plain_mask),
            lined: (&lined, &lined_mask),
        };
        let lists = Stops {
            tops: &[0],
            registered: 0,
        };
        let out = place_census(&corpora, &marks(), &codec, &lists, 4).expect("a census");
        let expected = [
            "A recall=4 read_on_reads_the_value=4 general=0 abstain=0",
            // One head asks for "my": the two "my maya" replies.
            "C form=C2 fit=train of=4 every_position=2 user_turns_only=2",
            // Two heads add the "is maya" reply. The last reply's row has one find, the later
            // position, so it is reached only when either head's own find counts.
            "C form=C2w fit=train of=4 every_position=3 user_turns_only=3 either_head_every_position=4 either_head_user_turns_only=4",
            "decide find: form=C2w user_turns_only=false reaches=3 of=4: built, with a share it cannot reach",
            // The register takes the row's one find: a wrong place on the last reply.
            "D fit=train rule=D1 reached=3 true_place=3 wrong_place=1",
            // "it's" opens: right on three replies, early on the doubled one, wrong on the last.
            "E fit=train arm=D1 right=3 of=3 wrong_recall=2",
            "value_start_is_the_first_open=2 of=4",
        ];
        let missing: Vec<&str> = expected
            .into_iter()
            .filter(|needle| !out.iter().any(|line| line.contains(needle)))
            .collect();
        assert!(missing.is_empty(), "missing {missing:#?} in {out:#?}");
    }

    #[test]
    fn a_recall_reply_to_the_first_question_is_kept_as_a_miss() {
        let turns = ["User|who is my friend", "Assistant|it's maya ."];
        let [(plain, plain_mask), (lined, lined_mask)] = twins(&turns, "System|Memory : maya .");
        let decode = |ids: &[u32]| words(ids).replace(" :", ":").replace(" .", ".");
        let encode = |text: &str| ids(text);
        let codec = Codec {
            decode: &decode,
            encode: &encode,
        };
        let docs = documents(&plain, &plain_mask, &marks()).expect("corpus");
        let lined_docs = documents(&lined, &lined_mask, &marks()).expect("corpus");
        let lines = pair_lines(&plain, &docs, &lined, &lined_docs).expect("twins");
        let (replies, misses) = cases(&plain, &docs, &lined, &lines, &codec).expect("cases");
        assert_eq!(replies.len(), 1);
        assert_eq!((replies[0].kind, replies[0].place), (Kind::Recall, None));
        assert_eq!(misses.len(), 1);
        assert_eq!(misses[0].0, "no earlier user turn states the value");
    }
}
