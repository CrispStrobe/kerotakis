"""Offline release-to-browser binding. Never executes a simulation or makes a request.

Receipt bytes must first be authenticated by the existing production/artifact/API
chain. This module checks their consistency; it does not replace those readers.
"""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import re

SOURCE = '33e043ef4a4a4225e5ea8d49ba93e8df99716c5f'
TREE = 'f77a9e82941f30a6433583e1bd417939118d13c6'
CANDIDATE = '18f2bebd6f9aafe85d3ba1d76e2c2f9cd5d11e9a'
ASSETS = {'iphreeqc.mjs', 'iphreeqc.wasm', 'kerotakis.mjs',
          'kerotakis_wasm.js', 'kerotakis_wasm_bg.wasm'}
RECEIPTS = ('API-ACCEPTED.json', 'verified-release.json',
            'script-assessment.json', 'bound-release-design.json',
            'production-run.json', 'release-artifact-metadata.json',
            'EXT4-ARTIFACT-VERIFIED.json', 'STORAGE-ARTIFACT-VERIFIED.json')
ORIGINAL_PROTOCOLS_SHA256 = '1539b8a581e1880fb004a17a61c7270d082cc3cf9137f403d350ed4403aab079'


def require(value, message):
    if not value:
        raise ValueError(message)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def canonical(obj):
    return json.dumps(obj, sort_keys=True, separators=(',', ':'),
                      ensure_ascii=True, allow_nan=False).encode('utf-8')


def hash_value(value):
    require(isinstance(value, str) and re.fullmatch('[a-f0-9]{64}', value),
            'Missing or malformed SHA256')
    return value


