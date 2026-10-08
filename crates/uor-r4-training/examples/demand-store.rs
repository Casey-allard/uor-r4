//! Build a demand-bearing short-form dialogue store.
//!
//! ```text
//! demand-store <output-dir> [--tokenizer <tokenizer.json>] [--documents N] [--seed N]
//! ```
//!
//! The project holds the two halves of this data separately. The 82 M-token chat corpus contains
//! answer-form instructions whose replies are long prose; the synthetic store contains short
//! canonical answers whose turns demand no form. This tool writes the missing combination
//! directly, so a geometric language model can see an explicit answer-form instruction in the
//! prefix and only the demanded short form in the response:
//!
//! ```text
//! BOS  User:  <question>  \n  Assistant:  <short answer>  EOS
//! mask 0      mask 0          mask 0      mask 1          mask 1
//! ```
//!
//! The role markers are the literal version-2 dialogue protocol strings, rendered as their exact
//! token IDs (`User:` = [55, 2728, 28], `Assistant:` = [35, 560, 652, 714, 28]); the tokenizer is
//! asked for those IDs and the run refuses to write if it does not get them. Forms covered: one
//! word, yes/no, a number, a short comma-separated list and a short factual name. Every answer
//! matches its demand by construction, so the response region never teaches prose in reply to a
//! short-form instruction.
//!
//! The store is written only after an in-tool validation pass reproduces the training-time
//! contract of `crates/uor-r4-training/src/dialogue_episodes.rs` (`record_response`) clause by
//! clause: one masked run per document, the five assistant marker IDs immediately before it and
//! unmasked, a genuine terminal EOS at the end of the run, no EOS inside it, an unmasked BOS at
//! every document start after the previous EOS, and no literal special-token IDs in content. A
//! store that fails any clause is refused, printing the offending document index and clause name;
//! nothing is written in that case. After writing, the store is re-read from disk and the real
//! `EpisodeIndex` is built over it, so acceptance is the training path's own check, not a copy.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use uor_r4_core::native_geometric::mmap_corpus::{
    CorpusWriter, MmapCorpusReader, CORPUS_HEADER_SIZE,
};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;
use uor_r4_training::dialogue_episodes::EpisodeContract;
use uor_r4_training::stack_dialogue::DialogueSplit;

/// The store's declared vocabulary, as the dialogue corpus binding requires.
const VOCAB_SIZE: u32 = 4096;
/// The bound special IDs: BOS 0 (read empirically from the chat corpus's first token), EOS 1,
/// UNK 2. BOS is not 1.
const BOS_ID: u32 = 0;
const EOS_ID: u32 = 1;
const UNK_ID: u32 = 2;
/// A real in-vocabulary filler for the episode contract's padding slot; never emitted.
const PADDING_ID: u32 = 3;
/// Literal `User:` and `Assistant:` under the version-2 protocol.
const USER_MARKER: [u32; 3] = [55, 2728, 28];
const ASSISTANT_MARKER: [u32; 5] = [35, 560, 652, 714, 28];
/// The canonical assistant-turn separator before the marker (unmasked).
const SEPARATOR: &str = "\n";
/// Full-prefix episode context, as the retained dialogue study uses.
const EPISODE_CONTEXT: usize = 256;
/// The manifest source label.
const LABEL: &str = "demand";
const DEFAULT_SEED: u64 = 0x6465_6d61_6e64;
/// How many offending documents the refusal prints before summarizing.
const MAX_REPORTED_OFFENDERS: usize = 20;

/// One question/answer pair: the answer is exactly the demanded form.
struct Item {
    answer: &'static str,
    questions: &'static [&'static str],
}

/// One demanded answer form: the instruction phrasings and the pairs they wrap.
struct Form {
    label: &'static str,
    demands: &'static [&'static str],
    items: &'static [Item],
}

const ONE_WORD_DEMANDS: &[&str] = &[
    "Answer with one word only. {Q}",
    "Reply with exactly one word. {Q}",
    "Give a one-word answer. {Q}",
    "Answer in a single word. {Q}",
    "Respond with just one word. {Q}",
    "Use one word only. {Q}",
    "One word answer, please. {Q}",
    "Answer using exactly one word. {Q}",
    "Keep it to one word. {Q}",
    "Answer with only one word, nothing else. {Q}",
];

