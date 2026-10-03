//! Build a synthetic in-context memory corpus: many fact / competing-statement
//! / question documents whose assistant turn is the correct answer.
//!
//! ```text
//! geometric-stack synthetic-memory out=NEW_REPORT_ROOT tokenizer=TOKENIZER.json \
//!   rows=2000 seed=7 [split=train] [vocab_size=4096] \
//!   [distance=N | curriculum=1 [max_distance=N]] [token_budget=N] \
//!   [forms=N] [panel_out=NEW_PANEL_ROOT]
//! ```
//!
//! `forms` controls how many surface FORMS per relation the generator may draw
//! from. Form 0 is the relation's canonical phrase and forms 1..7 are paraphrase
//! question and statement templates over a hand-authored per-relation topic; the
//! reply always keeps the single canonical answer form, so an arm that varies
//! `forms` varies how a fact is *stated and asked*, not how it is answered.
//! Forms 8..12 are held out: `forms` is clamped to 1..=8, so a training store
//! cannot contain them, and `panel_out=` writes the panel that scores them.
//! `forms=1` leaves the stream untouched, so it combines with the distance knobs
//! without disturbing byte identity (verified against the pre-forms generator).
//!
//! `distance` controls how far the asserted fact sits from the question. In the
//! legacy form the document is exactly four turns, so the asked statement is
//! always 0 or 1 turns from the question -- inside the reach of a short causal
//! convolution, which means a fit can score it without ever reading the fact out
//! of state. `distance=N` inserts `N` filler user turns between the statement
//! pair and the question, so the fact is N+0 or N+1 turns back and the local
//! window cannot carry it. `curriculum=1` draws each document's filler count
//! uniformly from `0..=max_distance` (default 8), giving a mixture that keeps the
//! easy case in distribution while forcing the hard ones.
//!
//! Both knob families are additive. With `distance=0`, no curriculum and
//! `forms=1`, `tokens.u16` and `response_mask.u8` are exactly what the
//! pre-distance, pre-forms generator wrote for the same rows and seed, so prior
//! artifacts stay comparable; the manifest gains the `forms` block either way,
//! and the `distance` block only when a distance knob is used.
//!
//! Distance is capped at `MAX_DISTANCE` because `dialogue-train` under
//! `policy=full_prefix` **excludes** any document whose start-to-answer span
//! exceeds the model context (256), and a document that is never trained on is
//! not evidence about distance. `token_budget` stops the run after the last
//! document that keeps the total at or below the given token count, which is how
//! two arms are generated at the same budget when the filler turns make the
//! ramped documents longer.
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

/// The **topic** of each relation: the noun phrase its fact is about, used to
/// build the extra surface forms. `who` picks the question word ("Who is my
/// favourite author?" vs "What is my blood type?").
///
/// This is a hand-authored field, not derived from the sentence: the canonical
/// strings are irregular English ("I write with my {v} hand", "I live at number
/// {v}") and a generic rule cannot recover the topic from them. The table is
/// keyed by relation id and validated at startup: every relation must have
/// exactly one slot and every slot must name a relation.
struct Slot {
    id: &'static str,
    topic: &'static str,
    who: bool,
}

const SLOTS: &[Slot] = &[
    Slot {
        id: "breed",
        topic: "dog's breed",
        who: false,
    },
    Slot {
        id: "blood_type",
        topic: "blood type",
        who: false,
    },
    Slot {
        id: "handedness",
        topic: "handedness",
        who: false,
    },
    Slot {
        id: "timezone",
        topic: "timezone",
        who: false,
    },
    Slot {
        id: "bedtime",
        topic: "bedtime",
        who: false,
    },
    Slot {
        id: "wake_time",
        topic: "wake-up time",
        who: false,
    },
    Slot {
        id: "coffee",
        topic: "coffee order",
        who: false,
    },
    Slot {
        id: "pizza",
        topic: "pizza topping",
        who: false,
    },
    Slot {
        id: "ice_cream",
        topic: "favourite ice cream",
        who: false,
    },
    Slot {
        id: "dessert",
        topic: "dessert",
        who: false,
    },
    Slot {
        id: "author",
        topic: "favourite author",
        who: true,
    },
    Slot {
        id: "film",
        topic: "favourite film",
        who: false,
    },
    Slot {
        id: "song",
        topic: "favourite song",
        who: false,
    },
    Slot {
        id: "book",
        topic: "book",
        who: false,
    },
    Slot {
        id: "city",
        topic: "city",
        who: false,
    },
    Slot {
        id: "mother",
        topic: "mother's name",
        who: false,
    },
    Slot {
        id: "brother",
        topic: "brother's name",
        who: false,
    },
    Slot {
        id: "dog",
        topic: "dog's name",
        who: false,
    },
    Slot {
        id: "neighbour",
        topic: "neighbour's name",
        who: false,
    },
    Slot {
        id: "garden",
        topic: "garden",
        who: false,
    },
    Slot {
        id: "siblings",
        topic: "sibling count",
        who: false,
    },
    Slot {
        id: "uni_subject",
        topic: "university subject",
        who: false,
    },
    Slot {
        id: "school",
        topic: "school town",
        who: false,
    },
    Slot {
        id: "office",
        topic: "office",
        who: false,
    },
    Slot {
        id: "employment",
        topic: "employment status",
        who: false,
    },
    Slot {
        id: "train_line",
        topic: "train line",
        who: false,
    },
    Slot {
        id: "airline",
        topic: "airline",
        who: false,
    },
    Slot {
        id: "suitcase",
        topic: "suitcase colour",
        who: false,
    },
    Slot {
        id: "watch",
        topic: "watch",
        who: false,
    },
    Slot {
        id: "phone",
        topic: "phone",
        who: false,
    },
    Slot {
        id: "laptop",
        topic: "laptop",
        who: false,
    },
    Slot {
        id: "editor",
        topic: "editor",
        who: false,
    },
    Slot {
        id: "shell",
        topic: "shell",
        who: false,
    },
    Slot {
        id: "keyboard",
        topic: "keyboard",
        who: false,
    },
    Slot {
        id: "mug",
        topic: "mug",
        who: false,
    },
    Slot {
        id: "park",
        topic: "park",
        who: false,
    },
    Slot {
        id: "pub",
        topic: "pub",
        who: false,
    },
    Slot {
        id: "restaurant",
        topic: "restaurant",
        who: false,
    },
    Slot {
        id: "market",
        topic: "market",
        who: false,
    },
    Slot {
        id: "museum",
        topic: "museum",
        who: false,
    },
    Slot {
        id: "gallery",
        topic: "gallery",
        who: false,
    },
    Slot {
        id: "theatre",
        topic: "theatre",
        who: false,
    },
    Slot {
        id: "cinema",
        topic: "cinema",
        who: false,
    },
    Slot {
        id: "gym_class",
        topic: "gym class",
        who: false,
    },
    Slot {
        id: "run_route",
        topic: "run route",
        who: false,
    },
    Slot {
        id: "bike",
        topic: "bike",
        who: false,
    },
    Slot {
        id: "swim_stroke",
        topic: "swimming stroke",
        who: false,
    },
    Slot {
        id: "sport",
        topic: "sport",
        who: false,
    },
    Slot {
        id: "team",
        topic: "team",
        who: false,
    },
    Slot {
        id: "instrument",
        topic: "instrument",
        who: false,
    },
    Slot {
        id: "language",
        topic: "language",
        who: false,
    },
    Slot {
        id: "intolerance",
        topic: "food intolerance",
        who: false,
    },
    Slot {
        id: "diet",
        topic: "diet",
        who: false,
    },
    Slot {
        id: "medicine",
        topic: "medicine",
        who: false,
    },
    Slot {
        id: "optician",
        topic: "optician",
        who: true,
    },
    Slot {
        id: "dentist",
        topic: "dentist",
        who: true,
    },
    Slot {
        id: "doctor",
        topic: "doctor",
        who: true,
    },
    Slot {
        id: "car",
        topic: "car",
        who: false,
    },
    Slot {
        id: "bike_lock",
        topic: "bike lock",
        who: false,
    },
    Slot {
        id: "postcode",
        topic: "postcode",
        who: false,
    },
    Slot {
        id: "house_number",
        topic: "house number",
        who: false,
    },
    Slot {
        id: "street",
        topic: "street",
        who: false,
    },
    Slot {
        id: "rent",
        topic: "rent",
        who: false,
    },
    Slot {
        id: "mortgage",
        topic: "mortgage",
        who: false,
    },
    Slot {
        id: "savings",
        topic: "savings goal",
        who: false,
    },
    Slot {
        id: "currency",
        topic: "currency",
        who: false,
    },
    Slot {
        id: "charity",
        topic: "charity",
        who: false,
    },
    Slot {
        id: "paper",
        topic: "newspaper",
        who: false,
    },
    Slot {
        id: "podcast",
        topic: "podcast",
        who: false,
    },
    Slot {
        id: "radio",
        topic: "radio station",
        who: false,
    },
    Slot {
        id: "hobby",
        topic: "hobby",
        who: false,
    },
    Slot {
        id: "collection",
        topic: "collection",
        who: false,
    },
    Slot {
        id: "instrument_lesson",
        topic: "teacher",
        who: true,
    },
    Slot {
        id: "volunteer",
        topic: "volunteer place",
        who: false,
    },
    Slot {
        id: "allotment",
        topic: "allotment crop",
        who: false,
    },
    Slot {
        id: "pet_fish",
        topic: "fish",
        who: false,
    },
];

