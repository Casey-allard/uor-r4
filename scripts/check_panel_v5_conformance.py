#!/usr/bin/env python3
"""Structural conformance of `conversational-v5*` (References #2029). CPU only.

`chat-grade check` is the authoritative structural validator, but this round was
restricted to panel construction with no grader run at all, so this script is a
line-by-line Python port of the parts of `crates/uor-r4-training/src/bin/chat-grade.rs`
that decide whether the panel is well formed and whether its controls do what the
panel README claims:

  * `words`, `contains_phrase`, `has_any`
  * `parse_checks` (the structural errors it raises)
  * `validate_checks` (history/turn agreement, expected and forbidden values and
    distractor keys in the right turns, no forbidden word in a user turn)
  * `abstains_strictly`, `abstention_fault`, `fabricated_specifics` and the phrase
    lists behind them
  * `copy_replies` and `check_only_controls` (constants, echo last, echo history,
    copy first/last stated, binding swap, expected value, adversarial abstentions)
  * `rust_string_literals`, `pattern`, `template_match`, `longest_shared_run`,
    `reference_overlap`, `STRICT_LEAK_N` and `MIN_TEMPLATE_LITERALS`, applied to the
    M-world sources that the in-repo unit test `panels_v3_v4_share_no_m_world_phrasing`
    uses for v3 and v4

It prints a JSON verdict and exits non-zero when anything fails. It reads the swaps
file passed on the command line, so it can also check that the control is not vacuous.
"""
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PANELS = os.path.join(ROOT, 'data', 'panels')
PANEL = 'conversational-v5'
STRICT_LEAK_N = 4
MIN_TEMPLATE_LITERALS = 3

NOT_SPECIFIC = ['i', "i'm", "i'll", "i've", "i'd", 'ok']
SENTENCE_OPENERS = [
    'a', 'about', 'actually', 'after', 'all', 'also', 'am', 'an', 'and', 'any', 'are', 'as',
    'ask', 'aw', 'aww', 'because', 'but', 'can', 'could', 'did', 'do', 'does', 'even', 'every',
    'for', 'good', 'great', 'has', 'have', 'he', 'hello', 'her', 'hey', 'hi', 'him', 'his', 'hm',
    'hmm', 'how', 'if', 'in', 'is', 'it', "it's", 'its', 'just', 'let', "let's", 'maybe', 'me',
    'my', 'nice', 'no', 'nobody', 'not', 'now', 'oh', 'okay', 'on', 'one', 'only', 'oops', 'or',
    'our', 'perhaps', 'please', 'really', 'sadly', 'she', 'should', 'so', 'some', 'sometimes',
    'sorry', 'still', 'sure', 'thank', 'thanks', 'that', "that's", 'the', 'their', 'them',
    'then', 'there', "there's", 'these', 'they', "they're", 'this', 'those', 'though', 'to',
    'try', 'uh', 'um', 'unfortunately', 'was', 'we', "we're", 'well', 'what', "what's", 'when',
    'where', 'which', 'who', 'why', 'will', 'with', 'without', 'would', 'wow', 'yeah', 'yes',
    'you', "you're", 'your',
]
NOT_INABILITY_AFTER = ['wait']
ABSTAIN_PHRASES = [
    "don't know", 'dont know', 'do not know', "didn't tell", 'did not tell', "haven't told",
    'have not told', 'not sure', 'no way to know', 'no way of knowing', "can't know",
    'cannot know', "can't", 'cant', 'cannot', 'can not', 'not able', 'unable', 'no idea',
    'i wish i could', 'impossible',
]
AFFIRM_PHRASES = [
    'yes', 'yeah', 'yep', 'yay', 'of course', "let's", 'lets', "i'd love to", 'i would love to',
    'here you go', 'there you go', 'here it is', 'no problem', 'sounds fun', 'sounds good',
    'nothing is impossible', 'i can do that', 'i can do it',
]
GUESS_MARKERS = [
    'maybe', 'probably', 'perhaps', 'i think', 'i guess', 'i bet', 'my guess', 'might be',
    'must be', 'could be', 'likely', 'i believe',
]
INVITE_PHRASES = ['tell me', 'ask', 'let me know', 'show me']
DEFAULT_CONSTANTS = [
    "I'm not sure. Can you tell me more about what you mean?",
    'That sounds nice! Thank you for telling me.',
    'Once upon a time, there was a little girl named Lily. She liked to play outside.',
]
ADVERSARIAL_ABSTENTIONS = [
    "Yes! I can't wait to play with you!",
    "I'm not sure. Sam is the one.",
    "I don't know, but they are under the bed.",
]


