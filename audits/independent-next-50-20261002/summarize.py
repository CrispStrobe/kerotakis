"""Summarize captured evidence; this does not decide physical agreement."""
import argparse
import json
from pathlib import Path

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('results', type=Path)
args = p.parse_args()
receipt = json.loads((args.results / 'execution.json').read_text())
rows = []
for run in receipt['runs']:
    if run['mode'] != 'json':
        continue
    path = args.results / run['stdout']
    parsed = []
    errors = []
    for line_no, line in enumerate(path.read_text().splitlines(), 1):
        try:
            parsed.append(json.loads(line, parse_constant=lambda x: (_ for _ in ()).throw(ValueError(x))))
        except (ValueError, json.JSONDecodeError) as exc:
            errors.append(dict(line=line_no, error=str(exc)))
    rows.append(dict(id=run['id'], exit_code=run['exit_code'], json_lines=len(parsed), parse_errors=errors,
                     stderr=(args.results / run['stderr']).read_text(),
                     events=[dict(step=step.get('step'), operator=step.get('operator'), events=step.get('events', []))
                             for step in parsed if step.get('events')],
                     final_bench=next((step['bench'] for step in reversed(parsed) if 'bench' in step), None)))
(args.results / 'structural-summary.json').write_text(json.dumps(rows, indent=2) + '\n')
print(json.dumps(dict(cases=len(rows), cli_failures=[r['id'] for r in rows if r['exit_code']],
                      malformed_json=[r['id'] for r in rows if r['parse_errors']]), indent=2))
