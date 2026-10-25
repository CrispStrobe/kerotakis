"""Independent saved-row heat arithmetic for balloon-pressure, without engine helpers.

The chemical reference basis is the reviewed MINTEQ master-species basis.
The only phase-sensitive solid is the recipe's NaHCO3; the reaction and gas
enthalpies are read directly from the pinned database and the dissolution
enthalpy from the source registry. NASA-9 Cp polynomials are integrated here
from their printed coefficients. This does not run the simulator.
"""

from collections import defaultdict
from decimal import Decimal, localcontext
from pathlib import Path
import hashlib
import json
import re
import subprocess


import argparse

ARGS = argparse.ArgumentParser(description=__doc__)
ARGS.add_argument("--repo", required=True, type=Path, help="Git checkout with the bound source commit")
ARGS.add_argument("--database-dir", required=True, type=Path, help="Pinned vendor/iphreeqc/database directory")
OPTIONS = ARGS.parse_args()
SOURCE_REPO = OPTIONS.repo.resolve()
DB_ROOT = OPTIONS.database_dir.resolve()
OBSERVATIONS = Path(__file__).resolve().parent / "observations.jsonl"
assert hashlib.sha256(OBSERVATIONS.read_bytes()).hexdigest() == "ded5fc34b56798bebb5e003aafb7eb96230be4d92296e478899803dfb45b9ddd"
REGISTRY_SOURCE = '6471e8d056f92c9bfb3d550bfa68b14566f84880'
REGISTRY_SHA = '75c825b4d1725871604ec5f048fc1995a86efc66d3a25cc4adcc3aeeb88eb132'
MINTEQ_SHA = '4a48bf5c357b3da3084606a5e322426c3ab1e969dd1c86f9c62a3a7995836ca3'
WATEQ_SHA = '93547b0343d9f151e73fb48e7927aa9e9c777399fedcb8c7497d00371af4d0ae'
R = Decimal('8.31446261815324')


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def dec(value):
    return Decimal(str(value))


def database_delta_h(raw, equation):
    """Only accept one exact equation followed by one explicit kJ enthalpy."""
    lines = raw.decode().splitlines()
    positions = [i for i, line in enumerate(lines) if line.strip() == equation]
    assert len(positions) == 1, (equation, positions)
    block = '\n'.join(lines[positions[0] + 1:positions[0] + 3])
    values = re.findall(r'^\s*delta_h\s+(-?\d+(?:\.\d+)?)\s+kJ\s*$', block, re.M)
    assert len(values) == 1, (equation, block)
    return Decimal(values[0])


def nasa_integral(coefficients, lower, upper):
    a = [dec(x) for x in coefficients]

    def primitive(t):
        return (-a[0] / t + a[1] * t.ln() + a[2] * t
                + a[3] * t**2 / 2 + a[4] * t**3 / 3
                + a[5] * t**4 / 4 + a[6] * t**5 / 5)

    return R * (primitive(upper) - primitive(lower))


def cp_integral(registry, species, phase, lower, upper):
    curve = next((x for x in registry['heat_capacity_polynomials']
                  if x['species_id'] == species and x['phase'] == phase), None)
    if curve:
        assert curve['form'] == 'nasa9'
        assert len([x for x in curve['intervals']
                    if dec(x['t_min_k']) <= min(lower, upper)
                    and max(lower, upper) <= dec(x['t_max_k'])]) == 1
        interval = next(x for x in curve['intervals']
                        if dec(x['t_min_k']) <= min(lower, upper)
                        and max(lower, upper) <= dec(x['t_max_k']))
        return nasa_integral(interval['coefficients'], lower, upper)
    values = [x for x in registry['phase_thermodynamics']
              if x.get('species_id') == species and x.get('property') == 'molar_heat_capacity']
    assert len(values) == 1, (species, phase, values)
    return dec(values[0]['quantity']['value']) * (upper - lower)


def row_contents(vessel):
    stock = defaultdict(Decimal)
    for p in vessel['contents']:
        stock[(p['species'], p['phase'])] += dec(p['moles'])
    return stock


