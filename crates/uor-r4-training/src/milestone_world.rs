//! M-world v1 (Lab 1): a typed-intent dialogue world for the §8 milestone's
//! three categories (responsive multi-turn, updated relation and new simple
//! instructions), with a frozen answer oracle and a train/development split of
//! the user phrasings.
//!
//! Every user turn comes from a typed intent: the intent fixes the reply the
//! world trains on and the frozen checks that judge any reply. Unlike
//! `uor_r4_core::answer_oracle`'s exact-string lists, several checks read the
//! reply (numbers, a letter, word counts, echo). Each intent's phrasings are
//! split once, here:
//! training renders only the `train` phrasings, development only the
//! `development` ones, so development measures phrasing generalization within
//! the milestone's intent types. The qualification panel's phrasings are not in
//! this file; ROADMAP §8 has them authored separately and sealed before the
//! final fit. Relation values also split (names, pets, cities), so a
//! development recall cannot be a memorized training pair.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::stack_tracking::Rng;
use crate::{invalid, Result};

/// Which phrasings (and relation values) a conversation draws from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Split {
    Train,
    Development,
}

/// The milestone category a turn is scored under.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Category {
    Responsive,
    Relation,
    Instruction,
}

/// One membership check of the frozen answer oracle. A reply passes when every
/// check of its turn holds. Words are compared lowercased, split at anything
/// that is not a letter, digit or apostrophe; a phrase matches consecutive
/// words.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Check {
    /// At least one of these words or phrases.
    AnyOf(Vec<String>),
    /// None of these words or phrases.
    NoneOf(Vec<String>),
    /// This literal text occurs.
    Literal(String),
    /// At least `at_least` distinct members of `set` occur.
    Members { set: Vec<String>, at_least: usize },
    /// The first number (digits or a number word up to twenty) is this one.
    FirstNumber(u32),
    /// The first word is this one.
    FirstWord(String),
    /// The last standalone single-letter word that does not begin a sentence
    /// is this uppercase letter, exactly, or the whole reply is that letter.
    /// Sentence-initial words are skipped so the article "A" and the pronoun
    /// "I" do not count.
    LastLetter(char),
    /// At least this many words.
    MinWords(usize),
    /// The reply does not contain the user's whole turn.
    NotEcho,
    /// Every one of these numbers occurs, as digits or a number word.
    Numbers(Vec<u32>),
    /// This number does not occur, as digits or a number word.
    NoNumber(u32),
    /// `answer` is the number the reply calls the larger one: "{answer} is
    /// bigger" or "the bigger one is {answer}", with no "than" before it; or
    /// the only number the reply names, without "smaller", "less" or "lower".
    Larger { answer: u32, other: u32 },
}

/// One user turn, the reply the world trains on, and the oracle's checks.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Turn {
    pub intent: String,
    pub category: Category,
    pub user: String,
    pub reply: String,
    pub checks: Vec<Check>,
}

/// A conversation of user turns and their replies, in order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conversation {
    pub turns: Vec<Turn>,
}

fn words(text: &str) -> Vec<String> {
    text.replace(['\u{2018}', '\u{2019}'], "'")
        .split(|c: char| !(c.is_alphanumeric() || c == '\''))
        .map(|w| w.trim_matches('\''))
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn contains_phrase(haystack: &[String], phrase: &str) -> bool {
    let needle = words(phrase);
    !needle.is_empty()
        && haystack.len() >= needle.len()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle.as_slice())
}

const NUMBER_WORDS: [&str; 21] = [
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
    "twenty",
];

fn number(word: &str) -> Option<u32> {
    word.parse::<u32>().ok().or_else(|| {
        NUMBER_WORDS
            .iter()
            .position(|&w| w == word)
            .map(|i| i as u32)
    })
}

/// The oracle: whether `reply` answers `user` under `checks`.
pub fn judge(checks: &[Check], user: &str, reply: &str) -> bool {
    let reply_words = words(reply);
    checks.iter().all(|check| match check {
        Check::AnyOf(options) => options.iter().any(|o| contains_phrase(&reply_words, o)),
        Check::NoneOf(options) => !options.iter().any(|o| contains_phrase(&reply_words, o)),
        Check::Literal(text) => reply.contains(text.as_str()),
        Check::Members { set, at_least } => {
            set.iter()
                .filter(|member| contains_phrase(&reply_words, member))
                .count()
                >= *at_least
        }
        Check::FirstNumber(value) => reply_words.iter().find_map(|w| number(w)) == Some(*value),
        Check::FirstWord(word) => reply_words.first().map(String::as_str) == Some(word.as_str()),
        Check::LastLetter(letter) => last_letter(reply) == Some(letter.to_ascii_uppercase()),
        Check::MinWords(n) => reply_words.len() >= *n,
        Check::NotEcho => {
            let user_words = words(user);
            user_words.is_empty() || !contains_phrase(&reply_words, &user_words.join(" "))
        }
        Check::Numbers(values) => values
            .iter()
            .all(|v| reply_words.iter().any(|w| number(w) == Some(*v))),
        Check::NoNumber(value) => !reply_words.iter().any(|w| number(w) == Some(*value)),
        Check::Larger { answer, other } => larger(&reply_words, *answer, *other),
    })
}

/// [`Check::Larger`]. Number words exclude "one", which in "the bigger one"
/// is a pronoun; the operands are written as digits.
fn larger(words: &[String], answer: u32, other: u32) -> bool {
    const MORE: [&str; 4] = ["bigger", "larger", "greater", "higher"];
    const LESS: [&str; 4] = ["smaller", "less", "lower", "fewer"];
    let value = |w: &str| if w == "one" { None } else { number(w) };
    let is = |w: &String, set: &[&str]| set.contains(&w.as_str());
    for (i, word) in words.iter().enumerate() {
        if value(word) != Some(answer) || (i > 0 && words[i - 1] == "than") {
            continue;
        }
        let after = &words[i + 1..(i + 4).min(words.len())];
        // "{answer} is bigger", up to the first "than".
        let stated_after = after
            .iter()
            .take_while(|w| w.as_str() != "than")
            .any(|w| is(w, &MORE));
        // "the bigger one is {answer}".
        let stated_before = i > 0
            && words[i - 1] == "is"
            && words[i.saturating_sub(4)..i - 1]
                .iter()
                .any(|w| is(w, &MORE));
        let negated = after.iter().any(|w| is(w, &LESS));
        if (stated_after || stated_before) && !negated {
            return true;
        }
    }
    let named: BTreeSet<u32> = words.iter().filter_map(|w| value(w)).collect();
    named.contains(&answer) && !named.contains(&other) && !words.iter().any(|w| is(w, &LESS))
}

/// The letter `Check::LastLetter` compares: the last standalone single-letter
/// word, in its original case, that does not begin a sentence; or the reply's
/// only word when it is a single letter.
fn last_letter(reply: &str) -> Option<char> {
    let mut sentence_start = true;
    let mut found = None;
    let mut count = 0usize;
    for token in reply.split_whitespace() {
        let word: String = token
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '\'')
            .collect();
        if !word.is_empty() {
            count += 1;
            let mut chars = word.chars();
            if let (Some(c), None) = (chars.next(), chars.next()) {
                if c.is_alphabetic() && (!sentence_start || count == 1) {
                    found = Some((c, sentence_start));
                }
            }
            sentence_start = false;
        }
        if token.ends_with(['.', '!', '?']) {
            sentence_start = true;
        }
    }
    match found {
        // A sentence-initial single letter counts only as the whole reply.
        Some((c, true)) if count == 1 => Some(c),
        Some((_, true)) => None,
        Some((c, false)) => Some(c),
        None => None,
    }
}

