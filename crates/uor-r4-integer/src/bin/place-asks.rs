//! place-asks (claude-359): fit the asks on a split by counting, and count what they find.
//!
//! ```text
//! place-asks fit    --tokens TRAIN/tokens.u16 --kit train-places.json --out asks.json \
//!                   [--open-reply-start 0|1]
//! place-asks census --tokens PANEL/tokens.u16 --kit panel-places.json --asks asks.json [--show N]
//! place-asks ablate --tokens TRAIN/tokens.u16 --kit train-places.json \
//!                   --held PANEL/tokens.u16 --held-kit panel-places.json [--held .. --held-kit ..] \
//!                   [--slots 1,2,3|none] [--question-last N] [--open-reply-start 0|1]
//! place-asks repeat --tokens TRAIN/tokens.u16 --kit train-places.json --asks asks.json --out with-pairs.json
//! place-asks forms  --tokens TRAIN/tokens.u16 --kit train-places.json --asks asks.json \
//!                   --rule tokens|frequent [--panel NAME=PANEL/tokens.u16,panel-places.json]... \
//!                   [--out with-floors.json]
//! place-asks experts --tokens TRAIN/tokens.u16 --kit train-places.json --out pairs.uorpex
//! place-asks experts-check --tokens TRAIN/tokens.u16 --kit train-places.json --experts pairs.uorpex \
//!                   --asks asks.json --prompts samples.json --rows rows.tsv --hold-out 0|1 \
//!                   --caps 300,2000 --keeps 1/2,1/4 --floors 0,1500 [--out PREFIX]
//! ```
//!
//! `experts` (claude-367) counts the pair experts' table on a split: each reply the place
//! register sees answer a user turn, taken up to (not including) its end mark, with that turn's
//! content. A reply that meets a start mark or the end of the tokens first is dropped, as is an
//! empty turn or reply. The table is tagged with the kit's tokens and never written over a file.
//!
//! `experts-check` (claude-367) replays a panel's finished replies through the one check the
//! served chat applies, under each cap and keep share of the grid, and prints how many general
//! replies end and are the reference or a reply their exact turn was counted with, before and
//! after the check. The table must be counted on `--tokens`; the asks are read whole, as the
//! served model reads them. `--rows` holds six columns: the prompt's index in `--prompts`, the
//! kind (general, recall, nevertold or repeat), the last user turn's ids, the reply's ids without
//! its end mark, whether it ends (1 or 0) and the reference's ids. With one cap, one keep and
//! `--out` it writes `PREFIX.jsonl`, each final reply's ids with the end mark when it ends, and
//! `PREFIX.rows.tsv`, the rows with the final reply and three more columns: the verdict, the
//! reply's vote and the top vote (0 and 0 when the experts were not asked).
//!
//! `repeat` (claude-364) counts the lead-in pairs on the tokens a written table was fitted on:
//! the pairs of neighbouring tokens in a user turn after which the rest of the turn is the
//! reply. It prints each pair it counted at least four times and writes a copy of the table
//! with the kept pairs; the table it read is left as it is.
//!
//! `--open-reply-start 1` (claude-362) sets the pair of the model marker's last two tokens open
//! in the fitted table, before the table is counted or written, and prints what the pair was: a
//! reply then starts on the value its asks find. 0, the default, leaves the fit's table as it is.
//! `census` reads a written table and takes only 0.
//!
//! `forms` (claude-365) counts a floor for each question form on `--tokens`, a written table's
//! own split: under `tokens` a form is a question's distinct tokens, under `frequent` those of
//! them that stand in at least 64 of the split's questions. It prints the count, then the census
//! of that split (`fitted`) and of each panel twice: with the table as it is (`one:`) and with
//! the form floors (`forms:`). Each panel is read with its own kit. With `--out` it writes a copy
//! of the table that holds the floors: under `tokens` only, only when `--tokens` are the tokens
//! the table was fitted on, never over a file, and never from a table that already holds floors.
//! Without `--out` it writes nothing; the table it read is left as it is.
//!
//! `ablate` (claude-360) fits under a cut, which keeps the named frame offsets (1 to 5 before a
//! place, nearest first; 6 to 8 the turn's last tokens, the last first) and the last N tokens
//! of a question, and prints the census of each held split under the same cut. It writes no
//! table: a table file never carries a cut.
//!
//! A token file is the `UORT` format the trainer reads: a 64-byte header holding the token count
//! and the vocabulary size, then the tokens as little-endian 16-bit integers. A kit is the place
//! kit the turn census writes: the marks and each recall reply's `[reply start, place, value
//! start, value length]`. Nothing here reads a model; every number is a count over tokens.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Deserialize;
use sha2::{Digest, Sha256};
use uor_r4_integer::fbs_snapshot::PlaceMarks;
use uor_r4_integer::pair_experts::{asked_turn, Check, Occurrence, PairExperts, Ratio, Setting, Verdict};
use uor_r4_integer::place_asks::{ask_cases, census, fit, fit_cut, fit_forms, fit_repeat, AsksIndex, Cut, FormRule, PlaceAsks, ASKS_FRAME};

const HEADER: usize = 64;

#[derive(Deserialize)]
struct Kit {
    tokens_sha256: String,
    marks: PlaceMarks,
    places: Vec<[u32; 4]>,
}

/// A token file's tokens, its vocabulary size and the SHA-256 of its bytes; `path` names it in
/// an error.
fn parse_tokens(bytes: &[u8], path: &str) -> Result<(Vec<u32>, u32, String), String> {
    if bytes.len() < HEADER || &bytes[..4] != b"UORT" {
        return Err(format!("{path}: not a UORT token file"));
    }
    let word = |at: usize| u64::from_le_bytes(bytes[at..at + 8].try_into().expect("eight bytes"));
    let (count, vocab) = (word(8), word(16));
    let payload = &bytes[HEADER..];
    if count.checked_mul(2) != Some(payload.len() as u64) {
        return Err(format!("{path}: the header counts {count} tokens and the file holds {} bytes of them", payload.len()));
    }
    let tokens = payload.chunks_exact(2).map(|pair| u32::from(u16::from_le_bytes([pair[0], pair[1]]))).collect();
    let vocab = u32::try_from(vocab).map_err(|_| format!("{path}: a vocabulary of {vocab}"))?;
    Ok((tokens, vocab, hex::encode(Sha256::digest(bytes))))
}

