import importlib.util
import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("curiosity_index", ROOT / "tools/curiosity-index.py")
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)


class CuriosityIndexTest(unittest.TestCase):
    def test_all_reviewed_prompts_are_browser_ready(self):
        doc = MODULE.build(ROOT / "tests/coverage/curiosity-v1")
        self.assertEqual(doc["schema"], 1)
        self.assertEqual(len(doc["prompts"]), 500)
        self.assertEqual(len({row["id"] for row in doc["prompts"]}), 500)
        self.assertTrue(all(row["question"] and row["support"] for row in doc["prompts"]))
        self.assertEqual(
            set(row["support"] for row in doc["prompts"]),
            {"computed", "curated", "qualitative", "boundary", "missing"},
        )


if __name__ == "__main__":
    unittest.main()
