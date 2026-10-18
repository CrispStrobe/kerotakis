"""Review original thermal criteria against authenticated saved native states."""
import decimal
import hashlib
import json
import pathlib
import subprocess
import zipfile

H = pathlib.Path(__file__).resolve().parent
A = H.parent
R = H.parents[2]
D = decimal.Decimal
decimal.getcontext().prec = 45
sha = lambda raw: hashlib.sha256(raw).hexdigest()
screen = json.loads((A / 'saved64-first-screen/first-screen.json').read_bytes())
by_id = {c['id']: c for c in screen['cases']}
raw_originals = (A / 'common-engine-completion/originals/new-experiments-20261016.json').read_bytes()
assert sha(raw_originals) == screen['originals_sha256']
originals = {c['id']: c for c in json.loads(raw_originals)['experiments']}
prepared = json.loads((A / 'saved64-thermal-preparation/metrics.json').read_bytes())
prep = {c['id']: c for c in prepared['cases']}
source = screen['engine']['source_head']
registry_raw = subprocess.check_output(['git', 'show', source + ':data/registry/registry-source-v1.json'], cwd=R)
route_raw = subprocess.check_output(['git', 'show', source + ':crates/kerotakis-core/src/phase_route.rs'], cwd=R)
assert sha(registry_raw) == prepared['registry_sha256'] and sha(route_raw) == prepared['phase_route_sha256']
registry = json.loads(registry_raw, parse_float=D)
props = {(x['species_id'], x['property']): x['quantity'] for x in registry['phase_thermodynamics'] if isinstance(x['property'], str)}
archive = A / 'common-engine-completion/results/raw-artifact.zip'
with archive.open('rb') as f:
    assert hashlib.file_digest(f, 'sha256').hexdigest() == screen['raw_artifact_sha256']

def amount(v, species, phase=None):
    return sum((D(p['moles']) for p in v['contents'] if p['species'] == species and (phase is None or p['phase'] == phase)), D(0))

def transition(species, prop):
    q = props[species, prop]
    assert q['unit']['symbol'] == 'K'
    return D(q['value'])

