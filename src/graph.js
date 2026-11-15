// The reduction graph: every term reachable from the start, one edge per redex.
//
// Visual grammar
//   row        exact number of steps from the start (shortest path)
//   shape      ○ reducible   ◇ weak head normal form   ■ normal form
//   ring       the start term; a blue halo marks the workbench's current term
//   number     term size (node count), beside each mark
//   edge       colored by the rule that fired; edges between rows point down, edges within a
//              row arc above it, edges back to an earlier row swing out to the right
//   route      an optional strategy drawn as a wide band along the edges it takes
import { KCLS, esc, exploreGraph, key, applicativeNext, normalNext, runStrategy, size, text, whnfNext } from './engine.js';
import { renderBlocks } from './blocks.js';
import { $$, motion, nextId } from './dom.js';

const CAP = 70;                        // stop exploring after this many terms
const GX = 46, ROWH = 58, R = 6;       // column pitch, row pitch, mark radius
const ML = 54, MT = 30, MR = 96;       // margins: axis on the left, loop-backs on the right
const KINDS = ['alpha', 'beta', 'eta', 'delta', 'zeta', 'iota', 'proj'];
const ROUTES = [
  { id: 'none', label: 'None' },
  { id: 'normal', label: 'Normal order', next: normalNext },
  { id: 'applicative', label: 'Applicative order', next: applicativeNext },
  { id: 'whnf', label: 'Head only', next: whnfNext },
];
const FORM_LABEL = { red: 'reducible', whnf: 'weak head normal form', nf: 'normal form' };

function hasCycle(G) {
  const out = new Map([...G.nodes.keys()].map(k => [k, []]));
  G.edges.forEach(e => out.get(e.from).push(e.to));
  const state = new Map();
  const dfs = k => {
    state.set(k, 1);
    for (const to of out.get(k)) { const s = state.get(to); if (s === 1 || (!s && dfs(to))) return true; }
    state.set(k, 2);
    return false;
  };
  return dfs(G.root);
}