fn pick<'a, T>(rng: &mut Rng, items: &'a [T]) -> &'a T {
    &items[rng.below(items.len())]
}

/// Phrasings of one intent, split once.
struct Phrasings {
    train: &'static [&'static str],
    development: &'static [&'static str],
}

impl Phrasings {
    fn pick(&self, rng: &mut Rng, split: Split) -> &'static str {
        match split {
            Split::Train => pick(rng, self.train),
            Split::Development => pick(rng, self.development),
        }
    }
}

fn fill(template: &str, slots: &[(&str, &str)]) -> String {
    let mut text = template.to_owned();
    for (name, value) in slots {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

// ---------------------------------------------------------------------------
// Responsive intents.

struct Social {
    name: &'static str,
    user: Phrasings,
    replies: &'static [&'static str],
    accept: &'static [&'static str],
    question: bool,
}

const SOCIAL: &[Social] = &[
    Social {
        name: "greet",
        user: Phrasings {
            train: &[
                "Hi!",
                "Hello!",
                "Hey there!",
                "Hi there.",
                "Good morning!",
                "Good evening.",
                "Hey!",
                "Hello there, friend.",
                "Hiya!",
                "Greetings!",
            ],
            development: &["Howdy!", "Hey, hello!", "Good afternoon!", "Well hello there."],
        },
        replies: &[
            "Hello! How can I help you today?",
            "Hi there! What can I do for you?",
            "Hey! Nice to hear from you.",
        ],
        accept: &[
            "hello",
            "hi",
            "hey",
            "greetings",
            "good morning",
            "good afternoon",
            "good evening",
            "howdy",
        ],
        question: false,
    },
    Social {
        name: "how_are_you",
        user: Phrasings {
            train: &[
                "How are you?",
                "How are you doing?",
                "How's it going?",
                "How are you today?",
                "Are you doing well?",
                "How have you been?",
            ],
            development: &[
                "How are things with you?",
                "How do you feel today?",
                "Are you having a good day?",
            ],
        },
        replies: &[
            "I'm doing well, thank you! How about you?",
            "I'm good, thanks for asking.",
            "Pretty good, thank you!",
        ],
        accept: &["well", "good", "great", "fine"],
        question: false,
    },
    Social {
        name: "thanks",
        user: Phrasings {
            train: &[
                "Thank you!",
                "Thanks a lot.",
                "Thanks for the help.",
                "Thank you so much!",
                "Thanks!",
                "I appreciate it.",
            ],
            development: &["Many thanks!", "Cheers, that helped.", "Thanks a bunch!"],
        },
        replies: &["You're welcome!", "Happy to help!", "Anytime!", "Glad I could help."],
        accept: &["welcome", "happy to help", "glad", "anytime", "pleasure"],
        question: false,
    },
    Social {
        name: "farewell",
        user: Phrasings {
            train: &[
                "Bye!",
                "Goodbye.",
                "See you later!",
                "I have to go now. Bye!",
                "Talk to you later.",
                "Good night!",
            ],
            development: &["Catch you later!", "I'm heading out now.", "Farewell!"],
        },
        replies: &["Goodbye! Have a great day.", "Bye! Take care.", "See you later!"],
        accept: &["bye", "goodbye", "take care", "see you", "later", "good night", "farewell"],
        question: false,
    },
    Social {
        name: "identity",
        user: Phrasings {
            train: &[
                "Who are you?",
                "What are you?",
                "Are you a robot?",
                "Are you a person?",
                "Tell me about yourself.",
            ],
            development: &["Who am I talking to?", "What kind of thing are you?", "Introduce yourself."],
        },
        replies: &[
            "I am a small assistant. I can chat, remember what you tell me, and follow simple instructions.",
            "I'm a helpful assistant that runs on your computer.",
        ],
        accept: &["assistant"],
        question: false,
    },
    Social {
        name: "clarify",
        user: Phrasings {
            train: &[
                "Can you help me with it?",
                "What about that one?",
                "Can you fix it?",
                "Do the thing.",
                "Tell me more about it.",
                "Is it good?",
            ],
            development: &[
                "Could you sort that out for me?",
                "What do you think of it?",
                "Can you check this?",
            ],
        },
        replies: &[
            "Sure! What would you like help with?",
            "Could you tell me more about what you mean?",
            "Which one do you mean?",
        ],
        accept: &["what", "which", "more", "mean"],
        question: true,
    },
];

fn social(rng: &mut Rng, split: Split, intent: &Social) -> Turn {
    let mut checks = vec![Check::AnyOf(strings(intent.accept))];
    if intent.question {
        checks.push(Check::Literal("?".into()));
    }
    Turn {
        intent: intent.name.into(),
        category: Category::Responsive,
        user: intent.user.pick(rng, split).into(),
        reply: (*pick(rng, intent.replies)).into(),
        checks,
    }
}

const THINGS: &[&str] = &[
    "puppy", "kitten", "bike", "book", "guitar", "phone", "plant", "fish", "kite", "hat", "watch",
    "jacket", "scooter", "lamp", "backpack", "camera",
];

const SHARE_EVENT: Phrasings = Phrasings {
    train: &[
        "I just got a new {x}!",
        "Guess what, I bought a {x} today.",
        "My friend gave me a {x}.",
        "I finally have a {x}.",
        "Today I found a {x}.",
    ],
    development: &[
        "Look, I have a brand new {x}.",
        "Yesterday I picked up a {x}.",
        "Someone surprised me with a {x}.",
    ],
};

const SHARE_EVENT_REPLIES: &[&str] = &[
    "A new {x}! That sounds wonderful.",
    "Oh, a {x}! How exciting.",
    "That's great news about the {x}!",
];

const FEELINGS_GOOD: &[&str] = &["happy", "excited", "proud", "calm"];
const FEELINGS_BAD: &[&str] = &[
    "tired", "sad", "bored", "nervous", "hungry", "sleepy", "lonely",
];

const SHARE_FEELING: Phrasings = Phrasings {
    train: &[
        "I feel {x} today.",
        "I am so {x}.",
        "I'm feeling {x} right now.",
        "Today I am {x}.",
    ],
    development: &[
        "Honestly, I'm kind of {x}.",
        "I've been {x} all day.",
        "Right now I just feel {x}.",
    ],
};

fn share_event(rng: &mut Rng, split: Split) -> Turn {
    let thing = *pick(rng, THINGS);
    let slots = [("x", thing)];
    Turn {
        intent: "share_event".into(),
        category: Category::Responsive,
        user: fill(SHARE_EVENT.pick(rng, split), &slots),
        reply: fill(pick(rng, SHARE_EVENT_REPLIES), &slots),
        checks: vec![Check::AnyOf(vec![thing.into()]), Check::NotEcho],
    }
}

