"""Freeze one declared-contact/activation CLI probe for every supplied reaction.

Usage: prepare.py DATASET_JSON
Registry identities are read from the exact source of the previously built CLI.
This adapter does not import new species or reactions into Kerotakis.
"""
from collections import defaultdict
import datetime
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

SOURCE = 'd791d3062886e4a06dbc7c9d1635383da3e05359'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def atoms(formula):
    if not formula:
        return None
    total = defaultdict(float)
    for segment in formula.split('·'):
        prefix = re.match(r'^(\d+(?:\.\d+)?)', segment)
        scale = float(prefix[0]) if prefix else 1.0
        body = segment[len(prefix[0]):] if prefix else segment
        tokens = re.findall(r'[A-Z][a-z]?|\d+(?:\.\d+)?|[()\[\]]', body)
        if ''.join(tokens) != body:
            return None
        stack = [defaultdict(float)]
        i = 0
        while i < len(tokens):
            token = tokens[i]
            if token in ('(', '['):
                stack.append(defaultdict(float)); i += 1; continue
            if token in (')', ']'):
                if len(stack) == 1:
                    return None
                part = stack.pop()
            elif token[0].isupper():
                part = {token: 1.0}
            else:
                return None
            i += 1
            multiplier = float(tokens[i]) if i < len(tokens) and tokens[i][0].isdigit() else 1.0
            if i < len(tokens) and tokens[i][0].isdigit():
                i += 1
            for element, count in part.items():
                stack[-1][element] += multiplier * count
        if len(stack) != 1:
            return None
        for element, count in stack[0].items():
            total[element] += scale * count
    return dict(total)


def normalize(name):
    return ' '.join(name.casefold().replace('_', ' ').split())


def name_variants(name):
    yield normalize(name)
    yield normalize(re.sub(r'\([^()]*\)', '', name))
    for parenthetical in re.findall(r'\(([^()]*)\)', name):
        if len(parenthetical) > 2:
            yield normalize(parenthetical)


def make_mapper(registry):
    identities = {row['id']: row for row in registry['identities']}
    compositions = {row['species_id']: row for row in registry['compositions']}
    names = defaultdict(set)
    for key, identity in identities.items():
        for name in [identity['name'], *identity.get('synonyms', [])]:
            if isinstance(name, str):
                for variant in name_variants(name):
                    names[variant].add(key)

    def consistent(participant, key):
        composition = compositions.get(key)
        return composition is not None and atoms(participant.get('formula')) == {
            row['element']: row['count']['value'] for row in composition['elements']
        } and participant.get('charge') == composition['net_charge']['value']

    def resolve(participant):
        key = participant['species_id']
        if key == 'H3O+' and participant['formula'] == 'H3O' and participant['charge'] == 1:
            return {'source_id': key, 'method': 'explicit aqueous hydronium = H+ plus associated H2O',
                    'components': ['H+', 'water'], 'limitations': []}
        candidates = []
        if key in identities:
            candidates.append((key, 'exact species ID'))
        for name in [participant['name_en'], participant['name_de']]:
            for variant in name_variants(name):
                matches = names[variant]
                if len(matches) == 1:
                    candidates.append((next(iter(matches)), 'exact registry name or parenthetical name, composition/charge verified'))
        charge = participant.get('charge')
        if isinstance(charge, int) and charge != 0 and participant.get('formula'):
            canonical = participant['formula'] + ('+' if charge > 0 else '-') + (str(abs(charge)) if abs(charge) > 1 else '')
            if canonical in identities:
                candidates.append((canonical, 'explicit ionic charge notation'))
        for candidate, method in candidates:
            if consistent(participant, candidate):
                return {'source_id': key, 'method': method, 'components': [candidate], 'limitations': []}
        if not re.fullmatch(r'[A-Za-z0-9_+().\[\]·-]+', key):
            raise ValueError('Unrepresentable token requires explicit review: ' + key)
        return {'source_id': key, 'method': 'unresolved original species ID submitted unchanged',
                'components': [key], 'limitations': ['No unambiguous admitted identity mapping; preserve actual CLI answer']}

    return resolve


