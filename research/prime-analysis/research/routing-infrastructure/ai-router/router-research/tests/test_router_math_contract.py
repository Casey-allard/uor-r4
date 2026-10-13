import unittest
import sys
from pathlib import Path

_ROOT = Path(__file__).resolve().parents[1]
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

import router_math_contract as rmc


class TestRouterMathContract(unittest.TestCase):
    def test_fibonacci_values_match_legacy_expectation(self):
        self.assertEqual(rmc.fibonacci_values_upto(20), [1, 2, 3, 5, 8, 13])

    def test_normalize_4d_coordinate_matches_mudbench_contract(self):
        self.assertEqual(rmc.normalize_4d_coordinate((3.0, 4.0, 0.0, 0.0)), (0.6, 0.8, 0.0, 0.0))

    def test_allocate_pair_bins_scalar_matches_vendored_behavior(self):
        self.assertEqual(rmc.allocate_pair_bins_scalar(8, min_bins=1, ratio_scale=2.0), (6, 1))
        self.assertEqual(rmc.allocate_pair_bins_scalar(13, min_bins=2, ratio_scale=1.25), (5, 2))

    def test_allocate_triplet_bins_budget_matches_vendored_behavior(self):
        self.assertEqual(
            rmc.allocate_triplet_bins_budget(8, min_first=2, min_second=2, min_third=2),
            (2, 2, 2),
        )
        self.assertEqual(
            rmc.allocate_triplet_bins_budget(25, min_first=2, min_second=2, min_third=2),
            (3, 4, 2),
        )

    def test_hopf_coordinate_components_scalar_matches_expected_sample(self):
        coord = rmc.normalize_4d_coordinate((1.0, 0.0, 0.0, 1.0))
        components = rmc.hopf_coordinate_components_scalar(coord)
        self.assertAlmostEqual(components["chi"], 0.7853981633974484)
        self.assertAlmostEqual(components["chi_u"], 0.5000000000000001)
        self.assertAlmostEqual(components["delta"], -1.5707963267948966)
        self.assertAlmostEqual(components["alpha"], 0.7853981633974483)
        self.assertAlmostEqual(components["cos_chi"], 0.7071067811865476)
        self.assertAlmostEqual(components["sin_chi"], 0.7071067811865476)

    def test_hopf_phase_transport_components_scalar_matches_expected_sample(self):
        coord = rmc.normalize_4d_coordinate((1.0, 0.0, 0.0, 1.0))
        components = rmc.hopf_phase_transport_components_scalar(
            coord,
            phase_transport_lambda=1.0,
        )
        self.assertAlmostEqual(components["transport_connection_weight"], -8.040613248383183e-17)
        self.assertAlmostEqual(components["transport_phase_shift"], 0.0)
        self.assertAlmostEqual(components["transported_alpha"], 0.7853981633974483)

    def test_assign_sector_hopf_base_scalar_matches_vendored_behavior(self):
        coord = rmc.normalize_4d_coordinate((1.0, 0.0, 0.0, 1.0))
        sector_info = rmc.assign_sector_hopf_base_scalar(coord, K=8)
        self.assertEqual(sector_info["sector_id"], 5)
        self.assertEqual(
            sector_info["sector_bins"],
            {"chi_bins": 2, "delta_bins": 4, "chi_bin": 1, "delta_bin": 1},
        )

    def test_assign_sector_hopf_transport_scalar_matches_vendored_behavior(self):
        coord = rmc.normalize_4d_coordinate((1.0, 0.0, 0.0, 1.0))
        sector_info = rmc.assign_sector_hopf_transport_scalar(
            coord,
            K=8,
            phase_transport_lambda=1.0,
            hopf_chi_bins=2,
        )
        self.assertEqual(sector_info["sector_id"], 5)
        self.assertEqual(
            sector_info["sector_bins"],
            {
                "chi_bins": 2,
                "delta_bins": 2,
                "alpha_bins": 2,
                "chi_bin": 1,
                "delta_bin": 0,
                "alpha_bin": 1,
            },
        )


if __name__ == "__main__":
    unittest.main()
