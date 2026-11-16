# Redex Workbench

Fire α, β, η, δ, ζ, ι and π redexes by hand, see every reduction path of a term as a graph, and
race three evaluation strategies. Plain HTML, CSS and ES modules: no build step, no dependencies.

```bash
npm run serve   # http://localhost:8000 (ES modules need a server; file:// will not load them)
npm test        # engine tests, node:test
```

## Layout

| Path | What it holds |
|---|---|
| `src/engine.js` | Terms, the seven rules as schemas with bindings, capture-avoiding substitution, node provenance, strategies, parser, text printer, graph exploration. No DOM. |
| `src/blocks.js` | The nested-block view of a term: one element per node, carrying its path and provenance. |
| `src/examples.js` | Example programs. Each carries its own `def`s; `--` lines become the caption. |
| `src/session.js` | Shared state (program, history). Widgets talk to each other only through it. |
| `src/rules.js` `workbench.js` `graph.js` `race.js` | One widget each: `mountX(root, session)`. The rules widget is a grid of rules plus example tiles, each tile a thumbnail of its reduction graph. |
| `src/redex.js` | `mountAll()`: mounts every `[data-redex]` element on the page with one shared session. |
| `src/redex.css` | Widget styles, all scoped under `.redex`, all colors and fonts from `--rdx-*` tokens. |
| `index.html`, `site.css` | The standalone page, styled after the Lean documentation (Verso), without its sidebar. |
| `scripts/bundle.mjs` | Inlines the page, CSS and modules into one file for a claude.ai artifact preview. |
| `test/engine.test.js` | Normal forms, strategy step counts, capture, confluence, parser, provenance, rule bindings. |

## Core model

- **Term**: a tree of `var`, `const`, `lam`, `app`, `let`, `pair`, `proj`. Every node has a `uid`;
  a node built from an older one records it in `from`.
- **Rule**: a schema over pattern variables, such as `(λ{x}. {t}) {s} ⟶ {t}[{x} := {s}]`, plus
  `bindings()`, which says where each pattern variable sits inside a redex.
- **Firing**: match (paint each bound piece in its variable's highlighter color), bind (list the
  bindings on the rule card), move (fly each piece to every node of the result whose `from` is that
  piece; pieces with no such node fade where they stood). The same three steps serve all seven rules.
- **Color channels**: rule ink (badges, edges), binder hue (λ boxes and their variable tags),
  highlighter (pattern variables, only while matching). They never stand in for each other.

## Embedding in a page

Widgets are independent `<div>`s, so prose can sit between them:

```html
<link rel="stylesheet" href="/assets/redex/redex.css">

<p>…</p>
<div data-redex="rules"></div>
<div data-redex="workbench" data-example="beta"></div>
<div data-redex="graph"></div>
<div data-redex="race"></div>

<script type="module">
  import { mountAll } from '/assets/redex/redex.js';
  mountAll();
</script>
```

For the Jekyll site, copy `src/` to `assets/redex/` and put the block above in a post; kramdown
passes raw HTML blocks through. Two things to handle there:

- **Theme.** Set the `--rdx-*` tokens to the site's own. The defaults are light; a dark page also
  needs lighter rule hues (`--rdx-alpha` … `--rdx-proj`). A starting point for krukah.com:

  ```css
  .prose .redex {
    --rdx-bg: var(--panel); --rdx-sunk: var(--panel2); --rdx-hover: var(--grid2);
    --rdx-ink: var(--ink); --rdx-muted: var(--dim); --rdx-line: var(--line); --rdx-node: var(--line2);
    --rdx-accent: var(--hot); --rdx-text-font: var(--mono); --rdx-code-font: var(--mono);
  }
  ```
- **Bleed.** `.prose code`, `.prose table` and similar host rules match elements inside the widgets.
  Either place the widgets outside `.prose` or reset those rules under `.prose .redex`.

## Syntax

`λx. t` or `fun x => t` · `let x := v; t` · `(a, b)` · `p.1` `p.2` · `Nat.rec z s n` · `zero`,
`succ`, numerals · `def name := t` on its own line · `--` comments. Unknown names are free variables.
