// Each item lists nested noun phrases from shortest to longest.
// Every verb phrase describes one direct object, with no possession reading.
pub struct Family {
    pub key: &'static str,
    pub people: [&'static str; 2],
    pub connectors: [&'static str; 3],
    pub items: [[&'static str; 3]; 2],
    pub ask: [&'static str; 2],
    pub absent: &'static str,
    pub ambiguity: &'static str,
}

pub const FAMILIES: [Family; 2] = [
    Family {
        key: "discovery",
        people: ["Briony", "Caspian"],
        connectors: ["found", "might find", "might still find"],
        items: [
            ["bells", "metal bells", "heavy metal bells"],
            ["plates", "red plates", "round red plates"],
        ],
        ask: [
            "Quote the complete object named in the sentence about ",
            ".",
        ],
        absent: "Romilly",
        ambiguity: "Give the complete object named for the person I am thinking of.",
    },
    Family {
        key: "selection",
        people: ["Linnea", "Oswin"],
        connectors: ["took", "could take", "could already take"],
        items: [
            ["coins", "gold coins", "old gold coins"],
            ["rings", "pink rings", "large pink rings"],
        ],
        ask: [
            "Which full object description belongs to the statement about ",
            "?",
        ],
        absent: "Odette",
        ambiguity: "Which exact object was associated with whoever I am asking about?",
    },
];
