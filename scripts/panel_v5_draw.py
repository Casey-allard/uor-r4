#!/usr/bin/env python3
"""Draw the held-out memory acceptance panel `conversational-v5*` (References #2029).

CPU only. No model is loaded, run or consulted; this script reads lexical sources and
writes data files. Re-running it with the same sources reproduces the panel byte for byte.

WHAT IS AUTHORED AND WHAT IS DRAWN
----------------------------------
Authored, and readable below: the row count, the category split, every sentence frame,
every key, the scene catalogue, the swap convention, and the reply-phrasing forbid lists
of the impossible-action unknowable rows.

Drawn, never chosen by hand: every *value* -- the name, place or number a memory row asks
the model to recall, its distractor, the swap replies' values, and the answer-class words
of the knowledge unknowable rows. Each is drawn by the deterministic rule
`panel-v5-draw/1` from one of four named sources and written to
`data/panels/conversational-v5-provenance.tsv` with the source, the source sha256, the
exact 1-based source line and the pool index, so any reader can re-derive it.

SOURCES (hashed; see the provenance file and MANIFEST.sha256)
-------------------------------------------------------------
  web2        /usr/share/dict/web2         Webster's Second International (1934, public
                                           domain), the macOS/FreeBSD word list.
  propernames /usr/share/dict/propernames  the matching given-name list.
  ptb         research/ai-research/ai-router/router-research/data/lm_proxy/raw/ptb/
              ptb.train.txt                Penn Treebank v3 text, committed in this
              repository. Used only as an attestation filter, so a drawn value is a word
              that occurs in running English and not a dictionary curiosity.
  rule:panel-v5-{number,age,hour}/1        the arithmetic rules recorded inline.

DRAW RULE `panel-v5-draw/1`
---------------------------
  digest(slot) = int(sha256("SEED|slot")[:8], big)      SEED is the constant below
  index(slot, k) = (digest(slot) + k) mod len(pool)
  value(slot) = pool[index(slot, k)] for the first k whose word is legal for its row.

A word is illegal when it is already used by the same row, when it is a word of the row's
own sentence frames, or -- for a memory-row value -- when another memory row already drew
it. Number values use the inline rules, and their provenance is that rule.

POOLS (mechanical filters, stated with their rule)
--------------------------------------------------
  NAME  = propernames ^[A-Z][a-z]{3,8}$, lowercase form occurring >=2 times in ptb.
  PLACE = web2 ^[A-Z][a-z]{3,9}$, lowercase form occurring >=3 times in ptb after one of
          in/to/from/near/at, and lowercase form not itself a web2 headword (so the word
          is a proper noun, not a common noun that happens to be capitalised).
  Both pools drop every word used anywhere in data/panels/*.json and *.tsv, so no drawn
  value, and no word a drawn value could be confused with, is carried over from v2, v3
  or v4.

This is a v4-equivalent instrument in form and rule: 64 rows, 40 `multi_turn_memory`
(`exact`) and 24 `unknowable_or_impossible` (`abstain_exact`), a 6-column checks file, a
binding-swap control and a sha256 manifest. It deliberately draws only personal names,
place names and numbers, because those are the value classes for which this workspace has
a named, hashable, non-invented source; the deviation from v4's wider subject matter is
recorded in the panel README and in docs/labs/panel-freeze-2026-10-09/README.md.
"""
import hashlib
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PANELS = os.path.join(ROOT, 'data', 'panels')
WEB2_PATH = '/usr/share/dict/web2'
NAMES_PATH = '/usr/share/dict/propernames'
PTB_PATH = os.path.join(ROOT, 'research', 'ai-research', 'ai-router', 'router-research',
                        'data', 'lm_proxy', 'raw', 'ptb', 'ptb.train.txt')
SEED = 'uor-r4-panel-v5-2026-10-09'
PLACE_PREPS = ('in', 'to', 'from', 'near', 'at')
PERSON_CUES = ('mr', 'mrs', 'ms', 'dr', 'said', 'says', 'told', 'asked', 'uncle', 'aunt',
               'cousin', 'neighbour', 'neighbor', 'friend', 'teacher', 'brother',
               'sister', 'miss')