def prepare(dataset_path):
    root = Path(__file__).resolve().parent
    dataset_bytes = dataset_path.read_bytes()
    data = json.loads(dataset_bytes)
    registry_bytes = subprocess.check_output(['git', 'show', SOURCE + ':data/registry/registry-source-v1.json'])
    resolve = make_mapper(json.loads(registry_bytes))
    target = root / 'design'
    target.mkdir()
    (target / 'scripts').mkdir()
    records = []
    for reaction in data['reactions']:
        rank, identifier = reaction['rank'], reaction['id']
        limitations = ['No source procedure supplied; this is a declared contact/activation screen, not reaction validation',
                       'Add uses the registry phase; dataset phase labels cannot override it']
        aqueous = any(p['phase'] in ('aq', 'solution') for p in reaction['reactants'] + reaction['products'])
        gas = any(p['phase'] == 'g' for p in reaction['reactants'] + reaction['products'])
        lines = ['# ' + identifier]
        if aqueous:
            lines.append('add v1 water 100mL @ 25C')
        if gas:
            lines.append('seal v1 1L')
            limitations.append('Declared sealed 1 L headspace includes initial air; baseline gases are not reaction products')
        lines.append('inspect v1')
        mappings = []
        for participant in reaction['reactants']:
            mapping = resolve(participant)
            coefficient = participant['stoichiometric_coefficient']
            if isinstance(coefficient, str):
                limitations.append('Symbolic coefficient ' + coefficient + ' normalized to 1 solely for a finite contact probe')
                coefficient = 1
            amount = .001 * coefficient
            mapping['amount_mol_per_component'] = amount
            for component in mapping['components']:
                lines.append(f'add v1 {component} {amount:.12g}mol')
            mappings.append(mapping)
            limitations.extend(mapping['limitations'])
        lines.append('inspect v1')
        activation = []
        summary = reaction['conditions']['summary'].casefold()
        if 'combustion' in identifier or identifier == 'hydrogen_oxygen':
            activation.append('ignite v1')
        elif 'electrolysis' in identifier:
            activation.append('electrolyse v1 0.1A 10s')
        elif 'photolysis' in identifier or identifier == 'methane_chlorination':
            activation.append('irradiate v1 254nm 10W/m2')
        elif re.search(r'\b(heating|thermal|high-temperature|elevated-temperature)\b', summary):
            activation.append('heat v1 1kJ')
        if activation:
            limitations.append('Activation dose is a frozen screening assumption, not a source-specified condition')
        if reaction['conditions']['catalysts_or_initiators'] or reaction['conditions']['auxiliary_reagents']:
            limitations.append('Unspecified catalysts/auxiliary procedures are recorded but not invented or silently supplied')
        if 'half_reaction' == reaction['equation']['representation']:
            limitations.append('Half-reaction has no supplied circuit or electron partner; electrons are not silently omitted')
        lines += activation + ['wait 10s', 'inspect v1']
        filename = f'{rank:03d}-{identifier}.lab'
        (target / 'scripts' / filename).write_text('\n'.join(lines) + '\n')
        records.append({'rank': rank, 'id': identifier, 'script': 'scripts/' + filename,
                        'name': reaction['name'], 'equation': reaction['equation'],
                        'reactants': reaction['reactants'], 'products': reaction['products'],
                        'expected_observations_from_source': reaction['observations_or_result'],
                        'conditions': reaction['conditions'], 'aqueous_solvent_added': aqueous,
                        'sealed_headspace': gas, 'activation_commands': activation,
                        'identity_mappings': mappings, 'limitations': list(dict.fromkeys(limitations))})
    assert len(records) == 200 and len({record['id'] for record in records}) == 200
    (target / 'manifest.json').write_text(json.dumps({'schema_version': 1, 'source_head': SOURCE,
        'dataset_sha256': digest(dataset_bytes), 'registry_sha256': digest(registry_bytes), 'records': records}, ensure_ascii=False, indent=2) + '\n')
    files = [target / 'manifest.json', *sorted((target / 'scripts').glob('*.lab'))]
    freeze = {'schema_version': 1, 'frozen_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'reaction_count': 200, 'files': {str(path.relative_to(target)): digest(path.read_bytes()) for path in files}}
    (target / 'freeze.json').write_text(json.dumps(freeze, indent=2) + '\n')
    print('Frozen 200 probes; no CLI executed')


if __name__ == '__main__':
    prepare(Path(sys.argv[1]))
