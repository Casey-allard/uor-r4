//! Build a synthetic in-context memory corpus: many fact / competing-statement
//! / question documents whose assistant turn is the correct answer.
//!
//! ```text
//! geometric-stack synthetic-memory out=NEW_REPORT_ROOT tokenizer=TOKENIZER.json \
//!   rows=2000 seed=7 [split=train] [vocab_size=4096]
//! ```
//!
//! Why this exists: the value-faithfulness diagnostic (#1512) found that a
//! competing turn costs the emitter about 2/10 of answer-target stability
//! (fact+question 7/10, fact+competing+question 4-5/10), and that the deficit is
//! *not* retrieval, selection or question binding. Testing whether it is
//! trainable needs a **distribution** of such bindings, and the development
//! panel is ten rows -- roughly 500 tokens, about 0.0006% of the 82.5M-token
//! chat-v0 response store, far too little to move a fitted model.
//!
//! The documents are written in the corpus format the trainer already loads
//! (schema `uor-r4-chat-corpus/v1`): `<|bos|>` then turns joined by a single
//! newline, each turn `<marker><content>` with markers `User: ` / `Assistant: `,
//! every assistant turn ending with `<|eos|>`; `tokens.u16` holds the ids and
//! `response_mask.u8` is 1 on each assistant response token and its terminating
//! EOS, 0 elsewhere. Encoding goes through the same `DialogueProtocol` and
//! `DialogueEncoder` the rest of the project uses, so role markers, separators
//! and masking are not re-implemented here.
//!
//! Both the asked relation and the competing statement are always *stated*, and
//! the answer is always the asked relation's value, so a model that learns the
//! corpus is learning to answer the asked relation rather than to prefer the
//! most recent value. Which relation is asked varies, and the competing
//! statement's relation is drawn independently.

use std::fs;
use std::path::PathBuf;

use serde_json::json;
use uor_r4_core::native_geometric::mmap_corpus::CorpusWriter;
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, String>;

/// A relation, the questions that ask it, and the answers it can state.
///
/// `asks` are question forms and `states` are statement forms; `{v}` is the
/// value. Keeping several of each stops the generator from teaching a single
/// surface pattern.
struct Relation {
    id: &'static str,
    asks: &'static [&'static str],
    states: &'static [&'static str],
    /// How the answer is phrased. A template rather than the relation id
    /// interpolated into one sentence, because the ids are not all noun phrases
    /// and `"Your brothers is seven."` is not English.
    answers: &'static [&'static str],
}

