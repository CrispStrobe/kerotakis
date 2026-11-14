"""Corrupt saved evidence; the independent predicate must reject each case."""
import copy
import json
import unittest

import review_saved


class CorruptionControls(unittest.TestCase):
    def rows(self, name):
        return [json.loads(line) for line in
                (review_saved.HERE / (name + '.jsonl')).read_text().splitlines()]

    def rejected(self, name, rows):
        with self.assertRaises((AssertionError, KeyError)):
            review_saved.review(name, rows)

    def test_actual_evidence(self):
        for name in ('soda', 'calcite', 'lesson'):
            review_saved.review(name)

    def test_gas_material_and_typed_holds_are_required(self):
        for name in ('soda', 'calcite', 'lesson'):
            original = self.rows(name)
            for mutation in ('gas', 'carbon', 'charge', 'water', 'scope', 'hold', 'endpoint'):
                with self.subTest(name=name, mutation=mutation):
                    rows = copy.deepcopy(original)
                    target = 1 if name == 'lesson' else 0
                    if mutation == 'gas':
                        event = next(e for r in rows for e in r.get('events', [])
                                     if e['event'] == 'gas_evolved' and e['vessel'] == target)
                        event['moles'] += 1e-5
                    elif mutation in ('carbon', 'charge', 'water'):
                        species = {'carbon': 'CO2(aq)', 'charge': 'HCO3-', 'water': 'water'}[mutation]
                        # Corrupt the addition's returned state, not only an inspect copy.
                        for row in rows:
                            if row.get('operator', {}).get('op') in ('add', 'add_material'):
                                for vessel in row['bench']['vessels']:
                                    if vessel['id'] == target:
                                        portions = [p for p in vessel['contents'] if p['species'] == species]
                                        if portions:
                                            portions[0]['moles'] += 1e-5
                    elif mutation == 'endpoint':
                        for row in rows:
                            row['events'] = [e for e in row.get('events', []) if e['event'] != 'not_yet_modeled']
                    else:
                        vessel = next(v for v in rows[-1]['bench']['vessels'] if v['id'] == target)
                        if mutation == 'scope':
                            vessel['solution']['scope'] = 'solvent_only'
                        else:
                            vessel['solution']['provenance']['routing_phrase']['key'] = 'routing.default'
                    self.rejected(name, rows)

    def test_balance_is_not_a_blessable_snapshot(self):
        rows = self.rows('lesson')
        event = next(e for r in rows for e in r.get('events', [])
                     if e['event'] == 'measured' and e['vessel'] == 1)
        event['value'] = 53.47
        self.rejected('lesson', rows)


if __name__ == '__main__':
    unittest.main()
