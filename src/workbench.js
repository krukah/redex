// The workbench: one live term. Click a rule mark to fire that redex, click a binder to α-rename it.
import {
  A, L, LET, V, CONSTS, KCLS, at, contract, esc, fresh, fv, isEta, key, names, normalNext,
  prepare, redexes, replace, show, size, stripMarks, stripSub, subst, substNaive, text, whnfNext,
} from './engine.js';
import { $$, drawArcs, fly, markFlat, motion, onScreen, wait } from './dom.js';

const KINFO = {
  alpha: { name: 'α-conversion', schema: 'λx. t  ≡  λy. t[x:=y]' },
  beta: { name: 'β-reduction', schema: '(λx. t) s  ⟶  t[x:=s]' },
  eta: { name: 'η-reduction', schema: 'λx. f x  ⟶  f' },
  etaS: { name: 'η for structures', schema: '(p.1, p.2)  ⟶  p' },
  delta: { name: 'δ-unfolding', schema: 'c  ⟶  definition of c' },
  zeta: { name: 'ζ-reduction', schema: 'let x := v; t  ⟶  t[x:=v]' },
  iota: { name: 'ι-reduction', schema: 'Nat.rec z s (succ n)  ⟶  s n (Nat.rec z s n)' },
  proj: { name: 'projection', schema: '(a, b).1  ⟶  a' },
};
const HINT = 'Click a rule mark to fire that redex. Click a binder to rename it. <kbd>→</kbd> steps, <kbd>←</kbd> goes back.';
const NAMES = ['x', 'y', 'z', 'w', 'u', 'v', 'a', 'b', 'c', 'f', 'g', 'h', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't'];