def chemical_heat(before, after, basis, gas_out=None):
    gas_out = gas_out or {}
    a, b = row_contents(before), row_contents(after)
    changes = []
    for key in sorted(set(a) | set(b) | set(gas_out)):
        if key[0] in ('H+', 'base_equivalents'):
            continue  # analytical basis coordinates; free OH is separate
        moved = b[key] - a[key] + dec(gas_out.get(key, 0))
        if abs(moved) > Decimal('1e-12'):
            assert key in basis, ('unpriced moved stock', key, moved)
            changes.append({'species': key[0], 'phase': key[1],
                            'delta_mol': str(moved),
                            'enthalpy_kj_per_mol': str(basis[key]),
                            'delta_h_j': str(moved * basis[key] * 1000)})
    delta_oh = dec(after['free_hydroxide']) - dec(before['free_hydroxide'])
    if abs(delta_oh) > Decimal('1e-12'):
        changes.append({'species': '__measured_free_OH', 'phase': 'aqueous',
                        'delta_mol': str(delta_oh),
                        'enthalpy_kj_per_mol': str(basis[('OH-', 'aqueous')]),
                        'delta_h_j': str(delta_oh * basis[('OH-', 'aqueous')] * 1000)})
    return -sum(Decimal(x['delta_h_j']) for x in changes), changes


def sensible_after(registry, vessel, lower, upper, sealed):
    terms = []
    for p in vessel['contents']:
        species, phase, n = p['species'], p['phase'], dec(p['moles'])
        if species == 'base_equivalents':
            continue  # bookkeeping basis has no registered Cp
        wanted = 'liquid' if phase == 'aqueous' else phase
        molar = cp_integral(registry, species, wanted, lower, upper)
        if phase == 'gas' and sealed:
            molar -= R * (upper - lower)
        terms.append({'species': species, 'phase': phase, 'moles': str(n),
                      'molar_sensible_j': str(molar), 'sensible_j': str(n * molar)})
    return sum(Decimal(x['sensible_j']) for x in terms), terms


