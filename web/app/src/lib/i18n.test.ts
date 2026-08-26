import { afterEach, describe, expect, it } from "vitest";
import { i18n, t } from "./i18n.svelte";

describe("i18n", () => {
  afterEach(() => {
    i18n.locale = "en";
  });

  it("translates interface and engine-domain vocabulary into German", () => {
    i18n.locale = "de";
    expect(t("save notes")).toBe("Notizen speichern");
    expect(t("sodium chloride")).toBe("Natriumchlorid");
    expect(t("supply cabinet")).toBe("Materialschrank");
    expect(t("high contrast")).toBe("Hoher Kontrast");
  });

  it("interpolates translated messages", () => {
    i18n.locale = "de";
    expect(t("timeline: step {position} of {total}", { position: 2, total: 5 })).toBe(
      "Zeitleiste: Schritt 2 von 5",
    );
  });

  it("uses the source message as the English catalog", () => {
    i18n.locale = "en";
    expect(t("save notes")).toBe("save notes");
  });
});