export function mountWorkbench(root, session) {
  root.innerHTML = `
    <div class="rdx-notes"></div>
    <div class="rdx-scroll"><div class="rdx-stage"><svg class="rdx-arcs" aria-hidden="true"></svg><div class="rdx-line rdx-tm"></div></div></div>
    <div class="rdx-status" aria-live="polite"></div>
    <div class="rdx-readout"></div>
    <div class="rdx-controls">
      <button type="button" class="rdx-btn primary" data-act="step">Step</button>
      <button type="button" class="rdx-btn" data-act="whnf">To WHNF</button>
      <button type="button" class="rdx-btn" data-act="nf">Normalize</button>
      <button type="button" class="rdx-btn" data-act="back">Back</button>
      <button type="button" class="rdx-btn" data-act="reset">Reset</button>
      <span class="rdx-grow"></span>
      <label class="rdx-toggle"><input type="checkbox" data-opt="arcs" checked> Binding links</label>
      <label class="rdx-toggle"><input type="checkbox" data-opt="db"> de Bruijn</label>
    </div>
    <div class="rdx-capture" hidden>
      <p><b>Why α fired first.</b> Without renaming, substitution gives the term below. The dashed link is the argument’s free variable, caught by an inner binder, which changes what the term means.</p>
      <div class="rdx-scroll"><div class="rdx-stage small arcs-on"><svg class="rdx-arcs" aria-hidden="true"></svg><div class="rdx-line rdx-tm"></div></div></div>
    </div>
    <div class="rdx-defs" hidden><div class="rdx-label">Definitions <span>· δ unfolds these</span></div><div class="rdx-defs-list"></div></div>
    <details class="rdx-trace" open><summary>Trace</summary><ol></ol></details>
    <details class="rdx-src"><summary>Edit source</summary>
      <textarea spellcheck="false" rows="5" aria-label="Program source"></textarea>
      <div class="rdx-src-row"><button type="button" class="rdx-btn primary" data-act="load">Load</button>
        <p class="rdx-syntax"><code>λx. t</code> or <code>fun x =&gt; t</code> · <code>let x := v; t</code> · <code>(a, b)</code> · <code>p.1</code> · <code>Nat.rec z s n</code> · numerals · <code>def name := t</code> on its own line · <code>--</code> comments</p></div>
      <p class="rdx-err" hidden></p>
    </details>`;

  const q = s => root.querySelector(s);
  const el = {
    notes: q('.rdx-notes'), stage: q('.rdx-stage'), line: q('.rdx-line'), status: q('.rdx-status'), readout: q('.rdx-readout'),
    capture: q('.rdx-capture'), capStage: q('.rdx-capture .rdx-stage'), capLine: q('.rdx-capture .rdx-line'),
    defs: q('.rdx-defs'), defsList: q('.rdx-defs-list'), trace: q('.rdx-trace ol'),
    srcBox: q('.rdx-src'), src: q('.rdx-src textarea'), err: q('.rdx-err'),
    act: name => q(`[data-act="${name}"]`),
  };
  const W = { arcs: true, db: false, hot: null, running: null, stop: false, note: '' };

  // ---------- rendering ----------
  function render() {
    const t = session.term;
    el.line.innerHTML = show(t, { html: true, chips: true, env: session.env, db: W.db });
    stripMarks(t);
    el.stage.classList.toggle('arcs-on', W.arcs);
    drawArcs(el.stage, el.line, W.arcs);
    W.hot = null; readout(null);
    renderStatus(); renderTrace(); renderCapture(); setControls();
  }
  function renderSource() {
    el.notes.innerHTML = session.notes.map(n => `<p><span class="rdx-dash">--</span> ${esc(n)}</p>`).join('');
    el.defs.hidden = !session.env.size;
    el.defsList.innerHTML = [...session.env].map(([n, b]) =>
      `<div class="rdx-def rdx-tm"><span class="rc rdef">${esc(n)}</span> <span class="rk">:=</span> <span data-def="${esc(n)}">${show(b, { html: true, env: session.env })}</span></div>`).join('');
    el.src.value = session.src;
    el.err.hidden = true;
  }
  function readout(rx) {
    if (!rx) { el.readout.innerHTML = HINT; return; }
    const k = rx.dataset.kind, info = KINFO[k];
    el.readout.innerHTML = `${markFlat(k)} <b>${info.name}</b> <code>${esc(info.schema)}</code>`;
  }
  function setHot(rx) {
    if (W.hot === rx) return;
    if (W.hot) W.hot.classList.remove('hot');
    W.hot = rx;
    if (rx) rx.classList.add('hot');
    readout(rx);
  }
  function setLit(id) {
    $$('.lit', el.stage).forEach(e => e.classList.remove('lit'));
    if (!id) return;
    $$(`[data-id="${id}"], [data-bid="${id}"] > .rv, .rb[data-anchor="${id}"]`, el.stage).forEach(e => e.classList.add('lit'));
  }
  function renderStatus() {
    const t = session.term, rs = redexes(t, session.env), main = rs.filter(r => !isEta(r.kind));
    let s;
    if (!main.length) s = `<b class="rdx-nf">Normal form</b>${rs.length ? ' <span>except for η</span>' : ''}`;
    else if (!whnfNext(t, session.env)) s = '<b class="rdx-whnf">Weak head normal form</b>';
    else s = `<b>${rs.length} redex${rs.length > 1 ? 'es' : ''}</b>`;
    const tally = {};
    rs.forEach(r => (tally[r.kind] = (tally[r.kind] || 0) + 1));
    const marks = Object.entries(tally).map(([k, n]) => `<span class="rdx-tally">${markFlat(k)}${n > 1 ? '×' + n : ''}</span>`).join('');
    const last = session.hist.length - 1;
    el.status.innerHTML = `${s}${marks ? `<span class="rdx-tallies">${marks}</span>` : ''}`
      + `<span class="rdx-stepno">step ${session.idx}${last > session.idx ? ' of ' + last : ''}</span>`
      + (W.note ? `<span class="rdx-note">${esc(W.note)}</span>` : '');
  }
  function renderTrace() {
    const ol = el.trace;
    ol.innerHTML = session.hist.map((h, i) => `<li class="${i === session.idx ? 'cur' : ''}${i > session.idx ? ' fut' : ''}"><button type="button" data-i="${i}"><span class="rdx-ti">${i}</span>${markFlat(h.kind)}<code>${esc(text(h.term))}</code>${h.note ? `<small>${esc(h.note)}</small>` : ''}</button></li>`).join('');
    const li = ol.children[session.idx];
    if (!li) return;
    const top = li.offsetTop, bot = top + li.offsetHeight;
    if (top < ol.scrollTop) ol.scrollTop = top - 4;
    else if (bot > ol.scrollTop + ol.clientHeight) ol.scrollTop = bot - ol.clientHeight + 4;
  }
  function renderCapture() {
    el.capture.hidden = !session.capture;
    if (!session.capture) return;
    el.capLine.innerHTML = show(session.capture, { html: true, env: session.env });
    drawArcs(el.capStage, el.capLine, true);
  }
  function setControls() {
    const t = session.term, busy = session.busy;
    const canN = !!normalNext(t, session.env), canW = !!whnfNext(t, session.env);
    el.act('step').disabled = busy || !canN;
    el.act('whnf').disabled = busy ? W.running !== 'whnf' : !canW;
    el.act('whnf').textContent = W.running === 'whnf' ? 'Stop' : 'To WHNF';
    el.act('nf').disabled = busy ? W.running !== 'nf' : !canN;
    el.act('nf').textContent = W.running === 'nf' ? 'Stop' : 'Normalize';
    el.act('back').disabled = busy || session.idx === 0;
    el.act('reset').disabled = busy || (session.idx === 0 && session.hist.length === 1);
  }

  // ---------- firing ----------
  async function fireAt(path, kind, speed = 1) {
    const t = session.term, node = at(t, path);
    if (kind === 'beta' || kind === 'zeta') {
      const lam = kind === 'beta' ? node.f : node, arg = kind === 'beta' ? node.a : node.v;
      const log = [], body = prepare(lam.b, lam.x, fv(arg), log);
      if (log.length) {
        session.capture = substNaive(lam.b, lam.x, arg);
        const renamed = kind === 'beta' ? A(L(lam.x, body), arg) : LET(node.x, node.v, body);
        const olds = log.map(l => l[0]).join(', ');
        session.push({ term: replace(t, path, renamed), kind: 'alpha', note: `${log.map(([a, b]) => `${a} → ${b}`).join(', ')}, so the argument’s free ${olds} stays free` });
        if (motion()) await wait(1000 / speed);
      }
    }
    await animate(path, kind, speed);
    const now = session.term;
    session.push({ term: replace(now, path, contract(at(now, path), kind, session.env, 'fl-' + KCLS(kind))), kind });
  }

  async function animate(path, kind, speed) {
    if (!motion()) return;
    const node = el.line.querySelector(`[data-path="${path}"]`);
    if (!node) return;
    setHot(null);
    node.classList.add('firing');
    if (kind === 'beta' || kind === 'zeta') {
      const src = el.line.querySelector(`[data-path="${path + (kind === 'beta' ? 'a' : 'v')}"]`);
      const binder = el.line.querySelector(`[data-path="${kind === 'beta' ? path + 'f' : path}"]`);
      const occ = binder ? $$(`[data-bid="${binder.dataset.binder}"]`, el.line) : [];
      if (binder) $$(':scope > .rk, :scope > .rb', binder).forEach(e => e.classList.add('consumed'));
      occ.forEach(o => o.style.setProperty('--rc', `var(--rdx-${kind})`));
      await fly(src, occ, speed);
    } else if (kind === 'delta') {
      const src = el.defsList.querySelector(`[data-def="${CSS.escape(at(session.term, path).n)}"]`);
      if (src && onScreen(src)) { node.style.setProperty('--rc', 'var(--rdx-delta)'); await fly(src, [node], speed, true); }
      else await wait(340 / speed);
    } else {
      await wait(360 / speed);
    }
  }

  async function fireOnce(path, kind) {
    if (session.busy) return;
    session.busy = true; W.note = ''; setControls();
    try { await fireAt(path, kind, 1); } finally { session.busy = false; setControls(); }
  }

  async function run(mode) {
    if (session.busy) { if (W.running === mode) W.stop = true; return; }
    session.busy = true; W.running = mode === 'step' ? null : mode; W.stop = false; W.note = ''; setControls();
    const seen = new Set(session.hist.slice(0, session.idx + 1).map(h => key(h.term)));
    let n = 0, msg = '';
    try {
      while (!W.stop) {
        const t = session.term;
        const r = mode === 'whnf' ? whnfNext(t, session.env) : normalNext(t, session.env);
        if (!r) break;
        await fireAt(r.path, r.kind, mode === 'step' ? 1 : n < 4 ? 1.4 : n < 16 ? 2.4 : 5);
        n++;
        if (mode === 'step') break;
        const k = key(session.term);
        if (seen.has(k)) { msg = 'Stopped: this term came back to one it already passed through. It loops.'; break; }
        seen.add(k);
        if (n >= 200) { msg = 'Stopped after 200 steps.'; break; }
        if (size(session.term) > 2000) { msg = 'Stopped: the term grew past 2000 nodes.'; break; }
      }
    } finally {
      session.busy = false; W.running = null; W.note = msg;
      renderStatus(); setControls();
    }
  }

  function rename(path) {
    if (session.busy || W.db) return;
    const t = session.term, n = at(t, path);
    const avoid = names(n.b); avoid.add(n.x);
    const i = NAMES.indexOf(stripSub(n.x));
    let y = null;
    for (let j = 1; j <= NAMES.length && !y; j++) {
      const c = NAMES[(i + j) % NAMES.length];
      if (!avoid.has(c) && !CONSTS.has(c)) y = c;
    }
    y ||= fresh(n.x, avoid);
    const b = subst(n.b, n.x, V(y), new Set([y]), 'fl-alpha');
    const m = n.k === 'lam' ? L(y, b) : LET(y, n.v, b);
    m.mark = 'alphaB';
    W.note = '';
    session.push({ term: replace(t, path, m), kind: 'alpha', note: `${n.x} → ${y}` });
  }

  function back() { if (!session.busy && session.idx > 0) { W.note = ''; session.seek(session.idx - 1); } }

  // ---------- events ----------
  el.line.addEventListener('mouseover', e => {
    if (session.busy) return;
    setHot(e.target.closest('.rr'));
    const b = e.target.closest('[data-bid], .rb');
    setLit(b ? (b.dataset.bid || b.dataset.anchor) : null);
  });
  el.line.addEventListener('mouseleave', () => { setHot(null); setLit(null); });
  el.line.addEventListener('focusin', e => { const rx = e.target.closest('.rr'); if (rx && !session.busy) setHot(rx); });
  el.line.addEventListener('click', e => {
    if (session.busy) return;
    const bn = e.target.closest('.rb');
    if (bn) { rename(bn.dataset.path); return; }
    const rx = e.target.closest('.rr');
    if (rx) fireOnce(rx.dataset.path, rx.dataset.kind);
  });
  root.addEventListener('click', e => {
    const act = e.target.closest('[data-act]')?.dataset.act;
    if (act === 'step' || act === 'whnf' || act === 'nf') run(act);
    else if (act === 'back') back();
    else if (act === 'reset' && !session.busy) { W.note = ''; session.reset(); }
    else if (act === 'load' && !session.busy) {
      try { session.load(el.src.value, { eta: session.eta }); }
      catch (err) { el.err.textContent = err.message; el.err.hidden = false; el.srcBox.open = true; }
    }
    const row = e.target.closest('.rdx-trace [data-i]');
    if (row && !session.busy) { W.note = ''; session.seek(+row.dataset.i); }
  });
  root.addEventListener('change', e => {
    const opt = e.target.dataset.opt;
    if (opt === 'arcs') W.arcs = e.target.checked;
    if (opt === 'db') W.db = e.target.checked;
    if (opt) render();
  });
  document.addEventListener('keydown', e => {
    if (e.metaKey || e.ctrlKey || e.altKey || e.target.closest('textarea, input, select')) return;
    if (e.key === 'ArrowRight' && !session.busy && normalNext(session.term, session.env)) { e.preventDefault(); run('step'); }
    if (e.key === 'ArrowLeft' && !session.busy && session.idx > 0) { e.preventDefault(); back(); }
  });

  let raf = 0;
  const redraw = () => {
    cancelAnimationFrame(raf);
    raf = requestAnimationFrame(() => {
      drawArcs(el.stage, el.line, W.arcs);
      if (session.capture) drawArcs(el.capStage, el.capLine, true);
    });
  };
  new ResizeObserver(redraw).observe(el.stage.parentElement);
  document.fonts?.ready.then(redraw);

  session.on(type => {
    if (type === 'load') { W.note = ''; renderSource(); }
    render();
  });
}