/// The kit's places are positions in one token file: another file's counts would be printed
/// under this one's name.
fn same_tokens(kit_sha256: &str, tokens_sha256: &str) -> Result<(), String> {
    if kit_sha256 == tokens_sha256 {
        return Ok(());
    }
    Err(format!("the kit was written from tokens {kit_sha256} and --tokens holds {tokens_sha256}"))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn flag(args: &[String], name: &str) -> Result<PathBuf, String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(PathBuf::from).ok_or_else(|| format!("{name} PATH is required"))
}

/// Every value of a flag that may be given more than once, in order.
fn flags(args: &[String], name: &str) -> Vec<PathBuf> {
    args.windows(2).filter(|pair| pair[0] == name).map(|pair| PathBuf::from(&pair[1])).collect()
}

/// A split: its tokens, its vocabulary size and the kit written from those tokens.
fn split(tokens: &Path, kit: &Path) -> Result<(Vec<u32>, u32, Kit), String> {
    let bytes = std::fs::read(tokens).map_err(|e| format!("{}: {e}", tokens.display()))?;
    let (tokens, vocab, tokens_sha256) = parse_tokens(&bytes, &tokens.display().to_string())?;
    let kit: Kit = read_json(kit)?;
    same_tokens(&kit.tokens_sha256, &tokens_sha256)?;
    Ok((tokens, vocab, kit))
}

/// The cut of `--slots` and `--question-last`: every offset and the whole question when a flag
/// is not given.
fn cut(args: &[String]) -> Result<Cut, String> {
    let value = |name: &str| args.iter().position(|a| a == name).map(|i| args.get(i + 1).map(String::as_str).ok_or_else(|| format!("{name} takes a value")));
    let mut cut = Cut::ALL;
    if let Some(text) = value("--slots") {
        let text = text?;
        cut.slots = 0;
        // `none` is the whole value or an error: `none,6` would run a cut nobody named.
        for part in text.split(',').filter(|_| text != "none") {
            let offset: usize = part.parse().ok().filter(|n| (1..=ASKS_FRAME).contains(n)).ok_or_else(|| format!("--slots {part}: an offset is 1 to {ASKS_FRAME}, or the one word none"))?;
            let bit = 1 << (offset - 1);
            if cut.slots & bit != 0 {
                return Err(format!("--slots names offset {offset} twice"));
            }
            cut.slots |= bit;
        }
    }
    if let Some(text) = value("--question-last") {
        cut.question_last = text?.parse().map_err(|_| "--question-last N".to_owned())?;
    }
    Ok(cut)
}

/// The held splits of `ablate`: each `--held` with the `--held-kit` at the same place.
fn held(args: &[String]) -> Result<Vec<(PathBuf, PathBuf)>, String> {
    let (tokens, kits) = (flags(args, "--held"), flags(args, "--held-kit"));
    if tokens.is_empty() || tokens.len() != kits.len() {
        return Err("ablate takes one --held-kit for each --held, and at least one".to_owned());
    }
    Ok(tokens.into_iter().zip(kits).collect())
}

/// A held split is counted with the fitted split's marks and vocabulary, or not at all.
fn same_marks(fitted: (&PlaceMarks, u32), held: (&PlaceMarks, u32), kit: &Path) -> Result<(), String> {
    if fitted == held {
        return Ok(());
    }
    Err(format!("{}: other marks or another vocabulary than the fitted split's", kit.display()))
}

/// The offsets a cut keeps, as `--slots` takes them.
fn slots(cut: Cut) -> String {
    let kept: Vec<String> = (1..=ASKS_FRAME).filter(|j| cut.slots & (1 << (j - 1)) != 0).map(|j| j.to_string()).collect();
    if kept.is_empty() {
        "none".to_owned()
    } else {
        kept.join(",")
    }
}

/// Whether `--open-reply-start` is 1; 0 when it is not given.
fn open_reply_start(args: &[String]) -> Result<bool, String> {
    match args.iter().position(|a| a == "--open-reply-start").map(|i| args.get(i + 1).map(String::as_str)) {
        None => Ok(false),
        Some(Some("0")) => Ok(false),
        Some(Some("1")) => Ok(true),
        Some(_) => Err("--open-reply-start takes 0 or 1".to_owned()),
    }
}

/// With `open`, set the fitted table's reply start pair open and print what it was.
fn open_start(asks: &mut PlaceAsks, open: bool) -> Result<(), String> {
    if !open {
        return Ok(());
    }
    let was = match asks.open_reply_start()? {
        Some(true) => "open",
        Some(false) => "not_open",
        None => "not_counted",
    };
    let pair = &asks.marks.assistant[asks.marks.assistant.len() - 2..];
    println!("asks-fit open_reply_start before={} token={} was={was}", pair[0], pair[1]);
    Ok(())
}

/// The 32 bytes a kit's `tokens_sha256` names.
fn tokens_tag(text: &str) -> Result<[u8; 32], String> {
    hex::decode(text).ok().and_then(|bytes| <[u8; 32]>::try_from(bytes).ok()).ok_or_else(|| format!("tokens_sha256 {text}: not 32 bytes of hex"))
}

/// The cases `ask_cases` finds and the training replies `experts` counts: each case's turn
/// content and its reply up to, not including, the end mark. A reply that meets a start mark or
/// the end of the tokens before its end mark is dropped, as is an empty turn or reply.
fn occurrences<'a>(tokens: &'a [u32], marks: &PlaceMarks) -> (usize, Vec<Occurrence<'a>>) {
    let cases = ask_cases(tokens, marks);
    let mut kept = Vec::new();
    for case in &cases {
        let turn = &tokens[case.asked.clone()];
        let rest = &tokens[case.reply..];
        let Some(end) = rest.iter().position(|&x| x == marks.eos || x == marks.bos) else {
            continue;
        };
        let reply = &rest[..end];
        if rest[end] != marks.eos || turn.is_empty() || reply.is_empty() {
            continue;
        }
        kept.push(Occurrence { turn, reply });
    }
    (cases.len(), kept)
}

