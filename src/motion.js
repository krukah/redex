// Motion: a small algebra of transitions, so every animation in the widgets is built the same way.
//
//   Motion      a function (clock) => Promise that plays something and resolves when it is done.
//               It always resolves: each primitive also races a timer, so a throttled or paused
//               timeline never leaves the page half animated.
//   clock       { speed }: durations and delays are written at speed 1 and divided by speed.
//
// Primitives, one per kind of change an element can go through:
//   enter(el)              fade in where it stands
//   exit(el)               fade out where it stands, then remove it
//   slide(el, dx, dy)      play back from an offset to where layout put it (FLIP)
//   resize(el, from, to)   grow or shrink its box between two rects
//   fly(el, dx, dy)        travel in a straight line by (dx, dy) and stay there
//   crossfade(a, b)        a fades out while b fades in; both stay
//   draw(path)             stroke an SVG path from start to end (needs pathLength="1" and dasharray 1)
//
// Compositions:
//   all(ms)                play together
//   seq(ms)                play one after another
//   after(ms, m)           start m after a delay
//   stagger(gap, ms)       play together, each one `gap` later than the one before
//   act(fn)                a side effect at a point in a sequence
//
// Easing curves, by role:
//   EASE.move      ease-in-out, a little quicker to start than to stop; for travel and layout
//   EASE.enter     decelerate; things arriving
//   EASE.exit      ease-in-out over opacity; a fade that neither pops nor lingers
//
// Nothing is ever scaled. A scaled element is a resampled bitmap: it blurs and changes size.
// When something must change shape on the way, two copies travel together and crossfade.
//
// CSS keyframes are left for color only: highlight afterglows and the graph's "here" pulse.
const reduced = typeof matchMedia === 'function' ? matchMedia('(prefers-reduced-motion: reduce)') : null;
export const motion = () => !reduced?.matches;
export const wait = ms => new Promise(r => setTimeout(r, ms));

export const BEAT = 620;   // the length of one firing's movement, at speed 1
export const EASE = {
  move: 'cubic-bezier(.45, 0, .2, 1)',
  enter: 'cubic-bezier(0, 0, .2, 1)',
  exit: 'cubic-bezier(.4, 0, .6, 1)',
};
const NONE = () => Promise.resolve();

// The one place that calls element.animate. `keep` leaves the final frame applied; otherwise the
// animation is cancelled when it ends, so the element returns to its own styles.
function tween(el, frames, { ms = BEAT, delay = 0, ease = EASE.move, keep = false } = {}) {
  if (!el) return NONE;
  return clock => {
    if (!motion()) return Promise.resolve();
    const s = clock?.speed || 1, duration = ms / s, wait0 = delay / s;
    const a = el.animate(frames, { duration, delay: wait0, easing: ease, fill: 'both' });
    return Promise.race([a.finished.catch(() => {}), wait(wait0 + duration + 60)])
      .then(() => { if (!keep) a.cancel(); });
  };
}

// ---------- primitives ----------
export const enter = (el, o = {}) => tween(el, [{ opacity: 0 }, { opacity: 1 }], { ease: EASE.enter, ...o });
export const exit = (el, o = {}) => !el ? NONE : seq([
  tween(el, [{ opacity: getComputedStyle(el).opacity }, { opacity: 0 }], { ms: BEAT * 0.55, ease: EASE.exit, ...o, keep: true }),
  act(() => el.remove()),
]);
export const slide = (el, dx, dy, o = {}) => (Math.abs(dx) < 0.5 && Math.abs(dy) < 0.5 ? NONE
  : tween(el, [{ transform: `translate(${dx}px, ${dy}px)` }, { transform: 'none' }], o));
export const resize = (el, from, to, o = {}) => (Math.abs(from.width - to.width) < 0.5 && Math.abs(from.height - to.height) < 0.5 ? NONE
  : tween(el, [{ width: `${from.width}px`, height: `${from.height}px` }, { width: `${to.width}px`, height: `${to.height}px` }], o));
export const draw = (path, o = {}) => tween(path, [{ strokeDashoffset: 1 }, { strokeDashoffset: 0 }], { ease: EASE.enter, ...o });
export const fly = (el, dx, dy, o = {}) =>
  tween(el, [{ transform: 'none' }, { transform: `translate(${dx}px, ${dy}px)` }], { ...o, keep: true });
export const crossfade = (a, b, o = {}) => all([
  tween(a, [{ opacity: 1 }, { opacity: 0 }], { ms: BEAT * 0.45, ease: EASE.exit, ...o, keep: true }),
  tween(b, [{ opacity: 0 }, { opacity: 1 }], { ms: BEAT * 0.45, ease: EASE.exit, ...o, keep: true }),
]);

// ---------- compositions ----------
export const all = ms => clock => Promise.all(ms.filter(Boolean).map(m => m(clock))).then(() => {});
export const seq = ms => async clock => { for (const m of ms) if (m) await m(clock); };
export const after = (ms, m) => (ms <= 0 ? m : clock => wait(ms / (clock?.speed || 1)).then(() => m(clock)));
export const stagger = (gap, ms) => all(ms.map((m, i) => m && after(i * gap, m)));
export const act = fn => () => Promise.resolve(fn());

// ---------- helpers for FLIP ----------
// A copy of an element, pinned over the page at its own size and centered on `at` (default: where
// the element is), ready to move. The original can then be replaced or removed while the copy plays.
//   hidden   start transparent, for a copy that will fade in
//   mark     classes to put on the copy
// The copy inherits the original's text settings and keeps its exact box, so it is the same
// picture as the original, not a re-layout of it in another context.
const INHERITED = ['font-family', 'font-size', 'font-weight', 'font-style', 'font-variant-ligatures', 'line-height', 'letter-spacing', 'white-space', 'color'];
export function ghost(el, { at, hidden = false, mark = [] } = {}) {
  const r = el.getBoundingClientRect(), c = at ? center(at) : center(r), cs = getComputedStyle(el);
  const w = document.createElement('div');
  w.className = 'redex rdx-ghost';
  w.style.cssText = `left:${c[0] - r.width / 2}px;top:${c[1] - r.height / 2}px;width:${r.width}px;height:${r.height}px;`
    + INHERITED.map(p => `${p}:${cs.getPropertyValue(p)}`).join(';') + (hidden ? ';opacity:0' : '');
  const copy = el.cloneNode(true);
  copy.classList.remove('hot', 'soft', 'arriving');
  copy.classList.add(...mark);
  copy.style.width = `${r.width}px`; copy.style.height = `${r.height}px`; copy.style.flex = 'none';
  w.appendChild(copy);
  document.body.appendChild(w);
  return w;
}
const center = r => [r.left + r.width / 2, r.top + r.height / 2];
// Offset from rect a's center to rect b's center.
export const delta = (a, b) => { const [ax, ay] = center(a), [bx, by] = center(b); return [bx - ax, by - ay]; };
