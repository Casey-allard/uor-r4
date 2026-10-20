"""Lead scratch: build (1) a real scope hierarchy from Rust sources and (2) a byte-level code corpus.

Tree: root -> crate -> file -> nested brace scopes ({...}), ignoring braces inside comments, strings and char
literals (a light lexer; lifetimes like 'a are not char literals).  Output: code_tree.npz with parent[] and depth[].
Corpus: every .rs file in crates/, split 95/5 by a hash of the path, written as code_train.bin / code_valid.bin.
"""
import hashlib, os, sys
import numpy as np

ROOT = "/home/user/uor-r4/crates"
TREE_CRATES = ["uor-r4-training", "uor-r4-graph-format", "uor-r4-router", "uor-r4-integer", "uor-r4-tokenizer"]


def scopes(src):
    """Yield +1 for '{' and -1 for '}' outside comments, strings and char literals."""
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if c == '/' and i + 1 < n and src[i + 1] == '/':
            j = src.find('\n', i); i = n if j < 0 else j; continue
        if c == '/' and i + 1 < n and src[i + 1] == '*':
            depth, i = 1, i + 2
            while i < n and depth:
                if src.startswith('/*', i): depth += 1; i += 2
                elif src.startswith('*/', i): depth -= 1; i += 2
                else: i += 1
            continue
        if c == 'r' and i + 1 < n and src[i + 1] in '#"' and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == '_')):
            j = i + 1; hashes = 0
            while j < n and src[j] == '#': hashes += 1; j += 1
            if j < n and src[j] == '"':
                end = src.find('"' + '#' * hashes, j + 1); i = n if end < 0 else end + 1 + hashes; continue
        if c == '"':
            j = i + 1
            while j < n and src[j] != '"':
                j += 2 if src[j] == '\\' else 1
            i = j + 1; continue
        if c == "'":
            # char literal: 'x' or '\n' or '\u{..}'; otherwise a lifetime
            if i + 2 < n and src[i + 1] != '\\' and src[i + 2] == "'": i += 3; continue
            if i + 1 < n and src[i + 1] == '\\':
                j = src.find("'", i + 2); i = n if j < 0 else j + 1; continue
            i += 1; continue
        if c == '{': yield 1
        elif c == '}': yield -1
        i += 1


def build_tree():
    parent, depth = [-1], [0]
    for crate in TREE_CRATES:
        cnode = len(parent); parent.append(0); depth.append(1)
        for dp, _, fs in os.walk(os.path.join(ROOT, crate)):
            for f in sorted(fs):
                if not f.endswith('.rs'): continue
                src = open(os.path.join(dp, f), encoding='utf-8', errors='replace').read()
                fnode = len(parent); parent.append(cnode); depth.append(2)
                stack = [fnode]
                for s in scopes(src):
                    if s > 0:
                        node = len(parent); parent.append(stack[-1]); depth.append(depth[stack[-1]] + 1); stack.append(node)
                    elif len(stack) > 1:
                        stack.pop()
    return np.array(parent), np.array(depth)


def build_corpus(out_dir):
    train, valid = bytearray(), bytearray()
    for dp, _, fs in os.walk(ROOT):
        if '/target' in dp: continue
        for f in sorted(fs):
            if not f.endswith('.rs'): continue
            path = os.path.join(dp, f)
            data = open(path, 'rb').read()
            h = int(hashlib.sha256(path.encode()).hexdigest(), 16) % 100
            (valid if h < 5 else train).extend(data + b"\n")
    open(os.path.join(out_dir, 'code_train.bin'), 'wb').write(bytes(train))
    open(os.path.join(out_dir, 'code_valid.bin'), 'wb').write(bytes(valid))
    return len(train), len(valid)


if __name__ == '__main__':
    out = sys.argv[1]
    parent, depth = build_tree()
    np.savez(os.path.join(out, 'code_tree.npz'), parent=parent, depth=depth)
    internal = np.zeros(len(parent), bool); internal[parent[parent >= 0]] = True
    print(f"tree: {len(parent)} nodes, {internal.sum()} internal, {(~internal).sum()} leaves, max depth {depth.max()}")
    print("depth histogram:", np.bincount(depth).tolist())
    kids = np.bincount(parent[parent >= 0], minlength=len(parent))[internal]
    print("children per internal node: mean %.2f, median %d, max %d" % (kids.mean(), np.median(kids), kids.max()))
    print("corpus bytes (train, valid):", build_corpus(out))
