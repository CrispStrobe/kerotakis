#!/usr/bin/env python3
"""Unit checks for the development-only HFO oracle."""

import importlib.util
from pathlib import Path
import unittest


SCRIPT = Path(__file__).parents[1] / "surface-oracle.py"
SPEC = importlib.util.spec_from_file_location("surface_oracle", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
ORACLE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ORACLE)


class SurfaceOracleTests(unittest.TestCase):
    def test_intrinsic_model_has_a_finite_monotonic_ph_edge(self):
        values = [ORACLE.oracle_bound_zinc(ph, 1.0e-4) for ph in (2.0, 4.0, 6.0, 8.0)]
        self.assertTrue(all(0.0 <= value <= 1.0e-4 for value in values))
        self.assertTrue(all(right > left for left, right in zip(values, values[1:])))
        self.assertLess(values[0], 0.01e-4)
        self.assertGreater(values[-1], 0.90e-4)

    def test_summary_persists_aggregates_not_case_output(self):
        cases = []
        for ph in (2.0, 4.0, 6.0):
            bound = ORACLE.oracle_bound_zinc(ph, 1.0e-4)
            cases.append(
                {"id": f"ph-{ph}", "ph": ph, "bound_zinc_mol": bound, "total_zinc_mol": 1.0e-4}
            )
        document = {
            "schema": 1,
            "benchmark": "hfo-zinc-ph-edge-v1",
            "producer": {"name": "synthetic-test", "version": "unit"},
            "retrieved": "2026-08-21",
            "cases": cases,
        }
        summary = ORACLE.summarize(document, "a" * 64)
        self.assertEqual(summary["metrics"]["max_absolute_bound_fraction_error"], 0.0)
        self.assertTrue(summary["metrics"]["monotonic_agreement"])
        self.assertNotIn("cases", summary)

    def test_invalid_inventory_is_refused(self):
        document = {
            "schema": 1,
            "benchmark": "hfo-zinc-ph-edge-v1",
            "producer": {"name": "bad", "version": "unit"},
            "retrieved": "2026-08-21",
            "cases": [
                {"id": str(index), "ph": 5.0 + index, "bound_zinc_mol": 2.0e-4, "total_zinc_mol": 1.0e-4}
                for index in range(3)
            ],
        }
        with self.assertRaisesRegex(ORACLE.InputError, "0 <= bound_zinc_mol"):
            ORACLE.summarize(document, "a" * 64)


if __name__ == "__main__":
    unittest.main()
