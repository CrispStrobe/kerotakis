"""Read authenticated saved KI005; no engine, network, build or catalogue write."""
import argparse
import copy
from decimal import Decimal as D
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]


def load(path):
    return json.loads(path.read_text())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def amount(vessel, species):
    return sum((D(str(p['moles'])) for p in vessel['contents']
                if p['species'] == species), D(0))


def close(actual, expected, tolerance=D('1e-10')):
    assert abs(actual - expected) <= tolerance, (str(actual), str(expected))


def authenticate():
    receipt = load(HERE / 'request-receipt.json')
    original = load(HERE / 'original.json')
    scope = load(HERE / 'frozen-review-scope.json')
    raw = (HERE / 'response.json').read_bytes()
    assert sha(raw) == receipt['case']['response_body_sha256']
    assert sha(original['script'].encode()) == receipt['case']['original_sha256']
    assert (HERE / 'original.lab').read_text() == original['script']
    assert scope['original_criteria'] == original['acceptance_checks']
    assert scope['forecast'] == original['expected_result']
    for field in ['manifest', 'original_corpus']:
        assert sha((REPO / receipt[field + '_path']).read_bytes()) == receipt[field + '_sha256']
    manifest = load(REPO / receipt['manifest_path'])
    assert receipt['case'] in manifest['cases'] and receipt['deployed'] == manifest['deployed']
    corpus = load(REPO / receipt['original_corpus_path'])
    assert original in corpus['experiments']
    body = json.loads(raw)
    assert body['script_sha256'] == receipt['case']['original_sha256']
    assert body['engine'] == {k: receipt['deployed'][k] for k in body['engine']}
    binding = load(HERE / 'source-binding.json')
    assert binding['source'] == body['engine']['source_head']
    for path, row in binding['files'].items():
        full = subprocess.check_output(['git', 'show', binding['source'] + ':' + path], cwd=REPO)
        assert sha(full) == row['full_sha256']
        lines = full.decode().splitlines()
        expected = ''.join(f'{i}: {lines[i-1]}\n' for lo, hi in row['line_ranges']
                           for i in range(lo, hi + 1))
        assert (HERE / row['excerpt']).read_text() == expected
    composition = load(HERE / 'source-compositions.json')
    reg = subprocess.check_output(['git', 'show', binding['source'] + ':' + composition['registry_path']], cwd=REPO)
    assert sha(reg) == composition['registry_sha256']
    assert composition['compositions'] == {x['species_id']: x for x in json.loads(reg)['compositions']
                                         if x['species_id'] in ['water', 'H2O2', 'O2', 'catalase']}
    return body, scope


