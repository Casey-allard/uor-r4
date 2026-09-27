"""Brace depth at every token of the BPE-coded held-out code (scratch analysis input).
Depth at a token = unmatched '{' before the token's first byte, ignoring braces in comments, strings and
char literals (the cycle-2 lexer); files are concatenated, so depth is clamped at 0 and reset per file."""
import sys, numpy as np
sys.path.insert(0, sys.argv[3])
from make_code_data import scopes  # noqa: E402  (yields +1/-1 for code braces only)

text = open(sys.argv[1], 'rb').read().decode('utf-8', errors='replace').encode('utf-8', errors='replace')
raw = open(sys.argv[1], 'rb').read()
toks = np.fromfile(sys.argv[2] + '/valid.u16', dtype='<u2')
lens = np.fromfile(sys.argv[2] + '/lens.u16', dtype='<u2')
assert int(lens[toks].astype(np.int64).sum()) == len(raw), (int(lens[toks].sum()), len(raw))
# per-byte depth: re-run the lexer, recording brace byte positions
src = raw.decode('latin-1')           # 1 char per byte, so positions are byte offsets
depth_delta = np.zeros(len(raw) + 1, np.int32)
i, n = 0, len(src)
def lex(src):
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if c == '/' and i + 1 < n and src[i + 1] == '/':
            j = src.find('\n', i); i = n if j < 0 else j; continue
        if c == '/' and i + 1 < n and src[i + 1] == '*':
            d, i = 1, i + 2
            while i < n and d:
                if src.startswith('/*', i): d += 1; i += 2
                elif src.startswith('*/', i): d -= 1; i += 2
                else: i += 1
            continue
        if c == 'r' and i + 1 < n and src[i + 1] in '#"' and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == '_')):
            j = i + 1; h = 0
            while j < n and src[j] == '#': h += 1; j += 1
            if j < n and src[j] == '"':
                e = src.find('"' + '#' * h, j + 1); i = n if e < 0 else e + 1 + h; continue
        if c == '"':
            j = i + 1
            while j < n and src[j] != '"': j += 2 if src[j] == '\\' else 1
            i = j + 1; continue
        if c == "'":
            if i + 2 < n and src[i + 1] != '\\' and src[i + 2] == "'": i += 3; continue
            if i + 1 < n and src[i + 1] == '\\':
                j = src.find("'", i + 2); i = n if j < 0 else j + 1; continue
            i += 1; continue
        if c == '{': yield i, 1
        elif c == '}': yield i, -1
        i += 1
depth = np.zeros(len(raw), np.int32)
cur = 0; last = 0
for pos, s in lex(src):
    depth[last:pos + 1] = cur          # depth before this brace applies up to and including it
    cur = max(0, cur + s); last = pos + 1
depth[last:] = cur
starts = np.concatenate([np.zeros(1, np.int64), np.cumsum(lens[toks].astype(np.int64))[:-1]])
tok_depth = depth[starts]
np.save(sys.argv[2] + '/valid_depth.npy', tok_depth)
print('tokens', len(toks), 'depth histogram', np.bincount(tok_depth)[:14].tolist(), 'max', tok_depth.max())