const RELATIONS: &[Relation] = &[
    Relation {
        id: "breed",
        asks: &["What breed is my dog?"],
        states: &["My dog is a {v}."],
        answers: &["Your dog is a {v}."],
    },
    Relation {
        id: "blood_type",
        asks: &["What is my blood type?"],
        states: &["My blood type is {v}."],
        answers: &["Your blood type is {v}."],
    },
    Relation {
        id: "handedness",
        asks: &["Am I left or right handed?"],
        states: &["I write with my {v} hand."],
        answers: &["You are {v}."],
    },
    Relation {
        id: "timezone",
        asks: &["Which timezone am I in?"],
        states: &["I am in the {v} timezone."],
        answers: &["Your timezone is {v}."],
    },
    Relation {
        id: "bedtime",
        asks: &["What time do I go to bed?"],
        states: &["I go to bed at {v}."],
        answers: &["You go to bed at {v}."],
    },
    Relation {
        id: "wake_time",
        asks: &["What time do I wake up?"],
        states: &["I wake up at {v}."],
        answers: &["You wake up at {v}."],
    },
    Relation {
        id: "coffee",
        asks: &["What coffee do I order?"],
        states: &["I always order {v}."],
        answers: &["You order {v}."],
    },
    Relation {
        id: "pizza",
        asks: &["What topping do I like on pizza?"],
        states: &["I like {v} on my pizza."],
        answers: &["You like {v} on pizza."],
    },
    Relation {
        id: "ice_cream",
        asks: &["What is my favourite ice cream?"],
        states: &["My favourite ice cream is {v}."],
        answers: &["Your favourite ice cream is {v}."],
    },
    Relation {
        id: "dessert",
        asks: &["What dessert do I like?"],
        states: &["I love {v} for dessert."],
        answers: &["You love {v} for dessert."],
    },
    Relation {
        id: "author",
        asks: &["Who is my favourite author?"],
        states: &["My favourite author is {v}."],
        answers: &["Your favourite author is {v}."],
    },
    Relation {
        id: "film",
        asks: &["What is my favourite film?"],
        states: &["My favourite film is {v}."],
        answers: &["Your favourite film is {v}."],
    },
    Relation {
        id: "song",
        asks: &["What is my favourite song?"],
        states: &["My favourite song is {v}."],
        answers: &["Your favourite song is {v}."],
    },
    Relation {
        id: "book",
        asks: &["What book am I reading?"],
        states: &["I am reading {v}."],
        answers: &["You are reading {v}."],
    },
    Relation {
        id: "city",
        asks: &["Which city do I want to visit?"],
        states: &["I want to visit {v}."],
        answers: &["You want to visit {v}."],
    },
    Relation {
        id: "mother",
        asks: &["What is my mother's name?"],
        states: &["My mother is called {v}."],
        answers: &["Your mother is called {v}."],
    },
    Relation {
        id: "brother",
        asks: &["What is my brother's name?"],
        states: &["My brother is called {v}."],
        answers: &["Your brother is called {v}."],
    },
    Relation {
        id: "dog",
        asks: &["What is my dog's name?"],
        states: &["My dog is called {v}."],
        answers: &["Your dog is called {v}."],
    },
    Relation {
        id: "neighbour",
        asks: &["What is my neighbour's name?"],
        states: &["My neighbour is called {v}."],
        answers: &["Your neighbour is called {v}."],
    },
    Relation {
        id: "garden",
        asks: &["Do I have a garden?"],
        states: &["I {v} a garden at home."],
        answers: &["You {v} a garden."],
    },
    Relation {
        id: "siblings",
        asks: &["How many siblings do I have?"],
        states: &["I have {v} siblings."],
        answers: &["You have {v} siblings."],
    },
    Relation {
        id: "uni_subject",
        asks: &["What did I read at university?"],
        states: &["I read {v} at university."],
        answers: &["You read {v} at university."],
    },
    Relation {
        id: "school",
        asks: &["Where did I go to school?"],
        states: &["I went to school in {v}."],
        answers: &["You went to school in {v}."],
    },
    Relation {
        id: "office",
        asks: &["Where is my office?"],
        states: &["My office is in {v}."],
        answers: &["Your office is in {v}."],
    },
    Relation {
        id: "employment",
        asks: &["What is my employment status?"],
        states: &["I am {v}."],
        answers: &["You are {v}."],
    },
    Relation {
        id: "train_line",
        asks: &["Which train line do I take?"],
        states: &["I take the {v} line."],
        answers: &["You take the {v} line."],
    },
    Relation {
        id: "airline",
        asks: &["Which airline do I fly with?"],
        states: &["I usually fly with {v}."],
        answers: &["You usually fly with {v}."],
    },
    Relation {
        id: "suitcase",
        asks: &["What colour is my suitcase?"],
        states: &["My suitcase is {v}."],
        answers: &["Your suitcase is {v}."],
    },
    Relation {
        id: "watch",
        asks: &["What kind of watch do I wear?"],
        states: &["I wear a {v} watch."],
        answers: &["You wear a {v} watch."],
    },
    Relation {
        id: "phone",
        asks: &["What phone do I use?"],
        states: &["I use a {v}."],
        answers: &["You use a {v}."],
    },
    Relation {
        id: "laptop",
        asks: &["What laptop do I use?"],
        states: &["I use a {v} laptop."],
        answers: &["You use a {v} laptop."],
    },
    Relation {
        id: "editor",
        asks: &["Which editor do I use?"],
        states: &["I write code in {v}."],
        answers: &["You write code in {v}."],
    },
    Relation {
        id: "shell",
        asks: &["Which shell do I use?"],
        states: &["I use {v}."],
        answers: &["You use {v}."],
    },
    Relation {
        id: "keyboard",
        asks: &["What keyboard do I type on?"],
        states: &["I type on a {v}."],
        answers: &["You type on a {v}."],
    },
    Relation {
        id: "mug",
        asks: &["Which mug is mine?"],
        states: &["My mug is the {v} one."],
        answers: &["Your mug is the {v} one."],
    },
    Relation {
        id: "park",
        asks: &["Which park do I walk in?"],
        states: &["I walk in {v} park."],
        answers: &["You walk in {v} park."],
    },
    Relation {
        id: "pub",
        asks: &["Which pub do I go to?"],
        states: &["I drink at the {v}."],
        answers: &["You drink at the {v}."],
    },
    Relation {
        id: "restaurant",
        asks: &["Which restaurant do I like?"],
        states: &["I like the {v} restaurant."],
        answers: &["You like the {v} restaurant."],
    },
    Relation {
        id: "market",
        asks: &["Which market do I shop at?"],
        states: &["I shop at {v} market."],
        answers: &["You shop at {v} market."],
    },
    Relation {
        id: "museum",
        asks: &["Which museum do I like?"],
        states: &["I like the {v} museum."],
        answers: &["You like the {v} museum."],
    },
    Relation {
        id: "gallery",
        asks: &["Which gallery do I visit?"],
        states: &["I visit the {v} gallery."],
        answers: &["You visit the {v} gallery."],
    },
    Relation {
        id: "theatre",
        asks: &["Which theatre do I go to?"],
        states: &["I go to the {v} theatre."],
        answers: &["You go to the {v} theatre."],
    },
    Relation {
        id: "cinema",
        asks: &["Which cinema do I use?"],
        states: &["I use the {v} cinema."],
        answers: &["You use the {v} cinema."],
    },
    Relation {
        id: "gym_class",
        asks: &["Which gym class do I take?"],
        states: &["I take {v}."],
        answers: &["You take {v}."],
    },
    Relation {
        id: "run_route",
        asks: &["Where do I run?"],
        states: &["I run along the {v}."],
        answers: &["You run along the {v}."],
    },
    Relation {
        id: "bike",
        asks: &["What kind of bike do I ride?"],
        states: &["I ride a {v} bike."],
        answers: &["You ride a {v} bike."],
    },
    Relation {
        id: "swim_stroke",
        asks: &["Which stroke do I swim?"],
        states: &["I swim {v}."],
        answers: &["You swim {v}."],
    },
    Relation {
        id: "sport",
        asks: &["Which sport do I play?"],
        states: &["I play {v}."],
        answers: &["You play {v}."],
    },
    Relation {
        id: "team",
        asks: &["Which team do I support?"],
        states: &["I support {v}."],
        answers: &["You support {v}."],
    },
    Relation {
        id: "instrument",
        asks: &["What instrument do I play?"],
        states: &["I play the {v}."],
        answers: &["You play the {v}."],
    },
    Relation {
        id: "language",
        asks: &["Which language am I learning?"],
        states: &["I am learning {v}."],
        answers: &["You are learning {v}."],
    },
    Relation {
        id: "intolerance",
        asks: &["What food intolerance do I have?"],
        states: &["I am intolerant to {v}."],
        answers: &["You are intolerant to {v}."],
    },
    Relation {
        id: "diet",
        asks: &["What diet do I follow?"],
        states: &["I follow a {v} diet."],
        answers: &["You follow a {v} diet."],
    },
    Relation {
        id: "medicine",
        asks: &["What medicine do I take?"],
        states: &["I take {v}."],
        answers: &["You take {v}."],
    },
    Relation {
        id: "optician",
        asks: &["Who is my optician?"],
        states: &["My optician is {v}."],
        answers: &["Your optician is {v}."],
    },
    Relation {
        id: "dentist",
        asks: &["Who is my dentist?"],
        states: &["My dentist is {v}."],
        answers: &["Your dentist is {v}."],
    },
    Relation {
        id: "doctor",
        asks: &["Who is my doctor?"],
        states: &["My doctor is {v}."],
        answers: &["Your doctor is {v}."],
    },
    Relation {
        id: "car",
        asks: &["What car do I drive?"],
        states: &["I drive a {v}."],
        answers: &["You drive a {v}."],
    },
    Relation {
        id: "bike_lock",
        asks: &["What lock do I use?"],
        states: &["I use a {v} lock."],
        answers: &["You use a {v} lock."],
    },
    Relation {
        id: "postcode",
        asks: &["What is my postcode?"],
        states: &["My postcode is {v}."],
        answers: &["Your postcode is {v}."],
    },
    Relation {
        id: "house_number",
        asks: &["What is my house number?"],
        states: &["I live at number {v}."],
        answers: &["You live at number {v}."],
    },
    Relation {
        id: "street",
        asks: &["What street do I live on?"],
        states: &["I live on {v} Street."],
        answers: &["You live on {v} Street."],
    },
    Relation {
        id: "rent",
        asks: &["How much rent do I pay?"],
        states: &["I pay {v} a month in rent."],
        answers: &["You pay {v} a month."],
    },
    Relation {
        id: "mortgage",
        asks: &["Do I have a mortgage?"],
        states: &["I {v} a mortgage."],
        answers: &["You {v} a mortgage."],
    },
    Relation {
        id: "savings",
        asks: &["What am I saving for?"],
        states: &["I am saving for {v}."],
        answers: &["You are saving for {v}."],
    },
    Relation {
        id: "currency",
        asks: &["Which currency do I use?"],
        states: &["I use {v}."],
        answers: &["You use {v}."],
    },
    Relation {
        id: "charity",
        asks: &["Which charity do I support?"],
        states: &["I support {v}."],
        answers: &["You support {v}."],
    },
    Relation {
        id: "paper",
        asks: &["Which paper do I read?"],
        states: &["I read the {v}."],
        answers: &["You read the {v}."],
    },
    Relation {
        id: "podcast",
        asks: &["Which podcast do I listen to?"],
        states: &["I listen to {v}."],
        answers: &["You listen to {v}."],
    },
    Relation {
        id: "radio",
        asks: &["Which radio station do I like?"],
        states: &["I like {v}."],
        answers: &["You like {v}."],
    },
    Relation {
        id: "hobby",
        asks: &["What is my hobby?"],
        states: &["My hobby is {v}."],
        answers: &["Your hobby is {v}."],
    },
    Relation {
        id: "collection",
        asks: &["What do I collect?"],
        states: &["I collect {v}."],
        answers: &["You collect {v}."],
    },
    Relation {
        id: "instrument_lesson",
        asks: &["Who teaches me?"],
        states: &["My teacher is {v}."],
        answers: &["Your teacher is {v}."],
    },
    Relation {
        id: "volunteer",
        asks: &["Where do I volunteer?"],
        states: &["I volunteer at the {v}."],
        answers: &["You volunteer at the {v}."],
    },
    Relation {
        id: "allotment",
        asks: &["What do I grow?"],
        states: &["I grow {v}."],
        answers: &["You grow {v}."],
    },
    Relation {
        id: "pet_fish",
        asks: &["What fish do I keep?"],
        states: &["I keep {v}."],
        answers: &["You keep {v}."],
    },
];

