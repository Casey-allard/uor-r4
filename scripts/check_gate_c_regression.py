#!/usr/bin/env python3
import sys
import json

# Single source of truth for the pinned record and thresholds:
# docs/transformerless/gate_c_pinned.json — shared with the trend-alarm
# job (scripts/gate_c_trend.sh). Re-pins happen in that one file, with an
# era note (#281 established the discipline after a dual-pin miss).
import os
_PIN_PATH = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "docs", "transformerless", "gate_c_pinned.json",
)
with open(_PIN_PATH) as _f:
    _PIN = json.load(_f)
PINNED_TOP1_AGREEMENT = _PIN["rule12_top1_agreement"]
PINNED_BITS_PER_TOKEN = _PIN["rule12_bits_per_token"]
MAX_TOP1_DROP = _PIN["alarm"]["top1_drop_abs"]
MAX_BPT_WORSEN = _PIN["alarm"]["bits_regress_abs"]

def main():
    if len(sys.argv) < 2:
        print("Usage: check_gate_c_regression.py <path/to/score_report.json>")
        sys.exit(1)

    report_path = sys.argv[1]
    with open(report_path, 'r') as f:
        report = json.load(f)
    
    rule12 = report.get('gate_c', {}).get('rule12_precedence')
    if not rule12:
        print("Error: 'rule12_precedence' metrics not found in the report.")
        sys.exit(1)
        
    top1 = rule12.get('top1_agreement', 0.0)
    bpt = rule12.get('bits_per_token', 0.0)
    
    print(f"Gate C (Rule 1+2) Current: top-1={top1:.4f} ({top1*100:.1f}%), bits/token={bpt:.4f}")
    print(f"Gate C (Rule 1+2) Pinned : top-1={PINNED_TOP1_AGREEMENT:.4f} ({PINNED_TOP1_AGREEMENT*100:.1f}%), bits/token={PINNED_BITS_PER_TOKEN:.4f}")
    
    failed = False
    
    if top1 < (PINNED_TOP1_AGREEMENT - MAX_TOP1_DROP):
        print(f"🚨 REGRESSION ALARM: top-1 agreement dropped by more than {MAX_TOP1_DROP*100:.1f} points!")
        print(f"   Delta: {(top1 - PINNED_TOP1_AGREEMENT)*100:.2f} points")
        failed = True
        
    if bpt > (PINNED_BITS_PER_TOKEN + MAX_BPT_WORSEN):
        print(f"🚨 REGRESSION ALARM: bits/token worsened by more than {MAX_BPT_WORSEN:.2f}!")
        print(f"   Delta: {bpt - PINNED_BITS_PER_TOKEN:+.4f} bits/token")
        failed = True
        
    if failed:
        print("Gate C regression check FAILED.")
        sys.exit(1)
    else:
        print("Gate C regression check PASSED.")
        sys.exit(0)

if __name__ == '__main__':
    main()
