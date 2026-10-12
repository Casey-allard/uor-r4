#!/usr/bin/env python3
"""D23 softmax-path check (owner decision D23, 2026-10-12: no softmax anywhere).

Answers, for a named executed path: does this path's source compute a
normalized exponential — softmax, log-softmax, softmax cross-entropy,
log-sum-exp normalization, or a table/renamed equivalent? D23 covers training,
serving, sampling, objectives, auxiliary losses and teachers, including the
implicit softmax inside cross-entropy and exp-table serving normalization.

A compliance claim attaches this report; the report is evidence about source,
not about a model run. The scan skips test modules and comments, and it does
not flag scalar exponentials that are not a softmax (sigmoid/tanh gates, KL
temperatures) — those are recorded separately as scalar-exp sites so a
reviewer can confirm the classification instead of trusting it.

  python3 scripts/softmax_path_check.py                 every profile
  python3 scripts/softmax_path_check.py --path d11-serve
  python3 scripts/softmax_path_check.py --self-test
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# Banned-op patterns, each with the census class it belongs to. Definition and
# call sites are both hits: a path that computes, or calls into, one of these
# computes a normalized exponential.
BANNED = [
    ("softmax-op", re.compile(r"\bsoftmax(?:_last_dim|_into|_over_support)?\s*\(", re.I)),
    ("log-softmax", re.compile(r"\blog_softmax\b", re.I)),
    ("logsumexp-norm", re.compile(r"\b(?:row_)?log_?sum_?exp\b", re.I)),
    ("softmax-ce", re.compile(r"\bcross_entropy\b|\bsoftmax_ce\b", re.I)),
    ("exp-normalize", re.compile(r"\.exp\(\)\s*/\s*\w+\.sum|/\s*\w+\.exp\(\)\.sum|softmax_weights|admitted_softmax", re.I)),
]

# Scalar exponentials that are not a softmax by themselves (sigmoid/tanh
# gates, exp tables as tables). Recorded, not flagged.
SCALAR = re.compile(r"\b(?:sigmoid|tanh|stack_exp_neg|exp_table|stack_sigmoid_q31)\b", re.I)

# The executed-path profiles. Each maps a name to (search root, file globs).
PROFILES = {
    "stack-objective": [
        ("crates/uor-r4-training/src", ["geometric_stack.rs", "geometric_stack/**/*.rs"]),
        ("crates/uor-r4-training/src", ["cuda_stack_kernels.rs", "metal_stack_kernels.rs"]),
    ],
    "stack-serve": [
        ("crates/uor-r4-training/src", ["geometric_stack.rs", "stack_dialogue.rs"]),
    ],
    "d11-serve": [
        ("crates/uor-r4-integer/src", ["**/*.rs"]),
    ],
    "native-learner": [
        ("crates/uor-r4-core/src/native_geometric", ["**/*.rs"]),
    ],
    "training-aids": [
        ("crates/uor-r4-training/src", ["**/*.rs"]),
    ],
}

TEST_MARK = re.compile(r"^\s*#\[cfg\(test\)\]")


def scan_file(path):
    """Yield (line_number, class, text) for banned ops outside tests/comments."""
    try:
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return
    in_tests = False
    depth = 0
    for number, line in enumerate(lines, 1):
        if TEST_MARK.match(line):
            in_tests = True
            depth = 0
        stripped = line.split("//", 1)[0] if not line.lstrip().startswith(("///", "//!")) else ""
        if in_tests:
            depth += line.count("{") - line.count("}")
            if depth <= 0 and "}" in line:
                in_tests = False
            continue
        if not stripped.strip():
            continue
        for class_name, pattern in BANNED:
            if pattern.search(stripped):
                yield (number, class_name, line.strip())
                break


def scalar_sites(path):
    try:
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return
    for number, line in enumerate(lines, 1):
        if SCALAR.search(line) and not line.lstrip().startswith(("///", "//!")):
            yield (number, line.strip())


def run_profile(name):
    hits, scalar = [], []
    for root, globs in PROFILES[name]:
        base = ROOT / root
        for pattern in globs:
            for path in sorted(base.glob(pattern)):
                if path.is_file():
                    hits.extend((path.relative_to(ROOT), n, c, t) for n, c, t in scan_file(path))
                    scalar.extend((path.relative_to(ROOT), n, t) for n, t in scalar_sites(path))
    return hits, scalar


def main():
    args = sys.argv[1:]
    if args == ["--self-test"]:
        return self_test()
    names = list(PROFILES)
    if args[:1] == ["--path"]:
        if len(args) != 2 or args[1] not in PROFILES:
            print(f"usage: --path <{'|'.join(PROFILES)}>", file=sys.stderr)
            return 2
        names = [args[1]]
    dirty = False
    for name in names:
        hits, scalar = run_profile(name)
        status = "VIOLATIONS" if hits else "CLEAN"
        dirty = dirty or bool(hits)
        print(f"== {name}: {status} ({len(hits)} banned-op sites, {len(scalar)} scalar-exp sites recorded)")
        for path, number, class_name, text in hits:
            print(f"   {path}:{number} [{class_name}] {text[:120]}")
    return 1 if dirty else 0


def self_test():
    fixture = ROOT / "target" / "softmax-path-check-fixture.rs"
    fixture.parent.mkdir(parents=True, exist_ok=True)
    fixture.write_text(
        "fn loss(z: &[f32]) -> f32 {\n"
        "    let lse = row_log_sum_exp(z);\n"
        "    z[0] - lse\n"
        "}\n"
        "#[cfg(test)]\n"
        "mod tests {\n"
        "    fn uses_softmax() { softmax(&x); }\n"
        "}\n",
        encoding="utf-8",
    )
    hits = list(scan_file(fixture))
    fixture.unlink()
    assert len(hits) == 1 and hits[0][1] == "logsumexp-norm", hits
    print("self-test: pass (one banned site found, the test module ignored)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
