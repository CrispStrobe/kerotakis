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

const openSandbox = async () => {
  await waitFor(page, `document.querySelector('form.bar input')`, { timeout: 60000 });
  await page.evaluate(`(() => {
    const button = [...document.querySelectorAll('button')].find((item) =>
      /enter Sandbox|Sandbox betreten/i.test(item.textContent || ""));
    button?.click();
  })()`);
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
  if (!panel) return JSON.stringify({ catalogue: 0, shelves: 0, items: 0, unnamed: 1, info: 0, viewportOverflow: 0 });
  const items = [...panel.querySelectorAll('button.item')];
  const rect = panel.getBoundingClientRect();
  // The header tally is "reachable/whole catalogue". The denominator is what
  // the cupboard KNOWS about, which is progression-independent; the number of
  // rendered items is not, because a Story learner is shown what they have
  // earned. Asserting the rendered count would have been asserting the
  // fixture's progress.
  //
  // Split rather than match: this whole function is a template literal, so a
  // regex written here loses its backslashes on the way to the browser — an
  // escaped slash arrives as a bare one and closes the literal early.
  const tally = (panel.querySelector('header b')?.textContent || "").split("/");
  return JSON.stringify({
    catalogue: tally.length === 2 ? Number(tally[1].trim()) : 0,
    shelves: panel.querySelectorAll('section.shelf').length,
    items: items.length,
    unnamed: items.filter((item) => !(item.textContent.trim() || item.getAttribute('aria-label'))).length,
    info: panel.querySelectorAll('button.info-toggle').length,
    viewportOverflow: Math.max(0, rect.right - document.documentElement.clientWidth, -rect.left),
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
  await page.goto(`${origin}/app/`);
  check("the desktop bench opens", await openSandbox());
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
  // Six shelves: measure, heat & cool, contain & connect, separate, drive,
  // and the kits. A missing shelf means an entry lost its group.
  check("the cupboard groups its equipment on shelves", cupboard.shelves >= 5, `${cupboard.shelves} shelves`);
  // 12 instruments + 12 apparatus + 6 transfer verbs + burette/mixer/column
  // train, with the reaction studio conditional on the session. The kits are
  // skins over those and are not counted twice.
  check("the cupboard knows the whole catalogue", cupboard.catalogue >= 33, `${cupboard.catalogue} tools`);
  check("the cupboard shows what this learner has", cupboard.items >= 12, `${cupboard.items} items`);
  check("every cupboard item is named", cupboard.unnamed === 0, `${cupboard.unnamed} unnamed`);
  check("every cupboard item can say what it models", cupboard.info === cupboard.items, `${cupboard.info} of ${cupboard.items}`);
  await page.evaluate(`document.querySelector('dialog.cupboard button.icon-close')?.click()`);

  const dockTargets = JSON.parse(await page.evaluate(`JSON.stringify(
    [...document.querySelectorAll('.actions button')].filter((button) => button.offsetParent)
      .map((button) => ({ name: button.textContent.trim(), width: button.getBoundingClientRect().width, height: button.getBoundingClientRect().height }))
  )`));
  const smallDockTargets = dockTargets.filter((target) => target.width < 48 || target.height < 48);
  check("primary vessel actions have 48 px targets", smallDockTargets.length === 0,
    smallDockTargets.map((target) => `${target.name}:${target.width}×${target.height}`).join(", "));

  await page.evaluate(`localStorage.setItem("kerotakis.locale", "de")`);
  await page.goto(`${origin}/app/`);
  await openSandbox();
  const german = JSON.parse(await layoutAudit());
  check("German desktop copy does not widen the page", german.bodyOverflow <= 1, `${german.bodyOverflow}px`);

  await viewport(390, 844);
  await page.goto(`${origin}/app/`);
  await openSandbox();
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

  // 320 CSS pixels remains a real supported width: compact phones, split
  // views and a 640px browser at 200% zoom all reach it. Audit every pane,
  // because the inactive drawers are deliberately absent from layout.
  await viewport(320, 700);
  await page.goto(`${origin}/app/`);
  await openSandbox();
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
  const narrowJournal = await chooseMobilePane(2);
  check("320 px workspace stays inside the page", narrowBench.bodyOverflow <= 1 && Boolean(narrowBench.bench), `${narrowBench.bodyOverflow}px`);
  check("320 px cabinet stays inside the page", narrowShelf.bodyOverflow <= 1 && Boolean(narrowShelf.cabinet), `${narrowShelf.bodyOverflow}px`);
  check("320 px journal stays inside the page", narrowJournal.bodyOverflow <= 1 && Boolean(narrowJournal.journal), `${narrowJournal.bodyOverflow}px`);

  // Text-only zoom is more demanding than page zoom: the viewport does not
  // shrink, but inherited type and rem-sized controls double. This catches
  // rigid chrome that a narrow-viewport test alone cannot see.
  await viewport(1440, 900);
  await page.goto(`${origin}/app/`);
  await openSandbox();
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
