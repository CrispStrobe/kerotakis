#!/usr/bin/env python3
"""QA003 supplemental reported-scene-volume assessment; no engine execution."""
import hashlib
import json
from pathlib import Path
import zipfile

HERE = Path(__file__).resolve().parent
RESULTS = HERE.parent

def sha(data):
    return hashlib.sha256(data).hexdigest()

archive = RESULTS / 'raw-artifact.zip'
metadata = json.loads((RESULTS / 'artifact-metadata.json').read_text())
assert sha(archive.read_bytes()) == metadata['digest'].removeprefix('sha256:')
saved = json.loads((RESULTS / 'cases/batch2_qa-003.json').read_text())
with zipfile.ZipFile(archive) as zipped:
    raw = zipped.read('batch2_qa-003/stdout.jsonl')
assert sha(raw) == saved['execution']['stdout_sha256']
rows = [json.loads(line) for line in raw.splitlines()]
measurements = [row for row in rows if row['operator'].get('op') == 'measure']
assert len(measurements) == 2
volumes = {}
for row in measurements:
    vessel_id = row['operator']['vessel']
    assert row['operator']['instrument'] == 'volume_meter'
    assert any(event.get('reason', {}).get('key') == 'not-modeled.volume-meter-on-an-open-vessel' for event in row['events'])
    scene = next(v for v in row['scene']['vessels'] if v['id'] == vessel_id)
    volumes[vessel_id] = scene['liquid']['volume_l']
assert volumes == {0: 0.15, 1: 0.05}
titrations = [event for row in rows for event in row['events'] if event['event'] == 'titrated']
assert all(event['endpoint_reached'] for event in titrations)
doses = {event['vessel']: event['concentration'] * event['total_volume'] for event in titrations}
concentrations = {v: doses[v] / volumes[v] for v in [0, 1]}
# Original criterion permits dosing resolution; use two original increments.
resolution = 2 * 0.1 * 0.0001 / min(volumes.values())
difference = abs(concentrations[0] - concentrations[1])
assert difference <= resolution
result = {
    'id': 'batch2:qa-003', 'source_engine': saved['engine'],
    'stdout_sha256': sha(raw), 'original_script_sha256': saved['original_script_sha256'],
    'original_criterion': saved['original_acceptance_checks'][1],
    'reported_pre_titration_scene_liquid_volume_l': volumes,
    'base_demand_mol': doses, 'reconstructed_acid_concentration_mol_l': concentrations,
    'concentration_difference_mol_l': difference, 'two_increment_resolution_mol_l': resolution,
    'physical_concentration_forecast': 'met_using_existing_reported_liquid_volumes',
    'requested_gas_meter_readings': 'both still unsupported; no liquid instrument reading fabricated',
    'qualification': 'Scene liquid volumes are computed model observables, not independent instrument readings or empirical calibration. The first reader omitted scene payloads; its result remains preserved separately.',
    'new_engine_executions': 0, 'catalog_additions': 0,
}
print(json.dumps(result, indent=2, allow_nan=False))
