#[derive(Clone, Copy)]
pub enum Split {
    Train,
    Held,
}

impl Split {
    pub fn name(self) -> &'static str {
        match self {
            Self::Train => "train",
            Self::Held => "held",
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum Role {
    Unspecified = 0,
    Requested = 1,
    Excluded = 2,
}

use Role::{Excluded, Requested, Unspecified};

pub const ROLE_PAIRS: [[Role; 2]; 8] = [
    [Requested, Excluded],
    [Excluded, Requested],
    [Requested, Requested],
    [Unspecified, Unspecified],
    [Requested, Unspecified],
    [Unspecified, Requested],
    [Excluded, Unspecified],
    [Unspecified, Excluded],
];

pub struct Panel {
    pub split: Split,
    pub names: &'static [[[&'static str; 2]; 2]],
    pub forms: usize,
}

pub const PANELS: [Panel; 2] = [
    Panel {
        split: Split::Train,
        names: &[
            [["Alice", "David"], ["Chris", "Barbara"]],
            [["John", "Mary"], ["Kevin", "Margaret"]],
            [["Sarah", "Thomas"], ["Linda", "Charles"]],
            [["Emma", "James"], ["Anna", "George"]],
        ],
        forms: 4,
    },
    Panel {
        split: Split::Held,
        names: &[
            [["Emily", "William"], ["Diana", "Patrick"]],
            [["Robert", "Helen"], ["Edward", "Susan"]],
            [["Henry", "Elizabeth"], ["Kate", "Daniel"]],
            [["Oliver", "Grace"], ["Victoria", "Michael"]],
        ],
        forms: 4,
    },
];

impl Panel {
    pub fn family(&self, form: usize) -> &'static str {
        ["near", "far", "root_order", "right"][form]
    }

    pub fn support(&self, group: usize) -> [bool; 2] {
        [[true, true], [true, false], [false, true], [true, true]][group]
    }

    pub fn items(&self) -> [&'static str; 2] {
        match self.split {
            Split::Train => ["blue cups", "green bowls"],
            Split::Held => ["brass lamps", "paper kites"],
        }
    }

    pub fn text(
        &self,
        names: &[&str; 2],
        group: usize,
        form: usize,
        roles: [Role; 2],
        ambiguous: bool,
        clause_order: usize,
        fact_order: usize,
    ) -> String {
        let items = self.items();
        let supported = self.support(group);
        let facts = |i: usize| {
            if supported[i] {
                format!("{} owns {}", names[i], items[i])
            } else {
                format!("the inventory lists {}", items[i])
            }
        };
        let clauses = if ambiguous {
            [
                format!(
                    "copy exactly one item belonging to {} or {}",
                    names[0], names[1]
                ),
                "the instruction leaves the choice unresolved".to_owned(),
            ]
        } else {
            [
                self.clause(names[0], form, roles[0]),
                self.clause(names[1], form, roles[1]),
            ]
        };
        let left = &clauses[clause_order];
        let right = &clauses[1 - clause_order];
        format!(
            "Facts. {}. {}. Return only the requested inventory items. {left}; {right}.",
            facts(fact_order),
            facts(1 - fact_order),
        )
    }

    fn clause(&self, name: &str, form: usize, role: Role) -> String {
        if role == Unspecified {
            let reference = match self.split {
                Split::Train => [
                    "the catalog mentions",
                    "the source material also refers to",
                    "another record briefly describes",
                    "the older inventory has an entry for",
                ],
                Split::Held => [
                    "an earlier note concerns",
                    "for reference there is also an entry concerning",
                    "the index provides general information on",
                    "elsewhere the archive contains a reference to",
                ],
            };
            return format!("{} {name}", reference[form]);
        }
        let (cue, opposite) = if role == Requested {
            ("keep", "drop")
        } else {
            ("drop", "keep")
        };
        match (self.split, form) {
            (Split::Train, 0) => format!("please {cue} {name}"),
            (Split::Train, 1) => {
                format!("please {cue} the item named in the earlier line for {name}")
            }
            (Split::Train, 2) => format!("please {cue} rather than {opposite} {name}"),
            (Split::Train, 3) => format!("for {name} please {cue} the item"),
            (Split::Held, 0) => {
                let directive = match role {
                    Requested => "include",
                    Excluded => "omit",
                    Unspecified => "aside about",
                };
                format!("{directive} {name}")
            }
            (Split::Held, 1) => format!("could you {cue} the exact thing listed above for {name}"),
            (Split::Held, 2) => format!("choose to {cue} instead of {opposite} {name}"),
            (Split::Held, 3) => {
                let ending = match role {
                    Requested => "the item should be included",
                    Excluded => "the item should be omitted",
                    Unspecified => "the item has a separate entry",
                };
                format!("for {name} {ending}")
            }
            _ => unreachable!("panel only defines four forms per split"),
        }
    }
}
