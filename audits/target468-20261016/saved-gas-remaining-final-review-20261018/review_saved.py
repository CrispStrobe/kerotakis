"""Independent saved-state review; no simulator or network calls."""
import decimal
import hashlib
import json
import pathlib
import re
import subprocess
import zipfile

H = pathlib.Path(__file__).resolve().parent
A, R = H.parent, H.parents[2]
D = decimal.Decimal
decimal.getcontext().prec = 45
sha = lambda b: hashlib.sha256(b).hexdigest()
screen = json.loads((A / 'saved64-first-screen/first-screen.json').read_bytes())
old = {c['id']: c for c in screen['cases']}
original_raw = (A / 'common-engine-completion/originals/new-experiments-20261016.json').read_bytes()
assert sha(original_raw) == screen['originals_sha256']
originals = {c['id']: c for c in json.loads(original_raw)['experiments']}
source = screen['engine']['source_head']
get = lambda p: subprocess.check_output(['git', 'show', source + ':' + p], cwd=R)
gitlink = subprocess.check_output(['git', 'ls-tree', source, 'vendor/iphreeqc'], cwd=R, text=True).split()[2]
database = subprocess.check_output(['git', 'show', gitlink + ':database/wateq4f.dat'], cwd=R / 'vendor/iphreeqc')
phase = re.search(rb'^CO2\(g\)\r?\n.*?(?=^O2\(g\)\r?\n)', database, re.M | re.S)[0]
assert phase == (H / 'database-capability-source.txt').read_bytes()
assert b'-analytical 108.3865 0.01985076 -6919.53 -40.45154 669365' in phase
assert not any(k in phase.lower() for k in [b'-t_c', b'-p_c', b'-omega'])
vessel_source = get('crates/kerotakis-core/src/vessel.rs')
assert b'const R_LITRE_PASCAL: f64 = 8_314.462_618;' in vessel_source
registry = json.loads(get('data/registry/registry-source-v1.json'), parse_float=D)
formula = {c['species_id']: {**{a['element']: D(a['count']['value']) for a in c['elements']}, 'charge': D(c['net_charge']['value'])} for c in registry['compositions']}
formula['base_equivalents'] = formula['OH-'].copy()
archive = A / 'common-engine-completion/results/raw-artifact.zip'
with archive.open('rb') as f:
    assert hashlib.file_digest(f, 'sha256').hexdigest() == screen['raw_artifact_sha256']

def amount(v, species, phase=None):
    return sum((D(p['moles']) for p in v['contents'] if p['species'] == species and (phase is None or p['phase'] == phase)), D(0))

def carbon(v, gas=False):
    return sum((D(p['moles']) for p in v['contents'] if p['species'] in ['CO2', 'CO2(aq)', 'HCO3-', 'CO3-2'] and (p['phase'] == 'gas') == gas), D(0))

def ledger(v):
    out = {}
    for p in v['contents']:
        assert D(p['moles']).is_finite() and D(p['moles']) >= 0
        for k, n in formula[p['species']].items():
            out[k] = out.get(k, D(0)) + D(p['moles']) * n
    return out

def delta(a, b, additions):
    before, after = ledger(a), ledger(b)
    expected = {}
    for species, n in additions.items():
        for k, v in formula[species].items():
            expected[k] = expected.get(k, D(0)) + n * v
    residual = {k: after.get(k, D(0)) - before.get(k, D(0)) - expected.get(k, D(0)) for k in before.keys() | after.keys() | expected.keys()}
    assert all(abs(x) < D('1e-8') for x in residual.values()), residual
    return residual

def metric(v):
    aq, gas = carbon(v), carbon(v, True)
    return {'temperature_K': v['temperature'], 'total_pressure_Pa': v['pressure'], 'aqueous_carbon_mol': aq, 'gas_carbon_mol': gas, 'aqueous_carbon_fraction': aq / (aq + gas), 'CO2_partial_pressure_Pa': amount(v, 'CO2', 'gas') * D('8314.462618') * D(v['temperature']) / D(v['headspace']['volume'])}