def words(text):
    text = text.replace('\u2019', "'").lower()
    out = []
    for piece in re.split(r"[^\w']|_", text):
        piece = piece.strip("'")
        if piece:
            out.append(piece)
    return out


def contains_phrase(text, phrase):
    return bool(phrase) and any(text[i:i + len(phrase)] == phrase
                                for i in range(len(text) - len(phrase) + 1))


def has_any(text, phrases):
    return any(contains_phrase(text, words(p)) for p in phrases)


def abstains_strictly(reply):
    for phrase in ABSTAIN_PHRASES:
        target = words(phrase)
        for i in range(len(reply) - len(target) + 1):
            if reply[i:i + len(target)] == target:
                nxt = reply[i + len(target)] if i + len(target) < len(reply) else None
                if nxt not in NOT_INABILITY_AFTER:
                    return True
    return False


def reply_sentences(reply):
    out, current = [], ''
    for char in reply:
        if char in '.!?\n':
            piece = words(current)
            if piece:
                out.append((piece, char == '?'))
            current = ''
        else:
            current += char
    piece = words(current)
    if piece:
        out.append((piece, False))
    return out


def fabricated_specifics(users, reply):
    user_words = set()
    for turn in users:
        user_words.update(words(turn))
    text = reply.replace('\u2019', "'")
    out, sentence_start, current = [], True, ''

    def flush(word, start):
        trimmed = word.strip("'")
        if trimmed:
            lower = trimmed.lower()
            digit = any(c.isdigit() for c in trimmed)
            capital = trimmed[0].isupper()
            told = lower in user_words
            named = (capital and lower not in NOT_SPECIFIC
                     and (not start or lower not in SENTENCE_OPENERS))
            if not told and (digit or named):
                out.append(trimmed)
            return False
        # chat-grade sets the sentence-start flag only for a non-empty word, so
        # whitespace after a full stop does not consume it.
        return start

    sentence_start = True
    for char in text:
        if char.isalnum() or char == "'":
            current += char
            continue
        sentence_start = flush(current, sentence_start)
        current = ''
        if char in '.!?\n"\u201c:':
            sentence_start = True
    flush(current, sentence_start)
    return out


def abstention_fault(users, reply):
    reply_words = words(reply)
    if not abstains_strictly(reply_words):
        return 'no abstain phrase'
    sure_agrees = any(w == 'sure' and (i == 0 or reply_words[i - 1] != 'not')
                      for i, w in enumerate(reply_words))
    if sure_agrees or has_any(reply_words, AFFIRM_PHRASES):
        return 'agrees'
    for sentence, question in reply_sentences(reply):
        invites = question or has_any(sentence, INVITE_PHRASES)
        if not invites and has_any(sentence, GUESS_MARKERS):
            return 'guesses'
        if 'but' in sentence:
            at = sentence.index('but')
            rest = sentence[at + 1:]
            if not (question or abstains_strictly(rest) or has_any(rest, INVITE_PHRASES)):
                return 'asserts after but'
    if fabricated_specifics(users, reply):
        return 'names a specific'
    return None


