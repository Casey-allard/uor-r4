#!/usr/bin/env python3
"""idstop: prove that a probe run with the serving-time rules OFF is bit-for-bit
the sealed reference, field by field. Every field the SEALED record has, at any
depth, must exist in the new record with an equal value -- reply, reply_ids,
reply_eos, window_ids, window_text, span positions and sizes, all read masses,
the pointer gate/attention/span masses and the marker probabilities. Fields only
the new record has are reported separately as additions, never as differences,
so a new channel (copy_stop, reply_trace, reply_stopped_at) does not show up as
a change. Used for the idstop3 default proof: 3 arms x 3 conditions = 9 files,
288 records, 0 differing fields. Companion to
docs/evidence/idstop_nonfiring_classification_2026-10-09.txt.

    idstop-compare-records.py SEALED.json=NEW.json [SEALED2.json=NEW2.json ...]

    Example (paths as documented by the evidence file; the sealed files and the
    default runs both live on the lab's EU-RO-1 network volume rfsx702p68 =
    uor-shared-EU-RO-1):
      W=/workspace/uor-r4/deepseek/term-weight
      idstop-compare-records.py \\
        $W/bindprobe/rate/arm-ptr-natural.json=$W/bindprobe/idstop3/default/default-arm-ptr-f0-natural.json

    Exit 1 if any pair differs, so it can gate a delivery.
"""
import argparse
import json
import sys


def flatten(prefix, value, out):
    if isinstance(value, dict):
        for key, item in value.items():
            flatten(prefix + "." + key, item, out)
    elif isinstance(value, list):
        out[prefix] = json.dumps(value, sort_keys=True)
    else:
        out[prefix] = value


def compare(sealed_path, new_path):
    sealed = json.load(open(sealed_path))
    new = json.load(open(new_path))
    sealed_records = {record["id"]: record for record in sealed["records"]}
    new_records = {record["id"]: record for record in new["records"]}
    differences = []
    for rid, sealed_record in sealed_records.items():
        new_record = new_records.get(rid)
        if new_record is None:
            differences.append((rid, "<record>", "present", "MISSING"))
            continue
        flat_sealed = {}
        flat_new = {}
        flatten("", sealed_record, flat_sealed)
        flatten("", new_record, flat_new)
        for key, value in flat_sealed.items():
            if key not in flat_new:
                differences.append((rid, key, value, "MISSING"))
            elif flat_new[key] != value:
                differences.append((rid, key, value, flat_new[key]))
    added_records = sorted(set(new_records) - set(sealed_records))
    return differences, added_records, len(sealed_records)


def main():
    parser = argparse.ArgumentParser(
        description="idstop: field-for-field comparison of a rule-off run against a sealed record",
        formatter_class=argparse.RawDescriptionHelpFormatter, epilog=__doc__)
    parser.add_argument("pairs", nargs="+", metavar="SEALED.json=NEW.json",
                        help="one or more sealed=new record-file pairs")
    args = parser.parse_args()
    bad = 0
    for argument in args.pairs:
        if "=" not in argument:
            parser.error("expected SEALED.json=NEW.json, got %s" % argument)
        sealed, new = argument.split("=", 1)
        differences, added, cells = compare(sealed, new)
        label = new.split("/")[-1]
        if differences:
            bad += 1
            print("DIFFERS  %-44s %d of %d records, %d differing fields"
                  % (label, len({d[0] for d in differences}), cells, len(differences)))
            for rid, key, was, now in differences[:10]:
                print("         %s %s: sealed %r new %r" % (rid, key, was, now))
        else:
            print("IDENTICAL %-43s %d records, 0 differing fields, added records %s"
                  % (label, cells, added or "none"))
    print("SEALED-COMPARE %s" % ("PASS" if bad == 0 else "FAIL (%d files)" % bad))
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