const ONE_WORD_ITEMS: &[Item] = &[
    Item {
        answer: "Green.",
        questions: &[
            "What colour is grass?",
            "What is the colour of grass?",
            "Which colour is grass?",
        ],
    },
    Item {
        answer: "Honey.",
        questions: &[
            "What do bees make?",
            "What is made by bees?",
            "What do bees produce?",
        ],
    },
    Item {
        answer: "Blue.",
        questions: &[
            "What colour is a clear daytime sky?",
            "What colour is the sky on a clear day?",
            "Which colour is the sky on a sunny day?",
        ],
    },
    Item {
        answer: "Water.",
        questions: &[
            "What do fish swim in?",
            "What do fish live in?",
            "What liquid do fish live in?",
        ],
    },
    Item {
        answer: "Snow.",
        questions: &[
            "What falls from clouds when it is freezing?",
            "What is frozen rain called?",
            "What white flakes fall in winter?",
        ],
    },
    Item {
        answer: "Winter.",
        questions: &[
            "Which season is the coldest?",
            "Which season brings snow?",
            "Which season comes after autumn?",
        ],
    },
    Item {
        answer: "Sun.",
        questions: &[
            "Which star is closest to Earth?",
            "Which star lights the day?",
            "What star do we see in the daytime?",
        ],
    },
    Item {
        answer: "Milk.",
        questions: &[
            "What do cows produce?",
            "What drink comes from cows?",
            "What white drink comes from a cow?",
        ],
    },
    Item {
        answer: "Yellow.",
        questions: &[
            "What colour is a ripe banana?",
            "What is the colour of a ripe banana?",
            "Which colour is a ripe banana?",
        ],
    },
    Item {
        answer: "Red.",
        questions: &[
            "What colour is a ripe tomato?",
            "What is the colour of blood?",
            "Which colour is a stop sign?",
        ],
    },
    Item {
        answer: "Ice.",
        questions: &[
            "What is frozen water called?",
            "What do you get when water freezes?",
            "What is water when it is solid?",
        ],
    },
    Item {
        answer: "Rain.",
        questions: &[
            "What falls from clouds?",
            "What falls from the sky when it rains?",
            "What is falling water from clouds called?",
        ],
    },
    Item {
        answer: "Hot.",
        questions: &[
            "What is the opposite of cold?",
            "Which word is the opposite of cold?",
        ],
    },
    Item {
        answer: "Night.",
        questions: &[
            "What is the opposite of day?",
            "Which word is the opposite of day?",
        ],
    },
    Item {
        answer: "Triangle.",
        questions: &[
            "What shape has three sides?",
            "Which shape has exactly three sides?",
            "What is a three-sided shape called?",
        ],
    },
    Item {
        answer: "Square.",
        questions: &[
            "What shape has four equal sides?",
            "Which shape has four equal sides?",
        ],
    },
    Item {
        answer: "Feathers.",
        questions: &[
            "What covers a bird's body?",
            "What is a bird's body covered with?",
        ],
    },
    Item {
        answer: "Oxygen.",
        questions: &[
            "Which gas do humans breathe in?",
            "What gas do people need to breathe?",
            "Which gas do we breathe to stay alive?",
        ],
    },
    Item {
        answer: "Sahara.",
        questions: &[
            "What is the largest hot desert called?",
            "Which desert is the largest hot desert?",
        ],
    },
    Item {
        answer: "Everest.",
        questions: &[
            "What is the highest mountain called?",
            "Which mountain is the tallest on Earth?",
        ],
    },
    Item {
        answer: "Piano.",
        questions: &[
            "Which instrument has 88 keys?",
            "What instrument has black and white keys?",
            "Which instrument do you play with keys?",
        ],
    },
    Item {
        answer: "Nile.",
        questions: &[
            "Which river is the longest in Africa?",
            "What is the longest river in Africa called?",
        ],
    },
    Item {
        answer: "Gold.",
        questions: &[
            "Which metal is a wedding ring often made of?",
            "What yellow metal is used in jewellery?",
        ],
    },
    Item {
        answer: "Woof.",
        questions: &["What sound does a dog make?", "What noise does a dog make?"],
    },
    Item {
        answer: "Meow.",
        questions: &["What sound does a cat make?", "What noise does a cat make?"],
    },
    Item {
        answer: "Trunk.",
        questions: &[
            "What is an elephant's long nose called?",
            "What do you call an elephant's long nose?",
        ],
    },
];

const YES_NO_DEMANDS: &[&str] = &[
    "Answer yes or no. {Q}",
    "Reply with yes or no. {Q}",
    "Answer only yes or no. {Q}",
    "Give a yes or no answer. {Q}",
    "Answer with a single word, yes or no. {Q}",
    "Say yes or no. {Q}",
    "Reply with a single word: yes or no. {Q}",
    "Answer yes or no only. {Q}",
    "Just say yes or no. {Q}",
    "Answer with yes or no and nothing more. {Q}",
];

const YES_NO_ITEMS: &[Item] = &[
    Item {
        answer: "Yes.",
        questions: &[
            "Is the sky blue?",
            "Is the sky blue on a clear day?",
            "Is the daytime sky blue?",
        ],
    },
    Item {
        answer: "No.",
        questions: &[
            "Do fish live in trees?",
            "Can fish live in trees?",
            "Do fish live in trees like birds?",
        ],
    },
    Item {
        answer: "Yes.",
        questions: &["Is water wet?", "Is water a liquid at room temperature?"],
    },
    Item {
        answer: "Yes.",
        questions: &["Do birds have feathers?", "Do all birds have feathers?"],
    },
    Item {
        answer: "No.",
        questions: &["Do cats bark?", "Can cats bark?"],
    },
    Item {
        answer: "Yes.",
        questions: &[
            "Does the sun rise in the east?",
            "Does the sun come up in the east?",
        ],
    },
    Item {
        answer: "No.",
        questions: &["Is ice hot?", "Is ice warm to the touch?"],
    },
    Item {
        answer: "Yes.",
        questions: &["Is two an even number?", "Is the number two even?"],
    },
    Item {
        answer: "No.",
        questions: &["Is seven an even number?", "Is the number seven even?"],
    },
    Item {
        answer: "Yes.",
        questions: &[
            "Do humans need water to live?",
            "Is water necessary for human life?",
        ],
    },
    Item {
        answer: "No.",
        questions: &[
            "Does a square have three sides?",
            "Does a square have exactly three sides?",
        ],
    },
    Item {
        answer: "Yes.",
        questions: &[
            "Does a triangle have three sides?",
            "Does a triangle have exactly three sides?",
        ],
    },
    Item {
        answer: "Yes.",
        questions: &["Is Paris in France?", "Is Paris the capital of France?"],
    },
    Item {
        answer: "No.",
        questions: &["Is Paris in Germany?", "Is Paris the capital of Germany?"],
    },
    Item {
        answer: "Yes.",
        questions: &[
            "Do spiders have eight legs?",
            "Does a spider have eight legs?",
        ],
    },
    Item {
        answer: "No.",
        questions: &["Do spiders have six legs?", "Does a spider have six legs?"],
    },
    Item {
        answer: "Yes.",
        questions: &["Is the Earth round?", "Is the Earth a sphere?"],
    },
    Item {
        answer: "No.",
        questions: &[
            "Is the Earth flat?",
            "Is the Earth shaped like a flat disc?",
        ],
    },
    Item {
        answer: "Yes.",
        questions: &[
            "Does a week have seven days?",
            "Are there seven days in a week?",
        ],
    },
    Item {
        answer: "Yes.",
        questions: &["Is a banana a fruit?", "Is a banana a kind of fruit?"],
    },
    Item {
        answer: "No.",
        questions: &["Is a carrot a fruit?", "Is a carrot a kind of fruit?"],
    },
    Item {
        answer: "Yes.",
        questions: &["Do mammals breathe air?", "Do all mammals breathe air?"],
    },
    Item {
        answer: "No.",
        questions: &["Can penguins fly?", "Are penguins able to fly?"],
    },
    Item {
        answer: "Yes.",
        questions: &["Does a dog have four legs?", "Do dogs have four legs?"],
    },
    Item {
        answer: "No.",
        questions: &["Is a whale a fish?", "Is a whale a kind of fish?"],
    },
    Item {
        answer: "Yes.",
        questions: &[
            "Does the Earth have one moon?",
            "Does Earth have exactly one moon?",
        ],
    },
    Item {
        answer: "No.",
        questions: &["Is the sun a planet?", "Is the sun a planet like Earth?"],
    },
    Item {
        answer: "Yes.",
        questions: &[
            "Is a triangle a polygon?",
            "Is a triangle a kind of polygon?",
        ],
    },
];