def parse_checks(text):
    rows = {}
    for number, line in enumerate(text.splitlines(), start=1):
        if not line.strip() or line.startswith('#'):
            continue
        fields = line.split('\t')
        if not 4 <= len(fields) <= 6:
            raise SystemExit(f'checks line {number}: expected 4 to 6 tab-separated fields')
        row_id, kind, history, terms = fields[:4]
        forbid = fields[4] if len(fields) > 4 else '-'
        keys = fields[5] if len(fields) > 5 else '-'

        def term_list(value):
            out = [words(t) for t in value.split('|')]
            if any(not t for t in out):
                raise SystemExit(f'checks line {number}: an empty term')
            return out

        if history not in ('none', 'recall', 'derived', 'topic'):
            raise SystemExit(f'checks line {number}: history {history}')
        if kind == 'any':
            parsed = ('any', term_list(terms))
        elif kind == 'exact' and terms != '-':
            parsed = ('exact', term_list(terms))
        elif kind == 'abstain' and terms == '-':
            parsed = ('abstain', [])
        elif kind == 'abstain_exact' and terms == '-':
            parsed = ('abstain_exact', [])
        else:
            raise SystemExit(f'checks line {number}: kind {kind} with terms {terms!r}')
        forbid_list = [] if forbid == '-' else term_list(forbid)
        keys_list = [] if keys == '-' else term_list(keys)
        if parsed[0] not in ('exact', 'abstain_exact') and (forbid_list or keys_list):
            raise SystemExit(f'checks line {number}: forbid or keys on a {parsed[0]} check')
        if keys_list and parsed[0] != 'exact':
            raise SystemExit(f'checks line {number}: keys on a {parsed[0]} check')
        if parsed[0] == 'exact' and not forbid_list:
            raise SystemExit(
                f'checks line {number}: an exact check needs a forbidden (distractor) value')
        for term in parsed[1]:
            if term in forbid_list:
                raise SystemExit(
                    f"checks line {number}: '{' '.join(term)}' is both expected and forbidden")
        if row_id in rows:
            raise SystemExit(f'checks repeat {row_id}')
        rows[row_id] = dict(kind=parsed[0], terms=parsed[1], history=history,
                            forbid=forbid_list, keys=keys_list)
    return rows


def any_in(terms, texts):
    for term in terms:
        if any(contains_phrase(text, term) for text in texts):
            return ' '.join(term)
    return None


def validate_checks(checks, requests):
    errors, seen = [], set()
    for request in requests:
        check = checks.get(request['id'])
        if check is None:
            if request['id'].startswith('conv-') and len(request['user_turns']) > 1:
                errors.append(f"multi-turn row {request['id']} has no check")
            continue
        seen.add(request['id'])
        multi = len(request['user_turns']) > 1
        if multi == (check['history'] == 'none'):
            errors.append(f"row {request['id']} has {len(request['user_turns'])} user turns "
                          f"but history {check['history']}")
        turns = [words(t) for t in request['user_turns']]
        earlier, last = turns[:-1], turns[-1:]
        if check['kind'] in ('any', 'exact'):
            term = any_in(check['terms'], last)
            if term:
                errors.append(f"row {request['id']}: check term '{term}' is in the last user turn")
            if check['history'] == 'recall' and not any_in(check['terms'], earlier):
                errors.append(f"recall row {request['id']}: no check term is in an earlier "
                              'user turn')
        if check['kind'] == 'abstain_exact':
            term = any_in(check['forbid'], turns)
            if term:
                errors.append(f"row {request['id']}: forbidden word '{term}' is in a user turn")
        if check['kind'] in ('abstain', 'question') and check['history'] in ('recall', 'topic'):
            errors.append(f"row {request['id']}: {check['history']} needs an any or exact check")
        if check['kind'] == 'exact':
            if check['history'] != 'recall':
                errors.append(f"exact row {request['id']} must be a recall row")
            term = any_in(check['forbid'], last)
            if term:
                errors.append(f"row {request['id']}: forbidden value '{term}' is in the last "
                              'user turn')
            if not any_in(check['forbid'], earlier):
                errors.append(f"exact row {request['id']}: no distractor value is stated in an "
                              'earlier turn')
            term = any_in(check['keys'], last)
            if term:
                errors.append(f"row {request['id']}: distractor key '{term}' is in the last "
                              'user turn')
            if check['keys'] and not any_in(check['keys'], earlier):
                errors.append(f"exact row {request['id']}: no distractor key is stated in an "
                              'earlier turn')
    unmatched = sorted(set(checks) - seen)
    return errors, unmatched