def evaluate(body):
    assert body['exit_status'] == 0 and body['execution_status'] == 'completed'
    assert body['diagnostics'] == ''
    frames = body['observations']
    assert len(frames) == 9
    assert [f['operator']['op'] for f in frames] == [
        'add', 'add', 'add', 'wait', 'new_vessel', 'decant', 'inspect', 'wait', 'inspect']
    assert frames[3]['operator']['seconds'] == 5 and frames[7]['operator']['seconds'] == 20
    assert frames[5]['operator'] == {'op': 'decant', 'from': 0, 'to': 1, 'fraction': 0.25}
    compositions = load(HERE / 'source-compositions.json')['compositions']
    budget = dict(H=D(0), O=D(0), charge=D(0))
    opaque_expected = D(0)
    max_residual = D(0)
    ledgers = []
    for index, frame in enumerate(frames):
        assert all(e['event'] not in ['solver_failed', 'not_modelled', 'material_lost'] for e in frame['events'])
        op = frame['operator']
        if op['op'] == 'add':
            n = D(str(op['moles']))
            if op['species'] == 'catalase':
                opaque_expected += n
            else:
                comp = compositions[op['species']]
                for e in comp['elements']:
                    budget[e['element']] += n * D(str(e['count']['value']))
                budget['charge'] += n * D(str(comp['net_charge']['value']))
        totals = dict(H=D(0), O=D(0), charge=D(0))
        opaque = D(0)
        for vessel in frame['bench']['vessels']:
            assert vessel['pending_co2_transfer_mol'] == 0 and vessel['solute_charge'] == 0
            assert len({p['species'] for p in vessel['contents']}) == len(vessel['contents'])
            for portion in vessel['contents']:
                species = portion['species']
                assert species in compositions
                assert portion['phase'] == {'water': 'liquid', 'H2O2': 'liquid',
                                            'catalase': 'aqueous', 'O2': 'gas'}[species]
                n = D(str(portion['moles']))
                assert n >= 0
                if species == 'catalase':
                    opaque += n
                    continue
                comp = compositions[species]
                for e in comp['elements']:
                    totals[e['element']] += n * D(str(e['count']['value']))
                totals['charge'] += n * D(str(comp['net_charge']['value']))
        residuals = {k: abs(totals[k] - budget[k]) for k in budget}
        for residual in residuals.values():
            close(residual, D(0))
            max_residual = max(max_residual, residual)
        # Catalase is an effective surrogate: its placeholder C formula is
        # not a real enzyme molecular formula. Preserve its separate pool.
        close(opaque, opaque_expected, D('1e-26'))
        ledgers.append({'frame': index, 'residuals': {k: str(v) for k, v in residuals.items()},
                        'opaque_catalase_mol': str(opaque)})
    pre = frames[4]['bench']['vessels'][0]
    split = frames[5]['bench']['vessels']
    end = frames[7]['bench']['vessels']
    for species in ['water', 'H2O2', 'catalase']:
        for vessel, fraction in zip(split, [D('.75'), D('.25')], strict=True):
            close(amount(vessel, species), amount(pre, species) * fraction,
                  max(D('1e-26'), abs(amount(pre, species)) * D('1e-14')))
    assert amount(split[0], 'O2') == amount(pre, 'O2') and amount(split[1], 'O2') == 0
    assert frames[5]['bench'] == frames[6]['bench']
    assert frames[7]['bench'] == frames[8]['bench']
    assert frames[7]['events'] == []
    for before, after in zip(split, end, strict=True):
        assert before['contents'] == after['contents']
        assert after['elapsed_seconds'] - before['elapsed_seconds'] == 20
    first = frames[3]
    oxygen = amount(first['bench']['vessels'][0], 'O2')
    gas = next(e for e in first['events'] if e['event'] == 'gas_produced')
    reacted = next(e for e in first['events'] if e['event'] == 'reacted')
    heat = next(e for e in first['events'] if e['event'] == 'reaction_heat_released')
    assert gas['reaction'] == reacted['reaction'] == heat['reaction'] == 'peroxide-decomposition'
    assert D(str(gas['moles'])) == D(str(reacted['moles'])) == oxygen
    close(D(str(heat['energy_j'])), oxygen * D('196400'), D('1e-10'))
    initial = amount(frames[2]['bench']['vessels'][0], 'H2O2')
    remaining = amount(pre, 'H2O2')
    close(initial - remaining, 2 * oxygen, D('1e-20'))
    # Pinned native source gate: DEPLETED=1e-12; DEPLETION_EVENT=10*DEPLETED;
    # direction_available requires strictly greater, RHS otherwise returns 0.
    floor = D('1e-11')
    start_amounts = [amount(v, 'H2O2') for v in split]
    assert remaining == floor and all(n <= floor for n in start_amounts)
    fractions = [(a - amount(v, 'H2O2')) / a for a, v in zip(start_amounts, end, strict=True)]
    assert fractions == [D(0), D(0)]
    return dict(qualified=False, criterion_results={
        'KI-005-C1': 'pass: proportional enzyme/substrate transfer; enzyme remains aqueous',
        'KI-005-C2': 'held: computed fractional losses are both zero below the consuming-rate gate; no active post-split rate comparison'},
        initial_peroxide_mol=str(initial), presplit_peroxide_mol=str(remaining),
        consumed_before_split_fraction=str((initial - remaining) / initial),
        post_split_peroxide_mol=[str(x) for x in start_amounts],
        post_split_fractional_losses=[str(x) for x in fractions], wait_seconds=20,
        consuming_rate_gate_mol=str(floor), post_split_consuming_gate_open=[n > floor for n in start_amounts],
        first_extent_mol=str(oxygen), first_reaction_heat_j=str(heat['energy_j']),
        max_known_H_O_charge_residual=str(max_residual), known_budget_checks=len(frames)*3,
        surrogate_formula_not_real_enzyme_formula=True, opaque_catalase_checkpoints=len(frames),
        all_ledgers=ledgers, forecast_confirmed=False, new_engine_executions=0,
        exact_blocker='Original waits exhaust active peroxide before representative aliquoting; both residual arms are below the unchanged consuming-rate gate. Saved zero/zero loss cannot identify fractional rate scaling.')


