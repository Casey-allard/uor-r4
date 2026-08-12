#!/usr/bin/env python3
"""Run the T_r* rebuild from spin_H_core_v6 and write the bounded audit CSV.

This runner generates:
    results/prime_transport_recursive_system/prime_transport_Tr_rebuild_v1.csv

Using geometry_native_operator_model_v22 where:
    - T_r*: radial_transport_component_v22 (rebuilt from spin_H_core_v6)
    - T_c:  coupled_torus_kick_component_v20 (from spin_H_core_v6; unchanged)
    - T_z': fiber_phase_lift_component_v21   (from spin_H_core_v6; unchanged)
    - All other components: v10 (unchanged from v21)
"""

from __future__ import annotations

import sys
import time
from pathlib import Path

_root = Path(__file__).resolve().parent
if str(_root) not in sys.path:
    sys.path.insert(0, str(_root))

from geometry_native_operator_model_v22 import (
    OUTPUT_PATH_TR_REBUILD_V1,
    summarize_operator_from_spinH_core_v6_radial,
    write_Tr_rebuild_v1,
)


def main(depth: int = 8) -> None:
    print(f"[v22] Running T_r* rebuild audit at depth={depth} ...")
    t0 = time.perf_counter()
    rows = summarize_operator_from_spinH_core_v6_radial(depth=depth)
    elapsed = time.perf_counter() - t0

    print(f"[v22] Completed in {elapsed:.1f}s")
    print()
    header = f"{'metric':<50}  {'value':>20}  note"
    print(header)
    print("-" * len(header))
    for row in rows:
        print(f"{row['metric']:<50}  {str(row['value']):>20}  {row['note']}")

    write_Tr_rebuild_v1(rows)
    print(f"\n[v22] Wrote {OUTPUT_PATH_TR_REBUILD_V1}")


if __name__ == "__main__":
    main()
