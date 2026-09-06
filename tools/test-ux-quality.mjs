/** Browser-level UX invariants: layout, accessible controls, touch size and reduced motion. */
import { serve, browser, waitFor } from "./lib/headless.mjs";

const PAYLOAD = process.argv[2];
if (!PAYLOAD) {
  console.error("usage: node tools/test-ux-quality.mjs <payload-dir>");
  process.exit(2);
}

let failures = 0;
const check = (name, ok, detail = "") => {
  console.log(`   ${ok ? "ok  " : "FAIL"}  ${name}${detail ? `  ${detail}` : ""}`);
  if (!ok) failures++;
};

const { server, origin } = await serve(PAYLOAD);
const page = await browser();

const viewport = (width, height) => page.cdp.send("Emulation.setDeviceMetricsOverride", {
  width, height, deviceScaleFactor: 1, mobile: width < 700,
}, page.sessionId);

/** The deployment enters the bench directly, so there is nothing to press:
 * waiting for the stage IS the entry. The mode is whichever laboratory was
 * last stood in, which is why callers set `kerotakis.mode.v1` first. */
const openBench = async () => {
  await waitFor(page, `document.querySelector('main .bench-pane')`, { timeout: 60000 });
  return waitFor(page, `document.querySelector('main')`, { timeout: 20000 });
};

/** GUI-102: one cupboard, reached from the dock, the MESSEN row and the shelf
 * pane. Three surfaces used to list overlapping equipment; this asserts the
 * survivor exists, groups what it holds, and names every item. */
const openCupboard = async () => {
  await page.evaluate(`(() => {
    const button = [...document.querySelectorAll('button')].find((item) =>
      /Geräteschrank|equipment cabinet/i.test(
        [item.textContent, item.getAttribute('title'), item.getAttribute('aria-label')]
          .filter(Boolean).join(' ')));
    button?.click();
  })()`);
  return waitFor(page, `document.querySelector('dialog.cupboard')`, { timeout: 20000 });
};

const cupboardAudit = () => page.evaluate(`(() => {
  const panel = document.querySelector('dialog.cupboard');
  if (!panel) return JSON.stringify({ catalogue: 0, shelves: 0, items: 0, unnamed: 1, info: 0, viewportOverflow: 0, locked: 0, tally: 0, kitNames: 0, unexplainedShelves: 1 });
  const items = [...panel.querySelectorAll('button.item')];
  const rect = panel.getBoundingClientRect();
  // The catalogue size is on the dialog rather than read out of the header
  // fraction, because since GUI-103 the fraction is printed only while
  // something is still locked — in Sandbox every tool is reachable and
  // "34/34" would be a fact rather than progress. The denominator itself is
  // progression-independent: it counts what a learner can EVER have, which
  // is why it can be asserted at all. The number of RENDERED items is not,
  // because a Story learner is shown what they have earned.
  //
  // Split rather than match: this whole function is a template literal, so a
  // regex written here loses its backslashes on the way to the browser — an
  // escaped slash arrives as a bare one and closes the literal early.
  const shelves = [...panel.querySelectorAll('section.shelf')];
  return JSON.stringify({
    catalogue: Number(panel.getAttribute('data-catalogue') || 0),
    shelves: shelves.length,
    unexplainedShelves: shelves.filter((shelf) => !(shelf.querySelector('h3')?.getAttribute('title') || '').trim()).length,
    items: items.length,
    unnamed: items.filter((item) => !(item.textContent.trim() || item.getAttribute('aria-label'))).length,
    info: panel.querySelectorAll('button.info-toggle').length,
    locked: items.filter((item) => item.classList.contains('locked')).length,
    tally: panel.querySelector('header b') ? 1 : 0,
    // The candle: one slot in both states, named for the kit in one of them.
    kitNames: items.filter((item) => /candle and wick|Kerze und Docht/.test(item.textContent)).length,
    viewportOverflow: Math.max(0, rect.right - document.documentElement.clientWidth, -rect.left),
  });
})()`);

/** GUI-103: the sets are a chip in the cupboard header, not a shelf. */
const toggleSets = async () => {
  await page.evaluate(`document.querySelector('dialog.cupboard button.sets-chip')?.click()`);
  return JSON.parse(await cupboardAudit());
};

/** GUI-103: the MESSEN strip is four recents plus the cupboard door, and it
 * never repeats the three readings the vessel dock carries. */
const openStrip = async () => {
  await page.evaluate(`(() => {
    const button = [...document.querySelectorAll('button')].find((item) =>
      /Messgeräte|measurement tools/i.test(
        [item.textContent, item.getAttribute('title'), item.getAttribute('aria-label')]
          .filter(Boolean).join(' ')));
    button?.click();
  })()`);
  return waitFor(page, `document.querySelector('.tray button')`, { timeout: 20000 });
};

