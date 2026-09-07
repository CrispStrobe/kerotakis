import { describe, expect, it } from "vitest";
import { registerText } from "./registerText";

/**
 * Quest prose is keyed by DETAIL LEVEL, and the picker treated that key as
 * if it were also the language key: `title[register]`, full stop. So a
 * German shell offered "Aufgabe auswählen…" over a list of English titles.
 * These pin the two axes apart, and pin the degradation: a level with no
 * German still reads its English rather than going blank, because a
 * sentence in the wrong language beats an empty row in a picker.
 */
describe("register-keyed prose", () => {
  const map = {
    lv1: "How sour is sour?",
    lv1_de: "Wie sauer ist sauer?",
    lv2: "Acid-base chemistry",
    lv3: "pH of strong and weak acids",
  };

  it("prefers the German cell at the level asked for", () => {
    expect(registerText(map, "lv1", "de")).toBe("Wie sauer ist sauer?");
  });

  it("keeps English out of a German reader's way only where German exists", () => {
    // lv2 has no German twin: English at lv2 beats German at lv1, because
    // the level is what the reader chose and the language is what they get.
    expect(registerText(map, "lv2", "de")).toBe("Acid-base chemistry");
  });

  it("never hands German to an English reader", () => {
    expect(registerText(map, "lv1", "en")).toBe("How sour is sour?");
    expect(registerText({ lv1_de: "nur Deutsch" }, "lv1", "en")).toBe("");
  });

  it("accepts the bare level spelling the payloads also use", () => {
    expect(registerText({ "1": "bare", "1_de": "blank" }, "lv1", "de")).toBe("blank");
  });

  it("falls back through lv2 before giving up", () => {
    expect(registerText({ lv2: "the middle voice" }, "lv3", "en")).toBe("the middle voice");
  });

  it("returns the caller's fallback, not a blank, when there is nothing at all", () => {
    expect(registerText({}, "lv2", "de", "acid-base")).toBe("acid-base");
    expect(registerText(null, "lv2", "de", "acid-base")).toBe("acid-base");
    expect(registerText(undefined, "lv2", "de")).toBe("");
  });

  it("does not fall back onto a translation and call it the neutral text", () => {
    // The last resort walks the map, and a `_de` cell must not be mistaken
    // for the unsuffixed one an English reader is owed.
    expect(registerText({ lv9_de: "Deutsch" }, "lv1", "en", "id")).toBe("id");
    expect(registerText({ lv9: "English", lv9_de: "Deutsch" }, "lv1", "en")).toBe("English");
  });
});
