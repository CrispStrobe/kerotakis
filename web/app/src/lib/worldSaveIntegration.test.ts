import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const ROOT = resolve(import.meta.dirname, "..");
const app = readFileSync(resolve(ROOT, "App.svelte"), "utf8");
const worldHome = readFileSync(resolve(ROOT, "lib/components/WorldHome.svelte"), "utf8");

describe("WORLD-001 App integration contract", () => {
  it("constructs Session and durable mode UI over AppSave storage", () => {
    expect(app).toContain("bootstrapAppSave(appStorage)");
    expect(app).toMatch(/new AppSaveModeStorage\(appSaveRepository, labMode,/);
    expect(app).toContain("new Session(");
    expect(app).toContain("modeStorage,");
    for (const key of ["MODE_LAYOUT_KEY", "MODE_APPARATUS_KEY", "MODE_GUIDES_KEY", "MODE_ROOM_KEY"]) {
      expect(app).toContain(key);
    }
    expect(app).not.toContain("new ModeStorage(");
  });

  it("offers only an explicit Story-to-Sandbox bench copy", () => {
    expect(worldHome).toContain('mode === "story" && canclone && onclone');
    expect(worldHome).toContain('t("Copy Story bench to Sandbox")');
    expect(worldHome).toContain('t("This replaces the existing Sandbox bench. Continue?")');
    expect(worldHome).toContain("sandboxHasBench ? (confirmingClone = true) : onclone?.()");
    expect(worldHome).toContain("onclone?.(); confirmingClone = false;");
    expect(worldHome).toContain("Story progress and supplies stay separate");
    expect(app).toContain("cloneStoryBenchToSandbox(appSaveRepository)");
    expect(`${app}\n${worldHome}`).not.toMatch(/cloneSandbox.*Story|Copy Sandbox bench to Story/);
  });

  it("does not navigate until after the copy action has returned", () => {
    const handler = app.slice(
      app.indexOf("function copyStoryBenchToSandbox()"),
      app.indexOf("let helpOpen"),
    );
    expect(handler).toContain("const result = cloneStoryBenchToSandbox(appSaveRepository)");
    expect(handler).not.toContain("enterLab(");
    expect(handler).not.toContain("location.reload");
  });

  it("renders nonblocking corrupt, unavailable, and recovery notices", () => {
    expect(app).toContain("appStorage === null");
    expect(app).toContain('appSaveBootstrap?.status === "corrupt"');
    expect(app).toContain('appSaveBootstrap?.status === "unavailable"');
    expect(app).toContain('appSaveBootstrap.source === "recovered"');
    expect(worldHome).toContain('class="save-notice" role="status"');
  });

  it("detects an existing Sandbox bench from every overwritten field", () => {
    expect(app).toContain('sandboxStorage?.getItem("kero.session.v1") !== null');
    expect(app).toContain("sandboxStorage?.getItem(MODE_LAYOUT_KEY) !== null");
    expect(app).toContain("sandboxStorage?.getItem(MODE_APPARATUS_KEY) !== null");
  });
});