fn share_feeling(rng: &mut Rng, split: Split) -> Turn {
    let good = rng.below(3) == 0;
    let feeling = *pick(rng, if good { FEELINGS_GOOD } else { FEELINGS_BAD });
    let slots = [("x", feeling)];
    let replies: &[&str] = if good {
        &[
            "I'm glad you feel {x}!",
            "That's lovely, it's good to feel {x}.",
        ]
    } else {
        &[
            "I'm sorry you feel {x}. I hope it gets better soon.",
            "Feeling {x} is hard. Is there anything I can do?",
        ]
    };
    Turn {
        intent: "share_feeling".into(),
        category: Category::Responsive,
        user: fill(SHARE_FEELING.pick(rng, split), &slots),
        reply: fill(pick(rng, replies), &slots),
        checks: vec![Check::AnyOf(vec![feeling.into()]), Check::NotEcho],
    }
}

/// A closed knowledge base: each relation's pairs, question phrasings and
/// reply templates (`{x}` the subject, `{y}` the answer).
struct Fact {
    name: &'static str,
    pairs: &'static [(&'static str, &'static str)],
    user: Phrasings,
    replies: &'static [&'static str],
    /// Other answers of the relation are rejected.
    exclusive: bool,
}

const FACTS: &[Fact] = &[
    Fact {
        name: "capital",
        pairs: &[
            ("France", "Paris"),
            ("Spain", "Madrid"),
            ("Italy", "Rome"),
            ("Germany", "Berlin"),
            ("Japan", "Tokyo"),
            ("China", "Beijing"),
            ("Egypt", "Cairo"),
            ("England", "London"),
            ("Russia", "Moscow"),
            ("Canada", "Ottawa"),
            ("Greece", "Athens"),
            ("Portugal", "Lisbon"),
            ("Ireland", "Dublin"),
            ("Norway", "Oslo"),
            ("Kenya", "Nairobi"),
            ("Peru", "Lima"),
            ("Cuba", "Havana"),
            ("Austria", "Vienna"),
            ("Poland", "Warsaw"),
            ("Thailand", "Bangkok"),
        ],
        user: Phrasings {
            train: &[
                "What is the capital of {x}?",
                "What's the capital city of {x}?",
                "Which city is the capital of {x}?",
                "Tell me the capital of {x}.",
                "Do you know the capital of {x}?",
            ],
            development: &[
                "Name the capital of {x}.",
                "What city is the capital of {x}?",
                "{x} has which city as its capital?",
            ],
        },
        replies: &[
            "The capital of {x} is {y}.",
            "It's {y}.",
            "{y} is the capital of {x}.",
        ],
        exclusive: true,
    },
    Fact {
        name: "animal_sound",
        pairs: &[
            ("dog", "woof"),
            ("cat", "meow"),
            ("cow", "moo"),
            ("duck", "quack"),
            ("sheep", "baa"),
            ("pig", "oink"),
            ("horse", "neigh"),
            ("lion", "roar"),
            ("owl", "hoot"),
            ("bee", "buzz"),
            ("frog", "ribbit"),
            ("snake", "hiss"),
        ],
        user: Phrasings {
            train: &[
                "What sound does a {x} make?",
                "What does a {x} say?",
                "What noise does a {x} make?",
                "How does a {x} sound?",
            ],
            development: &[
                "Which sound comes from a {x}?",
                "If a {x} talks, what does it say?",
            ],
        },
        replies: &["A {x} says {y}.", "{y}! That's the sound a {x} makes."],
        exclusive: true,
    },
    Fact {
        name: "color",
        pairs: &[
            ("the sky", "blue"),
            ("grass", "green"),
            ("snow", "white"),
            ("a banana", "yellow"),
            ("coal", "black"),
            ("a tomato", "red"),
            ("a carrot", "orange"),
            ("a lemon", "yellow"),
            ("milk", "white"),
            ("a strawberry", "red"),
            ("a grape", "purple"),
            ("chocolate", "brown"),
        ],
        user: Phrasings {
            train: &[
                "What color is {x}?",
                "What is the color of {x}?",
                "Tell me the color of {x}.",
                "What colour is {x}?",
            ],
            development: &["Which color is {x}?", "{x} is usually what color?"],
        },
        replies: &["{x} is {y}.", "It's {y}."],
        exclusive: true,
    },
    Fact {
        name: "legs",
        pairs: &[
            ("spider", "eight"),
            ("dog", "four"),
            ("bird", "two"),
            ("ant", "six"),
            ("horse", "four"),
            ("duck", "two"),
            ("cat", "four"),
            ("bee", "six"),
        ],
        user: Phrasings {
            train: &[
                "How many legs does a {x} have?",
                "How many legs has a {x} got?",
                "Count the legs of a {x}.",
            ],
            development: &[
                "A {x} has how many legs?",
                "What number of legs does a {x} have?",
            ],
        },
        replies: &["A {x} has {y} legs.", "It has {y} legs."],
        exclusive: false,
    },
    Fact {
        name: "next_day",
        pairs: &[
            ("Monday", "Tuesday"),
            ("Tuesday", "Wednesday"),
            ("Wednesday", "Thursday"),
            ("Thursday", "Friday"),
            ("Friday", "Saturday"),
            ("Saturday", "Sunday"),
            ("Sunday", "Monday"),
        ],
        user: Phrasings {
            train: &[
                "What day comes after {x}?",
                "Which day is after {x}?",
                "What is the day after {x}?",
            ],
            development: &[
                "After {x}, what day is it?",
                "Name the day that follows {x}.",
            ],
        },
        replies: &["The day after {x} is {y}.", "It's {y}."],
        exclusive: false,
    },
];

fn fact(rng: &mut Rng, split: Split, relation: &Fact) -> Turn {
    let (subject, answer) = *pick(rng, relation.pairs);
    let slots = [("x", subject), ("y", answer)];
    let mut user = fill(relation.user.pick(rng, split), &slots);
    let mut reply = fill(pick(rng, relation.replies), &slots);
    capitalize(&mut user);
    capitalize(&mut reply);
    // A number answer ("eight") is accepted as digits too.
    let mut accepted = vec![answer.to_owned()];
    if let Some(n) = number(answer) {
        accepted.push(n.to_string());
    }
    let mut checks = vec![Check::AnyOf(accepted)];
    if relation.exclusive {
        let others: Vec<String> = relation
            .pairs
            .iter()
            .map(|(_, y)| *y)
            .filter(|y| *y != answer)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(str::to_owned)
            .collect();
        checks.push(Check::NoneOf(others));
    }
    Turn {
        intent: relation.name.into(),
        category: Category::Responsive,
        user,
        reply,
        checks,
    }
}

fn capitalize(text: &mut String) {
    if let Some(first) = text.chars().next() {
        let upper: String = first.to_uppercase().collect();
        text.replace_range(..first.len_utf8(), &upper);
    }
}

fn responsive(rng: &mut Rng, split: Split) -> Turn {
    match rng.below(10) {
        0..=3 => {
            let relation = pick(rng, FACTS);
            fact(rng, split, relation)
        }
        4 => share_event(rng, split),
        5 => share_feeling(rng, split),
        _ => {
            let index = 1 + rng.below(SOCIAL.len() - 1);
            social(rng, split, &SOCIAL[index])
        }
    }
}

// ---------------------------------------------------------------------------
// Instructions.

const REPEAT_PHRASES: &[&str] = &[
    "good night",
    "the cat is happy",
    "blue sky",
    "I like apples",
    "see you soon",
    "hello world",
    "time for lunch",
    "red and green",
    "let's go home",
    "the sun is hot",
    "one more time",
    "a big red ball",
];