const stripAudit = () => page.evaluate(`(() => {
  const tray = document.querySelector('.tray');
  if (!tray) return JSON.stringify({ instruments: 0, doors: 0, repeated: 1 });
  const buttons = [...tray.querySelectorAll('button')];
  const tokens = buttons.map((item) => item.getAttribute('data-token')).filter(Boolean);
  return JSON.stringify({
    instruments: tokens.length,
    doors: buttons.filter((item) => item.classList.contains('cupboard-door')).length,
    repeated: tokens.filter((token) => ['eyes', 'thermometer', 'ph'].includes(token)).length,
  });
})()`);

/** GUI-472: the workstation strip is an overlay ON the stage, so it has to
 * stay inside the viewport at every width it is offered at. The panel it
 * replaced was a three-column grid whose middle column held the assembly;
 * the assembly now annotates the vessel and the strip is one row, but a
 * strip that hangs off a 390 px screen is the same bug wearing less. */
const openApparatus = async (action) => {
  // The dock button is disabled while the session is busy, and a click on a
  // disabled button is a silent no-op — so wait for it to be clickable
  // rather than clicking once and asking why nothing opened.
  const ready = await waitFor(page,
    `(() => { const b = document.querySelector('.actions button[data-action="${action}"]'); return Boolean(b) && !b.disabled; })()`,
    { timeout: 30000 });
  if (!ready) return false;
  await page.evaluate(`document.querySelector('.actions button[data-action="${action}"]')?.click()`);
  return waitFor(page, `document.querySelector('section.apparatus')`, { timeout: 20000 });
};

/** GUI-473: the pour chooser stands with the vessel it pours out of. It was
 * a banner between the top bar and the stage, which is the one place it
 * could not be: it named two vessels drawn below it and pushed them down by
 * its own height to do it. */
const pourAudit = () => page.evaluate(`(() => {
  const overlay = document.querySelector('.pour-overlay');
  if (!overlay) return JSON.stringify({ present: false });
  const rect = overlay.getBoundingClientRect();
  const surface = document.querySelector('.work-surface')?.getBoundingClientRect() ?? null;
  return JSON.stringify({
    present: true,
    banner: Boolean(document.querySelector('.transfer-banner:not(.mix-banner)')),
    onStage: Boolean(surface)
      && rect.top >= surface.top - 1 && rect.bottom <= surface.bottom + 1
      && rect.left >= surface.left - 1 && rect.right <= surface.right + 1,
    fractions: overlay.querySelectorAll('.fractions button').length,
    highlighted: document.querySelectorAll('.vessel.transfer-target').length,
    viewportOverflow: Math.max(0, rect.right - document.documentElement.clientWidth, -rect.left),
  });
})()`);

const apparatusAudit = () => page.evaluate(`(() => {
  const panel = document.querySelector('section.apparatus');
  if (!panel) return JSON.stringify({ present: false });
  const rect = panel.getBoundingClientRect();
  return JSON.stringify({
    present: true,
    strip: Boolean(panel.querySelector('.strip')),
    info: Boolean(panel.querySelector('button.info-toggle')),
    close: Boolean(panel.querySelector('button.icon-close')),
    run: Boolean(panel.querySelector('button.run')),
    viewportOverflow: Math.max(0, rect.right - document.documentElement.clientWidth, -rect.left),
    below: Math.max(0, rect.bottom - document.documentElement.clientHeight),
    height: rect.height,
  });
})()`);

const layoutAudit = () => page.evaluate(`(() => {
  const visible = (element) => {
    const style = getComputedStyle(element);
    const rect = element.getBoundingClientRect();
    return style.display !== "none" && style.visibility !== "hidden" && rect.width > 0 && rect.height > 0;
  };
  const rect = (selector) => {
    const element = document.querySelector(selector);
    return element && visible(element) ? element.getBoundingClientRect().toJSON() : null;
  };
  const unnamed = [...document.querySelectorAll('button')].filter((button) =>
    visible(button) && !(button.getAttribute('aria-label') || button.getAttribute('title') || button.textContent.trim()));
  const duplicateIds = [...document.querySelectorAll('[id]')]
    .map((element) => element.id).filter((id, index, ids) => ids.indexOf(id) !== index);
  return JSON.stringify({
    bodyOverflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
    cabinet: rect('nav.shelf-pane'), bench: rect('.bench-pane'), journal: rect('main > aside'),
    unnamed: unnamed.length, duplicateIds: [...new Set(duplicateIds)],
  });
})()`);

const mobileTabs = () => page.evaluate(`JSON.stringify(
  [...document.querySelectorAll('.tabs button')].filter((button) => button.offsetParent)
    .map((button) => ({
      name: button.textContent.trim(),
      width: button.getBoundingClientRect().width,
      height: button.getBoundingClientRect().height,
    }))
)`);

