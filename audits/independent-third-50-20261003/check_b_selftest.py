"""Lightweight mutation tests for evidence qualification versus corruption.
Run with Python; no chemistry executable or build is invoked.
"""
import copy
import unittest
import check_b


def fixture(case):
    """Small valid JSON-row shape for measurement invariance/unit checks."""
    instruments={'temp':'thermometer','mass':'balance','ph':'ph_meter',
                 'pressure':'pressure_gauge','volume':'volume_meter'}
    values={'thermometer':25.,'balance':99.7,'pressure_gauge':103.804,
            'ph_meter':7.,'volume_meter':1.}
    units={'thermometer':'°C','balance':'g','pressure_gauge':'kPa',
           'ph_meter':'pH','volume_meter':'L'}
    rows=[]
    for line in check_b._script(case).splitlines():
        words=line.split()
        if not words or words[0]=='register':continue
        row={'operator':{'op':'new_vessel' if words[0]=='new' else words[0]},
             'events':[], 'bench':{'vessels':[{'id':i,'temperature':298.15,
               'pressure':103804.,'contents':[{'species':'water','phase':'liquid','moles':5.534276991396059}]} for i in (0,1)]}}
        if words[0]=='measure':
            vessel=int(words[1][1:])-1;inst=instruments.get(words[2],words[2])
            row['operator'].update(vessel=vessel,instrument=inst)
            if inst=='volume_meter':
                row['events']=[{'event':'not_yet_modeled','vessel':vessel,
                    'cause':'boundary-mismatch','reason':{'key':'not-modeled.volume-meter-on-an-open-vessel'},
                    'what':'Volume meter reads headspace; sample is open.'}]
            else:
                row['events']=[{'event':'measured','vessel':vessel,'instrument':inst,
                    'value':values[inst],'unit':units[inst],
                    'model_support':{'status':'computed','reasons':[]}}]
        rows.append(row)
    return rows


def failed(results):
    return [r for r in results if r['ok'] is False]


class EvidenceMutationTests(unittest.TestCase):
    def setUp(self):self.rows=fixture('E124')

    def test_complete_control_passes(self):
        self.assertFalse(failed(check_b.check_case('E124',self.rows)))

    def test_deleted_required_measurement_row_fails(self):
        rows=copy.deepcopy(self.rows)
        del rows[next(i for i,r in enumerate(rows) if r['operator']['op']=='measure')]
        self.assertTrue(failed(check_b.check_case('E124',rows)))

    def test_deleted_required_measurement_event_fails(self):
        rows=copy.deepcopy(self.rows)
        next(r for r in rows if r['operator']['op']=='measure')['events']=[]
        self.assertTrue(failed(check_b.check_case('E124',rows)))

    def test_deleted_required_inspect_fails(self):
        rows=copy.deepcopy(self.rows)
        del rows[next(i for i,r in enumerate(rows) if r['operator']['op']=='inspect')]
        self.assertTrue(failed(check_b.check_case('E124',rows)))

    def test_corrupt_numbers_fail(self):
        for bad in ('25',float('nan'),float('inf'),True):
            with self.subTest(value=bad):
                rows=copy.deepcopy(self.rows)
                next(r for r in rows if r['operator']['op']=='measure')['events'][0]['value']=bad
                self.assertTrue(failed(check_b.check_case('E124',rows)))

    def test_wrong_units_fail(self):
        rows=copy.deepcopy(self.rows)
        next(r for r in rows if r['operator']['op']=='measure')['events'][0]['unit']='Pa'
        self.assertTrue(failed(check_b.check_case('E124',rows)))

    def test_explicit_unsupported_volume_is_qualification(self):
        results=check_b.check_case('E125',fixture('E125'))
        self.assertFalse(failed(results))
        self.assertTrue(any(r['name']=='reported_capability_limits' and r['ok'] is None for r in results))
        self.assertFalse(any(r['name']=='liquid_volume_instrument' and r['ok'] is True for r in results))

    def test_missing_unsupported_volume_outcome_fails(self):
        rows=fixture('E125')
        next(r for r in rows if r['operator'].get('instrument')=='volume_meter')['events']=[]
        self.assertTrue(failed(check_b.check_case('E125',rows)))

    def test_missing_support_is_corrupt_evidence(self):
        rows=copy.deepcopy(self.rows)
        del next(r for r in rows if r['operator']['op']=='measure')['events'][0]['model_support']
        self.assertTrue(failed(check_b.check_case('E124',rows)))

    def test_vessel_reordering_does_not_change_result(self):
        rows=copy.deepcopy(self.rows)
        for row in rows:row['bench']['vessels'].reverse()
        self.assertFalse(failed(check_b.check_case('E124',rows)))


class RootEvidenceMutationTests(unittest.TestCase):
    def test_pure_cut_requires_the_actual_public_azeotrope_flag(self):
        from verify import pure_cut_not_azeotropic
        self.assertTrue(pure_cut_not_azeotropic([{'events':[{'event':'distilled','azeotropic':False}]}]))
        for events in [[],[{'event':'distilled'}],[{'event':'distilled','azeotropic':True}],[{'event':'distilled','azeotrope_limited':False}]]:
            self.assertFalse(pure_cut_not_azeotropic([{'events':events}]))

    def test_exponent_overflow_is_invalid_evidence(self):
        from verify import strict
        for text in ['{"moles":1e400}','{"temperature":NaN}','{"pressure":Infinity}']:
            with self.assertRaises(ValueError):strict(text)
        self.assertEqual(strict('{"moles":1e-12}')['moles'],1e-12)


if __name__=='__main__':unittest.main()
