"""The catalogue-prose gate holds itself to account (I18N-1).

Every rule here exists because breaking it renders confidently and wrongly
rather than visibly: a stale key describes chemistry that is gone, a
misaligned `options` list marks a different answer correct, and a missing
string is English prose on a page the reader asked for in German. None of
those show up as an error at runtime, so they have to show up here.
"""

import copy
import importlib.util
import pathlib
import tomllib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "codex_locale_lint", ROOT / "tools/codex-locale-lint.py"
)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)


class CatalogueProseTests(unittest.TestCase):
    def setUp(self):
        self.english = MODULE.english_paths()
        with open(ROOT / "codex/i18n/de.toml", "rb") as handle:
            self.de = tomllib.load(handle)

    # -- what the catalogue actually contains ---------------------------

    def test_the_models_half_is_translatable_at_all(self):
        """`fails_at` is the field a model exists for; it must be counted.

        It was not, for as long as this lint existed, and `models.toml`
        reported 100% German while its names, powers, phenomena and stated
        boundaries were all English.
        """
        for field in ("name", "power", "explains", "fails_at"):
            self.assertIn(field, MODULE.TRANSLATABLE, field)
        model_fields = {
            src.field for src in self.english.values() if src.file == "models.toml"
        }
        self.assertTrue({"name", "power", "explains", "fails_at"} <= model_fields)

    def test_german_ships_and_is_therefore_complete(self):
        self.assertIn("de", MODULE.COMPLETE)
        self.assertEqual(MODULE.audit("de", self.de, self.english), [])

    def test_a_new_entry_fails_until_it_is_translated(self):
        """The coverage pin. Adding chemistry adds an obligation."""
        english = dict(self.english)
        english["a-brand-new-experiment.registers.lv1"] = MODULE.Source(
            "aqueous.toml", "lv1", "The beaker turns blue."
        )
        found = MODULE.audit("de", self.de, english)
        self.assertTrue(any("a-brand-new-experiment" in f for f in found), found)

    def test_an_untranslated_language_is_reported_not_refused(self):
        """A translation in progress must be committable.

        One file per language is only useful if the first commit of that
        file can be five strings. A language absent from COMPLETE is
        measured, never blocked.
        """
        self.assertNotIn("fr", MODULE.COMPLETE)
        partial = dict(list(self.de.items())[:5])
        self.assertEqual(MODULE.audit("fr", partial, self.english), [])

    # -- the rules ------------------------------------------------------

    def test_a_key_translating_nothing_is_refused(self):
        broken = copy.deepcopy(self.de)
        broken["no-such-entry.registers.lv1"] = "Ein Gespenst."
        self.assertTrue(
            any("STALE" in f for f in MODULE.audit("de", broken, self.english))
        )

    def test_a_shortened_options_list_is_refused(self):
        """The answer is an INDEX into this list."""
        key = next(
            k
            for k, src in self.english.items()
            if src.field == "options" and k in self.de
        )
        broken = copy.deepcopy(self.de)
        broken[key] = broken[key][:-1]
        self.assertTrue(
            any("ALIGNMENT" in f for f in MODULE.audit("de", broken, self.english))
        )

    def test_a_list_translated_as_a_string_is_refused(self):
        key = next(
            k
            for k, src in self.english.items()
            if src.field == "options" and k in self.de
        )
        broken = copy.deepcopy(self.de)
        broken[key] = "drei Möglichkeiten, in einem Satz"
        self.assertTrue(
            any("SHAPE" in f for f in MODULE.audit("de", broken, self.english))
        )

    # -- wording (GUI-470), duplicated because this prose bypasses the
    #    locale bundles that learnerWording.test.ts walks ---------------

    def test_an_age_band_is_refused(self):
        for offence in (
            "Dieser Versuch ist ab 8 Jahren geeignet.",
            "Für die Altersgruppe 9–12 Jahre.",
            "Geeignet für 10-12 Jahren.",
            "Recommended for ages 8 and up.",
            "Suitable for children.",
            "Ein Kinderversuch mit Essig.",
            "Im Alter von zwölf beginnt das.",
        ):
            self.assertTrue(MODULE.wording_offences(offence), offence)

    def test_chemistry_that_merely_mentions_time_is_allowed(self):
        """The bundle regex forbids `Jahre` outright. Prose cannot.

        Each of these is in the shipped German today. A gate that failed
        them would be teaching translators to write worse sentences.
        """
        for allowed in (
            "So wird seit viertausend Jahren Mörtelkalk hergestellt.",
            "Wie zwei Kinder, die dasselbe Seil festhalten.",
            "In den 1990er Jahren von Sproul und Allen quantitativ gemacht.",
            "Dieser Test ist fast zweihundert Jahre alt.",
            "Diese fünfzig Jahre Lücke sind der klarste verfügbare Fall.",
        ):
            self.assertEqual(MODULE.wording_offences(allowed), [], allowed)

    def test_the_glossary_is_enforced(self):
        broken = copy.deepcopy(self.de)
        key = next(iter(broken))
        broken[key] = "Stelle das Becherglas auf die Werkbank."
        self.assertTrue(
            any("GLOSSARY" in f for f in MODULE.audit("de", broken, self.english))
        )


if __name__ == "__main__":
    unittest.main()
