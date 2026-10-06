#!/usr/bin/env python3
"""T2 diagnostic analysis: is the gold source the argmax of the read's scores?

Reads the `dump_scores/` files written by `mqar-bench ... dump_scores=1`
(TEMPORARY T2 instrumentation) and reports, over the held-out items:

  (a) fraction where the gold source position is the argmax of the read's score
      vector (equivalently, of its softmax weight row) -- over sources, and over
      sources plus the learned NoRead slot;
  (b) fraction where the gold is in the top-3 / top-10;
  (c) the read weight (probability) on the gold: mean and distribution;
  (d) the NoRead probability: mean;
  (e) the rank of the gold position: distribution;
  (f) how many candidate positions compete (nominal row length and effective
      support);
plus the same view for the non-learned `reference_rule`'s implied source, the
identity of the argmax when it is not the gold, and per-layer/per-head detail.

Usage:
  python3 analyze_read_scores.py ROOT [--json OUT.json] [--md OUT.md]
where ROOT is the sealed report root (its `dump_scores/held_out*` is read).
"""

import argparse
import json
import math
import struct
from collections import Counter

import numpy as np


def load(root, label="held_out"):
    index = json.load(open(f"{root}/dump_scores/{label}.index.json"))
    raw = open(f"{root}/dump_scores/{index['weight_blob']}", "rb").read()
    blob = np.frombuffer(raw, dtype="<f4")
    assert blob.size == index["weight_blob_floats"], (blob.size, index["weight_blob_floats"])
    return index, blob


def rows_for_item(index, blob, row, layer_index):
    """[heads, context] weight matrix of one (item, layer)."""
    context = index["context"]
    heads = index["heads"]
    off = row["weights_offsets"][layer_index]
    return blob[off : off + heads * context].reshape(heads, context)


def quantiles(values):
    if not len(values):
        return {}
    a = np.asarray(values, dtype=np.float64)
    qs = [0, 1, 5, 10, 25, 50, 75, 90, 95, 99, 100]
    return {f"p{q}": float(np.percentile(a, q)) for q in qs} | {
        "mean": float(a.mean()),
        "min": float(a.min()),
        "max": float(a.max()),
    }


def histogram(values, edges):
    a = np.asarray(values, dtype=np.float64)
    out = {}
    for lo, hi in zip(edges[:-1], edges[1:]):
        out[f"[{lo:g},{hi:g})"] = int(((a >= lo) & (a < hi)).sum())
    return out


def landmark(pos, row, index):
    """A short structural name for a source position, for argmax diagnosis."""
    vf = row["value_first"]
    vlen = len(row["value"])
    if pos == row["gold"]:
        return "gold"
    if vf <= pos < vf + vlen:
        return "fact_value_other_piece"
    if pos == row["key_last"]:
        return "fact_key_last"
    if pos == row["query_key_last"]:
        return "query_key_last"
    if pos == row["query"]:
        return "query_row_self"
    if row["query_start"] <= pos < row["query"]:
        return "query_span"
    if pos == 0:
        return "position_0"
    if pos == row["query"] - 1:
        return "previous_position"
    if row["fact_start"] <= pos <= row["key_last"]:
        return "fact_key_span"
    return "other"


