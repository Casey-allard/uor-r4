#!/usr/bin/env python3
import os, sys, json, csv, datetime

FIELDS = [
    "timestamp","parsed","log_file",
    "seed","mode","sector_mode","phase_dims",
    "time_pressure_lambda","scale_mode","radial_bins",
    "extra_budget","max_slots_per_bucket","chart_beta","chart_iters",
    "test_mse_before","test_mse_after","train_label_sse_per","test_label_sse_per",
    "buckets","slots_used",
]

def main():
    if len(sys.argv) != 3:
        print("Usage: summarize.py <parsed_dir> <summary_csv>")
        sys.exit(2)

    parsed_dir, out_csv = sys.argv[1], sys.argv[2]
    rows = []

    for fn in sorted(os.listdir(parsed_dir)):
        if not fn.endswith(".json"):
            continue
        with open(os.path.join(parsed_dir, fn), "r", encoding="utf-8") as f:
            j = json.load(f)

        row = {k: "" for k in FIELDS}
        row["timestamp"] = datetime.datetime.now().isoformat(timespec="seconds")
        row["parsed"] = j.get("parsed", True)
        row["log_file"] = j.get("log_file", fn.replace(".json",".log"))

        args = j.get("args", {})
        metrics = j.get("metrics", {})
        for k in ["seed","mode","sector_mode","phase_dims","time_pressure_lambda","scale_mode","radial_bins",
                  "extra_budget","max_slots_per_bucket","chart_beta","chart_iters"]:
            row[k] = args.get(k, "")

        for k in ["test_mse_before","test_mse_after","train_label_sse_per","test_label_sse_per","buckets","slots_used"]:
            row[k] = metrics.get(k, "")

        rows.append(row)

    os.makedirs(os.path.dirname(out_csv) or ".", exist_ok=True)
    write_header = not os.path.exists(out_csv)
    with open(out_csv, "a", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=FIELDS)
        if write_header:
            w.writeheader()
        for r in rows:
            w.writerow(r)

    print(f"Appended {len(rows)} row(s) to {out_csv}")

if __name__ == "__main__":
    main()