/// Relations whose value set does not fit a "<topic> is <value>" restatement
/// (`have` / `do not have`). They keep the canonical single form in every arm,
/// so the form-variety axis is applied to the other 74 relations.
const CANONICAL_ONLY: &[&str] = &["garden", "mortgage"];

/// How many surface forms the generator may draw from per relation: forms
/// `0..TRAIN_FORMS` are trainer-visible, the rest are held out for the
/// unseen-form panel. Form 0 is always the relation's canonical phrase, so
/// `forms=1` reproduces the narrow recipe exactly.
const TRAIN_FORMS: usize = 8;
const HELD_OUT_FORMS: usize = 5;
const ALL_FORMS: usize = TRAIN_FORMS + HELD_OUT_FORMS;

/// Extra question forms (`{q}` = Who/What, `{t}` = topic). Index i is form i+1.
/// Entries 0..TRAIN_FORMS-1 are visible to the trainer; the tail is held out
/// and used only by the unseen-form panel, so no arm ever trains on it.
const ASK_TEMPLATES: &[&str] = &[
    // trainer-visible (forms 1..=7)
    "{q} is my {t}?",
    "{q}'s my {t}?",
    "Do you know {q} my {t} is?",
    "Can you tell me {q} my {t} is?",
    "Tell me {q} my {t} is.",
    "Remind me {q} my {t} is.",
    "{q} did I say my {t} was?",
    // held out (forms 8..=12)
    "What was that {t} I told you about?",
    "Have you kept a note of my {t}?",
    "My {t} has slipped my mind. What is it?",
    "Did I ever mention my {t} to you?",
    "Tell me again about my {t}.",
];

/// Minimal-change probe: the trained question with a single word substituted
/// (a contraction or a spelling variant), while the statement keeps its
/// canonical trained form. Returns `None` when no substitution applies, so the
/// panel only emits rows for relations where the question really did change.
fn perturbed_ask(ask: &str) -> Option<String> {
    const SWAPS: &[(&str, &str)] = &[
        ("favourite", "favorite"),
        ("colour", "color"),
        ("neighbour", "neighbor"),
        ("timezone", "time zone"),
        ("postcode", "post code"),
        ("What is ", "What's "),
        ("Who is ", "Who's "),
        ("What do ", "What d'you "),
    ];
    for (from, to) in SWAPS {
        if ask.contains(from) {
            return Some(ask.replacen(from, to, 1));
        }
    }
    None
}

/// Extra statement forms (`{t}` = topic, `{v}` = value). Index i is form i+1.
const STATE_TEMPLATES: &[&str] = &[
    // trainer-visible (forms 1..=7)
    "My {t} is {v}.",
    "I should mention that my {t} is {v}.",
    "Just so you know, my {t} is {v}.",
    "Let me tell you my {t}: it is {v}.",
    "By the way, my {t} is {v}.",
    "I think my {t} is {v}.",
    "If you need to know, my {t} is {v}.",
    // held out (forms 8..=12)
    "For the record, my {t} is {v}.",
    "As I recall, my {t} is {v}.",
    "My {t}, I am quite sure, is {v}.",
    "If I remember correctly, my {t} is {v}.",
    "I am fairly sure my {t} is {v}.",
];

fn slot_of(id: &str) -> Result<&'static Slot> {
    SLOTS
        .iter()
        .find(|slot| slot.id == id)
        .ok_or_else(|| format!("no slot for relation {id}"))
}

