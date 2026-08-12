#!/usr/bin/env python3
"""Run the T_y rebuild from spin_H_core_v6 and write the bounded audit CSV.

This runner generates:
    results/prime_transport_recursive_system/prime_transport_Ty_rebuild_v1.csv

Using geometry_native_operator_model_v25 where:
    - T_y:  composite_twist_component_v25     (rebuilt from spin_H_core_v6)
    - T_x:  composite_swap_component_v24      (from spin_H_core_v6; unchanged)
    - T_b:  torus_base_advance_component_v23  (from spin_H_core_v6; unchanged)
    - T_c:  coupled_torus_kick_component_v20  (from spin_H_core_v6; unchanged)
    - T_z': fiber_phase_lift_component_v21    (from spin_H_core_v6; unchanged)
    - T_r*: radial_transport_component_v22    (from spin_H_core_v6; unchanged)
    - hold: hold_component_v10 (unchanged)
"""

from __future__ import annotations

import sys
import time
from pathlib import Path

_root = Path(__file__).resolve().parent
if str(_root) not in sys.path:
    sys.path.insert(0, str(_root))

from geometry_native_operator_model_v25 import (
    OUTPUT_PATH_TY_REBUILD_V1,
    summarize_operator_from_spinH_core_v6_twist,
    write_Ty_rebuild_v1,
)


def main(depth: int = 8) -> None:
    print(f"[v25] Running T_y rebuild audit at depth={depth} ...")
    t0 = time.perf_counter()
    rows = summarize_operator_from_spinH_core_v6_twist(depth=depth)
    elapsed = time.perf_counter() - t0

    print(f"[v25] Completed in {elapsed:.1f}s")
    print()
    header = f"{'metric':<50}  {'value':>20}  note"
    print(header)
    print("-" * len(header))
    for row in rows:
        print(f"{row['metric']:<50}  {str(row['value']):>20}  {row['note']}")

    write_Ty_rebuild_v1(rows)
    print(f"\n[v25] Wrote {OUTPUT_PATH_TY_REBUILD_V1}")


if __name__ == "__main__":
    main()
