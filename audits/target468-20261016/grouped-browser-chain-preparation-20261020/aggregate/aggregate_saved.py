"""All64 aggregation of two independently executed saved reviews; no product calls.

The raw predicates and authenticated source must already have been reviewed.
Matching files alone cannot establish that a reviewer actually ran; invocation
records remain a separately required external parent provenance check.
"""
import argparse
import json
from pathlib import Path
from saved_envelope import authenticate, digest, load

SOURCE = '33e043ef4a4a4225e5ea8d49ba93e8df99716c5f'
FILES = {
    'browser37': 'BROWSER37-ACCEPTED.json',
    'targeted7': 'PRIOR44-ACCEPTED.json',
    'locale18': 'context18/LOCALE18-REVIEW-ACCEPTED.json',
    'heat2': 'heat-review/HEAT2-REVIEW-ACCEPTED.json',
}
ORIGINAL = {
    'locale18': 'context18/original-locale18-result.json',
    'heat2': 'heat-review/original-heat2-result.json',
}
PINS = {
    'locale18': ('84fe50ca53fb67a27bdda1d40869df70a56948b31c01daa1ad79b8c1597cec6a',
                 '4b66938c88db0da8863d44a10a7e4a34e72aa578891133e72ac1818d7dd4e063'),
    'heat2': ('817d9fdf9106d5581919a5b7696658074033e6737154c1336b5cbc0f3d516252',
              '705915e2fdb218c754c69c7408d135f1432bd521343336383f9aeadee0ae9264'),
}


def require(value, message):
    if not value:
        raise ValueError(message)


def equal_reviews(first, second, validated_links=()):
    # Exclude only the root wall-clock stamp; never clocks or physical values
    # inside raw results, source identities, report hashes or criterion details.
    allowed = {'unchanged_browser_acceptance_sha256', 'prior37_receipt_sha256',
               'prior44_receipt_sha256', 'original_reader_result_sha256',
               'postcollection_context_sha256', 'prior62_original_result_sha256'}
    require(set(validated_links) <= allowed, 'Only independently validated parent-chain hashes may normalize')
    a = {k: v for k, v in first.items() if k != 'utc' and k not in validated_links}
    b = {k: v for k, v in second.items() if k != 'utc' and k not in validated_links}
    require(a == b, 'Independent saved reviews disagree')