def named_source(row, index, name):
    vf = row["value_first"]
    vlen = len(row["value"])
    return {
        "gold": row["gold"],
        "gold_minus_1": row["gold"] - 1,
        "gold_plus_1": min(row["gold"] + 1, vf + vlen - 1),
        "fact_key_last": row["key_last"],
        "query_key_last": row["query_key_last"],
        "query_row_self": row["query"],
        "previous_position": row["query"] - 1,
        "position_0": 0,
        "fact_start": row["fact_start"],
    }[name]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("root")
    ap.add_argument("--label", default="held_out")
    ap.add_argument("--json", default=None)
    ap.add_argument("--md", default=None)
    args = ap.parse_args()

    index, blob = load(args.root, args.label)
    rows = index["rows"]
    layers = index["layers"]
    heads = index["heads"]
    context = index["context"]

    # ---- per (item, layer, head) read rows -------------------------------
    per_row = []  # one dict per (item, layer, head)
    for row in rows:
        for li, layer in enumerate(layers):
            W = rows_for_item(index, blob, row, li)
            sum_w = W.sum(axis=1)  # per head; = 1 - NoRead
            for h in range(heads):
                w = W[h, : row["query"] + 1].astype(np.float64)
                p_gold = float(W[h, row["gold"]])
                p_noread = float(1.0 - sum_w[h])
                order = np.sort(w)[::-1]
                rank = int(1 + (w > p_gold).sum())
                rank_with_noread = int(rank + (1 if p_noread > p_gold else 0))
                top = float(order[0]) if order.size else 0.0
                support = int((w > 0).sum())
                eff = int((w > 1e-6).sum())
                denom = float((w**2).sum())
                participation = float(w.sum() ** 2 / denom) if denom > 0 else 0.0
                entropy = float(-(w[w > 0] * np.log(w[w > 0])).sum())
                per_row.append(
                    {
                        "item": row,
                        "layer": layer,
                        "head": h,
                        "query": row["query"],
                        "gold": row["gold"],
                        "n_candidates": row["query"] + 1,
                        "p_gold": p_gold,
                        "p_noread": p_noread,
                        "p_top": top,
                        "rank": rank,
                        "rank_with_noread": rank_with_noread,
                        "argmax_source": int(np.argmax(w)) if w.size else -1,
                        "is_argmax_source": bool(rank == 1),
                        "is_argmax_all": bool(rank == 1 and p_noread <= p_gold),
                        "noread_wins": bool(p_noread > top),
                        "support": support,
                        "support_1e-6": eff,
                        "participation": participation,
                        "entropy": entropy,
                        "gold_gap_nats": float(math.log(top) - math.log(p_gold))
                        if p_gold > 0
                        else float("inf"),
                    }
                )

    def frac(flags):
        return float(np.mean(flags)) if len(flags) else float("nan")

    all_rows = per_row
    last_layer = layers[-1]
    last_rows = [r for r in per_row if r["layer"] == last_layer]

    def agg(subset, name):
        p_gold = [r["p_gold"] for r in subset]
        noread = [r["p_noread"] for r in subset]
        ranks = [r["rank"] for r in subset]
        support = [r["support"] for r in subset]
        eff = [r["support_1e-6"] for r in subset]
        part = [r["participation"] for r in subset]
        candidates = [r["n_candidates"] for r in subset]
        return {
            "name": name,
            "rows": len(subset),
            "items": len({id(r["item"]) for r in subset}),
            "gold_is_argmax_over_sources": frac([r["is_argmax_source"] for r in subset]),
            "gold_is_argmax_including_noread": frac([r["is_argmax_all"] for r in subset]),
            "gold_in_top3": frac([r["rank"] <= 3 for r in subset]),
            "gold_in_top10": frac([r["rank"] <= 10 for r in subset]),
            "gold_rank_1_incl_noread": frac([r["rank_with_noread"] == 1 for r in subset]),
            "noread_wins_row": frac([r["noread_wins"] for r in subset]),
            "p_gold": quantiles(p_gold),
            "p_top": quantiles([r["p_top"] for r in subset]),
            "p_gold_gt_0.5": frac([p > 0.5 for p in p_gold]),
            "p_gold_gt_0.1": frac([p > 0.1 for p in p_gold]),
            "p_gold_lt_1_over_n": frac(
                [r["p_gold"] < 1.0 / r["n_candidates"] for r in subset]
            ),
            "p_noread_mean": float(np.mean(noread)),
            "p_noread": quantiles(noread),
            "rank_hist": dict(sorted(Counter(ranks).items())),
            "rank_buckets": {
                "1": int(sum(1 for r in ranks if r == 1)),
                "2": int(sum(1 for r in ranks if r == 2)),
                "3": int(sum(1 for r in ranks if r == 3)),
                "4-10": int(sum(1 for r in ranks if 4 <= r <= 10)),
                "11-50": int(sum(1 for r in ranks if 11 <= r <= 50)),
                ">50": int(sum(1 for r in ranks if r > 50)),
            },
            "n_candidates_mean": float(np.mean(candidates)),
            "n_candidates_range": [int(min(candidates)), int(max(candidates))],
            "support_mean": float(np.mean(support)),
            "support_gt_1e-6_mean": float(np.mean(eff)),
            "support_gt_1e-6_range": [int(min(eff)), int(max(eff))],
            "participation_ratio_mean": float(np.mean(part)),
        }

    report = {
        "root": args.root,
        "label": args.label,
        "instrument": {
            "layers": layers,
            "heads": heads,
            "context": context,
            "items": index["items"],
            "sequences": index["sequences"],
            "logits_bit_identical": index["logits_bit_identical"],
            "max_abs_logit_gap": index["max_abs_logit_gap"],
        },
        "all_read_rows": agg(all_rows, "all read rows (every layer and head)"),
        "last_read_layer": agg(last_rows, f"last read layer {last_layer}"),
        "per_layer": [agg([r for r in per_row if r["layer"] == L], f"layer {L}") for L in layers],
        "per_head_last_layer": [
            agg([r for r in last_rows if r["head"] == h], f"layer {last_layer} head {h}")
            for h in range(heads)
        ],
    }

    # ---- the whole fact span as the source: a cap on any gold convention --
    fact_hits = []
    fact_span_mass = []
    near_gold = []
    for r in last_rows:
        row = r["item"]
        W = rows_for_item(index, blob, row, layers.index(r["layer"]))
        w = W[r["head"]]
        lo = row["fact_start"]
        hi = row["value_first"] + len(row["value"]) - 1
        j = int(np.argmax(w[: row["query"] + 1]))
        fact_hits.append(lo <= j <= hi)
        fact_span_mass.append(float(w[lo : hi + 1].sum()))
        near_gold.append(any(int(np.argmax(w[: row["query"] + 1])) == row["gold"] + o for o in (-1, 0, 1)))
    report["last_layer_any_fact_position_is_argmax"] = frac(fact_hits)
    report["last_layer_any_position_within_1_of_gold_is_argmax"] = frac(near_gold)
    report["last_layer_fact_span_mass"] = quantiles(fact_span_mass)

    # ---- breakdowns of the last-layer rows --------------------------------
    report["last_layer_by_bucket"] = {
        b: agg(
            [r for r in last_rows if r["item"]["bucket"] == b],
            f"last layer, bucket {b}",
        )
        for b in sorted({r["item"]["bucket"] for r in last_rows})
    }
    report["last_layer_by_form"] = {
        f: agg([r for r in last_rows if r["item"]["form"] == f], f"last layer, form {f}")
        for f in sorted({r["item"]["form"] for r in last_rows})
    }
    report["last_layer_by_gap"] = {
        str(g): agg([r for r in last_rows if r["item"]["gap"] == g], f"last layer, gap {g}")
        for g in sorted({r["item"]["gap"] for r in last_rows})
    }

    # ---- per item (last layer) -------------------------------------------
    items = {}
    for row in rows:
        items[id(row)] = row
    last_by_item = {}
    for r in last_rows:
        last_by_item.setdefault(id(r["item"]), []).append(r)
    item_flags = []
    for key, rs in last_by_item.items():
        item_flags.append(
            {
                "item_any_head_argmax": any(r["is_argmax_source"] for r in rs),
                "item_any_head_argmax_all": any(r["is_argmax_all"] for r in rs),
                "item_best_head_p_gold": max(r["p_gold"] for r in rs),
                "item_best_head_rank": min(r["rank"] for r in rs),
            }
        )
    report["per_item_last_layer"] = {
        "items": len(item_flags),
        "any_head_gold_is_argmax": frac([f["item_any_head_argmax"] for f in item_flags]),
        "any_head_gold_is_argmax_incl_noread": frac(
            [f["item_any_head_argmax_all"] for f in item_flags]
        ),
        "best_head_rank_1": frac([f["item_best_head_rank"] == 1 for f in item_flags]),
        "best_head_p_gold": quantiles([f["item_best_head_p_gold"] for f in item_flags]),
    }

    # ---- argmax identity when it is not the gold -------------------------
    argmax_names = Counter()
    for r in last_rows:
        if r["is_argmax_source"]:
            argmax_names["gold"] += 1
        else:
            argmax_names[landmark(r["argmax_source"], r["item"], index)] += 1
    report["last_layer_argmax_identity"] = dict(argmax_names.most_common())

    # ---- structural landmark agreement -----------------------------------
    report["landmarks_last_layer"] = {
        "mean_p_on_fact_key_last": float(
            np.mean([rows_for_item(index, blob, r["item"], len(layers) - 1)[r["head"], r["item"]["key_last"]] for r in last_rows])
        ),
        "mean_p_on_query_key_last": float(
            np.mean(
                [
                    rows_for_item(index, blob, r["item"], len(layers) - 1)[r["head"], r["item"]["query_key_last"]]
                    for r in last_rows
                ]
            )
        ),
        "mean_p_on_query_row_self": float(
            np.mean(
                [
                    rows_for_item(index, blob, r["item"], len(layers) - 1)[r["head"], r["item"]["query"]]
                    for r in last_rows
                ]
            )
        ),
    }

    # ---- instrument sanity: the weights are a probability row ------------
    sums = []
    for r in per_row:
        W = rows_for_item(index, blob, r["item"], layers.index(r["layer"]))
        sums.append(float(W[r["head"], : r["query"] + 1].sum()))
    report["weight_row_sanity"] = {
        "sum_w_mean": float(np.mean(sums)),
        "sum_w_min": float(np.min(sums)),
        "sum_w_max": float(np.max(sums)),
        "implied_noread_min": float(1 - np.max(sums)),
        "implied_noread_max": float(1 - np.min(sums)),
        "note": "sum_j w_j = 1 - NoRead(j) must lie in [0, 1]",
    }

    # ---- where the read actually looks: span masses and distance profile --
    bins = [(0, 0), (1, 2), (3, 8), (9, 32), (33, 128), (129, 384)]
    prof = {f"{lo}-{hi}": [] for lo, hi in bins}
    spans = {k: [] for k in [
        "fact_span", "query_span", "fact_key_span", "query_key_span",
        "fact_value_span", "position_0", "last_16", "gold",
    ]}
    gold_dist = []
    for r in last_rows:
        row = r["item"]
        W = rows_for_item(index, blob, row, layers.index(r["layer"]))
        w = W[r["head"], : row["query"] + 1].astype(np.float64)
        t = row["query"]
        vf, vlen = row["value_first"], len(row["value"])
        masks = {
            "fact_span": (row["fact_start"], vf + vlen - 1),
            "query_span": (row["query_start"], t),
            "fact_key_span": (row["fact_start"], row["key_last"]),
            "query_key_span": (row["query_start"], row["query_key_last"]),
            "fact_value_span": (vf, vf + vlen - 1),
            "position_0": (0, 0),
            "last_16": (max(0, t - 15), t),
            "gold": (row["gold"], row["gold"]),
        }
        for name, (lo, hi) in masks.items():
            spans[name].append(float(w[lo : hi + 1].sum()))
        gold_dist.append(t - row["gold"])
        for lo, hi in bins:
            prof[f"{lo}-{hi}"].append(float(w[max(0, t - hi) : t - lo + 1].sum()))
    report["last_layer_span_mass"] = {
        k: {"mean_total_mass": float(np.mean(v)), "rows": len(v)} for k, v in spans.items()
    }
    report["last_layer_distance_profile"] = {
        "bins_sources": [f"t-{hi}..t-{lo}" for lo, hi in bins],
        "mean_total_mass_in_bin": {k: float(np.mean(v)) for k, v in prof.items()},
    }
    report["gold_distance_from_query"] = quantiles(gold_dist)

    # ---- empirical score profile in distance (each row's score up to a shift)
    # `ln w[t-d] - ln w[t]` is exactly `s[t-d] - s[t]`: the score vector is
    # determined up to one additive constant per row, which cancels here.
    ds = list(range(0, 33)) + [48, 64, 96, 128, 192, 256, 320, 383]
    shape = {}
    for L in layers:
        acc = {d: [] for d in ds}
        li = layers.index(L)
        for r in per_row:
            if r["layer"] != L:
                continue
            row = r["item"]
            W = rows_for_item(index, blob, row, li)
            t = row["query"]
            base = float(W[r["head"], t])
            if base <= 0:
                continue
            lb = math.log(base)
            for d in ds:
                if t - d < 0:
                    continue
                v = float(W[r["head"], t - d])
                if v > 0:
                    acc[d].append(math.log(v) - lb)
        shape[f"layer_{L}"] = {
            str(d): {
                "mean_rel_log_score": float(np.mean(v)) if v else None,
                "rows": len(v),
            }
            for d, v in acc.items()
        }
    report["empirical_score_profile_in_distance"] = {
        "note": "mean over rows of (ln w[t-d] - ln w[t]) = mean of (s[t-d] - s[t]); the per-row additive constant of the score vector cancels",
        "by_layer": shape,
    }
    report["empirical_score_profile_last_layer"] = shape[f"layer_{layers[-1]}"]
    last_li = len(layers) - 1
    per_head_shape = {}
    for h in range(heads):
        acc = {d: [] for d in ds}
        for r in per_row:
            if r["layer"] != layers[-1] or r["head"] != h:
                continue
            row = r["item"]
            W = rows_for_item(index, blob, row, last_li)
            t = row["query"]
            base = float(W[h, t])
            if base <= 0:
                continue
            lb = math.log(base)
            for d in ds:
                if t - d < 0:
                    continue
                v = float(W[h, t - d])
                if v > 0:
                    acc[d].append(math.log(v) - lb)
        per_head_shape[f"head_{h}"] = {
            str(d): (float(np.mean(v)) if v else None) for d, v in acc.items()
        }
    report["empirical_score_profile_last_layer_by_head"] = per_head_shape

    # ---- sensitivity to the definition of the gold source ----------------
    names = [
        "gold",
        "gold_minus_1",
        "gold_plus_1",
        "fact_key_last",
        "query_key_last",
        "query_row_self",
        "previous_position",
        "position_0",
        "fact_start",
    ]
    sens = {}
    for L in layers:
        sub = [r for r in per_row if r["layer"] == L]
        per_name = {}
        for name in names:
            vals = []
            hits = []
            for r in sub:
                pos = named_source(r["item"], index, name)
                W = rows_for_item(index, blob, r["item"], layers.index(L))
                w = W[r["head"]].astype(np.float64)
                vals.append(float(w[pos]))
                j = int(np.argmax(w[: r["item"]["query"] + 1]))
                hits.append(j == pos)
            per_name[name] = {
                "mean_p": float(np.mean(vals)),
                "argmax_frac": float(np.mean(hits)),
            }
        sens[f"layer_{L}"] = per_name
    report["source_definition_sensitivity"] = sens

    # ---- the model's own answer vs gold rank -----------------------------
    correct_rows = []
    for row in rows:
        correct_rows.append(row["argmax_token"] == row["target_token"])
    report["model_first_content_accuracy_from_dump"] = frac(correct_rows)
    cross = {}
    for r in last_rows:
        key = "argmax" if r["is_argmax_source"] else ("top3" if r["rank"] <= 3 else "below_top3")
        cross.setdefault(key, []).append(r["item"]["argmax_token"] == r["item"]["target_token"])
    report["model_correct_by_gold_rank_last_layer"] = {
        k: {"rows": len(v), "model_correct": frac(v)} for k, v in cross.items()
    }
    # Same cross-tab pooled over every read row.
    cross_all = {}
    for r in all_rows:
        key = "argmax" if r["is_argmax_source"] else ("top3" if r["rank"] <= 3 else "below_top3")
        cross_all.setdefault(key, []).append(r["item"]["argmax_token"] == r["item"]["target_token"])
    report["model_correct_by_gold_rank_all_rows"] = {
        k: {"rows": len(v), "model_correct": frac(v)} for k, v in cross_all.items()
    }

    # ---- reference rule comparison ---------------------------------------
    rule = {
        "items": len(rows),
        "rule_source_present": frac([r["rule_source"] is not None for r in rows]),
        "rule_source_is_gold": frac([r["rule_source"] == r["gold"] for r in rows]),
        "rule_first_content_hit": frac([r["rule_first_content_hit"] for r in rows]),
        "rule_full_hit": frac([r["rule_full_hit"] for r in rows]),
        "model_correct": frac(correct_rows),
    }
    agree = []
    for r in last_rows:
        if r["item"]["rule_source"] is None:
            continue
        agree.append(r["item"]["rule_source"] == r["argmax_source"])
    rule["rule_source_is_model_argmax_last_layer"] = frac(agree)
    rule["rule_source_is_model_argmax_max_over_layers"] = frac(
        [
            any(
                r["item"]["rule_source"] == rr["argmax_source"]
                for rr in per_row
                if rr["item"] is r["item"]
            )
            for r in last_rows
            if r["item"]["rule_source"] is not None
        ]
    )
    report["reference_rule"] = rule

    text = json.dumps(report, indent=2, sort_keys=False, default=str)
    if args.json:
        open(args.json, "w").write(text + "\n")
    if args.md:
        open(args.md, "w").write(render_markdown(report))
    print(text)


