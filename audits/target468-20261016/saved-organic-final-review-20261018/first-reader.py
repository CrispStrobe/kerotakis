"""Original-criterion review using saved native output; no simulator requests."""
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
original_raw = (A / 'common-engine-completion/originals/new-experiments-20261016.json').read_bytes()
assert sha(original_raw) == screen['originals_sha256']
originals = {c['id']: c for c in json.loads(original_raw)['experiments']}
projection = json.loads((A / 'saved64-first-screen/material-projection.json').read_bytes())
prepared = json.loads((A / 'saved64-organic-preparation/metrics.json').read_bytes())
prep = {c['id']: c for c in prepared['cases']}
source = screen['engine']['source_head']
curated = subprocess.check_output(['git', 'show', source + ':crates/kerotakis-core/src/curated.rs'], cwd=R)
assert sha(curated) == prepared['curated_source_sha256']
assert b'equilibrium_log_k: Some(0.6020599913279624)' in curated
archive = A / 'common-engine-completion/results/raw-artifact.zip'
with archive.open('rb') as f:
    assert hashlib.file_digest(f, 'sha256').hexdigest() == screen['raw_artifact_sha256']

def amount(v, key):
    return sum((D(p['moles']) for p in v['contents'] if p['species'] == key), D(0))

def ledger(v):
    out = {}
    for p in v['contents']:
        n = D(p['moles'])
        assert n.is_finite() and n >= 0
        key = p['species']
        for element, count in {**projection['formulas'][key], 'charge': projection['charges'][key]}.items():
            out[element] = out.get(element, D(0)) + n * D(str(count))
    return out

def quotient(v):
    return amount(v, 'ethyl_acetate') * amount(v, 'water') / (amount(v, 'CH3COOH') * amount(v, 'ethanol'))

def inorganic(v):
    return sum((amount(v, key) for key in ['CO2', 'CO2(aq)', 'HCO3-', 'CO3-2']), D(0))

