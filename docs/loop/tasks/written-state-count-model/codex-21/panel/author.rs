#[derive(Clone, Copy)]
pub struct Fact {
    pub name: &'static str,
    pub connector: &'static str,
    pub item: &'static str,
}

struct Family {
    key: &'static str,
    facts: [Fact; 2],
    absent: &'static str,
    affirmative: &'static str,
    exclusion: &'static str,
}

const FAMILIES: [Family; 3] = [
    Family {
        key: "inventory",
        facts: [
            Fact {
                name: "Rosalind",
                connector: "took",
                item: "small glass jars",
            },
            Fact {
                name: "Benedict",
                connector: "could still find",
                item: "brushes",
            },
        ],
        absent: "Miranda",
        affirmative: "copy the exact item wording for ",
        exclusion: "leave out the entry for ",
    },
    Family {
        key: "collection",
        facts: [
            Fact {
                name: "Juliet",
                connector: "kept",
                item: "green glass beads",
            },
            Fact {
                name: "Sebastian",
                connector: "might take",
                item: "linen napkins",
            },
        ],
        absent: "Cordelia",
        affirmative: "quote the words naming the object for ",
        exclusion: "disregard the description for ",
    },
    Family {
        key: "supplies-transfer",
        facts: [
            Fact {
                name: "Helena",
                connector: "got",
                item: "blue glass cups",
            },
            Fact {
                name: "Ferdinand",
                connector: "can also take",
                item: "new paper bags",
            },
        ],
        absent: "Viola",
        affirmative: "quote the words naming the object for ",
        exclusion: "disregard the description for ",
    },
];

pub struct Case {
    pub id: String,
    pub pair: String,
    pub family: &'static str,
    pub direction: usize,
    pub form: Option<usize>,
    pub written_value_words: Option<usize>,
    pub stratum: &'static str,
    pub facts: Vec<Fact>,
    pub name: Option<&'static str>,
    pub excluded_name: Option<&'static str>,
    pub question: String,
    pub query_name_byte: usize,
    pub reversed: Option<bool>,
    pub distractor_first: Option<bool>,
    pub name_order_reversed: Option<bool>,
    pub object_swap: Option<bool>,
    pub connector_swap: Option<bool>,
}

pub fn make_cases() -> Vec<Case> {
    let mut rows = Vec::new();
    for (wording, family) in FAMILIES.iter().enumerate() {
        for object_swap in [false, true] {
            for connector_swap in [false, true] {
                let assigned = std::array::from_fn::<_, 2, _>(|person| Fact {
                    name: family.facts[person].name,
                    connector: family.facts[person ^ usize::from(connector_swap)].connector,
                    item: family.facts[person ^ usize::from(object_swap)].item,
                });
                for name_order_reversed in [false, true] {
                    for distractor_first in [false, true] {
                        let direction =
                            usize::from(name_order_reversed) ^ usize::from(distractor_first);
                        let target = assigned[direction];
                        let other = assigned[1 - direction];
                        let positive = format!("{}{}", family.affirmative, target.name);
                        let negative = format!("{}{}", family.exclusion, other.name);
                        let (clauses, query_name_byte) = if distractor_first {
                            (
                                format!("{negative}; {positive}"),
                                negative.len() + 2 + family.affirmative.len(),
                            )
                        } else {
                            (format!("{positive}; {negative}"), family.affirmative.len())
                        };
                        // Capitalize only the command's initial byte so both clause
                        // placements retain the same delimiters and proper names.
                        let mut question = format!("{clauses}.");
                        question[..1].make_ascii_uppercase();
                        let pair = format!(
                            "{}-p{direction}-o{}-c{}",
                            family.key,
                            u8::from(object_swap),
                            u8::from(connector_swap)
                        );
                        for reversed in [false, true] {
                            let mut facts = assigned;
                            if reversed {
                                facts.reverse();
                            }
                            rows.push(Case {
                                id: format!(
                                    "{pair}-q{}-r{}",
                                    u8::from(distractor_first),
                                    u8::from(reversed)
                                ),
                                pair: pair.clone(),
                                family: family.key,
                                direction,
                                form: Some(wording),
                                written_value_words: Some(target.item.split_whitespace().count()),
                                stratum: "supported",
                                facts: facts.to_vec(),
                                name: Some(target.name),
                                excluded_name: Some(other.name),
                                question: question.clone(),
                                query_name_byte,
                                reversed: Some(reversed),
                                distractor_first: Some(distractor_first),
                                name_order_reversed: Some(name_order_reversed),
                                object_swap: Some(object_swap),
                                connector_swap: Some(connector_swap),
                            });
                        }
                    }
                }
            }
        }
        if wording == 2 {
            continue;
        }
        for (stratum, name, before, after) in [
            (
                "unmentioned",
                Some(family.absent),
                "Copy the object wording recorded for ",
                " if there is an entry for that person.",
            ),
            (
                "conflict",
                Some(family.facts[0].name),
                "Which exact object did ",
                " get if the two entries are contradictory and equally credible?",
            ),
            (
                "unspecified",
                None,
                "I need the exact wording of the object for one particular person but cannot say which person.",
                "",
            ),
        ] {
            let mut facts = family.facts.to_vec();
            if stratum == "conflict" {
                facts[0].connector = family.facts[1].connector;
                facts.push(Fact {
                    name: family.facts[0].name,
                    connector: family.facts[1].connector,
                    item: family.facts[1].item,
                });
            }
            let id = format!("{}-{stratum}", family.key);
            rows.push(Case {
                id: id.clone(),
                pair: id,
                family: family.key,
                direction: 0,
                form: None,
                written_value_words: None,
                stratum,
                facts,
                name,
                excluded_name: None,
                question: format!("{before}{}{after}", name.unwrap_or("")),
                query_name_byte: before.len(),
                reversed: None,
                distractor_first: None,
                name_order_reversed: None,
                object_swap: None,
                connector_swap: None,
            });
        }
    }
    rows
}

impl Case {
    pub fn surface(&self, head: usize) -> (String, Vec<[usize; 3]>, usize) {
        let mut text = String::new();
        let mut fact_anchors = Vec::new();
        for fact in &self.facts {
            if !text.is_empty() {
                text.push(' ');
            }
            let name = head + text.len();
            text.push_str(fact.name);
            text.push(' ');
            let verb = head + text.len();
            text.push_str(fact.connector);
            text.push(' ');
            let item = head + text.len();
            text.push_str(fact.item);
            text.push('.');
            fact_anchors.push([name, verb, item]);
        }
        text.push(' ');
        let question = head + text.len();
        text.push_str(&self.question);
        (text, fact_anchors, question)
    }
}