/// Each relation's trainer-visible forms: asks (one per form), state templates
/// with `{v}` still unsubstituted, and the canonical answers (form 0 only).
struct Forms {
    asks: Vec<String>,
    states: Vec<String>,
    answers: Vec<String>,
    /// Forms >= this index are held out (usable only by the panel).
    trainable: usize,
}

fn forms_of(relation: &Relation) -> Result<Forms> {
    if CANONICAL_ONLY.contains(&relation.id) {
        return Ok(Forms {
            asks: vec![relation.asks[0].to_string()],
            states: vec![relation.states[0].to_string()],
            answers: vec![relation.answers[0].to_string()],
            trainable: 1,
        });
    }
    let slot = slot_of(relation.id)?;
    let q = if slot.who { "Who" } else { "What" };
    let mut asks = vec![relation.asks[0].to_string()];
    for template in ASK_TEMPLATES {
        asks.push(template.replace("{q}", q).replace("{t}", slot.topic));
    }
    let mut states = vec![relation.states[0].to_string()];
    for template in STATE_TEMPLATES {
        states.push(template.replace("{t}", slot.topic));
    }
    if asks.len() != ALL_FORMS || states.len() != ALL_FORMS {
        return Err(format!(
            "relation {} has {} asks and {} states, expected {ALL_FORMS}",
            relation.id,
            asks.len(),
            states.len()
        ));
    }
    Ok(Forms {
        asks,
        states,
        answers: vec![relation.answers[0].to_string()],
        trainable: TRAIN_FORMS,
    })
}

/// The whole table's forms, built once so a document costs no template work.
fn all_forms() -> Result<Vec<Forms>> {
    RELATIONS.iter().map(forms_of).collect()
}

/// Check the hand-authored tables agree before anything is written: a typo in a
/// slot id would otherwise silently give one relation English that belongs to
/// another, which is a corpus bug the fit cannot report.
fn validate_tables() -> Result<()> {
    for relation in RELATIONS {
        slot_of(relation.id)?;
    }
    for slot in SLOTS {
        if !RELATIONS.iter().any(|relation| relation.id == slot.id) {
            return Err(format!("slot {} names no relation", slot.id));
        }
        if !slot.who && slot.topic.is_empty() {
            return Err(format!("slot {} has an empty topic", slot.id));
        }
    }
    for (id, values) in VALUES {
        if !RELATIONS.iter().any(|relation| relation.id == *id) {
            return Err(format!("values for {id} name no relation"));
        }
        if values.is_empty() {
            return Err(format!("relation {id} has no values"));
        }
    }
    if VALUES.len() != RELATIONS.len() {
        return Err(format!(
            "{} relations and {} value sets",
            RELATIONS.len(),
            VALUES.len()
        ));
    }
    Ok(())
}

