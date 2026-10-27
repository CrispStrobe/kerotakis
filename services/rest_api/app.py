"""FastAPI worker or Vercel gateway. Execution is disabled without configuration."""
from __future__ import annotations

import asyncio
import base64
from contextlib import closing
from dataclasses import dataclass, field
import hashlib
import hmac
import json
import logging
import math
import os
from pathlib import Path
import re
import signal
import shutil
import sqlite3
import sys
import tempfile
import time
from urllib.parse import urlsplit
from typing import Literal
import uuid

from fastapi import Depends, FastAPI, HTTPException, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.security import HTTPAuthorizationCredentials, HTTPBearer
import httpx
from pydantic import BaseModel, ConfigDict, Field, ValidationError

BODY_LIMIT = 20_000
SCRIPT_LIMIT = 16_000
OUTPUT_LIMIT = 1_000_000
CPU_SECONDS = 12
WALL_SECONDS = 18
LEASE_SECONDS = 90
# English-only simulation commands. Filesystem, language and game/session
# controls remain excluded; register selects presentation only. Explicit loop
# budgets complement the independent child CPU/wall/output/memory limits.
VERBS = frozenset("new add heat cool wait stir seal open "
                  "ignite filter decant drain measure "
                  "chromatograph electrolyse electrolyze cell grind centrifuge "
                  "irradiate dilute magnet react test inspect particles sweep "
                  "register look observe explain evaporate regulate mix "
                  "extract distil distill titrate transport "
                  "remove discard stock smell waft zoom voltmeter".split())
# These shell words either emit plain text or are no-ops in `kero run --json`.
# Admitting them as if they were observations would silently mislead clients.
UNAVAILABLE_SESSION_WORDS = frozenset(
    "species help structure identify coverage materials find lang quest save load quit exit".split())
