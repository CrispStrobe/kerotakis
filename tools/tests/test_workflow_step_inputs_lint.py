"""The bug this lint exists for shipped, so the test is that exact shape."""
import importlib.util
import pathlib
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "workflow_step_inputs_lint", ROOT / "tools/workflow-step-inputs-lint.py"
)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)


def findings(body: str) -> list[str]:
    with tempfile.TemporaryDirectory() as d:
        path = pathlib.Path(d) / "sample.yml"
        path.write_text(body, encoding="utf-8")
        return MODULE.findings(path)


NODE_WITH_PATH = """
jobs:
  build:
    steps:
      - uses: actions/setup-node@v6
        with:
          node-version: "22"
          cache: npm
          cache-dependency-path: web/app/package-lock.json
"""

NODE_WITHOUT_PATH = """
jobs:
  build:
    steps:
      - uses: actions/setup-node@v6
        with:
          node-version: "22"
          cache: npm
"""


class WorkflowStepInputsLintTests(unittest.TestCase):
    def test_a_cache_without_a_dependency_path_is_a_finding(self):
        """What shipped: an inserted step landed between `cache: npm` and the
        `cache-dependency-path` below it, and the App Store upload for v0.1.1
        failed in setup-node."""
        rows = findings(NODE_WITHOUT_PATH)
        self.assertEqual(len(rows), 1)
        self.assertIn("cache-dependency-path", rows[0])

    def test_a_cache_with_its_path_is_fine(self):
        self.assertEqual(findings(NODE_WITH_PATH), [])

    def test_no_cache_at_all_is_fine(self):
        self.assertEqual(findings("""
jobs:
  build:
    steps:
      - uses: actions/setup-node@v6
        with:
          node-version: "22"
"""), [])

    def test_an_input_belonging_to_another_action_is_named(self):
        rows = findings("""
jobs:
  build:
    steps:
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"
          node-version: "22"
""")
        self.assertEqual(len(rows), 1)
        self.assertIn("setup-node", rows[0])

    def test_steps_without_uses_are_skipped(self):
        self.assertEqual(findings("""
jobs:
  build:
    steps:
      - name: a run step
        run: echo hi
"""), [])

    def test_the_repository_is_clean(self):
        rows = [r for p in sorted(MODULE.WORKFLOWS.glob("*.yml"))
                for r in MODULE.findings(p)]
        self.assertEqual(rows, [])


if __name__ == "__main__":
    unittest.main()