const NUMBER_DEMANDS: &[&str] = &[
    "Answer with a number only. {Q}",
    "Reply with just a number. {Q}",
    "Answer with digits only. {Q}",
    "Give a numeric answer only. {Q}",
    "Answer using a single number. {Q}",
    "Respond with a number and nothing else. {Q}",
    "Answer with only a number. {Q}",
    "Give the number alone. {Q}",
    "Reply with the number only. {Q}",
    "Answer with one number, no words. {Q}",
];

const NUMBER_ITEMS: &[Item] = &[
    Item {
        answer: "7.",
        questions: &[
            "How many days are in a week?",
            "How many days are there in a week?",
        ],
    },
    Item {
        answer: "4.",
        questions: &[
            "What is two plus two?",
            "What does two plus two equal?",
            "How much is two plus two?",
        ],
    },
    Item {
        answer: "12.",
        questions: &[
            "How many months are in a year?",
            "How many months are there in a year?",
        ],
    },
    Item {
        answer: "60.",
        questions: &[
            "How many minutes are in an hour?",
            "How many minutes are there in one hour?",
        ],
    },
    Item {
        answer: "24.",
        questions: &[
            "How many hours are in a day?",
            "How many hours are there in a day?",
        ],
    },
    Item {
        answer: "365.",
        questions: &[
            "How many days are in a common year?",
            "How many days are there in a non-leap year?",
        ],
    },
    Item {
        answer: "10.",
        questions: &[
            "How many fingers are on two hands?",
            "How many fingers do two hands have?",
        ],
    },
    Item {
        answer: "5.",
        questions: &[
            "How many sides does a pentagon have?",
            "How many sides are on a pentagon?",
        ],
    },
    Item {
        answer: "6.",
        questions: &[
            "How many sides does a hexagon have?",
            "How many sides are on a hexagon?",
        ],
    },
    Item {
        answer: "3.",
        questions: &[
            "How many sides does a triangle have?",
            "How many sides are on a triangle?",
        ],
    },
    Item {
        answer: "8.",
        questions: &[
            "How many legs does a spider have?",
            "How many legs do spiders have?",
        ],
    },
    Item {
        answer: "2.",
        questions: &[
            "How many wheels does a bicycle have?",
            "How many wheels are on a bicycle?",
        ],
    },
    Item {
        answer: "4.",
        questions: &[
            "How many wheels does a car have?",
            "How many wheels are on a car?",
        ],
    },
    Item {
        answer: "100.",
        questions: &[
            "How many cents are in a dollar?",
            "How many cents make one dollar?",
        ],
    },
    Item {
        answer: "1000.",
        questions: &[
            "How many metres are in a kilometre?",
            "How many metres make one kilometre?",
        ],
    },
    Item {
        answer: "9.",
        questions: &[
            "What is three times three?",
            "What does three times three equal?",
        ],
    },
    Item {
        answer: "20.",
        questions: &["What is ten plus ten?", "How much is ten plus ten?"],
    },
    Item {
        answer: "15.",
        questions: &[
            "What is five times three?",
            "What does five times three equal?",
        ],
    },
    Item {
        answer: "50.",
        questions: &[
            "How many states are in the United States?",
            "How many states does the United States have?",
        ],
    },
    Item {
        answer: "26.",
        questions: &[
            "How many letters are in the English alphabet?",
            "How many letters does the English alphabet have?",
        ],
    },
    Item {
        answer: "11.",
        questions: &[
            "How many players are on a soccer team?",
            "How many players are on a football team?",
        ],
    },
    Item {
        answer: "6.",
        questions: &[
            "How many strings does a standard guitar have?",
            "How many strings are on a standard guitar?",
        ],
    },
    Item {
        answer: "64.",
        questions: &[
            "How many squares are on a chessboard?",
            "How many squares does a chessboard have?",
        ],
    },
    Item {
        answer: "90.",
        questions: &[
            "How many degrees are in a right angle?",
            "How many degrees does a right angle have?",
        ],
    },
    Item {
        answer: "180.",
        questions: &[
            "What is the sum of the angles of a triangle in degrees?",
            "How many degrees do a triangle's angles add up to?",
        ],
    },
    Item {
        answer: "0.",
        questions: &[
            "What is five minus five?",
            "What does five minus five equal?",
        ],
    },
    Item {
        answer: "1.",
        questions: &["What is one plus zero?", "How much is one plus zero?"],
    },
];

const SHORT_LIST_DEMANDS: &[&str] = &[
    "Answer with a short list separated by commas. {Q}",
    "Give three items separated by commas. {Q}",
    "List exactly three items, separated by commas. {Q}",
    "Answer with a comma-separated list. {Q}",
    "Give a short comma-separated answer. {Q}",
    "Reply with a list of three items separated by commas. {Q}",
    "Answer with three items, comma-separated. {Q}",
    "Give exactly three answers separated by commas. {Q}",
    "Respond with a short comma-separated list. {Q}",
    "List three items, separated by commas, and nothing else. {Q}",
];

