"""Parse unfiltered native libtest output, including standard expected-panic labels.
This never executes tests or assigns scientific acceptance from forecasts.
"""
from __future__ import annotations
import re

CASE = re.compile(r'^test (\S+)(?: - should panic)? \.\.\. (ok|FAILED|ignored)$', re.M)
TOTAL = re.compile(r'^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;', re.M)

class InvalidReport(ValueError):
    pass

def parse_unfiltered(stdout: str, discovered: list[str]) -> dict:
    if len(discovered) != len(set(discovered)):
        raise InvalidReport('Duplicate discovery names')
    cases = CASE.findall(stdout)
    names = [name for name, _ in cases]
    if len(names) != len(set(names)) or set(names) != set(discovered):
        raise InvalidReport('Raw results do not account for every discovered test exactly once')
    totals = TOTAL.findall(stdout)
    if len(totals) != 1:
        raise InvalidReport('Missing or ambiguous target totals')
    status, *numbers = totals[0]
    passed, failed, ignored, measured, filtered = map(int, numbers)
    observed = {label: sum(result == label for _, result in cases) for label in ['ok', 'FAILED', 'ignored']}
    if (passed, failed, ignored) != (observed['ok'], observed['FAILED'], observed['ignored']):
        raise InvalidReport('Printed totals disagree with raw case results')
    if measured or filtered:
        raise InvalidReport('This component contract requires unfiltered nonbenchmark tests')
    if (status == 'ok') != (failed == 0):
        raise InvalidReport('Printed target status disagrees with failed cases')
    return {'cases': cases, 'passed': passed, 'failed': failed, 'ignored': ignored,
            'all_discovered_names_accounted': True,
            'all_assertions_pass': failed == ignored == 0}