def passes(check, users, reply):
    reply_words = words(reply)
    forbidden = any(contains_phrase(reply_words, t) for t in check['forbid'])
    if check['kind'] == 'any':
        return any(contains_phrase(reply_words, t) for t in check['terms'])
    if check['kind'] == 'abstain':
        return has_any(reply_words, ABSTAIN_PHRASES)
    if check['kind'] == 'question':
        return '?' in reply
    if check['kind'] == 'exact':
        return (not forbidden
                and not any(contains_phrase(reply_words, t) for t in check['keys'])
                and any(contains_phrase(reply_words, t) for t in check['terms']))
    return not forbidden and abstention_fault(users, reply) is None


def copy_replies(check, users):
    if check['kind'] != 'exact':
        return None
    earlier = users[:-1]
    found = []
    for t, turn in enumerate(earlier):
        turn_words = words(turn)
        for term in check['terms'] + check['forbid']:
            for w in range(len(turn_words) - len(term) + 1):
                if turn_words[w:w + len(term)] == term:
                    found.append(((t, w), term))
    if not found:
        return None
    return [' '.join(min(found)[1]), ' '.join(max(found)[1])]


def check_only_controls(requests, checks, swaps, constants):
    per_category = {}
    for request in requests:
        row = per_category.setdefault(request['category'], {
            'echo_last': [0, 0], 'echo_history': [0, 0], 'constants': [[0, 0] for _ in constants],
            'copy_first': [0, 0], 'copy_last': [0, 0], 'binding_swap': [0, 0],
            'expected_value': [0, 0],
            'adversarial': [[0, 0] for _ in ADVERSARIAL_ABSTENTIONS]})
        check = checks.get(request['id'])
        if check is None:
            continue
        users = list(request['user_turns'])
        last = users[-1] if users else ''

        def tally(slot, result):
            slot[0] += 1
            slot[1] += 1 if result else 0

        tally(row['echo_last'], passes(check, users, last))
        tally(row['echo_history'], passes(check, users, ' '.join(users)))
        copy = copy_replies(check, users)
        if copy:
            tally(row['copy_first'], passes(check, users, copy[0]))
            tally(row['copy_last'], passes(check, users, copy[1]))
        if request['id'] in swaps:
            tally(row['binding_swap'], passes(check, users, swaps[request['id']]))
        if check['kind'] == 'exact':
            tally(row['expected_value'],
                  all(passes(check, users, ' '.join(t) + '.') for t in check['terms']))
        if check['kind'] == 'abstain_exact':
            for slot, reply in zip(row['adversarial'], ADVERSARIAL_ABSTENTIONS):
                tally(slot, passes(check, users, reply))
        for slot, reply in zip(row['constants'], constants):
            tally(slot, passes(check, users, reply))
    return per_category


# --- the M-world leak rule, ported from chat-grade -----------------------------

