"""Prospective phase selection controls for an already compiled generator."""
import argparse, copy, hashlib, json, os, pathlib, re, subprocess
CASES = [('canonical', None, 'canonical', False), ('reverse_thermo', None, 'reverse', False)]
for prop in ['molar_heat_capacity', 'mass_density']:
    CASES += [(prop + '_foreign_' + pos, prop, 'foreign_' + pos, False) for pos in ['prepend', 'append']]
    CASES += [(prop + '_wrong_phase', prop, 'wrong_phase', True)]
for prop in ['molar_mass', 'molar_heat_capacity', 'mass_density']:
    CASES += [(prop + '_conflict_' + pos, prop, 'conflict_' + pos, True) for pos in ['prepend', 'append']]
def values(text, key):
    match = re.search(r'key:\s*"' + re.escape(key) + r'",', text)
    assert match, key
    row = text[match.end():].split('    SpeciesData {', 1)[0]
    fields = {field: float(re.search(r'^\s*' + field + r':\s*([^,]+),$', row, re.M)[1]) for field in ['molar_mass', 'density', 'heat_capacity']}
    fields['phase'] = re.search(r'^\s*standard_phase:\s*Phase::(\w+),$', row, re.M)[1].lower()
    return fields

def main():
    parser = argparse.ArgumentParser()
    for option in ['binary', 'registry', 'out']: parser.add_argument('--' + option, type=pathlib.Path, required=True)
    args = parser.parse_args(); binary = args.binary.resolve(); doc = json.loads(args.registry.read_text()); results = []
    expected = {}
    for identity in doc['identities']:
        key = identity['id']; rows = {r['property']: r for r in doc['phase_thermodynamics'] if r['species_id'] == key and isinstance(r['property'], str)}
        expected[key] = {'molar_mass': rows['molar_mass']['quantity']['value'], 'density': rows['mass_density']['quantity']['value'], 'heat_capacity': rows['molar_heat_capacity']['quantity']['value'], 'phase': rows['molar_mass']['phase']}
    zero_keys = [key for key, row in expected.items() if row['heat_capacity'] == 0]
    for name, prop, mutation, refusal in CASES:
        case = args.out / name; case.mkdir(parents=True, exist_ok=False)
        manifest = case / 'crates' / 'kerotakis-core'; manifest.mkdir(parents=True)
        data = case / 'data' / 'registry'; data.mkdir(parents=True)
        out = case / 'generated'; out.mkdir(); probe = copy.deepcopy(doc)
        if mutation == 'reverse': probe['phase_thermodynamics'].reverse()
        if prop:
            rows = [r for r in probe['phase_thermodynamics'] if r['species_id'] == 'MgSO4' and r['property'] == prop]; assert len(rows) == 1
            if mutation == 'wrong_phase':
                rows[0]['phase'] = 'gas'; rows[0]['quantity']['conditions']['phase'] = 'gas'
            else:
                extra = copy.deepcopy(rows[0]); extra['id'] = 'independent_' + name; extra['quantity']['value'] *= 2.0
                if mutation.startswith('foreign'): extra['phase'] = 'gas'; extra['quantity']['conditions']['phase'] = 'gas'
                if mutation.endswith('prepend'): probe['phase_thermodynamics'].insert(0, extra)
                else: probe['phase_thermodynamics'].append(extra)
        payload = json.dumps(probe).encode(); (data / 'registry-source-v1.json').write_bytes(payload)
        proc = subprocess.run([str(binary)], cwd=manifest, env=dict(os.environ, CARGO_MANIFEST_DIR=str(manifest.resolve()), OUT_DIR=str(out.resolve())), capture_output=True, timeout=15)
        (case / 'stdout.log').write_bytes(proc.stdout); (case / 'stderr.log').write_bytes(proc.stderr)
        if refusal:
            error = proc.stderr.lower(); assert proc.returncode != 0 and b'mgso4' in error and prop.encode() in error and (b'phase' in error or b'ambig' in error), (name, proc.stderr)
            assert not (out / 'species_generated.rs').exists()
        else:
            assert proc.returncode == 0, proc.stderr
            text = (out / 'species_generated.rs').read_text(); keys = re.findall(r'^\s*key:\s*"([^"\n]+)",$', text, re.M)
            assert len(keys) == len(expected) and set(keys) == set(expected)
            for key, e in expected.items():
                actual = values(text, key); assert actual['phase'] == e['phase'], (name, key)
                assert all(abs(actual[k]-e[k]) <= max(abs(e[k]), 1.0)*1e-12 for k in ['molar_mass', 'density', 'heat_capacity']), (name, key, actual, e)
            assert all(values(text, key)['heat_capacity'] == 0 for key in zero_keys)
            for file in ['solvent_generated.rs', 'materials_generated.rs']: assert (out / file).stat().st_size > 0
        results.append({'case': name, 'passed': True, 'exit_code': proc.returncode, 'input_sha256': hashlib.sha256(payload).hexdigest(), 'stdout_sha256': hashlib.sha256(proc.stdout).hexdigest(), 'stderr_sha256': hashlib.sha256(proc.stderr).hexdigest()})
    report = {'passed': len(results), 'cases': results, 'species_values_and_phases_preserved': len(expected), 'zero_cp_rows_preserved': len(zero_keys), 'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(), 'registry_sha256': hashlib.sha256(args.registry.read_bytes()).hexdigest()}
    (args.out / 'results.json').write_text(json.dumps(report, indent=2) + '\n'); print(json.dumps({'passed': len(results)}))
if __name__ == '__main__': main()
