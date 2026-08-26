# How R⁴ is becoming a geometric language model, explained like you're five

> The friendly version. When you want the grown-up version, read
> [UNDERGRADUATE.md](UNDERGRADUATE.md); when you want the full rigor, read
> the [Geometric Causal Decoder Roadmap](../geometric_causal_decoder_plan.md).
> The transformerless table/graph design and proof documents describe the
> historical filing-cabinet system retained later in this explainer.

The project learned something important: the filing cabinet can remember which
words often follow other words, and the map can remember useful context, but
neither one knows enough grammar to talk coherently by itself.

The new plan joins the working pieces:

```text
your words + remembered map points
              |
              v
     geometric next-word thinker
              |
              v
      language layers on a CPU
        powered by uor-matmul
              |
              v
          the next word
              |
              +--> remember the conversation
```

We first let the existing local language brain show that the plumbing can talk.
Then we replace one “who should this word listen to?” layer with geometry. If
real geometry behaves no better than scrambled geometry, we stop and redesign
it. Only after one layer works do we replace the rest.

The historical system below had two robot helpers. One is a **filing cabinet
robot**, one is a **librarian with a magic map**. Here's how they work.

## The big picture

```
        You ask them something about stories
                        │
        ┌───────────────┴───────────────┐
        ▼                               ▼
┌───────────────────┐           ┌───────────────────┐
│  TRANSFORMERLESS  │           │        R4         │
│  the filing       │           │  the librarian    │
│  cabinet robot    │           │  with a map       │
│                   │           │                   │
│  "What word       │           │  "Where does your │
│   comes NEXT?"    │           │   question LIVE?" │
└───────────────────┘           └───────────────────┘
        │                               │
        └───────────┬───────────────────┘
                    ▼
        BOTH hand you a RECEIPT
        proving how they got the answer
```

## How the filing cabinet robot was born

There was a giant story-brain. It was super smart but SO heavy it needed a
forklift (60 MB of numbers, and every guess needed millions of calculator
multiplies). So we took a photocopier and shrunk it into a little card file:

```
┌────────────────────┐
│ GIANT STORY-BRAIN  │
│ 60 MB, needs       │
│ a forklift         │
└─────────┬──────────┘
          │ photocopy ONCE, very carefully
          ▼
┌────────────────────┐
│ tiny card file     │
│ 2 MB, fits in      │
│ your pocket        │
│ (almost as smart!) │
└────────────────────┘
```

## How the robot guesses the next word

**Step 1: it draws a treasure map.** Every "what happened so far" gets
turned into 288 yes/no questions:

```
"Is it about a person?         YES  ▶ 1
 Is it about something old?    no   ▶ 0
 Is someone going somewhere?   YES  ▶ 1
 ... 285 more tiny questions ..."

 your map:    1 0 1 1 0 0 1 0 ...
 a friend's:  1 0 1 1 0 0 1 1 ...
              └─────────┘
 Same first 7 answers? You're in the SAME
 neighborhood up to question 7.
 More matching answers = closer neighbors!
```

**Step 2: it opens the right drawer.** The cabinet has drawers, from
"everything" down to "exactly like this":

```
 drawer 0: "ANY story ever"     → time: 9000, dog: 3000, cat: 2000
 drawer 1: "sort of like this"  → time: 800,  dog: 469
 drawer 2: "kind of like this"  → time: 90,   dog: 12
 drawer 3: "a lot like this"    → time: 8
 drawer 4: "EXACTLY like this"  → dog: 1

 RULE: open the DEEPEST drawer that isn't empty.
 Count the tally marks. The winner is the answer!
 Empty drawer? Back up one and try again.
```

**Step 3: it uses only kid tools — no calculator!**

```
 the robot's toolbox:
   ✅ add      ✅ shift     ✅ xor
   ✅ compare  ✅ read cards

   ❌ multiply — there's not even a BUTTON for it

 and it counts every tool it used:
   adds: 59,598   xors: 36,864   multiplies: ZERO
 (it can count to zero without using a calculator)
```

## How the librarian works

The librarian takes every question and puts a dot on a big round map:

```
                  N
            ┌───────────┐
         W  │    •you   │   E
            │  are here │
            └───────────┘
                  S

  "This question lives in Window 2 —
   the Duality & Polarity neighborhood!"
```

To place the dot, the librarian uses magic rulers:

- **512 special numbers** called zeta zeros (a secret ruler only librarians
  have)
- **prime numbers** stuck on every word, like name tags
- a **spinning-top dance** (called Hopf coordinates) that tells which way
  the dot is wobbling

Then the librarian either picks words from its own neighborhood memory, or
hands the coordinates to the transformerless storyteller to write the
answer.

## The most important part: the RECEIPT

Both robots, every single time, staple a receipt to their answer:

```
┌──────────────────────────────────┐
│  ANSWER: "time"                  │
│                                  │
│  how I got it:                   │
│   • your map ended at drawer 1   │
│   • "dog" had 469 tallies        │
│   • multiplies used: 0           │
│                                  │
│  fingerprint: 09c5017a… ✓        │
│  CHECK MY WORK — it all matches  │
└──────────────────────────────────┘
```

If anyone secretly changes a card or a map, the receipt stops matching, and
everyone can see it. Nobody can cheat.

## How they play together now

```
 you ask a question
      │
      ▼
 ┌─────────┐  "which neighborhood?"  ┌──────────┐
 │   R4    │────────────────────────►│ the map  │
 │librarian│                         └──────────┘
 └─────────┘
      │
      ▼
 ┌──────────────┐  "what comes next   ┌──────────┐
 │transformerless│ ◄───────────────── │ drawers  │
 │  cabinet     │   and prove it?"    └──────────┘
 └──────────────┘
      │
      ▼
 ONE receipt, both answers, wax-sealed.
```

And you can teach the cabinet new stories: read it one, new tally marks
appear in the drawers, and the wax seal changes so everyone can see it
learned something. You can even take a tally mark out — and the receipt
proves exactly which one was removed.

## The moral of the story

The filing cabinet robot is **honest but humble** — it's not the smartest
robot, but everything it says fits in your pocket and comes with proof. The
librarian is a **dreamer** — its maps are beautiful and some of its magic
rulers are just for show, but its receipts are real and it knows where
everything lives. Together: one tells you *where you are*, the other tells
you *what comes next*, and both always, always show their work. The end.
