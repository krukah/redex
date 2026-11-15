// Entry point. Put `<div data-redex="rules|workbench|graph|race">` anywhere on a page,
// then call mountAll(). Every widget in the scope shares one session.
// `data-example="<id>"` on any of them picks the first example (default: beta).
import { createSession } from './session.js';
import { mountRules } from './rules.js';
import { mountWorkbench } from './workbench.js';
import { mountGraph } from './graph.js';
import { mountRace } from './race.js';

const WIDGETS = { rules: mountRules, workbench: mountWorkbench, graph: mountGraph, race: mountRace };

export function mountAll(scope = document) {
  const session = createSession();
  const els = [...scope.querySelectorAll('[data-redex]')];
  for (const el of els) {
    const mount = WIDGETS[el.dataset.redex];
    if (!mount) continue;
    el.classList.add('redex');
    mount(el, session);
  }
  session.loadExample(els.find(el => el.dataset.example)?.dataset.example || 'beta');
  return session;
}
