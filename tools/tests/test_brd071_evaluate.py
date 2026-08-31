import importlib.util
import pathlib
import tempfile
import unittest


MODULE_PATH = pathlib.Path(__file__).parents[1] / "brd071_evaluate.py"
SPEC = importlib.util.spec_from_file_location("brd071_evaluate", MODULE_PATH)
assert SPEC and SPEC.loader
evaluation = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(evaluation)


class Brd071EvaluationTests(unittest.TestCase):
    def test_probe_parser_is_strict(self):
        digest = "ab" * 32
        self.assertEqual(
            evaluation.parse_probe(f'{{"trace_sha256":"{digest}","step_ms":[1,2.5]}}'),
            {"trace_sha256": digest, "step_ms": [1.0, 2.5]},
        )
        for invalid in ["[]", "{}", '{"trace_sha256":"xx","step_ms":[1]}',
                        f'{{"trace_sha256":"{digest}","step_ms":[]}}',
                        f'{{"trace_sha256":"{digest}","step_ms":[-1]}}']:
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                evaluation.parse_probe(invalid)

    def test_nearest_rank_p95_does_not_interpolate_away_a_slow_step(self):
        self.assertEqual(evaluation.percentile_nearest_rank(list(range(1, 21)), 0.95), 19)

    def test_probe_decision_separates_determinism_from_advisory_timing(self):
        digest = "01" * 32
        good = evaluation.probe_decision([
            {"trace_sha256": digest, "step_ms": [1.0, 2.0]},
            {"trace_sha256": digest, "step_ms": [1.0, 20.0]},
        ], [30.0, 31.0])
        self.assertTrue(good["deterministic"])
        self.assertFalse(good["performance_pass"])
        self.assertTrue(good["timing_is_advisory"])
        bad = evaluation.probe_decision([
            {"trace_sha256": digest, "step_ms": [1.0]},
            {"trace_sha256": "02" * 32, "step_ms": [1.0]},
        ], [1.0, 1.0])
        self.assertFalse(bad["deterministic"])

    def test_payload_budget_uses_compressed_delta(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            baseline = root / "before.wasm"
            candidate = root / "after.wasm"
            baseline.write_bytes(b"a" * 4096)
            candidate.write_bytes(b"a" * 4096 + b"b" * 4096)
            decision = evaluation.payload_decision(baseline, candidate)
            self.assertTrue(decision["pass"])
            self.assertEqual(decision["raw_delta_bytes"], 4096)
            self.assertLess(decision["gzip_delta_bytes"], 4096)

            evaluation.MAX_WASM_GZIP_DELTA = -1
            self.assertFalse(evaluation.payload_decision(baseline, candidate)["pass"])


if __name__ == "__main__":
    unittest.main()
