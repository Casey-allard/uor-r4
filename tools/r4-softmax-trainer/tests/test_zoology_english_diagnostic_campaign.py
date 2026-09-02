"""Only the #1065 reproduction stop and resource-accounting decisions."""

import unittest
from unittest.mock import patch

from r4_softmax_trainer.zoology_english_diagnostic.campaign import (
    ResourceBudgetExceeded, _Budget, _reproduction,
)


class CampaignTests(unittest.TestCase):
    def test_reproduction_requires_every_metric_and_digest(self):
        expected = {"top1_correct": 2396, "selected_logits_cid": "frozen", "nll": 1.6}
        self.assertTrue(_reproduction(dict(expected), expected)["exact"])
        changed = {**expected, "selected_logits_cid": "changed"}
        self.assertFalse(_reproduction(changed, expected)["exact"])
        self.assertFalse(_reproduction({"top1_correct": 2396}, expected)["exact"])
        self.assertFalse(_reproduction({}, {"unknown_nll": None})["exact"])

    def test_budget_carries_run_into_fresh_replay(self):
        with patch("r4_softmax_trainer.zoology_english_diagnostic.campaign.time.monotonic", side_effect=[0.0, 2.0]):
            budget = _Budget(299.0)
            with self.assertRaises(ResourceBudgetExceeded):
                budget.check()
        with self.assertRaises(ValueError):
            _Budget(float("nan"))


if __name__ == "__main__":
    unittest.main()
