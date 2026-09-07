import copy
import importlib.util
import json
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("step_prose", ROOT / "tools/step-prose.py")
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)

SOURCE = ROOT / "data/steps/step-prose-v1.json"


class StepProseTests(unittest.TestCase):
    def setUp(self):
        self.scripts = MODULE.codex_scripts()
        self.document = json.loads(SOURCE.read_text())
        self.german = json.loads((ROOT / "data/steps/step-prose-de-v1.json").read_text())

    def test_shipped_prose_aligns_with_every_script_it_paces(self):
        rows = MODULE.check_rows(self.document, self.scripts, "en")
        self.assertTrue(rows)
        for entry_id, sentences in rows.items():
            self.assertEqual(len(sentences), len(self.scripts[entry_id]), entry_id)

    def test_a_missing_entry_is_not_an_error(self):
        # Prose is optional at entry granularity: a catalogue with none is
        # the behaviour that shipped before this file existed.
        MODULE.check_rows({"schema": 1, "scripts": {}}, self.scripts, "en")

    def test_a_short_array_is_refused(self):
        entry_id = next(iter(self.document["scripts"]))
        broken = copy.deepcopy(self.document)
        broken["scripts"][entry_id] = broken["scripts"][entry_id][:-1]
        with self.assertRaisesRegex(ValueError, "must align with the script"):
            MODULE.check_rows(broken, self.scripts, "en")

    def test_a_long_array_is_refused(self):
        entry_id = next(iter(self.document["scripts"]))
        broken = copy.deepcopy(self.document)
        broken["scripts"][entry_id] = broken["scripts"][entry_id] + ["one too many"]
        with self.assertRaisesRegex(ValueError, "must align with the script"):
            MODULE.check_rows(broken, self.scripts, "en")

    def test_an_unknown_entry_is_refused(self):
        broken = {"schema": 1, "scripts": {"no-such-reaction": ["a sentence"]}}
        with self.assertRaisesRegex(ValueError, "not a codex entry"):
            MODULE.check_rows(broken, self.scripts, "en")

    def test_age_and_childhood_wording_is_refused_in_either_language(self):
        entry_id = next(iter(self.document["scripts"]))
        for offence in ("A grown-up pours it for children.", "Ab 8 Jahren geeignet."):
            broken = copy.deepcopy(self.document)
            broken["scripts"][entry_id] = [offence] * len(self.scripts[entry_id])
            with self.assertRaisesRegex(ValueError, "by age|as a child"):
                MODULE.check_rows(broken, self.scripts, "en")

    def test_the_german_file_covers_the_english_one_line_for_line(self):
        MODULE.check_rows(self.german, self.scripts, "de")
        merged = MODULE.add_translation(copy.deepcopy(self.document), self.german)
        for entry_id, row in merged["scripts"].items():
            self.assertEqual(len(row["say"]), len(row["say_de"]), entry_id)
            self.assertNotEqual(row["say"], row["say_de"], entry_id)

    def test_an_untranslated_entry_is_refused(self):
        thin = copy.deepcopy(self.german)
        thin["scripts"].pop(next(iter(thin["scripts"])))
        with self.assertRaisesRegex(ValueError, "missing de prose"):
            MODULE.add_translation(copy.deepcopy(self.document), thin)

    def test_every_script_the_runner_paces_says_what_to_watch_for(self):
        # The reason to pin this rather than leave prose optional in
        # practice: a script of three or more lines is one a learner will
        # step through, and stepping through it in silence is the gap this
        # data exists to close. A NEW entry with a long script therefore
        # arrives here as a request to write its sentences — which is a
        # cheaper reminder than a learner meeting the silence.
        paced = set(json.loads(SOURCE.read_text())["scripts"])
        long_scripts = {
            entry_id for entry_id, lines in self.scripts.items() if len(lines) >= 3
        }
        self.assertEqual(
            sorted(long_scripts - paced),
            [],
            "these scripts have three or more steps and no per-step prose; "
            "add one sentence per runnable line to data/steps/step-prose-v1.json "
            "and its German sibling",
        )

    def test_the_export_carries_both_languages(self):
        document = MODULE.build(SOURCE)
        row = document["scripts"]["vinegar-and-baking-soda"]
        self.assertEqual(len(row["say"]), 3)
        self.assertEqual(len(row["say_de"]), 3)


if __name__ == "__main__":
    unittest.main()
