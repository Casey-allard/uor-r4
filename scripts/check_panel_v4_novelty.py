#!/usr/bin/env python3
"""Check that conversational-v4 states no value word or proper name of an earlier panel.

References #820. Reads the v4 panel and checks file and every earlier panel file
(`data/panels/*` and `~/uor-r4-local/ladder/panel/*.json` when present, v4 files excluded).
Words follow chat-grade's `words()` rule: lowercased, split at anything that is not a
letter, digit or apostrophe, apostrophes trimmed from the ends.

Checked, word by word:
  * values: every spelling in the `terms` (expected) and `forbid` (distractor) columns of
    the v4 `exact` (memory) rows;
  * proper names: capitalised words of the v4 user turns that are inside a sentence, or
    begin one, never occur lowercase in a v4 turn and are not a sentence opener
    (chat-grade's SENTENCE_OPENERS plus EXTRA_OPENERS); titles Mr/Ms/Mrs and forms of I
    excluded. The list is printed so a reader can confirm it holds every name.
Not checked: distractor key words (`keys`), which name the other key ("grandpa",
"brother"), and the answer-class lists of the `abstain_exact` rows, which list ordinary
colours, numbers and places by design and are never stated in a v4 turn.

Exit status 1 when any checked word occurs in an earlier panel file.
"""
import glob
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PANELS = os.path.join(ROOT, 'data', 'panels')
LADDER = os.path.expanduser('~/uor-r4-local/ladder/panel')
NOT_NAMES = {'i', "i'm", "i'll", "i've", "i'd", 'mr', 'ms', 'mrs'}
# Sentence-initial words that are not names: chat-grade's SENTENCE_OPENERS (parsed from
# its source) and these.
EXTRA_OPENERS = {'tell', 'then', 'two', 'wait', 'oops', 'here', 'last', 'bus', 'today'}


def sentence_openers():
    source = open(os.path.join(ROOT, 'crates', 'uor-r4-training', 'src', 'bin',
                               'chat-grade.rs'), encoding='utf-8').read()
    block = source.split('const SENTENCE_OPENERS: &[&str] = &[', 1)[1].split('];', 1)[0]
    return set(re.findall(r'"([^"]+)"', block)) | EXTRA_OPENERS


def words(text):
    text = text.replace('’', "'").lower()
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


def main():
    earlier = sorted(f for f in glob.glob(os.path.join(PANELS, '*'))
                     if 'conversational-v4' not in os.path.basename(f))
    earlier += sorted(f for f in glob.glob(os.path.join(LADDER, '*.json'))
                      if 'conversational-v4' not in os.path.basename(f))
    vocabulary = {}
    for path in earlier:
        with open(path, encoding='utf-8') as f:
            text = f.read()
        if os.path.basename(path) == 'README.md':
            # The README's own v4 section describes v4; only the earlier text counts.
            text = re.sub(r'## Held-out memory panel `conversational-v4\*`.*?(?=\n## )', '',
                          text, flags=re.S)
        if os.path.basename(path) == 'MANIFEST.sha256':
            text = '\n'.join(l for l in text.splitlines() if 'conversational-v4' not in l)
        for w in words(text):
                vocabulary.setdefault(w, os.path.relpath(path, os.path.expanduser('~')))
    rows = json.load(open(os.path.join(PANELS, 'conversational-v4.json'), encoding='utf-8'))
    values = {}
    for line in open(os.path.join(PANELS, 'conversational-v4-checks.tsv'), encoding='utf-8'):
        if line.startswith('#') or not line.strip():
            continue
        fields = line.rstrip('\n').split('\t')
        if fields[1] != 'exact':
            continue
        for column in (fields[3], fields[4]):
            for term in column.split('|'):
                for w in words(term):
                    values.setdefault(w, fields[0])
    names = proper_names([t for r in rows for t in r['user_turns']])
    hits = []
    for kind, checked in (('value', sorted(values)), ('name', sorted(names))):
        for w in checked:
            if w in vocabulary:
                hits.append((kind, w, vocabulary[w]))
    print(json.dumps({
        'earlier_files': len(earlier),
        'earlier_vocabulary': len(vocabulary),
        'v4_rows': len(rows),
        'value_words_checked': len(values),
        'proper_names_checked': sorted(names),
        'collisions': [{'kind': k, 'word': w, 'first_file': f} for k, w, f in hits],
        'result': 'pass' if not hits else 'fail',
    }, indent=2))
    return 1 if hits else 0


if __name__ == '__main__':
    sys.exit(main())
