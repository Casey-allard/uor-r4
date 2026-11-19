// Authored ordinary storage statements, independent of any selector result.
#[derive(Clone, Copy)]
pub struct Fact {
    pub person: &'static str,
    pub verb: &'static str,
    pub literal: &'static str,
    pub literal_head: &'static str,
}

#[derive(Clone, Copy)]
pub struct Family {
    pub key: &'static str,
    pub people: [&'static str; 2],
    pub verbs: [&'static str; 2],
    pub items: [(&'static str, &'static str); 2],
    pub query: [&'static str; 2],
}
pub const FACTORIAL: [Family; 2] = [
    Family {
        key: "reserve",
        people: ["Inez", "Dorian"],
        verbs: ["saved", "set aside"],
        items: [("woven baskets", "baskets"), ("striped napkins", "napkins")],
        query: [
            "According to the two entries, what is the full item description listed for ",
            "?",
        ],
    },
    Family {
        key: "store",
        people: ["Sabine", "Gareth"],
        verbs: ["packed", "put away"],
        items: [
            ("padded envelopes", "envelopes"),
            ("braided cords", "cords"),
        ],
        query: ["What did ", " pack or put away?"],
    },
];

pub struct Pair {
    pub key: &'static str,
    pub first: Fact,
    pub second: Fact,
    pub ask: [&'static str; 2],
}

pub const STRUCTURED: [Pair; 4] = [
    Pair {
        key: "counter",
        first: Fact {
            person: "Vale",
            verb: "saved",
            literal: "brass coins",
            literal_head: "coins",
        },
        second: Fact {
            person: "Tessa",
            verb: "put away",
            literal: "paper cups",
            literal_head: "cups",
        },
        ask: ["Which objects were recorded for ", "?"],
    },
    Pair {
        key: "tray",
        first: Fact {
            person: "Della",
            verb: "gathered",
            literal: "river stones",
            literal_head: "stones",
        },
        second: Fact {
            person: "Wynn",
            verb: "set aside",
            literal: "spare cards",
            literal_head: "cards",
        },
        ask: ["From those two lines, what item was recorded for ", "?"],
    },
    Pair {
        key: "shelves",
        first: Fact {
            person: "Harlan",
            verb: "packed",
            literal: "clay pots",
            literal_head: "pots",
        },
        second: Fact {
            person: "Esme",
            verb: "put away",
            literal: "rubber stamps",
            literal_head: "stamps",
        },
        ask: ["Please identify the supplies listed for ", "."],
    },
    Pair {
        key: "tally",
        first: Fact {
            person: "Briar",
            verb: "set aside",
            literal: "chalk sticks",
            literal_head: "sticks",
        },
        second: Fact {
            person: "Cleo",
            verb: "saved",
            literal: "glass jars",
            literal_head: "jars",
        },
        ask: ["Which objects should the tally show for ", "?"],
    },
];

pub const ABSENT: [&str; 4] = ["Nina", "Omar", "Lena", "Rafael"];

pub const CONFLICT_ALTERNATIVES: [(&str, &str); 4] = [
    ("tin tokens", "tokens"),
    ("round pebbles", "pebbles"),
    ("silk scarves", "scarves"),
    ("wooden beads", "beads"),
];

pub const CONFLICT_ASK: [&str; 2] = [
    "The two records for ",
    " are competing versions of the same entry. Can the original item be identified?",
];
pub const ABSENT_ASK: [&str; 2] = ["Do these entries identify any supplies for ", "?"];