const SHORT_LIST_ITEMS: &[Item] = &[
    Item {
        answer: "Red, green, blue.",
        questions: &["Name three colours.", "List three colours."],
    },
    Item {
        answer: "Apple, banana, orange.",
        questions: &["Name three fruits.", "List three fruits."],
    },
    Item {
        answer: "Carrot, potato, onion.",
        questions: &["Name three vegetables.", "List three vegetables."],
    },
    Item {
        answer: "Cat, dog, horse.",
        questions: &["Name three animals.", "List three animals."],
    },
    Item {
        answer: "Lion, tiger, bear.",
        questions: &["Name three wild animals.", "List three wild animals."],
    },
    Item {
        answer: "One, two, three.",
        questions: &["Name three numbers.", "List three numbers."],
    },
    Item {
        answer: "Monday, Tuesday, Wednesday.",
        questions: &[
            "Name three days of the week.",
            "List three days of the week.",
        ],
    },
    Item {
        answer: "January, February, March.",
        questions: &["Name three months.", "List three months of the year."],
    },
    Item {
        answer: "Spring, summer, autumn.",
        questions: &["Name three seasons.", "List three seasons."],
    },
    Item {
        answer: "Rose, tulip, daisy.",
        questions: &["Name three flowers.", "List three flowers."],
    },
    Item {
        answer: "Oak, pine, maple.",
        questions: &["Name three trees.", "List three trees."],
    },
    Item {
        answer: "Bus, train, tram.",
        questions: &[
            "Name three kinds of public transport.",
            "List three types of public transport.",
        ],
    },
    Item {
        answer: "Car, van, lorry.",
        questions: &["Name three road vehicles.", "List three road vehicles."],
    },
    Item {
        answer: "Gold, silver, bronze.",
        questions: &[
            "Name three medal colours.",
            "List three Olympic medal colours.",
        ],
    },
    Item {
        answer: "Earth, Mars, Venus.",
        questions: &[
            "Name three planets.",
            "List three planets of the solar system.",
        ],
    },
    Item {
        answer: "Piano, violin, flute.",
        questions: &[
            "Name three musical instruments.",
            "List three musical instruments.",
        ],
    },
    Item {
        answer: "Knife, fork, spoon.",
        questions: &[
            "Name three items of cutlery.",
            "List three pieces of cutlery.",
        ],
    },
    Item {
        answer: "Shirt, trousers, jacket.",
        questions: &[
            "Name three items of clothing.",
            "List three pieces of clothing.",
        ],
    },
    Item {
        answer: "Soccer, tennis, golf.",
        questions: &["Name three sports.", "List three sports."],
    },
    Item {
        answer: "Red, blue, yellow.",
        questions: &["Name three primary colours.", "List three primary colours."],
    },
    Item {
        answer: "North, south, east.",
        questions: &[
            "Name three compass directions.",
            "List three points of the compass.",
        ],
    },
    Item {
        answer: "Inch, foot, yard.",
        questions: &[
            "Name three units of length.",
            "List three imperial units of length.",
        ],
    },
    Item {
        answer: "Gram, kilogram, tonne.",
        questions: &[
            "Name three units of mass.",
            "List three metric units of mass.",
        ],
    },
    Item {
        answer: "Mercury, Venus, Earth.",
        questions: &[
            "Name the first three planets from the sun.",
            "List the three planets closest to the sun.",
        ],
    },
    Item {
        answer: "Breakfast, lunch, dinner.",
        questions: &[
            "Name three meals of the day.",
            "List three meals of the day.",
        ],
    },
    Item {
        answer: "Red, orange, yellow.",
        questions: &["Name three warm colours.", "List three warm colours."],
    },
];

const SHORT_FACTUAL_DEMANDS: &[&str] = &[
    "Answer with the name only. {Q}",
    "Give just the name. {Q}",
    "Answer in a few words. {Q}",
    "Answer briefly. {Q}",
    "Give the shortest correct answer. {Q}",
    "Answer with a short phrase only. {Q}",
    "Reply briefly with the name. {Q}",
    "Answer with just the word or phrase. {Q}",
    "Keep the answer very short. {Q}",
    "Give a short answer, a name or phrase. {Q}",
];

const SHORT_FACTUAL_ITEMS: &[Item] = &[
    Item {
        answer: "Paris.",
        questions: &[
            "What is the capital of France?",
            "Which city is the capital of France?",
        ],
    },
    Item {
        answer: "London.",
        questions: &[
            "What is the capital of the United Kingdom?",
            "Which city is the capital of England?",
        ],
    },
    Item {
        answer: "Rome.",
        questions: &[
            "What is the capital of Italy?",
            "Which city is the capital of Italy?",
        ],
    },
    Item {
        answer: "Madrid.",
        questions: &[
            "What is the capital of Spain?",
            "Which city is the capital of Spain?",
        ],
    },
    Item {
        answer: "Tokyo.",
        questions: &[
            "What is the capital of Japan?",
            "Which city is the capital of Japan?",
        ],
    },
    Item {
        answer: "Berlin.",
        questions: &[
            "What is the capital of Germany?",
            "Which city is the capital of Germany?",
        ],
    },
    Item {
        answer: "Ottawa.",
        questions: &[
            "What is the capital of Canada?",
            "Which city is the capital of Canada?",
        ],
    },
    Item {
        answer: "Canberra.",
        questions: &[
            "What is the capital of Australia?",
            "Which city is the capital of Australia?",
        ],
    },
    Item {
        answer: "Nile.",
        questions: &[
            "What is the longest river in Africa?",
            "Which river is the longest in Africa?",
        ],
    },
    Item {
        answer: "Amazon.",
        questions: &[
            "What is the largest rainforest called?",
            "Which rainforest is the largest on Earth?",
        ],
    },
    Item {
        answer: "Everest.",
        questions: &[
            "What is the highest mountain in the world?",
            "Which mountain is the highest on Earth?",
        ],
    },
    Item {
        answer: "Pacific.",
        questions: &[
            "What is the largest ocean called?",
            "Which ocean is the largest?",
        ],
    },
    Item {
        answer: "Kilimanjaro.",
        questions: &[
            "What is the highest mountain in Africa?",
            "Which mountain is the highest in Africa?",
        ],
    },
    Item {
        answer: "Shakespeare.",
        questions: &[
            "Who wrote Romeo and Juliet?",
            "Who is the author of Romeo and Juliet?",
        ],
    },
    Item {
        answer: "Einstein.",
        questions: &[
            "Who developed the theory of relativity?",
            "Who is famous for the theory of relativity?",
        ],
    },
    Item {
        answer: "Newton.",
        questions: &[
            "Who formulated the law of gravity?",
            "Who is famous for the law of gravity?",
        ],
    },
    Item {
        answer: "Au.",
        questions: &[
            "What is the chemical symbol for gold?",
            "Which chemical symbol stands for gold?",
        ],
    },
    Item {
        answer: "NaCl.",
        questions: &[
            "What is the chemical formula for table salt?",
            "Which chemical formula stands for table salt?",
        ],
    },
    Item {
        answer: "Mercury.",
        questions: &[
            "Which planet is closest to the sun?",
            "What is the closest planet to the sun called?",
        ],
    },
    Item {
        answer: "Jupiter.",
        questions: &[
            "Which planet is the largest?",
            "What is the largest planet in the solar system called?",
        ],
    },
    Item {
        answer: "Hydrogen.",
        questions: &[
            "What is the lightest element?",
            "Which element is the lightest?",
        ],
    },
    Item {
        answer: "Blue whale.",
        questions: &[
            "What is the largest animal on Earth?",
            "Which animal is the largest on Earth?",
        ],
    },
    Item {
        answer: "Eiffel Tower.",
        questions: &[
            "What famous landmark stands in Paris?",
            "Which landmark is the symbol of Paris?",
        ],
    },
    Item {
        answer: "Mount Fuji.",
        questions: &[
            "What is the highest mountain in Japan?",
            "Which mountain is the highest in Japan?",
        ],
    },
    Item {
        answer: "Photosynthesis.",
        questions: &[
            "What process do plants use to make food?",
            "What is the process called when plants make food from sunlight?",
        ],
    },
    Item {
        answer: "Portuguese.",
        questions: &[
            "What language is spoken in Portugal?",
            "Which language do people speak in Portugal?",
        ],
    },
    Item {
        answer: "Canberra.",
        questions: &[
            "Which city is Australia's capital?",
            "What city is the seat of Australia's government?",
        ],
    },
];

