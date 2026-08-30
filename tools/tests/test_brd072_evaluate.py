import importlib.util
import pathlib
import tempfile
import unittest


MODULE_PATH = pathlib.Path(__file__).parents[1] / "brd072_evaluate.py"
SPEC = importlib.util.spec_from_file_location("brd072_evaluate", MODULE_PATH)
assert SPEC and SPEC.loader
evaluation = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(evaluation)


def probe(trace="01", chemistry="02", after=None, frame_ms=None, phase=True):
    return {
        "visual_trace_sha256": trace * 32,
        "chemistry_before_sha256": chemistry * 32,
        "chemistry_after_loss_sha256": (after or chemistry) * 32,
        "frame_ms": frame_ms or [5.0, 10.0],
        "particles_before": 100,
        "particles_after": 80,
        "phase_order_matches": phase,
    }


class Brd072EvaluationTests(unittest.TestCase):
    def test_probe_parser_requires_real_particle_loss_and_strict_values(self):
        import json

        parsed = evaluation.parse_probe(json.dumps(probe()))
        self.assertEqual(parsed["particles_before"] - parsed["particles_after"], 20)
        invalid = probe()
        invalid["particles_after"] = 100
        with self.assertRaisesRegex(ValueError, "exercise particle loss"):
            evaluation.parse_probe(json.dumps(invalid))
        invalid = probe(frame_ms=[float("nan")])
        with self.assertRaisesRegex(ValueError, "finite"):
            evaluation.parse_probe(json.dumps(invalid))

    def test_authority_and_replay_fail_independently(self):
        authority_failure = evaluation.probe_decision([probe(after="03"), probe(after="03")])
        self.assertFalse(authority_failure["chemistry_unchanged_by_particle_loss"])
        self.assertTrue(authority_failure["visual_deterministic"])

        replay_failure = evaluation.probe_decision([probe(), probe(trace="03")])
        self.assertTrue(replay_failure["chemistry_unchanged_by_particle_loss"])
        self.assertFalse(replay_failure["visual_deterministic"])

    def test_explicit_60_and_30_fps_budgets_classify_p95(self):
        decision = evaluation.probe_decision(
            [probe(frame_ms=[20.0] * 19 + [40.0]), probe(frame_ms=[20.0] * 20)]
        )
        self.assertFalse(decision["fps_60"]["p95_pass"])
        self.assertTrue(decision["fps_30"]["p95_pass"])
        self.assertAlmostEqual(decision["fps_60"]["frame_budget_ms"], 1000 / 60)
        self.assertAlmostEqual(decision["fps_30"]["frame_budget_ms"], 1000 / 30)

    def test_payload_is_compared_to_named_lightweight_baseline(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            baseline, candidate = root / "fluid.wasm", root / "salva.wasm"
            baseline.write_bytes(b"a" * 4096)
            candidate.write_bytes(b"a" * 4096 + b"b" * 4096)
            result = evaluation.payload_decision(baseline, candidate)
            self.assertEqual(result["baseline"], "lightweight-fluidScene")
            self.assertEqual(result["raw_delta_bytes"], 4096)
            self.assertTrue(result["pass"])

    def test_report_schema_and_reference_timing_gate(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            baseline, candidate = root / "fluid.wasm", root / "salva.wasm"
            baseline.write_bytes(b"a")
            candidate.write_bytes(b"b")
            slow = [probe(frame_ms=[40.0]), probe(frame_ms=[40.0])]
            advisory = evaluation.build_report(slow, baseline, candidate)
            self.assertEqual(advisory["schema"], "kerotakis.brd072-decision-report.v1")
            self.assertTrue(advisory["timing_is_advisory"])
            self.assertEqual(advisory["decision"], "go")
            reference = evaluation.build_report(slow, baseline, candidate, "Pixel 8 / Chrome")
            self.assertFalse(reference["timing_is_advisory"])
            self.assertEqual(reference["decision"], "no-go")

    def test_phase_order_is_a_hard_gate(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "same.wasm"
            path.write_bytes(b"wasm")
            report = evaluation.build_report([probe(phase=False), probe(phase=False)], path, path)
            self.assertFalse(report["stable_gate_pass"])
            self.assertEqual(report["decision"], "no-go")


if __name__ == "__main__":
    unittest.main()
