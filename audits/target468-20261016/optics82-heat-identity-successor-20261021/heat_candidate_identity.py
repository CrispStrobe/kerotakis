"""Exact source admission helper extracted from reviewed operational packer."""
import base64,re
CANDIDATE="62f647275fd800d4992f5516651dfc9e61f47a05"
def require(value,message):
    if not value:raise ValueError(message)
def check_heat_candidate(files, candidate):
    """Reject stale operational JS revision guards before harness creation/upload."""
    require(isinstance(candidate, str) and re.fullmatch('[a-f0-9]{40}', candidate),
            'Malformed reviewed candidate authority')
    patterns = {
        'heat-controls.mjs': r"const REVIEWED_CANDIDATE_SOURCE='([a-f0-9]{40})';",
        'run-heat.mjs': r"assert\.equal\(binding\.reviewed_candidate_source,'([a-f0-9]{40})'\)",
    }
    for name, pattern in patterns.items():
        raw = base64.b64decode(files[name]['base64'], validate=True)
        require(re.findall(pattern, raw.decode('utf-8')) == [candidate],
                'Heat launcher/module reviewed candidate differs: ' + name)
