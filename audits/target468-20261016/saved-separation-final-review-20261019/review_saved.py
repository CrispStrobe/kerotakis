"""Review original separation criteria and independent owned inventories, without execution."""
import decimal
import hashlib
import json
import pathlib
import subprocess
import zipfile

H = pathlib.Path(__file__).resolve().parent
A, R = H.parent, H.parents[2]
D = decimal.Decimal
decimal.getcontext().prec = 45
sha = lambda raw: hashlib.sha256(raw).hexdigest()
screen = json.loads((A / 'saved64-first-screen/first-screen.json').read_bytes())
prior = {c['id']: c for c in screen['cases']}
original_raw = (A / 'common-engine-completion/originals/new-experiments-20261016.json').read_bytes()
assert sha(original_raw) == screen['originals_sha256']
originals = {c['id']: c for c in json.loads(original_raw)['experiments']}
source = screen['engine']['source_head']
registry_raw = subprocess.check_output(['git', 'show', source + ':data/registry/registry-source-v1.json'], cwd=R)
registry = json.loads(registry_raw, parse_float=D)
formulas = {r['species_id']: {**{e['element']: D(e['count']['value']) for e in r['elements']}, 'charge': D(r['net_charge']['value'])} for r in registry['compositions']}
formulas['base_equivalents'] = formulas['OH-'].copy()
mass = {r['species_id']: D(r['quantity']['value']) for r in registry['phase_thermodynamics'] if r['property'] == 'molar_mass'}
archive = A / 'common-engine-completion/results/raw-artifact.zip'
with archive.open('rb') as f:
    assert hashlib.file_digest(f, 'sha256').hexdigest() == screen['raw_artifact_sha256']

def amount(v, species, phase=None):
    return sum((D(p['moles']) for p in v['contents'] if p['species'] == species and (phase is None or p['phase'] == phase)), D(0))

def xethanol(v):
    return amount(v, 'ethanol') / (amount(v, 'water') + amount(v, 'ethanol'))

def book(target, species, moles):
    for k, n in formulas[species].items():
        target[k] = target.get(k, D(0)) + D(moles) * n

def ledger(state):
    result = {}
    for v in state.values():
        assert not any(v.get(k, []) for k in ['unresolved_materials', 'material_objects', 'surfaces', 'electrodes', 'exchanges', 'adsorbed', 'solid_solutions'])
        for p in v['contents']:
            assert D(p['moles']).is_finite() and D(p['moles']) >= 0
            book(result, p['species'], p['moles'])
    return result

