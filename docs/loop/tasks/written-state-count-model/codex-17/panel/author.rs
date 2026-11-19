// The surface assertions and intended item boundaries were authored together.
pub struct Family {
    pub split: &'static str,
    pub key: &'static str,
    pub people: [&'static str; 2],
    pub connectors: [&'static str; 3],
    pub items: [&'static str; 2],
    pub ask: [&'static str; 2],
    pub absent: &'static str,
    pub ambiguity: &'static str,
}

pub const FAMILIES: [Family; 4] = [
    Family {
        split: "fit",
        key: "market",
        people: ["Leonie", "Norbert"],
        connectors: ["bought", "did buy", "did just buy"],
        items: ["blue cloth dolls", "green paper boxes"],
        ask: [
            "For ",
            ", which complete item description appears in the record?",
        ],
        absent: "Mireille",
        ambiguity: "Which object did the person I have in mind deal with?",
    },
    Family {
        split: "fit",
        key: "moving",
        people: ["Rowena", "Cedric"],
        connectors: ["moved", "did move", "did just move"],
        items: ["small wood frames", "dark cloth covers"],
        ask: [
            "In the entries above, what is the full named object for ",
            "?",
        ],
        absent: "Ewan",
        ambiguity: "Which of the two objects did the intended person handle?",
    },
    Family {
        split: "held",
        key: "parcel",
        people: ["Ariadne", "Malcolm"],
        connectors: ["wrapped", "has wrapped", "has now wrapped"],
        items: ["round wire loops", "fine gold chains"],
        ask: ["Tell me the full item description from the entry for ", "."],
        absent: "Aveline",
        ambiguity: "What is the item in the entry for the person I mean?",
    },
    Family {
        split: "held",
        key: "accident",
        people: ["Beatrix", "Fergus"],
        connectors: ["dropped", "had dropped", "had once dropped"],
        items: ["long white feathers", "hard black balls"],
        ask: [
            "What complete description of an object do these statements give for ",
            "?",
        ],
        absent: "Sylvester",
        ambiguity: "Can you name the complete object belonging to whichever person I meant?",
    },
];