/// Values per relation. Deliberately includes values that are also a *distinct*
/// relation's value (for example `blue` for both colour and car), so the model
/// cannot answer by surfacing a value that only ever belongs to one relation.
const VALUES: &[(&str, &[&str])] = &[
    ("breed", &["spaniel", "terrier", "poodle", "beagle", "corgi"]),
    ("blood_type", &["A positive", "O negative", "B positive", "AB negative", "O positive"]),
    ("handedness", &["left", "right"]),
    ("timezone", &["GMT", "CET", "EST", "PST", "JST"]),
    ("bedtime", &["ten", "eleven", "half past nine", "midnight", "nine"]),
    ("wake_time", &["six", "seven", "half past five", "eight", "five"]),
    ("coffee", &["a flat white", "an espresso", "a latte", "a cortado", "a filter coffee"]),
    ("pizza", &["olives", "pepperoni", "mushrooms", "anchovies", "pineapple"]),
    ("ice_cream", &["vanilla", "chocolate", "strawberry", "mint", "caramel"]),
    ("dessert", &["apple crumble", "cheesecake", "tiramisu", "brownies", "sorbet"]),
    ("author", &["Austen", "Murakami", "Le Guin", "Ishiguro", "Atwood"]),
    ("film", &["Alien", "Spirited Away", "Casablanca", "Parasite", "Jaws"]),
    ("song", &["Hey Jude", "Blue Monday", "Wonderwall", "Redemption Song", "Teardrop"]),
    ("book", &["Ulysses", "Dune", "Persuasion", "Solaris", "Beloved"]),
    ("city", &["Kyoto", "Marrakesh", "Reykjavik", "Cusco", "Hanoi"]),
    ("mother", &["Diane", "Rosa", "Ingrid", "Amara", "Yuki"]),
    ("brother", &["Tom", "Ravi", "Lars", "Kwame", "Nico"]),
    ("dog", &["Biscuit", "Nala", "Rufus", "Poppy", "Ziggy"]),
    ("neighbour", &["Harold", "Mei", "Pavel", "Fatima", "Colin"]),
    ("garden", &["have", "do not have"]),
    ("siblings", &["one", "two", "three", "four", "none"]),
    ("uni_subject", &["history", "chemistry", "economics", "philosophy", "biology"]),
    ("school", &["Leeds", "Galway", "Aarhus", "Nagoya", "Tulsa"]),
    ("office", &["Soho", "Camden", "Clifton", "Digbeth", "Leith"]),
    ("employment", &["full time", "part time", "self employed", "between jobs", "retired"]),
    ("train_line", &["Northern", "Central", "District", "Piccadilly", "Bakerloo"]),
    ("airline", &["KLM", "Emirates", "Qantas", "Lufthansa", "Iberia"]),
    ("suitcase", &["black", "red", "grey", "navy", "olive"]),
    ("watch", &["digital", "analog", "solar", "dive", "field"]),
    ("phone", &["Pixel", "iPhone", "Nothing", "Fairphone", "Galaxy"]),
    ("laptop", &["ThinkPad", "MacBook", "Framework", "Surface", "XPS"]),
    ("editor", &["Neovim", "Emacs", "VS Code", "Helix", "Zed"]),
    ("shell", &["zsh", "bash", "fish", "nu", "dash"]),
    ("keyboard", &["Model M", "HHKB", "Ergodox", "Kinesis", "Realforce"]),
    ("mug", &["blue", "chipped", "tall", "striped", "plain"]),
    ("park", &["Highbury", "Phoenix", "Riverside", "Queens", "Victoria"]),
    ("pub", &["Red Lion", "Anchor", "Crown", "Bell", "Ship"]),
    ("restaurant", &["Thai", "Nepalese", "Ethiopian", "Peruvian", "Georgian"]),
    ("market", &["Borough", "Broadway", "St George", "Portobello", "Camden"]),
    ("museum", &["Science", "V&A", "Natural History", "Design", "Transport"]),
    ("gallery", &["Tate", "Whitechapel", "Serpentine", "Hayward", "Barbican"]),
    ("theatre", &["Almeida", "Old Vic", "Young Vic", "Donmar", "Lyceum"]),
    ("cinema", &["Rio", "Everyman", "Prince Charles", "Genesis", "Lexi"]),
    ("gym_class", &["spin", "yoga", "pilates", "boxing", "climbing"]),
    ("run_route", &["canal", "towpath", "common", "river", "seafront"]),
    ("bike", &["road", "gravel", "folding", "cargo", "fixed"]),
    ("swim_stroke", &["front crawl", "breaststroke", "backstroke", "butterfly", "sidestroke"]),
    ("sport", &["squash", "cricket", "hockey", "badminton", "water polo"]),
    ("team", &["Fulham", "Celtic", "Everton", "Brighton", "Norwich"]),
    ("instrument", &["clarinet", "banjo", "oboe", "double bass", "accordion"]),
    ("language", &["Portuguese", "Korean", "Swahili", "Finnish", "Catalan"]),
    ("intolerance", &["lactose", "gluten", "fructose", "histamine", "caffeine"]),
    ("diet", &["vegetarian", "vegan", "kosher", "halal", "pescatarian"]),
    ("medicine", &["statins", "thyroxine", "metformin", "aspirin", "inhalers"]),
    ("optician", &["Specsavers", "Boots", "Vision Express", "Scrivens", "Leightons"]),
    ("dentist", &["Bupa", "mydentist", "Colosseum", "Portman", "Together"]),
    ("doctor", &["Dr Ellis", "Dr Rahman", "Dr Novak", "Dr Osei", "Dr Lindqvist"]),
    ("car", &["Fiesta", "Golf", "Volvo", "Prius", "Mini"]),
    ("bike_lock", &["D", "chain", "cable", "folding", "frame"]),
    ("postcode", &["N1", "SE15", "BS8", "EH6", "CF10"]),
    ("house_number", &["twelve", "forty one", "seven", "ninety", "three"]),
    ("street", &["Albion", "Meadow", "Chapel", "Grove", "Harbour"]),
    ("rent", &["nine hundred", "twelve hundred", "a thousand", "fifteen hundred", "seven hundred"]),
    ("mortgage", &["have", "do not have"]),
    ("savings", &["a house", "a car", "a holiday", "retirement", "a wedding"]),
    ("currency", &["sterling", "euros", "dollars", "yen", "francs"]),
    ("charity", &["Shelter", "Mind", "RNLI", "Oxfam", "Amnesty"]),
    ("paper", &["Guardian", "Times", "FT", "Independent", "Herald"]),
    ("podcast", &["Reply All", "99pi", "Radiolab", "Serial", "Witness"]),
    ("radio", &["Radio 4", "6 Music", "World Service", "3", "Classic FM"]),
    ("hobby", &["birdwatching", "woodwork", "pottery", "astronomy", "foraging"]),
    ("collection", &["stamps", "records", "maps", "cameras", "typewriters"]),
    ("instrument_lesson", &["Mr Hale", "Ms Ferreira", "Dr Banerjee", "Mrs Okafor", "Mr Lindgren"]),
    ("volunteer", &["library", "food bank", "hospice", "allotment", "museum"]),
    ("allotment", &["tomatoes", "courgettes", "beans", "chillies", "rhubarb"]),
    ("pet_fish", &["guppies", "tetras", "goldfish", "danios", "rasboras"]),
];