def validate_counts(receipts, locale_result, heat_result):
    require(receipts['browser37']['unchanged_checks'] == 37,
            'Missing original37 acceptance')
    prior = receipts['targeted7']
    require(prior['unchanged_API_controls'] == 11 and
            prior['unchanged_browser_controls'] == 37 and
            prior['fresh_targeted_browser_controls'] == 7,
            'Missing original PRIOR44 acceptance')
    require(locale_result['API_controls'] == 11 and
            locale_result['unchanged_browser_controls'] == 37 and
            locale_result['prior_targeted_browser_controls'] == 7 and
            locale_result['supplemental_locale_browser_controls'] == 18 and
            locale_result['all62_browser_checks_accepted'] is True,
            'Retained62 acceptance incomplete')
    require(heat_result['state'] == 'SAVED_HEAT_PREDICATES_ACCEPTED' and
            heat_result['source'] == SOURCE and
            heat_result['reviewed_candidate_source'] ==
            '18f2bebd6f9aafe85d3ba1d76e2c2f9cd5d11e9a' and
            heat_result['prior62_acceptance_replaced'] is False and
            set(heat_result['details']) == {'de', 'fr'} and
            heat_result['scientific_selections'] == 0 and
            heat_result['full_thermal_ledger_or_rate_calibration_claim'] is False,
            'Separate supplemental heat acceptance incomplete')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('first-review', 'second-review', 'evidence', 'design', 'worker',
                 'issue', 'api', 'verified', 'native-reference', 'out'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    first, second = args.first_review.resolve(), args.second_review.resolve()
    require(first != second and not first.is_relative_to(second) and
            not second.is_relative_to(first), 'Independent review directories overlap')
    require(not args.out.exists(), 'Never overwrite an accepted aggregation')
    envs = {group: authenticate(
        evidence=args.evidence, design_path=args.design, worker_path=args.worker,
        issue_path=args.issue, api_path=args.api, verified_path=args.verified,
        native_reference_path=args.native_reference, group=group) for group in FILES}
    base = envs['browser37']
    require(base['S'] == SOURCE, 'Different runtime source')
    frozen = {digest(p) for p in base['input_paths']}
    require(digest(Path(__file__)) in frozen and
            digest(Path(__file__).with_name('saved_envelope.py')) in frozen,
            'Aggregator was not frozen before execution')
    producer = load(base['E'] / 'COMPLETED.json')
    require(producer['all_four_children_executed_with_zero_status'] is True and
            producer['parent_acceptance_assigned'] is False and
            [row['group'] for row in producer['groups']] == list(FILES) and
            all(row['exit_status'] == 0 for row in producer['groups']),
            'Producer did not execute all four groups')
    for row in producer['groups']:
        require(row['report_sha256'] == digest(envs[row['group']]['e'] / 'actual-browser/report.json'),
                'Producer observed report hash differs from actual raw')
    receipts = {}
    other_receipts = {}
    for group, filename in FILES.items():
        x, y = first / filename, second / filename
        require(not x.is_symlink() and not y.is_symlink() and
                not x.samefile(y), 'Review file alias or same inode')
        a, b = load(x), load(y)
        # Context/parent hashes can legitimately differ between independently
        # timestamped review sets. Validate each own chain before comparison.
        env = envs[group]
        require(a['source'] == SOURCE and a['engine'] == base['binding']['engine'] and
                a['execution_design_sha256'] == env['execution_design_sha256'] and
                a['top_manifest_sha256'] == env['top_manifest_sha256'] and
                a['external_issue_sha256'] == env['external_issue_sha256'] and
                a['verified_artifact_receipt_sha256'] == env['verified_sha256'] and
                a['api_acceptance_sha256'] == env['api_sha256'] and
                a['native_reference_sha256'] == env['native_reference_sha256'],
                'Review source/design/manifest/issue mismatch')
        report = env['e'] / 'actual-browser/report.json'
        if group in ('browser37', 'targeted7'):
            report_key = 'browser_report_sha256' if group == 'browser37' else 'targeted_report_sha256'
            manifest_key = 'manifest_sha256' if group == 'browser37' else 'targeted_manifest_sha256'
            worker_key = 'combined_worker_sha256'
            expected_ast = ('1620105a7a9b67b0108a9c93535665edd4bc701a62310c43a8dae4c4895bd046'
                            if group == 'browser37' else
                            'b8cd7a7c50e2c1c31f4816af1638d5da10b7228f01e1a6b06c7c96bf544e3267')
            require(a['original_assertion_AST_verified'] is True and
                    a['original_assertion_AST_sha256'] == expected_ast and
                    a['postcollection_saved_review'] is True and
                    a['owner_credentials_supplied'] is False and
                    a['new_browser_engine_provider_calls'] == 0 and
                    a['scientific_catalog_additions'] == 0,
                    'Original37/7 review claims contradict frozen raw review scope')
        else:
            report_key, manifest_key, worker_key = 'report_sha256', 'group_manifest_sha256', 'worker_sha256'
            require(digest(first / ORIGINAL[group]) == a['original_reader_result_sha256'],
                    'Original reader result bytes mismatch')
            require(a['original_reader_sha256'] in frozen and
                    a['original_predicates_sha256'] in frozen,
                    'Original final predicate/reader not frozen before issue')
            require((a['original_reader_sha256'], a['original_predicates_sha256']) == PINS[group],
                    'Original reader/predicate identity changed')
            require(a['state'] == ('GROUPED_LOCALE18_SAVED_REVIEW_ACCEPTED' if group == 'locale18'
                                   else 'GROUPED_HEAT2_SAVED_REVIEW_ACCEPTED') and
                    a['all64_accepted'] is False and a['scientific_catalog_additions'] == 0 and
                    a['browser_engine_provider_calls'] == 0, 'Invalid final review state or scope')
            require(a['combined_worker_sha256'] == env['worker_sha256'],
                    'Final wrapper worker alias differs')
            if group == 'heat2':
                require(a['binding_sha256'] == base['design']['heat_binding_sha256'],
                        'Final heat wrapper binding differs')
        require(a[report_key] == digest(report) and
                a[manifest_key] == env['group_manifest_sha256'] and
                a[worker_key] == env['worker_sha256'] and
                a['config_canonical_sha256'] == env['config_canonical_sha256'],
                'Actual raw report or worker mismatch')
        receipts[group] = a
        other_receipts[group] = b
    for directory, current in ((first, receipts), (second, other_receipts)):
        require(current['targeted7']['unchanged_browser_acceptance_sha256'] ==
                digest(directory / FILES['browser37']), 'PRIOR44 parent37 link mismatch')
        for group in ('locale18', 'heat2'):
            require(current[group]['prior37_receipt_sha256'] == digest(directory / FILES['browser37']) and
                    current[group]['prior44_receipt_sha256'] == digest(directory / FILES['targeted7']),
                    'Final review parent receipt link mismatch')
            require(current[group]['original_reader_result_sha256'] == digest(directory / ORIGINAL[group]),
                    'Each original reader result must bind its own review set')
        prior62 = load(directory / ORIGINAL['locale18'])
        require(prior62['prior44_receipt_sha256'] == digest(directory / FILES['targeted7']),
                'Original locale result parent44 mismatch')
        require(prior62['source'] == SOURCE and prior62['engine'] == base['binding']['engine'] and
                prior62['locale_report_sha256'] == digest(envs['locale18']['e'] / 'actual-browser/report.json') and
                prior62['manifest_sha256'] == envs['locale18']['group_manifest_sha256'],
                'Original locale result does not describe actual raw bytes')
        original_heat = load(directory / ORIGINAL['heat2'])
        require(original_heat['report_sha256'] == digest(envs['heat2']['e'] / 'actual-browser/report.json') and
                original_heat['binding_sha256'] == base['design']['heat_binding_sha256'],
                'Original heat result report/binding mismatch')
        require(current['heat2']['prior62_original_result_sha256'] == digest(directory / ORIGINAL['locale18']),
                'Heat review lacks its own genuine retained62')
        context_path = directory / 'context18/postcollection-context.json'
        require(current['locale18']['postcollection_context_sha256'] == digest(context_path),
                'Locale context bytes mismatch')
        context = load(context_path)
        require(context['context_kind'] == 'POSTCOLLECTION_PARENT_REVIEW_NOT_PREISSUE_AUTHORIZATION' and
                context['immutable_execution_design_sha256'] == base['execution_design_sha256'] and
                context['immutable_config_canonical_sha256'] == base['config_canonical_sha256'] and
                context['immutable_top_manifest_sha256'] == base['top_manifest_sha256'] and
                context['immutable_locale18_group_manifest_sha256'] == envs['locale18']['group_manifest_sha256'] and
                context['actual_external_issue_sha256'] == base['external_issue_sha256'] and
                context['source'] == SOURCE and context['binding'] == base['binding'] and
                context['worker_sha256'] == base['worker_sha256'] and
                context['saved_reader_sha256'] == PINS['locale18'][0] and
                context['saved_predicates_sha256'] == PINS['locale18'][1] and
                context['genuine_parent37_receipt_sha256'] == digest(directory / FILES['browser37']) and
                context['genuine_parentPRIOR44_receipt_sha256'] == digest(directory / FILES['targeted7']) and
                context['prior44_receipt_sha256'] == digest(directory / FILES['targeted7']) and
                context['producer_raw_or_design_modified'] is False,
                'Postcollection context/source/parent chain mismatch')
        expected_prior = {'API-ACCEPTED.json': args.api, 'verified-release.json': args.verified,
                          'BROWSER37-ACCEPTED.json': directory / FILES['browser37'],
                          'PRIOR44-ACCEPTED.json': directory / FILES['targeted7']}
        require(context['prior_acceptance_files_sha256'] == {n: digest(p) for n, p in expected_prior.items()},
                'Postcollection context prior file closure mismatch')
        for name, original_path in expected_prior.items():
            require(digest(directory / 'context18/prior-acceptance' / name) == digest(original_path),
                    'Copied prior receipt differs')
        validate_counts(current, prior62, load(directory / ORIGINAL['heat2']))
    links = {
        'browser37': (),
        'targeted7': ('unchanged_browser_acceptance_sha256',),
        'locale18': ('prior37_receipt_sha256', 'prior44_receipt_sha256',
                     'original_reader_result_sha256', 'postcollection_context_sha256'),
        'heat2': ('prior37_receipt_sha256', 'prior44_receipt_sha256', 'prior62_original_result_sha256'),
    }
    for group in FILES:
        equal_reviews(receipts[group], other_receipts[group], links[group])
    locale = load(first / ORIGINAL['locale18'])
    heat = load(first / ORIGINAL['heat2'])
    validate_counts(receipts, locale, heat)
    equal_reviews(locale, load(second / ORIGINAL['locale18']), ('prior44_receipt_sha256',))
    equal_reviews(heat, load(second / ORIGINAL['heat2']))
    require(receipts['heat2']['prior62_original_result_sha256'] ==
            digest(first / ORIGINAL['locale18']), 'Heat review lacks genuine retained62')
    result = dict(state='ALL64_SAVED_REVIEW_AGREEMENT_PARENT_INVOCATION_PROVENANCE_REQUIRED',
                  source=SOURCE, engine=base['binding']['engine'],
                  retained_browser_controls=62, supplemental_heat_controls=2,
                  all64_raw_saved_reviews_agree=True,
                  actual_external_invocation_provenance_still_requires_parent_review=True,
                  execution_design_sha256=base['execution_design_sha256'],
                  config_canonical_sha256=base['config_canonical_sha256'],
                  top_manifest_sha256=base['top_manifest_sha256'],
                  worker_sha256=base['worker_sha256'],
                  external_issue_sha256=base['external_issue_sha256'],
                  review_files_sha256={label: {name: digest(root / path) for name, path in FILES.items()}
                                      for label, root in [('first', first), ('second', second)]},
                  scientific_selections=0, empirical_calibration_claim=False)
    with args.out.open('x', encoding='utf-8') as stream:
        json.dump(result, stream, indent=2, allow_nan=False)
        stream.write('\n')


if __name__ == '__main__':
    main()