const SPELL_WORDS: &[&str] = &[
    "cat", "dog", "sun", "hat", "red", "cup", "box", "pig", "bed", "map", "fish", "frog", "tree",
    "bird", "milk", "book", "cake", "lamp",
];

const OPPOSITES: &[(&str, &str)] = &[
    ("hot", "cold"),
    ("big", "small"),
    ("up", "down"),
    ("day", "night"),
    ("happy", "sad"),
    ("fast", "slow"),
    ("light", "dark"),
    ("tall", "short"),
    ("wet", "dry"),
    ("full", "empty"),
    ("loud", "quiet"),
    ("early", "late"),
    ("hard", "soft"),
    ("near", "far"),
    ("strong", "weak"),
    ("clean", "dirty"),
];

const CATEGORIES: &[(&str, &[&str])] = &[
    (
        "fruits",
        &[
            "apple", "banana", "orange", "grape", "pear", "peach", "cherry", "lemon", "mango",
            "plum",
        ],
    ),
    (
        "colors",
        &[
            "red", "blue", "green", "yellow", "purple", "pink", "black", "white", "brown", "orange",
        ],
    ),
    (
        "animals",
        &[
            "dog", "cat", "cow", "horse", "pig", "sheep", "lion", "tiger", "bear", "duck",
        ],
    ),
    (
        "days of the week",
        &[
            "monday",
            "tuesday",
            "wednesday",
            "thursday",
            "friday",
            "saturday",
            "sunday",
        ],
    ),
];

const FIRST_LETTER_WORDS: &[&str] = &[
    "apple", "banana", "cat", "dog", "egg", "fish", "goat", "house", "ice", "jam", "kite", "lion",
    "moon", "nest", "orange", "pig", "queen", "rain", "sun", "tree", "umbrella", "van", "water",
    "yellow", "zebra",
];

const YES_NO: &[(&str, bool)] = &[
    ("Is fire hot?", true),
    ("Is ice hot?", false),
    ("Is snow cold?", true),
    ("Can fish swim?", true),
    ("Can dogs fly?", false),
    ("Is the sun cold?", false),
    ("Do cows eat grass?", true),
    ("Is a mouse bigger than an elephant?", false),
    ("Can birds fly?", true),
    ("Is water wet?", true),
    ("Do cats bark?", false),
    ("Is the sky green?", false),
];

/// Development rephrasings of the same twelve facts, index for index, so a
/// development draw consumes the generator exactly as a training draw does.
const YES_NO_DEVELOPMENT: &[(&str, bool)] = &[
    ("Fire is hot, isn't it?", true),
    ("Is it true that ice is hot?", false),
    ("Snow is cold, right?", true),
    ("Is it true that fish can swim?", true),
    ("Would a dog be able to fly?", false),
    ("The sun is cold, isn't it?", false),
    ("Is it true that cows eat grass?", true),
    ("Would a mouse be bigger than an elephant?", false),
    ("Birds can fly, right?", true),
    ("Is it true that water is wet?", true),
    ("Would a cat bark?", false),
    ("The sky is green, right?", false),
];

const REPEAT: Phrasings = Phrasings {
    train: &[
        "Repeat after me: {x}.",
        "Say \"{x}\".",
        "Please repeat this: {x}.",
        "Can you say {x}?",
    ],
    development: &["Say back to me: {x}.", "Copy this exactly: {x}."],
};

const SPELL: Phrasings = Phrasings {
    train: &[
        "Spell the word {x}.",
        "How do you spell {x}?",
        "Spell {x} letter by letter.",
    ],
    development: &["What are the letters in {x}?", "Can you spell out {x}?"],
};

const COUNT: Phrasings = Phrasings {
    train: &[
        "Count from {a} to {b}.",
        "Can you count from {a} up to {b}?",
        "Please count from {a} to {b}.",
    ],
    development: &[
        "List the numbers from {a} to {b}.",
        "Count up from {a} until {b}.",
    ],
};

const ADD: Phrasings = Phrasings {
    train: &[
        "What is {a} plus {b}?",
        "Add {a} and {b}.",
        "What is {a} + {b}?",
    ],
    development: &["{a} plus {b} equals what?", "Sum {a} and {b}."],
};

const OPPOSITE: Phrasings = Phrasings {
    train: &[
        "What is the opposite of {x}?",
        "Give me the opposite of {x}.",
        "Tell me a word that means the opposite of {x}.",
    ],
    development: &["What's the reverse of {x}?", "Name the antonym of {x}."],
};

const LIST: Phrasings = Phrasings {
    train: &["Name {n} {x}.", "List {n} {x}.", "Give me {n} {x}, please."],
    development: &[
        "Tell me {n} kinds of {x}.",
        "Which {n} {x} can you think of?",
    ],
};

const FIRST_LETTER: Phrasings = Phrasings {
    train: &[
        "What letter does {x} start with?",
        "What is the first letter of {x}?",
        "Which letter begins the word {x}?",
    ],
    development: &[
        "{x} begins with which letter?",
        "Tell me the starting letter of {x}.",
    ],
};

const COMPARE: Phrasings = Phrasings {
    train: &[
        "Which is bigger, {a} or {b}?",
        "Which number is larger: {a} or {b}?",
        "What is the bigger number, {a} or {b}?",
    ],
    development: &[
        "Out of {a} and {b}, which is greater?",
        "Pick the larger number: {a} or {b}.",
    ],
};

const SENTENCE: Phrasings = Phrasings {
    train: &[
        "Use the word {x} in a sentence.",
        "Make a sentence with the word {x}.",
        "Write a sentence that has the word {x}.",
    ],
    development: &[
        "Put {x} into a short sentence.",
        "Say something using the word {x}.",
    ],
};

/// Every instruction phrasing table, for the disjointness checks.
const INSTRUCTIONS: [&Phrasings; 9] = [
    &REPEAT,
    &SPELL,
    &COUNT,
    &ADD,
    &OPPOSITE,
    &LIST,
    &FIRST_LETTER,
    &COMPARE,
    &SENTENCE,
];

const SENTENCE_WORDS: &[&str] = &[
    "dog", "rain", "happy", "school", "apple", "garden", "music", "friend", "river", "blue",
];