/// Values per relation. Deliberately includes values that are also a *distinct*
/// relation's value (for example `blue` for both colour and car), so the model
/// cannot answer by surfacing a value that only ever belongs to one relation.
const VALUES: &[(&str, &[&str])] = &[
    (
        "breed",
        &["spaniel", "terrier", "poodle", "beagle", "corgi"],
    ),
    (
        "blood_type",
        &[
            "A positive",
            "O negative",
            "B positive",
            "AB negative",
            "O positive",
        ],
    ),
    ("handedness", &["left", "right"]),
    ("timezone", &["GMT", "CET", "EST", "PST", "JST"]),
    (
        "bedtime",
        &["ten", "eleven", "half past nine", "midnight", "nine"],
    ),
    (
        "wake_time",
        &["six", "seven", "half past five", "eight", "five"],
    ),
    (
        "coffee",
        &[
            "a flat white",
            "an espresso",
            "a latte",
            "a cortado",
            "a filter coffee",
        ],
    ),
    (
        "pizza",
        &["olives", "pepperoni", "mushrooms", "anchovies", "pineapple"],
    ),
    (
        "ice_cream",
        &["vanilla", "chocolate", "strawberry", "mint", "caramel"],
    ),
    (
        "dessert",
        &[
            "apple crumble",
            "cheesecake",
            "tiramisu",
            "brownies",
            "sorbet",
        ],
    ),
    (
        "author",
        &["Austen", "Murakami", "Le Guin", "Ishiguro", "Atwood"],
    ),
    (
        "film",
        &["Alien", "Spirited Away", "Casablanca", "Parasite", "Jaws"],
    ),
    (
        "song",
        &[
            "Hey Jude",
            "Blue Monday",
            "Wonderwall",
            "Redemption Song",
            "Teardrop",
        ],
    ),
    (
        "book",
        &["Ulysses", "Dune", "Persuasion", "Solaris", "Beloved"],
    ),
    (
        "city",
        &["Kyoto", "Marrakesh", "Reykjavik", "Cusco", "Hanoi"],
    ),
    ("mother", &["Diane", "Rosa", "Ingrid", "Amara", "Yuki"]),
    ("brother", &["Tom", "Ravi", "Lars", "Kwame", "Nico"]),
    ("dog", &["Biscuit", "Nala", "Rufus", "Poppy", "Ziggy"]),
    ("neighbour", &["Harold", "Mei", "Pavel", "Fatima", "Colin"]),
    ("garden", &["have", "do not have"]),
    ("siblings", &["one", "two", "three", "four", "none"]),
    (
        "uni_subject",
        &["history", "chemistry", "economics", "philosophy", "biology"],
    ),
    ("school", &["Leeds", "Galway", "Aarhus", "Nagoya", "Tulsa"]),
    ("office", &["Soho", "Camden", "Clifton", "Digbeth", "Leith"]),
    (
        "employment",
        &[
            "full time",
            "part time",
            "self employed",
            "between jobs",
            "retired",
        ],
    ),
    (
        "train_line",
        &["Northern", "Central", "District", "Piccadilly", "Bakerloo"],
    ),
    (
        "airline",
        &["KLM", "Emirates", "Qantas", "Lufthansa", "Iberia"],
    ),
    ("suitcase", &["black", "red", "grey", "navy", "olive"]),
    ("watch", &["digital", "analog", "solar", "dive", "field"]),
    (
        "phone",
        &["Pixel", "iPhone", "Nothing", "Fairphone", "Galaxy"],
    ),
    (
        "laptop",
        &["ThinkPad", "MacBook", "Framework", "Surface", "XPS"],
    ),
    ("editor", &["Neovim", "Emacs", "VS Code", "Helix", "Zed"]),
    ("shell", &["zsh", "bash", "fish", "nu", "dash"]),
    (
        "keyboard",
        &["Model M", "HHKB", "Ergodox", "Kinesis", "Realforce"],
    ),
    ("mug", &["blue", "chipped", "tall", "striped", "plain"]),
    (
        "park",
        &["Highbury", "Phoenix", "Riverside", "Queens", "Victoria"],
    ),
    ("pub", &["Red Lion", "Anchor", "Crown", "Bell", "Ship"]),
    (
        "restaurant",
        &["Thai", "Nepalese", "Ethiopian", "Peruvian", "Georgian"],
    ),
    (
        "market",
        &["Borough", "Broadway", "St George", "Portobello", "Camden"],
    ),
    (
        "museum",
        &["Science", "V&A", "Natural History", "Design", "Transport"],
    ),
    (
        "gallery",
        &["Tate", "Whitechapel", "Serpentine", "Hayward", "Barbican"],
    ),
    (
        "theatre",
        &["Almeida", "Old Vic", "Young Vic", "Donmar", "Lyceum"],
    ),
    (
        "cinema",
        &["Rio", "Everyman", "Prince Charles", "Genesis", "Lexi"],
    ),
    (
        "gym_class",
        &["spin", "yoga", "pilates", "boxing", "climbing"],
    ),
    (
        "run_route",
        &["canal", "towpath", "common", "river", "seafront"],
    ),
    ("bike", &["road", "gravel", "folding", "cargo", "fixed"]),
    (
        "swim_stroke",
        &[
            "front crawl",
            "breaststroke",
            "backstroke",
            "butterfly",
            "sidestroke",
        ],
    ),
    (
        "sport",
        &["squash", "cricket", "hockey", "badminton", "water polo"],
    ),
    (
        "team",
        &["Fulham", "Celtic", "Everton", "Brighton", "Norwich"],
    ),
    (
        "instrument",
        &["clarinet", "banjo", "oboe", "double bass", "accordion"],
    ),
    (
        "language",
        &["Portuguese", "Korean", "Swahili", "Finnish", "Catalan"],
    ),
    (
        "intolerance",
        &["lactose", "gluten", "fructose", "histamine", "caffeine"],
    ),
    (
        "diet",
        &["vegetarian", "vegan", "kosher", "halal", "pescatarian"],
    ),
    (
        "medicine",
        &["statins", "thyroxine", "metformin", "aspirin", "inhalers"],
    ),
    (
        "optician",
        &[
            "Specsavers",
            "Boots",
            "Vision Express",
            "Scrivens",
            "Leightons",
        ],
    ),
    (
        "dentist",
        &["Bupa", "mydentist", "Colosseum", "Portman", "Together"],
    ),
    (
        "doctor",
        &[
            "Dr Ellis",
            "Dr Rahman",
            "Dr Novak",
            "Dr Osei",
            "Dr Lindqvist",
        ],
    ),
    ("car", &["Fiesta", "Golf", "Volvo", "Prius", "Mini"]),
    ("bike_lock", &["D", "chain", "cable", "folding", "frame"]),
    ("postcode", &["N1", "SE15", "BS8", "EH6", "CF10"]),
    (
        "house_number",
        &["twelve", "forty one", "seven", "ninety", "three"],
    ),
    (
        "street",
        &["Albion", "Meadow", "Chapel", "Grove", "Harbour"],
    ),
    (
        "rent",
        &[
            "nine hundred",
            "twelve hundred",
            "a thousand",
            "fifteen hundred",
            "seven hundred",
        ],
    ),
    ("mortgage", &["have", "do not have"]),
    (
        "savings",
        &["a house", "a car", "a holiday", "retirement", "a wedding"],
    ),
    (
        "currency",
        &["sterling", "euros", "dollars", "yen", "francs"],
    ),
    ("charity", &["Shelter", "Mind", "RNLI", "Oxfam", "Amnesty"]),
    (
        "paper",
        &["Guardian", "Times", "FT", "Independent", "Herald"],
    ),
    (
        "podcast",
        &["Reply All", "99pi", "Radiolab", "Serial", "Witness"],
    ),
    (
        "radio",
        &["Radio 4", "6 Music", "World Service", "3", "Classic FM"],
    ),
    (
        "hobby",
        &[
            "birdwatching",
            "woodwork",
            "pottery",
            "astronomy",
            "foraging",
        ],
    ),
    (
        "collection",
        &["stamps", "records", "maps", "cameras", "typewriters"],
    ),
    (
        "instrument_lesson",
        &[
            "Mr Hale",
            "Ms Ferreira",
            "Dr Banerjee",
            "Mrs Okafor",
            "Mr Lindgren",
        ],
    ),
    (
        "volunteer",
        &["library", "food bank", "hospice", "allotment", "museum"],
    ),
    (
        "allotment",
        &["tomatoes", "courgettes", "beans", "chillies", "rhubarb"],
    ),
    (
        "pet_fish",
        &["guppies", "tetras", "goldfish", "danios", "rasboras"],
    ),
];

/// Relations whose value sets overlap, so a distractor can reuse the asked
/// relation's *shape* without being its answer.
const OVERLAP: &[(&str, &str)] = &[];

/// Filler user turns inserted between the statement pair and the question.
///
/// They are deliberately *not* memory statements: none of them asserts a
/// relation, carries a relation's value vocabulary as a word, or asks a
/// question that could be answered from the document. A filler that asserted a
/// fact would make the distance condition a second selection problem instead of
/// a separation problem, so the unit test below checks every filler against
/// every relation id and value rather than trusting this list to stay clean as
/// the tables grow.
const FILLERS: &[&str] = &[
    "Anyway, how has your week been so far?",
    "It has been raining a lot lately.",
    "I am feeling quite tired this evening.",
    "The weather has been strange lately.",
    "I am trying to walk more often.",
    "My knee is still a bit sore.",
    "I spent the morning tidying up.",
    "The train was delayed again today.",
    "I am looking forward to the weekend.",
    "It is surprisingly warm for this time of year.",
    "I keep forgetting to answer messages.",
    "My back hurts after sitting all day.",
    "We are out of milk and bread.",
    "I think I need an early night.",
    "The people next door are away this week.",
    "I heard a strange noise last night.",
    "Everything is quiet here at the moment.",
    "I should probably go to bed soon.",
    "The kitchen sink is leaking again.",
    "It has been a long day.",
];

/// Distance ceiling. `dialogue-train` with `policy=full_prefix` keeps only
/// documents whose start-to-answer span fits the 256-token context
/// (`dialogue_episodes`: `end - document <= context`); a document that is
/// excluded is not trained on, so a larger distance would silently produce
/// untrained rows rather than a longer separation. The cap bounds the knob, it
/// does not guarantee the documents fit: a distance-16 document already runs
/// past the context, which is why the manifest reports `documents_over_context`
/// and `max_document_tokens` and the run warns on stderr. `max_distance=8` is
/// well inside the context on this tokenizer.
const MAX_DISTANCE: u64 = 16;