def rust_string_literals(source):
    chars, out, i = list(source), [], 0
    while i < len(chars):
        c = chars[i]
        if c == '/' and i + 1 < len(chars) and chars[i + 1] == '/':
            while i < len(chars) and chars[i] != '\n':
                i += 1
            continue
        if c == "'":
            if i + 1 < len(chars) and chars[i + 1] == '\\' and i + 3 < len(chars) \
                    and chars[i + 3] == "'":
                i += 4
            elif i + 2 < len(chars) and chars[i + 2] == "'":
                i += 3
            else:
                i += 1
            continue
        identifier_before = i > 0 and (chars[i - 1].isalnum() or chars[i - 1] == '_')
        if c == 'r' and not identifier_before and i + 1 < len(chars) and chars[i + 1] in '"#':
            j, hashes = i + 1, 0
            while j < len(chars) and chars[j] == '#':
                hashes += 1
                j += 1
            if j < len(chars) and chars[j] == '"':
                start, k = j + 1, j + 1
                while k < len(chars) and not (
                        chars[k] == '"'
                        and all(k + 1 + h < len(chars) and chars[k + 1 + h] == '#'
                                for h in range(hashes))):
                    k += 1
                out.append(''.join(chars[start:k]))
                i = k + 1 + hashes
                continue
        if c == '"':
            text, k = '', i + 1
            while k < len(chars) and chars[k] != '"':
                if chars[k] == '\\':
                    nxt = chars[k + 1] if k + 1 < len(chars) else None
                    if nxt in ('n', 't'):
                        text += ' '
                    elif nxt == '\n':
                        k += 2
                        while k < len(chars) and chars[k].isspace():
                            k += 1
                        continue
                    elif nxt is not None:
                        text += nxt
                    k += 2
                    continue
                text += chars[k]
                k += 1
            out.append(text)
            i = k + 1
            continue
        i += 1
    return out


def pattern(text):
    out, rest = [], text
    while '{' in rest:
        open_at = rest.index('{')
        close = rest.find('}', open_at)
        if close < 0:
            break
        out.extend((w, True) for w in words(rest[:open_at]))
        out.append((None, False))
        rest = rest[open_at + close + 1:]
    out.extend((w, True) for w in words(rest))
    return out


def template_match(pat, turn):
    if not pat:
        return not turn
    head, literal = pat[0]
    rest = pat[1:]
    if literal:
        return bool(turn) and turn[0] == head and template_match(rest, turn[1:])
    return any(template_match(rest, turn[k:]) for k in range(1, min(4, len(turn)) + 1))


def longest_shared_run(pat, turn):
    best = 0
    previous = [0] * (len(turn) + 1)
    for word, literal in pat:
        current = [0] * (len(turn) + 1)
        if literal:
            for j, token in enumerate(turn):
                if token == word:
                    current[j + 1] = previous[j] + 1
                    best = max(best, current[j + 1])
        previous = current
    return best


def reference_overlap(pat, turn):
    literals = sum(1 for _, literal in pat if literal)
    placeholders = len(pat) - literals
    whole = (literals > 0 and (placeholders == 0 or literals >= MIN_TEMPLATE_LITERALS)
             and template_match(pat, turn))
    return whole, longest_shared_run(pat, turn)


def m_world_patterns():
    out = []
    for source in ('milestone_world.rs', 'milestone_world_v2.rs'):
        path = os.path.join(ROOT, 'crates', 'uor-r4-training', 'src', source)
        for text in rust_string_literals(open(path, encoding='utf-8').read()):
            pat = pattern(text)
            if any(literal for _, literal in pat):
                out.append((text, pat))
    return out


