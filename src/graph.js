// The reduction graph: every term reachable from the start, one edge per redex.
// Layered by longest path, so every finished path flows down to the normal form.
import { GLYPH, KCLS, esc, exploreGraph, key, show, text } from './engine.js';
import { $$, motion, nextId } from './dom.js';

const CAP = 70;                                  // stop exploring after this many terms
const CW = 6.9, NH = 28, PADX = 11, GAP = 18;    // char width, node height, node padding, gap
const ROWH = 70, M = 26, RMARG = 90;             // row pitch, margin, room for loop-back edges
const KINDS = ['alpha', 'beta', 'eta', 'delta', 'zeta', 'iota', 'proj'];

export function mountGraph(root, session) {
  const uid = `rdx${nextId()}`;
  root.innerHTML = `
    <div class="rdx-gbar"><span class="rdx-gstats"></span><span class="rdx-grow"></span>
      <label class="rdx-toggle"><input type="checkbox" data-opt="eta"> Include η</label>
      <button type="button" class="rdx-btn" data-act="rebuild">Rebuild from current term</button></div>
    <p class="rdx-verdict"></p>
    <div class="rdx-gscroll"><svg class="rdx-gsvg" role="img" aria-label="Reduction graph"></svg></div>
    <div class="rdx-glegend"><span><i class="rdx-sw start"></i>start</span><span><i class="rdx-sw nf"></i>normal form</span><span><i class="rdx-sw here"></i>workbench is here</span><span><i class="rdx-sw open"></i>not explored</span><span>thick edge: the path you took</span></div>
    <div class="rdx-gsel"></div>`;
  const q = s => root.querySelector(s);
  const svg = q('.rdx-gsvg'), etaBox = q('[data-opt="eta"]');
  let G = null;

  function build(start) {
    G = exploreGraph(start, session.env, etaBox.checked, CAP);
    G.sel = null;
    draw();
  }

  function draw() {
    const { nodes, edges } = G;
    const list = [...nodes.values()];
    list.forEach((n, i) => {
      n.id = i; n.label = text(n.term);
      n.short = n.label.length > 36 ? n.label.slice(0, 35) + '…' : n.label;
      n.layer = 0; n.w = Math.round(n.short.length * CW + 2 * PADX);
    });
    G.byId = list;

    // Back edges (cycles) via DFS, then longest-path layers over the remaining DAG.
    const out = new Map(list.map(n => [n.key, []]));
    edges.forEach(e => { e.back = false; out.get(e.from).push(e); });
    const state = new Map(), post = [];
    const dfs = k => {
      state.set(k, 1);
      for (const e of out.get(k)) { const s = state.get(e.to); if (s === 1) e.back = true; else if (!s) dfs(e.to); }
      state.set(k, 2); post.push(k);
    };
    dfs(G.root);
    for (const k of post.reverse()) for (const e of out.get(k)) {
      if (!e.back) { const b = nodes.get(e.to); b.layer = Math.max(b.layer, nodes.get(k).layer + 1); }
    }

    // Rows, ordered by the mean x of each node's parents.
    const layers = [];
    list.forEach(n => (layers[n.layer] ||= []).push(n));
    const rowW = layers.map(r => r.reduce((s, n) => s + n.w, 0) + GAP * (r.length - 1));
    const W = Math.max(...rowW);
    const ins = new Map(list.map(n => [n.key, []]));
    edges.forEach(e => { if (!e.back) ins.get(e.to).push(nodes.get(e.from)); });
    layers.forEach((row, li) => {
      if (li > 0) {
        row.forEach(n => { const ps = ins.get(n.key); n.bc = ps.length ? ps.reduce((s, p) => s + p.x, 0) / ps.length : 0; });
        row.sort((a, b) => a.bc - b.bc);
      }
      let x = M + (W - rowW[li]) / 2;
      row.forEach(n => { n.x = x + n.w / 2; n.y = M + li * ROWH + NH / 2; x += n.w + GAP; });
    });
    const width = W + 2 * M + RMARG, height = 2 * M + (layers.length - 1) * ROWH + NH;

    const pairs = new Map();
    edges.forEach(e => { const k = e.from + '|' + e.to; e.pi = pairs.get(k) || 0; pairs.set(k, e.pi + 1); });
    let es = '';
    for (const e of edges) {
      const a = nodes.get(e.from), b = nodes.get(e.to);
      const off = (e.pi - (pairs.get(e.from + '|' + e.to) - 1) / 2) * 18;
      let d, lx, ly;
      if (a === b) {
        const xr = a.x + a.w / 2, y = a.y;
        d = `M${xr} ${y - 7} C${xr + 48} ${y - 34} ${xr + 48} ${y + 34} ${xr + 3} ${y + 8}`;
        lx = xr + 36; ly = y;
      } else if (e.back) {
        const x1 = a.x + a.w / 2, x2 = b.x + b.w / 2 + 3, bx = Math.max(x1, x2) + 56 + off;
        d = `M${x1} ${a.y} C${bx} ${a.y} ${bx} ${b.y} ${x2} ${b.y}`;
        lx = (x1 + 6 * bx + x2) / 8; ly = (a.y + b.y) / 2;
      } else {
        const x1 = a.x + off * 0.4, y1 = a.y + NH / 2, x2 = b.x + off * 0.4, y2 = b.y - NH / 2 - 3;
        const c = Math.max(24, (y2 - y1) * 0.5);
        d = `M${x1} ${y1} C${x1 + off} ${y1 + c} ${x2 + off} ${y2 - c} ${x2} ${y2}`;
        lx = (x1 + 3 * (x1 + off) + 3 * (x2 + off) + x2) / 8; ly = (y1 + 3 * (y1 + c) + 3 * (y2 - c) + y2) / 8;
      }
      es += `<g class="rdx-ge k-${KCLS(e.kind)}" data-from="${a.id}" data-to="${b.id}" style="--d:${a.layer}"><path pathLength="1" d="${d}" marker-end="url(#${uid}-${KCLS(e.kind)})"/><circle cx="${lx.toFixed(1)}" cy="${ly.toFixed(1)}" r="8.5"/><text x="${lx.toFixed(1)}" y="${ly.toFixed(1)}">${GLYPH[e.kind]}</text></g>`;
    }
    let ns = '';
    for (const n of list) {
      const cls = ['rdx-gn', n.key === G.root && 'start', n.nf && 'nf', n.open && 'open', G.sel === n.id && 'sel'].filter(Boolean).join(' ');
      ns += `<g class="${cls}" data-id="${n.id}" style="--d:${n.layer}" tabindex="0" role="button" aria-label="${esc(n.label)}"><title>${esc(n.label)}</title><rect x="${(n.x - n.w / 2).toFixed(1)}" y="${(n.y - NH / 2).toFixed(1)}" width="${n.w}" height="${NH}" rx="2"/><text x="${n.x.toFixed(1)}" y="${n.y.toFixed(1)}">${esc(n.short)}</text></g>`;
    }
    const markers = KINDS.map(k => `<marker id="${uid}-${k}" viewBox="0 0 10 10" refX="8.5" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0L10 5L0 10z" style="fill:var(--rdx-${k})"/></marker>`).join('');
    svg.setAttribute('width', width); svg.setAttribute('height', height);
    svg.setAttribute('viewBox', `0 0 ${width} ${height}`);
    svg.classList.toggle('animate', motion());
    svg.innerHTML = `<defs>${markers}</defs>${es}${ns}`;

    const nf = list.filter(n => n.nf).length, cyclic = edges.some(e => e.back);
    q('.rdx-gstats').textContent = `${list.length} term${list.length > 1 ? 's' : ''} · ${edges.length} step${edges.length === 1 ? '' : 's'} · ${nf} normal form${nf === 1 ? '' : 's'}${G.truncated ? ` · stopped at ${CAP} terms` : ''}`;
    let v = '';
    if (nf === 1) v = G.truncated ? 'Every finished path so far ends at the same normal form.' : 'Every path that finishes ends at the same normal form.';
    if (cyclic) v += nf ? ' Some paths loop instead of finishing.' : 'No normal form: every path keeps going.';
    if (!edges.length) v = 'Already in normal form. Nothing to fire.';
    q('.rdx-verdict').textContent = v.trim();
    renderSel();
    mark();
  }

  // Highlight the workbench's current term and the edges it walked.
  function mark() {
    if (!G?.byId) return;
    const idOf = t => G.nodes.get(key(t))?.id ?? -1;
    const here = idOf(session.term);
    $$('.rdx-gn', svg).forEach(g => g.classList.toggle('here', +g.dataset.id === here));
    const walked = new Set();
    for (let i = 1; i <= session.idx; i++) {
      const a = idOf(session.hist[i - 1].term), b = idOf(session.hist[i].term);
      if (a >= 0 && b >= 0 && a !== b) walked.add(a + '>' + b);
    }
    $$('.rdx-ge', svg).forEach(g => g.classList.toggle('walked', walked.has(g.dataset.from + '>' + g.dataset.to)));
  }

  function renderSel() {
    const box = q('.rdx-gsel');
    if (G.sel == null) { box.innerHTML = '<p>Click a term in the graph to see it in full and open it in the workbench.</p>'; return; }
    const n = G.byId[G.sel];
    const where = n.depth === 0 ? 'Start term' : `${n.depth} step${n.depth > 1 ? 's' : ''} from the start on the shortest path`;
    box.innerHTML = `<div class="rdx-tm">${show(n.term, { html: true, env: session.env })}</div><div class="rdx-row"><span>${where}</span>${n.nf ? '<b class="rdx-nf">Normal form</b>' : ''}<span class="rdx-grow"></span><button class="rdx-btn primary" type="button" data-act="open">Open in workbench</button></div>`;
  }
  function select(id) {
    G.sel = id;
    $$('.rdx-gn', svg).forEach(g => g.classList.toggle('sel', +g.dataset.id === id));
    renderSel();
  }
  function open(id) {
    if (session.busy) return;
    const chain = [];
    for (let k = G.byId[id].key; k;) { const m = G.nodes.get(k); chain.unshift(m); k = m.parent; }
    session.replaceHistory(chain.map((m, i) => ({ term: m.term, kind: i ? m.pkind : null })));
    document.querySelector('[data-redex="workbench"]')?.scrollIntoView({ behavior: motion() ? 'smooth' : 'auto', block: 'start' });
  }

  svg.addEventListener('click', e => { const g = e.target.closest('.rdx-gn'); if (g) select(+g.dataset.id); });
  svg.addEventListener('keydown', e => {
    const g = e.target.closest('.rdx-gn');
    if (g && (e.key === 'Enter' || e.key === ' ')) { e.preventDefault(); select(+g.dataset.id); }
  });
  root.addEventListener('click', e => {
    const act = e.target.closest('[data-act]')?.dataset.act;
    if (act === 'rebuild') build(session.term);
    if (act === 'open' && G.sel != null) open(G.sel);
  });
  etaBox.addEventListener('change', () => build(G.nodes.get(G.root).term));

  session.on(type => {
    if (type === 'load') { etaBox.checked = session.eta; build(session.start); }
    else mark();
  });
}
