#!/usr/bin/env python3
"""BRD-013: pin a USDA FoodData Central Foundation Foods snapshot.

This is a *build-time* tool. Nothing in the shipped crates or the runtime
touches the network; `crates/kerotakis-data/src/usda.rs` only ever reads the
bytes this script writes, and the BRD-003 `SnapshotManifest` it emits is what
makes the import reproducible.

Usage
-----

    tools/fetch-usda-snapshot.py \
        --out crates/kerotakis-data/tests/fixtures/quarantine/usda-fdc-v1

    # or, with the release archive already on disk and no network at all:
    tools/fetch-usda-snapshot.py --out DIR --archive FoodData_Central_....zip

Why the release archive and not the API
---------------------------------------

FoodData Central publishes each Foundation Foods release as one versioned zip
alongside the REST API. The archive is the better pin, for three reasons found
while building this:

1. It has a release identity. `FoodData_Central_foundation_food_json_2025-04-24`
   names exactly what was imported; the API answers "whatever is live now".
2. It needs no API key, so nothing secret is involved in a build at all — the
   honest way to satisfy "API keys never enter builds or runtime" is not to
   need one. It is also not rate limited.
3. The API is *missing records the release contains*. `/foods/list` and
   `/foods/search` both advertise `Milk, whole, 3.25% milkfat` (fdcId 746782)
   and `Eggs, Grade A, Large, egg whole` (748967), but `/food/746782` returns
   404 and the bulk `/foods` endpoint silently omits both — it returns 13
   records for a 15-id request without saying which two it dropped. Both foods
   are present and complete in the release archive.

The archive's own SHA-256 is pinned in `UPSTREAM_SHA256` and verified before
anything is read, so a substituted download is refused rather than imported.

Projection
----------

The snapshot stores a *declared projection*, version `PROJECTION_VERSION`,
rather than the raw release bytes: the release covers 340 foods and this
fixture pins 15. The projection keeps every value, unit, derivation,
per-nutrient uncertainty (dataPoints/min/max/median), portion, sample-input
food, conversion factor and release field the adapter or a reviewer reads. It
drops only upstream surrogate row ids, which carry no information and churn
between releases. It is declared here and in the manifest so a reviewer can
see exactly what was kept.

Determinism
-----------

Records are sorted by `fdcId`, keys are sorted, nutrients are sorted by
nutrient id, and the file is UTF-8 with 2-space indentation and a trailing
newline. Re-running against the same pinned release reproduces the file byte
for byte, so the manifest checksum is a real pin.
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import sys
import urllib.error
import urllib.request
import zipfile
from datetime import date
from typing import Any

ADAPTER_ID = "usda-fdc"
SOURCE_ID = "usda-fdc-foundation"
SCHEMA = 1
PROJECTION_VERSION = "usda-foundation-projection-v1"

UPSTREAM_RELEASE = "FoodData_Central_foundation_food_json_2025-04-24"
UPSTREAM_URL = f"https://fdc.nal.usda.gov/fdc-datasets/{UPSTREAM_RELEASE}.zip"
UPSTREAM_SHA256 = "8d1f520a9a63fd34bd66c541f7ae6ac7bcc7edefae5ee3ce90202d825d7f47a2"
UPSTREAM_MEMBER = f"{UPSTREAM_RELEASE}.json"

# The pinned fixture. Foundation Foods only: analytical, generic and stable,
# where Branded records are volatile reformulations with label-rounded numbers.
# Each entry is a food the shipped or planned BRD-014 material recipes name.
#
# One deliberate absence, so a reviewer does not have to rediscover it: honey
# has no Foundation Foods record at all, in this release or any other. Its
# composition would have to come from SR Legacy, which is a different data
# quality claim, so it is simply not here.
PINNED_FOODS: list[int] = [
    746775,   # Salt, table, iodized
    746782,   # Milk, whole, 3.25% milkfat, with added vitamin D
    746784,   # Sugars, granulated
    748366,   # Oil, soybean
    748967,   # Eggs, Grade A, Large, egg whole
    789828,   # Butter, stick, unsalted
    789951,   # Flour, wheat, all-purpose, enriched, unbleached
    2003590,  # Apple juice, with added vitamin C, from concentrate
    2003597,  # Orange juice, no pulp, not from concentrate, refrigerated
    2258586,  # Carrots, mature, raw
    2259793,  # Yogurt, plain, whole milk
    2346396,  # Oats, whole grain, rolled, old fashioned
    2346401,  # Potatoes, russet, without skin, raw
    2512381,  # Rice, white, long grain, unenriched, raw
    2644281,  # Beans, cannellini, dry
]

FOOD_KEYS = (
    "fdcId",
    "description",
    "dataType",
    "foodClass",
    "publicationDate",
    "ndbNumber",
    "footnote",
    "isHistoricalReference",
)
NUTRIENT_KEYS = ("id", "number", "name", "unitName", "rank")
VALUE_KEYS = ("amount", "dataPoints", "min", "max", "median", "minYearAcquired")


def load_archive(path: str | None) -> bytes:
    if path:
        with open(path, "rb") as handle:
            return handle.read()
    print(f"downloading {UPSTREAM_RELEASE}.zip ...", file=sys.stderr)
    request = urllib.request.Request(UPSTREAM_URL, headers={"Accept": "application/zip"})
    try:
        with urllib.request.urlopen(request, timeout=300) as response:
            return response.read()
    except urllib.error.HTTPError as error:
        raise SystemExit(
            f"FoodData Central refused {UPSTREAM_URL}: HTTP {error.code} {error.reason}"
        ) from error


def project_nutrient(raw: dict[str, Any]) -> dict[str, Any]:
    nutrient = raw.get("nutrient") or {}
    out: dict[str, Any] = {
        "nutrient": {key: nutrient[key] for key in NUTRIENT_KEYS if key in nutrient}
    }
    for key in VALUE_KEYS:
        if raw.get(key) is not None:
            out[key] = raw[key]

    derivation = raw.get("foodNutrientDerivation")
    if derivation:
        entry = {
            key: derivation[key] for key in ("code", "description") if key in derivation
        }
        source = derivation.get("foodNutrientSource")
        if source:
            entry["source"] = {
                key: source[key] for key in ("code", "description") if key in source
            }
        out["derivation"] = entry
    return out


def project_food(raw: dict[str, Any]) -> dict[str, Any]:
    out: dict[str, Any] = {
        key: raw[key] for key in FOOD_KEYS if raw.get(key) not in (None, "")
    }

    category = raw.get("foodCategory")
    if category:
        entry = {key: category[key] for key in ("code", "description") if key in category}
        if entry:
            out["foodCategory"] = entry

    factors = [
        {key: value for key, value in factor.items() if key != "id"}
        for factor in raw.get("nutrientConversionFactors") or []
    ]
    if factors:
        out["nutrientConversionFactors"] = sorted(
            factors, key=lambda factor: json.dumps(factor, sort_keys=True)
        )

    portions = []
    for portion in raw.get("foodPortions") or []:
        entry = {
            "amount": portion.get("amount", portion.get("value")),
            "gramWeight": portion.get("gramWeight"),
            "measureUnit": (portion.get("measureUnit") or {}).get("name"),
            "modifier": portion.get("modifier"),
        }
        entry = {key: value for key, value in entry.items() if value not in (None, "")}
        if entry:
            portions.append(entry)
    if portions:
        out["foodPortions"] = sorted(
            portions, key=lambda portion: json.dumps(portion, sort_keys=True)
        )

    inputs = []
    for item in raw.get("inputFoods") or []:
        inner = item.get("inputFood") or {}
        entry = {
            "foodDescription": item.get("foodDescription"),
            "inputFdcId": inner.get("fdcId"),
            "inputDataType": inner.get("dataType"),
        }
        entry = {key: value for key, value in entry.items() if value not in (None, "")}
        if entry:
            inputs.append(entry)
    if inputs:
        out["inputFoods"] = sorted(
            inputs, key=lambda item: json.dumps(item, sort_keys=True)
        )

    out["foodNutrients"] = sorted(
        (project_nutrient(nutrient) for nutrient in raw.get("foodNutrients") or []),
        key=lambda nutrient: nutrient["nutrient"].get("id", 0),
    )
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", required=True, help="quarantine fixture directory")
    parser.add_argument(
        "--archive",
        help="an already-downloaded release zip, so a rebuild needs no network",
    )
    parser.add_argument(
        "--retrieved",
        default=date.today().isoformat(),
        help="retrieval date recorded in the manifest (YYYY-MM-DD)",
    )
    args = parser.parse_args()

    archive = load_archive(args.archive)
    digest = hashlib.sha256(archive).hexdigest()
    if digest != UPSTREAM_SHA256:
        raise SystemExit(
            f"{UPSTREAM_RELEASE}.zip hashes to {digest}, not the pinned "
            f"{UPSTREAM_SHA256}; the release moved or the download is not the "
            "one this fixture was reviewed against"
        )

    with zipfile.ZipFile(io.BytesIO(archive)) as bundle:
        release = json.loads(bundle.read(UPSTREAM_MEMBER))
    foods = release["FoundationFoods"]
    by_id = {int(food["fdcId"]): food for food in foods}

    wanted = sorted(set(PINNED_FOODS))
    missing = [fdc for fdc in wanted if fdc not in by_id]
    if missing:
        raise SystemExit(
            f"{UPSTREAM_RELEASE} does not contain "
            + ", ".join(str(fdc) for fdc in missing)
            + "; the pinned list and the release disagree"
        )
    wrong_type = [fdc for fdc in wanted if by_id[fdc].get("dataType") != "Foundation"]
    if wrong_type:
        raise SystemExit(
            "not Foundation Foods: " + ", ".join(str(fdc) for fdc in wrong_type)
        )

    projected = [project_food(by_id[fdc]) for fdc in wanted]
    snapshot = (
        json.dumps(projected, sort_keys=True, indent=2, ensure_ascii=False) + "\n"
    ).encode("utf-8")

    manifest = {
        "schema": SCHEMA,
        "adapter_id": ADAPTER_ID,
        "source_id": SOURCE_ID,
        "source_revision": (
            f"{UPSTREAM_RELEASE}.zip sha256:{UPSTREAM_SHA256};{PROJECTION_VERSION}"
        ),
        "retrieved": args.retrieved,
        "raw_artifact": "raw/snapshot.json",
        "record_count": len(projected),
        "sha256": hashlib.sha256(snapshot).hexdigest(),
    }

    os.makedirs(os.path.join(args.out, "raw"), exist_ok=True)
    with open(os.path.join(args.out, "raw", "snapshot.json"), "wb") as handle:
        handle.write(snapshot)
    with open(os.path.join(args.out, "manifest.json"), "w", encoding="utf-8") as handle:
        handle.write(json.dumps(manifest, sort_keys=True, indent=2) + "\n")

    print(
        f"wrote {len(projected)} of {len(foods)} Foundation Foods records "
        f"({len(snapshot)} bytes, sha256 {manifest['sha256'][:16]}...)",
        file=sys.stderr,
    )
    for fdc in wanted:
        print(f"  {fdc:>8}  {by_id[fdc]['description']}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
