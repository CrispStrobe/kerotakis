#!/usr/bin/env python3
"""A minimal App Store Connect API client.

Auth is a short-lived ES256 JWT signed with the account's `.p8` key. The
key itself is never in this repository: it is looked up in the places
Apple's own tooling and this account's convention put it, or supplied
base64-encoded in `ASC_API_KEY_P8_BASE64` for CI.

Used as a library by the other scripts in this directory, or directly:

    python3 tools/asc/client.py GET '/v1/apps?limit=5'
    python3 tools/asc/client.py POST /v1/betaGroups '{"data": ...}'
"""

from __future__ import annotations

import base64
import json
import os
import pathlib
import sys
import time
import urllib.error
import urllib.request

BASE = "https://api.appstoreconnect.apple.com"
KEY_ID = os.environ.get("ASC_KEY_ID", "9RMU3C7422")
ISSUER_ID = os.environ.get("ASC_ISSUER_ID", "5f618ba3-98ef-42ad-835c-fbbef6c76cf5")
TEAM_ID = os.environ.get("ASC_TEAM_ID", "N9XSJ4M3GT")

ROOT = pathlib.Path(__file__).resolve().parent.parent.parent


def _private_key_pem() -> bytes:
    env = os.environ.get("ASC_API_KEY_P8_BASE64")
    if env:
        return base64.b64decode(env)
    candidates = [
        ROOT / ".appstoreconnect" / f"AuthKey_{KEY_ID}.p8",
        pathlib.Path.home() / ".appstoreconnect" / "private_keys" / f"AuthKey_{KEY_ID}.p8",
    ]
    for c in candidates:
        if c.exists():
            return c.read_bytes()
    raise SystemExit(
        "no App Store Connect key found. Put AuthKey_%s.p8 in .appstoreconnect/ "
        "or ~/.appstoreconnect/private_keys/, or set ASC_API_KEY_P8_BASE64.\n"
        "Tried:\n  %s" % (KEY_ID, "\n  ".join(str(c) for c in candidates))
    )


def token() -> str:
    """A fresh 20-minute bearer token. Cheap enough to mint per call."""
    from cryptography.hazmat.primitives import hashes, serialization
    from cryptography.hazmat.primitives.asymmetric import ec, utils

    def b64url(data: bytes) -> str:
        return base64.urlsafe_b64encode(data).rstrip(b"=").decode()

    key = serialization.load_pem_private_key(_private_key_pem(), password=None)
    now = int(time.time())
    header = {"alg": "ES256", "kid": KEY_ID, "typ": "JWT"}
    payload = {"iss": ISSUER_ID, "iat": now, "exp": now + 1190, "aud": "appstoreconnect-v1"}
    signing_input = (
        f"{b64url(json.dumps(header, separators=(',', ':')).encode())}."
        f"{b64url(json.dumps(payload, separators=(',', ':')).encode())}"
    )
    der = key.sign(signing_input.encode(), ec.ECDSA(hashes.SHA256()))
    r, s = utils.decode_dss_signature(der)
    return f"{signing_input}.{b64url(r.to_bytes(32, 'big') + s.to_bytes(32, 'big'))}"


def call(method: str, path: str, body: dict | None = None) -> tuple[int, dict]:
    """One request. Returns (status, parsed-body); never raises on 4xx/5xx,
    because the interesting cases (409 listing valid values, 403 telling you
    a resource is browser-only) are answers, not failures."""
    url = path if path.startswith("http") else BASE + path
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(url, data=data, method=method)
    req.add_header("Authorization", "Bearer " + token())
    if data:
        req.add_header("Content-Type", "application/json")
    try:
        with urllib.request.urlopen(req) as r:
            raw = r.read()
            return r.status, (json.loads(raw) if raw else {})
    except urllib.error.HTTPError as e:
        raw = e.read()
        try:
            return e.code, json.loads(raw)
        except ValueError:
            return e.code, {"raw": raw.decode(errors="replace")}


def expect(method: str, path: str, body: dict | None = None, ok=(200, 201, 204)) -> dict:
    """`call`, but a status outside `ok` is fatal and prints Apple's own
    reasons — which are far more specific than the HTTP code."""
    status, doc = call(method, path, body)
    if status not in ok:
        print(f"{method} {path} -> HTTP {status}", file=sys.stderr)
        for e in doc.get("errors", [{"detail": json.dumps(doc)}]):
            print(f"  {e.get('code', '')}: {e.get('detail', '')}", file=sys.stderr)
        raise SystemExit(1)
    return doc


def paged(path: str) -> list[dict]:
    """Every page of a collection, flattened."""
    out: list[dict] = []
    while path:
        doc = expect("GET", path)
        out.extend(doc.get("data", []))
        path = doc.get("links", {}).get("next", "")
    return out


def app_id(bundle_id: str) -> str | None:
    """The numeric app id `altool --apple-id` wants, from the bundle id.

    Resolved rather than configured on purpose: uploading with the wrong
    numeric id silently lands the build in a *different* app.
    """
    for app in paged("/v1/apps?limit=200"):
        if app["attributes"]["bundleId"] == bundle_id:
            return app["id"]
    return None


if __name__ == "__main__":
    method, path = sys.argv[1], sys.argv[2]
    body = json.loads(sys.argv[3]) if len(sys.argv) > 3 else None
    status, doc = call(method, path, body)
    print(f"HTTP {status}", file=sys.stderr)
    print(json.dumps(doc, indent=1))
