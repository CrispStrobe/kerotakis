"""The curiosity corpus's translations, and what the validator refuses.

Two halves. The first pins the SHIPPED German: the explorer's whole point
is that a German reader sees German questions, and "500" is the number
that makes that true — a corpus that grows to 520 with 500 translations
must fail here rather than ship twenty English rows into a German dialog.

The second half is the more valuable one. Each refusal below stands for a
way a translation goes wrong quietly: a missing id reads as a gap nobody
can distinguish from an unanswered question, a stale key hides the rename
that orphaned it, a translated task identifier stops joining to its task,
an echoed question survives any review that only counts rows, and age
wording reaches a learner the level words exist to protect.
"""

import copy
import importlib.util
import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("curiosity_prose", ROOT / "tools/curiosity-prose.py")
PROSE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(PROSE)

INDEX_SPEC = importlib.util.spec_from_file_location(
    "curiosity_index", ROOT / "tools/curiosity-index.py"
)
INDEX = importlib.util.module_from_spec(INDEX_SPEC)
assert INDEX_SPEC.loader
INDEX_SPEC.loader.exec_module(INDEX)

CORPUS = ROOT / "tests/coverage/curiosity-v1"


def german() -> dict:
    import tomllib

    return tomllib.loads((CORPUS / "i18n/de.toml").read_text())


class ShippedGermanTest(unittest.TestCase):
    def test_german_covers_every_question_class_and_tag(self):
        corpus = PROSE.read_corpus(CORPUS)
        [de] = PROSE.read_translations(CORPUS)
        self.assertEqual(de["locale"], "de")
        self.assertEqual(len(de["question"]), 500)
        self.assertEqual(set(de["question"]), set(corpus.questions))
        self.assertEqual(set(de["material_class"]), corpus.material_classes)
        self.assertEqual(set(de["tag"]), corpus.tags)
        self.assertEqual(len(corpus.material_classes), 186)
        self.assertEqual(len(corpus.tags), 367)

    def test_task_identifiers_stay_names(self):
        corpus = PROSE.read_corpus(CORPUS)
        self.assertIn("CAP-5", corpus.identifier_tags)
        self.assertIn("EXP-17", corpus.identifier_tags)
        self.assertNotIn("CAP-5", corpus.tags)
        self.assertNotIn("CAP-5", german()["tag"])

    def test_the_browser_index_carries_the_german(self):
        prompts = INDEX.build(CORPUS)["prompts"]
        self.assertEqual(len(prompts), 500)
        self.assertTrue(all(row["question_de"] for row in prompts))
        self.assertTrue(all(row["material_class_de"] for row in prompts))
        # A tag list and its German twin index together, so the chip a
        # reader reads is the tag the row actually carries.
        for row in prompts:
            self.assertEqual(len(row["tags_de"]), len(row["tags"]))
        first = next(row for row in prompts if row["id"] == "aq-001")
        self.assertNotEqual(first["question_de"], first["question"])

    def test_a_task_identifier_tag_survives_the_fold_untranslated(self):
        prompts = INDEX.build(CORPUS)["prompts"]
        owned = next(
            row for row in prompts if any(PROSE.IDENTIFIER.match(t) for t in row["tags"])
        )
        for tag, twin in zip(owned["tags"], owned["tags_de"]):
            if PROSE.IDENTIFIER.match(tag):
                self.assertEqual(tag, twin)


class RefusalTest(unittest.TestCase):
    def setUp(self):
        self.corpus = PROSE.read_corpus(CORPUS)
        self.doc = german()

    def refuse(self, document, needle):
        with self.assertRaises(ValueError) as raised:
            PROSE.check(document, self.corpus, "de.toml")
        self.assertIn(needle, str(raised.exception))

    def test_the_shipped_file_passes(self):
        PROSE.check(self.doc, self.corpus, "de.toml")

    def test_a_missing_question_is_refused(self):
        doc = copy.deepcopy(self.doc)
        del doc["question"]["aq-001"]
        self.refuse(doc, "aq-001")

    def test_a_stale_key_is_refused(self):
        doc = copy.deepcopy(self.doc)
        doc["question"]["aq-999"] = "Was passiert hier?"
        self.refuse(doc, "aq-999")

    def test_a_missing_material_class_is_refused(self):
        doc = copy.deepcopy(self.doc)
        del doc["material_class"]["salt-water"]
        self.refuse(doc, "salt-water")

    def test_a_missing_tag_is_refused(self):
        doc = copy.deepcopy(self.doc)
        del doc["tag"]["dissolution"]
        self.refuse(doc, "dissolution")

    def test_a_translated_task_identifier_is_refused(self):
        doc = copy.deepcopy(self.doc)
        doc["tag"]["CAP-5"] = "Fähigkeit 5"
        self.refuse(doc, "CAP-5")

    def test_an_untranslated_echo_is_refused(self):
        doc = copy.deepcopy(self.doc)
        doc["question"]["aq-001"] = self.corpus.questions["aq-001"]
        self.refuse(doc, "byte-identical")

    def test_age_wording_is_refused(self):
        doc = copy.deepcopy(self.doc)
        doc["question"]["aq-001"] = "Was passiert, wenn Kinder Salz in Wasser rühren?"
        self.refuse(doc, "child")

    def test_a_blank_translation_is_refused(self):
        doc = copy.deepcopy(self.doc)
        doc["question"]["aq-001"] = "   "
        self.refuse(doc, "blank")

    def test_an_unsorted_table_is_refused(self):
        doc = copy.deepcopy(self.doc)
        doc["tag"] = dict(reversed(list(doc["tag"].items())))
        self.refuse(doc, "sorted")

    def test_english_cannot_be_a_translation(self):
        doc = copy.deepcopy(self.doc)
        doc["locale"] = "en"
        self.refuse(doc, "English is the source")

    def test_the_locale_must_match_the_filename(self):
        doc = copy.deepcopy(self.doc)
        doc["locale"] = "fr"
        self.refuse(doc, "does not match the filename")

    def test_an_unknown_section_is_refused(self):
        doc = copy.deepcopy(self.doc)
        doc["boundary"] = {"aq-001": "Grenze"}
        self.refuse(doc, "unknown section")


class ApplyTest(unittest.TestCase):
    """Folding degrades one field at a time, and never invents one."""

    def test_a_language_with_only_questions_leaves_the_rest_english(self):
        prompts = [
            {"id": "aq-001", "material_class": "salt-water", "tags": ["dissolution", "CAP-5"]}
        ]
        PROSE.apply(
            prompts,
            [{"locale": "fr", "question": {"aq-001": "Que se passe-t-il ?"}, "material_class": {}, "tag": {}}],
        )
        self.assertEqual(prompts[0]["question_fr"], "Que se passe-t-il ?")
        self.assertNotIn("material_class_fr", prompts[0])
        # No glossary at all still yields a positional twin: every tag is
        # its own fallback, which is exactly what an identifier needs.
        self.assertEqual(prompts[0]["tags_fr"], ["dissolution", "CAP-5"])

    def test_no_translations_changes_nothing(self):
        prompts = [{"id": "aq-001", "material_class": "salt-water", "tags": ["dissolution"]}]
        self.assertEqual(PROSE.apply(copy.deepcopy(prompts), []), prompts)


if __name__ == "__main__":
    unittest.main()