/// The kind of a panel row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    General,
    Recall,
    NeverTold,
    Repeat,
}

/// One line of `--rows`: a panel prompt's finished reply.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    /// The six columns as read.
    columns: Vec<String>,
    i: usize,
    kind: Kind,
    turn: Vec<u32>,
    reply: Vec<u32>,
    ends: bool,
    reference: Vec<u32>,
}

fn parse_ids(text: &str, what: &str) -> Result<Vec<u32>, String> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    text.split(' ').map(|id| id.parse().map_err(|_| format!("{what}: {id:?} is not a token id"))).collect()
}

/// Line `n` (from 1) of `--rows`.
fn parse_row(line: &str, n: usize) -> Result<Row, String> {
    let columns: Vec<&str> = line.split('\t').collect();
    if columns.len() != 6 {
        return Err(format!("rows line {n}: {} columns, not 6", columns.len()));
    }
    let what = |c: usize| format!("rows line {n} column {c}");
    let i = columns[0].parse().map_err(|_| format!("{}: {:?} is not a prompt index", what(1), columns[0]))?;
    let kind = match columns[1] {
        "general" => Kind::General,
        "recall" => Kind::Recall,
        "nevertold" => Kind::NeverTold,
        "repeat" => Kind::Repeat,
        other => return Err(format!("{}: kind {other:?} is not general, recall, nevertold or repeat", what(2))),
    };
    let ends = match columns[4] {
        "1" => true,
        "0" => false,
        other => return Err(format!("{}: ends {other:?} is not 1 or 0", what(5))),
    };
    Ok(Row {
        i,
        kind,
        turn: parse_ids(columns[2], &what(3))?,
        reply: parse_ids(columns[3], &what(4))?,
        ends,
        reference: parse_ids(columns[5], &what(6))?,
        columns: columns.iter().map(|c| (*c).to_owned()).collect(),
    })
}

/// Every row names a prompt, and where the prompt opens a model turn its last user turn is the
/// row's turn column: the first row that breaks either is refused.
fn check_rows(rows: &[Row], prompts: &[Vec<u32>], marks: &PlaceMarks) -> Result<(), String> {
    let mut named = std::collections::BTreeSet::new();
    for (n, row) in rows.iter().enumerate() {
        let prompt = prompts.get(row.i).ok_or_else(|| format!("rows line {}: prompt {} of {}", n + 1, row.i, prompts.len()))?;
        if !named.insert(row.i) {
            return Err(format!("rows line {}: prompt {} is named twice", n + 1, row.i));
        }
        if let Some((asked, _)) = asked_turn(prompt, marks) {
            if prompt[asked] != row.turn[..] {
                return Err(format!("rows line {} (prompt {}): the turn column is not the prompt's last user turn", n + 1, row.i));
            }
        }
    }
    Ok(())
}

/// The reply a row ends with under a verdict: the table's top reply, which ends, when the
/// experts replace the row's reply, and otherwise the row's own.
fn final_reply(table: &PairExperts, verdict: Verdict, reply: &[u32], ends: bool) -> (Vec<u32>, bool) {
    match verdict {
        Verdict::Checked(Check::Replaced { reply: r, .. }) => (table.reply(r).expect("a replacing reply is one of the table's").to_vec(), true),
        _ => (reply.to_vec(), ends),
    }
}

fn verdict_name(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::NoTurn => "noturn",
        Verdict::LeftAlone => "alone",
        Verdict::Checked(Check::NoExpert) => "noexpert",
        Verdict::Checked(Check::Stands { .. }) => "stands",
        Verdict::Checked(Check::Unsure { .. }) => "unsure",
        Verdict::Checked(Check::Replaced { .. }) => "replaced",
        Verdict::NoRoom => "noroom",
    }
}

/// The reply's vote and the top vote of a checked row; 0 and 0 when the experts were not asked
/// or the turn has none.
fn votes_of(verdict: Verdict) -> (u64, u64) {
    match verdict {
        Verdict::Checked(Check::Stands { vote, top }) | Verdict::Checked(Check::Unsure { vote, top }) | Verdict::Checked(Check::Replaced { vote, top, .. }) => (vote, top),
        _ => (0, 0),
    }
}

/// A general reply is good when it ends and is the reference or a reply its exact turn was
/// counted with.
fn good(table: &PairExperts, row: &Row, reply: &[u32], ends: bool) -> bool {
    ends && (reply == row.reference.as_slice() || table.turn_holds(&row.turn, reply))
}

/// What one setting of the grid does to the rows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Tally {
    general: usize,
    good_before: usize,
    good_after: usize,
    newly_good: usize,
    lost: usize,
    noturn: usize,
    alone: usize,
    noexpert: usize,
    stands: usize,
    unsure: usize,
    replaced: usize,
    recall_replaced: usize,
    nevertold_replaced: usize,
    repeat_replaced: usize,
}

const GRID_HEADER: &str = "cap\tkeep\tfloor\tgeneral\tgood_before\tgood_after\tnewly_good\tlost\tnoturn\talone\tnoexpert\tstands\tunsure\treplaced\trecall_replaced\tnevertold_replaced\trepeat_replaced";

impl Tally {
    fn line(&self, setting: Setting) -> String {
        let counts = [
            self.general,
            self.good_before,
            self.good_after,
            self.newly_good,
            self.lost,
            self.noturn,
            self.alone,
            self.noexpert,
            self.stands,
            self.unsure,
            self.replaced,
            self.recall_replaced,
            self.nevertold_replaced,
            self.repeat_replaced,
        ];
        let counts: Vec<String> = counts.iter().map(usize::to_string).collect();
        format!("{}\t{}/{}\t{}\t{}", setting.cap, setting.keep.num, setting.keep.den, setting.floor, counts.join("\t"))
    }
}

/// One row's verdict and the reply it ends with.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Outcome {
    verdict: Verdict,
    reply: Vec<u32>,
    ends: bool,
}