# Only multiword identity aliases affect parameter positions. Keep this small
# policy view aligned with the engine inventory; the submitted script is intact.
# BEGIN generated multi-word aliases (tools/gen_api_multiword_aliases.py)
MULTIWORD_INGREDIENT_ALIASES = {
    ('acetate', 'ion'): 'CH3COO-',
    ('acetic', 'acid'): 'CH3COOH',
    ('acide', 'acétique'): 'CH3COOH',
    ('acide', 'ascorbique'): 'ascorbic_acid',
    ('acide', 'ascorbique', '(vitamine', 'c)'): 'ascorbic_acid',
    ('acide', 'borique'): 'H3BO3',
    ('acide', 'chlorhydrique'): 'HCl',
    ('acide', 'citrique'): 'citric_acid',
    ('acide', 'déhydroascorbique'): 'dehydroascorbic_acid',
    ('acide', 'hypochloreux'): 'HClO',
    ('acide', 'lactique'): 'lactic_acid',
    ('acide', 'malique'): 'malic_acid',
    ('acide', 'oxalique'): 'H2C2O4',
    ('acide', 'phosphorique'): 'H3PO4',
    ('acide', 'sulfurique'): 'H2SO4',
    ('activated', 'charcoal'): 'activated_charcoal',
    ('acétate', "d'éthyle"): 'ethyl_acetate',
    ('acétate', 'de', 'sodium'): 'NaOAc',
    ('ammonia', 'solution'): 'NH3',
    ('ammonium', 'chloride'): 'NH4Cl',
    ('ammonium', 'ion'): 'NH4+',
    ('anthocyan', '(rotkohl-farbstoff)'): 'anthocyanin',
    ('anthocyane', '(pigment', 'du', 'chou', 'rouge)'): 'anthocyanin',
    ('anthocyanin', '(red-cabbage', 'pigment)'): 'anthocyanin',
    ('antlerit', '(grünes', 'kupfersulfat)'): 'antlerite',
    ('antlerite', '(green', 'copper', 'sulfate)'): 'antlerite',
    ('antlérite', '(sulfate', 'de', 'cuivre', 'vert)'): 'antlerite',
    ('ascorbic', 'acid'): 'ascorbic_acid',
    ('ascorbic', 'acid', '(vitamin', 'c)'): 'ascorbic_acid',
    ('ascorbinsäure', '(vitamin', 'c)'): 'ascorbic_acid',
    ('atacamit', '(grüner', 'kupferrost)'): 'atacamite',
    ('atacamite', '(corrosion', 'verte', 'du', 'cuivre)'): 'atacamite',
    ('atacamite', '(green', 'copper', 'corrosion)'): 'atacamite',
    ('azote', 'liquide'): 'liquid_nitrogen',
    ('baking', 'soda', '(sodium', 'bicarbonate)'): 'NaHCO3',
    ('barium', 'chloride'): 'BaCl2',
    ('barium', 'hydroxide'): 'Ba(OH)2',
    ('barium', 'ion'): 'Ba+2',
    ('barium', 'sulfate'): 'BaSO4',
    ('beta-tricalcium', 'phosphate'): 'Ca3(PO4)2',
    ('betanin', '(beetroot', 'red)'): 'betanin',
    ('betanin', '(rote-bete-rot)'): 'betanin',
    ('bicarbonate', 'de', 'soude', '(hydrogénocarbonate', 'de', 'sodium)'): 'NaHCO3',
    ('bicarbonate', 'ion'): 'HCO3-',
    ('bisulfite', 'ion'): 'HSO3-',
    ('bleach', '(sodium', 'hypochlorite)'): 'NaOCl',
    ('bleached', 'hair', 'pigment'): 'hair_pigment_ox',
    ('bleached', 'hair', 'pigment', '(oxidised)'): 'hair_pigment_ox',
    ('bleichmittel', '(natriumhypochlorit)'): 'NaOCl',
    ('bleu', 'de', 'bromothymol'): 'bromothymol_blue',
    ('boric', 'acid'): 'H3BO3',
    ('branntkalk', '(calciumoxid)'): 'CaO',
    ('brochantit', '(grüne', 'kupferpatina)'): 'brochantite',
    ('brochantite', '(green', 'copper', 'patina)'): 'brochantite',
    ('brochantite', '(patine', 'verte', 'du', 'cuivre)'): 'brochantite',
    ('bromide', 'ion'): 'Br-',
    ('bromothymol', 'blue'): 'bromothymol_blue',
    ('bromure', "d'hydrogène"): 'HBr',
    ('bromure', 'de', 'sodium'): 'NaBr',
    ('bétanine', '(rouge', 'de', 'betterave)'): 'betanin',
    ('bétanine', 'oxydée'): 'betanin_ox',
    ('bétanine', 'oxydée', '(incolore)'): 'betanin_ox',
    ('bêta-phosphate', 'tricalcique'): 'Ca3(PO4)2',
    ('calcium', 'chloride'): 'CaCl2',
    ('calcium', 'ion'): 'Ca+2',
    ('carbon', '(charcoal)'): 'C',
    ('carbon', 'dioxide'): 'CO2',
    ('carbon', 'monoxide'): 'CO',
    ('carbonate', 'ion'): 'CO3-2',
    ('carbone', '(charbon', 'de', 'bois)'): 'C',
    ('carmin', "d'indigo"): 'indigo_carmine',
    ('carmin', "d'indigo", '(e132)'): 'indigo_carmine',
    ('carmin', "d'indigo", 'oxydé'): 'indigo_carmine_ox',
    ('carmin', "d'indigo", 'oxydé', '(incolore)'): 'indigo_carmine_ox',
    ('cellulose', '(anhydroglucose', 'unit)'): 'cellulose',
    ('cellulose', '(anhydroglucose-einheit)'): 'cellulose',
    ('cellulose', '(unité', 'anhydroglucose)'): 'cellulose',
    ('chalk', '(calcium', 'carbonate)'): 'CaCO3',
    ('charbon', 'actif'): 'activated_charcoal',
    ('chaux', 'vive'): 'CaO',
    ('chaux', 'vive', '(oxyde', 'de', 'calcium)'): 'CaO',
    ('chaux', 'éteinte'): 'Ca(OH)2',
    ('chaux', 'éteinte', '(hydroxyde', 'de', 'calcium)'): 'Ca(OH)2',
    ('chloride', 'ion'): 'Cl-',
    ('chlorine', 'gas'): 'Cl2',
    ('chlorophyll', 'a'): 'chlorophyll',
    ('chlorophylle', 'a'): 'chlorophyll',
    ('chlorure', "d'ammonium"): 'NH4Cl',
    ('chlorure', "d'argent"): 'AgCl',
    ('chlorure', 'de', 'baryum'): 'BaCl2',
    ('chlorure', 'de', 'calcium'): 'CaCl2',
    ('chlorure', 'de', 'fer(iii)'): 'FeCl3',
    ('chlorure', 'de', 'potassium'): 'KCl',
    ('chlorure', 'de', 'sodium'): 'NaCl',
    ('citrate', 'ion'): 'C6H5O7-3',
    ('citric', 'acid'): 'citric_acid',
    ('copper', 'sulfate'): 'CuSO4',
    ('copper(i)', 'ion'): 'Cu+1',
    ('copper(ii)', 'hydroxide'): 'Cu(OH)2',
    ('copper(ii)', 'ion'): 'Cu+2',
    ('copper(ii)', 'oxide'): 'CuO',
    ('copper(ii)', 'sulfate', 'pentahydrate'): 'chalcanthite',
    ('copper(ii)', 'sulfate', 'pentahydrate', '(chalcanthite)'): 'chalcanthite',
    ('craie', '(carbonate', 'de', 'calcium)'): 'CaCO3',
    ('cristaux', 'de', 'soude', '(carbonate', 'de', 'sodium)'): 'Na2CO3',
    ('curcumin', '(kurkuma-gelb)'): 'curcumin',
    ('curcumin', '(turmeric', 'yellow)'): 'curcumin',
    ('curcumine', '(jaune', 'de', 'curcuma)'): 'curcumin',
    ('curcumine', 'oxydée'): 'curcumin_ox',
    ('curcumine', 'oxydée', '(incolore)'): 'curcumin_ox',
    ('d-fructose', '(fruchtzucker)'): 'fructose',
    ('d-fructose', '(fruit', 'sugar)'): 'fructose',
    ('d-fructose', '(sucre', 'de', 'fruit)'): 'fructose',
    ('d-glucose', '(dextrose)'): 'glucose',
    ('d-glucose', '(traubenzucker)'): 'glucose',
    ('dehydroascorbic', 'acid'): 'dehydroascorbic_acid',
    ('dihydrogen', 'phosphate'): 'H2PO4-',
    ('dihydrogen', 'phosphate', 'ion'): 'H2PO4-',
    ('dioxyde', 'de', 'carbone'): 'CO2',
    ('dioxyde', 'de', 'carbone', 'dissous'): 'CO2(aq)',
    ('dioxyde', 'de', 'manganèse'): 'MnO2',
    ('dioxyde', 'de', 'silicium'): 'SiO2',
    ('dioxyde', 'de', 'silicium', '(quartz)'): 'SiO2',
    ('dioxyde', 'de', 'soufre'): 'SO2',
    ('dissolved', 'carbon', 'dioxide'): 'CO2(aq)',
    ('dry', 'ice'): 'dry_ice',
    ('eau', 'de', 'javel', '(hypochlorite', 'de', 'sodium)'): 'NaOCl',
    ('eisen(iii)-oxid', '(hämatit)'): 'Fe2O3',
    ('flüssiger', 'stickstoff'): 'liquid_nitrogen',
    ('gebleichtes', 'haarpigment'): 'hair_pigment_ox',
    ('gebleichtes', 'haarpigment', '(oxidiert)'): 'hair_pigment_ox',
    ('gelöstes', 'kohlenstoffdioxid'): 'CO2(aq)',
    ('gips', '(calciumsulfat-dihydrat)'): 'gypsum',
    ('gypse', '(sulfate', 'de', 'calcium', 'dihydraté)'): 'gypsum',
    ('gypsum', '(calcium', 'sulfate', 'dihydrate)'): 'gypsum',
    ('haarpigment', '(eumelanin-wiederholungseinheit)'): 'hair_pigment',
    ('hair', 'pigment'): 'hair_pigment',
    ('hair', 'pigment', '(eumelanin', 'repeat', 'unit)'): 'hair_pigment',
    ('hydrochloric', 'acid'): 'HCl',
    ('hydrogen', 'bromide'): 'HBr',
    ('hydrogen', 'iodide'): 'HI',
    ('hydrogen', 'ion'): 'H+',
    ('hydrogen', 'peroxide'): 'H2O2',
    ('hydrogen', 'phosphate'): 'HPO4-2',
    ('hydrogen', 'phosphate', 'ion'): 'HPO4-2',
    ('hydrogen', 'sulfide'): 'hydrogen_sulfide',
    ('hydrogénosulfate', 'de', 'sodium'): 'NaHSO4',
    ('hydrogénosulfite', 'de', 'sodium'): 'NaHSO3',
    ('hydroxide', 'ion'): 'OH-',
    ('hydroxyde', 'de', 'baryum'): 'Ba(OH)2',
    ('hydroxyde', 'de', 'cuivre(ii)'): 'Cu(OH)2',
    ('hydroxyde', 'de', 'fer(ii)'): 'Fe(OH)2',
    ('hydroxyde', 'de', 'fer(iii)'): 'Fe(OH)3',
    ('hydroxyde', 'de', 'magnésium'): 'Mg(OH)2',
    ('hydroxyde', 'de', 'potassium'): 'KOH',
    ('hydroxyde', 'de', 'sodium'): 'NaOH',
    ('hydroxyde', 'de', 'zinc'): 'Zn(OH)2',
    ('hypochlorige', 'säure'): 'HClO',
    ('hypochlorite', 'ion'): 'ClO-',
    ('hypochlorous', 'acid'): 'HClO',
    ('indigo', 'carmine'): 'indigo_carmine',
    ('indigo', 'carmine', '(e132)'): 'indigo_carmine',
    ('indigocarmin', '(e132)'): 'indigo_carmine',
    ('iodate', 'de', 'potassium'): 'KIO3',
    ('iodide', 'ion'): 'I-',
    ('iodure', "d'hydrogène"): 'HI',
    ('iodure', 'de', 'potassium'): 'KI',
    ('ion', 'acétate'): 'CH3COO-',
    ('ion', 'ammonium'): 'NH4+',
    ('ion', 'argent'): 'Ag+',
    ('ion', 'baryum'): 'Ba+2',
    ('ion', 'bromure'): 'Br-',
    ('ion', 'calcium'): 'Ca+2',
    ('ion', 'carbonate'): 'CO3-2',
    ('ion', 'chlorure'): 'Cl-',
    ('ion', 'citrate'): 'C6H5O7-3',
    ('ion', 'cuivre(i)'): 'Cu+1',
    ('ion', 'cuivre(ii)'): 'Cu+2',
    ('ion', 'dihydrogénophosphate'): 'H2PO4-',
    ('ion', 'fer(ii)'): 'Fe+2',
    ('ion', 'fer(iii)'): 'Fe+3',
    ('ion', 'hydrogène'): 'H+',
    ('ion', 'hydrogénocarbonate'): 'HCO3-',
    ('ion', 'hydrogénophosphate'): 'HPO4-2',
    ('ion', 'hydrogénosulfite'): 'HSO3-',
    ('ion', 'hydroxyde'): 'OH-',
    ('ion', 'hypochlorite'): 'ClO-',
    ('ion', 'iodure'): 'I-',
    ('ion', 'lactate'): 'lactate',
    ('ion', 'magnésium'): 'Mg+2',
    ('ion', 'manganate'): 'MnO4-2',
    ('ion', 'manganèse(ii)'): 'Mn+2',
    ('ion', 'manganèse(iii)'): 'Mn+3',
    ('ion', 'nitrate'): 'NO3-',
    ('ion', 'nitrite'): 'NO2-',
    ('ion', 'permanganate'): 'MnO4-',
    ('ion', 'phosphate'): 'PO4-3',
    ('ion', 'plomb(ii)'): 'Pb+2',
    ('ion', 'potassium'): 'K+',
    ('ion', 'sodium'): 'Na+',
    ('ion', 'strontium'): 'Sr+2',
    ('ion', 'sulfate'): 'SO4-2',
    ('ion', 'sulfite'): 'SO3-2',
    ('ion', 'thiocyanate'): 'SCN-',
    ('ion', 'thiosulfate'): 'S2O3-2',
    ('ion', 'tétrathionate'): 'S4O6-2',
    ('ion', 'zinc'): 'Zn+2',
    ('iron(ii)', 'hydroxide'): 'Fe(OH)2',
    ('iron(ii)', 'ion'): 'Fe+2',
    ('iron(ii)', 'sulfate'): 'FeSO4',
    ('iron(iii)', 'chloride'): 'FeCl3',
    ('iron(iii)', 'hydroxide'): 'Fe(OH)3',
    ('iron(iii)', 'ion'): 'Fe+3',
    ('iron(iii)', 'oxide'): 'Fe2O3',
    ('iron(iii)', 'oxide', '(hematite)'): 'Fe2O3',
    ('kaliumthiocyanat', '(wässriges', 'reagenz)'): 'KSCN',
    ('kohlenstoff', '(holzkohle)'): 'C',
    ('kreide', '(calciumcarbonat)'): 'CaCO3',
    ('kupfer(ii)-sulfat-pentahydrat', '(chalkanthit)'): 'chalcanthite',
    ('lactate', 'ion'): 'lactate',
    ('lactic', 'acid'): 'lactic_acid',
    ('langit', '(blaugrünes', 'kupfersulfat)'): 'langite',
    ('langite', '(blue-green', 'copper', 'sulfate)'): 'langite',
    ('langite', '(sulfate', 'de', 'cuivre', 'bleu-vert)'): 'langite',
    ('lead(ii)', 'ion'): 'Pb+2',
    ('lead(ii)', 'nitrate'): 'Pb(NO3)2',
    ('liquid', 'nitrogen'): 'liquid_nitrogen',
    ('löschkalk', '(calciumhydroxid)'): 'Ca(OH)2',
    ('magnesium', 'hydroxide'): 'Mg(OH)2',
    ('magnesium', 'ion'): 'Mg+2',
    ('magnesium', 'oxide'): 'MgO',
    ('magnesium', 'sulfate'): 'MgSO4',
    ('magnesium', 'sulfate', 'heptahydrate', '(epsomite)'): 'epsomite',
    ('magnesiumsulfat-heptahydrat', '(epsomit)'): 'epsomite',
    ('malic', 'acid'): 'malic_acid',
    ('manganate', 'ion'): 'MnO4-2',
    ('manganese', 'dioxide'): 'MnO2',
    ('manganese(ii)', 'ion'): 'Mn+2',
    ('manganese(iii)', 'ion'): 'Mn+3',
    ('methyl', 'orange'): 'methyl_orange',
    ('monoxyde', 'de', 'carbone'): 'CO',
    ('natriumtetraborat', '(borax)'): 'Na2B4O7',
    ('natron', '(natriumhydrogencarbonat)'): 'NaHCO3',
    ('neige', 'carbonique'): 'dry_ice',
    ('nitrate', "d'argent"): 'AgNO3',
    ('nitrate', 'de', 'plomb(ii)'): 'Pb(NO3)2',
    ('nitrate', 'de', 'potassium'): 'KNO3',
    ('nitrate', 'de', 'sodium'): 'NaNO3',
    ('nitrate', 'ion'): 'NO3-',
    ('nitrite', 'ion'): 'NO2-',
    ('nylon', '6,6'): 'nylon',
    ('nylon', '6,6', '(repeat', 'unit)'): 'nylon',
    ('nylon', '6,6', '(unité', 'de', 'répétition)'): 'nylon',
    ('nylon', '6,6', '(wiederholungseinheit)'): 'nylon',
    ('octacalcium', 'phosphate'): 'octacalcium_phosphate',
    ('orangé', 'de', 'méthyle'): 'methyl_orange',
    ('oxalic', 'acid'): 'H2C2O4',
    ('oxidiertes', 'betanin'): 'betanin_ox',
    ('oxidiertes', 'betanin', '(farblos)'): 'betanin_ox',
    ('oxidiertes', 'curcumin'): 'curcumin_ox',
    ('oxidiertes', 'curcumin', '(farblos)'): 'curcumin_ox',
    ('oxidiertes', 'indigocarmin'): 'indigo_carmine_ox',
    ('oxidiertes', 'indigocarmin', '(farblos)'): 'indigo_carmine_ox',
    ('oxidised', 'betanin'): 'betanin_ox',
    ('oxidised', 'betanin', '(colourless)'): 'betanin_ox',
    ('oxidised', 'curcumin'): 'curcumin_ox',
    ('oxidised', 'curcumin', '(colourless)'): 'curcumin_ox',
    ('oxidised', 'indigo', 'carmine'): 'indigo_carmine_ox',
    ('oxidised', 'indigo', 'carmine', '(colourless)'): 'indigo_carmine_ox',
    ('oxyde', 'de', 'cuivre(ii)'): 'CuO',
    ('oxyde', 'de', 'fer(iii)'): 'Fe2O3',
    ('oxyde', 'de', 'fer(iii)', '(hématite)'): 'Fe2O3',
    ('oxyde', 'de', 'magnésium'): 'MgO',
    ('permanganate', 'de', 'potassium'): 'KMnO4',
    ('permanganate', 'ion'): 'MnO4-',
    ('phosphate', 'ion'): 'PO4-3',
    ('phosphate', 'octacalcique'): 'octacalcium_phosphate',
    ('phosphoric', 'acid'): 'H3PO4',
    ('pigment', 'capillaire'): 'hair_pigment',
    ('pigment', 'capillaire', '(unité', 'de', 'répétition', 'de', "l'eumélanine)"): 'hair_pigment',
    ('pigment', 'capillaire', 'décoloré'): 'hair_pigment_ox',
    ('pigment', 'capillaire', 'décoloré', '(oxydé)'): 'hair_pigment_ox',
    ('poly(alcool', 'vinylique)'): 'PVA',
    ('poly(vinyl', 'alcohol)'): 'PVA',
    ('polyethylen', '(hdpe)'): 'PE',
    ('polyethylene', '(hdpe)'): 'PE',
    ('polyethylene', 'terephthalate'): 'PET',
    ('polyéthylène', '(pehd)'): 'PE',
    ('polyéthylène', 'téréphtalate'): 'PET',
    ('potassium', 'chloride'): 'KCl',
    ('potassium', 'hydroxide'): 'KOH',
    ('potassium', 'iodate'): 'KIO3',
    ('potassium', 'iodide'): 'KI',
    ('potassium', 'ion'): 'K+',
    ('potassium', 'nitrate'): 'KNO3',
    ('potassium', 'permanganate'): 'KMnO4',
    ('potassium', 'thiocyanate'): 'KSCN',
    ('potassium', 'thiocyanate', '(aqueous', 'reagent)'): 'KSCN',
    ('quicklime', '(calcium', 'oxide)'): 'CaO',
    ('siliciumdioxid', '(quarz)'): 'SiO2',
    ('silicon', 'dioxide'): 'SiO2',
    ('silicon', 'dioxide', '(quartz)'): 'SiO2',
    ('silver', 'chloride'): 'AgCl',
    ('silver', 'ion'): 'Ag+',
    ('silver', 'nitrate'): 'AgNO3',
    ('slaked', 'lime'): 'Ca(OH)2',
    ('slaked', 'lime', '(calcium', 'hydroxide)'): 'Ca(OH)2',
    ('sodium', 'acetate'): 'NaOAc',
    ('sodium', 'bisulfate'): 'NaHSO4',
    ('sodium', 'bisulfite'): 'NaHSO3',
    ('sodium', 'bromide'): 'NaBr',
    ('sodium', 'chloride'): 'NaCl',
    ('sodium', 'hydroxide'): 'NaOH',
    ('sodium', 'ion'): 'Na+',
    ('sodium', 'nitrate'): 'NaNO3',
    ('sodium', 'sulfate'): 'Na2SO4',
    ('sodium', 'sulfite'): 'Na2SO3',
    ('sodium', 'tetraborate'): 'Na2B4O7',
    ('sodium', 'tetraborate', '(borax)'): 'Na2B4O7',
    ('sodium', 'thiosulfate'): 'Na2S2O3',
    ('strontium', 'ion'): 'Sr+2',
    ('sulfate', 'de', 'baryum'): 'BaSO4',
    ('sulfate', 'de', 'cuivre'): 'CuSO4',
    ('sulfate', 'de', 'cuivre(ii)', 'pentahydraté'): 'chalcanthite',
    ('sulfate', 'de', 'cuivre(ii)', 'pentahydraté', '(chalcanthite)'): 'chalcanthite',
    ('sulfate', 'de', 'fer(ii)'): 'FeSO4',
    ('sulfate', 'de', 'magnésium'): 'MgSO4',
    ('sulfate', 'de', 'magnésium', 'heptahydraté'): 'epsomite',
    ('sulfate', 'de', 'magnésium', 'heptahydraté', '(epsomite)'): 'epsomite',
    ('sulfate', 'de', 'sodium'): 'Na2SO4',
    ('sulfate', 'de', 'zinc'): 'ZnSO4',
    ('sulfate', 'ion'): 'SO4-2',
    ('sulfite', 'de', 'sodium'): 'Na2SO3',
    ('sulfite', 'ion'): 'SO3-2',
    ('sulfur', 'dioxide'): 'SO2',
    ('sulfure', "d'hydrogène"): 'hydrogen_sulfide',
    ('sulfuric', 'acid'): 'H2SO4',
    ('tetrathionate', 'ion'): 'S4O6-2',
    ('thiocyanate', 'de', 'potassium'): 'KSCN',
    ('thiocyanate', 'de', 'potassium', '(réactif', 'aqueux)'): 'KSCN',
    ('thiocyanate', 'ion'): 'SCN-',
    ('thiosulfate', 'de', 'sodium'): 'Na2S2O3',
    ('thiosulfate', 'ion'): 'S2O3-2',
    ('tétraborate', 'de', 'sodium', '(borax)'): 'Na2B4O7',
    ('waschsoda', '(natriumcarbonat)'): 'Na2CO3',
    ('washing', 'soda', '(sodium', 'carbonate)'): 'Na2CO3',
    ('zinc', 'hydroxide'): 'Zn(OH)2',
    ('zinc', 'ion'): 'Zn+2',
    ('zinc', 'sulfate'): 'ZnSO4',
}
# END generated multi-word aliases
MAX_TITRATION_STEPS = 2000
MAX_STAGES = 16
MAX_TRANSPORT_STEPS = 256
MAX_LOOP_WORK = 4096
MAX_COMMAND_SECONDS = 30 * 86400
MAX_SCRIPT_SECONDS = 90 * 86400
bearer = HTTPBearer(auto_error=False)