const FORMS: [Form; 5] = [
    Form {
        label: "one_word",
        demands: ONE_WORD_DEMANDS,
        items: ONE_WORD_ITEMS,
    },
    Form {
        label: "yes_no",
        demands: YES_NO_DEMANDS,
        items: YES_NO_ITEMS,
    },
    Form {
        label: "number",
        demands: NUMBER_DEMANDS,
        items: NUMBER_ITEMS,
    },
    Form {
        label: "short_list",
        demands: SHORT_LIST_DEMANDS,
        items: SHORT_LIST_ITEMS,
    },
    Form {
        label: "short_factual",
        demands: SHORT_FACTUAL_DEMANDS,
        items: SHORT_FACTUAL_ITEMS,
    },
];

/// Every clause the store must satisfy. The names mirror the training-time checks in
/// `dialogue_episodes.rs`; the counter for each must end at zero or the store is refused.
const CLAUSES: [&str; 16] = [
    "store_mask_length",
    "document_bos_unmasked",
    "document_bos_follows_eos",
    "document_terminal_masked_eos",
    "document_no_interior_eos",
    "document_no_interior_bos",
    "document_vocabulary",
    "document_mask_binary",
    "user_marker_exact",
    "response_single_run",
    "response_nonempty",
    "response_marker_present",
    "response_marker_unmasked",
    "response_marker_exact",
    "response_terminal_eos",
    "response_no_interior_eos",
];

struct Document {
    form: usize,
    question: String,
    answer: String,
    tokens: Vec<u16>,
    mask: Vec<u8>,
}

struct Args {
    output: PathBuf,
    tokenizer: PathBuf,
    documents: Option<usize>,
    seed: u64,
}

struct Offender {
    document: usize,
    clause: &'static str,
    detail: String,
}

struct Ledger {
    counts: Vec<(&'static str, usize)>,
    offenders: Vec<Offender>,
}

impl Ledger {
    fn new() -> Self {
        Self {
            counts: CLAUSES.iter().map(|clause| (*clause, 0usize)).collect(),
            offenders: Vec::new(),
        }
    }

    fn hit(&mut self, clause: &'static str, document: usize, detail: String) {
        if let Some(entry) = self.counts.iter_mut().find(|entry| entry.0 == clause) {
            entry.1 += 1;
        }
        if self.offenders.len() < MAX_REPORTED_OFFENDERS {
            self.offenders.push(Offender {
                document,
                clause,
                detail,
            });
        }
    }