cases = []
with zipfile.ZipFile(archive) as z:
    for n in [1, 2, 5, 6, 7, 8, 9, 10]:
        cid = f'new120:sp-{n:03}'
        o, old = originals[cid], prior[cid]
        script = z.read(cid.replace(':', '_') + '/experiment.lab')
        raw = z.read(cid.replace(':', '_') + '/stdout.jsonl')
        assert sha(script) == old['original_script_sha256'] == o['script_sha256']
        assert script.decode() == o['script'] and o['acceptance_checks'] == old['original_criteria']
        assert sha(raw) == old['saved_execution_receipt']['stdout_sha256']
        rows = [json.loads(x, parse_float=D) for x in raw.splitlines()]
        state, frames, incoming, outgoing, residuals, limits = {}, [], {}, {}, [], []
        for i, row in enumerate(rows):
            before = state.copy()
            op = row['operator']
            if op['op'] == 'add':
                book(incoming, op['species'], op['moles'])
            for e in row['events']:
                assert e['event'] != 'solver_failed'
                if e['event'] == 'evaporated':
                    book(outgoing, 'water', e['moles'])
                if e['event'] == 'gas_evolved':
                    book(outgoing, e['species'], e['moles'])
                if e['event'] == 'gas_absorbed':
                    book(incoming, e['species'], e['moles'])
                if e['event'] == 'not_yet_modeled':
                    limits.append(e)
            published = {v['id']: v for v in row['bench']['vessels']}
            if set(state) - set(published):
                assert op['op'] == 'inspect' and not row['events']
                assert all(state[k] == v for k, v in published.items())
            state.update(published)
            actual = ledger(state)
            residual = {k: actual.get(k, D(0)) + outgoing.get(k, D(0)) - incoming.get(k, D(0)) for k in actual.keys() | outgoing.keys() | incoming.keys()}
            assert all(abs(x) < D('1e-8') for x in residual.values()), (cid, i, residual)
            residuals.append(residual)
            frames.append((row, before, state.copy()))
        metrics, reasons = {}, []
        if n in [1, 2, 10]:
            distils = [f for f in frames if f[0]['operator']['op'] == 'distil']
            for row, before, after in distils:
                op = row['operator']
                for species in ['water', 'ethanol']:
                    assert abs(sum((amount(v, species) for v in before.values()), D(0)) - sum((amount(v, species) for v in after.values()), D(0))) < D('1e-8')
                feed = before[op['from']]
                received = sum((amount(after[op['to']], k) - amount(before[op['to']], k) for k in ['water', 'ethanol']), D(0))
                assert abs(received - D(op['fraction']) * (amount(feed, 'water') + amount(feed, 'ethanol'))) < D('1e-8')
            if n == 1:
                first, second = state[1], state[2]
                assert xethanol(first) > xethanol(second)
                metrics['first_second_cut_ethanol_mole_fractions'] = [xethanol(first), xethanol(second)]
                metrics['actual_cut_moles'] = [sum((amount(v, k) for k in ['water', 'ethanol']), D(0)) for v in [first, second]]
                reasons = ['The first actual receiver has higher ethanol mole fraction than the second; unequal absolute ethanol amounts are not used as the criterion.', 'Each actual10% cut is independently reconciled against the then-remaining volatile-mole feed. Water/ethanol and all represented atoms/charge close across all three vessels.']
            elif n == 2:
                second = distils[1]
                assert xethanol(state[2]) > xethanol(second[1][1])
                initial = sum((D(r['operator']['moles']) for r in rows if r['operator']['op'] == 'add' and r['operator']['species'] == 'ethanol'), D(0))
                recovery = amount(state[2], 'ethanol') / initial
                assert 0 < recovery < 1
                metrics.update(first_distillate_feed_fraction=xethanol(second[1][1]), final_receiver_fraction=xethanol(state[2]), final_ethanol_recovery=recovery, original_ethanol_dose_mol=initial)
                reasons = ['The actual second distillate is richer in ethanol than its actual first-distillate feed.', 'Final ethanol recovery is calculated against the original10g dose, retaining both residual pots and all water/ethanol in the balance; enrichment is not complete recovery.']
            else:
                feed = distils[0][1][0]
                assert xethanol(state[1]) > xethanol(feed)
                assert all(amount(state[1], key) == 0 for key in ['Na+', 'Cl-', 'NaCl'])
                assert amount(state[1], 'water') > 0 and amount(state[1], 'ethanol') > 0
                metrics.update(feed_ethanol_fraction=xethanol(feed), distillate_ethanol_fraction=xethanol(state[1]), distillate_water_mol=amount(state[1], 'water'), distillate_ethanol_mol=amount(state[1], 'ethanol'))
                reasons = ['Actual ethanol enrichment and Na/Cl rejection are checked independently: both salt elements stay in the feed.', 'Both water and ethanol enter the receiver; the actual condensate is not assumed pure. Every operation closes represented atoms/charge and each distillation closes molecular water/ethanol across both vessels.']
        elif n == 5:
            f = next(f for f in frames if f[0]['operator']['op'] == 'filter')
            feed, receiver = f[1][0], state[1]
            for key in ['water', 'ethanol']:
                assert amount(receiver, key) == amount(feed, key) and amount(state[0], key) == 0
            assert xethanol(receiver) == xethanol(feed)
            metrics['transferred_ethanol_fraction'] = xethanol(receiver)
            reasons = ['The actual receiver retains the feed water and ethanol amounts and their mole fraction exactly.', 'Neither water nor ethanol remains as an invented filter cake. The ideal filter transfers the miscible liquids together; volatility is not used as its selection rule.']
        elif n == 6:
            assert amount(state[1], 'Fe', 'solid') > 0 and amount(state[0], 'SiO2', 'solid') > 0
            assert amount(state[2], 'NaCl', 'solid') > 0
            assert all(amount(state[vid], 'Fe') == 0 for vid in [0, 2])
            assert amount(state[2], 'water') > 0 and amount(state[2], 'Na+', 'aqueous') > 0
            exported = sum((D(e['moles']) for row in rows for e in row['events'] if e['event'] == 'evaporated'), D(0))
            assert exported > 0
            metrics.update(iron_recovery_mol=amount(state[1], 'Fe'), retained_solid_silica_mol=amount(state[0], 'SiO2', 'solid'), mother_liquor_silica_mol=amount(state[2], 'SiO2'), crystals_NaCl_mol=amount(state[2], 'NaCl', 'solid'), mother_liquor_Na_mol=amount(state[2], 'Na+'), exported_water_mol=exported)
            reasons = ['Actual solid iron is in the magnet receiver, most silica is in the filter residue and Na/Cl occupy the evaporated salt receiver.', 'The independent balance includes positive mother-liquor water, dissolved salt, dissolved silica and explicit water export. The preserved supersaturated-silica and iron/water kinetic limitations do not certify complete crystallisation or oxidation.']
        elif n == 7:
            assert {p['species'] for p in state[0]['contents']} == {'AgCl'}
            agcl = amount(state[0], 'AgCl', 'solid')
            assert 0 < agcl < D('.002')
            assert abs(amount(state[1], 'Ca+2') - D('.001')) < D('1e-8')
            assert abs(agcl + amount(state[1], 'Cl-') - D('.002')) < D('1e-8')
            measured = next(e for row in rows for e in row['events'] if e['event'] == 'measured' and e['instrument'] == 'balance')
            assert measured['unit'] == 'g' and abs(D(measured['value']) - agcl * mass['AgCl']) < D('1e-10')
            metrics.update(dry_AgCl_mol=agcl, dry_AgCl_mass_g=agcl * mass['AgCl'], ideal_dose_AgCl_mass_g=D('.002') * mass['AgCl'], dissolved_Ca_mol=amount(state[1], 'Ca+2'), residual_dissolved_chloride_mol=amount(state[1], 'Cl-'))
            reasons = ['The actual cake contains only identified dry AgCl; its amount plus dissolved chloride accounts for2mmol Cl from1mmol CaCl2.', 'Registry-derived ideal AgCl mass is about0.2866g; the small dissolved-chloride remainder explains the actual dry mass. Calcium remains dissolved, and the balance reading is independently reproduced from identified AgCl rather than wet material.']
        elif n == 8:
            assert abs(amount(state[1], 'glucose') - D('.001')) < D('1e-8')
            assert all(abs(amount(state[3], key) - D('.001')) < D('1e-8') for key in ['Na+', 'Cl-'])
            values = {e['vessel']: D(e['value']) for row in rows for e in row['events'] if e['event'] == 'measured' and e['instrument'] == 'conductivity_meter' and e['unit'] == 'µS/cm'}
            assert values[3] > values[1] >= 0
            metrics['glucose_and_salt_conductivity_uS_cm'] = [values[1], values[3]]
            reasons = ['Both actual filtrates retain their introduced dissolved solute:1mmol glucose or1mmol each Na+/Cl−.', 'Actual measured conductivity is higher in the salt filtrate, while the glucose inventory proves it is dissolved. The conclusion does not infer glucose insolubility from low conductivity.']
        else:
            drain = next(f for f in frames if f[0]['operator']['op'] == 'drain')
            index = next(i for i, f in enumerate(frames) if f is drain)
            layers = [e for row in rows[:index] for e in row['events'] if e['event'] == 'layers_formed']
            assert layers and all(e['lower'] == 'water' and e['upper'] == 'hexane' for e in layers)
            feed, receiver = drain[1][0], state[1]
            assert amount(receiver, 'water') == amount(feed, 'water') and amount(receiver, 'hexane') == 0
            assert amount(state[0], 'hexane') == amount(feed, 'hexane')
            assert all(amount(receiver, key) == amount(feed, key) for key in ['Na+', 'Cl-'])
            assert 0 < amount(receiver, 'I2') < amount(state[0], 'I2')
            metrics.update(actual_layer_events=layers, receiver_water_mol=amount(receiver, 'water'), source_hexane_mol=amount(state[0], 'hexane'), receiver_I2_mol=amount(receiver, 'I2'), source_all_phase_I2_mol=amount(state[0], 'I2'))
            reasons = ['Actual layer events identify water below hexane before the drain; layer existence is not inferred from colour or the command name.', 'Actual water/Na/Cl move to the receiver while hexane stays in the source. All solid/dissolved iodine across both vessels closes; phase inventory, not colour, supports the comparison. Missing iodine ionic speciation remains explicit and earns no pH/ionic-strength claim.']
        assert len(reasons) == len(o['acceptance_checks']) == 2
        cases.append({'id': cid, 'status': 'PASS_IN_ORIGINAL_BOUNDED_DOMAIN', 'script_sha256': sha(script), 'stdout_sha256': sha(raw), 'original_forecast': o['expected_result'], 'original_criteria_review': [dict(c, result='PASS', reason=r) for c, r in zip(o['acceptance_checks'], reasons)], 'metrics': metrics, 'explicit_limit_events_preserved': limits, 'maximum_whole_bench_atom_charge_residual_mol_or_eq': max(abs(x) for r in residuals for x in r.values()), 'source_binding': old['saved_execution_receipt']['engine'], 'qualification': 'Original ideal separation/declared property model; no empirical apparatus recovery, quantitative iodine speciation, general mixture enthalpy or current-production certification. Historical binary/lock hashes remain producer-reported where bytes are absent.'})
report = {'source': source, 'registry_sha256': sha(registry_raw), 'archive_sha256': screen['raw_artifact_sha256'], 'decimal_precision': 45, 'cases': cases, 'selected': len(cases), 'original_criteria_passed': 16, 'unselected': {'new120:sp-004': 'Energy-budget convention needs a separate explicit final review; no total sensible/latent or practical apparatus-energy closure inferred.', 'new120:sp-003': 'Prior held result remains unchanged.', 'new120:sp-011': 'Prior held result remains unchanged.', 'new120:sp-012': 'Missing retention data remain unsupported.'}, 'new_engine_or_API_executions': 0}
encoded = json.dumps(report, indent=2, default=str) + '\n'
out = H / 'assessment.json'
if out.exists():
    assert json.loads(out.read_bytes()) == json.loads(encoded)
else:
    out.write_text(encoded)
print(json.dumps({'original_cases_reviewed': len(cases), 'original_criteria_pass': 16, 'new_executions': 0}))