def render_markdown(report):
    L = []
    a = L.append
    a("# T2 — is the gold source the argmax of the read's score vector?\n")
    inst = report["instrument"]
    a(
        f"Instrument: {inst['items']} held-out items, {len(inst['layers'])} read layers "
        f"{inst['layers']}, {inst['heads']} heads, context {inst['context']}; "
        f"instrumented forward logits bit-identical to the scored forward: "
        f"{inst['logits_bit_identical']} (max |gap| {inst['max_abs_logit_gap']:.3e}).\n"
    )
    for key in ("all_read_rows", "last_read_layer"):
        s = report[key]
        a(f"## {s['name']} ({s['rows']} rows, {s['items']} items)\n")
        a(f"- gold is the argmax over sources: **{s['gold_is_argmax_over_sources']:.4f}**")
        a(f"- gold is the argmax including NoRead: {s['gold_is_argmax_including_noread']:.4f}")
        a(f"- gold in top-3: {s['gold_in_top3']:.4f}; top-10: {s['gold_in_top10']:.4f}")
        a(f"- gold mean weight: {s['p_gold']['mean']:.4f}; p>0.5: {s['p_gold_gt_0.5']:.4f}")
        a(f"- mean NoRead: {s['p_noread_mean']:.4f}; NoRead wins the row: {s['noread_wins_row']:.4f}")
        a(
            f"- candidates: mean {s['n_candidates_mean']:.0f} (nominal), effective support "
            f"(w>1e-6) mean {s['support_gt_1e-6_mean']:.1f}, participation ratio "
            f"{s['participation_ratio_mean']:.1f}"
        )
        a(f"- rank buckets: {s['rank_buckets']}\n")
    a("## Reference rule (non-learned)\n")
    for k, v in report["reference_rule"].items():
        a(f"- {k}: {v}")
    return "\n".join(L) + "\n"


if __name__ == "__main__":
    main()
