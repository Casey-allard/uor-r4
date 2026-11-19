// Each item lists nested natural descriptions of one head, shortest to longest.
pub struct Family {
    pub split: &'static str,
    pub key: &'static str,
    pub people: [&'static str; 2],
    pub connectors: [&'static str; 3],
    pub items: [[&'static str; 3]; 2],
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
        items: [
            ["dolls", "cloth dolls", "blue cloth dolls"],
            ["boxes", "paper boxes", "green paper boxes"],
        ],
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
        items: [
            ["frames", "wood frames", "small wood frames"],
            ["covers", "cloth covers", "dark cloth covers"],
        ],
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
        people: ["Marceline", "Alaric"],
        connectors: ["painted", "has painted", "has now painted"],
        items: [
            ["rods", "wire rods", "thin wire rods"],
            ["tiles", "glass tiles", "fine glass tiles"],
        ],
        ask: ["Tell me the full item description from the entry for ", "."],
        absent: "Lysandra",
        ambiguity: "What is the item in the entry for the person I mean?",
    },
    Family {
        split: "held",
        key: "accident",
        people: ["Winifred", "Silas"],
        connectors: ["added", "had added", "had once added"],
        items: [
            ["slats", "white slats", "long white slats"],
            ["strips", "black strips", "hard black strips"],
        ],
        ask: [
            "What complete description of an object do these statements give for ",
            "?",
        ],
        absent: "Pelagia",
        ambiguity: "Can you name the exact object described for the person I mean?",
    },
];
