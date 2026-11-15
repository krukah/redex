// DOM helpers shared by the widgets: motion, binding arcs, the flying-argument animation.
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
  const base = stage.getBoundingClientRect();
  let out = '';
  for (const v of line.querySelectorAll('[data-bid]')) {
    const id = v.dataset.bid, anchor = line.querySelector(`[data-anchor="${id}"]`);
    if (!anchor) continue;
    const a = anchor.getBoundingClientRect(), b = v.getBoundingClientRect();
    const x1 = a.left + a.width / 2 - base.left, x2 = b.left + b.width / 2 - base.left;
    const y = Math.min(a.top, b.top) - base.top + 3;
    const h = Math.min(56, 14 + Math.abs(x2 - x1) * 0.2);
    const cap = v.classList.contains('cap') ? ' cap' : '';
    out += `<path class="rarc${cap}" data-id="${id}" d="M${x1.toFixed(1)} ${y.toFixed(1)} Q${((x1 + x2) / 2).toFixed(1)} ${(y - 2 * h).toFixed(1)} ${x2.toFixed(1)} ${y.toFixed(1)}"/>`
      + `<circle class="rarc-end${cap}" data-id="${id}" cx="${x2.toFixed(1)}" cy="${y.toFixed(1)}" r="2.4"/>`;
  }
  svg.innerHTML = out;
}

export function onScreen(el) {
  const r = el.getBoundingClientRect();
  return r.bottom > 0 && r.top < innerHeight && r.width > 0;
}

// Clone `src` once per target and fly each clone along an arc into its target.
// With no targets, `src` shrinks away instead: the argument was discarded.
export function fly(src, targets, speed, keep) {
  if (!src) return wait(300 / speed);
  const sr = src.getBoundingClientRect();
  if (!targets.length) {
    src.style.display = 'inline-block';
    return src.animate(
      [{ opacity: 1, transform: 'none' }, { opacity: 0, transform: 'translateY(12px) scale(.6)' }],
      { duration: 520 / speed, easing: 'ease-in', fill: 'forwards' },
    ).finished;
  }
  if (!keep) src.classList.add('leaving');
  const fontSize = getComputedStyle(src).fontSize;
  return Promise.all(targets.map((tg, i) => {
    tg.classList.add('landing');
    const tr = tg.getBoundingClientRect();
    const w = document.createElement('div');
    w.className = 'redex rdx-flyer rdx-tm';
    w.style.cssText = `left:${sr.left}px;top:${sr.top}px;width:${sr.width}px;height:${sr.height}px;font-size:${fontSize}`;
    const c = src.cloneNode(true);
    c.classList.remove('leaving', 'firing', 'hot');
    w.appendChild(c);
    document.body.appendChild(w);
    const dx = tr.left + tr.width / 2 - (sr.left + sr.width / 2);
    const dy = tr.top + tr.height / 2 - (sr.top + sr.height / 2);
    const sc = Math.max(0.35, Math.min(1, (tr.width * 1.8) / sr.width));
    const lift = -Math.min(70, 26 + Math.abs(dx) * 0.15);
    const an = w.animate([
      { transform: 'translate(0,0) scale(1)', opacity: 0.97 },
      { transform: `translate(${dx / 2}px, ${dy / 2 + lift}px) scale(${(1 + sc) / 2})`, opacity: 0.97, offset: 0.5 },
      { transform: `translate(${dx}px, ${dy}px) scale(${sc})`, opacity: 0.2 },
    ], { duration: 720 / speed, delay: (i * 100) / speed, easing: 'cubic-bezier(.45,.05,.3,1)', fill: 'both' });
    return an.finished.then(() => w.remove(), () => w.remove());
  }));
}