class RunInput(BaseModel):
    model_config = ConfigDict(extra="forbid", strict=True)
    script: str = Field(min_length=1, max_length=SCRIPT_LIMIT)


class RunOutput(BaseModel):
    execution_status: Literal["completed", "refused", "timeout", "output_limit"]
    exit_status: int | None
    observations: list[dict]
    diagnostics: str
    elapsed_seconds: float
    script_sha256: str
    engine: dict
    physical_verdict: Literal["not_assigned"] = "not_assigned"


def require(condition):
    if not condition:
        raise ValueError("Invalid API configuration")


@dataclass
class Settings:
    mode: str = "disabled"
    executor: str = "direct"
    snapshot: str = ""
    keys: dict[str, str] = field(default_factory=dict)  # principal -> SHA256(key)
    origins: list[str] = field(default_factory=list)
    revision: str = "unknown"
    engine: dict = field(default_factory=dict)
    binary: str = ""
    libraries: str = ""
    loader: str = ""
    state: str = ""
    upstream: str = ""
    redis_url: str = ""
    redis_token: str = ""
    slots: int = 1
    minute: int = 30
    daily: int = 1000
    min_memory_mib: int = 1024
    min_disk_mib: int = 1024
    max_load: float = 3.5

    @classmethod
    def from_env(cls):
        try:
            cfg = cls(mode=os.getenv("KERO_API_MODE", "disabled"),
                      executor=os.getenv("KERO_API_EXECUTOR", "direct"),
                      snapshot=os.getenv("KERO_API_SNAPSHOT", ""),
                      keys=json.loads(os.getenv("KERO_API_KEYS", "{}")),
                      origins=json.loads(os.getenv("KERO_API_ORIGINS", "[]")),
                      revision=os.getenv("VERCEL_GIT_COMMIT_SHA", "unknown"),
                      state=os.getenv("KERO_API_STATE", ""),
                      upstream=os.getenv("KERO_API_UPSTREAM", ""),
                      redis_url=os.getenv("KERO_API_REDIS_URL", os.getenv("UPSTASH_REDIS_REST_URL", os.getenv("KV_REST_API_URL", ""))),
                      redis_token=os.getenv("KERO_API_REDIS_TOKEN", os.getenv("UPSTASH_REDIS_REST_TOKEN", os.getenv("KV_REST_API_TOKEN", ""))),
                      min_memory_mib=int(os.getenv("KERO_API_MIN_MEMORY_MIB", "1024")),
                      min_disk_mib=int(os.getenv("KERO_API_MIN_DISK_MIB", "1024")))
            if cfg.mode in ("worker", "vercel"):
                assets = Path(__file__).parent / "assets"
                manifest = json.loads(Path(os.getenv("KERO_API_ENGINE_MANIFEST", str(assets / "manifest.json"))).read_text())
                cfg.binary = str(Path(os.getenv("KERO_API_BINARY", str(assets / "kero"))).resolve(strict=True))
                require(hashlib.sha256(Path(cfg.binary).read_bytes()).hexdigest() == manifest["binary_sha256"])
                require(all(re.fullmatch(r"[a-f0-9]{64}", manifest[k]) for k in ("binary_sha256", "cargo_lock_sha256")))
                require(re.fullmatch(r"[a-f0-9]{40}", manifest["source_head"]))
                cfg.engine = {k: manifest[k] for k in ("source_head", "binary_sha256", "cargo_lock_sha256")}
                cfg.revision = os.getenv("KERO_API_REVISION", "unknown")
                require(os.access(cfg.binary, os.X_OK) and sys.platform == "linux")
                if cfg.mode == "worker":
                    require(Path(cfg.state).is_absolute())
                    require(Path("/usr/bin/bwrap").is_file() and os.geteuid() != 0)
                else:
                    require(os.getenv("VERCEL") == "1" and os.geteuid() != 0)
                    require(cfg.executor in ("direct", "microvm"))
                    if cfg.executor == "microvm":
                        require(re.fullmatch(r"snap_[A-Za-z0-9]+", cfg.snapshot))
                        binding = json.loads((assets / "microvm.json").read_text())
                        require(binding["snapshot_id"] == cfg.snapshot and binding["engine"] == manifest)
                        for name in ("vm_job", "sandbox"):
                            require(hashlib.sha256((assets / (name + ".runtime")).read_bytes()).hexdigest()
                                    == binding[name + "_sha256"])
                    url = urlsplit(cfg.redis_url)
                    require(url.scheme == "https" and url.hostname and not url.username and not url.password)
                    require(url.path in ("", "/") and not url.query and not url.fragment)
                    require(16 <= len(cfg.redis_token) <= 1024)
                    cfg.libraries = str(assets / "lib")
                    for name, digest in manifest.get("libraries", {}).items():
                        require(isinstance(name, str) and Path(name).name == name)
                        require(hashlib.sha256((assets / "lib" / name).read_bytes()).hexdigest() == digest)
                    if manifest.get("loader"):
                        require(manifest["loader"] in manifest.get("libraries", {}))
                        cfg.loader = str(assets / "lib" / manifest["loader"])
            elif cfg.mode == "gateway":
                url = urlsplit(cfg.upstream)
                require(url.scheme == "https" and url.hostname and not url.username and not url.password)
                require(not url.query and not url.fragment and url.path in ("", "/"))
                cfg.upstream = cfg.upstream.rstrip("/")
            else:
                return cls()
            require(isinstance(cfg.keys, dict) and 0 < len(cfg.keys) <= 64)
            require(all(re.fullmatch(r"[a-zA-Z0-9_-]{1,64}", k) and
                       isinstance(v, str) and re.fullmatch(r"[a-f0-9]{64}", v) for k, v in cfg.keys.items()))
            require(len(set(cfg.keys.values())) == len(cfg.keys))
            require(isinstance(cfg.origins, list) and all(isinstance(o, str) and
                       o.startswith("https://") and urlsplit(o).path == "" for o in cfg.origins))
            require(cfg.min_memory_mib >= 1024)
            require(cfg.min_disk_mib >= 1024)
            return cfg
        except (AssertionError, KeyError, ValueError, TypeError, OSError):
            # No partially configured execution service, no secret/path in logs.
            return cls()