const chooseMobilePane = async (index) => {
  await page.evaluate(`(() => {
    const buttons = [...document.querySelectorAll('.tabs button')].filter((button) => button.offsetParent);
    buttons[${index}]?.click();
  })()`);
  await new Promise((resolve) => setTimeout(resolve, 50));
  return JSON.parse(await layoutAudit());
};

const openPeriodicTable = async () => {
  const hasButton = await page.evaluate(`Boolean([...document.querySelectorAll('button.tool')].find((item) =>
    /elements|elemente/i.test(item.textContent || "")))`);
  if (!hasButton) {
    await page.evaluate(`document.querySelector('button.utility-toggle')?.click()`);
    await waitFor(page, `document.querySelector('.utility-drawer')`, { timeout: 5000 });
  }
  await page.evaluate(`(() => {
    const button = [...document.querySelectorAll('button.tool')].find((item) =>
      /elements|elemente/i.test(item.textContent || ""));
    button?.click();
  })()`);
  return waitFor(page, `document.querySelector('dialog.table-panel')`, { timeout: 5000 });
};

const clickButtonContaining = async (text) => page.evaluate(`(() => {
  const needle = ${JSON.stringify(text)}.toLocaleLowerCase();
  const button = [...document.querySelectorAll('button')].find((item) =>
    item.offsetParent && (item.textContent || "").toLocaleLowerCase().includes(needle));
  button?.click();
  return Boolean(button);
})()`);

/** One real-browser journey over persisted learning state. The scoped legacy
 * keys are a supported bootstrap input; using them before app boot exercises
 * migration and keeps test-only switches out of the product. */
const learningProgressJourney = async () => {
  await page.goto(`${origin}/privacy.html`);
  await page.evaluate(`(() => {
    localStorage.clear();
    localStorage.setItem("kerotakis.mode.v1", "story");
    localStorage.setItem("kero.mode.story.kero.missions.done.v1", JSON.stringify(["kitchen-hot-and-cold-packs"]));
    localStorage.setItem("kero.mode.story.kero.codex.done.v1", JSON.stringify(["hot-pack", "cold-pack"]));
  })()`);
  await page.goto(`${origin}/app/`);
  // The world map is a destination now, not a doorway: the brand mark is
  // the route a reader takes to it, so the journey takes the same one.
  await waitFor(page, `document.querySelector('button.brand')`, { timeout: 60000 });
  await page.evaluate(`document.querySelector('button.brand')?.click()`);
  await waitFor(page, `document.querySelector('.story-destination .destination-meta')?.textContent.includes('missions') && !document.querySelector('.story-destination .destination-meta')?.textContent.includes('arriving')`, { timeout: 60000 });

  check("the Mission Board opens with seeded progress", await clickButtonContaining("Mission Board")
    && await waitFor(page, `document.querySelector('dialog.story-map')`, { timeout: 5000 }));
  const story = JSON.parse(await page.evaluate(`(() => {
    const dialog = document.querySelector('dialog.story-map');
    const next = dialog?.querySelector('button.next-investigation strong')?.textContent?.trim() || "";
    const selected = dialog?.querySelector('button.district[aria-pressed="true"]');
    return JSON.stringify({ next, selected: selected?.textContent?.trim() || "" });
  })()`));
  check("Story exposes its Next investigation and selected district", Boolean(story.next && story.selected), `${story.selected}: ${story.next}`);
  check("the Experiment Library opens from the Mission Board", await page.evaluate(`(() => {
    const button = [...document.querySelectorAll('dialog.story-map footer button')]
      .find((item) => /experiment library/i.test(item.textContent || ""));
    button?.click(); return Boolean(button);
  })()`)
    && await waitFor(page, `document.querySelector('dialog.panel .progress-filters')`, { timeout: 5000 }));
  await waitFor(page, `document.querySelectorAll('dialog.panel article .completion').length > 0`, { timeout: 5000 });
  const experimentInitial = JSON.parse(await page.evaluate(`(() => {
    const group = document.querySelector('dialog.panel .progress-filters');
    return JSON.stringify({ name: group?.getAttribute('aria-label'), selected: group?.querySelectorAll('button[aria-pressed="true"]').length });
  })()`));
  check("Experiment completion filters expose one accessible selected state", Boolean(experimentInitial.name) && experimentInitial.selected === 1);
  await page.evaluate(`document.querySelector('dialog.panel .progress-filters button:last-child')?.click()`);
  // Wait for the list to have actually narrowed, not merely to be long
  // enough: with one surface the unfiltered list is every entry, so a
  // count alone is satisfied before the filter has been applied at all.
  await waitFor(page, `(() => {
    const rows = [...document.querySelectorAll('dialog.panel article .completion')];
    return rows.length >= 2 && rows.every((row) => /completed/i.test(row.textContent || ""));
  })()`, { timeout: 5000 });
  const experiments = JSON.parse(await page.evaluate(`(() => {
    const group = document.querySelector('dialog.panel .progress-filters');
    const rows = [...document.querySelectorAll('dialog.panel article .completion')].filter((item) => item.offsetParent);
    return JSON.stringify({ pressed: group?.querySelector('button[aria-pressed="true"]')?.textContent?.trim(), rows: rows.length, allComplete: rows.every((row) => /completed/i.test(row.textContent || "")) });
  })()`));
  check("Experiment completed filter shows only completed rows", /completed/i.test(experiments.pressed || "") && experiments.rows >= 2 && experiments.allComplete, `${experiments.rows} rows`);
  await page.evaluate(`document.querySelector('dialog.panel button.icon-close')?.click()`);
  await waitFor(page, `!document.querySelector('dialog.panel')`, { timeout: 5000 });

  await page.evaluate(`document.querySelector('button.brand')?.click()`);
  await waitFor(page, `document.querySelector('button.kids-node')`, { timeout: 5000 });
  await waitFor(page, `!document.querySelector('button.kids-node small')?.textContent.includes('syncing')`, { timeout: 60000 });
  // The second home-screen door opens the SAME catalogue pre-filtered to
  // the first level, so the walk widens it back to everything before
  // looking for a card that sits at another level. One surface, one list.
  check("the catalogue opens from the second home door", await page.evaluate(`(() => { const button = document.querySelector('button.kids-node'); button?.click(); return Boolean(button); })()`)
    && await waitFor(page, `document.querySelector('dialog #catalog-title')`, { timeout: 5000 }));
  await page.evaluate(`document.querySelector('dialog .chips.levels button')?.click()`);
  await waitFor(page, `[...document.querySelectorAll('dialog article h2')].some((item) => /Hot pack and cold pack/i.test(item.textContent || ""))`, { timeout: 5000 });
  const kids = JSON.parse(await page.evaluate(`(() => {
    const cards = [...document.querySelectorAll('dialog article')];
    const card = cards.find((item) => /Hot pack and cold pack/i.test(item.querySelector('h2')?.textContent || ""));
    const chips = document.querySelector('dialog .chips.levels');
    return JSON.stringify({
      progress: card?.querySelector('.learning-progress')?.getAttribute('data-progress'),
      count: card?.querySelector('.learning-progress strong')?.textContent?.trim(),
      replayLesson: /replay guided lesson/i.test(card?.textContent || ""),
      replayCodex: (card?.textContent?.match(/replay Codex investigation/gi) || []).length,
      selected: chips?.querySelectorAll('button[aria-pressed="true"]').length,
    });
  })()`));
  check("the card reports all linked learning and Replay actions", kids.progress === "all" && kids.count === "3/3" && kids.replayLesson && kids.replayCodex === 2, `${kids.progress} ${kids.count}`);
  check("level chips expose one accessible selected state", kids.selected === 1, `${kids.selected} selected`);
  await page.evaluate(`document.querySelector('dialog header button[aria-label="close"]')?.click()`);
};

