// Custody source. A, B, C, D are factual identities; R and X are requested/excluded.
pub const TRAIN: [[&str; 2]; 8] = [
    [
        "Fetch {R}'s entry. Leave {X}'s alone.",
        "Leave {X}'s entry alone and fetch {R}'s.",
    ],
    [
        "What did {R} claim, putting {X} aside?",
        "Putting {X} aside, what did {R} claim?",
    ],
    [
        "Give {R} priority over {X}: which item?",
        "Over {X}, give {R} priority: which item?",
    ],
    [
        "{R} gets the turn. Identify the object.",
        "Identify the object for this turn: {R}.",
    ],
    [
        "Tell us {R}'s choice, not {X}'s.",
        "Not {X}'s choice. Tell us {R}'s.",
    ],
    [
        "Pick what {R} listed. Disregard {X}.",
        "Disregard {X} and pick what {R} listed.",
    ],
    [
        "What was recorded under {R}? {B} observed.",
        "{B} observed. What was recorded under {R}?",
    ],
    [
        "Refer back to {R}. Which entry was that?",
        "Which entry was that? Refer back to {R}.",
    ],
];
pub const HELD: [[&str; 2]; 4] = [
    [
        "Name {R}'s item. {B} stayed nearby.",
        "{B} stayed nearby. Name {R}'s item.",
    ],
    ["Regarding {R}: item, {B}?", "{B}, regarding {R}: item?"],
    [
        "Omit {X}. {R}'s item, please.",
        "{R}'s item, please. Omit {X}.",
    ],
    ["No, {B}. {R}'s item, {C}?", "{R}'s item, {C}? No, {B}."],
];
pub const TRAIN_NAMES: [[&str; 4]; 4] = [
    ["Alice", "David", "Helen", "Frank"],
    ["Peter", "Susan", "Nancy", "Bruce"],
    ["Francesca", "Esteban", "Veronica", "Santiago"],
    ["Bernadette", "Anastasia", "Marcelino", "Guillermo"],
];
pub const HELD_NAMES: [[&str; 4]; 4] = [
    ["Mary", "John", "Amy", "Henry"],
    ["James", "Sarah", "Tim", "Paul"],
    ["Isabella", "Alejandro", "Mark", "Dmitri"],
    ["Evangeline", "Demetrius", "Tom", "Theodosia"],
];
pub const TRAIN_ADJ: [&str; 8] = [
    "navy", "silver", "amber", "indigo", "ivory", "bronze", "plum", "copper",
];
pub const HELD_ADJ: [&str; 8] = [
    "scarlet", "teal", "coral", "violet", "golden", "lavender", "peach", "crimson",
];
pub const TRAIN_THING: [&str; 8] = [
    "compass", "lantern", "spoon", "comb", "mug", "sash", "ribbon", "pouch",
];
pub const HELD_THING: [&str; 8] = [
    "flask", "whistle", "kite", "mitt", "pencil", "bucket", "basket", "scarf",
];
pub const TRAIN_PROSE: [(&str, &str); 8] = [
    (
        "How does frost form?",
        "Water vapor deposits as ice on a surface below freezing.",
    ),
    (
        "Why do most leaves look green?",
        "Chlorophyll absorbs red and blue and reflects green.",
    ),
    ("What causes a shadow?", "An object blocks light."),
    (
        "Why does salt dissolve in water?",
        "Water molecules surround and separate the ions.",
    ),
    (
        "What does a compass indicate?",
        "Its needle aligns with the local magnetic field.",
    ),
    (
        "Why is a window transparent?",
        "Glass transmits much of the incoming visible light.",
    ),
    (
        "How does a sail catch wind?",
        "Airflow produces a force on the sail.",
    ),
    (
        "What are clouds made of?",
        "Suspended tiny water droplets or ice crystals.",
    ),
];
pub const HELD_PROSE: [(&str, &str); 8] = [
    (
        "What makes an echo audible?",
        "Reflected sound reaches the listener after a delay.",
    ),
    (
        "What makes a wool coat warm?",
        "Trapped air slows heat loss.",
    ),
    (
        "Why do some rocks contain bands?",
        "Different minerals or layers accumulated or rearranged.",
    ),
    (
        "Why can a needle float on water?",
        "Surface tension can support it.",
    ),
    (
        "How does a pulley redirect force?",
        "The rope runs around a wheel and changes pulling direction.",
    ),
    (
        "Why does moonlight look dimmer than sunlight?",
        "The Moon reflects only a small fraction of sunlight back to us.",
    ),
    (
        "How does a sand dune move?",
        "Wind moves grains from the windward side toward the lee side.",
    ),
    ("Why do raindrops fall?", "Gravity pulls them toward Earth."),
];
