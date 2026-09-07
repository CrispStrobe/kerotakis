"""Emit an apply_patch for stoichiometrically derived aqueous basis identities.

No thermodynamic constants are supplied or fitted by this tool.
"""
import copy
import difflib
import json
import pathlib
import re

root = pathlib.Path(__file__).resolve().parents[2]
path = root / 'data/registry/registry-source-v1.json'
old = path.read_text()
document = json.loads(old)
source = 'kerotakis/aqueous-basis-v1'
if any(row['id'] == source for row in document['sources']):
    raise SystemExit('Basis source already exists; use registry-export to regenerate, not this one-time migration.')
additions = {key: [] for key in ['sources', 'identities', 'compositions', 'phase_thermodynamics']}
additions['sources'].append(dict(id=source, citation='Kerotakis analytical aqueous basis: formula stoichiometry and existing registry atomic mass basis. Aqueous partial heat capacities are not modeled (zero); density is an unused placeholder because solvent carries volume. No equilibrium or kinetic constants introduced.', licence='CC0-1.0', lane='runtime', origin='Kerotakis original stoichiometric derivation', revision='1', retrieved='2026-09-06'))
composition = next(row for row in document['compositions'] if row['species_id'] == 'OH-')
numbers = [row for row in document['phase_thermodynamics'] if row['species_id'] == 'OH-']
rows = [
    ('H+', 'hydrogen ion', {'H': 1}, 1, 1.008),
    ('CO2(aq)', 'dissolved carbon dioxide', {'C': 1, 'O': 2}, 0, 44.009),
    ('CO3-2', 'carbonate ion', {'C': 1, 'O': 3}, -2, 60.009),
    ('HPO4-2', 'hydrogen phosphate ion', {'H': 1, 'P': 1, 'O': 4}, -2, 95.978),
    ('PO4-3', 'phosphate ion', {'P': 1, 'O': 4}, -3, 94.97),
    ('NO2-', 'nitrite ion', {'N': 1, 'O': 2}, -1, 46.005),
]
for key, name, atoms, charge, mass in rows:
    evidence = dict(source_id=source, method=dict(kind='derived', detail='formula stoichiometry and registry atomic mass basis'))
    additions['identities'].append(dict(id=key, canonical_key=key, name=name, identifiers={}, synonyms=[], evidence=evidence))
    record = copy.deepcopy(composition)
    record.update(id='composition/' + key, species_id=key, formula='CO2' if key == 'CO2(aq)' else key, evidence=evidence, elements=[])
    for element, count in atoms.items():
        quantity = copy.deepcopy(composition['elements'][0]['count'])
        quantity.update(value=count, source_id=source, method=evidence['method'])
        record['elements'].append(dict(element=element, count=quantity))
    record['net_charge'].update(value=charge, source_id=source, method=evidence['method'])
    additions['compositions'].append(record)
    for template in numbers:
        record = copy.deepcopy(template)
        record.update(id=record['id'].replace('OH-', key), species_id=key)
        record['quantity'].update(source_id=source, method=evidence['method'])
        if record['property'] == 'molar_mass':
            record['quantity']['value'] = mass
        additions['phase_thermodynamics'].append(record)
new = old
keys = list(document)
for key, records in additions.items():
    next_key = keys[keys.index(key) + 1]
    position = new.index('\n  ],\n  "' + next_key + '"')
    new = new[:position] + ',\n' + ',\n'.join('    ' + json.dumps(r) for r in records) + new[position:]
json.loads(new)
patch = ''.join(list(difflib.unified_diff(old.splitlines(True), new.splitlines(True), n=2))[2:])
patch = re.sub(r'^@@.*@@$', '@@', patch, flags=re.M)
print('*** Begin Patch\n*** Update File: ' + str(path))
print(patch, end='')
print('*** End Patch')
