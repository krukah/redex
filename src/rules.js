// The rule rail and the example tiles. Clicking either loads its example into the session.
//
// Each tile carries a thumbnail of the example's reduction graph in the graph widget's grammar
// (○ reducible, ◇ weak head normal form, ■ normal form, edges in rule ink, ring on the start),
// so the shape of the whole reduction is visible before the example is loaded.
import { GLYPH, KCLS, esc, exploreGraph, hasCycle, normalNext, parseProgram, runStrategy, whnfNext } from './engine.js';
import { RULES, MORE } from './examples.js';

const TW = 120, TH = 46, PAD = 6, ROWS = 4, MR = 3;   // thumbnail box, rows shown, mark radius

function thumb(ex) {
  const { term, env, notes } = parseProgram(ex.src);
  const G = exploreGraph(term, env, !!ex.eta, 40);
  const list = [...G.nodes.values()];
  const shown = list.filter(n => n.depth <= ROWS);
  const rows = [];
  shown.forEach(n => (rows[n.depth] ||= []).push(n));
  const depth = rows.length - 1, wide = Math.max(...rows.map(r => r.length));
  const dy = depth ? Math.min(16, (TH - 2 * PAD) / depth) : 0;
  const dx = wide > 1 ? Math.min(22, (TW - 2 * PAD - 14) / (wide - 1)) : 0;
  const y0 = (TH - depth * dy) / 2;
  const parents = new Map(shown.map(n => [n.key, []]));
  G.edges.forEach(e => {
    const a = G.nodes.get(e.from), b = G.nodes.get(e.to);
    if (b.depth === a.depth + 1 && parents.has(b.key)) parents.get(b.key).push(a);
  });
  rows.forEach((row, d) => {
    if (d) {
      row.forEach(n => { const ps = parents.get(n.key); n.bc = ps.length ? ps.reduce((s, p) => s + p.tx, 0) / ps.length : 0; });
      row.sort((a, b) => a.bc - b.bc);
    }
    const x0 = TW / 2 - 7 - ((row.length - 1) * dx) / 2;
    row.forEach((n, i) => { n.tx = x0 + i * dx; n.ty = y0 + d * dy; });
  });

  const f = v => v.toFixed(1);
  let edges = '';
  for (const e of G.edges) {
    const a = G.nodes.get(e.from), b = G.nodes.get(e.to), k = KCLS(e.kind);
    if (a.depth > ROWS) continue;
    let d, cut = '';
    if (b.depth > ROWS) {        // leaves the thumbnail: a short dashed stub downward
      d = `M${f(a.tx)} ${f(a.ty + MR)} L${f(a.tx)} ${f(a.ty + MR + 6)}`; cut = ' cut';
    } else if (a === b) {
      d = `M${f(a.tx + 2)} ${f(a.ty - 2)} C${f(a.tx + 9)} ${f(a.ty - 13)} ${f(a.tx + 16)} ${f(a.ty - 2)} ${f(a.tx + MR + 1)} ${f(a.ty + 1)}`;
    } else if (b.depth === a.depth + 1) {
      d = `M${f(a.tx)} ${f(a.ty + MR)} L${f(b.tx)} ${f(b.ty - MR)}`;
    } else if (b.depth === a.depth) {
      const lift = 5 + Math.abs(b.tx - a.tx) * 0.25;
      d = `M${f(a.tx)} ${f(a.ty - MR)} Q${f((a.tx + b.tx) / 2)} ${f(a.ty - lift)} ${f(b.tx)} ${f(b.ty - MR)}`;
    } else {                     // back to an earlier row: swing out to the right
      const bx = Math.max(a.tx, b.tx) + 10 + Math.abs(a.ty - b.ty) * 0.3;
      d = `M${f(a.tx + MR)} ${f(a.ty)} C${f(bx)} ${f(a.ty)} ${f(bx)} ${f(b.ty)} ${f(b.tx + MR)} ${f(b.ty)}`;
    }
    edges += `<path class="te k-${k}${cut}" d="${d}"/>`;
  }
  const marks = shown.map(n => {
    const open = n.open ? ' open' : '', x = n.tx, y = n.ty;
    const ring = n.key === G.root ? `<circle class="ring" cx="${f(x)}" cy="${f(y)}" r="${MR + 2.4}"/>` : '';
    if (n.nf) return ring + `<rect class="m nf" x="${f(x - MR)}" y="${f(y - MR)}" width="${2 * MR}" height="${2 * MR}"/>`;
    if (!whnfNext(n.term, env)) {
      const r = MR + 0.8;
      return ring + `<path class="m whnf${open}" d="M${f(x)} ${f(y - r)}L${f(x + r)} ${f(y)}L${f(x)} ${f(y + r)}L${f(x - r)} ${f(y)}Z"/>`;
    }
    return ring + `<circle class="m red${open}" cx="${f(x)}" cy="${f(y)}" r="${MR}"/>`;
  }).join('');

  const run = runStrategy(term, env, normalNext, 60), cyclic = hasCycle(G), n = run.rows.length - 1;
  const steps = `${n} step${n === 1 ? '' : 's'}`;
  const nf = '<i class="rdx-nfmark" aria-hidden="true"></i><span class="rdx-sr">normal form</span>';
  const fate = run.status === 'done' ? `${nf} in ${steps}${cyclic ? ', or loops' : ''}`
    : cyclic ? 'loops forever' : 'keeps growing';
  const svg = `<svg class="rdx-thumb" viewBox="0 0 ${TW} ${TH}" aria-hidden="true">${edges}${marks}</svg>`;
  return { svg, fate, caption: notes[0] || '' };
}

export function mountRules(root, session) {
  root.innerHTML = `
    <div class="rdx-label">Rules</div>
    <ol class="rdx-rail">${RULES.map(r => `
      <li><button type="button" class="rdx-ri k-${r.k}" data-ex="${r.id}" aria-pressed="false">
        <span class="rdx-g" aria-hidden="true">${GLYPH[r.k]}</span>
        <span class="rdx-rn"><b>${r.name}</b> <span class="rdx-sub">${r.sub}</span></span>
        <code>${esc(r.schema)}</code>
      </button></li>`).join('')}
    </ol>
    <div class="rdx-label">More examples</div>
    <div class="rdx-tiles">${MORE.map(m => {
      const t = thumb(m);
      return `<button type="button" class="rdx-tile" data-ex="${m.id}" aria-pressed="false" title="${esc(t.caption)}">
        ${t.svg}<b>${esc(m.label)}</b><span class="rdx-sub">${t.fate}</span></button>`;
    }).join('')}
    </div>`;

  root.addEventListener('click', e => {
    const hit = e.target.closest('[data-ex]');
    if (hit && !session.busy) session.loadExample(hit.dataset.ex);
  });
  session.on(type => {
    if (type !== 'load') return;
    root.querySelectorAll('[data-ex]').forEach(el => {
      const on = el.dataset.ex === session.ex;
      el.classList.toggle('on', on);
      el.setAttribute('aria-pressed', String(on));
    });
  });
}