def validate_script(script: str):
    if len(script.encode("utf-8")) > SCRIPT_LIMIT or any(ord(c) < 32 and c not in "\n\r\t" for c in script):
        raise HTTPException(422, "Script exceeds byte limit or contains control characters")
    # Match the CLI's token-boundary comments without truncating embedded
    # hashes. Charge the command itself, not words in an explanatory comment.
    lines = [command for line in script.splitlines()
             if (command := re.split(r"(?<!\S)#", line, maxsplit=1)[0].strip())]
    if not lines or len(lines) > 128:
        raise HTTPException(422, "Supply 1–128 commands")
    for n, line in enumerate(lines, 1):
        words = line.split()
        if words[0] in UNAVAILABLE_SESSION_WORDS:
            raise HTTPException(422, f"Command {n} is unavailable in batch JSON execution")
        if len(words) > 32 or words[0] not in VERBS:
            raise HTTPException(422, f"Command {n} is outside the API command subset")
    if sum(line.split()[0] == "new" for line in lines) > 16:
        raise HTTPException(422, "At most 16 vessel creation commands")
    duration = 0.0
    loop_work = 0
    for line in lines:
        words = line.split()
        verb = words[0]
        position = {"stock": 1, "add": 2, "grind": 2, "titrate": 2, "extract": 3}.get(verb)
        if position is not None:
            for alias, canonical in sorted(MULTIWORD_INGREDIENT_ALIASES.items(), key=lambda row: -len(row[0])):
                if tuple(word.lower() for word in words[position:position + len(alias)]) == alias:
                    words[position:position + len(alias)] = [canonical]
                    break
        if verb == "register" and (len(words) != 2 or words[1] not in ("lv1", "lv2", "lv3")):
            raise HTTPException(422, "Use register lv1, lv2 or lv3")
        if verb in ("remove", "discard") and (
                len(words) != 2 or not re.fullmatch(r"v[1-9][0-9]*", words[1])):
            raise HTTPException(422, "Vessel lifecycle command expects one vessel")
        if verb == "stock" and len(words) != 3:
            raise HTTPException(422, "Stock expects an ingredient and amount")
        if verb in ("cell", "voltmeter") and (
                len(words) != 3 or any(not re.fullmatch(r"v[1-9][0-9]*", word) for word in words[1:])):
            raise HTTPException(422, "Cell measurement expects two vessels")
        if verb in ("look", "observe", "explain", "smell", "waft", "particles", "zoom", "inspect") and (
                len(words) > 2 or len(words) == 2 and not re.fullmatch(r"v[1-9][0-9]*", words[1])):
            raise HTTPException(422, "Observation expects an optional vessel")
        if verb == "evaporate" and len(words) != 3:
            raise HTTPException(422, "Evaporation expects vessel and fraction")
        if verb == "regulate" and len(words) != 4:
            raise HTTPException(422, "Regulation expects vessel, pressure and initial volume")
        if verb == "mix" and (len(words) != 7 or words[5] != "into"):
            raise HTTPException(422, "Mix expects two vessel fractions and a target")
        if verb in ("extract", "distil", "distill"):
            base = 5 if verb == "extract" else 4
            if len(words) == base:
                count = 1
            elif len(words) == base + 2 and words[base] == "stages":
                count = bounded_count(words[base + 1], MAX_STAGES)
            else:
                raise HTTPException(422, "Only a bounded stages count may follow the amount")
            loop_work += count
        if verb == "titrate":
            if len(words) < 7:
                raise HTTPException(422, "Titration needs an increment and endpoint")
            rest = words[4:] if words[3].endswith(("M", "m")) else words[3:]
            if len(rest) < 4 or rest[1] != "until":
                raise HTTPException(422, "Titration needs an increment and endpoint")
            if rest[2] == "ph":
                tail = rest[4:]
            elif rest[2] in ("colour", "color") and rest[3] == "persists":
                tail = rest[4:]
            elif rest[2] == "pe" and len(rest) >= 5 and rest[3] in (">", ">=", "<", "<=", "above", "below"):
                tail = rest[5:]
            else:
                raise HTTPException(422, "Unsupported titration endpoint")
            if not tail:
                count = 100  # The CLI's actual default, not an unbounded loop.
            elif len(tail) == 2 and tail[0] == "max":
                count = bounded_count(tail[1], MAX_TITRATION_STEPS)
            else:
                raise HTTPException(422, "Only a bounded max count may follow the endpoint")
            loop_work += count
        if verb == "transport":
            # Match the canonical form, including the entire suffix: duplicate
            # or reordered keywords must not bypass the charged loop count.
            match = re.fullmatch(
                r"transport((?: v[1-9][0-9]*){1,16}) from v[1-9][0-9]* to v[1-9][0-9]* steps ([0-9]+)(?: courant ([^ ]+))?",
                " ".join(words))
            if not match:
                raise HTTPException(422, "Use a bounded canonical transport chain")
            count = bounded_count(match[2], MAX_TRANSPORT_STEPS)
            if match[3] is not None:
                try:
                    courant = float(match[3])
                except ValueError:
                    raise HTTPException(422, "Use a finite Courant fraction") from None
                if not math.isfinite(courant) or not 0 <= courant <= 1:
                    raise HTTPException(422, "Use a finite Courant fraction")
            loop_work += count * len(match[1].split())
        if loop_work > MAX_LOOP_WORK:
            raise HTTPException(422, "Aggregate solver loop budget exceeded")
        index = {"wait": 1, "stir": 3, "electrolyse": 3, "electrolyze": 3, "centrifuge": 3}.get(verb)
        if index is None:
            continue
        if len(words) <= index:
            if verb == "stir":
                duration += 10  # The CLI's default stir duration.
                continue
            raise HTTPException(422, "Duration required")
        raw_duration = words[index]
        if verb == "wait":
            tail = words[index:]
            if len(tail) == 2 and re.fullmatch(
                    r"[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?", tail[0]) and re.fullmatch(
                    r"s|sec|secs|seconds|min|mins|minutes|h|hr|hours", tail[1].lower()):
                raw_duration = "".join(tail)
            elif len(tail) != 1:
                raise HTTPException(422, "Use a finite Kero duration")
        match = re.fullmatch(r"([+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?)(s|sec|secs|seconds|min|mins|minutes|h|hr|hours|)", raw_duration.lower())
        if not match:
            raise HTTPException(422, "Use a finite Kero duration")
        unit = match[2]
        value = float(match[1]) * (3600 if unit in ("h", "hr", "hours") else 60 if unit.startswith("min") else 1)
        if not math.isfinite(value) or not 0 <= value <= MAX_COMMAND_SECONDS:
            raise HTTPException(422, "Duration outside API limit")
        duration += value
        if duration > MAX_SCRIPT_SECONDS:
            raise HTTPException(422, "Aggregate simulated duration exceeds API limit")

    if duration > MAX_SCRIPT_SECONDS:
        raise HTTPException(422, "Aggregate simulated duration exceeds API limit")


