/** Browser-level UX invariants: layout, accessible controls, touch size and reduced motion. */
import { serve, browser, waitFor } from "./lib/headless.mjs";
import { readFile } from "node:fs/promises";
import { join } from "node:path";

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
  return waitFor(page, `document.querySelector('.instrument-tray button')`, { timeout: 20000 });
};

const stripAudit = () => page.evaluate(`(() => {
  const tray = document.querySelector('.instrument-tray');
  if (!tray) return JSON.stringify({ instruments: 0, doors: 0, repeated: 1 });
  const buttons = [...tray.querySelectorAll('button')];
  const tokens = buttons.map((item) => item.getAttribute('data-token')).filter(Boolean);
  return JSON.stringify({
    instruments: tokens.length,
    doors: buttons.filter((item) => item.classList.contains('cupboard-door')).length,
    repeated: tokens.filter((token) => ['eyes', 'thermometer', 'ph'].includes(token)).length,
  });
})()`);

/** Two frames and a tick: enough for a reflow and the shell's own
 * re-render to land before anything is measured. */
const settle = () => new Promise((resolve) => setTimeout(resolve, 300));

/** GUI-475, owner: "'Alle Geräte' overlays on top of the Messen pieces".
 * The door was `position: sticky; right: 0` in a scrolling flex row, which
 * pins it to the scrollport and lets the instrument pills travel beneath
 * it. Boxes rather than styles: whatever the CSS says, two buttons in one
 * row must not share pixels. */
const trayAudit = () => page.evaluate(`(() => {
  const tray = document.querySelector('.instrument-tray');
  if (!tray) return JSON.stringify({ present: false, buttons: 0, overlaps: [], small: [] });
  const named = (button) => (button.getAttribute('aria-label') || button.getAttribute('title') || button.textContent || '').trim();
  const boxes = [...tray.querySelectorAll('button')]
    .filter((button) => button.offsetParent)
    .map((button) => ({ name: named(button), box: button.getBoundingClientRect() }));
  const overlaps = [];
  for (let i = 0; i < boxes.length; i++) {
    for (let j = i + 1; j < boxes.length; j++) {
      const a = boxes[i].box;
      const b = boxes[j].box;
      const shared = Math.min(a.right, b.right) - Math.max(a.left, b.left);
      const stacked = Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top);
      if (shared > 1 && stacked > 1) overlaps.push(boxes[i].name + ' over ' + boxes[j].name);
    }
  }
  return JSON.stringify({
    present: true,
    buttons: boxes.length,
    overlaps,
    small: boxes.filter((item) => item.box.width < 43.5 || item.box.height < 43.5).map((item) => item.name),
  });
})()`);

/** GUI-053 again, owner: "there is not enough screen space for the Missions
 * below the map". The concept map is two surfaces — the graph and the
 * activities affiliated with the selected concept — and the second one is
 * what a reader opened the panel for. It is reached from the utilities
 * drawer; "Karte" is it, "Weltkarte" is the world map, so the label is
 * matched whole rather than by substring. */
const openConceptMap = async () => {
  const wanted = ["map", "karte"];
  const present = () => page.evaluate(`Boolean([...document.querySelectorAll('button.tool')].find((item) =>
    ${JSON.stringify(wanted)}.includes((item.textContent || "").trim().toLocaleLowerCase())))`);
  if (!(await present())) {
    await page.evaluate(`document.querySelector('button.utility-toggle')?.click()`);
    await waitFor(page, `document.querySelector('.utility-drawer')`, { timeout: 5000 });
  }
  // The codex export is fetched independently of the bench, and the button
  // is deliberately absent until it lands. Clicking before then races the
  // filesystem rather than testing a layout.
  await waitFor(page, `[...document.querySelectorAll('button.tool')].some((item) =>
    ${JSON.stringify(wanted)}.includes((item.textContent || "").trim().toLocaleLowerCase()))`,
    { timeout: 30000 });
  await page.evaluate(`(() => {
    const button = [...document.querySelectorAll('button.tool')].find((item) =>
      ${JSON.stringify(wanted)}.includes((item.textContent || "").trim().toLocaleLowerCase()));
    button?.click();
  })()`);
  return waitFor(page, `document.querySelector('dialog.map')`, { timeout: 20000 });
};

/** Choose the concept with the most entries behind it — the one whose
 * activity list is long enough for "below the fold" to mean anything. */
const pickBusiestConcept = async () => {
  const chosen = await page.evaluate(`(() => {
    const panel = document.querySelector('dialog.map');
    const nodes = [...(panel?.querySelectorAll('button.node') ?? [])];
    if (nodes.length === 0) return "";
    const count = (button) => Number((button.querySelector('small')?.textContent || "0").trim());
    const best = nodes.reduce((a, b) => (count(b) > count(a) ? b : a));
    best.click();
    return (best.textContent || "").trim();
  })()`);
  await waitFor(page, `document.querySelectorAll('dialog.map .teach li').length >= 3`, { timeout: 10000 });
  return chosen;
};