/// Replays every row through the check under one setting. The rows were checked against the
/// prompts first (`check_rows`).
fn replay(table: &PairExperts, asks: Option<&AsksIndex>, marks: &PlaceMarks, prompts: &[Vec<u32>], rows: &[Row], setting: Setting) -> (Tally, Vec<Outcome>) {
    let mut tally = Tally::default();
    let mut outcomes = Vec::with_capacity(rows.len());
    for row in rows {
        let verdict = table.check_prompt(asks, marks, &prompts[row.i], &row.reply, setting);
        let (reply, ends) = final_reply(table, verdict, &row.reply, row.ends);
        let replaced = matches!(verdict, Verdict::Checked(Check::Replaced { .. }));
        match row.kind {
            Kind::General => {
                tally.general += 1;
                let before = good(table, row, &row.reply, row.ends);
                let after = good(table, row, &reply, ends);
                tally.good_before += usize::from(before);
                tally.good_after += usize::from(after);
                tally.newly_good += usize::from(after && !before);
                tally.lost += usize::from(before && !after);
                match verdict {
                    Verdict::NoTurn => tally.noturn += 1,
                    Verdict::LeftAlone => tally.alone += 1,
                    Verdict::Checked(Check::NoExpert) => tally.noexpert += 1,
                    Verdict::Checked(Check::Stands { .. }) | Verdict::NoRoom => tally.stands += 1,
                    Verdict::Checked(Check::Unsure { .. }) => tally.unsure += 1,
                    Verdict::Checked(Check::Replaced { .. }) => tally.replaced += 1,
                }
            }
            Kind::Recall => tally.recall_replaced += usize::from(replaced),
            Kind::NeverTold => tally.nevertold_replaced += usize::from(replaced),
            Kind::Repeat => tally.repeat_replaced += usize::from(replaced),
        }
        outcomes.push(Outcome { verdict, reply, ends });
    }
    (tally, outcomes)
}

/// The text of `PREFIX.jsonl` and `PREFIX.rows.tsv` for one setting's outcomes.
fn outputs(rows: &[Row], outcomes: &[Outcome], eos: u32) -> (String, String) {
    let (mut jsonl, mut tsv) = (String::new(), String::new());
    for (row, outcome) in rows.iter().zip(outcomes) {
        let mut ids: Vec<String> = outcome.reply.iter().map(u32::to_string).collect();
        let reply = ids.join(" ");
        if outcome.ends {
            ids.push(eos.to_string());
        }
        jsonl.push_str(&format!("{{\"i\":{},\"ids\":[{}]}}\n", row.i, ids.join(",")));
        let ends = if outcome.ends { "1" } else { "0" };
        let c = &row.columns;
        let (vote, top) = votes_of(outcome.verdict);
        tsv.push_str(&format!("{}\t{}\t{}\t{reply}\t{ends}\t{}\t{}\t{vote}\t{top}\n", c[0], c[1], c[2], c[5], verdict_name(outcome.verdict)));
    }
    (jsonl, tsv)
}

/// The grid of `experts-check`: `--caps`, `--keeps`, `--floors` and `--hold-out`. A floor is in
/// thousandths of one expert's whole vote.
fn grid(args: &[String]) -> Result<(Vec<u32>, Vec<Ratio>, Vec<u32>, bool), String> {
    let value = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(String::as_str).ok_or_else(|| format!("{name} takes a value"));
    let caps = value("--caps")?.split(',').map(|cap| cap.parse().map_err(|_| format!("--caps {cap}: not a count"))).collect::<Result<Vec<u32>, String>>()?;
    let keeps = value("--keeps")?
        .split(',')
        .map(|keep| {
            let (num, den) = keep.split_once('/').ok_or_else(|| format!("--keeps {keep}: NUM/DEN"))?;
            match (num.parse(), den.parse()) {
                (Ok(num), Ok(den)) if den > 0 && num <= den => Ok(Ratio { num, den }),
                _ => Err(format!("--keeps {keep}: NUM/DEN with DEN at least 1 and NUM at most DEN")),
            }
        })
        .collect::<Result<Vec<Ratio>, String>>()?;
    let hold_out = match value("--hold-out")? {
        "0" => false,
        "1" => true,
        other => return Err(format!("--hold-out {other}: 0 or 1")),
    };
    let floors = value("--floors")?.split(',').map(|floor| floor.parse().map_err(|_| format!("--floors {floor}: not a count of thousandths"))).collect::<Result<Vec<u32>, String>>()?;
    Ok((caps, keeps, floors, hold_out))
}

/// The file made from `path` with `suffix` added, refused when one is there.
fn new_file(path: &Path, suffix: &str) -> Result<PathBuf, String> {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    let path = PathBuf::from(name);
    if path.exists() {
        return Err(format!("{}: exists; a replay never writes over a file", path.display()));
    }
    Ok(path)
}

fn write_new(path: &Path, text: &[u8]) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(path).map_err(|e| format!("{}: {e}; a file is never written over", path.display()))?;
    file.write_all(text).map_err(|e| format!("{}: {e}", path.display()))
}

/// A panel sample; keys other than the prompt are not read.
#[derive(Deserialize)]
struct Sample {
    prompt: Vec<u32>,
}