def bounded_count(raw: str, maximum: int) -> int:
    # Reject before int conversion; no huge integer or signed/float spelling.
    if not re.fullmatch(r"[0-9]{1,10}", raw):
        raise HTTPException(422, "Use a bounded positive integer loop count")
    count = int(raw)
    if not 1 <= count <= maximum:
        raise HTTPException(422, "Loop count outside API limit")
    return count


class Admission:
    """Atomic quotas and worker leases, shared by every process on one host.

    Use one authoritative worker host, on a local persistent filesystem.
    SQLite is intentionally not a distributed/serverless rate limiter.
    """
    def __init__(self, cfg: Settings):
        self.cfg = cfg

    def acquire(self, principal: str) -> str:
        cfg, now = self.cfg, time.time()
        try:
            with closing(sqlite3.connect(cfg.state, timeout=1)) as db, db:
                db.execute("CREATE TABLE IF NOT EXISTS quota (principal TEXT, bucket TEXT, count INTEGER, PRIMARY KEY(principal,bucket))")
                db.execute("CREATE TABLE IF NOT EXISTS leases (id TEXT PRIMARY KEY, expires REAL)")
                db.execute("BEGIN IMMEDIATE")
                db.execute("DELETE FROM leases WHERE expires <= ?", (now,))
                # Only configured principals may reach here; bounded key count.
                db.execute("DELETE FROM quota WHERE (bucket LIKE 'd%' AND bucket < ?) OR (bucket LIKE 'm%' AND bucket < ?)",
                           (f"d{int(now // 86400)-1:012d}", f"m{int(now // 60)-1:012d}"))
                if db.execute("SELECT count(*) FROM leases").fetchone()[0] >= cfg.slots:
                    raise HTTPException(429, "Worker busy", headers={"Retry-After": "10"})
                buckets = [(f"m{int(now // 60):012d}", cfg.minute), (f"d{int(now // 86400):012d}", cfg.daily)]
                for bucket, limit in buckets:
                    row = db.execute("SELECT count FROM quota WHERE principal=? AND bucket=?", (principal, bucket)).fetchone()
                    if row and row[0] >= limit:
                        retry = 60 if bucket[0] == "m" else 86400
                        raise HTTPException(429, "Quota exceeded", headers={"Retry-After": str(retry)})
                for bucket, _ in buckets:
                    db.execute("INSERT INTO quota VALUES (?,?,1) ON CONFLICT(principal,bucket) DO UPDATE SET count=count+1", (principal, bucket))
                lease = uuid.uuid4().hex
                db.execute("INSERT INTO leases VALUES (?,?)", (lease, now + LEASE_SECONDS))
                return lease
        except sqlite3.Error:
            raise HTTPException(503, "Admission store unavailable") from None

    def release(self, lease: str):
        try:
            with closing(sqlite3.connect(self.cfg.state, timeout=1)) as db, db:
                db.execute("DELETE FROM leases WHERE id=?", (lease,))
        except sqlite3.Error:
            pass  # Lease remains occupied until expiry; never grants extra work.