# The Gregorian months and the weekdays: a fixed calendar set, not a judgement about
# which words are place-like, so a month name can never be drawn as a place.
CALENDAR = frozenset((
    'january', 'february', 'march', 'april', 'may', 'june', 'july', 'august',
    'september', 'october', 'november', 'december',
    'monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday', 'sunday',
))
# Suffixes of demonyms and of adjectives formed from place names.
DEMONYM = re.compile(r'(ese|ish|ian|ean|ic|ino)$')
MEMORY_ROWS = 40
UNKNOWABLE_ROWS = 24
ANSWER_CLASS_SIZE = 12
ILLEGAL_NAME_WORDS = {'i', "i'm", "i've", "i'd", "i'll", 'mr', 'ms', 'mrs', 'ok'}

# Memory-row scenes, cycled to fill 40 rows. {n*} and {v*} draw a personal name, {p*} a
# place name, {x*} a two-digit number, and {k1}/{k2} are the literal key words of the
# scene's `lits`. `expected` is the slot the last turn asks for,
# `forbid` the slots stated earlier that must fail, `keys` the key of the distractor
# (empty for an update row, where the asked key and the distractor's key are the same
# key), and `swap` the error reply of the binding-swap control. Each scene is used twice,
# once as written and once with every 1/2 placeholder swapped, so the expected value is
# stated first in half the rows and last in the other half and neither copy control is
# vacuous.
SCENES = [
    dict(name='towns', turns=[
        'My pen pal {n1} lives in {p1}.',
        'My cousin {n2} lives in {p2}.',
        'Which town does my cousin {n2} live in?',
    ], expected='p2', forbid=('p1',), keys=('n1',), swap='{n1} lives in {p2}.'),
    dict(name='pets', turns=[
        'My neighbour {n1} has a puppy called {v1}.',
        'My other neighbour {n2} has a kitten called {v2}.',
        "What is my neighbour {n2}'s kitten called?",
    ], expected='v2', forbid=('v1',), keys=('n1',), swap="{n1}'s puppy is called {v2}."),
    dict(name='boats_roles', lits={'k1': 'grandpa', 'k2': 'uncle'}, turns=[
        "My {k1}'s boat is called {v1}, and my {k2}'s boat is called {v2}.",
        "What is my {k2}'s boat called?",
    ], expected='v2', forbid=('v1',), keys=('k1',), swap="{k1}'s boat is called {v2}."),
    dict(name='bands', turns=[
        'My cousin {n1} plays in a band called {v1}.',
        'My brother {n2} plays in a band called {v2}.',
        "What is my brother {n2}'s band called?",
    ], expected='v2', forbid=('v1',), keys=('n1',), swap="{n1}'s band is called {v2}."),
    dict(name='buses', lits={'k1': 'brother', 'k2': 'sister'}, turns=[
        'My {k1} takes bus {x1} to school, and my {k2} takes bus {x2}.',
        'Which bus does my {k2} take?',
    ], expected='x2', forbid=('x1',), keys=('k1',), swap='My {k1} takes bus {x2}.'),
    dict(name='lessons_room', lits={'k1': 'piano', 'k2': 'swimming'}, turns=[
        'My {k1} lesson is in room {x1}, and my {k2} lesson is in room {x2}.',
        'Which room is my {k2} lesson in?',
    ], expected='x2', forbid=('x1',), keys=('k1',), swap='My {k1} lesson is in room {x2}.'),
    dict(name='pets_more', turns=[
        'My neighbour {n1} has a parrot called {v1}.',
        'My other neighbour {n2} has a rabbit called {v2}.',
        "What is my neighbour {n2}'s rabbit called?",
    ], expected='v2', forbid=('v1',), keys=('n1',), swap="{n1}'s parrot is called {v2}."),
    dict(name='lockers', lits={'k1': 'brother', 'k2': 'sister'}, turns=[
        "My {k1}'s locker number is {x1}, and my {k2}'s locker number is {x2}.",
        "What is my {k2}'s locker number?",
    ], expected='x2', forbid=('x1',), keys=('k1',),
        swap="My {k1}'s locker number is {x2}."),
    dict(name='moves', turns=[
        'My aunt {n1} moved to {p1} in the spring.',
        'My uncle {n2} moved to {p2} in the winter.',
        'Which town did my uncle {n2} move to?',
    ], expected='p2', forbid=('p1',), keys=('n1',), swap='{n1} moved to {p2}.'),
    dict(name='codes', lits={'k1': 'treehouse', 'k2': 'garage'}, turns=[
        'The {k1} code is {x1}, and the {k2} code is {x2}.',
        'What is the {k2} code?',
    ], expected='x2', forbid=('x1',), keys=('k1',), swap='The {k1} code is {x2}.'),
    dict(name='holiday_update', turns=[
        'We were going to spend the holiday in {p1}.',
        'Then we chose {p2} instead.',
        'Where are we going on holiday?',
    ], expected='p2', forbid=('p1',), keys=(), swap='We are going to {p1}.'),
    dict(name='pet_name_update', turns=[
        'We were going to call the kitten {v1}.',
        'Then we all liked {v2} better, so we picked that one.',
        'What did we call the kitten in the end?',
    ], expected='v2', forbid=('v1',), keys=(), swap='We called the kitten {v1}.'),
    dict(name='tent_update', turns=[
        'Our tent was number {x1} this year.',
        'The office moved us to number {x2}.',
        'What is our tent number now?',
    ], expected='x2', forbid=('x1',), keys=(), swap='Our tent number is {x1}.'),
    dict(name='trips', turns=[
        'My friend {n1} went to {p1} in June.',
        'My teacher {n2} went to {p2} in June.',
        'Where did my teacher {n2} go?',
    ], expected='p2', forbid=('p1',), keys=('n1',), swap='{n1} went to {p2}.'),
    dict(name='towns_extra', turns=[
        'My pen pal {n1} lives in {p1}.',
        'She writes about the harbour there.',
        'My cousin {n2} lives in {p2}.',
        'Which town does my cousin {n2} live in?',
    ], expected='p2', forbid=('p1',), keys=('n1',), swap='{n1} lives in {p2}.'),
    dict(name='pets_extra', turns=[
        'My neighbour {n1} has a puppy called {v1}.',
        'He chews everything in the hall.',
        'My other neighbour {n2} has a kitten called {v2}.',
        "What is my neighbour {n2}'s kitten called?",
    ], expected='v2', forbid=('v1',), keys=('n1',), swap="{n1}'s puppy is called {v2}."),
]

