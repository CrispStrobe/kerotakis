import { describe, expect, it } from "vitest";
import { schedule, type Clock } from "./replay";

/** A hand-cranked clock: timeouts fire only when advance() reaches them. */
class FakeClock implements Clock {
  now = 0;
  private queue: { at: number; fn: () => void; id: number }[] = [];
  private nextId = 1;
  setTimeout(fn: () => void, ms: number): unknown {
    const id = this.nextId++;
    this.queue.push({ at: this.now + ms, fn, id });
    return id;
  }
  clearTimeout(id: unknown): void {
    this.queue = this.queue.filter((q) => q.id !== id);
  }
  advance(ms: number): void {
    const until = this.now + ms;
    for (;;) {
      const due = this.queue.filter((q) => q.at <= until).sort((a, b) => a.at - b.at)[0];
      if (!due) break;
      this.now = due.at;
      this.queue = this.queue.filter((q) => q.id !== due.id);
      due.fn();
    }
    this.now = until;
  }
}

const record = () => {
  const seen: number[] = [];
  let done = 0;
  return {
    seen,
    onPoint: (i: number) => seen.push(i),
    onDone: () => done++,
    doneCount: () => done,
  };
};

describe("the playback scheduler", () => {
  it("fires every point in order, then done", () => {
    const clock = new FakeClock();
    const r = record();
    schedule(4, 100, r.onPoint, r.onDone, { clock });
    expect(r.seen).toEqual([]);
    clock.advance(100);
    expect(r.seen).toEqual([0]);
    clock.advance(350);
    expect(r.seen).toEqual([0, 1, 2, 3]);
    expect(r.doneCount()).toBe(1);
  });

  it("clamps total time: many points compress their pacing", () => {
    const clock = new FakeClock();
    const r = record();
    schedule(200, 100, r.onPoint, r.onDone, { clock, maxMs: 1000 });
    clock.advance(1000);
    expect(r.seen.length).toBe(200);
    expect(r.doneCount()).toBe(1);
  });

  it("finish() delivers every remaining point immediately, exactly once", () => {
    const clock = new FakeClock();
    const r = record();
    const p = schedule(5, 100, r.onPoint, r.onDone, { clock });
    clock.advance(150); // point 0 fired
    p.finish();
    expect(r.seen).toEqual([0, 1, 2, 3, 4]);
    expect(r.doneCount()).toBe(1);
    clock.advance(1000); // nothing further
    expect(r.seen.length).toBe(5);
    expect(r.doneCount()).toBe(1);
  });

  it("cancel() stops everything, including done", () => {
    const clock = new FakeClock();
    const r = record();
    const p = schedule(5, 100, r.onPoint, r.onDone, { clock });
    clock.advance(150);
    p.cancel();
    clock.advance(1000);
    expect(r.seen).toEqual([0]);
    expect(r.doneCount()).toBe(0);
    expect(p.settled).toBe(true);
  });

  it("reduced motion jumps straight to the settled truth", () => {
    const clock = new FakeClock();
    const r = record();
    const p = schedule(3, 100, r.onPoint, r.onDone, { clock, reducedMotion: true });
    expect(r.seen).toEqual([0, 1, 2]);
    expect(r.doneCount()).toBe(1);
    expect(p.settled).toBe(true);
  });

  it("zero points settles immediately without waiting", () => {
    const clock = new FakeClock();
    const r = record();
    const p = schedule(0, 100, r.onPoint, r.onDone, { clock });
    expect(p.settled).toBe(true);
    expect(r.doneCount()).toBe(1);
  });
});