REDIS_ACQUIRE = '''
local now = tonumber(redis.call('TIME')[1])
redis.call('ZREMRANGEBYSCORE', KEYS[1], '-inf', now)
if redis.call('ZCARD', KEYS[1]) >= tonumber(ARGV[2]) then return 2 end
local windows = {60, 86400}
for i=1,2 do
  local bucket = math.floor(now/windows[i])
  local saved = tonumber(redis.call('HGET', KEYS[i+1], 'bucket') or '-1')
  local count = tonumber(redis.call('HGET', KEYS[i+1], 'count') or '0')
  if saved == bucket and count >= tonumber(ARGV[i+2]) then return 3 end
end
for i=1,2 do
  local bucket = math.floor(now/windows[i])
  local saved = tonumber(redis.call('HGET', KEYS[i+1], 'bucket') or '-1')
  if saved ~= bucket then redis.call('HSET', KEYS[i+1], 'bucket', bucket, 'count', 0) end
  redis.call('HINCRBY', KEYS[i+1], 'count', 1)
  redis.call('EXPIRE', KEYS[i+1], windows[i]*2)
end
redis.call('ZADD', KEYS[1], now+tonumber(ARGV[5]), ARGV[1])
redis.call('EXPIRE', KEYS[1], tonumber(ARGV[5])*2)
return 1
'''


