import {
  browserWebGpuProvider,
  type MediaQueryListLike,
  type VisibilityDocumentLike,
  type WebGpuProviderLike,
} from "./webGpuLifecycle";

export interface BrowserGpuEnvironment {
  provider: WebGpuProviderLike;
  reducedMotion: MediaQueryListLike;
  document: VisibilityDocumentLike;
  headless: boolean;
}

/** Resolve the complete browser policy seam or fail closed before listeners. */
export function browserGpuEnvironment(globalObject: unknown = globalThis): BrowserGpuEnvironment | null {
  try {
    if (typeof globalObject !== "object" || globalObject === null) return null;
    const provider = browserWebGpuProvider(globalObject);
    const matchMedia = Reflect.get(globalObject, "matchMedia");
    const documentValue = Reflect.get(globalObject, "document");
    const navigatorValue = Reflect.get(globalObject, "navigator");
    if (!provider || typeof matchMedia !== "function" || typeof documentValue !== "object" || documentValue === null) return null;
    const reducedMotion = Reflect.apply(matchMedia, globalObject, ["(prefers-reduced-motion: reduce)"]);
    if (typeof reducedMotion !== "object" || reducedMotion === null) return null;
    if (typeof Reflect.get(reducedMotion, "matches") !== "boolean") return null;
    if (typeof Reflect.get(reducedMotion, "addEventListener") !== "function") return null;
    if (typeof Reflect.get(reducedMotion, "removeEventListener") !== "function") return null;
    if (typeof Reflect.get(documentValue, "visibilityState") !== "string") return null;
    if (typeof Reflect.get(documentValue, "addEventListener") !== "function") return null;
    if (typeof Reflect.get(documentValue, "removeEventListener") !== "function") return null;
    return {
      provider,
      reducedMotion: reducedMotion as MediaQueryListLike,
      document: documentValue as VisibilityDocumentLike,
      headless: typeof navigatorValue === "object"
        && navigatorValue !== null
        && Reflect.get(navigatorValue, "webdriver") === true,
    };
  } catch {
    return null;
  }
}