def mutation_controls(body):
    def portion(b, frame, vessel, species):
        return next(p for p in b['observations'][frame]['bench']['vessels'][vessel]['contents'] if p['species'] == species)
    def lose_opaque(b):
        portion(b, 5, 1, 'catalase')['moles'] = 0
    def insoluble_enzyme(b):
        portion(b, 5, 1, 'catalase')['phase'] = 'solid'
    def unequal_transfer(b):
        portion(b, 5, 0, 'H2O2')['moles'] = 8e-12
        portion(b, 5, 1, 'H2O2')['moles'] = 2e-12
    def lost_water(b):
        portion(b, 5, 1, 'water')['moles'] -= 0.001
    def lost_oxygen(b):
        portion(b, 3, 0, 'O2')['moles'] = 0
    def invented_heat(b):
        next(e for e in b['observations'][3]['events'] if e['event'] == 'reaction_heat_released')['energy_j'] = 0
    def invented_loss(b):
        portion(b, 7, 1, 'H2O2')['moles'] = 1e-12
    def invented_pending(b):
        b['observations'][7]['bench']['vessels'][1]['pending_co2_transfer_mol'] = 1e-12
    def solver_failure(b):
        b['observations'][7]['events'].append({'event': 'solver_failed'})
    def changed_wait(b):
        b['observations'][7]['operator']['seconds'] = 10
    def reactive_reserve(b):
        portion(b, 4, 0, 'H2O2')['moles'] = 1e-6
    def changed_inspect(b):
        portion(b, 8, 1, 'water')['moles'] += 0.001
    rejected = []
    for mutate in [lose_opaque, insoluble_enzyme, unequal_transfer, lost_water, lost_oxygen,
                   invented_heat, invented_loss, invented_pending, solver_failure, changed_wait,
                   reactive_reserve, changed_inspect]:
        changed = copy.deepcopy(body)
        mutate(changed)
        try:
            evaluate(changed)
        except AssertionError:
            rejected.append(mutate.__name__)
        else:
            raise AssertionError('Mutation survived: ' + mutate.__name__)
    return rejected


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    body, scope = authenticate()
    result = evaluate(body)
    result['mutations_rejected'] = mutation_controls(body)
    catalog = (REPO / scope['previous_catalog']).read_bytes()
    assert sha(catalog) == scope['previous_catalog_sha256']
    entries = json.loads(catalog)['experiments']
    assert len(entries) == 434 and not any(e.get('original_id') == 'batch2:ki-005' for e in entries)
    result['all_434_catalog_objects_unchanged'] = True
    with args.out.open('x') as f:
        json.dump(result, f, indent=2)
        f.write('\n')
    print(json.dumps({k: v for k, v in result.items() if k != 'all_ledgers'}))


if __name__ == '__main__':
    main()