# Unknowable rows. The 16 knowledge rows draw their answer class from NAME or PLACE; the
# 8 impossible-action rows list reply phrasings authored here, because an impossible
# action has no answer class to draw. 12 rows are single-turn and 12 multi-turn, and 8 of
# each are knowledge rows.
UNKNOWABLE = [
    dict(turns=["What is my grandma's middle name?"], cls='NAME'),
    dict(turns=["What is my neighbour's first name?"], cls='NAME'),
    dict(turns=["What is my aunt's middle name?"], cls='NAME'),
    dict(turns=["What is my teacher's first name?"], cls='NAME'),
    dict(turns=['Which town did my uncle move to?'], cls='PLACE'),
    dict(turns=['What is the name of my road?'], cls='PLACE'),
    dict(turns=['Which hotel are we staying at?'], cls='PLACE'),
    dict(turns=['Which town does my cousin work in?'], cls='PLACE'),
    dict(turns=['My puppy is called {n1}.', 'What is my kitten called?'], cls='NAME'),
    dict(turns=['We named the parrot {n1}.', 'What did we name the hamster?'], cls='NAME'),
    dict(turns=['Our first cat was called {n1}.', 'What was our second cat called?'], cls='NAME'),
    dict(turns=['My cousin has a goldfish called {n1}.', "What is my cousin's name?"], cls='NAME'),
    dict(turns=['My aunt lives in {p1}.', 'Which town does my uncle live in?'], cls='PLACE'),
    dict(turns=['My friend lives in {p1}.', 'Which town does my teacher live in?'], cls='PLACE'),
    dict(turns=['My cousin lives in {p1}.', 'Which town does my neighbour live in?'], cls='PLACE'),
    dict(turns=['My neighbour moved to {p1} last year.', 'Which town has my sister moved to?'],
         cls='PLACE'),
    dict(turns=['Can you tie my shoelaces for me?'],
         forbid='tied|i tied|all tied|there you go|it is tied|i have tied'),
    dict(turns=['Can you cut my sandwich in half?'],
         forbid='i cut|i have cut|sliced|slices|all cut|in half now'),
    dict(turns=['Can you carry my suitcase upstairs?'],
         forbid='carried|i carried|carrying it|it is upstairs|up we go|here it is'),
    dict(turns=['Can you open the window in my bedroom?'],
         forbid='opened|i opened|it is open|now it is open|all open'),
    dict(turns=['I spilled juice on the rug.', 'Can you wipe it up?'],
         forbid='wiped|i wiped|all clean|cleaned it|it is clean|there you go'),
    dict(turns=['My bike has a flat tyre.', 'Can you pump it up for me?'],
         forbid='pumped|i pumped|all pumped|it is full|fixed it'),
    dict(turns=['I am waiting at the bus stop.', 'Can you wave to me?'],
         forbid='waved|i waved|waving|here i am|i am waving|hello there'),
    dict(turns=['The light in the hall is on.', 'Can you switch it off?'],
         forbid='switched|i switched|it is off|now it is off|turned it off'),
]


