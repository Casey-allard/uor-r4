#!/usr/bin/env python3
"""Check that `conversational-v5` shares no value word or proper name with an earlier panel.

References #2029. The v4 checker (`scripts/check_panel_v4_novelty.py`) is unchanged and
still passes; this is its v5 counterpart, with the direction that the v4 checker enforces
implicitly (v4's values must not occur in v5's files) also checked explicitly: v5's value
words and proper names are checked against every earlier panel file, and every earlier
panel's value words and proper names are checked against v5's files.

Reads `data/panels/conversational-v5.json` and `-checks.tsv` and every earlier panel file
(`data/panels/*` and `~/uor-r4-local/ladder/panel/*.json` when present, v5 files
excluded). Words follow chat-grade's `words()` rule: lowercased, split at anything that
is not a letter, digit or apostrophe, apostrophes trimmed from the ends.

Checked, word by word:
  * values: every spelling in the `terms` (expected) and `forbid` (distractor) columns of
    the v5 `exact` (memory) rows, in both directions;
  * proper names: capitalised words of the v5 user turns that are inside a sentence, or
    begin one, never occur lowercase in a v5 turn and are not a sentence opener
    (chat-grade's SENTENCE_OPENERS plus EXTRA_OPENERS), in both directions. The list is
    printed so a reader can confirm it holds every name.
Not checked: distractor key words (`keys`), which name the other key, and the answer-class
lists of the `abstain_exact` rows, which list ordinary names and places by design.

Exit status 1 when any checked word occurs in the other direction.
"""
import glob
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PANELS = os.path.join(ROOT, 'data', 'panels')
LADDER = os.path.expanduser('~/uor-r4-local/ladder/panel')
SUBJECT = 'conversational-v5'
NOT_NAMES = {'i', "i'm", "i'll", "i've", "i'd", 'mr', 'ms', 'mrs'}
EXTRA_OPENERS = {'tell', 'then', 'two', 'wait', 'oops', 'here', 'last', 'bus', 'today'}


def sentence_openers():
    source = open(os.path.join(ROOT, 'crates', 'uor-r4-training', 'src', 'bin',
                               'chat-grade.rs'), encoding='utf-8').read()
    block = source.split('const SENTENCE_OPENERS: &[&str] = &[', 1)[1].split('];', 1)[0]
    return set(re.findall(r'"([^"]+)"', block)) | EXTRA_OPENERS


def words(text):
    text = text.replace('\u2019', "'").lower()
    out = []
    for w in re.split(r"[^\w']|_", text):
        w = w.strip("'")
        if w:
            out.append(w)
    return out


def proper_names(turns):
    openers = sentence_openers()
    lower = set()
    for turn in turns:
        lower |= {w for w in re.findall(r"[\w']+", turn) if w[:1].islower()}
    names = set()
    for turn in turns:
        for sentence in re.split(r'(?<=[.!?])\s+', turn):
            tokens = re.findall(r"[\w']+", sentence)
            for i, token in enumerate(tokens):
                if not token[:1].isupper() or token.lower() in NOT_NAMES:
                    continue
                if i > 0 or (token.lower() not in lower and token.lower() not in openers):
                    names.add(token.strip("'").lower())
    return names


def strip_readme(text, marker):
    return re.sub(rf'## {marker}.*?(?=\n## )', '', text, flags=re.S)