fn instruction(rng: &mut Rng, split: Split) -> Turn {
    let (name, user, reply, checks) = match rng.below(10) {
        0 => {
            let phrase = *pick(rng, REPEAT_PHRASES);
            let template = REPEAT.pick(rng, split);
            let mut reply = format!("{phrase}.");
            capitalize(&mut reply);
            (
                "repeat",
                fill(template, &[("x", phrase)]),
                reply,
                vec![Check::AnyOf(vec![phrase.into()])],
            )
        }
        1 => {
            let word = *pick(rng, SPELL_WORDS);
            let spelled: Vec<String> = word.chars().map(|c| c.to_string()).collect();
            let template = SPELL.pick(rng, split);
            (
                "spell",
                fill(template, &[("x", word)]),
                format!("{word} is spelled {}.", spelled.join("-")),
                vec![Check::AnyOf(vec![spelled.join(" ")])],
            )
        }
        2 => {
            let start = 1 + rng.below(4) as u32;
            let end = start + 2 + rng.below(5) as u32;
            let (a, b) = (start.to_string(), end.to_string());
            let template = COUNT.pick(rng, split);
            let numbers: Vec<String> = (start..=end).map(|n| n.to_string()).collect();
            (
                "count",
                fill(template, &[("a", &a), ("b", &b)]),
                format!("{}.", numbers.join(", ")),
                vec![
                    Check::Numbers((start..=end).collect()),
                    Check::NoNumber(end + 1),
                ],
            )
        }
        3 => {
            let a = 1 + rng.below(9) as u32;
            let b = 1 + rng.below(9) as u32;
            let (sa, sb) = (a.to_string(), b.to_string());
            let template = ADD.pick(rng, split);
            (
                "add",
                fill(template, &[("a", &sa), ("b", &sb)]),
                format!("{a} plus {b} is {}.", a + b),
                vec![Check::AnyOf(vec![
                    (a + b).to_string(),
                    NUMBER_WORDS[(a + b) as usize].into(),
                ])],
            )
        }
        4 => {
            let (word, opposite) = *pick(rng, OPPOSITES);
            let template = OPPOSITE.pick(rng, split);
            (
                "opposite",
                fill(template, &[("x", word)]),
                format!("The opposite of {word} is {opposite}."),
                vec![Check::AnyOf(vec![opposite.into()])],
            )
        }
        5 => {
            let (category, members) = *pick(rng, CATEGORIES);
            let count = 2 + rng.below(3);
            let template = LIST.pick(rng, split);
            let mut chosen: Vec<&str> = Vec::new();
            while chosen.len() < count {
                let member = *pick(rng, members);
                if !chosen.contains(&member) {
                    chosen.push(member);
                }
            }
            let shown: Vec<String> = chosen
                .iter()
                .map(|m| {
                    let mut m = (*m).to_owned();
                    if category == "days of the week" {
                        capitalize(&mut m);
                    }
                    m
                })
                .collect();
            (
                "list",
                fill(template, &[("n", NUMBER_WORDS[count]), ("x", category)]),
                format!(
                    "Here are {} {category}: {}.",
                    NUMBER_WORDS[count],
                    shown.join(", ")
                ),
                vec![Check::Members {
                    set: strings(members),
                    at_least: count,
                }],
            )
        }
        6 => {
            let word = *pick(rng, FIRST_LETTER_WORDS);
            let letter = word.chars().next().unwrap_or('a').to_ascii_uppercase();
            let template = FIRST_LETTER.pick(rng, split);
            let mut user = fill(template, &[("x", word)]);
            capitalize(&mut user);
            (
                "first_letter",
                user,
                format!("{word} starts with the letter {letter}."),
                vec![Check::LastLetter(letter)],
            )
        }
        7 => {
            let a = 1 + rng.below(20) as u32;
            let mut b = 1 + rng.below(20) as u32;
            if b == a {
                b = if a == 20 { 19 } else { a + 1 };
            }
            let (sa, sb) = (a.to_string(), b.to_string());
            let template = COMPARE.pick(rng, split);
            let (big, small) = (a.max(b), a.min(b));
            (
                "compare",
                fill(template, &[("a", &sa), ("b", &sb)]),
                format!("{big} is bigger."),
                vec![Check::Larger {
                    answer: big,
                    other: small,
                }],
            )
        }
        8 => {
            let questions = match split {
                Split::Train => YES_NO,
                Split::Development => YES_NO_DEVELOPMENT,
            };
            let (question, yes) = *pick(rng, questions);
            let reply = if yes { "Yes." } else { "No." };
            (
                "yes_no",
                question.into(),
                reply.into(),
                vec![Check::FirstWord(if yes { "yes" } else { "no" }.into())],
            )
        }
        _ => {
            let word = *pick(rng, SENTENCE_WORDS);
            let template = SENTENCE.pick(rng, split);
            let sentences: &[&str] = &[
                "I saw a {x} today.",
                "The {x} made me smile.",
                "We talked about the {x} all day.",
            ];
            let reply = match word {
                "happy" => "I feel happy when I am with my friends.".to_owned(),
                "blue" => "The sky is blue today.".to_owned(),
                "rain" => "The rain fell on the roof all night.".to_owned(),
                "music" => "We listened to music after dinner.".to_owned(),
                _ => fill(pick(rng, sentences), &[("x", word)]),
            };
            (
                "sentence",
                fill(template, &[("x", word)]),
                reply,
                vec![
                    Check::AnyOf(vec![word.into()]),
                    Check::MinWords(4),
                    Check::NotEcho,
                ],
            )
        }
    };
    Turn {
        intent: name.into(),
        category: Category::Instruction,
        user,
        reply,
        checks,
    }
}

// ---------------------------------------------------------------------------
// Relations: assert, optionally update, then query, in context.

struct Relation {
    name: &'static str,
    train_values: &'static [&'static str],
    development_values: &'static [&'static str],
    assert: Phrasings,
    update: Phrasings,
    query: Phrasings,
    acks: &'static [&'static str],
    answers: &'static [&'static str],
}