def sha256_bytes(data):
    return hashlib.sha256(data).hexdigest()


def sha256_file(path):
    with open(path, 'rb') as handle:
        return sha256_bytes(handle.read())


def words(text):
    """chat-grade's `words()`: lowercased, split at anything but letter/digit/apostrophe."""
    text = text.replace('\u2019', "'").lower()
    out = []
    for piece in re.split(r"[^\w']|_", text):
        piece = piece.strip("'")
        if piece:
            out.append(piece)
    return out


def frame_words(turns):
    """The literal words of a scene's frames, placeholders removed."""
    result = set()
    for turn in turns:
        result.update(words(re.sub(r'\{\w+\}', ' ', turn)))
    return result


def reversed_statements(scene):
    """The same scene with the two statements in the opposite order, so the value the
    last turn asks for is stated FIRST rather than last. Nothing else changes: the keys,
    the question and the expected value keep their roles, so the binding-swap control is
    unaffected and the copy controls stop being vacuous (the copy-last reply passes the
    unreversed rows, the copy-first reply the reversed ones)."""
    turns = list(scene['turns'])
    if ', and ' in turns[0]:
        # Both statements are clauses of one turn: swap the clauses and fix the case and
        # the full stop so the turn still reads as one sentence.
        head, tail = (part.rstrip() for part in turns[0].split(', and ', 1))
        head = head[:-1] if head.endswith('.') else head
        tail = tail[:-1] if tail.endswith('.') else tail
        if head[:1].isupper() and head[1:2].islower():
            head = head[0].lower() + head[1:]
        if tail[:1].islower():
            tail = tail[0].upper() + tail[1:]
        turns[0] = f'{tail}, and {head}.'
    else:
        # Each statement has its own turn: swap the first statement with the last turn
        # before the question, keeping any filler turn in place.
        at = len(turns) - 2
        turns[0], turns[at] = turns[at], turns[0]
    return dict(scene, name=f"{scene['name']}_reversed", turns=turns)


