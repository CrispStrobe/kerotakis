import importlib.util
import json
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("kids_catalog", ROOT / "tools/kids-catalog.py")
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)


class KidsCatalogTests(unittest.TestCase):
    def setUp(self):
        self.document = json.loads((ROOT / "data/kids/experiments-v1.json").read_text())
        self.german = json.loads((ROOT / "data/kids/experiments-de-v1.json").read_text())

    def test_catalog_is_the_exact_audited_sixty(self):
        rows = MODULE.validate(self.document)
        self.assertEqual([row["id"] for row in rows], [f"K{i:02d}" for i in range(1, 61)])

    def test_non_computed_rows_explain_the_boundary(self):
        rows = MODULE.validate(self.document)
        self.assertTrue(all(row.get("boundary") for row in rows if row["status"] != "computed"))

    def test_every_launch_link_exists(self):
        rows = MODULE.validate(self.document)
        self.assertTrue(all((ROOT / "lessons" / row["lesson"]).is_file() for row in rows if row.get("lesson")))
        self.assertTrue(all(not row.get("lesson") and not row.get("quest") for row in rows if row["status"] in {"declined", "unreachable"}))

    def test_lemon_voltage_and_bone_boundary_are_named_honestly(self):
        rows = {row["id"]: row for row in MODULE.validate(self.document)}
        lemon = rows["K34"]
        self.assertEqual(lemon["status"], "computed")
        self.assertIn("Open-circuit", lemon["title"])
        self.assertIn("not a powered lemon battery", lemon["boundary"])
        self.assertIn("internal resistance", lemon["boundary"])
        bone = rows["K15"]
        self.assertEqual(bone["status"], "partial")
        self.assertEqual(bone["lesson"], "rubbery-bone-boundary.lab")
        self.assertIn("collagen", bone["boundary"])
        self.assertIn("calcium-phosphate", bone["boundary"])

    def test_cross_references_are_checked_against_their_sources(self):
        broken = json.loads(json.dumps(self.document))
        broken["experiments"][0]["capabilities"] = ["not-a-reviewed-prompt"]
        with self.assertRaisesRegex(ValueError, "capabilities must contain existing exact identifiers"):
            MODULE.validate(broken)

        broken = json.loads(json.dumps(self.document))
        broken["experiments"][0]["codex"] = ["not-a-codex-entry"]
        with self.assertRaisesRegex(ValueError, "codex must contain existing exact identifiers"):
            MODULE.validate(broken)

    def test_german_is_complete_and_merged_without_replacing_english(self):
        english = [row["title"] for row in self.document["experiments"]]
        merged = MODULE.add_translation(self.document, self.german)["experiments"]
        self.assertEqual([row["title"] for row in merged], english)
        self.assertTrue(all(row.get("title_de") and row.get("phenomenon_de") for row in merged))
        self.assertTrue(all(row.get("boundary_de") for row in merged if row.get("boundary")))

    def test_german_must_have_exactly_the_same_rows(self):
        broken = json.loads(json.dumps(self.german))
        broken["experiments"].pop()
        with self.assertRaisesRegex(ValueError, "same K01 through K60"):
            MODULE.add_translation(self.document, broken)

    def test_newly_computed_filter_and_luminol_keep_their_honest_routes(self):
        rows = {row["id"]: row for row in MODULE.validate(self.document)}
        self.assertEqual(rows["K33"]["status"], "computed")
        self.assertEqual(rows["K33"]["lesson"], "water-filter.lab")
        self.assertEqual(rows["K33"]["quest"], "water-filter")
        self.assertEqual(rows["K33"]["capabilities"], ["aq-071"])
        self.assertEqual(rows["K33"]["codex"], ["filtering-a-precipitate"])
        self.assertEqual(rows["K59"]["status"], "computed")
        self.assertIn("absolute photon yield", rows["K59"]["boundary"])


if __name__ == "__main__":
    unittest.main()