const RELATIONS: &[Relation] = &[
    Relation {
        name: "name",
        train_values: &[
            "Sam", "Mia", "Leo", "Nora", "Ben", "Lily", "Max", "Ella", "Jack", "Zoe",
        ],
        development_values: &["Ruby", "Owen", "Ivy", "Theo"],
        assert: Phrasings {
            train: &[
                "My name is {v}.",
                "I'm {v}.",
                "Call me {v}.",
                "People call me {v}.",
            ],
            development: &["You can call me {v}.", "The name's {v}."],
        },
        update: Phrasings {
            train: &[
                "Actually, my name is {v}.",
                "Sorry, I meant my name is {v}.",
                "Please call me {v} instead.",
            ],
            development: &["Correction: call me {v}."],
        },
        query: Phrasings {
            train: &[
                "What is my name?",
                "What's my name?",
                "Do you remember my name?",
            ],
            development: &["Tell me what I'm called.", "What name did I give you?"],
        },
        acks: &["Nice to meet you, {v}!", "Hi {v}, I'll remember that."],
        answers: &["Your name is {v}.", "You're {v}."],
    },
    Relation {
        name: "job",
        train_values: &[
            "doctor", "nurse", "cook", "farmer", "pilot", "baker", "driver", "painter",
        ],
        development_values: &["dentist", "plumber", "singer"],
        assert: Phrasings {
            train: &["I work as a {v}.", "My job is {v}.", "I am a {v} by trade."],
            development: &["I earn a living as a {v}."],
        },
        update: Phrasings {
            train: &[
                "Actually, I work as a {v} now.",
                "I changed jobs. Now I'm a {v}.",
            ],
            development: &["These days my job is {v}."],
        },
        query: Phrasings {
            train: &[
                "What is my job?",
                "What do I do for work?",
                "What do I work as?",
            ],
            development: &["What's my occupation?", "Remind me what my job is."],
        },
        acks: &["Got it, you work as a {v}.", "A {v}, that's great."],
        answers: &["You work as a {v}.", "Your job is {v}."],
    },
    Relation {
        name: "home",
        train_values: &[
            "Paris", "Madrid", "Rome", "Berlin", "Oslo", "Lima", "Cairo", "Dublin",
        ],
        development_values: &["Vienna", "Warsaw", "Lisbon"],
        assert: Phrasings {
            train: &["I live in {v}.", "My home is in {v}.", "I moved to {v}."],
            development: &["I reside in {v}."],
        },
        update: Phrasings {
            train: &["Actually, I live in {v} now.", "I just moved to {v}."],
            development: &["These days my home is in {v}."],
        },
        query: Phrasings {
            train: &[
                "Where do I live?",
                "What city do I live in?",
                "Where is my home?",
            ],
            development: &["Which town am I living in?", "Remind me where I live."],
        },
        acks: &["{v} sounds like a nice place.", "Got it, you live in {v}."],
        answers: &["You live in {v}.", "Your home is in {v}."],
    },
    Relation {
        name: "favorite_food",
        train_values: &[
            "pasta", "soup", "rice", "cake", "bread", "tacos", "noodles", "salad",
        ],
        development_values: &["curry", "pancakes", "sushi"],
        assert: Phrasings {
            train: &[
                "My favorite food is {v}.",
                "I love eating {v}.",
                "I like {v} more than any other food.",
            ],
            development: &["The food I like best is {v}."],
        },
        update: Phrasings {
            train: &[
                "Actually, my favorite food is {v} now.",
                "I changed my mind, I like {v} best.",
            ],
            development: &["These days my favorite food is {v}."],
        },
        query: Phrasings {
            train: &[
                "What is my favorite food?",
                "What food do I love?",
                "Which food do I like best?",
            ],
            development: &[
                "Remind me of my favorite food.",
                "What food did I say I like most?",
            ],
        },
        acks: &["Yum, {v} is tasty!", "Got it, you love {v}."],
        answers: &["Your favorite food is {v}.", "You love {v}."],
    },
    Relation {
        name: "pet_name",
        train_values: &[
            "Buddy", "Luna", "Coco", "Rex", "Bella", "Milo", "Daisy", "Oscar",
        ],
        development_values: &["Pepper", "Ziggy", "Hazel"],
        assert: Phrasings {
            train: &[
                "My dog is named {v}.",
                "I have a dog called {v}.",
                "My dog's name is {v}.",
            ],
            development: &["My dog goes by {v}."],
        },
        update: Phrasings {
            train: &["Actually, my dog is named {v}.", "We renamed my dog {v}."],
            development: &["My dog's new name is {v}."],
        },
        query: Phrasings {
            train: &[
                "What is my dog's name?",
                "What's my dog called?",
                "What is my dog named?",
            ],
            development: &[
                "What did I name my dog?",
                "Remind me what my dog is called.",
            ],
        },
        acks: &[
            "{v} is a lovely name for a dog.",
            "Got it, your dog is {v}.",
        ],
        answers: &["Your dog is named {v}.", "Your dog's name is {v}."],
    },
];

fn relation_values(relation: &Relation, split: Split) -> &'static [&'static str] {
    match split {
        Split::Train => relation.train_values,
        Split::Development => relation.development_values,
    }
}

/// Which part of a relation conversation a turn is.
#[derive(Clone, Copy)]
enum Act {
    Assert,
    Update,
    Query,
}

fn relation_turn(
    rng: &mut Rng,
    split: Split,
    relation: &Relation,
    act: Act,
    value: &str,
    checks: Vec<Check>,
    replies: &[&str],
) -> Turn {
    let slots = [("v", value)];
    let (phrasings, suffix, category) = match act {
        Act::Assert => (&relation.assert, "assert", Category::Responsive),
        Act::Update => (&relation.update, "update", Category::Responsive),
        Act::Query => (&relation.query, "query", Category::Relation),
    };
    Turn {
        intent: format!("{}_{suffix}", relation.name),
        category,
        user: fill(phrasings.pick(rng, split), &slots),
        reply: fill(pick(rng, replies), &slots),
        checks,
    }
}

/// Acknowledgment phrases for an assertion or an update, beside the stated
/// value itself. Generic praise ("great", "nice") is not an acknowledgment:
/// filler replies are full of it.
const ACK_WORDS: &[&str] = &[
    "got it",
    "okay",
    "ok",
    "i'll remember",
    "noted",
    "thanks for telling me",
];

/// Assert, optional distractor turns and an optional update, then the query.
fn relation_conversation(rng: &mut Rng, split: Split) -> Vec<Turn> {
    let relation = pick(rng, RELATIONS);
    let values = relation_values(relation, split);
    let first = *pick(rng, values);
    let mut turns = Vec::new();
    let ack_checks = |value: &str| {
        let mut words = strings(ACK_WORDS);
        words.push(value.to_lowercase());
        vec![Check::AnyOf(words)]
    };
    turns.push(relation_turn(
        rng,
        split,
        relation,
        Act::Assert,
        first,
        ack_checks(first),
        relation.acks,
    ));
    if rng.below(2) == 0 {
        turns.push(responsive(rng, split));
    }
    let mut current = first;
    let updated = rng.below(2) == 0;
    if updated {
        let mut second = *pick(rng, values);
        while second == first {
            second = *pick(rng, values);
        }
        turns.push(relation_turn(
            rng,
            split,
            relation,
            Act::Update,
            second,
            ack_checks(second),
            &[
                "Okay, I'll remember that.",
                "Got it, thanks for telling me.",
            ],
        ));
        current = second;
    }
    // One query in four asks for a relation that was never stated: the
    // answer is an abstention, never the stated value.
    if rng.below(4) == 0 {
        let mut other = pick(rng, RELATIONS);
        while other.name == relation.name {
            other = pick(rng, RELATIONS);
        }
        // Reject this conversation's values and every value the asked
        // relation can take, so a hedged guess fails.
        let rejected: BTreeSet<String> = [first, current]
            .into_iter()
            .chain(other.train_values.iter().copied())
            .chain(other.development_values.iter().copied())
            .map(str::to_lowercase)
            .collect();
        turns.push(Turn {
            intent: format!("{}_absent", other.name),
            category: Category::Relation,
            user: other.query.pick(rng, split).into(),
            reply: (*pick(rng, ABSENT_REPLIES)).into(),
            checks: vec![
                Check::AnyOf(strings(ABSENT_ACCEPT)),
                Check::NoneOf(rejected.into_iter().collect()),
            ],
        });
        return turns;
    }
    let mut checks = vec![Check::AnyOf(vec![current.to_lowercase()])];
    if updated {
        checks.push(Check::NoneOf(vec![first.to_lowercase()]));
    }
    turns.push(relation_turn(
        rng,
        split,
        relation,
        Act::Query,
        current,
        checks,
        relation.answers,
    ));
    turns
}

const ABSENT_REPLIES: &[&str] = &[
    "I don't know. You haven't told me yet.",
    "You haven't told me that yet.",
    "I'm not sure, you didn't tell me.",
];

const ABSENT_ACCEPT: &[&str] = &[
    "don't know",
    "do not know",
    "haven't told",
    "didn't tell",
    "not sure",
];

// ---------------------------------------------------------------------------
// Conversations.

/// The world. Stateless: every conversation comes from the caller's `Rng`.
pub struct MWorld;

