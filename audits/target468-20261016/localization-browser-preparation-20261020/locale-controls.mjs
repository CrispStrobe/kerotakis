// Preparation only: importing this module launches nothing and makes no requests.
// Caller supplies its fresh real-browser page and authenticated release binding.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';

export const design = Object.freeze({
  status: 'prepared_not_executed_source_forwarding_gap',
  sourcePrerequisite: 'CombinedSolver must forward real time_boundaries before family checks can pass; do not waive missing events',
  sourceReviewed: 'b4daecf742be080577e9fda57fc1ef55aefef3d3',
  locales: ['de', 'fr'],
  casesPerLocale: 9,
  retainExistingBrowserControls: 37,
  retainExistingTargetedControls: 7,
  scientificSelections: 0,
});

const words = {
  de: {blank: 'Autoprotolyse', temperature: 'Temperatur', minimum: 'mindestens', catalyst: 'Katalysator', alternatives: ['Schwefelsäure', 'Wasserstoffionen', ' oder ']},
  fr: {blank: 'autoprotolyse', temperature: 'température', minimum: 'au moins', catalyst: 'catalyseur', alternatives: ['acide sulfurique', 'ions hydrogène', ' ou ']},
};
const phrase = (value, key) => {
  if (!value || typeof value !== 'object') return null;
  if (value.key === key) return value;
  for (const child of Object.values(value)) {
    const result = phrase(child, key);
    if (result) return result;
  }
  return null;
};
const allEvents = result => result.steps.flatMap(step => step.events);
const rendered = result => result.steps.flatMap(step => step.rendered ?? []).join('\n');
const slot = (value, name) => value.slots.find(([key]) => key === name)?.[1];
const amount = (v, key) => v.contents.filter(p => p.species === key).reduce((sum, p) => sum + p.moles, 0);

