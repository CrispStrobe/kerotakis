#!/usr/bin/env python3
"""BRD-010: pin a PubChem PUG REST/PUG View snapshot for the quarantine fixture.

Network access is build-time only (BREADTH's inherited rules). This script is
the only thing in the tree that talks to PubChem; everything downstream reads
the pinned bytes it writes. Running it is a deliberate, reviewable refresh:

    tools/fetch-pubchem-snapshot.py
    cargo run -p kerotakis-data --bin quarantine-review -- \
        diff old-candidates.json new-candidates.json

What it writes, next to the seed plan:

    raw/snapshot.json   every response body this run received
    manifest.json       the BRD-003 SnapshotManifest pinning that file's SHA-256

Fidelity rules, so a reviewer knows exactly what "pinned" means here:

* `name_to_cid`, `property_table` and `pug_view` bodies are stored **verbatim**,
  in the order PubChem returned them.
* `synonyms` bodies are stored with each record's `Synonym` array truncated to
  `synonym_cap` entries, because a single record (ethanol) carries over nine
  thousand depositor-supplied synonyms and the fixture has to stay reviewable.
  The truncation is declared in the snapshot and the SHA-256 of the *full*
  upstream body is recorded beside it, so the elision is auditable rather than
  silent. None of those synonyms is promotable anyway — PubChem's own data
  model calls that list "Depositor-Supplied Synonyms" — so the cap costs the
  importer nothing but review signal.

Service limits (https://pubchem.ncbi.nlm.nih.gov/docs/pug-rest): no more than
5 requests/second and 400 requests/minute. This script sleeps between requests
and batches CIDs, so a full run is well inside both.
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

REST = "https://pubchem.ncbi.nlm.nih.gov/rest/pug"
VIEW = "https://pubchem.ncbi.nlm.nih.gov/rest/pug_view/data/compound"
USER_AGENT = "kerotakis-brd010-snapshot/1 (+https://github.com/CrispStrobe/kerotakis)"

# The property table BRD-010 requests. It deliberately includes fields the
# promotion policy does NOT allowlist (XLogP, TPSA, Complexity, the H-bond and
# bond counts): fetching them and watching them be refused by name is how the
# field allowlist is shown to work, rather than asserted.
PROPERTIES = [
    "MolecularFormula",
    "MolecularWeight",
    "MonoisotopicMass",
    "ExactMass",
    "Charge",
    "SMILES",
    "ConnectivitySMILES",
    "InChI",
    "InChIKey",
    "IUPACName",
    "Title",
    "XLogP",
    "TPSA",
    "Complexity",
    "HBondDonorCount",
    "HBondAcceptorCount",
    "RotatableBondCount",
    "HeavyAtomCount",
]

# PubChem answers a property table for at most a few hundred CIDs; 25 keeps the
# URL short and the failure blast radius small.
CID_BATCH = 25
REQUEST_PAUSE_SECONDS = 0.25


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def get_json(url: str, *, retries: int = 4) -> tuple[dict, int]:
    """Fetch one JSON body, retrying on PubChem's throttle responses."""
    last: Exception | None = None
    for attempt in range(retries):
        request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
        try:
            with urllib.request.urlopen(request, timeout=60) as response:
                body = json.loads(response.read().decode("utf-8"))
                return body, response.status
        except urllib.error.HTTPError as error:
            payload = error.read().decode("utf-8", "replace")
            if error.code in (404, 400):
                # A "no CID found" fault is data, not a failure: it is exactly
                # what a curator needs to see about a name that does not
                # resolve, so it is pinned like any other answer.
                try:
                    return json.loads(payload), error.code
                except json.JSONDecodeError:
                    return {"_non_json_body": payload}, error.code
            last = error
        except (urllib.error.URLError, TimeoutError, json.JSONDecodeError) as error:
            last = error
        time.sleep(2.0 * (attempt + 1))
    raise SystemExit(f"fetch-pubchem-snapshot: giving up on {url}: {last}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--fixture",
        type=Path,
        default=Path(__file__).resolve().parents[1]
        / "crates/kerotakis-data/tests/fixtures/quarantine/pubchem-v1",
        help="fixture directory holding seed.json (default: the BRD-010 fixture)",
    )
    parser.add_argument(
        "--retrieved",
        default=dt.date.today().isoformat(),
        help="retrieval date recorded in the manifest (default: today)",
    )
    arguments = parser.parse_args()

    fixture: Path = arguments.fixture
    seed = json.loads((fixture / "seed.json").read_text(encoding="utf-8"))
    adapter_id = seed["adapter_id"]
    synonym_cap = int(seed["synonym_cap"])
    heading = seed["annotation_heading"]

    responses: list[dict] = []
    resolutions: list[dict] = []
    seen_cids: list[int] = []

    # 1. Resolve every seed name. The name→CID answer is pinned verbatim,
    #    including faults, because "PubChem answered this name with that
    #    record" is the claim the fixture has to be able to re-examine.
    for entry in seed["seeds"]:
        name = entry["name"]
        url = f"{REST}/compound/name/{urllib.parse.quote(name)}/cids/JSON"
        body, status = get_json(url)
        cids = body.get("IdentifierList", {}).get("CID", [])
        responses.append(
            {"kind": "name_to_cid", "query": name, "url": url, "status": status, "body": body}
        )
        resolutions.append({"name": name, "class": entry["class"], "cids": cids})
        for cid in cids:
            if cid not in seen_cids:
                seen_cids.append(cid)
        print(f"  name  {name:38s} -> {cids}", file=sys.stderr)
        time.sleep(REQUEST_PAUSE_SECONDS)

    seen_cids.sort()
    print(f"  {len(seed['seeds'])} seeds -> {len(seen_cids)} distinct CIDs", file=sys.stderr)

    # 2. Property tables, batched.
    properties = ",".join(PROPERTIES)
    for start in range(0, len(seen_cids), CID_BATCH):
        batch = seen_cids[start : start + CID_BATCH]
        joined = ",".join(str(cid) for cid in batch)
        url = f"{REST}/compound/cid/{joined}/property/{properties}/JSON"
        body, status = get_json(url)
        responses.append(
            {"kind": "property_table", "cids": batch, "url": url, "status": status, "body": body}
        )
        print(f"  props {len(batch)} CIDs", file=sys.stderr)
        time.sleep(REQUEST_PAUSE_SECONDS)

    # 3. Synonyms, batched and capped (see the module docstring).
    for start in range(0, len(seen_cids), CID_BATCH):
        batch = seen_cids[start : start + CID_BATCH]
        joined = ",".join(str(cid) for cid in batch)
        url = f"{REST}/compound/cid/{joined}/synonyms/JSON"
        body, status = get_json(url)
        full_bytes = json.dumps(body, ensure_ascii=False, sort_keys=True).encode("utf-8")
        totals = {}
        for information in body.get("InformationList", {}).get("Information", []):
            synonyms = information.get("Synonym", [])
            totals[str(information.get("CID"))] = len(synonyms)
            if len(synonyms) > synonym_cap:
                information["Synonym"] = synonyms[:synonym_cap]
        responses.append(
            {
                "kind": "synonyms",
                "cids": batch,
                "url": url,
                "status": status,
                "synonym_cap": synonym_cap,
                "synonym_total_by_cid": totals,
                "full_body_sha256": sha256_hex(full_bytes),
                "body": body,
            }
        )
        print(f"  syns  {len(batch)} CIDs", file=sys.stderr)
        time.sleep(REQUEST_PAUSE_SECONDS)

    # 4. PUG View annotations for the declared subset. These are the
    #    depositor-annotated physical properties: each one arrives with the
    #    upstream annotation source and that source's own licence note, which
    #    is what the adapter's annotation-source gate reads.
    by_name = {resolution["name"]: resolution["cids"] for resolution in resolutions}
    annotation_cids: list[int] = []
    for name in seed["annotation_cids_by_seed"]:
        for cid in by_name.get(name, []):
            if cid not in annotation_cids:
                annotation_cids.append(cid)
    for cid in sorted(annotation_cids):
        url = f"{VIEW}/{cid}/JSON?heading={urllib.parse.quote(heading)}"
        body, status = get_json(url)
        responses.append(
            {
                "kind": "pug_view",
                "cid": cid,
                "heading": heading,
                "url": url,
                "status": status,
                "body": body,
            }
        )
        print(f"  view  {cid} {heading}", file=sys.stderr)
        time.sleep(REQUEST_PAUSE_SECONDS)

    snapshot = {
        "schema": 1,
        "adapter_id": adapter_id,
        "service": REST,
        "retrieved": arguments.retrieved,
        "synonym_cap": synonym_cap,
        "annotation_heading": heading,
        "fidelity": (
            "name_to_cid, property_table and pug_view bodies are verbatim; "
            "synonyms bodies are truncated to synonym_cap entries per CID with "
            "the full upstream body's SHA-256 recorded alongside"
        ),
        "resolutions": resolutions,
        "responses": responses,
    }

    raw_path = fixture / "raw" / "snapshot.json"
    raw_path.parent.mkdir(parents=True, exist_ok=True)
    raw_bytes = (
        json.dumps(snapshot, ensure_ascii=False, indent=2, sort_keys=False) + "\n"
    ).encode("utf-8")
    raw_path.write_bytes(raw_bytes)

    manifest = {
        "schema": 1,
        "adapter_id": adapter_id,
        "source_id": "pubchem-pug-rest",
        "source_revision": f"pug-rest-retrieved-{arguments.retrieved}",
        "retrieved": arguments.retrieved,
        "raw_artifact": "raw/snapshot.json",
        "record_count": len(seen_cids),
        "sha256": sha256_hex(raw_bytes),
    }
    (fixture / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )

    print(
        f"wrote {raw_path} ({len(raw_bytes)} bytes, {len(seen_cids)} records, "
        f"{len(responses)} responses)",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