class RedisAdmission:
    """Authoritative atomic admission shared by every Vercel function replica."""
    def __init__(self, cfg):
        self.cfg = cfg

    async def command(self, command):
        try:
            async with asyncio.timeout(3):
                async with httpx.AsyncClient(timeout=3, follow_redirects=False, trust_env=False) as client:
                    async with client.stream("POST", self.cfg.redis_url, json=command,
                            headers={"Authorization": "Bearer " + self.cfg.redis_token}) as response:
                        if response.status_code != 200:
                            raise ValueError
                        data = bytearray()
                        async for chunk in response.aiter_bytes():
                            if len(data) + len(chunk) > 8192:
                                raise ValueError
                            data.extend(chunk)
                        result = json.loads(data)
                        if not isinstance(result, dict) or "error" in result or type(result.get("result")) is not int:
                            raise ValueError
                        return result["result"]
        except (TimeoutError, httpx.HTTPError, ValueError, KeyError):
            raise HTTPException(503, "Shared admission unavailable") from None

    async def acquire(self, principal):
        lease = uuid.uuid4().hex
        prefix = "{kerotakis-api-v1}"
        code = await self.command(["EVAL", REDIS_ACQUIRE, "3", prefix + ":leases",
            prefix + ":" + principal + ":minute", prefix + ":" + principal + ":day",
            lease, str(self.cfg.slots), str(self.cfg.minute), str(self.cfg.daily), str(LEASE_SECONDS)])
        if code in (2, 3):
            raise HTTPException(429, "Shared execution or request limit reached", headers={"Retry-After": "60"})
        if code != 1:
            raise HTTPException(503, "Shared admission rejected the request")
        return lease

    async def release(self, lease):
        try:
            await self.command(["ZREM", "{kerotakis-api-v1}:leases", lease])
        except HTTPException:
            pass  # Expiring lease remains occupied; never fall back to local state.


def check_resources(cfg: Settings):
    try:
        memory = dict(line.split(":", 1) for line in Path("/proc/meminfo").read_text().splitlines())
        free_disk = min(shutil.disk_usage(tempfile.gettempdir()).free,
                        shutil.disk_usage(Path(cfg.state).parent).free) // 1024 ** 2
        if (int(memory["MemAvailable"].split()[0]) // 1024 < cfg.min_memory_mib
                or os.getloadavg()[0] > cfg.max_load or free_disk < cfg.min_disk_mib):
            raise HTTPException(503, "Worker resource gate closed", headers={"Retry-After": "30"})
    except (OSError, KeyError, ValueError):
        raise HTTPException(503, "Worker resources unavailable") from None


