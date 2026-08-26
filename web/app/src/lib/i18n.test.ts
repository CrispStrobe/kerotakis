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
    expect(t("place vessel here")).toBe("Gefäß hier abstellen");
    expect(t("Your saves stay separate.")).toBe("Deine Spielstände bleiben getrennt.");
    expect(t("after one mission")).toBe("nach einer Mission");
    expect(t("mission kit")).toBe("Missionsset");
    expect(t("place on bench")).toBe("auf den Labortisch stellen");
  });

  it("interpolates translated messages", () => {
    i18n.locale = "de";
    expect(t("timeline: step {position} of {total}", { position: 2, total: 5 })).toBe(
      "Zeitleiste: Schritt 2 von 5",
    );
    expect(t("vessel v{vessel} moved to {zone}", { vessel: 2, zone: t("analyse") })).toBe(
      "Gefäß v2 nach Analysieren verschoben",
    );
    expect(t("after {count} missions", { count: 3 })).toBe("nach 3 Missionen");
  });

  it("uses the source message as the English catalog", () => {
    i18n.locale = "en";
    expect(t("save notes")).toBe("save notes");
  });
});
