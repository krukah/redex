// DOM helpers shared by the widgets: motion, ids, rule letters, binding arcs.
import { GLYPH, KCLS } from './engine.js';

export const $$ = (sel, root) => [...root.querySelectorAll(sel)];
const reduced = matchMedia('(prefers-reduced-motion: reduce)');
export const motion = () => !reduced.matches;
export const wait = ms => new Promise(r => setTimeout(r, ms));

let uid = 0;
export const nextId = () => ++uid;

// A rule letter in running UI text (trace, status, race lanes).
export const markFlat = k => (k
  ? `<span class="rm flat k-${KCLS(k)}">${GLYPH[k]}</span>`
  : '<span class="rm flat k-none">·</span>');

// One arc from each bound occurrence up to its binder.
export function drawArcs(stage, line, on) {
  const svg = stage.querySelector('.rdx-arcs');
  if (!on) { svg.innerHTML = ''; return; }
  const base = stage.getBoundingClientRect(), ceiling = line.getBoundingClientRect().top - base.top - 30;
  const f = n => n.toFixed(1);
  let out = '';
  for (const v of line.querySelectorAll('[data-bid]')) {
    const id = v.dataset.bid, anchor = line.querySelector(`[data-anchor="${id}"]`);
    if (!anchor) continue;
    const a = anchor.getBoundingClientRect(), b = v.getBoundingClientRect();
    const x1 = a.left + a.width / 2 - base.left, x2 = b.left + b.width / 2 - base.left;
    const y1 = a.top - base.top + 2, y2 = b.top - base.top + 2;
    // Both ends rise to a shared apex; longer links rise higher, never past the ceiling.
    const apex = Math.max(ceiling, Math.min(y1, y2) - 10 - Math.abs(x2 - x1) * 0.12);
    const cap = v.classList.contains('cap') ? ' cap' : '';
    out += `<path class="rarc${cap}" data-id="${id}" d="M${f(x1)} ${f(y1)} C${f(x1)} ${f(apex)} ${f(x2)} ${f(apex)} ${f(x2)} ${f(y2)}"/>`
      + `<circle class="rarc-end${cap}" data-id="${id}" cx="${f(x2)}" cy="${f(y2)}" r="2.2"/>`;
  }
  svg.innerHTML = out;
}