/// Relations whose value sets overlap, so a distractor can reuse the asked
/// relation's *shape* without being its answer.
const OVERLAP: &[(&str, &str)] = &[];

fn values_of(id: &str) -> Result<&'static [&'static str]> {
    VALUES
        .iter()
        .find(|(relation, _)| *relation == id)
        .map(|(_, values)| *values)
        .ok_or_else(|| format!("no values for relation {id}"))
}

/// A small deterministic generator: splitmix64, so a run is reproducible from
/// its seed alone and needs no external rng dependency.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }
}

/// One document's turns as owned strings: a stated asked relation, a stated
/// competing relation, the question, and the correct answer.
///
/// Owned strings rather than borrowed `Message`s so the caller can borrow them
/// for the duration of the encode call -- `Message` holds `&str`, and leaking
/// to satisfy it would be a real leak for no benefit.
struct Document {
    first: String,
    second: String,
    question: String,
    answer: String,
    wanted: String,
}

fn document(rng: &mut Rng) -> Result<Document> {
    let asked = rng.pick(RELATIONS);
    let asked_values = values_of(asked.id)?;
    let wanted = *rng.pick(asked_values);
    // A competing relation, never the asked one.
    let competing = loop {
        let candidate = rng.pick(RELATIONS);
        if candidate.id != asked.id {
            break candidate;
        }
    };
    // Half the time the competing value is drawn from the asked relation's
    // value set when the two overlap, which is the hardest case: the competing
    // statement is a plausible answer to the question.
    let competing_value = match OVERLAP
        .iter()
        .find(|(a, _)| *a == competing.id)
        .map(|(_, other)| *other)
    {
        Some(other) if other == asked.id && rng.below(2) == 0 => *rng.pick(asked_values),
        _ => *rng.pick(values_of(competing.id)?),
    };

    let asked_statement = rng.pick(asked.states).replace("{v}", wanted);
    let competing_statement = rng.pick(competing.states).replace("{v}", competing_value);
    let question = rng.pick(asked.asks).to_string();
    // The asked statement is sometimes stated first and sometimes second, so
    // recency cannot be the rule the corpus teaches.
    let (first, second) = if rng.below(2) == 0 {
        (asked_statement, competing_statement)
    } else {
        (competing_statement, asked_statement)
    };
    Ok(Document {
        first,
        second,
        question,
        answer: rng.pick(asked.answers).replace("{v}", wanted),
        wanted: wanted.to_string(),
    })
}

