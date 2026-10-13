#!/usr/bin/env python3
import os, sys, json, re

JSON_RE = re.compile(r"^__JSON_SUMMARY__\s*(\{.*\})\s*$")

def main():
    if len(sys.argv) != 3:
        print("Usage: parse_logs.py <raw_log_dir> <parsed_out_dir>")
        sys.exit(2)

    raw_dir, out_dir = sys.argv[1], sys.argv[2]
    os.makedirs(out_dir, exist_ok=True)

    n = 0
    for fn in sorted(os.listdir(raw_dir)):
        if not fn.endswith(".log"):
            continue
        path = os.path.join(raw_dir, fn)
        summary = None
        with open(path, "r", encoding="utf-8", errors="ignore") as f:
            for line in f:
                m = JSON_RE.match(line.strip())
                if m:
                    summary = json.loads(m.group(1))
        if summary is None:
            summary = {"parsed": False, "log_file": fn}

        out_path = os.path.join(out_dir, fn.replace(".log", ".json"))
        with open(out_path, "w", encoding="utf-8") as g:
            json.dump(summary, g, indent=2, sort_keys=True)
        n += 1

    print(f"Parsed {n} log(s) into {out_dir}")

if __name__ == "__main__":
    main()