    fn offenders(&self) -> usize {
        self.counts.iter().map(|entry| entry.1).sum()
    }
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = run(&argv) {
        eprintln!("demand-store: {error}");
        std::process::exit(1);
    }
}

fn run(argv: &[String]) -> Result<(), String> {
    let args = parse_args(argv)?;
    let bytes = fs::read(&args.tokenizer)
        .map_err(|e| format!("read tokenizer {}: {e}", args.tokenizer.display()))?;
    let tokenizer_sha256 = uor_r4_training::sha256_bytes(&bytes);
    let tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(&bytes).ok_or_else(|| {
        format!(
            "{} is not a byte-level BPE tokenizer",
            args.tokenizer.display()
        )
    })?;
    if tokenizer.vocab_size() != VOCAB_SIZE as usize {
        return Err(format!(
            "tokenizer vocabulary {} is not {VOCAB_SIZE}",
            tokenizer.vocab_size()
        ));
    }
    // The marker IDs are empirical, not assumed: encode the literal role strings and require the
    // exact sequences the prepared chat corpus carries.
    let user = tokenizer.encode("User:");
    let assistant = tokenizer.encode("Assistant:");
    if user != USER_MARKER.to_vec() || assistant != ASSISTANT_MARKER.to_vec() {
        return Err(format!(
            "tokenizer role markers differ from the bound contract: \"User:\" -> {user:?} \
             (want {USER_MARKER:?}), \"Assistant:\" -> {assistant:?} (want {ASSISTANT_MARKER:?})"
        ));
    }

    let mut documents = generate(&tokenizer);
    shuffle(&mut documents, args.seed);
    if let Some(limit) = args.documents {
        documents.truncate(limit);
    }
    if documents.is_empty() {
        return Err("no documents generated".into());
    }

    let mut stream: Vec<u16> = Vec::new();
    let mut mask: Vec<u8> = Vec::new();
    for document in &documents {
        stream.extend_from_slice(&document.tokens);
        mask.extend_from_slice(&document.mask);
    }

    let ledger = validate(&stream, &mask);
    let response_tokens: usize = documents
        .iter()
        .map(|document| document.mask.iter().filter(|&&value| value == 1).count())
        .sum();
    println!(
        "validation: {} documents, {} tokens, {} response tokens, {} clauses, {} offenders",
        documents.len(),
        stream.len(),
        response_tokens,
        ledger.counts.len(),
        ledger.offenders()
    );
    for (clause, count) in &ledger.counts {
        println!("  clause {clause}: {count}");
    }
    if ledger.offenders() != 0 {
        for offender in &ledger.offenders {
            eprintln!(
                "  document {} clause {}: {}",
                offender.document, offender.clause, offender.detail
            );
        }
        return Err(format!(
            "refusing to write: {} clause offenders ({} shown); the store would be rejected with \
             \"dialogue response must retain exact unmasked marker and genuine terminal EOS\" or a \
             neighbouring corpus contract",
            ledger.offenders(),
            ledger.offenders.len()
        ));
    }

    write_store(&args.output, &stream, &mask)?;
    let population = verify_store(&args.output, &stream, &mask)?;

    let mut per_form: Vec<(&'static str, usize, BTreeSet<(String, String)>)> = FORMS
        .iter()
        .map(|form| (form.label, 0usize, BTreeSet::new()))
        .collect();
    for document in &documents {
        let entry = &mut per_form[document.form];
        entry.1 += 1;
        entry
            .2
            .insert((document.question.clone(), document.answer.clone()));
    }

    println!(
        "wrote {} documents to {} ({} tokens, {} mask bytes)",
        documents.len(),
        args.output.display(),
        stream.len(),
        mask.len()
    );
    println!(
        "  tokenizer sha256 {tokenizer_sha256}; seed 0x{:016x}; forms:",
        args.seed
    );
    for (label, count, pairs) in &per_form {
        println!(
            "    {label}: {count} documents, {} distinct (question, answer) pairs",
            pairs.len()
        );
    }
    let tokens_path = args.output.join("tokens.u16");
    let mask_path = args.output.join("response_mask.u8");
    println!(
        "  tokens.u16 sha256 {} ({} bytes incl. {CORPUS_HEADER_SIZE}-byte UORT header)",
        uor_r4_training::sha256_file(&tokens_path).map_err(|e| e.to_string())?,
        fs::metadata(&tokens_path).map_err(|e| e.to_string())?.len()
    );
    println!(
        "  response_mask.u8 sha256 {} ({} bytes)",
        uor_r4_training::sha256_file(&mask_path).map_err(|e| e.to_string())?,
        fs::metadata(&mask_path).map_err(|e| e.to_string())?.len()
    );
    println!(
        "  episode index (training path): documents {}, response runs {}, response tokens {}, \
         eligible responses {}, eligible response tokens {}",
        population.documents,
        population.response_runs,
        population.response_tokens,
        population.eligible_responses,
        population.eligible_response_tokens
    );
    println!(
        "  fractions: response/token {:.6}, tokens/document {:.3}",
        response_tokens as f64 / stream.len() as f64,
        stream.len() as f64 / documents.len() as f64
    );

    for (index, document) in documents.iter().take(3).enumerate() {
        let ids: Vec<u32> = document
            .tokens
            .iter()
            .map(|&token| u32::from(token))
            .collect();
        println!(
            "document {index} ({}): {}",
            FORMS[document.form].label,
            tokenizer.decode(&ids)
        );
        println!("  tokens: {ids:?}");
        println!("  mask:   {:?}", document.mask);
    }
    Ok(())
}

fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut output: Option<PathBuf> = None;
    let mut tokenizer: Option<PathBuf> = None;
    let mut documents: Option<usize> = None;
    let mut seed = DEFAULT_SEED;
    let mut index = 0;
    while index < argv.len() {
        match argv[index].as_str() {
            "--tokenizer" => {
                tokenizer = Some(PathBuf::from(value(argv, &mut index, "--tokenizer")?));
            }
            "--documents" => {
                let raw = value(argv, &mut index, "--documents")?;
                documents = Some(
                    raw.parse::<usize>()
                        .map_err(|_| format!("--documents expects a count, got {raw}"))?,
                );
            }
            "--seed" => {
                let raw = value(argv, &mut index, "--seed")?;
                let parsed = raw
                    .strip_prefix("0x")
                    .map(|hex| u64::from_str_radix(hex, 16))
                    .unwrap_or_else(|| raw.parse::<u64>())
                    .map_err(|_| format!("--seed expects a u64, got {raw}"))?;
                seed = parsed;
            }
            "--help" | "-h" => {
                println!(
                    "usage: demand-store <output-dir> [--tokenizer <tokenizer.json>] \
                     [--documents N] [--seed N]"
                );
                std::process::exit(0);
            }
            other if other.starts_with("--") => return Err(format!("unknown flag {other}")),
            other => {
                if output.is_some() {
                    return Err(format!("unexpected extra argument {other}"));
                }
                output = Some(PathBuf::from(other));
            }
        }
        index += 1;
    }
    Ok(Args {
        output: output.ok_or_else(|| {
            "usage: demand-store <output-dir> [--tokenizer <tokenizer.json>] [--documents N] \
             [--seed N]"
                .to_string()
        })?,
        tokenizer: tokenizer.unwrap_or_else(default_tokenizer),
        documents,
        seed,
    })
}

fn value(argv: &[String], index: &mut usize, flag: &str) -> Result<String, String> {
    *index += 1;
    argv.get(*index)
        .cloned()
        .ok_or_else(|| format!("{flag} needs a value"))
}

fn default_tokenizer() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join("uor-r4-local/entry-scorer-inputs/tokenizer.json")
}

