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
}

const FAMILIES: [Family; 2] = [
    Family {
        key: "collection",
        facts: [
            Fact {
                name: "Elspeth",
                connector: "found",
                item: "red brass keys",
            },
            Fact {
                name: "Leander",
                connector: "might take",
                item: "silk scarves",
            },
        ],
        absent: "Beatrix",
    },
    Family {
        key: "inventory",
        facts: [
            Fact {
                name: "Maren",
                connector: "could still find",
                item: "brushes",
            },
            Fact {
                name: "Ivor",
                connector: "took",
                item: "small glass jars",
            },
        ],
        absent: "Cerys",
    },
];

pub struct Case {
    pub id: String,
    pub pair: String,
    pub family: &'static str,
    pub direction: usize,
    pub form: Option<usize>,
    pub written_value_words: Option<usize>,
    pub triple: Option<String>,
    pub stratum: &'static str,
    pub facts: Vec<Fact>,
    pub name: Option<&'static str>,
    pub excluded_name: Option<&'static str>,
    pub question: String,
    pub query_name_byte: usize,
    pub the: Option<bool>,
    pub about: Option<bool>,
    pub reversed: Option<bool>,
    pub distractor_first: Option<bool>,
}

pub fn make_cases() -> Vec<Case> {
    let mut rows = Vec::new();
    for family in FAMILIES {
        for reversed in [false, true] {
            let mut facts = family.facts;
            if reversed {
                facts.reverse();
            }
            for direction in 0..2 {
                let target = family.facts[direction];
                let other = family.facts[1 - direction];
                for the in [false, true] {
                    let article = if the { "the " } else { "" };
                    for about in [false, true] {
                        let relation = if about { "about" } else { "for" };
                        let pair =
                            format!("{}-r{reversed}-d{direction}-t{the}-a{about}", family.key);
                        for distractor_first in [false, true] {
                            let (before, after) = if distractor_first {
                                (
                                    format!("Ignoring {}, quote {article}exact object wording {relation} ", other.name),
                                    ".".to_owned(),
                                )
                            } else {
                                (
                                    format!("Quote {article}exact object wording {relation} "),
                                    format!(", ignoring {}.", other.name),
                                )
                            };
                            rows.push(Case {
                                id: format!("{pair}-x{distractor_first}"),
                                pair: pair.clone(),
                                family: family.key,
                                direction,
                                form: None,
                                written_value_words: Some(target.item.split_whitespace().count()),
                                triple: None,
                                stratum: "supported",
                                facts: facts.to_vec(),
                                name: Some(target.name),
                                excluded_name: Some(other.name),
                                question: format!("{before}{}{after}", target.name),
                                query_name_byte: before.len(),
                                the: Some(the),
                                about: Some(about),
                                reversed: Some(reversed),
                                distractor_first: Some(distractor_first),
                            });
                        }
                    }
                }
            }
        }
        for (stratum, name, before, after) in [
            (
                "unmentioned",
                Some(family.absent),
                "Quote the exact object wording about ",
                ", if any entry supports it.",
            ),
            (
                "conflict",
                Some(family.facts[0].name),
                "With both entries for ",
                " presented as equally credible versions of one event, what exact object can be determined?",
            ),
            (
                "unspecified",
                None,
                "Quote the exact object wording for the person I have in mind.",
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
                triple: None,
                stratum,
                facts,
                name,
                excluded_name: None,
                question: format!("{before}{}{after}", name.unwrap_or("")),
                query_name_byte: before.len(),
                the: None,
                about: None,
                reversed: None,
                distractor_first: None,
            });
        }
    }
    rows
}
