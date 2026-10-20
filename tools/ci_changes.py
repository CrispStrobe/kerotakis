#!/usr/bin/env python3
"""Conservative complete Git-range classification; unknown input requires full CI."""
import argparse
import json
import pathlib
import re
import subprocess

# Reviewed operating summaries, not embedded runtime/fixture/contract documents.
DOCS_ONLY = frozenset({
    'docs/STATUS.md',
    'docs/improvement-agenda-20261014.md',
    'docs/next-agent-handoff.md',
    'docs/peroxide-medium-and-i18n-20261019.md',
    'docs/post-hydrolysis-gap-lanes-20261019.md',
    'docs/scientific-coverage-expansion-20261020.md',
    'docs/unifac-input-admission-20261019.md',
    'docs/wrap-up-and-open-lanes-20261019.md',
})
SHA = re.compile(r'[0-9a-f]{40}\Z')


def event_range(event_name, event):
    if not isinstance(event, dict):
        raise ValueError('event must be an object')
    if event_name == 'pull_request':
        return event['pull_request']['base']['sha'], event['pull_request']['head']['sha']
    if event_name == 'push' and not event.get('created') and not event.get('deleted'):
        return event['before'], event['after']
    raise ValueError('unsupported event or branch creation/deletion')


def classify(repo, event_name, event):
    """Return full on ambiguity, missing history, malformed records or nonallowlisted paths."""
    try:
        base, head = event_range(event_name, event)
        if not all(isinstance(s, str) and SHA.fullmatch(s) and int(s, 16) for s in (base, head)):
            raise ValueError('missing, zero or invalid commit SHA')
        def git(*args):
            return subprocess.check_output(['git', '--no-replace-objects', *args], cwd=repo, stderr=subprocess.PIPE)
        for sha in (base, head):
            if git('cat-file', '-t', sha).strip() != b'commit':
                raise ValueError('range endpoint is not a commit')
        raw = git('diff', '--raw', '-z', '--no-renames', '--no-abbrev', base, head, '--')
        fields = raw.split(b'\0')
        if not raw or fields.pop() != b'' or len(fields) % 2:
            raise ValueError('empty or malformed complete diff')
        count = 0
        for index in range(0, len(fields), 2):
            header = fields[index].decode('ascii').split()
            path = fields[index + 1].decode('utf-8', errors='strict')
            if (len(header) != 5 or not header[0].startswith(':') or header[4] != 'M'
                    or header[0][1:] != '100644' or header[1] != '100644'):
                raise ValueError('unexpected status or file mode')
            if path not in DOCS_ONLY:
                return {'mode': 'full', 'reason': 'nonallowlisted path', 'changed_paths': index // 2 + 1}
            count += 1
        return {'mode': 'docs', 'reason': 'only reviewed documentation summaries', 'changed_paths': count}
    except (KeyError, TypeError, ValueError, UnicodeError, OSError, subprocess.CalledProcessError):
        return {'mode': 'full', 'reason': 'unavailable or ambiguous complete Git range', 'changed_paths': None}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=pathlib.Path, default=pathlib.Path('.'))
    parser.add_argument('--event-name', required=True)
    parser.add_argument('--event-path', type=pathlib.Path, required=True)
    parser.add_argument('--output', type=pathlib.Path)
    args = parser.parse_args()
    try:
        event = json.loads(args.event_path.read_text(encoding='utf-8'))
    except (ValueError, OSError):
        event = None
    result = classify(args.repo, args.event_name, event)
    if args.output:
        with args.output.open('a', encoding='utf-8') as stream:
            stream.write('mode=' + result['mode'] + '\n')
    print(json.dumps(result, sort_keys=True))


if __name__ == '__main__':
    main()