async def run_engine(cfg: Settings, script: str) -> dict:
    if cfg.mode == "vercel" and cfg.executor == "microvm":
        return await run_microvm_engine(cfg, script)
    started = time.monotonic()
    if hashlib.sha256(Path(cfg.binary).read_bytes()).hexdigest() != cfg.engine["binary_sha256"]:
        raise HTTPException(503, "Engine identity mismatch")
    with tempfile.TemporaryDirectory(prefix="kero-api-") as directory:
        path = Path(directory) / "input.lab"
        path.write_text(script, encoding="utf-8")
        wrapper = Path(__file__).with_name("worker.py")
        if not wrapper.is_file():
            wrapper = Path(__file__).parent / "assets/worker.runtime"
        arguments = [cfg.binary, str(path)]
        environment = {"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8", "RAYON_NUM_THREADS": "1"}
        if cfg.mode == "vercel":
            arguments = ["--vercel", *arguments, cfg.libraries, cfg.loader]
        spawning = asyncio.create_task(asyncio.create_subprocess_exec(sys.executable, "-I", str(wrapper), *arguments,
            stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE,
            stdin=asyncio.subprocess.DEVNULL, cwd=directory, env=environment, start_new_session=True))
        try:
            proc = await asyncio.shield(spawning)
        except asyncio.CancelledError:
            proc = await spawning
            try:
                os.killpg(proc.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            await proc.communicate()
            raise
        except OSError:
            raise HTTPException(503, "Execution worker could not start") from None
        total = 0

        async def read(stream):
            nonlocal total
            chunks = []
            while chunk := await stream.read(8192):
                total += len(chunk)
                if total > OUTPUT_LIMIT:
                    raise OverflowError
                chunks.append(chunk)
            return b"".join(chunks)

        tasks = [asyncio.create_task(read(proc.stdout)), asyncio.create_task(read(proc.stderr)), asyncio.create_task(proc.wait())]
        status = "completed"
        try:
            stdout, stderr, code = await asyncio.wait_for(asyncio.gather(*tasks), WALL_SECONDS)
        except (TimeoutError, OverflowError) as exc:
            status = "timeout" if isinstance(exc, TimeoutError) else "output_limit"
            stdout, stderr, code = b"", b"Execution limit reached", None
        finally:
            # Kill the whole group even when its leader exited with open pipes.
            try:
                os.killpg(proc.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            for task in tasks:
                if not task.done():
                    task.cancel()
            await asyncio.gather(*tasks, return_exceptions=True)
            # A paused asyncio pipe transport can keep wait() pending after
            # exit. Drain its bounded residual buffers after killing the group.
            await proc.communicate()
        return build_result(cfg, script, started, status, stdout, stderr, code, path)


def build_result(cfg, script, started, status, stdout, stderr, code, path):
    # A process killed during execution may leave a partial JSON line. Classify
    # the signal before decoding observations, and never call it a CLI refusal
    # or invent which resource (if any) caused SIGKILL.
    if status == "completed" and code is not None and code < 0:
        logging.getLogger("kerotakis.api").warning("Engine terminated: signal=%s", -code)
        raise HTTPException(503, {"code": "engine_terminated", "signal": -code,
                                  "cause": "not_determined"})
    observations = []
    def finite_float(value):
        number = float(value)
        if not math.isfinite(number):
            raise ValueError
        return number

    def invalid_constant(_value):
        raise ValueError

    try:
        for line in stdout.splitlines():
            row = json.loads(line, parse_float=finite_float, parse_constant=invalid_constant)
            if not isinstance(row, dict):
                raise ValueError
            observations.append(row)
    except (ValueError, UnicodeError):
        raise HTTPException(502, "Engine returned invalid observations") from None
    if status == "completed" and code:
        if not stderr.startswith(b"kero:"):
            reason = stderr.strip().decode("ascii", "ignore")
            permitted = {"sandbox: " + value for value in (
                "architecture", "landlock", "privileges", "capabilities",
                "filesystem_create", "filesystem_rule", "filesystem_install", "seccomp", "exec")}
            logging.getLogger("kerotakis.api").warning(
                "Engine startup failed: exit=%s restriction=%s", code,
                reason if reason in permitted or re.fullmatch(
                    r"sandbox: landlock_abi=-?\d{1,5}_errno=\d{1,5}", reason) else "unclassified")
            raise HTTPException(503, "Engine or sandbox failed to execute")
        status = "refused"
    result = {"execution_status": status, "exit_status": code,
            "observations": observations, "diagnostics": stderr.decode("utf-8", "replace").replace("/job/input.lab", "<script>").replace(str(path), "<script>"),
            "elapsed_seconds": time.monotonic() - started,
            "script_sha256": hashlib.sha256(script.encode()).hexdigest(), "engine": cfg.engine,
            "physical_verdict": "not_assigned"}
    if len(json.dumps(result, ensure_ascii=False, allow_nan=False, separators=(",", ":")).encode()) > OUTPUT_LIMIT * 2:
        raise HTTPException(502, "Serialized engine response exceeds limit")
    return result


async def run_microvm_engine(cfg: Settings, script: str):
    # The function authenticates/admit requests; the disposable VM receives only
    # this script, immutable engine snapshot, fixed limits and no API secrets.
    from vercel import sandbox
    from vercel.api import session
    started = time.monotonic()
    try:
        async with asyncio.timeout(35):
            async with session():
                async with sandbox.create_sandbox(
                        source=sandbox.SnapshotSource(snapshot_id=cfg.snapshot),
                        resources=sandbox.SandboxResources(vcpus=1, memory=2048),
                        execution_time_limit=30, network_policy=sandbox.NetworkPolicy.deny_all(),
                        env={}, ports=[], persistent=False, region="iad1") as box:
                    await box.fs.write_text("/vercel/sandbox/engine/input.lab", script, mode=0o644)
                    async with asyncio.timeout(23):
                        completed = await box.run_process("python3", ["-I", "/vercel/sandbox/engine-job.py"],
                            sudo=True, capture_output=True, kill_after=21)
                    require(completed.returncode == 0 and len(completed.stdout.encode()) <= OUTPUT_LIMIT * 2)
                    raw = json.loads(completed.stdout)
                    require(raw["binary_sha256"] == cfg.engine["binary_sha256"])
                    require(raw["status"] in ("completed", "timeout", "output_limit"))
                    require(type(raw["code"]) is int or raw["code"] is None)
                    stdout = base64.b64decode(raw["stdout"], validate=True)
                    stderr = base64.b64decode(raw["stderr"], validate=True)
                    require(len(stdout) + len(stderr) <= OUTPUT_LIMIT)
                    return build_result(cfg, script, started, raw["status"], stdout, stderr, raw["code"], "/input.lab")
    except HTTPException:
        raise
    except Exception as exc:
        # Error category only; SDK errors may contain credentials or job data.
        logging.getLogger("kerotakis.api").warning("Isolated Vercel execution unavailable: %s", type(exc).__name__)
        raise HTTPException(503, "Isolated Vercel execution unavailable") from None


async def forward(cfg: Settings, token: str, script: str):
    try:
        async with asyncio.timeout(50):
            async with httpx.AsyncClient(timeout=50, follow_redirects=False, trust_env=False) as client:
                async with client.stream("POST", cfg.upstream + "/run", json={"script": script},
                        headers={"Authorization": "Bearer " + token}) as response:
                    data = bytearray()
                    async for chunk in response.aiter_bytes():
                        if len(data) + len(chunk) > OUTPUT_LIMIT * 2:
                            raise ValueError
                        data.extend(chunk)
                    if response.status_code in (401, 403, 422, 429, 503):
                        raise HTTPException(response.status_code, "Execution worker declined the request",
                            headers={"Retry-After": "30"} if response.status_code in (429, 503) else None)
                    if response.status_code != 200:
                        raise ValueError
                    return RunOutput.model_validate_json(data).model_dump()
    except (TimeoutError, httpx.HTTPError, ValueError, ValidationError):
        raise HTTPException(502, "Execution worker unavailable or returned an invalid response") from None


def create_app(cfg: Settings | None = None):
    cfg = cfg or Settings.from_env()
    app = FastAPI(title="Kerotakis REST API", version="1.0.0",
                  swagger_ui_parameters={"persistAuthorization": False})
    app.add_middleware(CORSMiddleware, allow_origins=cfg.origins,
                       allow_methods=["POST", "GET"], allow_headers=["Authorization", "Content-Type"])
    admission = Admission(cfg)
    shared_admission = RedisAdmission(cfg)

    @app.middleware("http")
    async def headers(request, call_next):
        response = await call_next(request)
        response.headers["Cache-Control"] = "no-store"
        response.headers["X-Content-Type-Options"] = "nosniff"
        return response

    async def authenticate(credentials: HTTPAuthorizationCredentials | None = Depends(bearer)):
        if cfg.mode == "disabled":
            raise HTTPException(503, "Execution service is not configured")
        if credentials is None or len(credentials.credentials) > 512:
            raise HTTPException(401, "Bearer credential required", headers={"WWW-Authenticate": "Bearer"})
        digest = hashlib.sha256(credentials.credentials.encode()).hexdigest()
        principal = None
        for name, expected in cfg.keys.items():
            if hmac.compare_digest(digest, expected):
                principal = name
        if principal is None:
            raise HTTPException(401, "Invalid bearer credential", headers={"WWW-Authenticate": "Bearer"})
        return principal, credentials.credentials

    @app.get("/health")
    async def health():
        return {"service": "kerotakis", "api_version": 1, "mode": cfg.mode,
                "configured": cfg.mode != "disabled", "revision": cfg.revision,
                "execution_backend": cfg.executor if cfg.mode == "vercel" else cfg.mode,
                "engine": cfg.engine or None}

    @app.get("/capabilities")
    async def capabilities():
        # Public, static policy only: no principal, key, host path or snapshot.
        return {"api_version": 1, "revision": cfg.revision,
                "language": "en", "commands": sorted(VERBS),
                "unavailable_session_commands": sorted(UNAVAILABLE_SESSION_WORDS),
                "limits": {"body_bytes": BODY_LIMIT, "script_bytes": SCRIPT_LIMIT,
                    "commands": 128, "tokens_per_command": 32, "vessel_creations": 16,
                    "titration_steps": MAX_TITRATION_STEPS, "stages": MAX_STAGES,
                    "transport_steps": MAX_TRANSPORT_STEPS, "transport_cells": 16,
                    "aggregate_loop_work": MAX_LOOP_WORK,
                    "simulated_seconds_per_command": MAX_COMMAND_SECONDS,
                    "aggregate_simulated_seconds": MAX_SCRIPT_SECONDS,
                    "child_cpu_seconds": CPU_SECONDS, "child_wall_seconds": WALL_SECONDS},
                "physical_verdict": "not_assigned"}

    @app.post("/run", response_model=RunOutput, openapi_extra={"requestBody": {
        "required": True, "content": {"application/json": {"schema": RunInput.model_json_schema()}}}})
    async def run(request: Request, identity=Depends(authenticate)):
        if request.headers.get("content-type", "").split(";", 1)[0].strip().lower() != "application/json":
            raise HTTPException(415, "Use application/json")
        if request.headers.get("content-encoding", "identity") != "identity":
            raise HTTPException(415, "Compressed request bodies are not supported")
        body = bytearray()
        try:
            async with asyncio.timeout(5):
                async for chunk in request.stream():
                    if len(body) + len(chunk) > BODY_LIMIT:
                        raise HTTPException(413, "Request body too large")
                    body.extend(chunk)
        except TimeoutError:
            raise HTTPException(408, "Request body timeout") from None
        try:
            payload = RunInput.model_validate_json(body)
        except ValidationError:
            raise HTTPException(422, "Expected only a bounded non-empty script string") from None
        validate_script(payload.script)
        principal, token = identity
        if cfg.mode == "gateway":
            return await forward(cfg, token, payload.script)
        if cfg.mode == "vercel":
            lease = await shared_admission.acquire(principal)
        else:
            check_resources(cfg)
            lease = admission.acquire(principal)
        try:
            return await run_engine(cfg, payload.script)
        finally:
            if cfg.mode == "vercel":
                await shared_admission.release(lease)
            else:
                admission.release(lease)

    return app


app = create_app()
