// The workbench: one live term drawn as nested blocks.
//
// Every rule firing follows one grammar:
//   match  hovering a redex shows its rule's schema; the schema's pattern variables and the
//          pieces of the term they matched share a highlighter color
//   bind   the rule card lists what each pattern variable is bound to
//   move   on firing, each bound piece flies to every place the result uses it;
//          pieces the result drops fade out where they stood
import {
  A, L, LET, V, CONSTS, KCLS, GLYPH, at, bindings, contract, esc, fresh, fv, isEta, key, names,
  normalNext, prepare, redexes, replace, schemaFor, schemaVars, size, stripMarks, stripSub, subst,
  substNaive, text, whnfNext,
} from './engine.js';
import { renderBlocks } from './blocks.js';
import { $$, drawArcs, markFlat, motion, wait } from './dom.js';

const RULE_NAME = {
  alpha: 'α-conversion', beta: 'β-reduction', eta: 'η-reduction', etaS: 'η for structures',
  delta: 'δ-unfolding', zeta: 'ζ-reduction', iota: 'ι-reduction', proj: 'projection',
};
const HINT = 'Hover a badge to see its rule match. Click to fire it. Click a binder to rename it. <kbd>→</kbd> steps, <kbd>←</kbd> goes back.';
const NAMES = ['x', 'y', 'z', 'w', 'u', 'v', 'a', 'b', 'c', 'f', 'g', 'h', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't'];
const short = (s, n = 34) => (s.length > n ? s.slice(0, n - 1) + '…' : s);
// An animation's end, or its scheduled end, whichever comes first: a throttled or paused
// timeline must never leave a firing half done.
const settle = (anim, ms) => Promise.race([anim.finished.catch(() => {}), wait(ms + 60)]);