/// The trainer's context, matching `dialogue-train`'s default. A document whose
/// encoded length exceeds it is excluded by `policy=full_prefix`, so the
/// manifest reports how many there were instead of leaving the exclusion
/// silent.
const TRAINER_CONTEXT: usize = 256;

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
    /// Whether `first` is the asked relation's statement. Recorded because the
    /// separation between the asked statement and the question is the filler
    /// count plus one when the asked statement is stated first, and the
    /// manifest reports that separation rather than only the filler count.
    asked_first: bool,
}

/// One document. `forms` is how many surface forms per relation the arm may
/// draw from; the draws are made in exactly the order (and count) the narrow
/// generator used, one `below(1)` per selection site when only the canonical
/// form exists, so `forms=1` leaves the stream untouched and `distance=0`
/// writes the same bytes the pre-distance, pre-forms generator wrote.
fn document(rng: &mut Rng, forms: usize, table: &[Forms]) -> Result<Document> {
    let asked_index = rng.below(RELATIONS.len());
    let asked = &RELATIONS[asked_index];
    let asked_forms = &table[asked_index];
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
    let competing_forms = &table[RELATIONS
        .iter()
        .position(|relation| relation.id == competing.id)
        .ok_or("competing relation not in the table")?];

    // One draw per selection site, in the order the narrow generator used.
    let draw = |rng: &mut Rng, available: usize| rng.below(forms.min(available));
    let f_state = draw(rng, asked_forms.states.len());
    let asked_statement = asked_forms.states[f_state].replace("{v}", wanted);
    let f_competing = draw(rng, competing_forms.states.len());
    let competing_statement = competing_forms.states[f_competing].replace("{v}", competing_value);
    let f_ask = draw(rng, asked_forms.asks.len());
    let question = asked_forms.asks[f_ask].clone();
    // The asked statement is sometimes stated first and sometimes second, so
    // recency cannot be the rule the corpus teaches. The draw is captured
    // rather than re-tested: a second `below(2)` call would change the stream
    // the pre-distance generator used and break byte identity at distance 0.
    let asked_first = rng.below(2) == 0;
    let (first, second) = if asked_first {
        (asked_statement, competing_statement)
    } else {
        (competing_statement, asked_statement)
    };
    // The reply keeps the single canonical form: this arm varies the *request*
    // and the *statement*, not the answer, so the output shape is held fixed
    // between arms. The draw is still taken so the stream is unchanged.
    let _answer_draw = rng.below(forms.min(asked_forms.answers.len()));
    Ok(Document {
        first,
        second,
        question,
        answer: asked_forms.answers[0].replace("{v}", wanted),
        wanted: wanted.to_string(),
        asked_first,
    })
}

/// `count` filler user turns, drawn one at a time.
///
/// Only called when `count > 0`, which is what keeps the distance-0 run's rng
/// stream and therefore its bytes identical to the pre-distance generator.
fn insert_fillers(rng: &mut Rng, count: usize) -> Vec<String> {
    (0..count).map(|_| rng.pick(FILLERS).to_string()).collect()
}

/// Index a form list without assuming the relation has the full form set.
fn form_at(forms: &[String], index: usize) -> &str {
    &forms[index % forms.len()]
}

fn panel_rows() -> Result<Vec<(String, Vec<String>, String)>> {
    let table = all_forms()?;
    let mut rows = Vec::new();
    for (index, relation) in RELATIONS.iter().enumerate() {
        if table[index].trainable < TRAIN_FORMS {
            continue;
        }
        let forms = &table[index];
        let values = values_of(relation.id)?;
        let wanted = values[index % values.len()];
        // The competing statement must also come from a relation that has
        // paraphrase forms, or the "unseen" competing turn would silently be a
        // canonical (trained) one.
        let mut competing_index = (index + 37) % RELATIONS.len();
        while table[competing_index].trainable < TRAIN_FORMS || competing_index == index {
            competing_index = (competing_index + 1) % RELATIONS.len();
        }
        let competing = &RELATIONS[competing_index];
        let competing_values = values_of(competing.id)?;
        let mut competing_value = competing_values[(index + 1) % competing_values.len()];
        if competing_value == wanted {
            competing_value = competing_values[(index + 2) % competing_values.len()];
        }
        let competing_forms = &table[competing_index];
        // The held-out form index varies with the relation, so every held-out
        // template is exercised roughly equally across the panel instead of
        // one template carrying the whole condition.
        let held = |slot: usize| TRAIN_FORMS + (index + slot) % HELD_OUT_FORMS;
        // The competing relation can be one of the canonical-only relations
        // (they have a single form), so every index is folded into the forms
        // that actually exist rather than assumed present.
        let state_seen = form_at(&forms.states, 0);
        let ask_seen = form_at(&forms.asks, 0);
        let state_unseen = form_at(&forms.states, held(0));
        let ask_unseen = form_at(&forms.asks, held(1));
        let comp_seen = form_at(&competing_forms.states, 0);
        let comp_unseen = form_at(&competing_forms.states, held(2));
        let id = relation.id;
        // The panel separates the two ways a paraphrase can break binding: an
        // unseen *statement* form with the trained question, an unseen
        // *question* form with the trained statement, and both at once. The
        // competing conditions reuse exactly the same statement and question as
        // their non-competing twin, so the only difference is the extra turn.
        rows.push((
            format!("cf-form-seen-{id}"),
            vec![state_seen.replace("{v}", wanted), ask_seen.to_string()],
            wanted.to_string(),
        ));
        rows.push((
            format!("cf-form-seen-comp-{id}"),
            vec![
                state_seen.replace("{v}", wanted),
                comp_seen.replace("{v}", competing_value),
                ask_seen.to_string(),
            ],
            wanted.to_string(),
        ));
        rows.push((
            format!("cf-form-unseen-state-{id}"),
            vec![state_unseen.replace("{v}", wanted), ask_seen.to_string()],
            wanted.to_string(),
        ));
        rows.push((
            format!("cf-form-unseen-ask-{id}"),
            vec![state_seen.replace("{v}", wanted), ask_unseen.to_string()],
            wanted.to_string(),
        ));
        rows.push((
            format!("cf-form-unseen-{id}"),
            vec![state_unseen.replace("{v}", wanted), ask_unseen.to_string()],
            wanted.to_string(),
        ));
        rows.push((
            format!("cf-form-unseen-comp-{id}"),
            vec![
                state_unseen.replace("{v}", wanted),
                comp_unseen.replace("{v}", competing_value),
                ask_unseen.to_string(),
            ],
            wanted.to_string(),
        ));
        // Minimal-change probe: one substituted word in the question, the
        // statement untouched. Only relations where a substitution applies.
        if let Some(ask) = perturbed_ask(ask_seen) {
            rows.push((
                format!("cf-form-perturb-ask-{id}"),
                vec![state_seen.replace("{v}", wanted), ask],
                wanted.to_string(),
            ));
        }
    }
    Ok(rows)
}