export async function runLocaleDeliveryChecks(page, binding) {
  assert.match(binding.expected_engine_source, /^[a-f0-9]{40}$/);
  assert.equal(new URL(page.url()).protocol, 'https:');
  const origin = new URL(page.url()).origin;
  const report = {design, binding, origin, checks: [], raw: {}, assets: {}, no_owner_credentials_supplied: true};
  const check = (name, condition, evidence) => {
    report.checks.push({name, passed: Boolean(condition), evidence});
    assert(condition, name);
  };
  try {
    assert(Object.keys(binding.frontend_assets).length >= 5, 'actual release asset binding required');
    for (const [name, digest] of Object.entries(binding.frontend_assets)) {
      const url = new URL('/' + name, origin);
      const response = await fetch(url, {redirect: 'error', signal: AbortSignal.timeout(15000), cache: 'no-store'});
      const bytes = Buffer.from(await response.arrayBuffer());
      const actual = createHash('sha256').update(bytes).digest('hex');
      report.assets[name] = {status: response.status, sha256: actual, bytes: bytes.length};
      assert(response.ok && actual === digest, 'served bytes differ: ' + name);
    }
    // The module imports and pool wiring are the actual shipped public APIs,
    // already used by the separately retained seven targeted browser controls.
    const run = async (locale, mode, prefix) => page.evaluate(async ({locale, mode, prefix}) => {
      const wasm = await import('/kerotakis_wasm.js');
      await wasm.default();
      const {PhreeqcPool} = await import('/kerotakis.mjs');
      const factory = (await import('/iphreeqc.mjs')).default;
      const lab = new wasm.Lab();
      try {
        lab.setLocale(locale);
        const meta = JSON.parse(lab.meta());
        const pool = await PhreeqcPool.create(factory, async file => {
          const response = await fetch('/db/' + file, {redirect: 'error'});
          if (!response.ok) throw Error('database status ' + response.status);
          return response.text();
        }, tag => lab.aqueousDatabase(tag));
        let calls = 0;
        lab.setSolver((tag, input) => { calls++; return pool.solve(tag, input); });
        const preparation = JSON.parse(lab.runScript(prefix));
        if (mode === 'family' || mode === 'live') return {meta, preparation, calls};
        const original = JSON.parse(lab.snapshot());
        const invalidated = JSON.parse(JSON.stringify(original));
        // Public restore permits an uncharacterized bench state. Only derived
        // readbacks are withdrawn; no material, identity, dose or temperature
        // is edited to manufacture a chemical result.
        const derived = ['solution', 'resolved', 'free_proton', 'free_hydroxide'];
        const physical = state => state.vessels.map(v => Object.fromEntries(Object.entries(v).filter(([key]) => !derived.includes(key))));
        for (const v of invalidated.vessels) {
          v.solution = null;
          v.resolved = {solution: null, valid: false};
          v.free_proton = 0;
          v.free_hydroxide = 0;
        }
        if (JSON.stringify(physical(original)) !== JSON.stringify(physical(invalidated))) throw Error('physical fixture changed');
        lab.restore(JSON.stringify(invalidated));
        const restored = JSON.parse(lab.snapshot());
        if (JSON.stringify(physical(restored)) !== JSON.stringify(physical(invalidated))) throw Error('restore changed physical fixture');
        const callsBeforeRead = calls;
        const lv2 = JSON.parse(lab.runScript('register lv2\nmeasure v1 ph\n'));
        const lv3 = JSON.parse(lab.runScript('register lv3\nmeasure v1 ph\n'));
        return {meta, preparation, original, invalidated, restored, lv2, lv3, callsBeforeRead, callsAfterRead: calls, after: JSON.parse(lab.snapshot())};
      } finally { lab.free(); }
    }, {locale, mode, prefix});
    const bound = raw => assert(binding.expected_engine_source.startsWith(raw.meta.git_rev) && raw.meta.git_rev.length >= 7, 'actual Wasm source differs');
    for (const locale of design.locales) {
      const text = words[locale];
      let raw = await run(locale, 'live', 'register lv3\nadd v1 water 2mol\nmeasure v1 ph\n');
      bound(raw); report.raw[locale + '-live'] = raw;
      const live = allEvents(raw.preparation).filter(e => e.event === 'measured' && e.instrument === 'ph_meter');
      check(locale + ' ordinary live water uses its actual analytical characterization', live.length === 1 && Number.isFinite(live[0].value) && !live[0].note_reason && Number.isFinite(raw.preparation.bench.vessels[0].solution?.ph), {events: live});
      for (const glucose of [false, true]) {
        raw = await run(locale, 'restored', 'register lv3\nadd v1 water 2mol\n' + (glucose ? 'add v1 glucose 0.001mol\n' : ''));
        bound(raw); report.raw[locale + '-restored-' + glucose] = raw;
        for (const result of [raw.lv2, raw.lv3]) {
          const events = allEvents(result).filter(e => e.event === 'measured' && e.instrument === 'ph_meter');
          assert.equal(events.length, 1); assert.equal(events[0].value, 7);
          assert.equal(events[0].note_reason?.key, 'measured.neutral-water-baseline');
          assert(rendered(result).includes(text.blank));
          assert(!rendered(result).includes('listed neutral unspeciated solutes'));
        }
        check(locale + ' restored uncharacterized ' + (glucose ? 'glucose' : 'water') + ' blank caveat renders at LV2 and LV3', raw.callsAfterRead === raw.callsBeforeRead && JSON.stringify(raw.after.vessels) === JSON.stringify(raw.restored.vessels), {rendered_lv2: rendered(raw.lv2), rendered_lv3: rendered(raw.lv3), calls: raw.callsAfterRead});
      }
      for (const [name, prefix] of [
        ['warm', 'add v1 water 2mol @ 40C\n'],
        ['acid', 'add v1 water 2mol\nadd v1 HCl 0.0001mol\n'],
        ['dry', 'register lv3\n'],
      ]) {
        raw = await run(locale, 'restored', prefix);
        bound(raw); report.raw[locale + '-negative-' + name] = raw;
        for (const result of [raw.lv2, raw.lv3]) {
          assert(!allEvents(result).some(e => e.event === 'measured' && e.instrument === 'ph_meter'));
          assert(allEvents(result).some(e => e.event === 'not_yet_modeled' && e.reason?.key === 'not-modeled.ph-meter-has-no-solution'));
          assert(!rendered(result).includes(text.blank));
        }
        check(locale + ' uncharacterized ' + name + ' state remains outside the blank domain', raw.callsAfterRead === raw.callsBeforeRead && JSON.stringify(raw.after.vessels) === JSON.stringify(raw.restored.vessels), {rendered: rendered(raw.lv3)});
      }
      const coldValues = [];
      for (const [name, temperature, gate] of [['cold15', 15, 'temperature'], ['cold25', 25, 'temperature'], ['warm65', 65, 'catalyst']]) {
        const script = 'register lv3\nadd v1 CH3COOH 0.01mol @ ' + temperature + 'C\nadd v1 ethanol 0.01mol @ ' + temperature + 'C\nwait 1s\n';
        raw = await run(locale, 'family', script);
        bound(raw); report.raw[locale + '-family-' + name] = raw;
        const event = allEvents(raw.preparation).find(e => e.event === 'not_yet_modeled' && e.reason?.key === 'not-modeled.family-declined-at-gate' && slot(e.reason, 'family')?.text?.text === 'fischer-esterification');
        assert(event, 'real shipped structural oracle must match Fischer family');
        const v = raw.preparation.bench.vessels[0];
        assert.equal(slot(event.reason, 'gate')?.term?.en, gate === 'temperature' ? 'temperature_k' : 'catalyst');
        assert(amount(v, 'CH3COOH') > 0 && amount(v, 'ethanol') > 0);
        assert(!allEvents(raw.preparation).some(e => e.event === 'org_reacted' && e.name === 'fischer-esterification'));
        assert.equal(amount(v, 'ethyl_acetate'), 0);
        const output = rendered(raw.preparation);
        if (gate === 'temperature') {
          const detail = phrase(event.reason, 'family-gate.temperature');
          assert(detail && output.includes(text.temperature) && output.includes(text.minimum));
          const actual = slot(detail, 'actual')?.number?.text;
          const window = phrase(detail, 'family-gate.window-min');
          const minimum = slot(window, 'min')?.number?.text;
          assert(Math.abs(Number(minimum) - 333.15) < 0.06);
          assert(output.includes(minimum.replace('.', ',')));
          assert(Number.isFinite(Number(actual)) && Math.abs(Number(actual) - v.temperature) < 0.06);
          assert(output.includes(actual.replace('.', ',')));
          assert(!output.includes('the vessel stands at') && !output.includes('temperature_k'));
          coldValues.push(actual);
        } else {
          const detail = phrase(event.reason, 'family-gate.catalyst-missing');
          assert(detail && JSON.stringify(detail).includes('H2SO4') && JSON.stringify(detail).includes('H+'));
          assert(output.includes(text.catalyst) && text.alternatives.every(word => output.includes(word)));
          assert(!output.includes('the family needs its catalyst present'));
          assert(v.temperature >= 333.15 && v.free_proton === 0 && amount(v, 'H2SO4') === 0);
        }
        check(locale + ' shipped Fischer ' + name + ' gate renders actual dynamic conditions', true, {event, rendered: output, temperature_k: v.temperature, external_calls: raw.calls});
      }
      assert.notEqual(coldValues[0], coldValues[1], 'two actual condition coordinates must differ');
    }
    assert.equal(report.checks.length, design.casesPerLocale * design.locales.length);
    report.all_checks_passed = report.checks.every(c => c.passed);
    return report;
  } catch (error) {
    report.all_checks_passed = false;
    report.error = String(error);
    error.localeDeliveryReport = report;
    throw error;
  }
}
