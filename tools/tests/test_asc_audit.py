"""Offline TestFlight readbacks: pagination, unknown states and tester privacy."""
import contextlib
import importlib.util
import io
import pathlib
import unittest
from unittest.mock import patch


ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("asc_audit", ROOT / "tools/asc/audit.py")
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)


class TestFlightAuditTests(unittest.TestCase):
    def test_collection_reads_all_pages(self):
        with patch.object(MODULE.client, "call", side_effect=[
            (200, {"data": [{"id": "one"}], "links": {"next": "/next"}}),
            (200, {"data": [{"id": "two"}], "links": {"next": None}}),
        ]) as call:
            self.assertEqual(MODULE.read_collection("/first"),
                             [{"id": "one"}, {"id": "two"}])
        self.assertEqual(call.call_args_list[1].args, ("GET", "/next"))

    def test_failed_later_page_discards_partial_count_and_private_errors(self):
        output = io.StringIO()
        with patch.object(MODULE.client, "call", side_effect=[
            (200, {"data": [{"id": "private-tester"}], "links": {"next": "/next"}}),
            (403, {"errors": [{"detail": "private@example.test"}]}),
        ]), contextlib.redirect_stdout(output):
            self.assertIsNone(MODULE.read_collection("/first"))
        self.assertIn("HTTP 403", output.getvalue())
        self.assertNotIn("private", output.getvalue())

    def run_audit(self, failed=False):
        def paged(path):
            if "/builds?" in path:
                return [{"id": "build-one", "attributes": {"version": "5"}}]
            if "/betaGroups?" in path:
                return [{"id": "group-one", "attributes": {"name": "Internal"}}]
            return []

        def call(method, path):
            self.assertEqual(method, "GET")
            if path.endswith("/preReleaseVersion"):
                return 200, {"data": {"id": "train-one", "attributes": {
                    "version": "0.1.5", "platform": "IOS"}}}
            if "/betaAppReviewSubmissions?" in path:
                if failed:
                    return 403, {"errors": [{"detail": "private@example.test"}]}
                return 200, {"data": [{"id": "review-one", "attributes": {
                    "betaReviewState": "WAITING_FOR_REVIEW"}}]}
            if "/relationships/builds?" in path:
                return 200, {"data": [{"id": "build-one"}]}
            if "/relationships/betaTesters?" in path:
                return 200, {"data": [{"id": "private-tester-one"},
                                      {"id": "private-tester-two"}]}
            self.fail(f"unexpected request: {path}")

        output = io.StringIO()
        with patch.object(MODULE.client, "expect", return_value={"data": {}}) as expect, \
                patch.object(MODULE.client, "paged", side_effect=paged), \
                patch.object(MODULE.client, "call", side_effect=call), \
                contextlib.redirect_stdout(output):
            result = MODULE.main()
        self.assertEqual(expect.call_args.args[0], "GET")
        return result, output.getvalue()

    def test_audit_reports_train_review_memberships_and_private_counts(self):
        result, output = self.run_audit()
        self.assertEqual(result, 0)
        for value in ("train-one", "0.1.5", "IOS", "WAITING_FOR_REVIEW",
                      "build ids ['build-one']", "tester count 2"):
            self.assertIn(value, output)
        self.assertNotIn("private-tester", output)

    def test_unavailable_review_is_incomplete_not_absent(self):
        result, output = self.run_audit(failed=True)
        self.assertEqual(result, 1)
        self.assertIn("beta review: unavailable", output)
        self.assertIn("Audit incomplete", output)
        self.assertNotIn("no submission returned", output)
        self.assertNotIn("private@example.test", output)


if __name__ == "__main__":
    unittest.main()