export function mountWorkbench(root, session) {
  root.innerHTML = `
    <div class="rdx-notes"></div>
    <div class="rdx-rule" aria-live="polite"></div>
    <div class="rdx-scroll"><div class="rdx-stage"><svg class="rdx-arcs" aria-hidden="true"></svg><div class="rdx-line rdx-tm"></div></div></div>
    <div class="rdx-status"></div>
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
      <div class="rdx-scroll"><div class="rdx-stage small"><svg class="rdx-arcs" aria-hidden="true"></svg><div class="rdx-line rdx-tm"></div></div></div>
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
    notes: q('.rdx-notes'), rule: q('.rdx-rule'), stage: q('.rdx-stage'), line: q('.rdx-line'), status: q('.rdx-status'),
    capture: q('.rdx-capture'), capStage: q('.rdx-capture .rdx-stage'), capLine: q('.rdx-capture .rdx-line'),
    defs: q('.rdx-defs'), defsList: q('.rdx-defs-list'), trace: q('.rdx-trace ol'),
    srcBox: q('.rdx-src'), src: q('.rdx-src textarea'), err: q('.rdx-err'),
    act: name => q(`[data-act="${name}"]`),
  };
  const W = { arcs: true, db: false, hot: null, running: null, stop: false, note: '' };
  const node = p => el.line.querySelector(`[data-path="${p}"]`);

  // ---------- rendering ----------
  function render() {
    const t = session.term;
    el.line.innerHTML = renderBlocks(t, { live: true, env: session.env, db: W.db });
    stripMarks(t);
    drawArcs(el.stage, el.line, W.arcs);
    W.hot = null;
    if (!session.busy) showRule(null);
    renderStatus(); renderTrace(); renderCapture(); setControls();
  }
  function renderSource() {
    el.notes.innerHTML = session.notes.map(n => `<p><span class="rdx-dash">--</span> ${esc(n)}</p>`).join('');
    el.defs.hidden = !session.env.size;
    el.defsList.innerHTML = [...session.env].map(([n, b]) =>
      `<div class="rdx-def rdx-tm"><span class="b b-const b-def">${esc(n)}</span><span class="rk">:=</span><span data-def="${esc(n)}">${renderBlocks(b, { env: session.env })}</span></div>`).join('');
    el.src.value = session.src;
    el.err.hidden = true;
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
      + `<span class="rdx-stepno">step ${session.idx}${last > session.idx ? ' of ' + last : ''} · size ${size(t)}</span>`
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
    el.capLine.innerHTML = renderBlocks(session.capture, { env: session.env });
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

  // ---------- match: the rule card and the highlighted pieces ----------
  // Returns the pieces of the redex at `path`: [{ name, slot, el, uid }].
  function match(path, kind) {
    const redex = at(session.term, path), schema = schemaFor(redex, kind), vars = schemaVars(schema);
    const pieces = [];
    for (const b of bindings(redex, kind)) {
      const slot = vars.indexOf(b.name) + 1, owner = node(path + b.rel);
      if (!owner) continue;
      if (b.binder) {
        const tab = owner.querySelector(':scope > .bt > .rb') || owner.querySelector(':scope > .bt > .rk');
        if (tab) pieces.push({ name: b.name, slot, el: tab, binder: owner.dataset.binder });
      } else {
        pieces.push({ name: b.name, slot, el: owner, uid: owner.dataset.uid, text: text(at(redex, b.rel)) });
      }
    }
    if (kind === 'delta') {
      const body = el.defsList.querySelector(`[data-def="${CSS.escape(redex.n)}"] > .b`);
      if (body) pieces.push({ name: 'c', slot: 1, el: body, uid: body.dataset.uid, external: true });
    }
    return { redex, schema, vars, pieces };
  }
  function paint(m, strength) {
    unpaint();
    for (const pc of m.pieces) {
      if (!pc.slot) continue;
      pc.el.classList.add('mv', `s${pc.slot}`, strength);
      if (pc.binder) $$(`[data-bid="${pc.binder}"]`, el.line).forEach(v => v.classList.add('mv', `s${pc.slot}`, strength));
    }
  }
  function unpaint() {
    $$('.mv', root).forEach(e => e.classList.remove('mv', 's1', 's2', 's3', 's4', 'soft', 'hard'));
  }
  function showRule(rx, m) {
    if (!rx) { el.rule.innerHTML = `<p class="rdx-hint">${HINT}</p>`; return; }
    const kind = rx.dataset.kind;
    const schema = m.schema.replace(/\{(\w)\}/g, (_, v) => `<span class="mv s${m.vars.indexOf(v) + 1}">${v}</span>`);
    const binds = m.pieces.filter(pc => pc.slot && !pc.external).map(pc =>
      `<span class="rdx-bind"><span class="mv s${pc.slot}">${pc.name}</span> = <code>${esc(short(pc.binder ? pc.el.textContent : pc.text))}</code></span>`).join('');
    el.rule.innerHTML = `<p class="rdx-schema"><span class="rm flat k-${KCLS(kind)}">${GLYPH[kind]}</span><b>${RULE_NAME[kind]}</b><code>${schema}</code></p><p class="rdx-binds">${binds}</p>`;
  }
  function setHot(rx) {
    if (W.hot === rx) return;
    if (W.hot) W.hot.classList.remove('hot');
    W.hot = rx;
    if (!rx) { unpaint(); showRule(null); return; }
    rx.classList.add('hot');
    const m = match(rx.dataset.path, rx.dataset.kind);
    paint(m, 'soft'); showRule(rx, m);
  }
  function setLit(id) {
    $$('.lit', el.stage).forEach(e => e.classList.remove('lit'));
    if (!id) return;
    $$(`[data-id="${id}"], [data-bid="${id}"], .rb[data-anchor="${id}"]`, el.stage).forEach(e => e.classList.add('lit'));
  }

  // ---------- move: fly each bound piece to where the result uses it ----------
  function snapshot(m) {
    return m.pieces.map(pc => ({ ...pc, rect: pc.el.getBoundingClientRect(), ghost: pc.el.cloneNode(true), font: getComputedStyle(pc.el).fontSize }));
  }
  function ghostAt(pc) {
    const w = document.createElement('div');
    w.className = 'redex rdx-flyer rdx-tm';
    w.style.cssText = `left:${pc.rect.left}px;top:${pc.rect.top}px;width:${pc.rect.width}px;height:${pc.rect.height}px;font-size:${pc.font}`;
    pc.ghost.classList.remove('hot', 'soft');
    w.appendChild(pc.ghost.cloneNode(true));
    document.body.appendChild(w);
    return w;
  }
  async function move(snap, path, speed) {
    const result = node(path);
    if (!result) return;
    result.classList.add('emerging');
    const flights = snap.map((pc, i) => {
      const hits = !pc.uid ? [] : [result, ...result.querySelectorAll('[data-uid], [data-from]')]
        .filter(e => e.dataset.uid === pc.uid || e.dataset.from === pc.uid);
      const delay = (i * 150) / speed;
      if (!hits.length) {
        if (pc.external) return null;
        const g = ghostAt(pc), ms = 480 / speed;
        return settle(g.animate([{ opacity: 1, transform: 'none' }, { opacity: 0, transform: 'translateY(10px) scale(.7)' }],
          { duration: ms, delay, easing: 'ease-in', fill: 'both' }), delay + ms).then(() => g.remove());
      }
      return Promise.all(hits.map((tg, j) => {
        tg.classList.add('arriving');
        const tr = tg.getBoundingClientRect(), g = ghostAt(pc);
        const dx = tr.left + tr.width / 2 - (pc.rect.left + pc.rect.width / 2);
        const dy = tr.top + tr.height / 2 - (pc.rect.top + pc.rect.height / 2);
        const sc = Math.max(0.3, Math.min(1.6, tr.width / Math.max(1, pc.rect.width)));
        const lift = -Math.min(64, 20 + Math.abs(dx) * 0.12);
        const ms = 760 / speed, lag = delay + (j * 90) / speed, end = `translate(${dx}px, ${dy}px) scale(${sc})`;
        // The ghost reaches the target at 75%, then dissolves while the real piece fades in under it.
        const flight = settle(g.animate([
          { transform: 'translate(0,0) scale(1)', opacity: 0.97 },
          { transform: `translate(${dx / 2}px, ${dy / 2 + lift}px) scale(${(1 + sc) / 2})`, opacity: 0.97, offset: 0.4 },
          { transform: end, opacity: 0.97, offset: 0.75 },
          { transform: end, opacity: 0 },
        ], { duration: ms, delay: lag, easing: 'cubic-bezier(.45,.05,.3,1)', fill: 'both' }), lag + ms).then(() => g.remove());
        const reveal = wait(lag + ms * 0.72).then(() => {
          tg.classList.remove('arriving');
          if (pc.slot) tg.classList.add('landed', `s${pc.slot}`);
          return settle(tg.animate([{ opacity: 0 }, { opacity: 1 }], { duration: ms * 0.3, easing: 'ease-out' }), ms * 0.3);
        });
        return Promise.all([flight, reveal]);
      }));
    });
    await Promise.all(flights);
  }

  // The rest of the term glides from its old layout to the new one instead of snapping:
  // boxes that enclose the redex animate their size, nodes beside it slide, the stage resizes.
  function layout() {
    const rects = new Map([['stage', el.stage.getBoundingClientRect()]]);
    el.line.querySelectorAll('[data-uid]').forEach(e => rects.set(e.dataset.uid, e.getBoundingClientRect()));
    return rects;
  }
  function glide(before, path, speed) {
    const result = node(path), ms = 560 / speed, ease = 'cubic-bezier(.3,.1,.2,1)', anims = [];
    if (!result) return null;
    const size = (e, old) => {
      const now = e.getBoundingClientRect();
      if (Math.abs(now.width - old.width) > 0.5 || Math.abs(now.height - old.height) > 0.5) {
        anims.push(e.animate([{ width: `${old.width}px`, height: `${old.height}px` }, { width: `${now.width}px`, height: `${now.height}px` }], { duration: ms, easing: ease }));
      }
    };
    const visit = parent => {
      for (const c of parent.children) {
        if (c === result) continue;
        if (c.dataset.uid === undefined) { visit(c); continue; }
        const old = before.get(c.dataset.uid);
        if (c.contains(result)) { if (old) size(c, old); visit(c); continue; }
        if (!old) { anims.push(c.animate([{ opacity: 0 }, { opacity: 1 }], { duration: ms, easing: ease })); continue; }
        const now = c.getBoundingClientRect(), dx = old.left - now.left, dy = old.top - now.top;
        if (Math.abs(dx) > 0.5 || Math.abs(dy) > 0.5) anims.push(c.animate([{ transform: `translate(${dx}px, ${dy}px)` }, { transform: 'none' }], { duration: ms, easing: ease }));
      }
    };
    visit(el.line);
    size(el.stage, before.get('stage'));
    el.stage.classList.add('moving');
    return Promise.all(anims.map(a => settle(a, ms))).then(() => {
      el.stage.classList.remove('moving');
      drawArcs(el.stage, el.line, W.arcs);
    });
  }

  // ---------- firing ----------
  async function fireAt(path, kind, speed = 1) {
    const t = session.term, redex = at(t, path);
    if (kind === 'beta' || kind === 'zeta') {
      const lam = kind === 'beta' ? redex.f : redex, arg = kind === 'beta' ? redex.a : redex.v;
      const log = [], body = prepare(lam.b, lam.x, fv(arg), log);
      if (log.length) {
        session.capture = substNaive(lam.b, lam.x, arg);
        const renamed = kind === 'beta' ? A(L(lam.x, body), arg) : LET(redex.x, redex.v, body);
        const olds = log.map(l => l[0]).join(', ');
        session.push({ term: replace(t, path, renamed), kind: 'alpha', note: `${log.map(([a, b]) => `${a} → ${b}`).join(', ')}, so the argument’s free ${olds} stays free` });
        if (motion()) await wait(1000 / speed);
      }
    }
    let snap = [], before = null;
    if (motion()) {
      const rx = node(path);
      const m = match(path, kind);
      W.hot = null;
      rx?.classList.add('firing');
      showRule(rx, m); paint(m, 'hard');
      await wait(560 / speed);
      snap = snapshot(m);
      before = layout();
    }
    const now = session.term;
    session.push({ term: replace(now, path, contract(at(now, path), kind, session.env)), kind });
    if (motion()) await Promise.all([move(snap, path, speed), glide(before, path, speed)]);
  }

  async function fireOnce(path, kind) {
    if (session.busy) return;
    session.busy = true; W.note = ''; setControls();
    try { await fireAt(path, kind, 1); } finally { session.busy = false; unpaint(); showRule(null); setControls(); }
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
        await fireAt(r.path, r.kind, mode === 'step' ? 1 : n < 3 ? 1.3 : n < 12 ? 2.2 : 4.5);
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
      unpaint(); showRule(null); renderStatus(); setControls();
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
  el.line.addEventListener('mouseleave', () => { if (!session.busy) { setHot(null); setLit(null); } });
  el.line.addEventListener('focusin', e => { const rx = e.target.closest('.rr'); if (rx && !session.busy) setHot(rx); });
  el.line.addEventListener('click', e => {
    if (session.busy) return;
    const bn = e.target.closest('.rb');
    if (bn && bn.tagName === 'BUTTON') { rename(bn.dataset.path); return; }
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