def collect(paths):
    vocabulary, values, names = {}, {}, set()
    for path in paths:
        base = os.path.basename(path)
        text = open(path, encoding='utf-8', errors='replace').read()
        if base == 'README.md':
            text = strip_readme(text, r'Held-out memory panel `conversational-v4\*`')
            text = strip_readme(text, r'Held-out memory acceptance panel `conversational-v5\*`')
        if base == 'MANIFEST.sha256':
            text = '\n'.join(l for l in text.splitlines() if SUBJECT not in l)
        for w in words(text):
            vocabulary.setdefault(w, os.path.relpath(path, os.path.expanduser('~')))
        if base == f'{SUBJECT}-checks.tsv':
            for line in text.splitlines():
                if line.startswith('#') or not line.strip():
                    continue
                fields = line.rstrip('\n').split('\t')
                if fields[1] != 'exact':
                    continue
                for column in (fields[3], fields[4]):
                    for term in column.split('|'):
                        for w in words(term):
                            values.setdefault(w, fields[0])
        elif base.endswith('-checks.tsv') and base.startswith('conversational-'):
            for line in text.splitlines():
                if line.startswith('#') or not line.strip():
                    continue
                fields = line.rstrip('\n').split('\t')
                if len(fields) < 5 or fields[1] != 'exact':
                    continue
                for column in (fields[3], fields[4]):
                    for term in column.split('|'):
                        for w in words(term):
                            values.setdefault(w, f'{base}:{fields[0]}')
    return vocabulary, values, names


def main():
    earlier_files = sorted(
        f for f in glob.glob(os.path.join(PANELS, '*'))
        if SUBJECT not in os.path.basename(f) and os.path.isfile(f))
    earlier_files += sorted(glob.glob(os.path.join(LADDER, '*.json')))
    subject_files = sorted(glob.glob(os.path.join(PANELS, f'{SUBJECT}*')))
    earlier_vocabulary, earlier_values, earlier_names = collect(earlier_files)
    subject_vocabulary, subject_values, subject_names = collect(subject_files)
    rows = json.load(open(os.path.join(PANELS, f'{SUBJECT}.json'), encoding='utf-8'))
    names = proper_names([t for r in rows for t in r['user_turns']])
    subject_names |= names

    # The reverse direction is scoped to `conversational-v4` on purpose: that is the
    # panel whose checker (`scripts/check_panel_v4_novelty.py`) now reads the v5 files as
    # part of its vocabulary, so v4's checked words are the ones v5 must not contain. The
    # v2 and v3 name extraction also returns sentence openers ("the", "we", "at") and
    # ordinary key words ("grandpa"), which a natural panel cannot avoid and which no
    # checker inspects; v5 does reuse ordinary frame words (brother, sister, aunt, uncle,
    # room, desk) that occur in earlier panels, and those are not values.
    v4_files = [f for f in earlier_files if 'conversational-v4' in os.path.basename(f)]
    _, v4_values, v4_names = collect(v4_files)
    for path in v4_files:
        if os.path.basename(path).endswith('.json') and 'checks' not in os.path.basename(path):
            try:
                payload = json.load(open(path, encoding='utf-8'))
            except ValueError:
                continue
            v4_names |= proper_names([t for r in payload for t in r.get('user_turns', [])])

    collisions = []
    for kind, checked, vocabulary in (
            ('v5 value in an earlier panel', sorted(subject_values),
             earlier_vocabulary),
            ('v5 name in an earlier panel', sorted(names), earlier_vocabulary),
            ('v4 value or name in v5', sorted(set(v4_values) | v4_names),
             subject_vocabulary)):
        for w in checked:
            if w in vocabulary:
                collisions.append({'kind': kind, 'word': w, 'found_in': vocabulary[w]})
    print(json.dumps({
        'subject': SUBJECT,
        'earlier_files': len(earlier_files),
        'earlier_vocabulary': len(earlier_vocabulary),
        'subject_files': [os.path.basename(f) for f in subject_files],
        'subject_vocabulary': len(subject_vocabulary),
        'subject_rows': len(rows),
        'v5_value_words_checked': len(subject_values),
        'v4_value_and_name_words_checked': len(set(v4_values) | v4_names),
        'v5_proper_names_checked': sorted(names),
        'collisions': collisions,
        'result': 'pass' if not collisions else 'fail',
    }, indent=2))
    return 1 if collisions else 0


if __name__ == '__main__':
    sys.exit(main())
