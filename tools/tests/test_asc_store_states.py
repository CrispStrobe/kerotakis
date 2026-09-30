"""Which App Store version states the pipeline will edit.

This set is the whole reason `store.py` refused to touch the macOS version
after Apple rejected it: only PREPARE_FOR_SUBMISSION was listed, and a
rejected version is precisely the one you have to edit in order to answer the
rejection. Apple permits that edit; the script did not.

The exclusion matters as much as the inclusions, so both are pinned here.
"""
import importlib.util
import json
import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("asc_store", ROOT / "tools/asc/store.py")
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)


class EditableStateTests(unittest.TestCase):
    def test_a_rejected_version_is_editable(self):
        """The bug: Apple rejects, and the tooling then will not let you
        answer, because REJECTED was not on the list."""
        self.assertIn("REJECTED", MODULE.EDITABLE)

    def test_the_other_resubmittable_states_are_editable(self):
        for state in ("PREPARE_FOR_SUBMISSION", "DEVELOPER_REJECTED",
                      "METADATA_REJECTED", "INVALID_BINARY"):
            self.assertIn(state, MODULE.EDITABLE)

    def test_a_live_version_is_not_editable(self):
        """Deliberate, and not an oversight to be helpfully corrected: App
        Store Connect answers `409 STATE_ERROR: Attribute 'description' cannot
        be edited at this time` for a READY_FOR_SALE version. Shipping an
        update needs a new appStoreVersion, not an edit to the live one."""
        self.assertNotIn("READY_FOR_SALE", MODULE.EDITABLE)

    def test_states_in_review_are_not_editable(self):
        """A version with Apple is not ours to edit; it has to be withdrawn
        first, which is a decision rather than a step."""
        for state in ("WAITING_FOR_REVIEW", "IN_REVIEW", "PENDING_APPLE_RELEASE"):
            self.assertNotIn(state, MODULE.EDITABLE)


class ReviewNotesTests(unittest.TestCase):
    def test_the_notes_fit_app_store_connect(self):
        """4000 characters is the limit; a longer value is rejected on PATCH
        and the whole listing step fails with it."""
        notes = json.loads(
            (ROOT / "tools/asc/metadata.json").read_text(encoding="utf-8")
        )["review"]["notes"]
        self.assertLessEqual(len(notes), 4000)
        self.assertTrue(notes.strip())


if __name__ == "__main__":
    unittest.main()