impl MWorld {
    /// One conversation of 1 to [`MAX_TURNS`] user turns: responsive chat,
    /// instructions or a relation, sometimes opened with a greeting and closed
    /// with thanks or a farewell.
    pub fn conversation(rng: &mut Rng, split: Split) -> Conversation {
        let mut turns = Vec::new();
        if rng.below(3) == 0 {
            turns.push(social(rng, split, &SOCIAL[0]));
        }
        // Weights 2 : 3 : 2 (responsive : instructions : relation), with up to
        // three instructions. Responsive turns still outnumber them, since
        // openers, closers, acknowledgments and distractors are responsive.
        match rng.below(7) {
            0 | 1 => {
                let body = 1 + rng.below(3);
                for _ in 0..body {
                    turns.push(responsive(rng, split));
                }
            }
            2..=4 => {
                let body = 1 + rng.below(3);
                for _ in 0..body {
                    turns.push(instruction(rng, split));
                }
            }
            _ => turns.extend(relation_conversation(rng, split)),
        }
        if turns.len() < MAX_TURNS && rng.below(3) == 0 {
            let index = 2 + rng.below(2);
            turns.push(social(rng, split, &SOCIAL[index]));
        }
        for turn in &mut turns {
            turn.user = articles(&turn.user);
            turn.reply = articles(&turn.reply);
        }
        Conversation { turns }
    }

    /// A conversation none of whose user turns equals an `excluded` turn
    /// (compared by [`normalized`]); rejected draws are counted in `rejected`.
    pub fn conversation_excluding(
        rng: &mut Rng,
        split: Split,
        excluded: &BTreeSet<String>,
        rejected: &mut usize,
    ) -> Result<Conversation> {
        for _ in 0..10_000 {
            let conversation = Self::conversation(rng, split);
            if conversation
                .turns
                .iter()
                .all(|turn| !excluded.contains(&normalized(&turn.user)))
            {
                return Ok(conversation);
            }
            *rejected += 1;
        }
        Err(invalid(
            "the exclusion list rejects every M-world conversation",
        ))
    }

    /// Every user phrasing template of `split`, for disjointness checks.
    pub fn templates(split: Split) -> Vec<&'static str> {
        let mut all: Vec<&'static str> = Vec::new();
        let mut add = |p: &Phrasings| {
            all.extend(match split {
                Split::Train => p.train,
                Split::Development => p.development,
            })
        };
        for social in SOCIAL {
            add(&social.user);
        }
        add(&SHARE_EVENT);
        add(&SHARE_FEELING);
        for fact in FACTS {
            add(&fact.user);
        }
        for relation in RELATIONS {
            add(&relation.assert);
            add(&relation.update);
            add(&relation.query);
        }
        for instruction in INSTRUCTIONS {
            add(instruction);
        }
        all.extend(
            match split {
                Split::Train => YES_NO,
                Split::Development => YES_NO_DEVELOPMENT,
            }
            .iter()
            .map(|(question, _)| *question),
        );
        all
    }

    /// SHA-256 of every table the world draws from and the version string, so
    /// two builds' worlds can be compared without reading the source.
    pub fn digest() -> String {
        let phrasings = |p: &Phrasings| serde_json::json!([p.train, p.development]);
        let tables = serde_json::json!({
            "version": "m-world-v1",
            "social": SOCIAL.iter().map(|s| serde_json::json!([s.name, phrasings(&s.user), s.replies, s.accept, s.question])).collect::<Vec<_>>(),
            "share": [phrasings(&SHARE_EVENT), SHARE_EVENT_REPLIES, THINGS, phrasings(&SHARE_FEELING), FEELINGS_GOOD, FEELINGS_BAD],
            "facts": FACTS.iter().map(|f| serde_json::json!([f.name, f.pairs, phrasings(&f.user), f.replies, f.exclusive])).collect::<Vec<_>>(),
            "instructions": INSTRUCTIONS.iter().map(|p| phrasings(p)).collect::<Vec<_>>(),
            "instruction_tables": [REPEAT_PHRASES, SPELL_WORDS, FIRST_LETTER_WORDS, SENTENCE_WORDS],
            "opposites": OPPOSITES,
            "categories": CATEGORIES,
            "yes_no": [YES_NO, YES_NO_DEVELOPMENT],
            "relations": RELATIONS.iter().map(|r| serde_json::json!([r.name, r.train_values, r.development_values, phrasings(&r.assert), phrasings(&r.update), phrasings(&r.query), r.acks, r.answers])).collect::<Vec<_>>(),
            "acknowledgments": ACK_WORDS,
            "abstentions": [ABSENT_REPLIES, ABSENT_ACCEPT],
        });
        hex::encode(Sha256::digest(tables.to_string().as_bytes()))
    }
}

/// "a" before a word that begins with a vowel letter becomes "an" ("an owl",
/// "an apple"). The world's vocabulary has no vowel letter sounded as a
/// consonant.
fn articles(text: &str) -> String {
    let pieces: Vec<&str> = text.split(' ').collect();
    let mut out = Vec::with_capacity(pieces.len());
    for (i, piece) in pieces.iter().enumerate() {
        let vowel = pieces
            .get(i + 1)
            .and_then(|next| next.chars().next())
            .is_some_and(|c| "aeiouAEIOU".contains(c));
        out.push(match (*piece, vowel) {
            ("a", true) => "an",
            ("A", true) => "An",
            _ => piece,
        });
    }
    out.join(" ")
}

/// The most user turns in one conversation: a greeting, an assertion, a
/// distractor, an update and the query.
pub const MAX_TURNS: usize = 5;

