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
    before: &'static str,
    exclusion: &'static str,
}

const FAMILIES: [Family; 2] = [
    Family {
        key: "collection",
        facts: [
            Fact {
                name: "Adelaide",
                connector: "found",
                item: "red brass keys",
            },
            Fact {
                name: "Bertram",
                connector: "might take",
                item: "silk scarves",
            },
        ],
        absent: "Dorian",
        before: "What exact object wording is given for ",
        exclusion: ", setting aside the entry for ",
    },
    Family {
        key: "inventory",
        facts: [
            Fact {
                name: "Winifred",
                connector: "could still find",
                item: "brushes",
            },
            Fact {
                name: "Caspian",
                connector: "took",
                item: "small glass jars",
            },
        ],
        absent: "Nerissa",
        before: "Repeat the exact words that name the object associated with ",
        exclusion: ", leaving out the entry for ",
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
                for direction in 0..2 {
                    let target = assigned[direction];
                    let other = assigned[1 - direction];
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
                        let punctuation = if wording == 0 { '?' } else { '.' };
                        rows.push(Case {
                            id: format!("{pair}-r{}", u8::from(reversed)),
                            pair: pair.clone(),
                            family: family.key,
                            direction,
                            form: Some(wording),
                            written_value_words: Some(target.item.split_whitespace().count()),
                            triple: None,
                            stratum: "supported",
                            facts: facts.to_vec(),
                            name: Some(target.name),
                            excluded_name: Some(other.name),
                            question: format!(
                                "{}{}{}{}{punctuation}",
                                family.before, target.name, family.exclusion, other.name
                            ),
                            query_name_byte: family.before.len(),
                            the: None,
                            about: None,
                            reversed: Some(reversed),
                            distractor_first: Some(false),
                            object_swap: Some(object_swap),
                            connector_swap: Some(connector_swap),
                        });
                    }
                }
            }
        }
        for (stratum, name, before, after) in [
            (
                "unmentioned",
                Some(family.absent),
                "If the entries mention an object for ",
                ", repeat its exact wording.",
            ),
            (
                "conflict",
                Some(family.facts[0].name),
                "The two equally credible entries for ",
                " are mutually exclusive accounts, so can you determine the exact object?",
            ),
            (
                "unspecified",
                None,
                "Repeat the exact object wording for one of these people, keeping in mind that I mean one specific person.",
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
                object_swap: None,
                connector_swap: None,
            });
        }
    }
    rows
}