results = []
with zipfile.ZipFile(archive) as z:
    for n in [2, 4, 6, 8]:
        cid = f'new120:gs-{n:03}'
        o, prior = originals[cid], old[cid]
        prefix = cid.replace(':', '_') + '/'
        script, raw = z.read(prefix + 'experiment.lab'), z.read(prefix + 'stdout.jsonl')
        assert sha(script) == o['script_sha256'] == prior['original_script_sha256']
        assert script.decode() == o['script']
        assert sha(raw) == prior['saved_execution_receipt']['stdout_sha256']
        assert o['acceptance_checks'] == prior['original_criteria']
        assert prior['material_projection']['max_per_step_residual'] < 1e-8
        rows = [json.loads(line, parse_float=D) for line in raw.splitlines()]
        assert not any(e['event'] in ['solver_failed', 'not_yet_modeled'] for r in rows for e in r['events'])
        routes = [e['provenance'] for r in rows for e in r['events'] if e['event'] == 'solution_routed']
        assert any('wateq4f.dat' in p['dataset'] for p in routes)
        assert not any('pitzer' in p['dataset'].lower() for p in routes)
        at = lambda i, vid: next(v for v in rows[i]['bench']['vessels'] if v['id'] == vid)
        residuals = []
        if n in [2, 4]:
            a, b = at(4, 0), at(10, 1)
            for base, end, species in [(at(1, 0), a, 'N2' if n == 2 else 'CO2'), (at(7, 1), b, 'CO2')]:
                residuals.append(delta(base, end, {species: D('.002')}))
            if n == 2:
                nitrogen_gas_gain = amount(a, 'N2', 'gas') - amount(at(1, 0), 'N2', 'gas')
                carbon_gas_gain = carbon(b, True) - carbon(at(7, 1), True)
                assert abs(nitrogen_gas_gain - D('.002')) < D('1e-8')
                assert 0 < carbon_gas_gain < nitrogen_gas_gain
                assert carbon(b) - carbon(at(7, 1)) > carbon(a) - carbon(at(1, 0))
                extra = {'added_N2_gas_fraction': nitrogen_gas_gain / D('.002'), 'added_CO2_gas_fraction': carbon_gas_gain / D('.002')}
                reasons = ['Subtract each arm’s own trapped-air baseline: the added N2 remains an ideal inert carrier while part of the equal CO2 dose enters owned aqueous carbon.', 'The actual CO2 arm stores more of its dose in liquid. This supports the original qualitative comparison within the ideal-carrier domain; quantitative real N2 dissolution is not implemented or certified.']
            else:
                assert D(b['temperature']) > D(a['temperature'])
                assert metric(b)['aqueous_carbon_fraction'] < metric(a)['aqueous_carbon_fraction']
                extra = {'actual_temperature_dependent_CO2_phase_source': phase.decode()}
                reasons = ['Compare the actual dissolved-carbon fractions at both recorded temperatures and each finite trapped-air baseline; the warm arm retains a smaller fraction.', 'The actual pinned WATEQ CO2 phase has a five-coefficient temperature relation and nonzero dissolution enthalpy. This is temperature-dependent native partitioning, not fixed-Henry or pressure-only inference; no empirical calibration is earned.']
        elif n == 6:
            a, b = at(4, 0), at(9, 0)
            residuals = [delta(at(2, 0), a, {'CO2': D('.001')}), delta(a, b, {'HCl': D('.003')})]
            ions = lambda v: amount(v, 'HCO3-') + amount(v, 'CO3-2')
            assert ions(b) < ions(a) and carbon(b, True) > carbon(a, True)
            assert abs(carbon(b) + carbon(b, True) - carbon(a) - carbon(a, True)) < D('1e-8')
            extra = {'inorganic_carbon_ions_before_after_mol': [ions(a), ions(b)]}
            reasons = ['Owned bicarbonate/carbonate decrease after acidification while gas CO2 increases, at the actual recorded temperatures.', 'The original finite carbon dose closes against its trapped-air baseline, and acid addition closes independently for all represented atoms/charge. Carbon is redistributed rather than deleted or supplied externally.']
        else:
            a, b = at(3, 0), at(8, 0)
            residuals = [delta(a, b, {'N2': D('.005')})]
            ma, mb = metric(a), metric(b)
            assert D(b['pressure']) > D(a['pressure']) and a['headspace']['volume'] == b['headspace']['volume']
            assert abs(carbon(b) + carbon(b, True) - carbon(a) - carbon(a, True)) < D('1e-8')
            assert abs(mb['aqueous_carbon_fraction'] - ma['aqueous_carbon_fraction']) < D('1e-8')
            thermal_corrected = (mb['CO2_partial_pressure_Pa'] / ma['CO2_partial_pressure_Pa']) / (D(b['temperature']) / D(a['temperature']))
            assert abs(thermal_corrected - 1) < D('1e-8')
            extra = {'temperature_corrected_CO2_partial_pressure_ratio': thermal_corrected, 'nonideal_correction_scope': 'The actual WATEQ CO2 phase lacks critical-parameter EOS entries and uses ideal gas behavior; inert N2 is an excluded-redox carrier. No real-gas correction or empirical fugacity claim. The original forecast is conditional on ideal mixing.'}
            reasons = ['Total pressure rises at fixed headspace while all owned carbon is conserved; the nitrogen addition closes independently.', 'Actual-temperature CO2 partial pressures and dissolved fractions are compared explicitly. Removing the recorded temperature ratio leaves a unit partial-pressure ratio at inventory resolution. The pinned phase’s ideal EOS domain is explicit; no nonideal gas correction is silently claimed.']
        assert len(reasons) == len(o['acceptance_checks']) == 2
        results.append({'id': cid, 'status': 'PASS_IN_ORIGINAL_BOUNDED_DOMAIN', 'script_sha256': sha(script), 'stdout_sha256': sha(raw), 'original_forecast': o['expected_result'], 'original_criteria_review': [dict(c, result='PASS', reason=r) for c, r in zip(o['acceptance_checks'], reasons)], 'before_after_or_paired_metrics': [metric(a), metric(b)], 'additional_metrics': extra, 'independent_material_charge_residuals_mol_or_eq': residuals, 'source_binding': prior['saved_execution_receipt']['engine'], 'qualification': 'Historical ideal-carrier/aqueous-carbonate model. No current-production, empirical solubility, general nonideal-gas, quantitative N2 uptake or reaction-energy certification.'})
report = {'source': source, 'database_gitlink': gitlink, 'database_sha256': sha(database), 'CO2_phase_sha256': sha(phase), 'vessel_source_sha256': sha(vessel_source), 'archive_sha256': screen['raw_artifact_sha256'], 'decimal_precision': 45, 'cases': results, 'selected': 4, 'original_criteria_pass': 8, 'new_engine_or_API_executions': 0}
encoded = json.dumps(report, indent=2, default=str) + '\n'
out = H / 'assessment.json'
if out.exists():
    assert json.loads(out.read_bytes()) == json.loads(encoded)
else:
    out.write_text(encoded)
print(json.dumps({'original_cases_reviewed': len(results), 'original_criteria_pass': 8, 'new_executions': 0}))