/// A user turn's form for exclusion: its words, lowercased, without
/// punctuation.
pub fn normalized(text: &str) -> String {
    words(text).join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_oracle_judges_membership_only() {
        let any = [Check::AnyOf(vec!["paris".into()])];
        assert!(judge(&any, "q", "The capital of France is Paris."));
        assert!(!judge(&any, "q", "The capital is Parisian."));
        let none = [
            Check::AnyOf(vec!["paris".into()]),
            Check::NoneOf(vec!["rome".into()]),
        ];
        assert!(!judge(&none, "q", "Paris or Rome."));
        let members = [Check::Members {
            set: strings(&["apple", "pear", "plum"]),
            at_least: 2,
        }];
        assert!(judge(&members, "q", "Apple and pear."));
        assert!(!judge(&members, "q", "Apple and apple."));
        assert!(judge(
            &[Check::FirstNumber(7)],
            "q",
            "Seven is bigger than 3."
        ));
        assert!(!judge(
            &[Check::FirstNumber(7)],
            "q",
            "3 is smaller than 7."
        ));
        assert!(judge(&[Check::FirstWord("yes".into())], "q", "Yes, it is."));
        assert!(judge(&[Check::LastLetter('B')], "q", "It starts with a B."));
        assert!(!judge(&[Check::LastLetter('B')], "q", "It starts with a."));
        // The baseline's false positives: the article and a sentence-initial
        // "A" or "I" are not an answer.
        let a = [Check::LastLetter('A')];
        assert!(!judge(&a, "q", "I'd recommend using a mixture to create a"));
        assert!(!judge(&a, "q", "A gentle breeze is here."));
        assert!(judge(&a, "q", "A."));
        assert!(judge(&a, "q", "apple starts with the letter A."));
        let i = [Check::LastLetter('I')];
        assert!(!judge(&i, "q", "I think it is great."));
        assert!(judge(&i, "q", "I think it starts with I."));
        let mut acknowledgments = strings(ACK_WORDS);
        acknowledgments.push("pepper".into());
        let ack = [Check::AnyOf(acknowledgments)];
        assert!(!judge(
            &ack,
            "My dog goes by Pepper.",
            "My dog's dog is a great way to play."
        ));
        assert!(judge(
            &ack,
            "My dog goes by Pepper.",
            "Got it, your dog is Pepper."
        ));
        let echo = [Check::NotEcho];
        assert!(!judge(&echo, "I got a kite.", "I got a kite!"));
        assert!(judge(&echo, "I got a kite.", "A kite! Fun."));
        assert!(judge(&[Check::Literal("?".into())], "q", "Which one?"));
    }

    #[test]
    fn the_review_cases_judge_as_intended() {
        // #1503 review, required 3: the pronoun is not an answer, and a
        // trailing sentence does not hide one.
        assert!(!judge(&[Check::LastLetter('I')], "q", "I do not know."));
        let z = "Zebra starts with the letter Z. I hope that helps!";
        assert!(judge(&[Check::LastLetter('Z')], "q", z));
        // Suggestion 1: the number called larger, not the first number.
        let larger = |answer, other, reply| judge(&[Check::Larger { answer, other }], "q", reply);
        assert!(larger(7, 3, "Out of 3 and 7, 7 is greater."));
        assert!(larger(12, 3, "The bigger one is 12."));
        assert!(larger(7, 3, "7 is bigger than 3."));
        assert!(larger(12, 3, "12."));
        assert!(!larger(7, 3, "3 is bigger than 7."));
        assert!(!larger(12, 3, "12 is smaller."));
        assert!(!larger(7, 3, "3 and 7."));
        // Suggestion 2: numerals as digits or words.
        let count = [Check::Numbers(vec![1, 2, 3]), Check::NoNumber(4)];
        assert!(judge(&count, "q", "One, two, three."));
        assert!(judge(&count, "q", "1, 2, 3."));
        assert!(!judge(&count, "q", "1, 2, 3, 4."));
        assert!(!judge(&count, "q", "1, 2."));
        // Suggestion 4: typographic apostrophes and quotes.
        assert!(judge(
            &[Check::AnyOf(vec!["don't know".into()])],
            "q",
            "I don\u{2019}t know."
        ));
        assert!(judge(&[Check::AnyOf(vec!["paris".into()])], "q", "'Paris'"));
        // Suggestion 8: articles.
        assert_eq!(articles("A owl says hoot."), "An owl says hoot.");
        assert_eq!(
            articles("I saw a apple and a dog."),
            "I saw an apple and a dog."
        );
        assert_eq!(MWorld::digest().len(), 64);
    }

    #[test]
    fn a_filler_reply_fails_every_intent() {
        // Suggestion 12: the baseline model's typical filler answers nothing.
        // ("I'm glad I could help" is a fair reply to thanks, so it is not used.)
        let filler = "The main difference is a simple yet simple way to spend the day at the park.";
        for split in [Split::Train, Split::Development] {
            let mut rng = Rng::new(19);
            for _ in 0..2000 {
                for turn in MWorld::conversation(&mut rng, split).turns {
                    assert!(
                        !judge(&turn.checks, &turn.user, filler),
                        "{:?}: {:?} passes filler",
                        turn.intent,
                        turn.checks
                    );
                }
            }
        }
    }

    #[test]
    fn every_trained_reply_passes_its_own_checks() {
        for split in [Split::Train, Split::Development] {
            let mut rng = Rng::new(7);
            for _ in 0..3000 {
                let conversation = MWorld::conversation(&mut rng, split);
                assert!(!conversation.turns.is_empty() && conversation.turns.len() <= MAX_TURNS);
                for turn in &conversation.turns {
                    assert!(
                        judge(&turn.checks, &turn.user, &turn.reply),
                        "{:?}: {:?} -> {:?} fails {:?}",
                        turn.intent,
                        turn.user,
                        turn.reply,
                        turn.checks
                    );
                }
            }
        }
    }

    #[test]
    fn development_phrasings_never_appear_in_training() {
        let train: BTreeSet<&str> = MWorld::templates(Split::Train).into_iter().collect();
        for template in MWorld::templates(Split::Development) {
            assert!(!train.contains(template), "{template:?} is in both splits");
        }
        for relation in RELATIONS {
            for value in relation.development_values {
                assert!(!relation.train_values.contains(value), "{value}");
            }
        }
    }

    #[test]
    fn relation_queries_answer_the_latest_value() {
        let mut rng = Rng::new(11);
        let (mut updated, mut absent) = (0, 0);
        for _ in 0..2000 {
            let turns = relation_conversation(&mut rng, Split::Train);
            let query = turns.last().expect("a query");
            assert_eq!(query.category, Category::Relation);
            if query.intent.ends_with("_absent") {
                absent += 1;
                // The abstention never states a value of this conversation.
                let stated = query
                    .checks
                    .iter()
                    .find_map(|c| match c {
                        Check::NoneOf(values) => values.first().cloned(),
                        _ => None,
                    })
                    .expect("the stated values are rejected");
                let hedge = format!("I don't know, maybe {stated}.");
                assert!(!judge(&query.checks, &query.user, &hedge));
                assert!(judge(&query.checks, &query.user, "I don't know."));
                // Review, required 4: a guess from outside the conversation
                // fails too.
                let asked = RELATIONS
                    .iter()
                    .find(|r| query.intent == format!("{}_absent", r.name))
                    .expect("the asked relation");
                for value in asked.train_values.iter().chain(asked.development_values) {
                    let guess = format!("You haven't told me that yet, but I think it is {value}.");
                    assert!(!judge(&query.checks, &query.user, &guess), "{guess}");
                }
            }
            if turns.iter().any(|t| t.intent.ends_with("_update")) {
                updated += 1;
                assert!(query.checks.iter().any(|c| matches!(c, Check::NoneOf(_))));
            }
        }
        assert!(updated > 500 && absent > 300, "{updated} {absent}");
    }

    #[test]
    fn the_milestone_memory_sentences_are_excluded() {
        // The ten memory requests of the 38-request development panel.
        let panel = [
            "My name is Alex.",
            "My cat is named Momo.",
            "My favorite color is green.",
            "I work as a teacher.",
            "My sister lives in Tokyo.",
            "I have two brothers.",
            "My birthday is in July.",
            "I drive a blue car.",
            "I am learning to play the piano.",
            "My favorite food is pizza.",
            "What is my name?",
            "What is my cat's name?",
            "What is my favorite color?",
            "What is my job?",
            "Where does my sister live?",
            "How many brothers do I have?",
            "When is my birthday?",
            "What color is my car?",
            "What instrument am I learning?",
            "What is my favorite food?",
        ];
        let excluded: BTreeSet<String> = panel.iter().map(|s| normalized(s)).collect();
        let (mut rng, mut rejected) = (Rng::new(3), 0);
        for _ in 0..5000 {
            let conversation =
                MWorld::conversation_excluding(&mut rng, Split::Train, &excluded, &mut rejected)
                    .expect("a conversation");
            for turn in &conversation.turns {
                assert!(
                    !excluded.contains(&normalized(&turn.user)),
                    "{:?}",
                    turn.user
                );
            }
        }
        // Three training queries are panel questions verbatim, so some draws
        // are rejected, and only those.
        assert!(rejected > 0 && rejected < 2500, "{rejected}");
    }
}
