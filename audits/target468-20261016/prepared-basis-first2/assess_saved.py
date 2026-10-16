"""Assess frozen responses without executing chemistry or making API requests."""
import argparse
from collections import defaultdict
import hashlib
import json
from pathlib import Path
import subprocess


def sha(data):
    return hashlib.sha256(data).hexdigest()


def assess(design_path, results_dir):
    design = json.loads(design_path.read_text())
    source = design['engine']['source_head']
    registry_bytes = subprocess.check_output(
        ['git', 'show', source + ':data/registry/registry-source-v1.json'])
    registry = json.loads(registry_bytes)
    compositions = {c['species_id']: c for c in registry['compositions']}

    def inventory(vessel):
        totals = defaultdict(float)
        for portion in vessel['contents']:
            composition = compositions[portion['species']]
            for element in composition['elements']:
                totals[element['element']] += portion['moles'] * element['count']['value']
            totals['formal_charge'] += portion['moles'] * composition['net_charge']['value']
        return dict(totals)

    assessments = []
    for case in design['cases']:
        raw = (results_dir / case['id'] / 'response.body').read_bytes()
        result = json.loads(raw)
        assert result['engine'] == design['engine']
        assert result['script_sha256'] == case['script_sha256'] == sha(case['script'].encode())
        assert result['execution_status'] == 'completed' and result['exit_status'] == 0
        observations = result['observations']
        baseline = inventory(observations[2]['bench']['vessels'][0])
        final_vessel = observations[-1]['bench']['vessels'][0]
        final = inventory(final_vessel)
        events = [(i, e) for i, o in enumerate(observations) for e in o.get('events', [])]
        checks = []

        def check(label, actual, expected, tolerance):
            checks.append({'quantity': label, 'actual': actual, 'expected': expected,
                           'absolute_tolerance': tolerance,
                           'met': abs(actual - expected) <= tolerance})

        def species_total(names):
            return sum(p['moles'] for p in final_vessel['contents'] if p['species'] in names)

        if case['original_rank'] == 118:
            check('retained permanganate mol', species_total({'KMnO4', 'MnO4-'}), 0, 1e-8)
            # Observation5 is the permanganate addition, not initial trapped-CO2 repartition.
            gas = [e for i, e in events if i == 5 and e['event'] in
                   ('gas_contained', 'gas_evolved') and e['species'] == 'CO2']
            assert len(gas) == 1
            check('reaction CO2 event mol', gas[0]['moles'], .01, 1e-8)
            check('retained added carbon mol atoms', final['C'] - baseline['C'], .01, 1e-8)
            if case['id'] == '118-complete-stocks':
                for element, expected in [('Mn', .002), ('K', .002), ('S', .01)]:
                    check('retained ' + element + ' mol atoms', final[element], expected, 1e-8)
                ph = final_vessel['solution']['ph']
                checks.append({'quantity': 'final pH < 2', 'actual': ph, 'met': ph < 2})
        else:
            electrode = [e for _, e in events if e['event'] == 'electrolysed']
            assert len(electrode) == 1
            electrode = electrode[0]
            assert electrode['cathode_species'] == 'Cu' and electrode['anode_species'] == 'Cl2'
            check('electrons mol', electrode['electrons'], 1 / 96485.33212, 1e-10)
            check('cathode Cu mol', electrode['cathode_moles'], 1 / 96485.33212 / 2, 1e-9)
            check('anode Cl2 mol', electrode['anode_moles'], 1 / 96485.33212 / 2, 1e-9)
            check('retained Cu mol atoms including atacamite', final['Cu'], .001, 1e-8)
            check('retained Cl mol atoms including atacamite', final['Cl'], .002, 1e-8)

        failures = [e for _, e in events if e['event'] == 'solver_failed']
        check('solver_failed count', len(failures), 0, 0)
        feed = case.get('feed_mol_atoms_excluding_original_carrier_and_trapped_air',
                        case.get('feed_equivalence', {}).get('new_mol_atoms'))
        charge = case.get('feed_net_formal_charge_mol',
                          case.get('feed_equivalence', {}).get('new_mol_formal_charge'))
        residuals = {element: final.get(element, 0) - baseline.get(element, 0) - feed.get(element, 0)
                     for element in sorted(set(final) | set(baseline) | set(feed))
                     if element != 'formal_charge'}
        residuals['formal_charge'] = final.get('formal_charge', 0) - baseline.get('formal_charge', 0) - charge
        boundaries = [{'observation': i, 'event': e} for i, e in events
                      if e['event'] == 'not_yet_modeled']
        assessments.append({
            'id': case['id'], 'original_rank': case['original_rank'],
            'script_sha256': case['script_sha256'], 'response_sha256': sha(raw),
            'checks': checks, 'all_bounded_checks_met': all(c['met'] for c in checks),
            'baseline_observation': 2, 'final_observation': len(observations) - 1,
            'baseline_mol_atoms_and_charge': baseline, 'final_mol_atoms_and_charge': final,
            'feed_balance_residuals_mol': residuals,
            'all_material_and_formal_charge_residuals_below_1e-8':
                all(abs(r) <= 1e-8 for r in residuals.values()),
            'model_boundaries': boundaries,
            'verdict': 'bounded_forecast_met_with_declared_model_limits' if all(c['met'] for c in checks)
                       else 'bounded_forecast_not_met',
            'catalog_selection': 'pending_separate_review', 'original_case_closed': False,
        })
    return {'design_sha256': sha(design_path.read_bytes()), 'engine': design['engine'],
            'registry_sha256': sha(registry_bytes), 'cases': assessments,
            'method': 'Saved JSON only; exact-source registry composition, all retained phases; no engine execution.',
            'qualification': 'Bounded model consistency, not empirical kinetics, heat, yield or closure of missing original identities.'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--design', type=Path, required=True)
    parser.add_argument('--results', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    report = assess(args.design, args.results)
    with args.out.open('x') as handle:
        json.dump(report, handle, indent=2)
        handle.write('\n')