def checked_binding(template, raw, source, tree, original_protocols):
    """Use actual saved receipt bytes, preserving every other template field."""
    require(set(RECEIPTS) <= set(raw), 'Incomplete actual release receipts')
    docs = {name: json.loads(raw[name]) for name in RECEIPTS}
    require(digest(original_protocols) == ORIGINAL_PROTOCOLS_SHA256,
            'Original eleven protocols changed')
    api = docs['API-ACCEPTED.json']
    verified = docs['verified-release.json']
    assessment = docs['script-assessment.json']
    design = docs['bound-release-design.json']
    engine = api['engine']
    require(set(engine) == {'source_head', 'binary_sha256', 'cargo_lock_sha256'},
            'Engine identity must include actual binary and producer lock')
    require(api['source'] == engine['source_head'] == source,
            'API/source mismatch')
    for key in ('binary_sha256', 'cargo_lock_sha256'):
        hash_value(engine[key])
    require(api['eleven_original_protocols_pass'] is True and
            api['concurrency'] == [200, 429], 'API acceptance incomplete')
    require(all(verified['engine'][k] == v for k, v in engine.items()) and
            verified['engine']['source_tree'] == tree, 'Artifact/source mismatch')
    for key in ('actual_binary_bytes_rehashed', 'actual_producer_lock_bytes_rehashed',
                'source_and_tree_equal_accepted_main'):
        require(verified[key] is True, 'Actual artifact authentication missing: ' + key)
    require(verified['runtime_library_bytes_rehashed'] == verified['engine']['libraries']
            and len(verified['engine']['libraries']) == 5,
            'Actual runtime library authentication missing')
    isolation = verified['snapshot_and_isolation']
    require(isolation['native_smoke_completed'] is True and
            isolation['native_smoke_observations'] == 2 and
            isolation['network'] == 'deny-all' and isolation['vcpus'] == 1 and
            isolation['memory_mib'] == 2048 and
            isolation['actual_isolation_observation']['isolation_controls_pass'] is True and
            isolation['snapshot_and_native_manifest_agree'] is True,
            'Actual snapshot/isolation prerequisites missing')
    require(assessment['engine'] == design['accepted_engine'] == engine and
            assessment['all_script_controls_pass'] is True and
            assessment['design_sha256'] == digest(raw['bound-release-design.json']),
            'Saved API assessment/design mismatch')
    require(len(assessment['cases']) == len(design['cases']) == 11 and
            design['cases'] == json.loads(original_protocols)['cases'] and
            [c['id'] for c in assessment['cases']] == [c['id'] for c in design['cases']] and
            all(c['status'] == 'PASS' and c['criteria'] and
                all(v is True for v in c['criteria'].values())
                for c in assessment['cases']), 'Missing original API criteria')
    require(design['verified_release_receipt_sha256'] == digest(raw['verified-release.json'])
            and design['authenticated_artifact_engine'] == verified['engine'] and
            design['api_revision'] == source, 'API artifact receipt mismatch')
    run = docs['production-run.json']
    metadata = docs['release-artifact-metadata.json']
    require(run['source'] == source and run['run'] == verified['workflow_run'] ==
            metadata['workflow_run']['id'] and metadata['workflow_run']['head_sha'] == source
            and metadata['id'] == verified['artifact_id'] and metadata['expired'] is False,
            'Actual production workflow/artifact mismatch')
    archive_sha = hash_value(verified['artifact_sha256'])
    require(metadata['digest'] == 'sha256:' + archive_sha and
            all(docs[n]['sha256'] == archive_sha for n in
                ('EXT4-ARTIFACT-VERIFIED.json', 'STORAGE-ARTIFACT-VERIFIED.json')),
            'Verified transfer identity mismatch')
    require(docs['EXT4-ARTIFACT-VERIFIED.json']['ext4_readback_verified'] is True and
            docs['STORAGE-ARTIFACT-VERIFIED.json']['storage_readback_verified'] is True and
            type(docs['EXT4-ARTIFACT-VERIFIED.json']['bytes']) is int and
            docs['EXT4-ARTIFACT-VERIFIED.json']['bytes'] > 0 and
            docs['EXT4-ARTIFACT-VERIFIED.json']['bytes'] ==
            docs['STORAGE-ARTIFACT-VERIFIED.json']['bytes'],
            'Artifact transfer/readback not complete')
    assets = verified['frontend_assets']
    require(set(assets) == ASSETS | {'rest.html'}, 'Actual frontend closure incomplete')
    for row in assets.values():
        hash_value(row['sha256'])
        require(type(row['bytes']) is int and row['bytes'] > 0, 'Empty frontend asset')
    require(design['client_sha256'] == assets['rest.html']['sha256'],
            'REST HTML is not the authenticated client')
    reference = assessment['native_browser_reference']
    require(reference['engine'] == engine, 'Native bridge reference uses another engine')
    actual_reference = [c for c in assessment['cases']
                        if c['id'] == 'uncached-native-browser-reference']
    require(len(actual_reference) == 1, 'Missing unique actual native reference case')
    actual_reference = actual_reference[0]
    for key in ('script_sha256', 'response_sha256'):
        hash_value(reference[key])
        require(reference[key] == actual_reference[key],
                'Native bridge reference differs from actual accepted case: ' + key)
    for key in ('ph', 'ionic_strength'):
        value = reference['solution'][key]
        require(type(value) in (int, float) and math.isfinite(value),
                'Nonfinite actual native reference')
        require(value == actual_reference['metrics'][key],
                'Native bridge reference differs from actual accepted metric: ' + key)
    require(reference['solution']['ionic_strength'] >= 0, 'Negative native ionic strength')
    binding = copy.deepcopy(template)
    binding.update(expected_engine_source=source,
                   expected_binary_sha256=engine['binary_sha256'],
                   client_sha256=assets['rest.html']['sha256'],
                   frontend_assets={n: assets[n]['sha256'] for n in sorted(ASSETS)},
                   native_bridge_reference=reference,
                   producer_artifact_receipt_sha256=digest(raw['verified-release.json']),
                   protocol_design_sha256=digest(raw['bound-release-design.json']))
    prior = dict(API_receipt_sha256=digest(raw['API-ACCEPTED.json']),
                 verified_artifact_receipt_sha256=digest(raw['verified-release.json']),
                 native_binary_sha256=engine['binary_sha256'],
                 native_actual_producer_lock_sha256=engine['cargo_lock_sha256'],
                 frontend_assets=binding['frontend_assets'],
                 client_sha256=binding['client_sha256'])
    return binding, prior


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--production', type=Path, required=True)
    parser.add_argument('--template', type=Path, required=True)
    parser.add_argument('--template-sha256', required=True)
    parser.add_argument('--original-protocols', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    # Read all prerequisites before creating output. Missing actual new receipts
    # are a hold, never populated with historical or guessed identities.
    template_raw = args.template.read_bytes()
    require(digest(template_raw) == args.template_sha256, 'Unreviewed template bytes')
    raw = {n: (args.production / n).read_bytes() for n in RECEIPTS}
    original = args.original_protocols.read_bytes()
    binding, prior = checked_binding(json.loads(template_raw)['binding'], raw, SOURCE, TREE, original)
    heat = dict(binding, reviewed_candidate_source=CANDIDATE)
    args.out.mkdir(mode=0o700, exist_ok=False)
    for name, value in [('release-binding.json', binding),
                        ('heat-release-binding.json', heat),
                        ('actual-prior-release.json', prior),
                        ('ACTUAL-RECEIPT-BINDING.json', {
                            'state': 'OFFLINE_RECEIPT_CONSISTENCY_ONLY_NOT_BROWSER_ACCEPTANCE',
                            'runtime_source': SOURCE, 'runtime_tree': TREE,
                            'reviewed_candidate_source': CANDIDATE,
                            'receipt_files_sha256': {n: digest(b) for n, b in raw.items()},
                            'template_sha256': args.template_sha256,
                            'original_protocols_sha256': digest(original),
                            'builder_sha256': digest(Path(__file__).read_bytes()),
                            'issue_calls': 0, 'browser_acceptance_claim': False})]:
        with (args.out / name).open('xb') as stream:
            stream.write(json.dumps(value, indent=2, allow_nan=False).encode() + b'\n')


if __name__ == '__main__':
    main()
