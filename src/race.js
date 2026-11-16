// The strategy race: one start term, three rules for picking the next redex, run side by side.
import { applicativeNext, esc, isEta, normalNext, parseProgram, redexes, runStrategy, text, whnfNext } from './engine.js';
import { $$, markFlat } from './dom.js';
import { enter, motion } from './motion.js';
import { RACES } from './examples.js';

const CAP = 24, TICK = 420;
const LANES = [
  { name: 'Normal order', sub: 'Leftmost-outermost. Reaches a normal form whenever one exists.', next: normalNext },
  { name: 'Applicative order', sub: 'Leftmost-innermost. Arguments first, like a strict language.', next: applicativeNext },
  { name: 'Head only', sub: 'What Lean’s whnf does. Stops once the head is a λ, a constructor or stuck.', next: whnfNext },
];

export function mountRace(root, session, { start = 'dup' } = {}) {
  root.innerHTML = `
    <div class="rdx-race-bar"><div class="rdx-pills"></div><button type="button" class="rdx-btn primary" data-act="race">Race</button></div>
    <div class="rdx-lanes"></div>`;
  const pills = root.querySelector('.rdx-pills'), lanesEl = root.querySelector('.rdx-lanes');
  let pick = start, timer = 0;

  const source = () => {
    if (pick === 'bench') return session.hist.length ? { term: session.start, env: session.env } : null;
    const p = parseProgram(RACES.find(r => r.id === pick).src);
    return { term: p.term, env: p.env };
  };
  const count = r => {
    const n = r.rows.length - 1;
    return r.status === 'done' ? `${n}<small>step${n === 1 ? '' : 's'}</small>` : '∞';
  };
  const ending = (r, env) => {
    if (r.status === 'cap') return ['loops', `Still going after ${CAP} steps. It loops.`];
    if (r.status === 'big') return ['loops', 'Stopped: the term keeps growing.'];
    const left = redexes(r.rows[r.rows.length - 1].term, env).filter(x => !isEta(x.kind)).length;
    return left ? ['part', `Head exposed. ${left} redex${left > 1 ? 'es' : ''} left inside.`] : ['nf', 'Normal form'];
  };

  function render(animate) {
    clearTimeout(timer);
    pills.innerHTML = RACES.map(r => `<button type="button" class="rdx-pill${pick === r.id ? ' on' : ''}" data-race="${r.id}" aria-pressed="${pick === r.id}">${esc(r.label)}</button>`).join('');
    const src = source();
    if (!src) { lanesEl.innerHTML = ''; return; }
    const runs = LANES.map(l => ({ l, ...runStrategy(src.term, src.env, l.next, CAP) }));
    lanesEl.innerHTML = runs.map(r => {
      const [cls, msg] = ending(r, src.env);
      return `<div class="rdx-lane"><header><div class="rdx-lane-top"><b>${r.l.name}</b><span class="rdx-count">${count(r)}</span></div><p>${r.l.sub}</p></header>`
        + `<ol>${r.rows.map((row, i) => `<li data-i="${i}">${markFlat(row.kind)}<code title="${esc(text(row.term))}">${esc(text(row.term))}</code></li>`).join('')}`
        + `<li class="rdx-end ${cls}" data-i="${r.rows.length}">${msg}</li></ol></div>`;
    }).join('');
    if (!animate || !motion()) return;

    const lanes = $$('.rdx-lane', lanesEl);
    const max = Math.max(...runs.map(r => r.rows.length));
    lanes.forEach(l => {
      $$('li', l).forEach(li => { if (+li.dataset.i > 0) li.classList.add('pending'); });
      l.querySelector('.rdx-count').textContent = '0';
    });
    let i = 0;
    const tick = () => {
      i++;
      lanes.forEach((l, j) => {
        const li = l.querySelector(`li[data-i="${i}"]`), ol = l.querySelector('ol');
        if (li) { li.classList.remove('pending'); enter(li, { ms: 300 })(); ol.scrollTop = ol.scrollHeight; }
        const steps = runs[j].rows.length - 1;
        l.querySelector('.rdx-count').innerHTML = i <= steps ? String(i) : count(runs[j]);
      });
      if (i < max) timer = setTimeout(tick, TICK);
    };
    timer = setTimeout(tick, 350);
  }

  root.addEventListener('click', e => {
    const p = e.target.closest('[data-race]');
    if (p) { pick = p.dataset.race; render(true); }
    if (e.target.closest('[data-act="race"]')) render(true);
  });
  session.on(type => { if (type === 'load' && pick === 'bench') render(false); });
  render(false);
}