/// The cross product of form, item, question phrasing and demand phrasing, in a fixed order; the
/// caller shuffles it with the fixed seed.
fn generate(tokenizer: &HfBpeTokenizer) -> Vec<Document> {
    let mut documents = Vec::new();
    for (form_index, form) in FORMS.iter().enumerate() {
        for item in form.items {
            for question in item.questions {
                for demand in form.demands {
                    let question = demand.replace("{Q}", question);
                    documents.push(encode_document(
                        tokenizer,
                        form_index,
                        &question,
                        item.answer,
                    ));
                }
            }
        }
    }
    documents
}

/// `BOS User: <question> \n Assistant: <answer> EOS`, with only the answer and its EOS masked.
fn encode_document(
    tokenizer: &HfBpeTokenizer,
    form: usize,
    question: &str,
    answer: &str,
) -> Document {
    let mut tokens: Vec<u16> = Vec::new();
    let mut mask: Vec<u8> = Vec::new();
    push(&mut tokens, &mut mask, &[BOS_ID], 0);
    push(&mut tokens, &mut mask, &USER_MARKER, 0);
    push(
        &mut tokens,
        &mut mask,
        &tokenizer.encode(&format!(" {question}")),
        0,
    );
    push(&mut tokens, &mut mask, &tokenizer.encode(SEPARATOR), 0);
    push(&mut tokens, &mut mask, &ASSISTANT_MARKER, 0);
    push(
        &mut tokens,
        &mut mask,
        &tokenizer.encode(&format!(" {answer}")),
        1,
    );
    push(&mut tokens, &mut mask, &[EOS_ID], 1);
    Document {
        form,
        question: question.to_owned(),
        answer: answer.to_owned(),
        tokens,
        mask,
    }
}

/// Ids out of `u16` range are saturated so the vocabulary clause sees them rather than a
/// truncation that would hide the fault.
fn push(tokens: &mut Vec<u16>, mask: &mut Vec<u8>, ids: &[u32], selected: u8) {
    for &id in ids {
        tokens.push(u16::try_from(id).unwrap_or(u16::MAX));
        mask.push(selected);
    }
}

/// Deterministic Fisher-Yates over the generated order, so the store is fixed by `seed`.
fn shuffle(documents: &mut [Document], seed: u64) {
    let mut state = seed;
    for index in (1..documents.len()).rev() {
        let pick = (splitmix64(&mut state) % (index as u64 + 1)) as usize;
        documents.swap(index, pick);
    }
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

/// The training-time contract of `dialogue_episodes.rs::record_response`, clause by clause, over
/// the flat stream: document framing, per-document mask runs, the exact unmasked assistant
/// marker, a genuine terminal EOS and no EOS inside the response.
fn validate(stream: &[u16], mask: &[u8]) -> Ledger {
    let mut ledger = Ledger::new();
    if stream.len() != mask.len() {
        ledger.hit(
            "store_mask_length",
            0,
            format!("{} tokens against {} mask bytes", stream.len(), mask.len()),
        );
        return ledger;
    }
    if stream.is_empty() {
        ledger.hit("document_bos_unmasked", 0, "empty stream".to_string());
        return ledger;
    }

    let mut starts: Vec<usize> = Vec::new();
    for (position, &token) in stream.iter().enumerate() {
        if u32::from(token) == BOS_ID {
            if position != 0 && u32::from(stream[position - 1]) != EOS_ID {
                ledger.hit(
                    "document_bos_follows_eos",
                    starts.len(),
                    format!("BOS at {position} follows token {}", stream[position - 1]),
                );
            }
            starts.push(position);
        }
    }
    if u32::from(stream[0]) != BOS_ID {
        ledger.hit(
            "document_bos_unmasked",
            0,
            format!("first token is {} not BOS", stream[0]),
        );
    }

    for (document, &start) in starts.iter().enumerate() {
        let end = starts.get(document + 1).copied().unwrap_or(stream.len());
        let tokens = &stream[start..end];
        let mask = &mask[start..end];
        let last = tokens.len() - 1;

        if u32::from(tokens[0]) != BOS_ID || mask[0] != 0 {
            ledger.hit(
                "document_bos_unmasked",
                document,
                format!(
                    "document start {start} is {} with mask {}",
                    tokens[0], mask[0]
                ),
            );
        }
        if tokens.len() < 4
            || tokens[1..4] != USER_MARKER.map(|id| id as u16)
            || mask[1..4].iter().any(|&value| value != 0)
        {
            ledger.hit(
                "user_marker_exact",
                document,
                format!(
                    "tokens 1..4 are {:?} with mask {:?}",
                    &tokens[..tokens.len().min(4)],
                    &mask[..mask.len().min(4)]
                ),
            );
        }
        if u32::from(tokens[last]) != EOS_ID || mask[last] != 1 {
            ledger.hit(
                "document_terminal_masked_eos",
                document,
                format!("final token {} carries mask {}", tokens[last], mask[last]),
            );
        }
        let interior = if tokens.len() > 2 {
            &tokens[1..last]
        } else {
            &tokens[0..0]
        };
        if interior.iter().any(|&token| u32::from(token) == EOS_ID) {
            ledger.hit(
                "document_no_interior_eos",
                document,
                "an EOS appears before the document end".to_string(),
            );
        }
        if tokens[1..].iter().any(|&token| u32::from(token) == BOS_ID) {
            ledger.hit(
                "document_no_interior_bos",
                document,
                "a BOS appears inside the document".to_string(),
            );
        }
        for (offset, &token) in tokens.iter().enumerate() {
            if u32::from(token) >= VOCAB_SIZE || u32::from(token) == UNK_ID {
                ledger.hit(
                    "document_vocabulary",
                    document,
                    format!("token {} at document offset {offset}", token),
                );
            }
        }
        for (offset, &value) in mask.iter().enumerate() {
            if value > 1 {
                ledger.hit(
                    "document_mask_binary",
                    document,
                    format!("mask byte {value} at document offset {offset}"),
                );
            }
        }

        let mut runs: Vec<(usize, usize)> = Vec::new();
        let mut offset = 0;
        while offset < mask.len() {
            if mask[offset] == 1 {
                let run_start = offset;
                while offset < mask.len() && mask[offset] == 1 {
                    offset += 1;
                }
                runs.push((run_start, offset));
            } else {
                offset += 1;
            }
        }
        if runs.len() != 1 {
            ledger.hit(
                "response_single_run",
                document,
                format!("{} maximal runs of 1s", runs.len()),
            );
            continue;
        }
        let (response_start, response_end) = runs[0];
        if response_start >= response_end {
            ledger.hit(
                "response_nonempty",
                document,
                "empty response run".to_string(),
            );
            continue;
        }
        // `marker_start <= document` (the BOS position) is the training check, so the run must
        // start after offset 5: five marker tokens strictly inside the document.
        if response_start <= ASSISTANT_MARKER.len() {
            ledger.hit(
                "response_marker_present",
                document,
                format!("response run starts at document offset {response_start}"),
            );
            continue;
        }
        let marker_start = response_start - ASSISTANT_MARKER.len();
        if mask[marker_start..response_start]
            .iter()
            .any(|&value| value != 0)
        {
            ledger.hit(
                "response_marker_unmasked",
                document,
                format!(
                    "mask {:?} over the marker",
                    &mask[marker_start..response_start]
                ),
            );
        }
        if tokens[marker_start..response_start] != ASSISTANT_MARKER.map(|id| id as u16) {
            ledger.hit(
                "response_marker_exact",
                document,
                format!(
                    "tokens {:?} before the response run",
                    &tokens[marker_start..response_start]
                ),
            );
        }
        if u32::from(tokens[response_end - 1]) != EOS_ID {
            ledger.hit(
                "response_terminal_eos",
                document,
                format!(
                    "last response token is {} not EOS",
                    tokens[response_end - 1]
                ),
            );
        }
        if tokens[response_start..response_end - 1]
            .iter()
            .any(|&token| u32::from(token) == EOS_ID)
        {
            ledger.hit(
                "response_no_interior_eos",
                document,
                "an EOS appears inside the response run".to_string(),
            );
        }
    }
    ledger
}

fn write_store(directory: &Path, stream: &[u16], mask: &[u8]) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|e| format!("create {}: {e}", directory.display()))?;
    let tokens_path = directory.join("tokens.u16");
    CorpusWriter::write_file(&tokens_path, VOCAB_SIZE, stream)
        .map_err(|e| format!("write {}: {e}", tokens_path.display()))?;
    let mask_path = directory.join("response_mask.u8");
    fs::write(&mask_path, mask).map_err(|e| format!("write {}: {e}", mask_path.display()))?;
    let manifest = serde_json::json!({
        "drops": {"special_token_occurrences": 0},
        "files": [{
            "label": LABEL,
            "tokens": stream.len(),
            "special_token_occurrences": 0,
        }],
        "mask_bytes": mask.len(),
    });
    let manifest_path = directory.join("manifest.json");
    let bytes =
        serde_json::to_vec_pretty(&manifest).map_err(|e| format!("serialize manifest: {e}"))?;
    fs::write(&manifest_path, bytes)
        .map_err(|e| format!("write {}: {e}", manifest_path.display()))?;
    Ok(())
}

