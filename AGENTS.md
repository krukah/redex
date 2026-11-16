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
- One firing grammar for every rule: match, bind, move (README "Core model"). A new rule needs a
  schema and `bindings()` in the engine, and every node it builds from an old one needs `from`;
  the workbench then animates it with no rule-specific code.
- Firing motion is direct: pieces fly in straight lines (no arcs), and flights and the context glide
  share one duration and easing (`FLIGHT`, `EASE` in `workbench.js`).
- Keep the three color channels apart: rule ink, binder hue, highlighter.
- Every animation checks `motion()`, the CSS honors `prefers-reduced-motion`, and no firing waits on
  an animation without a timeout (`settle`).
- Examples are self-contained: each carries the `def`s it uses.

## Check

```bash
npm test
npm run serve   # then open http://localhost:8000
```
