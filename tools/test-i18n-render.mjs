/**
 * Prove the German build is actually German (I18N-1, I18N-2, GUI-087).
 *
 * The shell degrades silently by design: `t()` returns its English key
 * when the dictionary has no answer, and `tEngine()` falls back per
 * field. That is the right behaviour — a missing string should never
 * blank a label — and it is exactly why a coverage count is not proof.
 * `tools/codex-locale-lint.py` can report 100% while the map still
 * renders English, because the map's German lives in a different file
 * keyed by a de-slugged identifier, and nothing but a browser knows
 * whether those keys match what the component actually asks for.
 *
 * So this loads the real page in German and reads the rendered text.
 *
 * Usage: node tools/test-i18n-render.mjs <payload-dir>
 */
import { serve, browser, waitFor } from "./lib/headless.mjs";

const PAYLOAD = process.argv[2];
if (!PAYLOAD) {
  console.error("usage: node tools/test-i18n-render.mjs <payload-dir>");
  process.exit(2);
}

let failures = 0;
const check = (name, ok, detail = "") => {
  console.log(`   ${ok ? "ok  " : "FAIL"}  ${name}${detail ? `  ${detail}` : ""}`);
  if (!ok) failures++;
};

const { server, origin } = await serve(PAYLOAD);
const page = await browser();

/** Click the first button whose visible text matches, or return false. */
const clickByText = (re) =>
  page.evaluate(`(() => {
    const b = [...document.querySelectorAll('button')]
      .find((el) => ${re}.test((el.textContent || "").trim()));
    if (!b) return false;
    b.click();
    return true;
  })()`);

/** The map and the toolbox both live inside the utilities drawer. */
const openUtilities = () =>
  page.evaluate(`(() => {
    const b = [...document.querySelectorAll('button')]
      .find((el) => /Werkzeuge und Dateien|tools and files/i.test(el.getAttribute('aria-label') || ""));
    if (!b) return false;
    b.click();
    return true;
  })()`);