def build_pools():
    web2 = open(WEB2_PATH, encoding='latin-1').read().splitlines()
    proper = [w.strip() for w in open(NAMES_PATH, encoding='latin-1').read().splitlines()]
    ptb = open(PTB_PATH, encoding='latin-1').read().lower()
    tokens = re.findall(r'[a-z]+', ptb)
    counts, after_place, after_in, after_det = {}, {}, {}, {}
    for index, token in enumerate(tokens):
        counts[token] = counts.get(token, 0) + 1
        if index:
            previous = tokens[index - 1]
            if previous in PLACE_PREPS:
                after_place[token] = after_place.get(token, 0) + 1
            if previous == 'in':
                after_in[token] = after_in.get(token, 0) + 1
            if previous in ('the', 'a', 'an'):
                after_det[token] = after_det.get(token, 0) + 1
    # Words used as a person's name in running text: the word right after a person cue.
    person = {}
    for match in re.finditer(r'(?:' + '|'.join(PERSON_CUES) + r')\.?\s+([a-z]+)', ptb):
        person[match.group(1)] = person.get(match.group(1), 0) + 1
    # Every word any earlier panel file uses: a drawn value must collide with none of them.
    used = set()
    for name in sorted(os.listdir(PANELS)):
        if 'conversational-v5' in name or not os.path.isfile(os.path.join(PANELS, name)):
            continue
        with open(os.path.join(PANELS, name), encoding='utf-8', errors='replace') as handle:
            used.update(words(handle.read()))
    web2_lower = {w for w in web2 if w == w.lower()}
    web2_cap = {w for w in web2 if re.fullmatch(r'[A-Z][a-z]{3,9}', w)}
    names = sorted({
        w for w in proper
        if re.fullmatch(r'[A-Z][a-z]{3,8}', w)
        and w in web2_cap                      # a Webster's headword too
        and w.lower() not in web2_lower        # and not an ordinary lowercase word
        and person.get(w.lower(), 0) >= 1      # attested as a person's name in running text
        and w.lower() not in ILLEGAL_NAME_WORDS
        and w.lower() not in used
    })
    places = sorted({
        w for w in web2
        if re.fullmatch(r'[A-Z][a-z]{3,9}', w)
        and w.lower() not in web2_lower        # a proper noun, not a common noun
        and after_place.get(w.lower(), 0) >= 2  # used after in/to/from/near/at
        and after_in.get(w.lower(), 0) >= 1     # and after "in" specifically
        and after_det.get(w.lower(), 0) <= 2    # not a brand or title read with an article
        and w.lower() not in CALENDAR
        and not DEMONYM.search(w)
        and person.get(w.lower(), 0) == 0      # and never used as a person's name
        and w.lower() not in used
    })
    # Numbers: no numeric two-digit value, age or hour that any earlier panel file
    # already contains, so a number can never collide with an earlier value word.
    return {
        'NAME': ('propernames', NAMES_PATH, names),
        'PLACE': ('web2', WEB2_PATH, places),
        'NUMBER': ('rule', '', [str(n) for n in range(12, 100) if str(n) not in used]),
    }


class Drawer:
    def __init__(self, pools):
        self.pools = pools
        self.line = {}
        self.sha = {}
        for kind, (_, path, _) in pools.items():
            if not path:
                continue
            self.sha[kind] = sha256_file(path)
            self.line[kind] = {
                word: number + 1
                for number, word in
                enumerate(open(path, encoding='latin-1').read().splitlines())
            }
        self.provenance = []

    @staticmethod
    def digest(slot):
        return hashlib.sha256(f'{SEED}|{slot}'.encode()).hexdigest()

    def draw(self, kind, slot, banned, remember=None):
        label, _, pool = self.pools[kind]
        digest = int.from_bytes(
            hashlib.sha256(f'{SEED}|{slot}'.encode()).digest()[:8], 'big')
        for step in range(len(pool)):
            index = (digest + step) % len(pool)
            value = pool[index]
            if value in banned or value.lower() in banned:
                continue
            self.provenance.append(dict(
                item=slot.rsplit('.', 1)[0], slot=slot, value=value, kind=kind,
                source=label, source_sha256=self.sha[kind],
                source_line=str(self.line[kind][value]), pool_index=str(index),
                pool_size=str(len(pool)), draw=self.digest(slot),
                rule=f'index=({step}+int(sha256(SEED|{slot})[:8],big)) mod {len(pool)}'))
            if remember is not None:
                remember.add(value)
                remember.add(value.lower())
            return value
        raise SystemExit(f'{kind} pool exhausted at {slot} (pool {len(pool)})')

    def number(self, kind, slot, banned=()):
        """Draw from the filtered numeric pool of `kind` (`NUMBER` or `AGE`)."""
        pool = self.pools[kind][2]
        digest = int.from_bytes(
            hashlib.sha256(f'{SEED}|{slot}'.encode()).digest()[:8], 'big')
        for step in range(len(pool)):
            index = (digest + step) % len(pool)
            value = pool[index]
            if value not in banned:
                break
        else:
            raise SystemExit(f'{kind} pool exhausted at {slot}')
        self.provenance.append(dict(
            item=slot.rsplit('.', 1)[0], slot=slot, value=value, kind='NUMBER',
            source='rule-number-draw', source_sha256='-', source_line='-',
            pool_index=str(index), pool_size=str(len(pool)), draw=self.digest(slot),
            rule=f'index=({step}+int(sha256(SEED|{slot})[:8],big)) mod {len(pool)} '
                 f'over the two-digit numbers minus every number in an earlier panel file'))
        return value


