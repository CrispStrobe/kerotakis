# Optics combined full: preserved first failure

The full run on source `047d04513d1e55e162ad4269cbd5192e432e83e1` **failed**. Native tests passed627/627; core tests passed2205/2207. The failures were `lessons_produce_stable_output` and `five_release_lessons_match_numeric_scene_golden`. All22 optics Rust controls passed. Thermo184, the remaining peroxide-medium command, frontend21SSR/Svelte check, the later hook/fleet workflow stages and all33 original logical fleet jobs were not reached. A separate earlier curiosity check processed500 prompts with zero failures or baseline drift; that does not mean the33-job fleet ran or that500 experiments qualified scientifically.

Saved evidence shows disclosure changes:

- The five-lesson scene snapshot has222 differences:74 added `optical_ph_gaps` arrays,74 typed missing-pH notes and74 appended appearance sentences, all in `cabbage-rainbow`. Removing exactly those additions restores the original scene JSON. Every other field, command and event tag matches, including physical numbers at the snapshot's six-decimal precision. The other four lessons match completely.
- The combined113-lesson output has five changed caption strings: three in `cabbage-rainbow` and two in `lemonade-indicator`. The exact historical test source reports every differing line, capped at600; all five were shown. Reconstruction changes only these five lines. It is derived evidence, not a retained raw `lessons.actual.json` file.

`evidence/scene.actual.extracted.json` contains the exact inner JSON decoded from the captured Rust assertion string. It is an extracted artifact, not the complete stderr log. `evidence/frozen-five-line-diff.json` preserves the bounded actual/expected line text; the full raw stderr remains private. `records/SCENE-STRUCTURED-DIFF.json` lists every222 scene path change. Expected golden files are read from the exact source commit; none was modified. The parent independently reproduced all four original mathematical review outputs byte for byte.

The private collection contained613 manifest members plus the manifest itself (614 collected files). All original member bytes and the debug CLI/failing test executable gzip payloads were independently authenticated before publication. This capsule publishes their SHA receipts and source/actual668 lock bindings, without the4GiB raw collection or executable bytes. It does not claim those omitted bytes can be reverified using this capsule alone. Extracted and derived artifacts are identified explicitly; this is not a redacted raw bundle.

Run the public saved-only reader with an available Git repository containing the source commit and a fresh output directory:

```sh
python3 audits/target468-20261016/optics-combined-full-first-failure-20261021/review_public.py --repo . --out /tmp/optics-first-failure-review
```

It reviews only public saved inputs and source blobs, proves the bounded scene projection and five-string reconstruction, and calls no chemistry engine. Its actual publication-time exit/status is recorded separately. Initial reader assumptions about compact versus pretty plan serialization, Rust's standard should-panic suffix, and the ARCH-001 assertion identifier are preserved as safe correction records. The earlier null-liquid traversal error was corrected without changing scientific inputs or assertions.

These results justify reviewing this specific coverage/disclosure change class; they do not authorize a blanket golden refresh. Later source `e648249119e1fe028735bf31723da2867d87db77` adds generic species-name resolution and requires its own actual outputs. No full, platform, product or scientific acceptance is granted here.