results = []
with zipfile.ZipFile(archive) as z:
    for number in [1, 2, 3, 4, 5, 10, 11, 12]:
        cid = f'new120:or-{number:03}'
        old, original = by_id[cid], originals[cid]
        prefix = cid.replace(':', '_') + '/'
        script, raw = z.read(prefix + 'experiment.lab'), z.read(prefix + 'stdout.jsonl')
        assert sha(script) == original['script_sha256'] == old['original_script_sha256']
        assert script.decode() == original['script']
        assert sha(raw) == old['saved_execution_receipt']['stdout_sha256'] == prep[cid]['stdout_sha256']
        assert original['acceptance_checks'] == old['original_criteria'] == prep[cid]['original_criteria']
        assert old['material_projection']['max_per_step_residual'] < 1e-8
        rows = [json.loads(line, parse_float=D) for line in raw.splitlines()]
        state, reactions, residuals = {}, [], []
        for row in rows:
            before = state.copy()
            state.update({v['id']: v for v in row['bench']['vessels']})
            assert not any(e['event'] == 'solver_failed' for e in row['events'])
            op = row['operator']
            if op['op'] == 'react':
                v0, v1 = before[op['vessel']], state[op['vessel']]
                a, b = ledger(v0), ledger(v1)
                residual = {k: b.get(k, D(0)) - a.get(k, D(0)) for k in set(a) | set(b)}
                assert all(abs(n) < D('1e-8') for n in residual.values()), (cid, residual)
                residuals.append(residual)
                reactions.append((v0, v1))
        reasons, metrics = [], {'reaction_material_charge_residuals': residuals}
        if number in [1, 2, 3, 4, 5]:
            q = [quotient(after) for _, after in reactions]
            metrics['actual_post_tail_molecular_quotients'] = q
            if number != 3:
                assert all(abs(v - 4) < D('1e-10') for v in q)
        if number == 1:
            before, after = reactions[0]
            assert all(amount(after, key) > 0 for key in ['ethyl_acetate', 'CH3COOH', 'ethanol'])
            reasons = ['Ester, acid and ethanol all remain in the owned inventory after the explicit equilibrium request.', 'The actual molecular quotient is approximately4 and the ester fraction approximately two-thirds in the declared ideal model. Organic-rich ionic speciation is explicitly uncharacterized; this does not calibrate aqueous activities or an experimental yield.']
        elif number in [2, 3]:
            assert len(reactions) == 2
            yields = [amount(after, 'ethyl_acetate') / D('.1') for _, after in reactions]
            metrics['ester_per_initial_acid_mol'] = yields
            if number == 2:
                assert yields[1] > yields[0]
                assert all(abs(r.get(k, D(0))) < D('1e-8') for r in residuals for k in ['C', 'O'])
                reasons = ['The alcohol-rich arm converts a larger fraction of the same0.1mol initial acid dose; owned ester fractions are compared rather than concentrations in unequal volumes.', 'Both independently reconstructed owned carbon and oxygen ledgers close through each reaction. The source requests an ideal equilibrium conversion, not measured mixed-solvent activities or reaction heat.']
            else:
                assert yields[1] < yields[0]
                before, after = reactions[1]
                water_gain = amount(after, 'water') - amount(before, 'water')
                ester_gain = amount(after, 'ethyl_acetate') - amount(before, 'ethyl_acetate')
                assert abs(amount(before, 'water') - D('.2')) < D('1e-12')
                assert abs(water_gain - ester_gain) < D('1e-8')
                metrics.update(initial_product_water=amount(before, 'water'), final_product_water=amount(after, 'water'), generated_water=water_gain)
                reasons = ['The added-water arm forms less ester per the same initial acid dose than the initially dry control.', 'The comparison includes the original0.2mol water plus generated water. The actual post-tail molecular quotient is3.998559711649207, retained rather than rounded to4; aqueous acetate/proton redistribution precludes claiming an exact measured equilibrium constant.']
        elif number == 4:
            before, after = reactions[0]
            changes = {key: amount(after, key) - amount(before, key) for key in ['CH3COOH', 'ethanol', 'ethyl_acetate', 'water']}
            assert changes['CH3COOH'] > 0 and changes['ethanol'] > 0 and changes['ethyl_acetate'] < 0 and changes['water'] < 0
            assert abs(changes['ethyl_acetate'] + D('.016')) < D('1e-12')
            metrics['molecular_amount_changes'] = changes
            reasons = ['Acid and ethanol each gain0.016mol while ester and water each lose0.016mol; the actual quotient reaches the declared ideal4.', 'The owned negative ester extent demonstrates a real reverse request, rather than labeling absent forward conversion an equilibrium success. Organic-rich ionic speciation remains uncharacterized.']
        elif number == 5:
            assert len(reactions) == 2
            first, second = reactions[0][1], reactions[1][1]
            differences = {key: amount(second, key) - amount(first, key) for key in ['CH3COOH', 'ethanol', 'ethyl_acetate', 'water']}
            assert all(n == 0 for n in differences.values())
            metrics['second_request_amount_changes'] = differences
            reasons = ['All four owned acid/alcohol/ester/water amounts are identical between the first and second equilibrium snapshots.', 'The second request creates zero additional ester and retains the same quotient; this is idempotence of the explicit ideal model, not a kinetic observation.']
        elif number == 10:
            before, after = reactions[0]
            extent = amount(before, 'ethanol') - amount(after, 'ethanol')
            oxygen_loss = amount(before, 'O2') - amount(after, 'O2')
            acid_gain = sum((amount(after, k) - amount(before, k) for k in ['CH3COOH', 'CH3COO-', 'NaOAc']), D(0))
            assert abs(extent - D('.001')) < D('1e-12')
            assert abs(oxygen_loss - extent) < D('1e-12') and abs(acid_gain - extent) < D('1e-8')
            assert amount(state[1], 'ethanol') == D('.001')
            assert sum((amount(state[1], k) for k in ['CH3COOH', 'CH3COO-', 'NaOAc']), D(0)) == 0
            metrics.update(ethanol_loss=extent, oxygen_loss=oxygen_loss, acetate_family_gain=acid_gain, no_command_ethanol=amount(state[1], 'ethanol'))
            reasons = ['The requested arm consumes0.001mol ethanol and matching oxygen and gains the matching acetic-acid/acetate pool, with full owned atom/charge closure.', 'The otherwise matched no-command vessel retains its ethanol and has no acetate-family product after60s. This explicitly requested stoichiometry does not establish organisms, rate, reaction heat or an acetaldehyde mechanism.']
        else:
            before, after = reactions[0]
            glucose_loss = amount(before, 'glucose') - amount(after, 'glucose')
            oxygen_loss = amount(before, 'O2') - amount(after, 'O2')
            carbon_gain = inorganic(after) - inorganic(before)
            assert abs(oxygen_loss - 6 * glucose_loss) < D('1e-12')
            assert abs(carbon_gain - 6 * glucose_loss) < D('1e-8')
            metrics.update(glucose_consumed=glucose_loss, oxygen_consumed=oxygen_loss, inorganic_carbon_gain=carbon_gain, oxygen_owned_before=amount(before, 'O2'), oxygen_ceiling_glucose=amount(before, 'O2') / 6)
            if number == 11:
                assert abs(glucose_loss - D('.001')) < D('1e-12')
                reasons = ['The requested complete glucose conversion produces six times its consumed amount as new total owned inorganic carbon, including gas CO2, CO2(aq), bicarbonate and carbonate.', 'Actual owned O2 consumption is six per glucose. This closes the requested overall stoichiometry; ATP, cellular mechanism, metabolic heat and empirical calibration are outside the claim.']
            else:
                assert amount(after, 'glucose') > 0
                assert abs(glucose_loss - amount(before, 'O2') / 6) < D('1e-12')
                assert amount(before, 'O2') > D('.001')
                assert not any(amount(after, k) > 0 for k in ['ethanol', 'CH3COOH', 'CH3COO-'])
                reasons = ['Glucose consumption matches the actual pre-command total oxygen divided by six and leaves glucose; original trapped-air oxygen is counted.', 'The finite oxygen inventory includes more than the explicitly added0.001mol. No anaerobic ethanol/acetate product is invented; only the requested overall aerobic stoichiometry is accepted.']
        assert len(reasons) == len(original['acceptance_checks']) == 2
        results.append({'id': cid, 'status': 'PASS_IN_ORIGINAL_BOUNDED_DOMAIN', 'original_forecast': original['expected_result'], 'original_criteria_review': [dict(c, result='PASS', reason=reason) for c, reason in zip(original['acceptance_checks'], reasons)], 'script_sha256': sha(script), 'stdout_sha256': sha(raw), 'metrics': metrics, 'source_binding': old['saved_execution_receipt']['engine'], 'material_projection': old['material_projection'], 'explicit_limits_preserved': prep[cid]['explicit_limits'], 'qualification': 'Original explicit ideal-equilibrium or requested overall stoichiometric model. Organic-rich ionic activity, rates, reaction heat and biological mechanisms are not inferred. Historical engine/lock metadata remains producer-qualified; this is not current-production certification.'})
report = {'schema': 'kerotakis-saved-organic-final-review-v1', 'source': source, 'curated_source_sha256': sha(curated), 'archive_sha256': screen['raw_artifact_sha256'], 'decimal_precision': 45, 'cases': results, 'selected': len(results), 'new_engine_or_API_executions': 0, 'excluded': {f'new120:or-{n:03}': 'Original literal-hydroxide prerequisite failure preserved; separate repaired-source native acceptance/delivery replay required.' for n in [6, 7, 8, 9]}}
# JSON number strings retain every computed Decimal without losing precision.
encoded = json.dumps(report, indent=2, default=str) + '\n'
out = H / 'assessment.json'
if out.exists():
    assert json.loads(out.read_bytes()) == json.loads(encoded)
else:
    out.write_text(encoded)
print(json.dumps({'original_cases_reviewed': len(results), 'original_criteria_pass': 16, 'new_executions': 0}))