/// Re-read the emitted bytes and rebuild the real episode index over them, so the store is
/// accepted by the training path's own contract rather than by the copy above.
fn verify_store(
    directory: &Path,
    stream: &[u16],
    mask: &[u8],
) -> Result<uor_r4_training::dialogue_episodes::EpisodePopulation, String> {
    let tokens_path = directory.join("tokens.u16");
    let mask_path = directory.join("response_mask.u8");
    let manifest_path = directory.join("manifest.json");
    let reader = MmapCorpusReader::open(&tokens_path)
        .map_err(|e| format!("reopen {}: {e}", tokens_path.display()))?;
    if reader.total_tokens() != stream.len() || reader.vocab_size() != VOCAB_SIZE {
        return Err(format!(
            "{} declares {} tokens at vocabulary {}; expected {} at {VOCAB_SIZE}",
            tokens_path.display(),
            reader.total_tokens(),
            reader.vocab_size(),
            stream.len()
        ));
    }
    if reader.as_slice() != stream {
        return Err(format!(
            "{} payload differs from the generated stream",
            tokens_path.display()
        ));
    }
    let stored = fs::read(&mask_path).map_err(|e| format!("read {}: {e}", mask_path.display()))?;
    if stored != mask {
        return Err(format!(
            "{} differs from the generated mask",
            mask_path.display()
        ));
    }
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(&manifest_path).map_err(|e| format!("read {}: {e}", manifest_path.display()))?,
    )
    .map_err(|e| format!("parse {}: {e}", manifest_path.display()))?;
    let files = manifest["files"]
        .as_array()
        .ok_or_else(|| "manifest has no files array".to_string())?;
    if manifest["drops"]["special_token_occurrences"] != 0
        || manifest["mask_bytes"].as_u64() != Some(mask.len() as u64)
        || files.len() != 1
        || files[0]["label"] != LABEL
        || files[0]["tokens"].as_u64() != Some(stream.len() as u64)
        || files[0]["special_token_occurrences"] != 0
    {
        return Err("manifest fields do not match the written store".to_string());
    }
    let contract = EpisodeContract {
        context: EPISODE_CONTEXT,
        vocab_size: VOCAB_SIZE as usize,
        bos_id: BOS_ID,
        eos_id: EOS_ID,
        unk_id: UNK_ID,
        padding_id: PADDING_ID,
        assistant_marker_ids: ASSISTANT_MARKER.to_vec(),
    };
    // Load through the stack's own manifest reader, then index with the training contract, so
    // acceptance is the training path's check rather than a copy of it.
    let split = DialogueSplit::load(&tokens_path, &mask_path, &manifest_path)
        .map_err(|e| format!("stack dialogue manifest reader rejected the store: {e}"))?;
    if split.vocab_size() != VOCAB_SIZE as usize {
        return Err(format!(
            "loaded vocabulary {} is not {VOCAB_SIZE}",
            split.vocab_size()
        ));
    }
    if split.source_range(LABEL) != Some(0..stream.len()) {
        return Err(format!(
            "no single source labelled {LABEL} covers the store"
        ));
    }
    let index = split
        .index(contract)
        .map_err(|e| format!("the training-time episode index rejected the emitted store: {e}"))?;
    Ok(index.population().clone())
}