/** The activities, measured after being scrolled to: a list whose tail can
 * be reached is the whole claim, and a panel that simply grew past the
 * screen would fail it at the last row rather than the first. */
const conceptMapAudit = () => page.evaluate(`(() => {
  const panel = document.querySelector('dialog.map');
  if (!panel) return JSON.stringify({ present: false });
  const doc = document.documentElement;
  const outside = (box) => box
    ? Math.max(0, box.right - doc.clientWidth, -box.left, box.bottom - doc.clientHeight, -box.top)
    : 0;
  const teach = panel.querySelector('.teach');
  const items = [...(teach?.querySelectorAll('li') ?? [])];
  items[items.length - 1]?.scrollIntoView({ block: 'end', inline: 'nearest' });
  const tail = items[items.length - 1]?.getBoundingClientRect();
  const head = panel.querySelector('header');
  return JSON.stringify({
    present: true,
    compact: panel.classList.contains('compact'),
    items: items.length,
    lockedBadges: panel.querySelectorAll('.ready.locked').length,
    teachWidth: teach ? Math.round(teach.getBoundingClientRect().width) : 0,
    panelOutside: Math.round(outside(panel.getBoundingClientRect())),
    headerOutside: Math.round(outside(head?.getBoundingClientRect())),
    close: Boolean(panel.querySelector('header button.icon-close')),
    tailOutside: Math.round(outside(tail)),
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

/** GUI, owner: "Feststoff", "Flüssigkeit" and "Gas" drawn over the reagent
 * rows, unreadable or cut off. The shelf's two chip groups share one
 * horizontal scroller, and a horizontal scroller is a scroll container in
 * BOTH axes — so its automatic minimum height is zero, and as a flex item
 * of the shelf column it was shrunk in proportion with a list thousands of
 * pixels tall. Measured at 390 px it stood 5 px tall around 23 px chips.
 *
 * Boxes rather than styles, and the same shape as `trayAudit` above: a
 * chip that is clipped by its own rail, or that shares pixels with a
 * bottle, is the bug whatever the CSS says. */
const shelfFilterAudit = () => page.evaluate(`(() => {
  const rail = document.querySelector('nav.shelf-pane .filter-rail');
  if (!rail) return JSON.stringify({ present: false });
  const chips = [...rail.querySelectorAll('button')].filter((chip) => chip.offsetParent);
  const railBox = rail.getBoundingClientRect();
  const rows = [...document.querySelectorAll('nav.shelf-pane ul li')]
    .filter((row) => row.offsetParent).slice(0, 12);
  const overlaps = [];
  for (const chip of chips) {
    const box = chip.getBoundingClientRect();
    for (const row of rows) {
      const other = row.getBoundingClientRect();
      const shared = Math.min(box.right, other.right) - Math.max(box.left, other.left);
      const stacked = Math.min(box.bottom, other.bottom) - Math.max(box.top, other.top);
      if (shared > 1 && stacked > 1) {
        overlaps.push(chip.textContent.trim() + ' over ' + (row.textContent || '').trim().slice(0, 24));
      }
    }
  }
  return JSON.stringify({
    present: true,
    chips: chips.length,
    // One row: the rail scrolls sideways rather than growing downwards.
    rows: new Set(chips.map((chip) => Math.round(chip.getBoundingClientRect().top))).size,
    small: chips.filter((chip) => chip.getBoundingClientRect().height < 43.5).map((chip) => chip.textContent.trim()),
    // Cut off: the rail is shorter than what it holds, in either axis.
    clipped: chips.filter((chip) => {
      const box = chip.getBoundingClientRect();
      return box.top < railBox.top - 1 || box.bottom > railBox.bottom + 1;
    }).map((chip) => chip.textContent.trim()),
    railHeight: Math.round(railBox.height),
    overlaps: overlaps.slice(0, 6),
  });
})()`);

/** WORLD-003, owner: a Story gate firing in Sandbox, with a sentence
 * offering the permanent stock "after 0 completed missions". Sandbox gates
 * nothing — no bottle wears a lock there, and the sentence under an opened
 * row never counts a milestone of zero.
 *
 * The row has to be OPENED and then left alone for a frame: the note only
 * exists for an open row, and the shell re-renders on a later tick than
 * the click. Reading it in the same breath reads the DOM before the
 * component has answered, which passes whatever the component says. */
const openFirstShelfRow = () => page.evaluate(`(() => {
  const row = [...document.querySelectorAll('nav.shelf-pane ul li')].find((item) => item.offsetParent);
  row?.querySelector('button.species')?.click();
  return Boolean(row);
})()`);

const shelfGateAudit = () => page.evaluate(`(() => {
  const pane = document.querySelector('nav.shelf-pane');
  if (!pane) return JSON.stringify({ present: false });
  return JSON.stringify({
    present: true,
    rows: [...pane.querySelectorAll('ul li')].filter((row) => row.offsetParent).length,
    locked: pane.querySelectorAll('button.species.locked').length,
    // The progression note only. An empty bottle wears the same class with
    // depleted-note beside it, and that one is a fact about the ledger
    // rather than a gate: it is allowed to be there in any mode.
    notes: [...pane.querySelectorAll('.stock-lock:not(.depleted-note)')].map((note) => note.textContent.trim()),
    // Proof the row actually expanded: every arm of that branch renders
    // one of these, so zero means the click did nothing and the audit
    // would have passed on an empty shelf.
    opened: pane.querySelectorAll('form.amounts, .stock-lock').length,
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

/** GUI-052: the provenance drawer, reached from the overflow menu. It has to
 * open with no result on the bench too — a reader asking "who computed
 * this?" before pressing anything gets an honest "nothing routed yet"
 * rather than a control that does not respond. */
const openProvenance = async () => {
  const hasButton = await page.evaluate(`Boolean([...document.querySelectorAll('button.tool')].find((item) =>
    /provenance|herkunft/i.test(item.textContent || "")))`);
  if (!hasButton) {
    await page.evaluate(`document.querySelector('button.utility-toggle')?.click()`);
    await waitFor(page, `document.querySelector('.utility-drawer')`, { timeout: 5000 });
  }
  await page.evaluate(`(() => {
    const button = [...document.querySelectorAll('button.tool')].find((item) =>
      /provenance|herkunft/i.test(item.textContent || ""));
    button?.click();
  })()`);
  return waitFor(page, `document.querySelector('dialog.provenance-drawer')`, { timeout: 5000 });
};

const provenanceAudit = () => page.evaluate(`(() => {
  const panel = document.querySelector('dialog.provenance-drawer');
  if (!panel) return JSON.stringify({ present: false });
  const rect = panel.getBoundingClientRect();
  return JSON.stringify({
    present: true,
    viewportOverflow: Math.max(0, rect.right - document.documentElement.clientWidth, -rect.left),
    below: Math.max(0, rect.bottom - document.documentElement.clientHeight),
    width: rect.width,
    // The one close affordance the whole bench shares.
    close: Boolean(panel.querySelector('button.icon-close')),
    // A drawer with no heading is a box; the heading is what says which
    // question it answers.
    titled: Boolean(panel.querySelector('h2')?.textContent.trim()),
    // It must always say something, even with nothing routed yet.
    said: (panel.querySelector('.headline')?.textContent || '').trim().length,
  });
})()`);

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
  // The list is a WINDOW: only the rows near the viewport are in the DOM,
  // so a card partway down the library is not there to be found by title
  // until something brings it there. Ask for it the way a reader does —
  // the box greps titles and descriptions alike — rather than widening the
  // assertion to whichever card happens to be drawn.
  await page.evaluate(`(() => {
    const box = document.querySelector('dialog input.filter');
    if (!box) return;
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")
      .set.call(box, "Hot pack and cold pack");
    box.dispatchEvent(new Event("input", { bubbles: true }));
  })()`);
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

  // A collapsed side panel used to keep a 44 px column open to say nothing,
  // and then to keep the whole panel one hover away from covering the stage.
  // Collapsed now means gone: a floating chevron at the screen edge, the
  // stage taking the rest, and nothing that can reappear under the pointer.
  await page.evaluate(`(() => {
    document.querySelector('nav.shelf-pane .panel-collapse')?.click();
    document.querySelector('main > aside .panel-collapse')?.click();
  })()`);
  await new Promise((resolve) => setTimeout(resolve, 300));
  const collapsed = JSON.parse(await layoutAudit());
  check("a collapsed panel is a chevron at the edge, not a column",
    Boolean(collapsed.cabinet && collapsed.journal)
      && collapsed.cabinet.width <= 40 && collapsed.journal.width <= 40,
    `${Math.round(collapsed.cabinet?.width ?? -1)}px / ${Math.round(collapsed.journal?.width ?? -1)}px`);
  check("the bench stage takes the freed width",
    collapsed.bench.width > desktop.bench.width + 150,
    `${Math.round(desktop.bench.width)}px → ${Math.round(collapsed.bench.width)}px`);
  check("collapsing keeps every control named and the page unscrolled",
    collapsed.unnamed === 0 && collapsed.bodyOverflow <= 1, `${collapsed.unnamed} unnamed`);
  // The regression this replaces: the panel used to sit in the document,
  // invisible, and a hover or a stray focus put it back over the bench. It
  // must now be out of the layout entirely, and stay out when hovered and
  // when the chevron itself takes focus.
  const hidden = JSON.parse(await page.evaluate(`(() => {
    const measure = (paneSelector) => {
      const body = document.querySelector(paneSelector + ' .pane-body');
      if (!body) return { present: false, laidOut: false, shown: false };
      const style = getComputedStyle(body);
      const rect = body.getBoundingClientRect();
      return {
        present: true,
        laidOut: rect.width > 0 && rect.height > 0,
        shown: style.display !== "none" && style.visibility === "visible",
      };
    };
    const rail = document.querySelector('nav.shelf-pane button.pane-rail');
    rail?.focus();
    return JSON.stringify({
      rail: Boolean(rail),
      named: Boolean(rail?.getAttribute('aria-label')),
      expanded: rail?.getAttribute('aria-expanded'),
      focused: document.activeElement === rail,
      railWidth: rail?.getBoundingClientRect().width ?? 0,
      cabinet: measure('nav.shelf-pane'),
      journal: measure('main > aside'),
      focusables: [...document.querySelectorAll('nav.shelf-pane .pane-body button, main > aside .pane-body button')]
        .filter((button) => button.offsetParent).length,
    });
  })()`));
  check("the chevron is a named, focusable control", hidden.rail && hidden.named && hidden.focused);
  check("the chevron reports the panel as collapsed", hidden.expanded === "false", String(hidden.expanded));
  check("the chevron is a small floating button, not a full-height rail",
    hidden.railWidth > 0 && hidden.railWidth <= 40, `${Math.round(hidden.railWidth)}px`);
  check("a collapsed panel is absent from the layout",
    !hidden.cabinet.laidOut && !hidden.cabinet.shown && !hidden.journal.laidOut && !hidden.journal.shown,
    `cabinet ${JSON.stringify(hidden.cabinet)} journal ${JSON.stringify(hidden.journal)}`);
  check("a collapsed panel leaves nothing behind to focus", hidden.focusables === 0, `${hidden.focusables} controls`);
  await page.evaluate(`(() => {
    document.querySelector('nav.shelf-pane button.pane-rail')?.click();
    document.querySelector('main > aside button.pane-rail')?.click();
  })()`);
  await new Promise((resolve) => setTimeout(resolve, 300));
  const pinned = JSON.parse(await layoutAudit());
  check("pressing the chevron puts the panel back into the layout",
    pinned.cabinet.width > 100 && pinned.journal.width > 100,
    `${Math.round(pinned.cabinet.width)}px / ${Math.round(pinned.journal.width)}px`);
  check("the restored panel pushes the stage instead of covering it",
    pinned.cabinet.right <= pinned.bench.left + 1 && pinned.bench.right <= pinned.journal.left + 1,
    `${Math.round(pinned.cabinet.right)} | ${Math.round(pinned.bench.left)}-${Math.round(pinned.bench.right)} | ${Math.round(pinned.journal.left)}`);
  const restored = JSON.parse(await page.evaluate(`(() => {
    const body = document.querySelector('nav.shelf-pane .pane-body');
    return JSON.stringify({ width: body?.getBoundingClientRect().width ?? 0 });
  })()`));
  check("the restored panel renders its body again", restored.width > 120, `${Math.round(restored.width)}px`);

  check("the periodic table opens from the bench", await openPeriodicTable());
  const labTable = JSON.parse(await periodicAudit());
  check("the default table keeps Fe, Cu, and Zn", ["Fe", "Cu", "Zn"].every((symbol) => labTable.symbols.includes(symbol)));
  check("the default table omits hazardous and synthetic identities",
    ["Po", "At", "Fr", "Ra", "Og"].every((symbol) => !labTable.symbols.includes(symbol)));
  check("every element cell has an accessible name", labTable.unnamed === 0, `${labTable.unnamed} unnamed`);

  // "Nothing happens when I click a reagent in the periodic table." The
  // press did reach the engine, but the surface never came back to the
  // bench that had just changed, so the whole gesture looked ignored. The
  // only honest way to test that is to press it and see the bench move.
  const reagentPress = JSON.parse(await page.evaluate(`(async () => {
    const before = document.querySelectorAll('.feed > *').length;
    const cell = [...document.querySelectorAll('dialog.table-panel button.el')]
      .find((element) => !element.classList.contains('unsupported'));
    cell?.click();
    await new Promise((resolve) => setTimeout(resolve, 120));
    const chip = document.querySelector('dialog.table-panel button.add[data-key]');
    const key = chip?.getAttribute('data-key') ?? null;
    chip?.click();
    await new Promise((resolve) => setTimeout(resolve, 900));
    return JSON.stringify({
      element: Boolean(cell),
      chip: Boolean(chip),
      key,
      closed: !document.querySelector('dialog.table-panel'),
      grew: document.querySelectorAll('.feed > *').length > before,
    });
  })()`));
  check("picking an element offers its shelf reagents", reagentPress.element && reagentPress.chip);
  check("a periodic-table reagent carries the key the bench command needs",
    Boolean(reagentPress.key), String(reagentPress.key));
  check("pressing a periodic-table reagent closes the table and moves the bench",
    reagentPress.closed && reagentPress.grew,
    `closed ${reagentPress.closed}, journal grew ${reagentPress.grew}`);

  check("the periodic table reopens from the bench", await openPeriodicTable());
  await page.evaluate(`document.querySelector('dialog.table-panel button.mode')?.click()`);
  const fullTable = JSON.parse(await periodicAudit());
  check("the explicit full-table mode exposes all 118 identities", fullTable.options === 118, `${fullTable.options} cells`);
  await page.evaluate(`document.querySelector('dialog.table-panel button.icon-close')?.click()`);

  check("the provenance drawer opens from the bench", await openProvenance());
  const provenance = JSON.parse(await provenanceAudit());
  check("the desktop provenance drawer stays inside the viewport",
    provenance.viewportOverflow <= 1 && provenance.below <= 1,
    `${provenance.viewportOverflow}px right, ${provenance.below}px below`);
  check("the provenance drawer names itself and carries the shared close",
    provenance.titled && provenance.close);
  check("the provenance drawer says something even before anything is routed",
    provenance.said > 0, `${provenance.said} characters`);
  await page.evaluate(`document.querySelector('dialog.provenance-drawer button.icon-close')?.click()`);

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

  // The concept map at the three widths it is actually read at. The dialog
  // is a fixed overlay, so the viewport moves under one open panel rather
  // than reloading the app three times — which is also the harder test:
  // the layout has to survive the change, not merely be born into it.
  if (await openConceptMap()) {
    const concept = await pickBusiestConcept();
    const wide = JSON.parse(await conceptMapAudit());
    check("the concept map lists at least three activities for a busy concept",
      wide.items >= 3, `${concept}: ${wide.items} links`);
    check("the desktop concept map gives the activities a column of their own",
      wide.compact === false && wide.teachWidth >= 320, `${wide.teachWidth}px`);
    check("the desktop concept map keeps its last activity on screen",
      wide.panelOutside <= 1 && wide.tailOutside <= 1,
      `${wide.panelOutside}px panel, ${wide.tailOutside}px tail`);
    check("the concept map header stays fixed with its close",
      wide.close === true && wide.headerOutside <= 1, `${wide.headerOutside}px`);
    // Sandbox is the laboratory this audit stands in, and it gates nothing.
    check("Sandbox marks no experiment locked in the concept map",
      wide.lockedBadges === 0, `${wide.lockedBadges} locked badges`);

    // A resize is announced, not applied: the browser reflows and the shell
    // re-renders on later frames, so measuring in the same breath measures
    // the width that has just gone.
    await viewport(768, 900);
    await settle();
    const tablet = JSON.parse(await conceptMapAudit());
    check("the 768 px concept map keeps the activities beside the graph",
      tablet.compact === false && tablet.teachWidth >= 320, `${tablet.teachWidth}px`);
    check("the 768 px concept map keeps its last activity on screen",
      tablet.panelOutside <= 1 && tablet.tailOutside <= 1,
      `${tablet.panelOutside}px panel, ${tablet.tailOutside}px tail`);
    check("Sandbox marks no experiment locked at 768 px", tablet.lockedBadges === 0,
      `${tablet.lockedBadges} locked badges`);

    await viewport(390, 844);
    await settle();
    const phone = JSON.parse(await conceptMapAudit());
    check("the 390 px concept map collapses the graph to a list",
      phone.compact === true && phone.items >= 3, `${phone.items} links`);
    check("the 390 px concept map keeps its last activity on screen",
      phone.panelOutside <= 1 && phone.tailOutside <= 1,
      `${phone.panelOutside}px panel, ${phone.tailOutside}px tail`);
    check("Sandbox marks no experiment locked at 390 px", phone.lockedBadges === 0,
      `${phone.lockedBadges} locked badges`);

    await viewport(1440, 900);
    await page.evaluate(`document.querySelector('dialog.map button.icon-close')?.click()`);
  } else {
    check("the concept map opens from the utilities drawer", false, "no dialog.map");
  }

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
  await waitFor(page, `[...document.querySelectorAll('nav.shelf-pane .name')].some((item) => item.offsetParent)`, { timeout: 60000 });
  await settle();
  const germanFilters = JSON.parse(await shelfFilterAudit());
  check("the German phase chips stand clear of the bottles",
    germanFilters.present && germanFilters.overlaps.length === 0,
    germanFilters.overlaps.join(", ") || `${germanFilters.chips} chips`);
  check("the German phase chips are drawn whole, not clipped by their rail",
    germanFilters.clipped?.length === 0, `${(germanFilters.clipped ?? []).join(", ")} in a ${germanFilters.railHeight}px rail`);
  check("the German phase chips keep 44 px touch targets",
    germanFilters.small?.length === 0, (germanFilters.small ?? []).join(", "));
  // Sandbox is the laboratory this audit stands in, and it gates nothing —
  // not the codex badge, not the concept map, and not the shelf.
  check("a shelf row opens its amount form", await openFirstShelfRow());
  await settle();
  const germanGate = JSON.parse(await shelfGateAudit());
  check("Sandbox locks no material on the shelf",
    germanGate.present && germanGate.rows > 0 && germanGate.locked === 0,
    `${germanGate.locked} of ${germanGate.rows} locked`);
  // The opened form proves the audit looked at an expanded row rather than
  // at a shelf that never answered the click — the note and the form are
  // the two arms of the same branch, so one of them is always rendered.
  check("Sandbox explains no material away with a milestone",
    germanGate.opened === 1 && (germanGate.notes ?? []).length === 0,
    (germanGate.notes ?? []).join(" | ") || `${germanGate.opened} open row(s)`);

  await viewport(390, 844);
  await page.goto(`${origin}/app/`);
  await openBench();
  const mobile = JSON.parse(await layoutAudit());
  check("phone layout has no page-level horizontal overflow", mobile.bodyOverflow <= 1, `${mobile.bodyOverflow}px`);
  const tabs = JSON.parse(await mobileTabs());
  check("phone navigation exposes three tabs", tabs.length === 3, `${tabs.length} tabs`);
  check("phone tabs meet the 44 px touch minimum", tabs.every((tab) => tab.width >= 44 && tab.height >= 44));
  const phoneShelf = await chooseMobilePane(1);
  await waitFor(page, `[...document.querySelectorAll('nav.shelf-pane .name')].some((item) => item.offsetParent)`, { timeout: 60000 });
  await settle();
  const phoneFilters = JSON.parse(await shelfFilterAudit());
  // 390 px is where the owner read it: the narrower the pane, the more of
  // the column the list claims and the less of it the rail was left.
  check("the 390 px phase chips stand clear of the bottles",
    phoneFilters.present && phoneFilters.overlaps.length === 0,
    phoneFilters.overlaps.join(", ") || `${phoneFilters.chips} chips`);
  check("the 390 px phase chips are drawn whole, not clipped by their rail",
    phoneFilters.clipped?.length === 0, `${(phoneFilters.clipped ?? []).join(", ")} in a ${phoneFilters.railHeight}px rail`);
  check("the 390 px phase chips keep 44 px touch targets",
    phoneFilters.small?.length === 0, (phoneFilters.small ?? []).join(", "));
  check("the 390 px cabinet stays inside the page", phoneShelf.bodyOverflow <= 1, `${phoneShelf.bodyOverflow}px`);
  await chooseMobilePane(0);
  check("the periodic table opens on a phone", await openPeriodicTable());
  const phoneTable = JSON.parse(await periodicAudit());
  check("the phone periodic table stays inside the viewport", phoneTable.viewportOverflow <= 1, `${phoneTable.viewportOverflow}px`);
  await page.evaluate(`document.querySelector('dialog.table-panel button.icon-close')?.click()`);
  check("the equipment cupboard opens on a phone", await openCupboard());
  const phoneCupboard = JSON.parse(await cupboardAudit());
  check("the phone cupboard stays inside the viewport", phoneCupboard.viewportOverflow <= 1, `${phoneCupboard.viewportOverflow}px`);
  await page.evaluate(`document.querySelector('dialog.cupboard button.icon-close')?.click()`);

  check("the provenance drawer opens on a phone", await openProvenance());
  const phoneProvenance = JSON.parse(await provenanceAudit());
  // A right-anchored drawer is the shape that hangs off the edge when it is
  // sized in rem and the phone is narrower than the rem figure. Full-bleed
  // below 30rem is what keeps this at zero.
  check("the phone provenance drawer stays inside the viewport",
    phoneProvenance.viewportOverflow <= 1 && phoneProvenance.below <= 1,
    `${phoneProvenance.viewportOverflow}px right, ${phoneProvenance.below}px below`);
  const phoneLayout = JSON.parse(await layoutAudit());
  check("the open provenance drawer does not widen the phone page",
    phoneLayout.bodyOverflow <= 1, `${phoneLayout.bodyOverflow}px`);
  await page.evaluate(`document.querySelector('dialog.provenance-drawer button.icon-close')?.click()`);

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

  // The MESSEN strip at the width the owner reported it at. Boxes are only
  // boxes once the row is laid out, so the wait is for a VISIBLE button
  // rather than for one in the document.
  if (await openStrip()
      && await waitFor(page, `Boolean(document.querySelector('.instrument-tray button')?.offsetParent)`, { timeout: 10000 })) {
    const tray = JSON.parse(await trayAudit());
    check("the MESSEN strip is one row of separate buttons at 390 px",
      tray.present && tray.buttons >= 2 && tray.overlaps.length === 0,
      tray.overlaps.join(", ") || `${tray.buttons} buttons`);
    check("every MESSEN button keeps a 44 px touch target at 390 px",
      tray.small.length === 0, tray.small.join(", "));
  } else {
    check("the MESSEN strip is one row of separate buttons at 390 px", false, "no .instrument-tray");
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
  const narrowFilters = JSON.parse(await shelfFilterAudit());
  check("320 px phase chips share one row and stand clear of the bottles",
    narrowFilters.present && narrowFilters.rows === 1 && narrowFilters.overlaps.length === 0,
    narrowFilters.overlaps.join(", ") || `${narrowFilters.chips} chips on ${narrowFilters.rows} row(s)`);
  check("320 px phase chips are drawn whole, not clipped by their rail",
    narrowFilters.clipped?.length === 0, `${(narrowFilters.clipped ?? []).join(", ")} in a ${narrowFilters.railHeight}px rail`);
  check("320 px phase chips keep 44 px touch targets",
    narrowFilters.small?.length === 0, (narrowFilters.small ?? []).join(", "));
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

  /* -- the cabinet that never answers ---------------------------------- */
  //
  // #599's reproduction, made repeatable. It used a payload whose engine
  // has no `catalog` method at all — a service worker pairing a new app
  // bundle with a cached older engine, and equally what any dropped round
  // trip looks like from the app's side. The shelf's half of that is
  // fixed; the half left open was that `refreshCatalog` swallowed the
  // failure, so a deployment in that state was broken in silence.
  //
  // The engine is doctored rather than replaced: the payload's own
  // wasm-bindgen module, with one line appended that takes the method off
  // the class. Version-independent — it neither knows nor cares how the
  // method is declared — and a no-op against an engine that genuinely
  // predates the endpoint, which is the same engine it is imitating.
  //
  // A second origin, because a service worker and its caches belong to
  // one: the doctored engine cannot reach the registration the rest of
  // this file has been driving, and that registration cannot answer for
  // the doctored one.
  /* -- the remove-vessel dialog's three exits ---------------------------
   *
   * The owner: "we can NOT anymore get actually rid of vessels ... clicking
   * on 'Entsorgungsstation öffnen' just does nothing". Every unit test in
   * web/app renders through `svelte/server`, which produces markup and
   * never fires a handler, so all three exits were perfect on paper and
   * dead in a browser.
   *
   * The cause was Svelte 5 semantics. `App.svelte` mounts the dialog under
   * `{#if removeVessel}{@const vessel = removeVessel}`, and an `{@const}`
   * in Svelte 5 is a DERIVED. Each handler cleared `removeRequest` — the
   * state `removeVessel` derives from — and then read `vessel.id`, which
   * re-evaluated the derived against the state just cleared and read `.id`
   * off null. The dialog was already gone by then, so the reader saw a
   * button that did nothing.
   *
   * This is the level the defect is visible at, which is why the guard
   * lives here and not in vitest: it needs a real click and a real
   * uncaught exception.
   */
  console.log("");
  const errors = await page.evaluate(`(() => {
    // Whatever the checks above left open, close it: this one drives a
    // gesture that starts on the bench, and a scrim over it would swallow
    // the first click and fail for the wrong reason.
    document.querySelectorAll('.scrim, .world-scrim').forEach((scrim) => scrim.click());
    window.__uxErrors = [];
    window.addEventListener("error", (event) => window.__uxErrors.push(String(event.message)));
    window.addEventListener("unhandledrejection", (event) => window.__uxErrors.push(String(event.reason)));
    return "armed";
  })()`);
  check("the error trap is armed", errors === "armed", errors);

  await page.evaluate(`(() => {
    const row = [...document.querySelectorAll('nav.shelf-pane button.species')]
      .find((button) => /\\bH2O\\b/.test(button.textContent));
    row?.click();
  })()`);
  await settle();
  await page.evaluate(`document.querySelector('form.amounts button.add-amount')?.click()`);
  // The form closing is the app accepting the add; the wait after it is the
  // engine's round trip, which is a wasm solve and not instant.
  await waitFor(page, `!document.querySelector('nav.shelf-pane form.amounts')`, { timeout: 30000 });
  await new Promise((resolve) => setTimeout(resolve, 4000));

  const dialogOpened = JSON.parse(await page.evaluate(`(() => {
    document.querySelector('.placement-toggle')?.click();
    return JSON.stringify({ toggled: Boolean(document.querySelector('.placement-toggle')) });
  })()`));
  await settle();
  await page.evaluate(`document.querySelector('.bench button.remove')?.click()`);
  await settle();
  const dialog = JSON.parse(await page.evaluate(`(() => JSON.stringify({
    open: Boolean(document.querySelector('dialog[aria-labelledby="remove-vessel-title"]')),
    waste: Boolean(document.querySelector('button.waste')),
  }))()`));
  check("a vessel holding something opens the remove dialog with a waste exit",
    dialogOpened.toggled && dialog.open && dialog.waste,
    JSON.stringify({ ...dialogOpened, ...dialog }));

  if (dialog.waste) {
    await page.evaluate(`document.querySelector('button.waste')?.click()`);
    await settle();
    const landed = JSON.parse(await page.evaluate(`(() => JSON.stringify({
      station: Boolean(document.querySelector('#utility-title')),
      removeGone: !document.querySelector('dialog[aria-labelledby="remove-vessel-title"]'),
      errors: window.__uxErrors,
    }))()`));
    // The whole defect in one assertion: the signpost has to ARRIVE
    // somewhere. Closing the dialog and opening nothing is the failure.
    check("\"open waste station\" lands on the utility station", landed.station,
      JSON.stringify(landed));
    check("and it throws nothing on the way", landed.errors.length === 0,
      landed.errors.join(" | "));
  } else {
    check("\"open waste station\" lands on the utility station", false, "no waste exit to press");
  }
  await page.evaluate(`document.querySelector('.scrim')?.click()`);
  await settle();

  console.log("");
  let servedDoctored = 0;
  const { server: engineless, origin: mismatched } = await serve(PAYLOAD, {
    handleRequest: async (request, response) => {
      if (!/\/kerotakis_wasm\.js$/.test(new URL(request.url, "http://x").pathname)) return false;
      servedDoctored += 1;
      const module = await readFile(join(PAYLOAD, "kerotakis_wasm.js"), "utf8");
      response.writeHead(200, { "content-type": "text/javascript" });
      // Guarded: if a future wasm-bindgen stops exporting the class under
      // this name the rig must fail the checks below rather than fail to
      // PARSE, which would take the whole engine down and produce a
      // different bug than the one being reproduced.
      response.end(`${module}\n// An engine from before the catalog endpoint existed.\ntry { delete Lab.prototype.catalog; } catch { /* not this shape any more */ }\n`);
      return true;
    },
  });
  try {
    await viewport(1440, 900);
    await page.goto(`${mismatched}/app/`);
    await page.evaluate(`localStorage.setItem("kerotakis.locale", "de");
      localStorage.setItem("kerotakis.mode.v1", "story");
      localStorage.setItem("kerotakis.console.v1", "hidden");`);
    await page.goto(`${mismatched}/app/`);
    await openBench();
    check("the rig served its own copy of the engine module", servedDoctored > 0,
      `${servedDoctored} intercepted`);

    // Bounded, and this is where that is asserted: an app that retried
    // forever would sit here until the timeout with nothing to show. The
    // policy is three asks and at most 1.6 s of waiting, so 30 s is slack
    // for a cold wasm boot rather than patience with a spinner.
    const spoke = await waitFor(page,
      `Boolean(document.querySelector('[data-status="cabinet-silent"]'))`, { timeout: 30000 });
    check("the journal says the cabinet did not answer, rather than waiting forever", spoke === true);

    const notice = JSON.parse(await page.evaluate(`(() => {
      const notes = [...document.querySelectorAll('[data-status="cabinet-silent"]')];
      return JSON.stringify({
        count: notes.length,
        text: notes[0]?.textContent.trim() ?? "",
        icons: notes.filter((note) => note.querySelector('.status-mark')).length,
      });
    })()`));
    // Once per episode. Five call sites reporting five identical notices
    // down the journal is its own defect.
    check("it says so once, not once per caller", notice.count === 1, `${notice.count} notices`);
    check("it is marked as session bookkeeping, like a save that failed", notice.icons === 1);
    // In the reader's language, through t(). An English sentence here is a
    // missing German row, which is invisible to every coverage count.
    check("it is in the reader's language", /Vorratsschrank/.test(notice.text), notice.text);
    // The reason is the diagnostic: without it the journal says a
    // deployment is broken without saying how.
    check("it carries the engine's own reason", /catalog/i.test(notice.text), notice.text);

    await settle();
    // Story opens on the "unlocked" scope, whose filter asks `available` —
    // which answers no for everything while the catalogue is silent. So
    // the first thing a Story learner meets here is an EMPTY shelf, and
    // before this it blamed the filter: "nothing on the shelf matches",
    // over a cabinet of 188 bottles.
    const empty = await page.evaluate(
      `document.querySelector('nav.shelf-pane li.none')?.textContent.trim() ?? ""`);
    check("an empty Story shelf blames the cabinet rather than the filter",
      /Vorratsschrank/.test(empty) && /nicht geantwortet/.test(empty), empty || "no empty-state line");

    // And with the whole cabinet showing, the bottles themselves. Story
    // gates materials, so these rows ARE locked — correctly, because
    // nothing has said they are reachable. What #599 left was the sentence
    // underneath, which said "not yet" and meant "not ever".
    await page.evaluate(`(() => {
      const chips = [...document.querySelectorAll('nav.shelf-pane [role="radio"]')];
      chips[chips.length - 1]?.click();
    })()`);
    await settle();
    const stranded = JSON.parse(await page.evaluate(`(() => {
      const rows = [...document.querySelectorAll('nav.shelf-pane ul li')]
        .filter((item) => item.offsetParent && !item.classList.contains('none'));
      rows[0]?.querySelector('button.species')?.click();
      return JSON.stringify({ rows: rows.length });
    })()`));
    check("the whole cabinet is reachable through the scope chips", stranded.rows > 0,
      `${stranded.rows} rows`);
    await settle();
    const note = JSON.parse(await page.evaluate(`(() => {
      const notes = [...document.querySelectorAll('nav.shelf-pane .stock-lock:not(.depleted-note)')];
      return JSON.stringify({ count: notes.length, text: notes[0]?.textContent.trim() ?? "" });
    })()`));
    check("the bottle says the cabinet did not answer, not that it is still checking",
      note.count === 1 && /nicht geantwortet/.test(note.text), note.text || `${note.count} notes`);
    check("and it still invents no milestone", !/\b0\b/.test(note.text), note.text);
  } finally {
    engineless.close();
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