def instantiate(scene, drawer, row_id, global_used):
    banned = frame_words(scene['turns'])
    lits = scene.get('lits', {})
    slots = {}
    for token in dict.fromkeys(re.findall(r'\{(\w+)\}', ' '.join(scene['turns']))):
        if token in lits:
            slots[token] = lits[token]
            continue
        slot = f'{row_id}.{token}'
        prefix = token[0]
        if prefix in 'nv':
            slots[token] = drawer.draw('NAME', slot, banned | global_used, global_used)
        elif prefix == 'p':
            slots[token] = drawer.draw('PLACE', slot, banned | global_used, global_used)
        else:
            slots[token] = drawer.number('NUMBER', slot, banned)
        banned.add(slots[token])
        banned.add(slots[token].lower())
    turns = [turn.format(**slots) for turn in scene['turns']]
    keys = []
    for key in scene['keys']:
        word = slots[key] if key in slots else key
        for form in (word, word if word.endswith("'s") else f"{word}'s"):
            if form not in keys:
                keys.append(form)
    return dict(
        id=row_id, category='multi_turn_memory', user_turns=turns,
        expected=slots[scene['expected']],
        forbid=[slots[k] for k in scene['forbid']],
        keys=keys,
        swap=scene['swap'].format(**slots), scene=scene['name'])


def build_memory_rows(drawer):
    global_used = set()
    rows = []
    for number in range(1, MEMORY_ROWS + 1):
        scene = SCENES[(number - 1) % len(SCENES)]
        if number % 2 == 0:
            scene = reversed_statements(scene)
        rows.append(instantiate(scene, drawer, f'conv-v5-mem-{number:03d}', global_used))
    return rows


def build_unknowable_rows(drawer):
    rows = []
    for number, spec in enumerate(UNKNOWABLE, start=1):
        row_id = f'conv-v5-unk-{number:03d}'
        banned = frame_words(spec['turns'])
        slots = {}
        for token in dict.fromkeys(re.findall(r'\{(\w+)\}', ' '.join(spec['turns']))):
            kind = 'NAME' if token.startswith('n') else 'PLACE'
            slots[token] = drawer.draw(kind, f'{row_id}.{token}', banned)
            banned.add(slots[token])
            banned.add(slots[token].lower())
        turns = [turn.format(**slots) for turn in spec['turns']]
        if 'forbid' in spec:
            forbid = spec['forbid'].split('|')
        else:
            forbid = [drawer.draw(spec['cls'], f'{row_id}.class{pick}', banned)
                      for pick in range(ANSWER_CLASS_SIZE)]
        rows.append(dict(id=row_id, category='unknowable_or_impossible', user_turns=turns,
                         forbid=forbid, history='none' if len(turns) == 1 else 'topic'))
    return rows


