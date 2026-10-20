"""Synthetic agreement/schema checks, never real raw or browser acceptance."""
import copy
import unittest
from aggregate_saved import equal_reviews, validate_counts, SOURCE


class Agreement(unittest.TestCase):
    def test_wall_clock_only_is_ignored(self):
        equal_reviews({'utc': 'first', 'raw_elapsed': 1}, {'utc': 'second', 'raw_elapsed': 1})

    def test_raw_clock_drift_is_never_normalized(self):
        with self.assertRaises(ValueError):
            equal_reviews({'utc': 'first', 'raw_elapsed': 1}, {'utc': 'second', 'raw_elapsed': 2})

    def test_parent_hash_difference_requires_explicit_validated_edge(self):
        a, b = {'prior44_receipt_sha256': 'a'}, {'prior44_receipt_sha256': 'b'}
        with self.assertRaises(ValueError): equal_reviews(a, b)
        equal_reviews(a, b, ('prior44_receipt_sha256',))

    def test_physical_field_cannot_be_waived(self):
        with self.assertRaises(ValueError):
            equal_reviews({'heat_j': 1}, {'heat_j': 2}, ('heat_j',))

    def test_validated_parent_links_do_not_hide_metric_drift(self):
        with self.assertRaises(ValueError):
            equal_reviews({'prior44_receipt_sha256': 'a', 'heat_j': 1},
                          {'prior44_receipt_sha256': 'b', 'heat_j': 2},
                          ('prior44_receipt_sha256',))

    def fixture(self):
        receipts = {'browser37': {'unchanged_checks': 37},
                    'targeted7': {'unchanged_API_controls': 11,
                                 'unchanged_browser_controls': 37,
                                 'fresh_targeted_browser_controls': 7}}
        locale = {'API_controls': 11, 'unchanged_browser_controls': 37,
                  'prior_targeted_browser_controls': 7,
                  'supplemental_locale_browser_controls': 18,
                  'all62_browser_checks_accepted': True}
        heat = {'state': 'SAVED_HEAT_PREDICATES_ACCEPTED', 'source': SOURCE,
                'reviewed_candidate_source': '18f2bebd6f9aafe85d3ba1d76e2c2f9cd5d11e9a',
                'prior62_acceptance_replaced': False, 'scientific_selections': 0,
                'full_thermal_ledger_or_rate_calibration_claim': False,
                'details': {'de': {'synthetic_schema_only': True},
                            'fr': {'synthetic_schema_only': True}}}
        return receipts, locale, heat

    def test_schema_only_positive_is_not_actual_acceptance(self):
        validate_counts(*self.fixture())

    def test_missing_retained_locale_case(self):
        a, b, c = self.fixture(); b['supplemental_locale_browser_controls'] = 17
        with self.assertRaises(ValueError): validate_counts(a, b, c)

    def test_missing_heat_locale(self):
        a, b, c = self.fixture(); del c['details']['fr']
        with self.assertRaises(ValueError): validate_counts(a, b, c)

    def test_unsupported_calibration_claim(self):
        a, b, c = self.fixture(); c['full_thermal_ledger_or_rate_calibration_claim'] = True
        with self.assertRaises(ValueError): validate_counts(a, b, c)

    def test_heat_never_replaces_retained62(self):
        a, b, c = self.fixture(); c['prior62_acceptance_replaced'] = True
        with self.assertRaises(ValueError): validate_counts(a, b, c)


if __name__ == '__main__':
    unittest.main()
