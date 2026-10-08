#!/usr/bin/env python3
"""Execute only the independently frozen sixth forecast on its bound executable."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
ROOT=Path(__file__).resolve().parents[2]
FORECAST_SHA='45e554283e8dd27e9483d45fe9cd33dd66a2d4fa208a8e47b7d3b2fb99ea3a0d'
SEMANTIC_SHA='9296a251a40c432155f841eae8dba48e1ab4bf8c358be4cd4ef47b1f401acd66'
BINARY_SHA='fcf424a65384c5ef5949960f228fda4e1c3e893ec93c65d58fe5b12064352967'
def digest(path):
    h=hashlib.sha256()
    with Path(path).open('rb') as f:
        for block in iter(lambda:f.read(1024*1024),b''): h.update(block)
    return h.hexdigest()
def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--binary',type=Path,required=True)
    parser.add_argument('--forecast',type=Path,required=True)
    parser.add_argument('--out',type=Path,required=True)
    parser.add_argument('--mode',choices=['sixth-fifty','semantic-replay'],default='sixth-fifty')
    args=parser.parse_args()
    expected=SEMANTIC_SHA if args.mode=='semantic-replay' else FORECAST_SHA
    if digest(args.forecast)!=expected: raise SystemExit('Frozen forecast hash mismatch')
    if digest(args.binary)!=BINARY_SHA: raise SystemExit('Bound executable hash mismatch')
    forecast=json.loads(args.forecast.read_text())
    ids=([f'S{i:02}' for i in [4,5,7,8,12,13,45,46,47,48,50]] if args.mode=='semantic-replay' else [f'S{i:02}' for i in range(1,51)])
    if [c['id'] for c in forecast['cases']] != ids:
        raise SystemExit('Frozen case identities mismatch')
    path=ROOT/'tools/fifth-independent-50/run.py'
    spec=importlib.util.spec_from_file_location('cli_evidence',path)
    shared=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(shared)
    args.binary=args.binary.resolve()
    args.out.mkdir(parents=True,exist_ok=False)
    # S-prefixed cases never inherit the earlier F-prefixed refusal exceptions.
    return shared.experiments(args,forecast,args.out)
if __name__=='__main__': sys.exit(main())