fn run(args: &[String]) -> Result<()> {
    let arg = |key: &str| -> Result<PathBuf> {
        args.iter()
            .find_map(|a| a.strip_prefix(&format!("{key}=")))
            .map(PathBuf::from)
            .ok_or_else(|| format!("missing {key}="))
    };
    let number = |key: &str, default: u64| -> Result<u64> {
        match args.iter().find_map(|a| a.strip_prefix(&format!("{key}="))) {
            None => Ok(default),
            Some(v) => v.parse().map_err(|_| format!("{key}= must be a number")),
        }
    };
    let out = arg("out")?;
    let tokenizer_path = arg("tokenizer")?;
    let rows = number("rows", 2000)? as usize;
    let seed = number("seed", 7)?;
    let split = args
        .iter()
        .find_map(|a| a.strip_prefix("split="))
        .unwrap_or("train")
        .to_string();
    if rows == 0 {
        return Err("rows must be at least 1".into());
    }

    let tokenizer_json = fs::read(&tokenizer_path).map_err(|e| e.to_string())?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_json)
        .ok_or("unreadable tokenizer.json")?;
    let vocab_size = number("vocab_size", 4096)? as u32;
    let protocol = DialogueProtocol::literal_roles_v1(&tokenizer).map_err(|e| e.to_string())?;
    let encoder: DialogueEncoder<'_> = protocol.bind(&tokenizer).map_err(|e| e.to_string())?;

    // Claim the root exclusively before writing anything into it.
    report_output::claim(&out).map_err(|e| e.to_string())?;
    let (bos, eos) = (protocol.bos_id, protocol.eos_id);

    let mut rng = Rng(seed);
    let mut documents: Vec<(Vec<u16>, Vec<u8>)> = Vec::with_capacity(rows);
    let mut total = 0usize;
    let mut response_tokens = 0usize;
    for _ in 0..rows {
        let doc = document(&mut rng)?;
        let messages = [
            Message {
                role: "user",
                content: &doc.first,
            },
            Message {
                role: "user",
                content: &doc.second,
            },
            Message {
                role: "user",
                content: &doc.question,
            },
            Message {
                role: "assistant",
                content: &doc.answer,
            },
        ];
        let encoded = encoder.encode_document(&messages);
        if encoded.emitted_turns != messages.len() {
            return Err(format!(
                "{} turns were asked for but {} were emitted",
                messages.len(),
                encoded.emitted_turns
            ));
        }
        let ids: Vec<u16> = encoded
            .tokens
            .iter()
            .map(|&t| u16::try_from(t).map_err(|_| "token id above u16".to_string()))
            .collect::<Result<_>>()?;
        // The mask must mark the answer, or the fit scores nothing.
        if encoded.response_mask.iter().filter(|&&m| m == 1).count() == 0 {
            return Err("a document has no masked response tokens".into());
        }
        if encoded.tokens.len() != encoded.response_mask.len() {
            return Err(format!(
                "the encoder returned {} tokens and {} mask bytes",
                encoded.tokens.len(),
                encoded.response_mask.len()
            ));
        }
        // Decode exactly the masked region -- the scored answer and its EOS --
        // and require the wanted value to appear in it. This is the check that
        // matters: if the mask does not cover the answer, the fit scores the
        // wrong tokens and the corpus is silently useless.
        let scored: Vec<u32> = ids
            .iter()
            .zip(&encoded.response_mask)
            .filter(|(_, &m)| m == 1)
            .map(|(&id, _)| id as u32)
            .collect();
        let scored_text = tokenizer.decode(&scored);
        if !scored_text
            .to_lowercase()
            .contains(&doc.wanted.to_lowercase())
        {
            return Err(format!(
                "the scored region {scored_text:?} does not contain the answer {:?}",
                doc.wanted
            ));
        }
        total += ids.len();
        response_tokens += encoded.response_mask.iter().filter(|&&m| m == 1).count();
        documents.push((ids, encoded.response_mask));
    }

    let tokens_path = out.join("tokens.u16");
    let mut writer = CorpusWriter::create(&tokens_path, vocab_size).map_err(|e| e.to_string())?;
    let mut mask: Vec<u8> = Vec::with_capacity(total);
    for (ids, row_mask) in &documents {
        writer.write_tokens(ids).map_err(|e| e.to_string())?;
        mask.extend(row_mask);
    }
    let written = writer.finish().map_err(|e| e.to_string())?;
    if written as usize != total {
        return Err(format!("wrote {written} tokens, expected {total}"));
    }
    // The mask must be exactly as long as the token stream: the trainer reads
    // them as parallel arrays, and a short mask would silently misalign every
    // document after the first divergence. `total` is the sum of the encoded
    // token vectors, so the two agree by construction unless the encoder's
    // `tokens` and `response_mask` are themselves different lengths, which is
    // checked here because it is the one way this can still be wrong.
    if mask.len() != total {
        return Err(format!(
            "the mask is {} bytes for {total} tokens",
            mask.len()
        ));
    }
    let mask_path = out.join("response_mask.u8");
    fs::write(&mask_path, &mask).map_err(|e| e.to_string())?;

    let tokens_sha = uor_r4_training::sha256_file(&tokens_path).map_err(|e| e.to_string())?;
    let mask_sha = uor_r4_training::sha256_file(&mask_path).map_err(|e| e.to_string())?;
    let tokenizer_sha = uor_r4_training::sha256_file(&tokenizer_path).map_err(|e| e.to_string())?;
    let manifest = json!({
        "schema": "uor-r4-chat-corpus/v1",
        "mask_schema": "uor-r4-response-mask/u8/v1",
        "split": split,
        "template_rule": "<|bos|> then turns joined by a single '\\n' separator (placed before every turn after the first); a turn is '<marker><content>' with markers 'System: ', 'User: ', 'Assistant: '; every assistant turn ends with <|eos|>; a document-terminal <|eos|> is appended only when the final emitted turn is not assistant. Content is \\r\\n/\\r-normalised and trimmed; interior whitespace preserved.",
        "mask_rule": "1 = each token of an assistant turn's response content and its terminating <|eos|>; 0 = <|bos|>, all role markers, turn separators, system/user content, and an unmasked document-terminal <|eos|>.",
        // Required by `Split::load`, which reads `drops.special_token_occurrences`
        // and a `files` array whose per-source token counts must exactly cover
        // the store. A missing key fails the check rather than defaulting, so a
        // generator that omits these produces a store the trainer refuses with
        // "a source holds literal special-token text" -- an error about missing
        // metadata, not about the tokens.
        "drops": {
            "rows_dropped_no_messages": 0,
            "rows_dropped_empty": 0,
            "rows_dropped_no_response": 0,
            "rows_dropped_oversized": 0,
            "special_token_occurrences": 0
        },
        "files": [{
            "label": "synthetic-memory-rows",
            "tokens": total,
            "response_tokens": response_tokens,
            "special_token_occurrences": 0,
            "rows_used": rows,
            "rows_total": rows
        }],
        "synthetic": {
            "generator": "geometric-stack synthetic-memory",
            "why": "the value path needs a distribution of asked-relation vs competing-statement bindings; the ten-row development panel is about 500 tokens",
            "rows": rows,
            "seed": seed,
            "relations": RELATIONS.iter().map(|r| r.id).collect::<Vec<_>>(),
            "properties": [
                "the asked relation and the competing statement are always both stated",
                "the answer is always the asked relation's value",
                "the competing relation is never the asked relation",
                "the asked statement is first half the time and second half the time",
                "overlapping relations (colour/car) sometimes share a competing value"
            ]
        },
        "tokenizer": {"path": tokenizer_path.display().to_string(), "sha256": tokenizer_sha},
        "dialogue_protocol": protocol.schema,
        "bos_id": bos,
        "eos_id": eos,
        "rows_used": rows,
        "tokens": total,
        "response_tokens": response_tokens,
        "response_fraction": response_tokens as f64 / total as f64,
        "tokens_bytes": fs::metadata(&tokens_path).map_err(|e| e.to_string())?.len(),
        "mask_bytes": mask.len(),
        "tokens_sha256": tokens_sha,
        "mask_sha256": mask_sha,
    });
    fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!(
        "{rows} documents, {total} tokens, {response_tokens} response tokens ({:.1}%)",
        100.0 * response_tokens as f64 / total as f64
    );
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = run(&args) {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