fn run(args: &[String]) -> Result<(), String> {
    let mode = args.first().map(String::as_str).ok_or("usage: place-asks fit|census|ablate|repeat|forms|experts|experts-check ...")?;
    let open = open_reply_start(args)?;
    let (tokens, vocab, kit) = split(&flag(args, "--tokens")?, &flag(args, "--kit")?)?;
    match mode {
        "ablate" => {
            let cut = cut(args)?;
            let held = held(args)?;
            let (mut asks, _) = fit_cut(&tokens, &kit.marks, vocab, &kit.places, kit.tokens_sha256.clone(), cut)?;
            open_start(&mut asks, open)?;
            let index = AsksIndex::new(&asks)?.with_cut(cut);
            println!(
                "asks-ablate slots={} question_last={} fitted_on={} shared={} pairs={} margin={} floor={} floor_other={} asking_floor={}",
                slots(cut),
                cut.question_last,
                asks.tokens_sha256,
                asks.shared.len(),
                asks.pairs.len(),
                asks.margin,
                asks.floor,
                asks.floor_other,
                asks.asking_floor
            );
            for (n, (tokens, kit_path)) in held.iter().enumerate() {
                let (tokens, held_vocab, held_kit) = split(tokens, kit_path)?;
                same_marks((&kit.marks, vocab), (&held_kit.marks, held_vocab), kit_path)?;
                let (counts, _) = census(&tokens, &held_kit.marks, &held_kit.places, &index, &asks.forms, 0);
                for line in counts.lines(&format!("held{}", n + 1)) {
                    println!("{line}");
                }
            }
        }
        "fit" => {
            let out = flag(args, "--out")?;
            if out.exists() {
                return Err(format!("{}: exists; a fit never writes over a table", out.display()));
            }
            let (mut asks, lines) = fit(&tokens, &kit.marks, vocab, &kit.places, kit.tokens_sha256.clone())?;
            for line in &lines {
                println!("{line}");
            }
            open_start(&mut asks, open)?;
            let index = AsksIndex::new(&asks)?;
            let (counts, _) = census(&tokens, &kit.marks, &kit.places, &index, &asks.forms, 0);
            for line in counts.lines("fitted") {
                println!("{line}");
            }
            let text = serde_json::to_string(&asks).map_err(|e| e.to_string())?;
            // The file is made here and only if no file is there: a table written since the
            // check above is not written over.
            let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&out).map_err(|e| format!("{}: {e}; a fit never writes over a table", out.display()))?;
            file.write_all(text.as_bytes()).map_err(|e| format!("{}: {e}", out.display()))?;
            println!("asks-fit wrote {} fitted_on={}", out.display(), asks.tokens_sha256);
        }
        "census" => {
            if open {
                return Err("census reads a written table: --open-reply-start 1 is for fit or ablate".to_owned());
            }
            let asks: PlaceAsks = read_json(&flag(args, "--asks")?)?;
            if asks.marks != kit.marks || asks.vocab != vocab {
                return Err("the asks were fitted with other marks or another vocabulary than this split's".to_owned());
            }
            let show = match args.iter().position(|a| a == "--show") {
                Some(i) => args.get(i + 1).and_then(|n| n.parse().ok()).ok_or("--show N")?,
                None => 0,
            };
            let index = AsksIndex::new(&asks)?;
            let (counts, missed) = census(&tokens, &kit.marks, &kit.places, &index, &asks.forms, show);
            println!(
                "asks-census tokens={} fitted_on={} margin={} floor={} floor_other={} asking_floor={}",
                kit.tokens_sha256, asks.tokens_sha256, asks.margin, asks.floor, asks.floor_other, asks.asking_floor
            );
            for line in counts.lines("held") {
                println!("{line}");
            }
            for line in missed {
                println!("{line}");
            }
        }
        "repeat" => {
            if open {
                return Err("repeat reads a written table: --open-reply-start 1 is for fit or ablate".to_owned());
            }
            let out = flag(args, "--out")?;
            if out.exists() {
                return Err(format!("{}: exists; a table is never written over", out.display()));
            }
            let mut asks: PlaceAsks = read_json(&flag(args, "--asks")?)?;
            if asks.marks != kit.marks || asks.vocab != vocab {
                return Err("the asks were fitted with other marks or another vocabulary than this split's".to_owned());
            }
            // The pairs are counted on the tokens the table was fitted on, or not at all.
            if asks.tokens_sha256 != kit.tokens_sha256 {
                return Err(format!("the asks were fitted on tokens {} and --tokens holds {}", asks.tokens_sha256, kit.tokens_sha256));
            }
            if !asks.repeat.is_empty() || !asks.repeat3.is_empty() {
                return Err(format!("the asks already hold {} lead-in pairs and {} triples", asks.repeat.len(), asks.repeat3.len()));
            }
            let (pairs, triples, lines) = fit_repeat(&tokens, &kit.marks, vocab)?;
            for line in &lines {
                println!("{line}");
            }
            asks.repeat = pairs;
            asks.repeat3 = triples;
            AsksIndex::new(&asks)?;
            let text = serde_json::to_string(&asks).map_err(|e| e.to_string())?;
            let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&out).map_err(|e| format!("{}: {e}; a table is never written over", out.display()))?;
            file.write_all(text.as_bytes()).map_err(|e| format!("{}: {e}", out.display()))?;
            println!("asks-repeat wrote {} pairs={} triples={} fitted_on={}", out.display(), asks.repeat.len(), asks.repeat3.len(), asks.tokens_sha256);
        }
        "forms" => {
            if open {
                return Err("forms reads a written table: --open-reply-start 1 is for fit or ablate".to_owned());
            }
            let mut asks: PlaceAsks = read_json(&flag(args, "--asks")?)?;
            if asks.marks != kit.marks || asks.vocab != vocab {
                return Err("the asks were fitted with other marks or another vocabulary than this split's".to_owned());
            }
            // The count reads the table's one floor: a table that holds form floors has none to add.
            if !asks.ask_floors.is_empty() {
                return Err(format!("the asks already hold {} question-form floors", asks.ask_floors.len()));
            }
            let out = match args.iter().position(|a| a == "--out") {
                Some(_) => Some(flag(args, "--out")?),
                None => None,
            };
            if let Some(out) = &out {
                if out.exists() {
                    return Err(format!("{}: exists; a table is never written over", out.display()));
                }
                // The floors are counted on the tokens the table was fitted on, or not written.
                if asks.tokens_sha256 != kit.tokens_sha256 {
                    return Err(format!("the asks were fitted on tokens {} and --tokens holds {}", asks.tokens_sha256, kit.tokens_sha256));
                }
            }
            let rule = match args.iter().position(|a| a == "--rule").and_then(|i| args.get(i + 1)).map(String::as_str) {
                Some("tokens") => FormRule::Tokens,
                Some("frequent") => FormRule::Frequent,
                _ => return Err("--rule takes tokens or frequent".to_owned()),
            };
            // Refused before any count: a table holds no list of kept tokens.
            if out.is_some() && matches!(rule, FormRule::Frequent) {
                return Err("--out writes the floors of --rule tokens: a table holds no list of kept tokens".to_owned());
            }
            // Each panel is read before any count, so a malformed one stops the run at once.
            let mut panels = Vec::new();
            for spec in args.windows(2).filter(|pair| pair[0] == "--panel").map(|pair| &pair[1]) {
                let (name, paths) = spec.split_once('=').ok_or_else(|| format!("--panel {spec}: NAME=TOKENS,KIT"))?;
                let (tokens_path, kit_path) = paths.split_once(',').ok_or_else(|| format!("--panel {spec}: NAME=TOKENS,KIT"))?;
                let (panel_tokens, panel_vocab, panel_kit) = split(Path::new(tokens_path), Path::new(kit_path))?;
                same_marks((&asks.marks, asks.vocab), (&panel_kit.marks, panel_vocab), Path::new(kit_path))?;
                panels.push((name.to_owned(), panel_tokens, panel_kit));
            }
            let index = AsksIndex::new(&asks)?;
            let (floors, lines) = fit_forms(&tokens, &kit.marks, &kit.places, &index, rule);
            for line in &lines {
                println!("{line}");
            }
            let rows = floors.rows();
            let with = index.clone().with_form_floors(floors);
            let splits = std::iter::once(("fitted".to_owned(), &tokens, &kit)).chain(panels.iter().map(|(name, t, k)| (name.clone(), t, k)));
            for (name, tokens, kit) in splits {
                for (prefix, reader) in [("one", &index), ("forms", &with)] {
                    let (counts, _) = census(tokens, &kit.marks, &kit.places, reader, &asks.forms, 0);
                    for line in counts.lines(&format!("{prefix}:{name}")) {
                        println!("{line}");
                    }
                }
            }
            if let Some(out) = out {
                asks.ask_floors = rows.ok_or("--out writes the floors of --rule tokens: a table holds no list of kept tokens")?;
                AsksIndex::new(&asks)?;
                let text = serde_json::to_string(&asks).map_err(|e| e.to_string())?;
                let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&out).map_err(|e| format!("{}: {e}; a table is never written over", out.display()))?;
                file.write_all(text.as_bytes()).map_err(|e| format!("{}: {e}", out.display()))?;
                println!("asks-forms wrote {} floors={} fitted_on={}", out.display(), asks.ask_floors.len(), asks.tokens_sha256);
            }
        }
        "experts" => {
            if open {
                return Err("experts reads no asks: --open-reply-start 1 is for fit or ablate".to_owned());
            }
            let out = flag(args, "--out")?;
            if out.exists() {
                return Err(format!("{}: exists; a table is never written over", out.display()));
            }
            let tag = tokens_tag(&kit.tokens_sha256)?;
            let (cases, kept) = occurrences(&tokens, &kit.marks);
            let table = PairExperts::count(vocab, tag, &kept)?;
            let bytes = table.to_bytes();
            write_new(&out, &bytes)?;
            println!(
                "pair-experts wrote {} cases={cases} dropped={} counted={} experts={} replies={} turns={} entries={} bytes={} sha256={} tokens={}",
                out.display(),
                cases - kept.len(),
                kept.len(),
                table.experts(),
                table.replies(),
                table.turns(),
                table.entries(),
                bytes.len(),
                table.sha256(),
                kit.tokens_sha256
            );
        }
        "experts-check" => {
            if open {
                return Err("experts-check reads a written table: --open-reply-start 1 is for fit or ablate".to_owned());
            }
            let path = flag(args, "--experts")?;
            let table = PairExperts::from_bytes(&std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?).map_err(|e| format!("{}: {e}", path.display()))?;
            table.expect(vocab, &tokens_tag(&kit.tokens_sha256)?)?;
            let asks: PlaceAsks = read_json(&flag(args, "--asks")?)?;
            if asks.marks != kit.marks || asks.vocab != vocab {
                return Err("the asks were fitted with other marks or another vocabulary than this split's".to_owned());
            }
            // As the served model reads a table (`set_place_asks`): the whole table, no cut.
            let index = AsksIndex::new(&asks)?;
            let samples: Vec<Sample> = read_json(&flag(args, "--prompts")?)?;
            let prompts: Vec<Vec<u32>> = samples.into_iter().map(|s| s.prompt).collect();
            let rows_path = flag(args, "--rows")?;
            let text = std::fs::read_to_string(&rows_path).map_err(|e| format!("{}: {e}", rows_path.display()))?;
            let rows = text.lines().enumerate().map(|(n, line)| parse_row(line, n + 1)).collect::<Result<Vec<Row>, String>>().map_err(|e| format!("{}: {e}", rows_path.display()))?;
            check_rows(&rows, &prompts, &kit.marks).map_err(|e| format!("{}: {e}", rows_path.display()))?;
            let (caps, keeps, floors, hold_out) = grid(args)?;
            let out = match args.iter().position(|a| a == "--out") {
                Some(_) => {
                    if caps.len() != 1 || keeps.len() != 1 || floors.len() != 1 {
                        return Err("--out writes one setting's replies: give one cap, one keep and one floor".to_owned());
                    }
                    let prefix = flag(args, "--out")?;
                    Some((new_file(&prefix, ".jsonl")?, new_file(&prefix, ".rows.tsv")?))
                }
                None => None,
            };
            println!("{GRID_HEADER}");
            for &cap in &caps {
                for &keep in &keeps {
                    for &floor in &floors {
                        let setting = Setting { cap, keep, hold_out, floor };
                        let (tally, outcomes) = replay(&table, Some(&index), &kit.marks, &prompts, &rows, setting);
                        println!("{}", tally.line(setting));
                        if let Some((jsonl_path, tsv_path)) = &out {
                            let (jsonl, tsv) = outputs(&rows, &outcomes, kit.marks.eos);
                            write_new(jsonl_path, jsonl.as_bytes())?;
                            write_new(tsv_path, tsv.as_bytes())?;
                        }
                    }
                }
            }
        }
        other => return Err(format!("unknown mode {other}: fit, census, ablate, repeat, forms, experts or experts-check")),
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("place-asks: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cut_is_read_from_its_flags_and_written_back_the_same() {
        let args = |text: &str| text.split(' ').map(str::to_owned).collect::<Vec<String>>();
        assert_eq!(cut(&args("ablate")), Ok(Cut::ALL));
        assert_eq!(cut(&args("ablate --slots 1,6,8 --question-last 2")), Ok(Cut { slots: 0b1010_0001, question_last: 2 }));
        assert_eq!(cut(&args("ablate --slots none")), Ok(Cut { slots: 0, question_last: 0 }));
        assert_eq!(slots(Cut { slots: 0b1010_0001, question_last: 0 }), "1,6,8");
        assert_eq!(slots(Cut { slots: 0, question_last: 0 }), "none");
        assert_eq!(slots(Cut::ALL), "1,2,3,4,5,6,7,8");
        for refused in ["ablate --slots 0", "ablate --slots 9", "ablate --slots 1,x", "ablate --slots", "ablate --slots none,6", "ablate --slots 1,1", "ablate --question-last x", "ablate --question-last"] {
            assert!(cut(&args(refused)).is_err(), "{refused}");
        }
        assert_eq!(flags(&args("ablate --held a --held-kit k --held b"), "--held"), vec![PathBuf::from("a"), PathBuf::from("b")]);
    }

    #[test]
    fn the_open_reply_start_flag_is_0_unless_given_1() {
        let args = |text: &str| text.split(' ').map(str::to_owned).collect::<Vec<String>>();
        assert_eq!(open_reply_start(&args("fit --out a")), Ok(false));
        assert_eq!(open_reply_start(&args("fit --open-reply-start 0")), Ok(false));
        assert_eq!(open_reply_start(&args("fit --open-reply-start 1")), Ok(true));
        for refused in ["fit --open-reply-start", "fit --open-reply-start 2", "fit --open-reply-start yes"] {
            assert!(open_reply_start(&args(refused)).is_err(), "{refused}");
        }
    }

    #[test]
    fn the_held_splits_come_in_pairs_with_the_marks_of_the_fitted_split() {
        let args = |text: &str| text.split(' ').map(str::to_owned).collect::<Vec<String>>();
        let pair = |tokens: &str, kit: &str| (PathBuf::from(tokens), PathBuf::from(kit));
        assert_eq!(held(&args("ablate --held a --held-kit ka --held b --held-kit kb")), Ok(vec![pair("a", "ka"), pair("b", "kb")]));
        assert!(held(&args("ablate")).is_err());
        assert!(held(&args("ablate --held a --held-kit ka --held b")).is_err());
        let marks = |user: u32| -> PlaceMarks { serde_json::from_str(&format!(r#"{{"bos":0,"eos":1,"sep":[2],"system":[3],"user":[{user}],"assistant":[5]}}"#)).expect("marks") };
        let kit = Path::new("kit.json");
        assert_eq!(same_marks((&marks(4), 9), (&marks(4), 9), kit), Ok(()));
        assert!(same_marks((&marks(4), 9), (&marks(6), 9), kit).is_err());
        assert!(same_marks((&marks(4), 9), (&marks(4), 10), kit).is_err());
    }

    /// A token file of `ids` whose header counts `count` tokens in a vocabulary of 9.
    fn file(count: u64, ids: &[u16]) -> Vec<u8> {
        let mut bytes = vec![0u8; HEADER];
        bytes[..4].copy_from_slice(b"UORT");
        bytes[8..16].copy_from_slice(&count.to_le_bytes());
        bytes[16..24].copy_from_slice(&9u64.to_le_bytes());
        bytes.extend(ids.iter().flat_map(|id| id.to_le_bytes()));
        bytes
    }

    #[test]
    fn a_token_file_is_read_with_the_hash_of_its_own_bytes() {
        let bytes = file(3, &[4, 5, 6]);
        let (tokens, vocab, sha256) = parse_tokens(&bytes, "t").expect("read");
        assert_eq!((tokens, vocab), (vec![4, 5, 6], 9));
        assert_eq!(sha256, hex::encode(Sha256::digest(&bytes)));
        // Another file's kit is refused, and this file's is taken.
        assert!(same_tokens(&sha256, &sha256).is_ok());
        assert!(same_tokens(&hex::encode(Sha256::digest(file(3, &[4, 5, 7]))), &sha256).is_err());
    }

    #[test]
    fn a_malformed_token_file_is_an_error_and_never_a_panic() {
        // A count that does not fit twice in 64 bits, a count that is not the file's, a file
        // shorter than its header, and a file that is not a token file.
        assert!(parse_tokens(&file(u64::MAX, &[4, 5, 6]), "t").is_err());
        assert!(parse_tokens(&file(1 << 63, &[]), "t").is_err());
        assert!(parse_tokens(&file(2, &[4, 5, 6]), "t").is_err());
        assert!(parse_tokens(&file(3, &[4, 5, 6])[..40], "t").is_err());
        let mut other = file(3, &[4, 5, 6]);
        other[0] = b'X';
        assert!(parse_tokens(&other, "t").is_err());
    }

    /// Start 0, end 1, separator 2, system 3, user 4, model 5; words from 6 in a vocabulary of 16.
    fn chat_marks() -> PlaceMarks {
        serde_json::from_str(r#"{"bos":0,"eos":1,"sep":[2],"system":[3],"user":[4],"assistant":[5]}"#).expect("marks")
    }

    #[test]
    fn experts_count_each_reply_up_to_its_end_mark_and_drop_a_reply_that_never_ends() {
        let m = chat_marks();
        let tokens = [
            0, 4, 6, 7, 2, 5, 8, 9, 1, 2, // a reply that ends: [8, 9]
            4, 7, 6, 2, 5, 10, // a reply that meets the next start mark
            0, 4, 6, 2, 5, 11, 1, 2, // a reply that ends: [11]
            4, 9, 2, 5, 12, // a reply that meets the end of the tokens
        ];
        let (cases, kept) = occurrences(&tokens, &m);
        assert_eq!(cases, 4);
        let pairs: Vec<(Vec<u32>, Vec<u32>)> = kept.iter().map(|o| (o.turn.to_vec(), o.reply.to_vec())).collect();
        assert_eq!(pairs, vec![(vec![6, 7], vec![8, 9]), (vec![6], vec![11])]);
        let table = PairExperts::count(16, [3; 32], &kept).expect("count");
        assert_eq!((table.turns(), table.replies(), table.experts()), (2, 2, 1));
        assert!(table.turn_holds(&[6, 7], &[8, 9]));
        assert!(table.turn_holds(&[6], &[11]));
        assert!(!table.turn_holds(&[6, 7], &[8, 9, 1]));
        assert!(!table.turn_holds(&[7, 6], &[10]));
        assert_eq!(tokens_tag(&hex::encode([3u8; 32])), Ok([3; 32]));
        assert!(tokens_tag("abc").is_err());
        assert!(tokens_tag(&hex::encode([3u8; 31])).is_err());
    }

    /// Turn [6, 7, 8] counted twice with [9]; turn [6, 7] once with [10].
    fn replay_table() -> PairExperts {
        let occurrences = [Occurrence { turn: &[6, 7, 8], reply: &[9] }, Occurrence { turn: &[6, 7, 8], reply: &[9] }, Occurrence { turn: &[6, 7], reply: &[10] }];
        PairExperts::count(16, [3; 32], &occurrences).expect("count")
    }

    /// Prompts 0, 2, 3 and 4 ask turn [6, 7, 8]; prompt 1 opens no model turn.
    fn replay_prompts() -> Vec<Vec<u32>> {
        let ask = vec![0, 4, 6, 7, 8, 2, 5];
        vec![ask.clone(), vec![0, 4, 6, 7], ask.clone(), ask.clone(), ask]
    }

    fn rows(lines: &[&str]) -> Vec<Row> {
        lines.iter().enumerate().map(|(n, line)| parse_row(line, n + 1).expect("row")).collect()
    }

    const KEEP_HALF: Setting = Setting { cap: 100, keep: Ratio { num: 1, den: 2 }, hold_out: false, floor: 0 };

    #[test]
    fn the_replay_replaces_a_reply_the_turns_experts_do_not_hold_and_counts_it_newly_good() {
        let (table, m, prompts) = (replay_table(), chat_marks(), replay_prompts());
        let rows = rows(&[
            "0\tgeneral\t6 7 8\t11\t1\t13", // replaced by [9], which the turn holds
            "2\tgeneral\t6 7 8\t9\t1\t9",   // good, and stands
            "1\tgeneral\t6 7\t10\t1\t10",   // no model turn: left as it is, good
            "3\trecall\t6 7 8\t11\t0\t12",  // replaced
            "4\tnevertold\t6 7 8\t9\t1\t9", // stands
        ]);
        assert_eq!(check_rows(&rows, &prompts, &m), Ok(()));
        let (tally, outcomes) = replay(&table, None, &m, &prompts, &rows, KEEP_HALF);
        let want = Tally { general: 3, good_before: 2, good_after: 3, newly_good: 1, lost: 0, noturn: 1, alone: 0, noexpert: 0, stands: 1, unsure: 0, replaced: 1, recall_replaced: 1, nevertold_replaced: 0, repeat_replaced: 0 };
        assert_eq!(tally, want);
        assert_eq!(tally.general, tally.noturn + tally.alone + tally.noexpert + tally.stands + tally.unsure + tally.replaced);
        assert_eq!(tally.good_after + tally.lost, tally.good_before + tally.newly_good);
        assert_eq!((outcomes[0].reply.clone(), outcomes[0].ends), (vec![9], true));
        assert_eq!((outcomes[2].reply.clone(), outcomes[2].verdict), (vec![10], Verdict::NoTurn));
        assert_eq!((outcomes[3].reply.clone(), outcomes[3].ends), (vec![9], true));
        let (jsonl, tsv) = outputs(&rows, &outcomes, m.eos);
        assert_eq!(jsonl.lines().next(), Some(r#"{"i":0,"ids":[9,1]}"#));
        assert_eq!(tsv.lines().next(), Some("0\tgeneral\t6 7 8\t9\t1\t13\treplaced\t0\t2796202"));
        assert_eq!(tsv.lines().nth(2), Some("1\tgeneral\t6 7\t10\t1\t10\tnoturn\t0\t0"));
        assert_eq!(tally.line(KEEP_HALF), "100\t1/2\t0\t3\t2\t3\t1\t0\t1\t0\t0\t1\t0\t1\t1\t0\t0");
    }

    #[test]
    fn hold_out_turns_a_standing_reply_counted_only_with_its_own_turn_into_a_replaced_one() {
        let (table, m, prompts) = (replay_table(), chat_marks(), replay_prompts());
        let rows = rows(&["0\tgeneral\t6 7 8\t9\t1\t9"]);
        let (_, kept) = replay(&table, None, &m, &prompts, &rows, KEEP_HALF);
        assert!(matches!(kept[0].verdict, Verdict::Checked(Check::Stands { .. })));
        // With the turn's own two counts out, only turn [6, 7]'s reply [10] is left to vote.
        let (tally, held) = replay(&table, None, &m, &prompts, &rows, Setting { hold_out: true, ..KEEP_HALF });
        assert!(matches!(held[0].verdict, Verdict::Checked(Check::Replaced { .. })));
        assert_eq!(held[0].reply, vec![10]);
        assert_eq!((tally.replaced, tally.good_before, tally.lost), (1, 1, 1));
    }

    #[test]
    fn a_row_whose_turn_is_not_its_prompts_last_user_turn_is_refused() {
        let (m, prompts) = (chat_marks(), replay_prompts());
        let refused = check_rows(&rows(&["1\tgeneral\t6 7\t10\t1\t10", "0\tgeneral\t6 7\t9\t1\t9", "0\tgeneral\t6 8\t9\t1\t9"]), &prompts, &m).expect_err("refused");
        assert!(refused.contains("rows line 2 (prompt 0)"), "{refused}");
        assert!(check_rows(&rows(&["5\tgeneral\t6 7\t10\t1\t10"]), &prompts, &m).is_err());
        let twice = check_rows(&rows(&["0\tgeneral\t6 7 8\t9\t1\t9", "0\tgeneral\t6 7 8\t9\t1\t9"]), &prompts, &m).expect_err("refused");
        assert!(twice.contains("rows line 2: prompt 0 is named twice"), "{twice}");
        for bad in ["0\tgeneral\t6 7 8\t9\t1", "0\tother\t6 7 8\t9\t1\t9", "0\tgeneral\t6 7 8\t9\t2\t9", "0\tgeneral\t6 x\t9\t1\t9", "x\tgeneral\t6\t9\t1\t9"] {
            assert!(parse_row(bad, 1).is_err(), "{bad}");
        }
    }
}