try {
  // Choose German before the app boots: the shell reads this key on
  // startup, so setting it afterwards would test the switcher instead of
  // the translation.
  await page.goto(`${origin}/app/`);
  await page.evaluate(`window.localStorage.setItem("kerotakis.locale", "de")`);
  await page.goto(`${origin}/app/`);

  const ready = await waitFor(page, `document.querySelector('form.bar input')`, { timeout: 60000 });
  if (!ready) throw new Error("the command bar never appeared");

  check(
    "the document declares German",
    (await page.evaluate(`document.documentElement.lang`)) === "de",
  );

  // ---- the map -------------------------------------------------------
  // Its node labels are the case that motivated this file: they come from
  // the dictionary via a de-slugged concept id, so a rename on either
  // side breaks them without breaking anything else.
  check("the utilities drawer opens", (await openUtilities()) === true);
  // The command bar is ready before the independently fetched codex export.
  // Until that fetch resolves the map button is intentionally absent, so
  // clicking immediately is a race against the network/filesystem rather
  // than a localization assertion.
  const mapAvailable = await waitFor(
    page,
    `[...document.querySelectorAll('button')]
       .some((el) => /^Karte$/.test((el.textContent || "").trim()))`,
    { timeout: 30000 },
  );
  check("the map becomes available", mapAvailable === true);
  check("the map opens", (await clickByText("/^Karte$/")) === true);

  const nodes = await waitFor(page, `document.querySelectorAll('button.node').length > 0`,
                              { timeout: 30000 });
  check("the map draws its nodes", nodes === true);
  if (!nodes) console.log("      (the concept checks below did not run)");

  if (nodes) {
    const labels = JSON.parse(await page.evaluate(
      `JSON.stringify([...document.querySelectorAll('button.node')]
         .map((n) => (n.childNodes[0]?.textContent ?? "").trim()).filter(Boolean))`));
    check("the map has concepts", labels.length > 20, `${labels.length} nodes`);

    // A label that fell through renders as its own English words, so look
    // for those rather than for the absence of German.
    const english = labels.filter((l) =>
      /^(acids|bases|activity|buffers|solubility|equilibrium|catalysis|electrolysis|titration|activation energy|ionic strength)$/i
        .test(l));
    check("no concept fell back to English", english.length === 0, english.slice(0, 5).join(", "));

    const german = labels.filter((l) => /[äöüßÄÖÜ]|ung\b|keit\b|heit\b|säure|Gleichgewicht/i.test(l));
    check("the concepts read as German", german.length >= 10, `${german.length} clearly German`);
  }

  // ---- the toolbox ---------------------------------------------------
  // GUI-087's purpose and validity come from the *engine*, through
  // tEngine(), not the dictionary — a different path that fails on its own.
  // Reload rather than hunt for a close button: the map is a full overlay,
  // and a stale one leaves ITS nav in the DOM for the next query to find —
  // which is how the first version of this test reported seven relations
  // while the toolbox had never opened.
  await page.goto(`${origin}/app/`);
  await waitFor(page, `document.querySelector('form.bar input')`, { timeout: 60000 });
  await openUtilities();
  check("the toolbox opens", (await clickByText("/^Werkzeugkasten$/")) === true);

  const listed = await waitFor(page, `document.querySelectorAll('nav button').length >= 7`,
                               { timeout: 30000 });
  check("the engine lists its seven relations", listed === true);

  if (listed) {
    await page.evaluate(`document.querySelectorAll('nav button')[0].click()`);
    await waitFor(page, `document.querySelector('.equation')`, { timeout: 10000 });

    const purpose = await page.evaluate(`document.querySelector('.purpose')?.textContent?.trim() ?? ""`);
    const validity = await page.evaluate(`document.querySelector('.validity')?.textContent?.trim() ?? ""`);
    const source = await page.evaluate(`document.querySelector('.source')?.textContent?.trim() ?? ""`);

    check("the relation says what it is for", purpose.length > 20, JSON.stringify(purpose.slice(0, 50)));
    check("the relation says where it holds", validity.length > 20, JSON.stringify(validity.slice(0, 50)));
    check("the purpose is in German", /[äöüßÄÖÜ]|\b(die|der|das|wie|einer)\b/.test(purpose));
    check("the validity carries its label", /Wo sie gilt/.test(validity));
    // GUI-096: the citation is shown before anything is computed, so a
    // learner can check the claim rather than take it. A year is the part
    // that makes it checkable.
    check("the relation names its source", /\b1[89]\d\d\b/.test(source), JSON.stringify(source.slice(0, 60)));
    check("the source carries its label", /Woher sie stammt/.test(source));
  }

  // ---- the experiment prose -------------------------------------------
  // The part that was English while every counter read 100%. It travels a
  // different road from the map's labels: not the shell dictionary, but
  // the JSON the codex crate exports, through typed structs that dropped
  // every `_de` field they were not told about.
  await page.goto(`${origin}/app/`);
  await waitFor(page, `document.querySelector('form.bar input')`, { timeout: 60000 });
  check("the research library opens", (await clickByText("/Forschungsbibliothek/")) === true);

  const grouped = await waitFor(page,
    `document.querySelectorAll('dialog.panel article').length > 0`, { timeout: 20000 });
  // A section that quietly skips is a section that reports success without
  // testing anything, which is how the catalogue stayed English through a
  // green run of this very file. Not finding the list is a failure.
  check("the catalogue renders its list", grouped === true);
  if (grouped) {
    const entries = await page.evaluate(`document.querySelectorAll('dialog.panel article').length`);
    check("the catalogue lists its experiments", entries > 50, `${entries} entries`);

    // A card that actually carries a script: the register prose and the
    // prediction below only exist for those, and which card sorts first
    // depends on the reader's language.
    await page.evaluate(`(() => {
      const panel = document.querySelector('dialog.panel');
      const card = panel?.querySelector('article[data-id="strong-base"]')
        || panel?.querySelector('article[data-run="script"]');
      card?.querySelector('button.details')?.click();
    })()`);
    await waitFor(page, `document.querySelector('.prose')`, { timeout: 20000 });

    const title = await page.evaluate(`document.querySelector('h2')?.textContent?.trim() ?? ""`);
    const prose = await page.evaluate(`document.querySelector('.prose')?.textContent?.trim() ?? ""`);
    check("the entry has a German title", /[äöüßÄÖÜ]|^(Starke|Der|Die|Das|Ein|Eine)\b/.test(title),
          JSON.stringify(title.slice(0, 40)));
    check("the register prose is written", prose.length > 60, `${prose.length} chars`);
    // Look for English function words rather than for German characters: a
    // German sentence may happen to contain none, but an English one will
    // almost always contain one of these.
    check("the register prose is not English",
          !/\b(the|and|with|that|which|water is|this liquid)\b/i.test(prose),
          JSON.stringify(prose.slice(0, 60)));

    // The predict tab: question and options come through the export's
    // typed structs, which is where they were being dropped.
    await clickByText("/vorhersagen|predict/i");
    await waitFor(page, `document.querySelector('.question')`, { timeout: 20000 });
    const question = await page.evaluate(`document.querySelector('.question')?.textContent?.trim() ?? ""`);
    const options = JSON.parse(await page.evaluate(
      `JSON.stringify([...document.querySelectorAll('button.option')].map((b) => b.textContent.trim()))`));
    check("the prediction asks in German",
          question.length > 20 && !/\b(the|what will|goes into)\b/i.test(question),
          JSON.stringify(question.slice(0, 55)));
    check("the answers are offered in German",
          options.length > 1 && !options.some((o) => /\b(is not|so the|cannot|because)\b/i.test(o)),
          JSON.stringify(options.slice(0, 2)));
  }

  // ---- the journal and the vessel line --------------------------------
  // Engine prose: composed in Rust out of fragments, so neither the shell
  // dictionary nor the codex `_de` keys can reach it. This is also the
  // only place the DECIMAL is checked end to end — the engine writes
  // 11,0686 because it knows the locale while the number is still a
  // float, and no layer downstream of it could.
  await page.goto(`${origin}/app/`);
  await waitFor(page, `document.querySelectorAll('button').length > 3`, { timeout: 60000 });
  await clickByText("/Sandbox betreten|enter Sandbox/");
  const barReady = await waitFor(page,
    `!!document.querySelector('form.bar input') &&
     !document.querySelector('form.bar input').disabled`, { timeout: 60000 });
  check("the bench takes commands", barReady === true);

  if (barReady) {
    await page.evaluate(`(() => {
      const input = document.querySelector('form.bar input');
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")
        .set.call(input, "add v1 water 200mL");
      input.dispatchEvent(new Event("input", { bubbles: true }));
      input.form.dispatchEvent(new SubmitEvent("submit", { bubbles: true, cancelable: true }));
    })()`);
    await waitFor(page, `!document.querySelector('form.bar input').disabled`, { timeout: 60000 });

    const journal = await page.evaluate(`(() => {
      const t = [...document.querySelectorAll('*')].filter((n) => !n.children.length)
        .map((n) => (n.textContent || "").trim());
      return t.find((x) => /^v1: \\+/.test(x)) ?? "";
    })()`);
    check("the journal names the species in German", /Wasser/.test(journal),
          JSON.stringify(journal.slice(0, 50)));
    check("the journal counts with a decimal comma", /,\d/.test(journal) && !/\.\d/.test(journal),
          JSON.stringify(journal.slice(0, 50)));

    // The vessel summary lives behind the dock's measurement-tools button.
    // Use its accessible contract: the visible label changed when Details
    // became a real instrument drawer, while the title still identifies the
    // action and selected vessel unambiguously.
    const openedMeasurements = await page.evaluate(`(() => {
      const b = [...document.querySelectorAll('button')].find((el) =>
        /Messgeräte|Messwerkzeuge|measurement tools/i.test(
          [el.textContent, el.getAttribute('title'), el.getAttribute('aria-label')]
            .filter(Boolean).join(' '),
        ));
      if (!b) return false;
      b.click();
      return true;
    })()`);
    check("the selected vessel opens its measurement tools", openedMeasurements === true);
    await waitFor(page, `(() => {
      const t = [...document.querySelectorAll('*')].filter((n) => !n.children.length)
        .map((n) => (n.textContent || "").trim());
      return t.some((x) => /^v1 \\(/.test(x));
    })()`, { timeout: 20000 });
    const vessel = await page.evaluate(`(() => {
      const t = [...document.querySelectorAll('*')].filter((n) => !n.children.length)
        .map((n) => (n.textContent || "").trim());
      return t.find((x) => /^v1 \\(/.test(x)) ?? "";
    })()`);
    check("the vessel line names its glassware in German", /Becherglas/.test(vessel),
          JSON.stringify(vessel.slice(0, 56)));
    check("the vessel line is not English", !/\(beaker\)|mL liquid|open to atmosphere/.test(vessel));
    check("the vessel line measures with commas", /25,\d/.test(vessel), JSON.stringify(vessel.slice(0, 40)));
  }
} catch (err) {
  console.error(`i18n render: ${err.stack ?? err.message}`);
  failures++;
} finally {
  await page.close?.();
  server.close();
}

console.log(failures ? `\n${failures} check(s) failed` : "\nthe German build renders German");
process.exit(failures ? 1 : 0);