def main():
    requests = json.load(open(os.path.join(PANELS, f'{PANEL}.json'), encoding='utf-8'))
    a = json.load(open(os.path.join(PANELS, f'{PANEL}-a.json'), encoding='utf-8'))
    b = json.load(open(os.path.join(PANELS, f'{PANEL}-b.json'), encoding='utf-8'))
    checks = parse_checks(open(os.path.join(PANELS, f'{PANEL}-checks.tsv'),
                               encoding='utf-8').read())
    swaps = {}
    for line in open(os.path.join(PANELS, f'{PANEL}-swaps.tsv'), encoding='utf-8'):
        if line.strip() and not line.startswith('#'):
            row_id, reply = line.rstrip('\n').split('\t')
            if row_id in swaps:
                raise SystemExit(f'swaps repeat {row_id}')
            swaps[row_id] = reply

    problems = []
    if len(requests) != 64:
        problems.append(f'expected 64 rows, found {len(requests)}')
    if [r['id'] for r in a] + [r['id'] for r in b] != [r['id'] for r in requests]:
        problems.append('the -a and -b files are not the panel split in order')
    if len({r['id'] for r in requests}) != len(requests):
        problems.append('a row id repeats')
    counts = {}
    for request in requests:
        counts[request['category']] = counts.get(request['category'], 0) + 1
    if counts != {'multi_turn_memory': 40, 'unknowable_or_impossible': 24}:
        problems.append(f'category counts {counts}')

    errors, unmatched = validate_checks(checks, requests)
    problems.extend(errors)
    if any(i.startswith('conv-v5-') for i in unmatched):
        problems.append(f'matched no request: {[i for i in unmatched if i.startswith("conv-v5-")]}')

    memory_ids = {r['id'] for r in requests if r['category'] == 'multi_turn_memory'}
    if set(swaps) != memory_ids:
        problems.append('the swaps file is not exactly the memory rows')

    controls = check_only_controls(requests, checks, swaps, DEFAULT_CONSTANTS)
    memory = controls['multi_turn_memory']
    unknowable = controls['unknowable_or_impossible']
    if memory['echo_last'][1] or memory['echo_history'][1]:
        problems.append('a memory row is answerable by echoing a turn or the history')
    if any(c[1] for c in memory['constants']):
        problems.append('a constant passes a memory row')
    if memory['binding_swap'][1]:
        problems.append('a binding swap passes its row')
    if memory['expected_value'][1] != memory['expected_value'][0]:
        problems.append('a bare expected spelling does not pass its own row')
    for slot, reply in zip(unknowable['adversarial'], ADVERSARIAL_ABSTENTIONS):
        if slot[1]:
            problems.append(f'adversarial abstention passes an unknowable row: {reply}')
    if unknowable['constants'][0][1] != unknowable['constants'][0][0]:
        problems.append('constant 1 does not pass every unknowable row')

    leaks = []
    patterns = m_world_patterns()
    for request in requests:
        for turn in request['user_turns']:
            turn_words = words(turn)
            for text, pat in patterns:
                whole, run = reference_overlap(pat, turn_words)
                if whole or run >= STRICT_LEAK_N:
                    leaks.append({'id': request['id'], 'turn': turn, 'reference': text,
                                  'whole': whole, 'run': run})
                    break
    if leaks:
        problems.append(f'{len(leaks)} M-world phrasing leak(s)')

    print(json.dumps({
        'panel': PANEL,
        'rows': len(requests),
        'categories': counts,
        'checks_parsed': len(checks),
        'validate_checks_errors': errors,
        'checks_without_request': unmatched,
        'm_world_patterns': len(patterns),
        'm_world_leaks': leaks,
        'check_only_controls': {
            'multi_turn_memory': {
                'expected_value': memory['expected_value'],
                'binding_swap': memory['binding_swap'],
                'echo_last': memory['echo_last'],
                'echo_history': memory['echo_history'],
                'copy_first_stated': memory['copy_first'],
                'copy_last_stated': memory['copy_last'],
                'constants': memory['constants'],
            },
            'unknowable_or_impossible': {
                'echo_last': unknowable['echo_last'],
                'echo_history': unknowable['echo_history'],
                'constants': unknowable['constants'],
                'adversarial_abstentions': [
                    {'reply': r, 'result': c}
                    for r, c in zip(ADVERSARIAL_ABSTENTIONS, unknowable['adversarial'])],
            },
        },
        'result': 'pass' if not problems else 'fail',
        'problems': problems,
    }, indent=2))
    return 1 if problems else 0


if __name__ == '__main__':
    sys.exit(main())
