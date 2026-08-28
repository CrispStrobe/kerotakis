import { describe, expect, it } from "vitest";
import { INSTRUMENTS, instrumentCommand, instrumentVerb } from "./instruments";

describe("instrument catalog", () => {
  it("gives every released instrument a stable cabinet verb", () => {
    expect(new Set(INSTRUMENTS.map((item) => instrumentVerb(item.token))).size).toBe(INSTRUMENTS.length);
  });

  it("compiles special and ordinary measurements to public operators", () => {
    expect(instrumentCommand(1, "thermometer")).toBe("measure v2 thermometer");
    expect(instrumentCommand(1, "chromatograph")).toBe("chromatograph v2");
    expect(instrumentCommand(1, "smell")).toBe("smell v2");
  });
});