def main():
    with localcontext() as ctx:
        ctx.prec = 55
        assert subprocess.check_output(['git', '-C', str(DB_ROOT.parent), 'rev-parse', 'HEAD']).decode().strip() == '47b459c4a5e3e20b9990fa304e207eca0dcc6ff1'
        gitlink = subprocess.check_output(['git', '-C', str(SOURCE_REPO), 'ls-tree', REGISTRY_SOURCE, 'vendor/iphreeqc']).decode()
        assert '160000 commit 47b459c4a5e3e20b9990fa304e207eca0dcc6ff1' in gitlink
        registry_raw = subprocess.check_output(['git', '-C', str(SOURCE_REPO), 'show', REGISTRY_SOURCE + ':data/registry/registry-source-v1.json'])
        assert sha(registry_raw) == REGISTRY_SHA
        registry = json.loads(registry_raw)
        minteq_raw = (DB_ROOT / 'minteq.v4.dat').read_bytes()
        wateq_raw = (DB_ROOT / 'wateq4f.dat').read_bytes()
        assert sha(minteq_raw) == MINTEQ_SHA and sha(wateq_raw) == WATEQ_SHA
        dh = {
            'bicarbonate': database_delta_h(minteq_raw, 'H+ + CO3-2 = HCO3-'),
            'hydrated_carbon': database_delta_h(minteq_raw, '2 H+ + CO3-2 = H2CO3'),
            'acetic_acid': database_delta_h(minteq_raw, 'H+ + Acetate- = H(Acetate)'),
            'hydroxide': database_delta_h(minteq_raw, 'H2O = OH- + H+'),
            'carbon_gas_dissolution': database_delta_h(minteq_raw, 'CO2 + H2O = 2 H+ + CO3-2'),
        }
        salt = [x for x in registry['phase_thermodynamics']
                if x.get('species_id') == 'NaHCO3' and x.get('property') == 'enthalpy_of_dissolution']
        assert len(salt) == 1
        dissolution = dec(salt[0]['quantity']['value'])
        assert dissolution == Decimal('16.7')
        basis = {
            ('water', 'liquid'): Decimal(0), ('N2', 'gas'): Decimal(0),
            ('O2', 'gas'): Decimal(0), ('Na+', 'aqueous'): Decimal(0),
            ('CO3-2', 'aqueous'): Decimal(0), ('CH3COO-', 'aqueous'): Decimal(0),
            ('HCO3-', 'aqueous'): dh['bicarbonate'],
            ('CO2(aq)', 'aqueous'): dh['hydrated_carbon'],
            ('CH3COOH', 'aqueous'): dh['acetic_acid'],
            ('CO2', 'gas'): -dh['carbon_gas_dissolution'],
            ('NaHCO3', 'solid'): dh['bicarbonate'] - dissolution,
            ('OH-', 'aqueous'): dh['hydroxide'],
        }
        rows = [json.loads(line) for line in OBSERVATIONS.read_text().splitlines()]
        assert len(rows) == 8 and [r['step'] for r in rows] == list(range(8))
        states = [r['bench']['vessels'][0] for r in rows]
        assert rows[2]['operator']['op'] == 'add_material'
        assert rows[5]['operator']['op'] == 'regulate'
        assert not [e for i in (2, 5) for e in rows[i]['events'] if e['event'] in ('gas_evolved', 'gas_absorbed')]
        added = next(x for x in rows[2]['events'] if x['event'] == 'material_added')
        n_salt = dec(next(x['moles'] for x in added['components'] if x['species'] == 'NaHCO3'))
        before2 = dict(states[1])
        before2['contents'] = states[1]['contents'] + [{'species': 'NaHCO3', 'phase': 'solid', 'moles': str(n_salt)}]
        before2['free_hydroxide'] = states[1]['free_hydroxide']
        results = []
        for start, end, before, sealed in ((1, 2, before2, True), (2, 5, states[2], False)):
            after = states[end]
            q, chemical_terms = chemical_heat(before, after, basis)
            if end == 2:
                prechem = next(e for e in rows[2]['events']
                               if e['event'] == 'temperature_changed')
                initial_t = dec(prechem['to'])
            else:
                initial_t = dec(states[start]['temperature'])
            high = dec(after['temperature'])
            reference = Decimal('298.15')
            before_reference, before_sensible_terms = sensible_after(
                registry, before, reference, initial_t, sealed)
            after_reference, after_sensible_terms = sensible_after(
                registry, after, reference, high, sealed)
            sensible = after_reference - before_reference
            results.append({'from_step': start, 'to_step': end,
                            'thermal_start_k': str(initial_t),
                            'temperature_delta_k': str(high - initial_t),
                            'chemical_heat_released_j': str(q),
                            'before_reference_sensible_j': str(before_reference),
                            'after_reference_sensible_j': str(after_reference),
                            'reference_sensible_change_j': str(sensible),
                            'energy_residual_chemical_minus_reference_sensible_j': str(q - sensible),
                            'chemical_terms': chemical_terms,
                            'before_sensible_terms': before_sensible_terms,
                            'after_sensible_terms': after_sensible_terms})
        state2 = states[2]
        state5 = states[5]
        initial_volume_l = dec(rows[5]['operator']['initial_volume'])
        final_volume_l = dec(state5['headspace']['volume'])
        regulated_pressure_pa = dec(rows[5]['operator']['pressure'])
        assert regulated_pressure_pa == Decimal('100000')
        constant_external_pressure_work_j = regulated_pressure_pa * (final_volume_l - initial_volume_l) / 1000
        ideal_pv = []
        for i in (2, 5):
            state = states[i]
            n_gas = sum(dec(p['moles']) for p in state['contents'] if p['phase'] == 'gas')
            volume_l = dec(state['headspace']['volume'])
            pressure = n_gas * R * dec(state['temperature']) / (volume_l / 1000)
            ideal_pv.append({'step': i, 'gas_mol': str(n_gas),
                             'computed_pressure_pa': str(pressure),
                             'saved_pressure_pa': str(state['pressure']),
                             'difference_pa': str(pressure - dec(state['pressure']))})
        assert all(abs(Decimal(row['energy_residual_chemical_minus_reference_sensible_j'])) <= Decimal('0.001') for row in results)
        assert all(abs(Decimal(row['difference_pa']) / Decimal(row['saved_pressure_pa'])) < Decimal('1e-5') for row in ideal_pv)
        report = {'source': REGISTRY_SOURCE,
                  'vendor_gitlink': '47b459c4a5e3e20b9990fa304e207eca0dcc6ff1',
                  'registry_sha256': REGISTRY_SHA,
                  'minteq_sha256': MINTEQ_SHA, 'wateq_sha256': WATEQ_SHA,
                  'saved_rows_sha256': sha(OBSERVATIONS.read_bytes()),
                  'salt_dissolution_enthalpy_kj_per_mol': str(dissolution),
                  'reaction_enthalpies_kj_per_mol': {k: str(v) for k, v in dh.items()},
                  'rows': results,
                  'ideal_headspace_pv': ideal_pv,
                  'illustrative_constant_external_pressure_expansion_work_j': str(constant_external_pressure_work_j),
                  'mechanical_work_status': 'HOLD: path/work exchange is absent from saved rows; the 1 bar product is illustrative, not an observed work ledger',
                  'execution': 'pure independent Python arithmetic from saved data and pinned source only'}
        print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
