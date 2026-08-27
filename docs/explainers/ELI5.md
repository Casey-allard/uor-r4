# How R⁴ is becoming a geometric language model, explained like you're five

> The friendly version. When you want the grown-up version, read
> [UNDERGRADUATE.md](UNDERGRADUATE.md); when you want the full rigor, read
> the [Geometric Intelligence Programme](../geometric_intelligence_programme.md).
> The transformerless table/graph design and proof documents describe the
> historical filing-cabinet system retained later in this explainer.

Right now, R⁴ has built a careful geometric address book. It can turn known
pieces of text into prime-number locations, attach a spinning state and a
checkable receipt, and look up stored routes with bounded work. It cannot yet
use that machinery alone to hold a real conversation. The old fluent path used
another language model; the new programme must replace that missing part.

The current plan treats a sentence like a path through a field of waves on a
sphere:

```text
your text
   -> reversible word pieces
   -> prime locations + spinning/harmonic state
   -> local route
   -> sentence route
   -> paragraph route
   -> conversation route
   -> bounded global route
   -> choose the next route
   -> turn that route back into text
```

The work is deliberately ordered. #961 made arbitrary text reversible and
kept all five route levels up to date. #952 found that its reusable summaries
forgot earlier order. #967 repaired that memory and produced different shapes
for the two possible next pieces, but its one-number ruler assigned both the
same distance in all six trials. It therefore kept the state without claiming
attention. #967 established only that the one-number ruler failed; it did not
establish a placement defect. #970 then kept a paired-H4 witness—both H4 shapes
for each candidate—combined them in their fixed order, and read an exact R4
heatmap instead of inventing another ruler. Its complete check covered all
14,400 ordered shape
pairs: 120 relative shapes became 45 heatmap kinds, including 480 typed-null
pairs that must abstain. On the 36 fixed candidate decisions, eight of 14
heatmap classes still meant conflicting answers and no construction rule made a
strict choice on any of the six validation histories. The result therefore
retains the paired shape and stops before readout or placement. This says only
that this exact heatmap cannot identify the frozen answer rule; it does not say
the geometry is useless.

The heatmap keeps exact golden-number arithmetic: `sin=+1` or `-1` with
`cos=0` maps to bit 1, while `sin=0` with `cos=+1` or `-1` maps to bit 0, and
the sign remains attached. Histories and candidate support are prepared before
answers are attached, and even/odd order is recomputed from each history. Zeta
phases, ordered prime groups, golden-radius moves, and typed chart conversion
remain structural ingredients; the project has not supplied a rule turning a
zeta/prime group into a golden-shell exponent.

#970 remains active until this corrected result lands through the protected
merge path, so #969 is still blocked. After that delivery, #969 must prove that
multiple channels, not a memorized answer, actually change what the system
attends to. Only after that does #953 build the bounded source-free next-word
loop; durable chat belongs to #962. Correct answers (#954) and multi-step
reasoning (#955) come later. A pretty route or a readable stored sentence does
not skip those steps.

The system is allowed to keep a shape because it stores or reconstructs the
route without letting that shape vote on the next word. A1Q/#969 must earn that
semantic vote with matched evidence first.

The preserved earlier system below had two robot helpers. One is a **filing
cabinet robot**, one is a **librarian with a magic map**. Their receipts and
measurements remain useful, but they are not the current talking engine.

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
│ (much less capable)│
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

## How the earlier systems were paired

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
rulers are just for show, but its receipts are real. Those older robots taught
the project how to store, locate, and witness data. The unfinished job is the
new one: make the complete geometric route decide what matters, produce the
next piece of language without an outside model, check whether the answer is
right, and only then reason. That is what the current programme means by
geometric intelligence.