export function mountGraph(root, session) {
  const uid = `rdx${nextId()}`;
  root.innerHTML = `
    <div class="rdx-gbar"><span class="rdx-gstats"></span><span class="rdx-grow"></span>
      <label class="rdx-toggle"><input type="checkbox" data-opt="eta"> Include η</label>
      <button type="button" class="rdx-btn" data-act="rebuild">Rebuild from current term</button></div>
    <div class="rdx-gbar"><span class="rdx-label">Route</span><div class="rdx-pills" data-routes></div><span class="rdx-route-note"></span></div>
    <p class="rdx-verdict"></p>
    <div class="rdx-gwrap"><div class="rdx-gscroll"><svg class="rdx-gsvg" role="img" aria-label="Reduction graph"></svg></div><div class="rdx-tip" hidden></div></div>
    <div class="rdx-glegend">
      <span><svg width="14" height="14"><circle cx="7" cy="7" r="5" class="m red"/></svg>reducible</span>
      <span><svg width="14" height="14"><path d="M7 1.5L12.5 7L7 12.5L1.5 7Z" class="m whnf"/></svg>weak head normal form</span>
      <span><svg width="14" height="14"><rect x="2" y="2" width="10" height="10" class="m nf"/></svg>normal form</span>
      <span><svg width="18" height="18"><circle cx="9" cy="9" r="4.5" class="m red"/><circle cx="9" cy="9" r="8" class="ring"/></svg>start</span>
      <span><svg width="18" height="18"><circle cx="9" cy="9" r="4.5" class="m red"/><circle cx="9" cy="9" r="8" class="halo"/></svg>workbench is here</span>
      <span><b class="rdx-sz">12</b> term size</span>
    </div>
    <div class="rdx-gsel"></div>`;
  const q = s => root.querySelector(s);
  const svg = q('.rdx-gsvg'), tip = q('.rdx-tip'), etaBox = q('[data-opt="eta"]');
  let G = null, route = 'none';

  function build(start) {
    G = exploreGraph(start, session.env, etaBox.checked, CAP);
    G.sel = null;
    draw();
  }

  function draw() {
    const { nodes, edges } = G;
    const list = [...nodes.values()];
    list.forEach((n, i) => {
      n.id = i; n.label = text(n.term); n.size = size(n.term);
      n.form = n.nf ? 'nf' : whnfNext(n.term, session.env) ? 'red' : 'whnf';
    });
    G.byId = list;

    // Rows are BFS depth. Within a row, order by the mean x of parents one row up.
    const rows = [];
    list.forEach(n => (rows[n.depth] ||= []).push(n));
    const ins = new Map(list.map(n => [n.key, []]));
    edges.forEach(e => { const a = nodes.get(e.from), b = nodes.get(e.to); if (b.depth === a.depth + 1) ins.get(b.key).push(a); });
    const span = Math.max(...rows.map(r => r.length - 1)) * GX;
    rows.forEach((row, d) => {
      if (d > 0) {
        row.forEach(n => { const ps = ins.get(n.key); n.bc = ps.length ? ps.reduce((s, p) => s + p.x, 0) / ps.length : 0; });
        row.sort((a, b) => a.bc - b.bc);
      }
      const x0 = ML + (span - (row.length - 1) * GX) / 2;
      row.forEach((n, i) => { n.x = x0 + i * GX; n.y = MT + d * ROWH; });
    });
    const width = ML + span + MR, height = MT * 2 + (rows.length - 1) * ROWH;

    // Edge geometry, shared by the edges and the route bands.
    const pairs = new Map();
    edges.forEach(e => { const k = e.from + '|' + e.to; e.pi = pairs.get(k) || 0; pairs.set(k, e.pi + 1); });
    for (const e of edges) {
      const a = nodes.get(e.from), b = nodes.get(e.to);
      const off = (e.pi - (pairs.get(e.from + '|' + e.to) - 1) / 2) * 12;
      if (a === b) {
        e.d = `M${a.x + 4} ${a.y - 5} C${a.x + 20} ${a.y - 34} ${a.x + 40} ${a.y - 10} ${a.x + R + 3} ${a.y + 1}`;
      } else if (b.depth === a.depth + 1) {
        const y1 = a.y + R + 1, y2 = b.y - R - 3, c = (y2 - y1) * 0.5;
        e.d = `M${a.x} ${y1} C${a.x + off} ${y1 + c} ${b.x + off} ${y2 - c} ${b.x} ${y2}`;
      } else if (b.depth === a.depth) {
        const lift = 16 + Math.abs(b.x - a.x) * 0.18 + off;
        const dir = Math.sign(b.x - a.x) || 1;
        e.d = `M${a.x + dir * 3} ${a.y - R} C${a.x + dir * 6} ${a.y - lift} ${b.x - dir * 6} ${b.y - lift} ${b.x - dir * 2} ${b.y - R - 3}`;
      } else {
        const bx = Math.max(a.x, b.x) + 34 + Math.abs(a.y - b.y) * 0.12 + off;
        e.d = `M${a.x + R + 1} ${a.y} C${bx} ${a.y} ${bx} ${b.y} ${b.x + R + 3} ${b.y}`;
      }
    }

    const grid = rows.map((_, d) => `<line class="rdx-gridline" x1="${ML - 18}" x2="${width - 12}" y1="${MT + d * ROWH}" y2="${MT + d * ROWH}"/><text class="rdx-ax" x="${ML - 26}" y="${MT + d * ROWH}">${d}</text>`).join('')
      + `<text class="rdx-ax head" x="${ML - 26}" y="${MT - 18}">steps</text>`;
    const es = edges.map(e => `<path class="rdx-ge k-${KCLS(e.kind)}" data-from="${nodes.get(e.from).id}" data-to="${nodes.get(e.to).id}" style="--d:${nodes.get(e.from).depth}" pathLength="1" d="${e.d}" marker-end="url(#${uid}-${KCLS(e.kind)})"><title>${KCLS(e.kind) === 'proj' ? 'π' : e.kind}</title></path>`).join('');
    const ns = list.map(n => {
      const mark = n.form === 'nf' ? `<rect class="m nf" x="${n.x - R}" y="${n.y - R}" width="${2 * R}" height="${2 * R}"/>`
        : n.form === 'whnf' ? `<path class="m whnf" d="M${n.x} ${n.y - R - 1.5}L${n.x + R + 1.5} ${n.y}L${n.x} ${n.y + R + 1.5}L${n.x - R - 1.5} ${n.y}Z"/>`
        : `<circle class="m red" cx="${n.x}" cy="${n.y}" r="${R}"/>`;
      const cls = ['rdx-gn', n.key === G.root && 'start', n.open && 'open', G.sel === n.id && 'sel'].filter(Boolean).join(' ');
      return `<g class="${cls}" data-id="${n.id}" style="--d:${n.depth}" tabindex="0" role="button" aria-label="${esc(`${FORM_LABEL[n.form]}, size ${n.size}: ${n.label}`)}">`
        + `<circle class="hit" cx="${n.x}" cy="${n.y}" r="15"/><circle class="halo" cx="${n.x}" cy="${n.y}" r="${R + 6}"/>`
        + (n.key === G.root ? `<circle class="ring" cx="${n.x}" cy="${n.y}" r="${R + 4}"/>` : '')
        + `${mark}<text class="rdx-sz" x="${n.x + R + 5}" y="${n.y + 9}">${n.size}</text></g>`;
    }).join('');
    const markers = KINDS.map(k => `<marker id="${uid}-${k}" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="5.5" markerHeight="5.5" orient="auto-start-reverse"><path d="M0 0L10 5L0 10z" style="fill:var(--rdx-${k})"/></marker>`).join('');
    svg.setAttribute('width', width); svg.setAttribute('height', height);
    svg.setAttribute('viewBox', `0 0 ${width} ${height}`);
    svg.classList.toggle('animate', motion());
    svg.innerHTML = `<defs>${markers}</defs><g class="grid">${grid}</g><g class="routes"></g><g class="edges">${es}</g><g class="nodes">${ns}</g>`;

    const nf = list.filter(n => n.nf).length, cyclic = hasCycle(G);
    q('.rdx-gstats').textContent = `${list.length} term${list.length > 1 ? 's' : ''} · ${edges.length} step${edges.length === 1 ? '' : 's'} · ${nf} normal form${nf === 1 ? '' : 's'}${G.truncated ? ` · stopped at ${CAP} terms` : ''}`;
    let v = '';
    if (nf === 1) v = G.truncated ? 'Every finished path so far ends at the same normal form.' : 'Every path that finishes ends at the same normal form.';
    if (cyclic) v += nf ? ' Some paths loop instead of finishing.' : 'No normal form: every path keeps going.';
    if (!edges.length) v = 'Already in normal form. Nothing to fire.';
    q('.rdx-verdict').textContent = v.trim();
    renderRoutes(); renderSel(); mark();
  }

  // ---------- strategy routes ----------
  function renderRoutes() {
    q('[data-routes]').innerHTML = ROUTES.map(r => `<button type="button" class="rdx-pill${route === r.id ? ' on' : ''}" data-route="${r.id}" aria-pressed="${route === r.id}">${r.label}</button>`).join('');
    const band = svg.querySelector('.routes'), note = q('.rdx-route-note');
    band.innerHTML = ''; note.textContent = '';
    const r = ROUTES.find(x => x.id === route);
    if (!r.next) return;
    const start = G.nodes.get(G.root).term;
    const run = runStrategy(start, session.env, r.next, 40);
    let paths = '', steps = 0, left = false;
    for (let i = 1; i < run.rows.length; i++) {
      const a = G.nodes.get(key(run.rows[i - 1].term)), b = G.nodes.get(key(run.rows[i].term));
      if (!a || !b) { left = true; break; }
      const e = G.edges.find(e => e.from === a.key && e.to === b.key && e.kind === run.rows[i].kind) || G.edges.find(e => e.from === a.key && e.to === b.key);
      if (e) paths += `<path class="rdx-band" d="${e.d}"/>`;
      steps = i;
    }
    const end = G.nodes.get(key(run.rows[Math.min(steps, run.rows.length - 1)].term));
    if (end && run.status === 'done' && !left) paths += `<circle class="rdx-band-end" cx="${end.x}" cy="${end.y}" r="${R + 9}"/>`;
    band.innerHTML = paths;
    note.textContent = left ? `${r.label} leaves the explored graph after ${steps} steps.`
      : run.status === 'done' ? `${r.label}: ${steps} step${steps === 1 ? '' : 's'} to a ${run.rows.length && FORM_LABEL[end?.form] || 'stop'}.`
      : `${r.label} never stops: it loops.`;
  }

  // ---------- workbench position ----------
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

  // ---------- hover and selection ----------
  function showTip(g) {
    if (!g) { tip.hidden = true; return; }
    const n = G.byId[+g.dataset.id];
    tip.innerHTML = `<div class="rdx-tip-meta"><b>${FORM_LABEL[n.form]}</b> · size ${n.size} · ${n.depth} step${n.depth === 1 ? '' : 's'} from the start</div><div class="rdx-tm">${esc(n.label)}</div>`;
    tip.hidden = false;
    const wrap = q('.rdx-gwrap').getBoundingClientRect(), r = g.querySelector('.hit').getBoundingClientRect();
    const left = Math.min(Math.max(8, r.left - wrap.left + r.width / 2 - tip.offsetWidth / 2), wrap.width - tip.offsetWidth - 8);
    const below = r.top - wrap.top < tip.offsetHeight + 12;
    tip.style.left = `${left}px`;
    tip.style.top = `${below ? r.bottom - wrap.top + 6 : r.top - wrap.top - tip.offsetHeight - 6}px`;
  }
  function renderSel() {
    const box = q('.rdx-gsel');
    if (G.sel == null) { box.innerHTML = '<p>Hover a mark to read its term. Click it to open it in the workbench.</p>'; return; }
    const n = G.byId[G.sel];
    const where = n.depth === 0 ? 'Start term' : `${n.depth} step${n.depth > 1 ? 's' : ''} from the start on the shortest path`;
    box.innerHTML = `<div class="rdx-scroll"><div class="rdx-stage small flat rdx-tm">${renderBlocks(n.term, { env: session.env })}</div></div><div class="rdx-row"><span>${where} · ${FORM_LABEL[n.form]} · size ${n.size}</span><span class="rdx-grow"></span><button class="rdx-btn primary" type="button" data-act="open">Open in workbench</button></div>`;
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

  svg.addEventListener('mouseover', e => showTip(e.target.closest('.rdx-gn')));
  svg.addEventListener('mouseleave', () => showTip(null));
  svg.addEventListener('focusin', e => showTip(e.target.closest('.rdx-gn')));
  svg.addEventListener('focusout', () => showTip(null));
  svg.addEventListener('click', e => { const g = e.target.closest('.rdx-gn'); if (g) select(+g.dataset.id); });
  svg.addEventListener('keydown', e => {
    const g = e.target.closest('.rdx-gn');
    if (g && (e.key === 'Enter' || e.key === ' ')) { e.preventDefault(); select(+g.dataset.id); }
  });
  root.addEventListener('click', e => {
    const act = e.target.closest('[data-act]')?.dataset.act;
    if (act === 'rebuild') build(session.term);
    if (act === 'open' && G.sel != null) open(G.sel);
    const r = e.target.closest('[data-route]');
    if (r) { route = r.dataset.route; renderRoutes(); }
  });
  etaBox.addEventListener('change', () => build(G.nodes.get(G.root).term));

  session.on(type => {
    if (type === 'load') { etaBox.checked = session.eta; build(session.start); }
    else mark();
  });
}
