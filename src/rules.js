// The rule table: one row per rule, click a row to load its example into the session.
import { GLYPH, esc } from './engine.js';
import { RULES, MORE } from './examples.js';

export function mountRules(root, session) {
  root.innerHTML = `
    <div class="rdx-tablewrap">
      <table class="rdx-rules">
        <thead><tr><th scope="col" colspan="2">Rule</th><th scope="col">Schema</th><th scope="col"><span class="rdx-sr">Load</span></th></tr></thead>
        <tbody>${RULES.map(r => `
          <tr class="k-${r.k}" data-ex="${r.id}">
            <td class="rdx-g">${GLYPH[r.k]}</td>
            <td><b>${r.name}</b><span class="rdx-sub">${r.sub}</span></td>
            <td><code>${esc(r.schema)}</code></td>
            <td><button type="button" class="rdx-btn" data-ex="${r.id}" aria-label="Load the ${r.name} example">Load</button></td>
          </tr>`).join('')}
        </tbody>
      </table>
    </div>
    <p class="rdx-more">More examples:
      ${MORE.map(m => `<button type="button" class="rdx-link" data-ex="${m.id}">${esc(m.label)}</button>`).join('<span aria-hidden="true"> · </span>')}
    </p>`;

  root.addEventListener('click', e => {
    const hit = e.target.closest('[data-ex]');
    if (hit && !session.busy) session.loadExample(hit.dataset.ex);
  });
  session.on(type => {
    if (type !== 'load') return;
    root.querySelectorAll('[data-ex]').forEach(el => {
      const on = el.dataset.ex === session.ex;
      el.classList.toggle('on', on);
      if (el.tagName === 'BUTTON') el.setAttribute('aria-pressed', String(on));
    });
  });
}
