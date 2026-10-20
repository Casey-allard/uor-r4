"""Attribute macOS `sample` self samples of a D11 run to step-0d categories.

usage: sample_shares.py SAMPLE.txt [ROOT_SUBSTRING]

Each sample's leaf is classified by walking from the leaf toward the root and
taking the first frame that names a category kernel. Weight-map kernels are
checked first at every frame, so the read layers' q/k/v/null/out projections
and the recurrence's gate/in/out projections count as weight maps, and only
the scan over cached positions (heads, L2 distance, softmax tables, mixture)
counts as read. With ROOT_SUBSTRING, only samples below a frame whose line
contains it are counted (for example the hot loop's call site).
Prints JSON.
"""
import json
import os
import re
import sys

LINE = re.compile(r'^([ +!:|]*)(\d+) (.*)$')

CATEGORIES = [
    ('weight_maps', ['stack_gemv_pairs', 'stack_gemv', 'stack_pair_tables', 'stack_map_pairs2',
                     'stack_map_pairs', 'stack_map_nibbles', 'stack_map_task', 'stack_row_task',
                     'split_rows', 'stack_activation_tables', 'stack_dequant_row']),
    ('read', ['stack_heads', 'split_heads', 'stack_head_task', 'stack_l2_distance',
              'stack_lorentz_distance', 'stack_query_tables', 'stack_dot', 'stack_mix_row',
              'stack_exp_neg', 'stack_read']),
    ('recurrence', ['stack_recurrence', 'stack_lanes', 'stack_rotation', 'stack_hamilton',
                    'stack_snap']),
    ('pointer', ['stack_pointer', 'pointer_coefficient']),
    ('mlp_elementwise', ['stack_swiglu']),
]
IDLE_LEAVES = ['__psynch_cvwait', '__psynch_mutexwait', 'semaphore_wait_trap', '__ulock_wait',
               'swtch_pri', 'thread_switch', '__workq_kernreturn', 'mach_msg2_trap',
               '__semwait_signal', 'kevent']


ALL_KERNELS = sorted({k for _, ks in CATEGORIES for k in ks}, key=len, reverse=True)


def kernel_name(frame):
    """The category kernels a v0-mangled Rust symbol names (identifiers are
    length-prefixed, so `16stack_gemv_pairs` is not `10stack_gemv`), and the
    symbol."""
    symbol = frame.split('  (in ')[0].strip()
    names = [k for k in ALL_KERNELS
             if re.search(r'(?<!\d)' + str(len(k)) + re.escape(k) + r'(?![a-z_])', symbol)]
    if not names:
        tail = re.findall(r'\d+([A-Za-z_][A-Za-z_]*)', symbol)
        names = tail[-1:] if tail else []
    return names, symbol


def classify(path):
    leaf_names, leaf_symbol = kernel_name(path[-1])
    # A thread blocked or yielding (a pool join waiting inside a map or a
    # read, an idle worker) is waiting, whatever frame it waits in. In the
    # critical-path view (CRITICAL=1), a wait inside a phase is charged to the
    # phase: it is the step's wall time spent waiting for that phase's tasks.
    critical = os.environ.get('CRITICAL') == '1'
    if not critical and any(leaf_symbol.startswith(w) for w in IDLE_LEAVES):
        return 'idle_wait'
    if any(s in leaf_symbol for s in ('rayon_core', 'crossbeam')):
        return 'pool_overhead'
    for frame in reversed(path):
        names, _ = kernel_name(frame)
        for category, kernels in CATEGORIES:
            if any(n in kernels for n in names):
                return category
    if any(leaf_symbol.startswith(w) for w in IDLE_LEAVES):
        return 'idle_wait'
    return 'other'


def main():
    text = open(sys.argv[1]).read().split('\n')
    root = sys.argv[2] if len(sys.argv) > 2 else None
    start = next(i for i, l in enumerate(text) if l.startswith('Call graph:')) + 1
    stack = []  # (depth, line, count)
    totals = {}
    detail = {}

    def close(node, children):
        depth, line, count, path = node
        self_count = count - children
        if self_count <= 0:
            return
        if root is not None and not any(root in f for f in path):
            return
        category = classify(path)
        totals[category] = totals.get(category, 0) + self_count
        leaf = kernel_name(path[-1])[0]
        key = (category, leaf[-1] if leaf else kernel_name(path[-1])[1][:60])
        detail[key] = detail.get(key, 0) + self_count

    child_sums = []
    for line in text[start:]:
        if not line.strip() or line.startswith('Total number') or line.startswith('Sort by'):
            if line.startswith('Total number') or line.startswith('Sort by'):
                break
            continue
        m = LINE.match(line)
        if not m:
            continue
        depth = len(m.group(1))
        count = int(m.group(2))
        frame = m.group(3)
        while stack and stack[-1][0] >= depth:
            node = stack.pop()
            children = child_sums.pop()
            close(node, children)
        if stack:
            child_sums[-1] += count
        path = (stack[-1][3] if stack else []) + [frame]
        stack.append((depth, frame, count, path))
        child_sums.append(0)
    while stack:
        node = stack.pop()
        close(node, child_sums.pop())
    busy = sum(v for k, v in totals.items() if k != 'idle_wait')
    out = {
        'file': sys.argv[1],
        'root': root,
        'samples': totals,
        'busy_samples': busy,
        'busy_shares': {k: round(v / busy, 4) for k, v in totals.items() if k != 'idle_wait'},
        'top_leaves': [
            {'category': c, 'kernel': n, 'samples': v, 'busy_share': round(v / busy, 4)}
            for (c, n), v in sorted(detail.items(), key=lambda kv: -kv[1])[:25]
        ],
    }
    print(json.dumps(out, indent=1))


main()