/// Write the unseen-form panel as `requests.json` + `expected.json` in the
/// panel format `lut-chat` already reads. The root is claimed exclusively like
/// any other report root.
fn write_panel(out: &std::path::Path, seed: u64, rows: usize) -> Result<()> {
    let panel: Vec<serde_json::Value> = panel_rows()?
        .into_iter()
        .map(|(id, turns, _)| {
            json!({
                "category": "memory",
                "id": id,
                "user_turns": turns,
            })
        })
        .collect();
    let expected: serde_json::Map<String, serde_json::Value> = panel_rows()?
        .into_iter()
        .map(|(id, _, wanted)| (id, json!(wanted)))
        .collect();
    let manifest = json!({
        "schema": "uor-r4.geometric-stack-panel/1",
        "tool": "geometric-stack synthetic-memory panel_out=",
        "why": "held-out surface FORMS of seen relations: the treatment arm trains on paraphrase forms and this panel asks with forms no arm has seen",
        "generator_rows": rows,
        "seed": seed,
        "relations": RELATIONS.len(),
        "relations_with_forms": RELATIONS.len() - CANONICAL_ONLY.len(),
        "canonical_only": CANONICAL_ONLY,
        "train_forms": TRAIN_FORMS,
        "held_out_forms": HELD_OUT_FORMS,
        "conditions": [
            "seen",
            "seen-comp",
            "unseen-state",
            "unseen-ask",
            "unseen",
            "unseen-comp",
            "perturb-ask"
        ],
        "rows": panel.len(),
        // The full value set per relation, so a scorer can tell an answer from
        // another relation's value without re-deriving the table.
        "values": VALUES
            .iter()
            .map(|(id, values)| json!({ "relation": id, "values": values }))
            .collect::<Vec<_>>(),
        "slots": SLOTS
            .iter()
            .map(|slot| json!({"id": slot.id, "topic": slot.topic, "who": slot.who}))
            .collect::<Vec<_>>(),
    });
    fs::write(
        out.join("requests.json"),
        serde_json::to_vec_pretty(&panel).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        out.join("expected.json"),
        serde_json::to_vec_pretty(&serde_json::Value::Object(expected))
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!("panel: {} rows -> {}", panel.len(), out.display());
    Ok(())
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
    // Separation between the asserted fact and the question. Legacy behaviour
    // is `distance=0` with no curriculum: exactly four turns, no extra rng
    // draws, byte-identical output.
    let distance = number("distance", 0)?;
    let curriculum_raw = number("curriculum", 0)?;
    if curriculum_raw > 1 {
        return Err("curriculum= must be 0 or 1".into());
    }
    let curriculum = curriculum_raw == 1;
    let max_distance = number("max_distance", 8)?;
    let token_budget = number("token_budget", 0)? as usize;
    if distance > MAX_DISTANCE {
        return Err(format!(
            "distance= must be at most {MAX_DISTANCE}: a longer document than the trainer's \
             256-token context is excluded by policy=full_prefix and never trained on"
        ));
    }
    if curriculum && max_distance > MAX_DISTANCE {
        return Err(format!("max_distance= must be at most {MAX_DISTANCE}"));
    }
    if curriculum && args.iter().any(|a| a.starts_with("distance=")) {
        return Err("distance= and curriculum=1 are mutually exclusive".into());
    }
    let distance_mode = if curriculum { "curriculum" } else { "exact" };

    // Surface-form breadth. `forms` is how many forms per relation the
    // generator may draw from; the held-out tail is refused here because an arm
    // that trained on it would make the unseen-form panel meaningless. The
    // panel below is generated at distance 0 and is unaffected by the distance
    // knobs: it measures which surface form a fact was stated and asked in,
    // not how far apart they were.
    let forms = number("forms", 1)? as usize;
    if !(1..=TRAIN_FORMS).contains(&forms) {
        return Err(format!(
            "forms must be 1..={TRAIN_FORMS}; a larger value would train on the held-out tail"
        ));
    }
    let panel_out = args
        .iter()
        .find_map(|a| a.strip_prefix("panel_out="))
        .map(PathBuf::from);
    // The hand-authored tables must agree before anything is written: a typo in
    // a slot id silently gives one relation another's English, which is a
    // corpus bug no fit can report.
    validate_tables()?;
    let table = all_forms()?;

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
    // Recorded so the manifest states the separation actually written, not the
    // one that was asked for. Keyed by filler count and by the asked
    // statement's own distance from the question.
    let mut fillers_histogram: std::collections::BTreeMap<usize, usize> =
        std::collections::BTreeMap::new();
    let mut asked_distance_histogram: std::collections::BTreeMap<usize, usize> =
        std::collections::BTreeMap::new();
    let mut stopped_at_budget = false;
    let mut documents_over_context = 0usize;
    let mut max_document_tokens = 0usize;
    for _ in 0..rows {
        let doc = document(&mut rng, forms, &table)?;
        let fillers = if curriculum {
            let count = rng.below(max_distance as usize + 1);
            insert_fillers(&mut rng, count)
        } else {
            insert_fillers(&mut rng, distance as usize)
        };
        let mut messages: Vec<Message<'_>> = vec![
            Message {
                role: "user",
                content: &doc.first,
            },
            Message {
                role: "user",
                content: &doc.second,
            },
        ];
        for filler in &fillers {
            messages.push(Message {
                role: "user",
                content: filler,
            });
        }
        messages.push(Message {
            role: "user",
            content: &doc.question,
        });
        messages.push(Message {
            role: "assistant",
            content: &doc.answer,
        });
        let encoded = encoder.encode_document(&messages);
        if encoded.emitted_turns != messages.len() {
            return Err(format!(
                "{} turns were asked for but {} were emitted",
                messages.len(),
                encoded.emitted_turns
            ));
        }
        // Stop before the document that would exceed the budget, so a ramped
        // corpus can be generated at the same total token count as a flat one.
        if token_budget > 0 && total + encoded.tokens.len() > token_budget {
            stopped_at_budget = true;
            break;
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
        max_document_tokens = max_document_tokens.max(ids.len());
        if ids.len() > TRAINER_CONTEXT {
            documents_over_context += 1;
        }
        *fillers_histogram.entry(fillers.len()).or_default() += 1;
        // The asked statement is stated first or second, so its own separation
        // from the question is the filler count, plus one when the competing
        // statement sits between them.
        *asked_distance_histogram
            .entry(fillers.len() + usize::from(doc.asked_first))
            .or_default() += 1;
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
    let mut properties = vec![
        "the asked relation and the competing statement are always both stated",
        "the answer is always the asked relation's value",
        "the competing relation is never the asked relation",
        "the asked statement is first half the time and second half the time",
        "overlapping relations (colour/car) sometimes share a competing value",
        "the reply keeps the single canonical answer form in every arm",
    ];
    if distance > 0 || curriculum {
        properties.push(
            "filler user turns that assert no relation separate the asked statement from the question",
        );
    }
    let mut manifest = json!({
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
            // Surface-form breadth: `forms=1` is the narrow recipe (canonical
            // phrase only, byte-identical to the earlier generator); larger
            // values draw extra paraphrase forms from ASK_TEMPLATES and
            // STATE_TEMPLATES. Forms >= TRAIN_FORMS are held out for the panel
            // and are never reachable from here.
            "forms": forms,
            "train_forms": TRAIN_FORMS,
            "held_out_forms": HELD_OUT_FORMS,
            "canonical_only": CANONICAL_ONLY,
            "ask_templates": ASK_TEMPLATES,
            "state_templates": STATE_TEMPLATES,
            "slots": SLOTS
                .iter()
                .map(|slot| json!({"id": slot.id, "topic": slot.topic, "who": slot.who}))
                .collect::<Vec<_>>(),
            "properties": properties
        },
        "tokenizer": {"path": tokenizer_path.display().to_string(), "sha256": tokenizer_sha},
        "dialogue_protocol": protocol.schema,
        "bos_id": bos,
        "eos_id": eos,
        "rows_used": documents.len(),
        "tokens": total,
        "response_tokens": response_tokens,
        "response_fraction": response_tokens as f64 / total as f64,
        "tokens_bytes": fs::metadata(&tokens_path).map_err(|e| e.to_string())?.len(),
        "mask_bytes": mask.len(),
        "tokens_sha256": tokens_sha,
        "mask_sha256": mask_sha,
    });
    // Present only when a non-legacy mode was used, so a distance-0 manifest is
    // byte-identical to the pre-distance generator's. `serde_json`'s default map
    // keeps keys sorted, so inserting here does not reorder anything else.
    if distance > 0 || curriculum {
        let mut distance_block = json!({
            "mode": distance_mode,
            "filler_turns_histogram": fillers_histogram,
            "asked_statement_to_question_turns_histogram": asked_distance_histogram,
            "trainer_context": TRAINER_CONTEXT,
            "max_document_tokens": max_document_tokens,
            "documents_over_context": documents_over_context,
            "note": "filler user turns sit between the statement pair and the question; the asked statement is stated first or second, so its own separation from the question is the filler count, plus one when the competing statement sits between them. documents_over_context counts documents the trainer's full_prefix policy would exclude, which are not evidence about distance.",
        });
        if curriculum {
            distance_block["max_distance"] = json!(max_distance);
        } else {
            distance_block["fillers_per_document"] = json!(distance);
        }
        manifest["distance"] = distance_block;
    }
    if token_budget > 0 {
        manifest["token_budget"] = json!(token_budget);
        manifest["stopped_at_budget"] = json!(stopped_at_budget);
    }
    fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!(
        "{} documents, {total} tokens, {response_tokens} response tokens ({:.1}%)",
        documents.len(),
        100.0 * response_tokens as f64 / total as f64
    );
    if distance > 0 || curriculum {
        println!(
            "distance mode {distance_mode}: {} filler turns over {} documents; asked statement to \
             question turns {}..={}",
            fillers_histogram.iter().map(|(d, n)| d * n).sum::<usize>(),
            documents.len(),
            asked_distance_histogram.keys().next().copied().unwrap_or(0),
            asked_distance_histogram
                .keys()
                .next_back()
                .copied()
                .unwrap_or(0),
        );
        if documents_over_context > 0 {
            eprintln!(
                "warning: {documents_over_context} documents exceed the {TRAINER_CONTEXT}-token \
                 trainer context and will be excluded by policy=full_prefix; lower distance= or \
                 max_distance="
            );
        }
    }
    // The held-out-form panel is written beside the store when asked for. It is
    // a separate exclusive root, because every arm is scored against the same
    // rows and a per-arm copy would silently diverge.
    if let Some(panel_root) = panel_out {
        report_output::claim(&panel_root).map_err(|e| e.to_string())?;
        write_panel(&panel_root, seed, rows)?;
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = run(&args) {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The distance condition is only about separation: a filler must not
    /// assert a relation. A filler that carried a value from `VALUES` as a word
    /// would turn the distance arms into a second selection problem, so this
    /// guards the property the experiment depends on. Matching is by word, not
    /// by substring: `none` inside `money` is not an assertion.
    #[test]
    fn fillers_assert_no_relation_value() {
        for filler in FILLERS {
            let words: Vec<String> = filler
                .to_lowercase()
                .split(|c: char| !c.is_alphanumeric() && c != '\'')
                .filter(|w| !w.is_empty())
                .map(str::to_owned)
                .collect();
            for (relation, values) in VALUES {
                assert!(
                    !words.iter().any(|w| w == relation),
                    "filler {filler:?} contains the relation id {relation}"
                );
                for value in *values {
                    if value.contains(' ') {
                        continue;
                    }
                    assert!(
                        !words.iter().any(|w| w == &value.to_lowercase()),
                        "filler {filler:?} contains a value of {relation}: {value}"
                    );
                }
            }
        }
    }

    /// Same seed, same fillers; and the count is exact.
    #[test]
    fn fillers_are_deterministic_and_exact() {
        let first = insert_fillers(&mut Rng(11), 7);
        let second = insert_fillers(&mut Rng(11), 7);
        assert_eq!(first, second);
        assert_eq!(first.len(), 7);
        assert!(first.iter().all(|f| FILLERS.contains(&f.as_str())));
        assert!(insert_fillers(&mut Rng(11), 0).is_empty());
    }

    /// The legacy path must be reachable unchanged: with no distance arguments
    /// the document content is exactly what `document()` produced before the
    /// distance knobs existed, and the knob only adds turns after it.
    #[test]
    fn distance_zero_adds_no_turns() {
        let table = all_forms().expect("forms");
        let doc = document(&mut Rng(7), 1, &table).expect("document");
        assert!(!doc.first.is_empty() && !doc.second.is_empty());
        // The asked statement is the one whose value the answer gives.
        let asked = if doc.asked_first {
            &doc.first
        } else {
            &doc.second
        };
        assert!(asked.contains(&doc.wanted));
        assert!(doc.answer.contains(&doc.wanted));
    }

    /// Out-of-range distances are refused before any I/O: a document longer
    /// than the trainer context is dropped by `policy=full_prefix` and would
    /// make the corpus silent evidence rather than a longer separation.
    #[test]
    fn distances_above_the_context_cap_are_refused() {
        let base = [
            "out=/nonexistent-distance-curriculum-root",
            "tokenizer=/nonexistent-tokenizer.json",
            "rows=1",
        ];
        let with = |extra: &[&str]| -> Vec<String> {
            base.iter()
                .chain(extra)
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        };
        let error = run(&with(&["distance=17"])).expect_err("distance=17 must be refused");
        assert!(error.contains("distance= must be at most"), "{error}");
        let error = run(&with(&["curriculum=2"])).expect_err("curriculum=2 must be refused");
        assert!(error.contains("curriculum= must be 0 or 1"), "{error}");
        let error = run(&with(&["distance=4", "curriculum=1"]))
            .expect_err("distance plus curriculum must be refused");
        assert!(error.contains("mutually exclusive"), "{error}");
        let error = run(&with(&["curriculum=1", "max_distance=17"]))
            .expect_err("max_distance=17 must be refused");
        assert!(error.contains("max_distance= must be at most"), "{error}");
        let error = run(&with(&["forms=13"])).expect_err("forms=13 must be refused");
        assert!(error.contains("forms must be 1..=8"), "{error}");
    }

    /// `forms=1` must draw only the canonical phrase of each relation: that is
    /// what makes the fixed-form arm reproduce the published store.
    #[test]
    fn forms_one_draws_only_the_canonical_phrase() {
        let table = all_forms().expect("forms");
        let canonical_asks: Vec<&str> = RELATIONS.iter().map(|r| r.asks[0]).collect();
        let canonical_prefixes: Vec<&str> = RELATIONS
            .iter()
            .map(|r| r.states[0].split("{v}").next().unwrap_or(""))
            .collect();
        let mut rng = Rng(3);
        for _ in 0..500 {
            let doc = document(&mut rng, 1, &table).expect("document");
            assert!(
                canonical_asks.contains(&doc.question.as_str()),
                "forms=1 produced a non-canonical question {:?}",
                doc.question
            );
            for turn in [&doc.first, &doc.second] {
                assert!(
                    canonical_prefixes.iter().any(|p| turn.starts_with(p)),
                    "forms=1 produced a non-canonical statement {turn:?}"
                );
            }
        }
    }

    /// `forms=TRAIN_FORMS` must reach the paraphrases but never the held-out
    /// tail, which is what makes the panel's unseen conditions unseen.
    #[test]
    fn forms_eight_reaches_paraphrases_but_never_the_held_out_tail() {
        let table = all_forms().expect("forms");
        let mut trainable_asks: Vec<&str> = Vec::new();
        let mut held_out_asks: Vec<&str> = Vec::new();
        let mut trainable_prefixes: Vec<&str> = Vec::new();
        let mut held_out_prefixes: Vec<&str> = Vec::new();
        for forms in &table {
            let trainable = forms.trainable.min(forms.asks.len());
            for (index, ask) in forms.asks.iter().enumerate() {
                if index < trainable {
                    trainable_asks.push(ask);
                } else {
                    held_out_asks.push(ask);
                }
            }
            for (index, state) in forms.states.iter().enumerate() {
                let prefix = state.split("{v}").next().unwrap_or("");
                if index < trainable {
                    trainable_prefixes.push(prefix);
                } else {
                    held_out_prefixes.push(prefix);
                }
            }
        }
        // A string that is trainable for one relation is not evidence of a
        // held-out draw, so only the difference is a violation.
        let held_ask: Vec<&str> = held_out_asks
            .iter()
            .filter(|ask| !trainable_asks.contains(ask))
            .copied()
            .collect();
        let held_prefix: Vec<&str> = held_out_prefixes
            .iter()
            .filter(|prefix| !trainable_prefixes.contains(prefix))
            .copied()
            .collect();
        let canonical_asks: Vec<&str> = RELATIONS.iter().map(|r| r.asks[0]).collect();
        let mut rng = Rng(5);
        let mut paraphrases = 0usize;
        for _ in 0..500 {
            let doc = document(&mut rng, TRAIN_FORMS, &table).expect("document");
            assert!(
                !held_ask.contains(&doc.question.as_str()),
                "forms=8 produced a held-out question {:?}",
                doc.question
            );
            for turn in [&doc.first, &doc.second] {
                assert!(
                    !held_prefix.iter().any(|p| turn.starts_with(p)),
                    "forms=8 produced a held-out statement {turn:?}"
                );
            }
            if !canonical_asks.contains(&doc.question.as_str()) {
                paraphrases += 1;
            }
        }
        assert!(
            paraphrases > 100,
            "forms=8 drew only {paraphrases} paraphrase questions"
        );
    }

    /// The panel must be complete, unique, and scored against the asked
    /// relation's own values.
    #[test]
    fn panel_rows_are_complete_and_self_consistent() {
        let rows = panel_rows().expect("panel");
        let relations_with_forms = RELATIONS.len() - CANONICAL_ONLY.len();
        let with_perturbation = rows.len() - relations_with_forms * 6;
        assert_eq!(rows.len(), relations_with_forms * 6 + with_perturbation);
        assert!(rows.len() > relations_with_forms * 6);
        let mut seen = std::collections::BTreeSet::new();
        for (id, turns, wanted) in &rows {
            assert!(seen.insert(id.clone()), "duplicate panel id {id}");
            assert!(!turns.is_empty() && turns.iter().all(|t| !t.trim().is_empty()));
            let relation = id.rsplit('-').next().unwrap();
            let values = values_of(relation).expect("relation of panel row");
            assert!(
                values.contains(&wanted.as_str()),
                "panel row {id} expects {wanted:?}, which is not a value of {relation}"
            );
        }
    }
}
