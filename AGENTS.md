# AGENTS.md

Interactive reduction rules for the lambda calculus, headed for a post on krukah.com. See README.md
for the file layout and how a page embeds the widgets.

## Rules

- `src/engine.js` stays free of the DOM and covered by `test/engine.test.js`. Run `npm test` after
  any engine or example change.
- A widget queries only inside its own root and talks to others only through the session. No
  element ids, no module-level DOM state: a host page may hold several widgets.
- Every class in widget markup is prefixed (`rdx-` for structure, `r…` for term tokens) and every
  selector in `redex.css` starts with `.redex`. Host pages run their own CSS (Jekyll prose, rouge's
  `.n`/`.c`/`.k` highlighting), so unprefixed names collide.
- Colors and fonts come only from `--rdx-*` tokens, declared under `:where(:root)` so any host rule
  overrides them. No literal colors in widget rules.
- Visual direction: the Lean documentation, not an app. Black on white, Lean blue (`#0073a3`), thin
  gray rules, square corners, Source Sans 3 and Source Code Pro, keywords bold, bound variables italic.
- Strategies never perform η, as in Lean, where η only appears when comparing terms. η fires only
  when clicked.
- Every animation checks `motion()` and the CSS honors `prefers-reduced-motion`.
- Examples are self-contained: each carries the `def`s it uses.

## Check

```bash
npm test
npm run serve   # then open http://localhost:8000
```