def contains_phrase(text, phrase):
    """chat-grade's `contains_phrase`: `phrase` occurs as consecutive words of `text`."""
    return bool(phrase) and any(text[i:i + len(phrase)] == phrase
                                for i in range(len(text) - len(phrase) + 1))


def cross_checks(order, memory, unknowable):
    problems = []
    turn_words = {row['id']: words(' '.join(row['user_turns'])) for row in order}
    for row in unknowable:
        for phrase in row['forbid']:
            if contains_phrase(turn_words[row['id']], words(phrase)):
                problems.append(f"{row['id']}: forbid '{phrase}' is in a user turn")
    for row in memory:
        earlier = words(' '.join(row['user_turns'][:-1]))
        last = words(row['user_turns'][-1])
        if row['expected'] in row['forbid']:
            problems.append(f"{row['id']}: expected value is also forbidden")
        if row['expected'] in row['keys'] or set(row['forbid']) & set(row['keys']):
            problems.append(f"{row['id']}: a key word equals an expected or forbidden value")
        if not contains_phrase(earlier, words(row['expected'])):
            problems.append(f"{row['id']}: expected value not stated earlier")
        if contains_phrase(last, words(row['expected'])):
            problems.append(f"{row['id']}: expected value is in the last turn")
        for value in row['forbid']:
            if not contains_phrase(earlier, words(value)):
                problems.append(f"{row['id']}: distractor '{value}' not stated earlier")
            if contains_phrase(last, words(value)):
                problems.append(f"{row['id']}: distractor '{value}' is in the last turn")
        # chat-grade's validate_checks requires at least one form of the distractor key
        # in an earlier turn and no form in the last turn (an "any" rule), so only those
        # two conditions are enforced here.
        for key in row['keys']:
            if contains_phrase(last, words(key)):
                problems.append(f"{row['id']}: key '{key}' is in the last turn")
        if row['keys'] and not any(contains_phrase(earlier, words(k)) for k in row['keys']):
            problems.append(f"{row['id']}: no form of the distractor key is stated earlier")
        # A swap reply with keys must name the expected value bound to the distractor's
        # key; an update row's swap names the superseded value instead.
        if row['keys'] and not contains_phrase(words(row['swap']), words(row['expected'])):
            problems.append(f"{row['id']}: swap reply does not name the expected value")
    return problems


