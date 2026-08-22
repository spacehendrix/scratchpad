// One shared animation ticker for every spinner in the app (a single
// setInterval, not one per component). Honors prefers-reduced-motion by
// simply never ticking.
class Ticker {
  frame = $state(0);

  constructor() {
    if (
      typeof window !== "undefined" &&
      !window.matchMedia("(prefers-reduced-motion: reduce)").matches
    ) {
      setInterval(() => {
        this.frame = (this.frame + 1) % 1024;
      }, 80);
    }
  }
}

export const ticker = new Ticker();
