#!/usr/bin/env python3
"""I18N-3: every engine term substituted by the web shell has German.

The required set is derived from the records/code that can emit it.  There is
no second hand-maintained vocabulary list: registry names and visual words,
safety labels, lesson ids, and the core locale's value-lookup sections are the
sources of truth.
"""
from __future__ import annotations
import json, pathlib, re, sys, tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent

def required(root: pathlib.Path = ROOT) -> dict[str, set[str]]:
    registry=json.loads((root/'data/registry/registry-source-v1.json').read_text())
    species={x['name'] for x in registry['identities']}
    visual=set()
    for x in registry.get('optical',[]):
        for key in ('appearance','flame_colour'):
            if isinstance(x.get(key),str): visual.add(x[key])
    appearance=(root/'crates/kerotakis-core/src/appearance.rs').read_text()
    visual |= set(re.findall(r'=>\s*"([a-z][a-z -]+)"', appearance))
    safety=(root/'crates/kerotakis-safety/src/lib.rs').read_text()
    body=safety[safety.index('pub fn hazard_assessment'):safety.index('pub fn groups')]
    hazards=set(re.findall(r'=>\s*"([a-z][a-z_]+)"',body))
    lessons={p.stem.replace('-',' ') for p in (root/'lessons').glob('*.lab')}
    # These sections are looked up by their emitted English value in the core
    # renderer; their keys are wire identifiers and are intentionally excluded.
    with open(root/'crates/kerotakis-core/i18n/de.toml','rb') as stream:
        core=tomllib.load(stream)
    apparatus=set()
    for section in ('glassware','instrument','verb'):
        apparatus |= set(core.get(section,{}))
    return {'species':species,'colours':visual,'hazards':hazards,'lessons':lessons,'apparatus/events':apparatus}

def audit(root: pathlib.Path = ROOT) -> list[str]:
    doc=json.loads((root/'web/app/src/locales/de.json').read_text())
    terms=doc.get('terms',{})
    problems=[]
    for category, words in required(root).items():
        for word in sorted(words):
            value=terms.get(word)
            # Cognates and loanwords (orange, Isopropanol, Pipette) can be
            # correct German while byte-identical to English. Presence and a
            # non-blank answer are the invariant here.
            if not isinstance(value,str) or not value.strip():
                problems.append(f"{category}: missing German term for {word!r}")
    return problems

if __name__ == '__main__':
    problems=audit()
    for p in problems: print(p)
    total=sum(map(len,required().values()))
    print(f"\n{total} derived engine terms; {len(problems)} uncovered")
    raise SystemExit(1 if problems else 0)