const periodicAudit = () => page.evaluate(`(() => {
  const panel = document.querySelector('dialog.table-panel');
  const options = [...(panel?.querySelectorAll('[role="option"]') || [])];
  const symbols = options.map((option) => option.querySelector('.sym')?.textContent?.trim());
  return JSON.stringify({
    options: options.length,
    symbols,
    unnamed: options.filter((option) => !option.getAttribute('aria-label')).length,
    panelOverflow: panel ? panel.scrollWidth - panel.clientWidth : null,
    viewportOverflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
    animations: panel?.getAnimations({ subtree: true }).filter((animation) => animation.playState === 'running').length ?? 0,
  });
})()`);

try {
  await viewport(1440, 900);
  await learningProgressJourney();
  // The journey above left this browser in Story; the rest of the audit is
  // the unlocked bench, chosen the way the shell itself remembers it.
  await page.evaluate(`localStorage.setItem("kerotakis.mode.v1", "sandbox")`);
  await page.goto(`${origin}/app/`);
  check("the desktop bench opens", await openBench());
  const entry = JSON.parse(await page.evaluate(`JSON.stringify({
    chooser: Boolean(document.querySelector('dialog.world')),
    console: Boolean(document.querySelector('form.bar')),
    mapRoute: Boolean(document.querySelector('button.brand')),
  })`));
  check("the deployment lands on the bench, not on a chooser", entry.chooser === false);
  check("the command line is off until it is asked for", entry.console === false);
  check("the world map stays one press away", entry.mapRoute === true);
  const header = JSON.parse(await page.evaluate(`(() => {
    const dial = document.querySelector('.dial select');
    const locale = document.querySelector('.locale select');
    const selected = (element) => element?.options[element.selectedIndex]?.textContent?.trim() ?? "";
    return JSON.stringify({
      levels: dial?.options.length ?? 0,
      dialButtons: document.querySelectorAll('.dial button').length,
      dialWidth: dial?.getBoundingClientRect().width ?? 0,
      dialShows: selected(dial),
      localeShows: selected(locale),
      localeName: locale?.getAttribute('aria-label') ?? "",
    });
  })()`));
  check("the register is one compact control holding all three levels",
    header.levels === 3 && header.dialButtons === 0 && header.dialWidth <= 180,
    `${header.levels} levels, ${Math.round(header.dialWidth)}px`);
  check("the register names only the level it is on", /^lv[123]/.test(header.dialShows), header.dialShows);
  check("the language switcher shows a code and still names the language",
    /^[A-Z]{2}$/.test(header.localeShows) && /English|Deutsch/.test(header.localeName),
    `${header.localeShows} — ${header.localeName}`);
  await waitFor(page, `document.querySelector('.observation-status')?.textContent?.trim().length > 0`, { timeout: 60000 });
  const observationStatus = JSON.parse(await page.evaluate(`(() => {
    const status = document.querySelector('.observation-status');
    return JSON.stringify({ live: status?.getAttribute('aria-live'), atomic: status?.getAttribute('aria-atomic'), words: status?.textContent?.trim().length || 0 });
  })()`));
  check("vessel observations have one polite atomic live status", observationStatus.live === "polite" && observationStatus.atomic === "true" && observationStatus.words > 0);
  const desktop = JSON.parse(await layoutAudit());
  check("desktop has no page-level horizontal overflow", desktop.bodyOverflow <= 1, `${desktop.bodyOverflow}px`);
  check("desktop cabinet, bench, and journal are present", Boolean(desktop.cabinet && desktop.bench && desktop.journal));
  if (desktop.cabinet && desktop.bench && desktop.journal) {
    check("desktop panels do not overlap", desktop.cabinet.right <= desktop.bench.left + 1 && desktop.bench.right <= desktop.journal.left + 1);
  }
  check("visible buttons have an accessible name", desktop.unnamed === 0, `${desktop.unnamed} unnamed`);
  check("the rendered page has no duplicate ids", desktop.duplicateIds.length === 0, desktop.duplicateIds.join(", "));

  // A collapsed side panel used to keep a 44 px column open to say nothing.
  // Collapsed is a rail at the screen edge; the stage takes the rest, and
  // the panel comes back over it on focus and pins on press.
  await page.evaluate(`(() => {
    document.querySelector('nav.shelf-pane .panel-collapse')?.click();
    document.querySelector('main > aside .panel-collapse')?.click();
  })()`);
  await new Promise((resolve) => setTimeout(resolve, 300));
  const collapsed = JSON.parse(await layoutAudit());
  check("a collapsed panel is a slim rail, not a column",
    Boolean(collapsed.cabinet && collapsed.journal)
      && collapsed.cabinet.width <= 24 && collapsed.journal.width <= 24,
    `${Math.round(collapsed.cabinet?.width ?? -1)}px / ${Math.round(collapsed.journal?.width ?? -1)}px`);
  check("the bench stage takes the freed width",
    collapsed.bench.width > desktop.bench.width + 150,
    `${Math.round(desktop.bench.width)}px → ${Math.round(collapsed.bench.width)}px`);
  check("collapsing keeps every control named and the page unscrolled",
    collapsed.unnamed === 0 && collapsed.bodyOverflow <= 1, `${collapsed.unnamed} unnamed`);
  const revealed = JSON.parse(await page.evaluate(`(() => {
    const rail = document.querySelector('nav.shelf-pane button.pane-rail');
    rail?.focus();
    const body = document.querySelector('nav.shelf-pane .pane-body');
    return JSON.stringify({
      rail: Boolean(rail),
      named: Boolean(rail?.getAttribute('aria-label')),
      focused: document.activeElement === rail,
      shown: body ? getComputedStyle(body).visibility === "visible" : false,
      width: body?.getBoundingClientRect().width ?? 0,
    });
  })()`));
  check("the rail is a named, focusable control", revealed.rail && revealed.named && revealed.focused);
  check("focusing the rail reveals the panel over the stage",
    revealed.shown && revealed.width > 120, `${Math.round(revealed.width)}px`);
  await page.evaluate(`(() => {
    document.querySelector('nav.shelf-pane button.pane-rail')?.click();
    document.querySelector('main > aside button.pane-rail')?.click();
  })()`);
  await new Promise((resolve) => setTimeout(resolve, 300));
  const pinned = JSON.parse(await layoutAudit());
  check("pressing the rail pins the panel back into the layout",
    pinned.cabinet.width > 100 && pinned.journal.width > 100,
    `${Math.round(pinned.cabinet.width)}px / ${Math.round(pinned.journal.width)}px`);

  check("the periodic table opens from the bench", await openPeriodicTable());
  const labTable = JSON.parse(await periodicAudit());
  check("the default table keeps Fe, Cu, and Zn", ["Fe", "Cu", "Zn"].every((symbol) => labTable.symbols.includes(symbol)));
  check("the default table omits hazardous and synthetic identities",
    ["Po", "At", "Fr", "Ra", "Og"].every((symbol) => !labTable.symbols.includes(symbol)));
  check("every element cell has an accessible name", labTable.unnamed === 0, `${labTable.unnamed} unnamed`);
  await page.evaluate(`document.querySelector('dialog.table-panel button.mode')?.click()`);
  const fullTable = JSON.parse(await periodicAudit());
  check("the explicit full-table mode exposes all 118 identities", fullTable.options === 118, `${fullTable.options} cells`);
  await page.evaluate(`document.querySelector('dialog.table-panel button.icon-close')?.click()`);

  check("the equipment cupboard opens from the bench", await openCupboard());
  const cupboard = JSON.parse(await cupboardAudit());
  // Five shelves since GUI-103: measure, heat & cool, prepare & convert,
  // contain & connect, separate. `drive` was folded into its neighbours and
  // the kits stopped being a shelf. A missing one means an entry lost its
  // group; a sixth means one was added without a decision.
  check("the cupboard groups its equipment on five shelves", cupboard.shelves === 5, `${cupboard.shelves} shelves`);
  check("every shelf says what lives on it", cupboard.unexplainedShelves === 0, `${cupboard.unexplainedShelves} unexplained`);
  // 12 instruments + 12 apparatus + 6 transfer verbs + burette, mixer,
  // column train and the reaction studio. The reaction studio is counted
  // whether or not this session offers one, so the denominator cannot move
  // mid-session; the kits are names for these and are not counted twice.
  check("the cupboard knows the whole catalogue", cupboard.catalogue === 34, `${cupboard.catalogue} tools`);
  check("the cupboard shows what this learner has", cupboard.items >= 12, `${cupboard.items} items`);
  check("every cupboard item is named", cupboard.unnamed === 0, `${cupboard.unnamed} unnamed`);
  check("every cupboard item can say what it models", cupboard.info === cupboard.items, `${cupboard.info} of ${cupboard.items}`);
  // The tally is printed only while it carries information. In Sandbox
  // nothing is locked, so a fraction there would read "34/34".
  check("the cupboard shows its tally only while something is locked",
        (cupboard.locked > 0) === (cupboard.tally === 1),
        `${cupboard.locked} locked, tally ${cupboard.tally}`);
  check("the cupboard opens under laboratory names", cupboard.kitNames === 0, `${cupboard.kitNames} kit names`);
  const setsOn = await toggleSets();
  // The chip renames slots; it never adds or removes one. As a shelf, the
  // kits put the candle on the wall twice.
  check("the kits chip renames a tool rather than adding a second one",
        setsOn.items === cupboard.items && setsOn.kitNames === 1,
        `${setsOn.items} items, ${setsOn.kitNames} kit names`);
  check("the kits chip is not a mode: the shelves keep their tools", setsOn.shelves === cupboard.shelves,
        `${setsOn.shelves} shelves`);
  const setsOff = await toggleSets();
  check("the kits chip turns back off", setsOff.kitNames === 0 && setsOff.items === cupboard.items,
        `${setsOff.items} items, ${setsOff.kitNames} kit names`);
  await page.evaluate(`document.querySelector('dialog.cupboard button.icon-close')?.click()`);


  const dockTargets = JSON.parse(await page.evaluate(`JSON.stringify(
    [...document.querySelectorAll('.actions button')].filter((button) => button.offsetParent)
      .map((button) => ({ name: button.textContent.trim(), width: button.getBoundingClientRect().width, height: button.getBoundingClientRect().height }))
  )`));
  const smallDockTargets = dockTargets.filter((target) => target.width < 48 || target.height < 48);
  check("primary vessel actions have 48 px targets", smallDockTargets.length === 0,
    smallDockTargets.map((target) => `${target.name}:${target.width}×${target.height}`).join(", "));

  // Opened last of the desktop checks: the inspector's own gas-test row is
  // also `.actions`, and mounting it would put four small buttons into the
  // dock's 48 px measurement above.
  check("the MESSEN strip opens with the measurement tools", await openStrip());
  const strip = JSON.parse(await stripAudit());
  check("the quick strip holds four instruments and the cupboard door",
        strip.instruments > 0 && strip.instruments <= 4 && strip.doors === 1,
        `${strip.instruments} instruments, ${strip.doors} doors`);
  check("the quick strip never repeats the dock's three readings", strip.repeated === 0,
        `${strip.repeated} repeated`);

  await page.evaluate(`localStorage.setItem("kerotakis.locale", "de")`);
  await page.goto(`${origin}/app/`);
  await openBench();
  const german = JSON.parse(await layoutAudit());
  check("German desktop copy does not widen the page", german.bodyOverflow <= 1, `${german.bodyOverflow}px`);

  await viewport(390, 844);
  await page.goto(`${origin}/app/`);
  await openBench();
  const mobile = JSON.parse(await layoutAudit());
  check("phone layout has no page-level horizontal overflow", mobile.bodyOverflow <= 1, `${mobile.bodyOverflow}px`);
  const tabs = JSON.parse(await mobileTabs());
  check("phone navigation exposes three tabs", tabs.length === 3, `${tabs.length} tabs`);
  check("phone tabs meet the 44 px touch minimum", tabs.every((tab) => tab.width >= 44 && tab.height >= 44));
  check("the periodic table opens on a phone", await openPeriodicTable());
  const phoneTable = JSON.parse(await periodicAudit());
  check("the phone periodic table stays inside the viewport", phoneTable.viewportOverflow <= 1, `${phoneTable.viewportOverflow}px`);
  await page.evaluate(`document.querySelector('dialog.table-panel button.icon-close')?.click()`);
  check("the equipment cupboard opens on a phone", await openCupboard());
  const phoneCupboard = JSON.parse(await cupboardAudit());
  check("the phone cupboard stays inside the viewport", phoneCupboard.viewportOverflow <= 1, `${phoneCupboard.viewportOverflow}px`);
  await page.evaluate(`document.querySelector('dialog.cupboard button.icon-close')?.click()`);

  // The workstation, on a phone. It is deployed from the vessel dock rather
  // than the cupboard because the dock is the surface a learner reaches it
  // from, and `data-action` is a wire key, not a translated label.
  if (await openApparatus("heat")) {
    const strip = JSON.parse(await apparatusAudit());
    check("the workstation strip opens over the stage on a phone", strip.present === true);
    check("the workstation strip stays inside the viewport at 390 px",
          strip.viewportOverflow <= 1 && strip.below <= 1,
          `${strip.viewportOverflow}px right, ${strip.below}px below`);
    // One row of chrome, and the sentence behind the (i). A strip taller
    // than half the screen is the three-column panel again.
    check("the workstation strip leaves the stage visible", strip.height <= 844 * 0.62, `${Math.round(strip.height)}px`);
    check("the workstation strip carries its name, its (i), its run and its close",
          strip.strip && strip.info && strip.run && strip.close);
    await page.evaluate(`document.querySelector('section.apparatus button.icon-close')?.click()`);
  } else {
    check("the workstation strip opens over the stage on a phone", false, "no section.apparatus");
  }

  // The pour chooser, on the same phone. `.pour` sets the source to the
  // selected vessel, so the chooser opens anchored rather than waiting.
  await page.evaluate(`document.querySelector('.actions button.pour')?.click()`);
  if (await waitFor(page, `document.querySelector('.pour-overlay')`, { timeout: 20000 })) {
    const pour = JSON.parse(await pourAudit());
    check("the pour chooser opens on the stage, not above it", pour.onStage === true && pour.banner === false);
    check("the pour chooser offers the four fractions", pour.fractions === 4, `${pour.fractions} chips`);
    await page.evaluate(`document.querySelector('.pour-overlay .cancel')?.click()`);
  } else {
    check("the pour chooser opens on the stage, not above it", false, "no .pour-overlay");
  }

  // 320 CSS pixels remains a real supported width: compact phones, split
  // views and a 640px browser at 200% zoom all reach it. Audit every pane,
  // because the inactive drawers are deliberately absent from layout.
  await viewport(320, 700);
  await page.goto(`${origin}/app/`);
  await openBench();
  const narrowTabs = JSON.parse(await mobileTabs());
  check("320 px navigation exposes all three destinations", narrowTabs.length === 3,
    narrowTabs.map((tab) => tab.name).join(", "));
  check("320 px tabs retain 44 px touch targets", narrowTabs.every((tab) => tab.width >= 44 && tab.height >= 44));
  const narrowBench = await chooseMobilePane(0);
  const narrowShelf = await chooseMobilePane(1);
  // The shelf is where the longest words in the product live:
  // "Wasserstoffperoxid" is wider than a 320px phone, and a name that
  // cannot break forces the pane wider than the page. Measured per name
  // rather than through the page's own overflow, because a shelf row that
  // clips its own text passes a page-level check while still hiding the
  // one word the reader needs.
  const shelfNamed = await waitFor(page,
    `[...document.querySelectorAll('nav.shelf-pane .name')].some((item) => item.offsetParent)`,
    { timeout: 60000 });
  check("320 px cabinet lists its substances", shelfNamed === true);
  const overflowingNames = JSON.parse(await page.evaluate(`JSON.stringify(
    [...document.querySelectorAll('nav.shelf-pane .name')].filter((item) => item.offsetParent)
      .filter((item) => item.scrollWidth - item.clientWidth > 1)
      .map((item) => item.textContent.trim()).slice(0, 6)
  )`));
  check("320 px substance names wrap instead of overflowing their row",
        overflowingNames.length === 0, overflowingNames.join(", "));
  // The cabinet's two filter groups share one rail. Two stacked rows of
  // chrome above the bottles is what this replaced, so the assertion is
  // about rows: the rail scrolls sideways rather than growing downwards,
  // and the chips stay big enough to hit at the width that forces it.
  const rail = JSON.parse(await page.evaluate(`(() => {
    const rail = document.querySelector('nav.shelf-pane .cabinet-rail');
    const chips = [...(rail?.querySelectorAll('button') ?? [])].filter((chip) => chip.offsetParent);
    const rows = new Set(chips.map((chip) => Math.round(chip.getBoundingClientRect().top)));
    return JSON.stringify({
      present: Boolean(rail),
      chips: chips.length,
      rows: rows.size,
      touchable: chips.every((chip) => chip.getBoundingClientRect().height >= 44),
      scrolls: rail ? getComputedStyle(rail).overflowX !== "visible" : false,
    });
  })()`));
  check("320 px cabinet filters share one row", rail.present && rail.chips >= 2 && rail.rows === 1,
        `${rail.chips} chips on ${rail.rows} row(s)`);
  check("320 px cabinet filters scroll rather than wrap", rail.scrolls);
  check("320 px cabinet filter chips keep 44 px touch targets", rail.touchable);
  const narrowJournal = await chooseMobilePane(2);
  check("320 px workspace stays inside the page", narrowBench.bodyOverflow <= 1 && Boolean(narrowBench.bench), `${narrowBench.bodyOverflow}px`);
  check("320 px cabinet stays inside the page", narrowShelf.bodyOverflow <= 1 && Boolean(narrowShelf.cabinet), `${narrowShelf.bodyOverflow}px`);
  check("320 px journal stays inside the page", narrowJournal.bodyOverflow <= 1 && Boolean(narrowJournal.journal), `${narrowJournal.bodyOverflow}px`);

  // Text-only zoom is more demanding than page zoom: the viewport does not
  // shrink, but inherited type and rem-sized controls double. This catches
  // rigid chrome that a narrow-viewport test alone cannot see.
  await viewport(1440, 900);
  // The console is opt-in, so the sections below that drive it turn it on
  // the way a reader does — from the utilities menu, remembered on reload.
  await page.evaluate(`localStorage.setItem("kerotakis.console.v1", "shown")`);
  await page.goto(`${origin}/app/`);
  await openBench();
  check("the command line comes back when it is asked for",
    Boolean(await page.evaluate(`Boolean(document.querySelector('form.bar input'))`)));
  await page.evaluate(`(() => {
    const style = document.createElement("style");
    style.id = "ux-text-zoom";
    style.textContent = "html { font-size: 200% !important; } body { font-size: 200% !important; }";
    document.head.append(style);
  })()`);
  const zoomed = JSON.parse(await layoutAudit());
  check("200% text zoom has no page-level horizontal overflow", zoomed.bodyOverflow <= 1, `${zoomed.bodyOverflow}px`);
  check("200% text zoom keeps the three surfaces separate", Boolean(zoomed.cabinet && zoomed.bench && zoomed.journal)
    && zoomed.cabinet.right <= zoomed.bench.left + 1 && zoomed.bench.right <= zoomed.journal.left + 1);
  check("200% text zoom keeps controls named", zoomed.unnamed === 0, `${zoomed.unnamed} unnamed`);

  await page.cdp.send("Emulation.setEmulatedMedia", {
    media: "screen", features: [{ name: "prefers-reduced-motion", value: "reduce" }],
  }, page.sessionId);
  check("the periodic table opens with reduced motion", await openPeriodicTable());
  const reducedTable = JSON.parse(await periodicAudit());
  check("reduced motion leaves no running periodic-table animation", reducedTable.animations === 0, `${reducedTable.animations} animations`);
  await page.evaluate(`document.querySelector('dialog.table-panel button.icon-close')?.click()`);
  const inputReady = await waitFor(page, `!document.querySelector('form.bar input')?.disabled`, { timeout: 60000 });
  if (inputReady) {
    await page.evaluate(`(() => {
      const input = document.querySelector('form.bar input');
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value").set.call(input, "stir v1 500rpm 10s");
      input.dispatchEvent(new Event("input", { bubbles: true }));
      input.form.dispatchEvent(new SubmitEvent("submit", { bubbles: true, cancelable: true }));
    })()`);
    await waitFor(page, `!document.querySelector('form.bar input')?.disabled`, { timeout: 60000 });
    const moving = await page.evaluate(`document.querySelector('.bench')?.getAnimations({ subtree: true }).filter((animation) => animation.playState === "running").length ?? 0`);
    check("reduced motion leaves no running bench animation", moving === 0, `${moving} animations`);
  } else {
    check("reduced-motion bench accepts a command", false);
  }
} catch (error) {
  console.error(`UX quality: ${error.stack ?? error.message}`);
  failures++;
} finally {
  await page.close?.();
  server.close();
}

console.log(failures ? `\n${failures} UX quality check(s) failed` : "\nUX quality gates passed");
process.exit(failures ? 1 : 0);