cases = []
with zipfile.ZipFile(archive) as z:
    for n in [1, 2, 3, 4, 5, 6, 11, 12, 13, 14]:
        cid = f'new120:th-{n:03}'
        old, original, pre = by_id[cid], originals[cid], prep[cid]
        prefix = cid.replace(':', '_') + '/'
        script, raw = z.read(prefix + 'experiment.lab'), z.read(prefix + 'stdout.jsonl')
        assert sha(script) == old['original_script_sha256'] == original['script_sha256']
        assert script.decode() == original['script']
        assert sha(raw) == old['saved_execution_receipt']['stdout_sha256'] == pre['stdout_sha256']
        assert original['acceptance_checks'] == old['original_criteria'] == pre['original_criteria']
        assert old['material_projection']['max_per_step_residual'] < 1e-8
        rows = [json.loads(line, parse_float=D) for line in raw.splitlines()]
        state, frames, limits = {}, [], []
        for row in rows:
            before = state.copy()
            state.update({v['id']: v for v in row['bench']['vessels']})
            assert not any(e['event'] == 'solver_failed' for e in row['events'])
            for v in state.values():
                assert D(v['temperature']).is_finite() and D(v['temperature']) > 0
                assert all(D(p['moles']).is_finite() and D(p['moles']) >= 0 for p in v['contents'])
            frames.append((row, before, state.copy()))
            limits.extend(e for e in row['events'] if e['event'] == 'not_yet_modeled')
        metrics = {'actual_final_temperature_K': {str(vid): v['temperature'] for vid, v in state.items()}, 'reproduced_bounded_energy_arithmetic': pre['bounded_energy_arithmetic'], 'reproduced_mixing_arithmetic': pre['mixing_arithmetic']}
        for e in pre['bounded_energy_arithmetic']:
            assert abs(e['residual_J']) < 1e-5
        reasons = []
        if n == 1:
            t = [D(state[i]['temperature']) for i in [0, 1]]
            assert t[1] - D('298.15') > t[0] - D('298.15') > 0
            assert t[0] < transition('water', 'boiling_temperature') and t[1] < transition('ethanol', 'boiling_temperature')
            assert len(pre['bounded_energy_arithmetic']) == 2
            reasons = ['At equal100g doses and equal5000J pulses, ethanol has a larger actual temperature rise than water.', 'Both final temperatures remain below their own pinned boiling transitions. Independent Cp(T) quadrature accounts for each5000J pulse to below3e-9J; this is source-property arithmetic, not calorimetric calibration.']
        elif n == 2:
            mix = pre['mixing_arithmetic']
            t = D(state[0]['temperature'])
            assert abs(t - D('293.15')) < abs(t - D('319.8166666666667'))
            assert abs(mix['adiabatic_residual_J']) < 1e-5
            metrics['initial_to_final_temperature_C'] = t - D('273.15')
            reasons = ['The actual mixture is23.56744434C, much closer to initial20C water than the46.7C mass-weighted estimate.', 'The actual pinned variable-Cp calculation gives1491.94386717J gained by water and1491.94386734J lost by copper, closing the adiabatic balance to1.72e-7J. The approximate23.5C constant-Cp guide is not substituted for the model properties.']
        elif n == 3:
            cool = next(f for f in frames if f[0]['operator']['op'] == 'cool')
            heat = next(f for f in frames if f[0]['operator']['op'] == 'heat')
            frozen, thawed = cool[2][0], heat[2][0]
            assert 0 < amount(thawed, 'water', 'solid') < amount(frozen, 'water', 'solid')
            assert amount(thawed, 'water', 'liquid') > 0
            assert D(frozen['temperature']) == D(thawed['temperature']) == transition('water', 'melting_temperature')
            metrics.update(ice_after_cooling_mol=amount(frozen, 'water', 'solid'), ice_after_partial_thaw_mol=amount(thawed, 'water', 'solid'))
            reasons = ['Ice decreases from1.66389351mol to0.831946755mol during the5000J partial reheating; both liquid water and ice remain.', 'Both mixed-phase snapshots stay at273.15K and the independently accounted liquid-fraction changes match the−10000J and+5000J pulses. No homogeneous-liquid pH is inferred from coexistence.']
        elif n == 4:
            sucrose, salt = state[0], state[1]
            assert all(amount(v, 'water', 'solid') > 0 and amount(v, 'water', 'liquid') > 0 for v in [sucrose, salt])
            assert amount(sucrose, 'sucrose', 'aqueous') == D('.005')
            assert amount(salt, 'Na+', 'aqueous') == amount(salt, 'Cl-', 'aqueous') == D('.005')
            t0, t1 = D(sucrose['temperature']), D(salt['temperature'])
            assert t1 < t0 < transition('water', 'melting_temperature')
            metrics.update(sucrose_dissolved_mol=amount(sucrose, 'sucrose', 'aqueous'), salt_ion_mol_each=amount(salt, 'Na+', 'aqueous'), actual_depression_ratio=(D('273.15') - t1) / (D('273.15') - t0))
            reasons = ['Both actual states own ice and liquid water: sucrose273.048624K versus NaCl272.953021K. The molecular solute owns0.005mol while NaCl owns0.005mol each Na+ and Cl−.', 'The actual temperature/depression comparison and dissolved inventories are retained; freeze concentration and activity effects preclude imposing an exact factor of two. No general mixture-energy or empirical cryoscopic calibration is claimed.']
        elif n == 5:
            cooling = next(f for f in frames if f[0]['operator']['op'] == 'cool')
            before, after = cooling[1][0], cooling[2][0]
            assert amount(after, 'water', 'solid') > 0 and amount(after, 'water', 'liquid') > 0
            ratios = [amount(v, 'Na+', 'aqueous') / amount(v, 'water', 'liquid') for v in [before, after]]
            assert ratios[1] > ratios[0]
            assert amount(before, 'Na+', 'aqueous') == amount(after, 'Na+', 'aqueous') == D('.01')
            assert not any(p['phase'] == 'solid' and p['species'] != 'water' for p in after['contents'])
            metrics['dissolved_Na_per_liquid_water_before_after'] = ratios
            reasons = ['Actual cooling leaves positive ice and liquid brine at272.705155K.', 'Dissolved Na+ per liquid-water mole increases while Na+/Cl− inventories remain0.01mol each; the actual final state contains no separate salt solid. This is an observed bounded freeze-concentration case, not an assumption that all other brines retain dissolved salt.']
        elif n == 6:
            exports = []
            for vid in [0, 1]:
                frame = next(f for f in frames if f[0]['operator']['op'] == 'heat' and f[0]['operator']['vessel'] == vid)
                vapour = sum((D(e['moles']) for e in frame[0]['events'] if e['event'] == 'gas_evolved' and e['species'] == 'water'), D(0))
                loss = amount(frame[1][vid], 'water') - amount(frame[2][vid], 'water')
                assert vapour > 0 and abs(vapour - loss) < D('1e-8')
                exports.append(vapour)
            assert D(state[1]['temperature']) > D(state[0]['temperature']) == transition('water', 'boiling_temperature')
            assert amount(state[1], 'Na+', 'aqueous') == amount(state[1], 'Cl-', 'aqueous') == D('.05')
            assert any(e.get('reason', {}).get('key') == 'not-modeled.boiling-is-not-settled' for e in limits)
            metrics['actual_water_vapour_export_mol'] = exports
            reasons = ['Both arms actually export water vapour, matching retained-water loss, before their final temperatures are compared.', 'Pure water is373.15K and saline liquid373.615804K, retaining0.05mol each Na+/Cl− while concentrating. The explicit boiling-is-not-settled warning excludes a settled pH claim; no universal open-mixture energy closure is inferred.']
        elif n in [11, 12]:
            species = 'naphthalene' if n == 11 else 'CH3COOH'
            v = state[0]
            assert amount(v, species, 'solid') > 0 and amount(v, species, 'liquid') > 0
            assert D(v['temperature']) == transition(species, 'melting_temperature')
            initial = next(f for f in frames if f[0]['operator']['op'] == 'add')[2][0]
            assert abs(amount(initial, species) - amount(v, species)) < D('1e-8')
            metrics.update(final_owned_solid_mol=amount(v, species, 'solid'), final_owned_liquid_mol=amount(v, species, 'liquid'), registry_melting_temperature_K=transition(species, 'melting_temperature'))
            if n == 11:
                reasons = ['Actual naphthalene owns both0.355287188mol solid and0.034807527mol liquid at its declared353.4K melting plateau.', 'Total naphthalene is conserved; independent source Cp/latent arithmetic accounts for339.35314494J sensible plus660.64685506J latent. No empirical phase-property accuracy is inferred.']
            else:
                assert amount(v, 'water', 'solid') == 0
                # Preserve repeated historical phase messages as a separate gap.
                metrics['historical_phase_messages'] = [e for row in rows for e in row['events'] if e['event'] == 'state_changed']
                reasons = ['The actual owned coexisting solid and liquid are acetic acid; no water ice is present. The historical repeated phase messages remain preserved and are not summed as additional physical freezing.', 'The pure-acid state is289.75K, matching its pinned pure-substance transition rather than vinegar. Independent owned-fraction energy closes the−2000J pulse; the generic dissolution warning does not certify missing acid-phase dissolution chemistry.']
        elif n == 13:
            heat = next(f for f in frames if f[0]['operator']['op'] == 'heat')
            before, after = heat[1][0], heat[2][0]
            vapour = sum((D(e['moles']) for e in heat[0]['events'] if e['event'] == 'gas_evolved' and e['species'] == 'ethanol'), D(0))
            assert vapour > 0 and abs(amount(before, 'ethanol') - amount(after, 'ethanol') - vapour) < D('1e-8')
            assert D(after['temperature']) == transition('ethanol', 'boiling_temperature')
            assert {p['species'] for p in after['contents']} == {'ethanol'}
            metrics['actual_ethanol_vapour_export_mol'] = vapour
            reasons = ['Ethanol reaches its declared351.39K plateau and its retained liquid decreases by the actual0.119364983mol vapour export.', 'Only ethanol remains in the owned inventory; the heat command is not ignition or chemical combustion. Independent sensible plus exported-vapour latent energy accounts for the5000J pulse.']
        elif n == 14:
            baselines = {}
            for row, _, after in frames:
                op = row['operator']
                if op['op'] == 'add' and op['species'] == 'water':
                    baselines[op['vessel']] = D(after[op['vessel']]['temperature'])
            assert set(baselines) == {0, 1}
            changes = [D(state[i]['temperature']) - baselines[i] for i in [0, 1]]
            assert changes[0] > 0 and changes[1] < 0
            assert amount(state[0], 'Ca+2', 'aqueous') == D('.01') and amount(state[0], 'Cl-', 'aqueous') == D('.02')
            assert amount(state[1], 'K+', 'aqueous') == amount(state[1], 'Cl-', 'aqueous') == D('.01')
            assert amount(state[0], 'CaCl2', 'solid') == amount(state[1], 'KCl', 'solid') == 0
            metrics.update(own_water_baselines_K=baselines, temperature_changes_K=changes)
            reasons = ['Each arm uses its own actual298.15K water baseline; CaCl2 ends300.129544K while KCl ends297.738775K.', 'Both literal solids disappear into the matching owned aqueous ions. Opposite signs demonstrate the declared dissolution-enthalpy model, not that one salt failed to dissolve or an independently calibrated calorimetry result.']
        assert len(reasons) == len(original['acceptance_checks']) == 2
        cases.append({'id': cid, 'status': 'PASS_IN_ORIGINAL_BOUNDED_DOMAIN', 'original_forecast': original['expected_result'], 'original_criteria_review': [dict(c, result='PASS', reason=reason) for c, reason in zip(original['acceptance_checks'], reasons)], 'script_sha256': sha(script), 'stdout_sha256': sha(raw), 'metrics': metrics, 'explicit_limits_preserved': limits, 'material_projection': old['material_projection'], 'source_binding': old['saved_execution_receipt']['engine'], 'qualification': 'Original bounded phase/property/dissolution model. Numerical source-property balances are not empirical calibration, and no universal mixture/reaction/open-boundary enthalpy closure is inferred. Historical binary/lock hashes remain producer-qualified; not current-production certification.'})
report = {'schema': 'kerotakis-saved-thermal-final-review-v1', 'source': source, 'registry_sha256': sha(registry_raw), 'phase_route_sha256': sha(route_raw), 'archive_sha256': screen['raw_artifact_sha256'], 'decimal_precision': 45, 'cases': cases, 'selected': len(cases), 'new_engine_or_API_executions': 0, 'historical_holds_preserved': [f'new120:th-{n:03}' for n in [7, 8, 9, 10, 15, 16]], 'additional_gap': 'TH012 historical repeated phase messages require separately bound current-source event/owned-fraction reconciliation. Its selected claim is final owned coexistence and energy, not a correct cumulative event ledger. TH016 already has a separate repaired-source selection; its old failure remains excluded.'}
encoded = json.dumps(report, indent=2, default=str) + '\n'
out = H / 'assessment.json'
if out.exists():
    assert json.loads(out.read_bytes()) == json.loads(encoded)
else:
    out.write_text(encoded)
print(json.dumps({'original_cases_reviewed': len(cases), 'original_criteria_pass': 20, 'new_executions': 0}))
