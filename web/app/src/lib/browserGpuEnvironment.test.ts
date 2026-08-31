import { describe, expect, it, vi } from "vitest";
import { browserGpuEnvironment } from "./browserGpuEnvironment";

const eventSource = () => ({
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
});

describe("browser GPU environment", () => {
  it("resolves one complete structural environment", () => {
    const media = { ...eventSource(), matches: false };
    const document = { ...eventSource(), visibilityState: "visible" };
    const gpu = {
      requestAdapter: vi.fn(async () => null),
      getPreferredCanvasFormat: () => "bgra8unorm",
    };
    const environment = browserGpuEnvironment({
      navigator: { gpu, webdriver: true },
      document,
      matchMedia: vi.fn(() => media),
    });
    expect(environment).toMatchObject({ reducedMotion: media, document, headless: true });
    expect(environment?.provider.preferredCanvasFormat?.()).toBe("bgra8unorm");
  });

  it.each([
    undefined,
    {},
    { navigator: { gpu: { requestAdapter() {} } }, document: {}, matchMedia: (): null => null },
    { navigator: { gpu: { requestAdapter() {} } }, document: { ...eventSource(), visibilityState: "visible" }, matchMedia: () => ({ matches: false }) },
  ])("fails an incomplete environment closed (case %#)", (globalObject) => {
    expect(browserGpuEnvironment(globalObject)).toBeNull();
  });

  it("contains hostile browser getters and media factories", () => {
    const hostile = Object.defineProperty({}, "navigator", { get: () => { throw new Error("navigator"); } });
    expect(browserGpuEnvironment(hostile)).toBeNull();
    expect(browserGpuEnvironment({
      navigator: { gpu: { requestAdapter() {} } },
      document: { ...eventSource(), visibilityState: "visible" },
      matchMedia: () => { throw new Error("media"); },
    })).toBeNull();
  });
});