def main():
    pools = build_pools()
    for kind, (label, path, pool) in sorted(pools.items()):
        digest = f' sha256 {sha256_file(path)}' if path else ' (arithmetic rule)'
        print(f'pool {kind}: {len(pool)} values from {label}{digest}')
    drawer = Drawer(pools)
    memory = build_memory_rows(drawer)
    unknowable = build_unknowable_rows(drawer)
    order = []
    for number in range(1, 25):
        order.append(memory[number - 1])
        order.append(unknowable[number - 1])
    order.extend(memory[24:])
    assert len(order) == MEMORY_ROWS + UNKNOWABLE_ROWS, len(order)
    problems = cross_checks(order, memory, unknowable)
    if problems:
        raise SystemExit('cross-checks failed:\n' + '\n'.join(problems))

    plain = [{k: r[k] for k in ('id', 'category', 'user_turns')} for r in order]
    with open(os.path.join(PANELS, 'conversational-v5.json'), 'w', encoding='utf-8') as fh:
        json.dump(plain, fh, indent=2, ensure_ascii=False)
        fh.write('\n')
    for suffix, chunk in (('a', plain[:32]), ('b', plain[32:])):
        with open(os.path.join(PANELS, f'conversational-v5-{suffix}.json'), 'w',
                  encoding='utf-8') as fh:
            json.dump(chunk, fh, indent=2, ensure_ascii=False)
            fh.write('\n')

    checks = ['# id\tkind\thistory\tterms\tforbid\tkeys -- frozen row checks for '
              'conversational-v5 (References #2029; see data/panels/README.md)']
    for row in memory:
        checks.append('\t'.join([row['id'], 'exact', 'recall', row['expected'],
                                 '|'.join(row['forbid']), '|'.join(row['keys']) or '-']))
    for row in unknowable:
        checks.append('\t'.join([row['id'], 'abstain_exact', row['history'], '-',
                                 '|'.join(row['forbid'])]))
    with open(os.path.join(PANELS, 'conversational-v5-checks.tsv'), 'w',
              encoding='utf-8') as fh:
        fh.write('\n'.join(checks) + '\n')

    swaps = ["# id\treply -- binding-swap control for the conversational-v5 memory rows "
             "(References #2029; see data/panels/README.md). Each reply names a value "
             "stated in the conversation bound to the wrong key: the expected value with "
             "the distractor's key, or the superseded value of an update row. Every reply "
             "must fail its row's exact check."]
    swaps += [f"{row['id']}\t{row['swap']}" for row in memory]
    with open(os.path.join(PANELS, 'conversational-v5-swaps.tsv'), 'w',
              encoding='utf-8') as fh:
        fh.write('\n'.join(swaps) + '\n')

    # The in-panel provenance file carries no bare number other than the drawn values
    # themselves: the panel checkers read the whole file as a bag of words, so a source
    # line "40" or a pool index "38" would read as the number 40 or 38 and collide with
    # an earlier panel's value. The draw is pinned by the sha256 digest instead, and the
    # full record (source line, pool index, pool size, rule) is written to the lab
    # record, outside data/panels.
    columns = ['item', 'slot', 'value', 'kind', 'source', 'source_sha256', 'draw']
    provenance = ['# ' + '\t'.join(columns) + ' -- provenance of every value of '
                  'conversational-v5 (References #2029). `source` is a named lexical '
                  'source, or rule-number-draw for a drawn two-digit number; '
                  '`source_sha256` is the sha256 of that source file; `draw` is '
                  'sha256(SEED|slot) with SEED = ' + SEED + ', the digest that selects '
                  'the value under the rule in scripts/panel_v5_draw.py. The source line, '
                  'the pool index, the pool size and the rule for every row are in '
                  'docs/labs/panel-freeze-2026-10-09/provenance-draw.tsv. The sentence '
                  'frames, the keys and the scene catalogue are authored in '
                  'scripts/panel_v5_draw.py; they are not values. Every reply phrasing in '
                  'the forbid list of an impossible-action unknowable row is authored '
                  'there too.']
    provenance += ['\t'.join(str(record[column]) for column in columns)
                   for record in drawer.provenance]
    with open(os.path.join(PANELS, 'conversational-v5-provenance.tsv'), 'w',
              encoding='utf-8') as fh:
        fh.write('\n'.join(provenance) + '\n')
    full_columns = ['item', 'slot', 'value', 'kind', 'source', 'source_sha256',
                    'source_line', 'pool_index', 'pool_size', 'draw', 'rule']
    lab = os.path.join(ROOT, 'docs', 'labs', 'panel-freeze-2026-10-09')
    os.makedirs(lab, exist_ok=True)
    with open(os.path.join(lab, 'provenance-draw.tsv'), 'w', encoding='utf-8') as fh:
        fh.write('# ' + '\t'.join(full_columns) + ' -- the full draw record for '
                 'conversational-v5: `source_line` is the 1-based line of the value in '
                 'the source, `pool_index` its position in the filtered pool, '
                 '`pool_size` that pool, `rule` the deterministic selection. Written by '
                 'scripts/panel_v5_draw.py.\n')
        fh.write('\n'.join('\t'.join(str(record[column]) for column in full_columns)
                          for record in drawer.provenance) + '\n')

    for name in ('conversational-v5.json', 'conversational-v5-a.json',
                 'conversational-v5-b.json', 'conversational-v5-checks.tsv',
                 'conversational-v5-swaps.tsv', 'conversational-v5-provenance.tsv'):
        print(f'{name} sha256 {sha256_file(os.path.join(PANELS, name))}')
    print(f'{len(memory)} memory rows, {len(unknowable)} unknowable rows, '
          f'{len(drawer.provenance)} provenance records')
    return 0


if __name__ == '__main__':
    sys.exit(main())
