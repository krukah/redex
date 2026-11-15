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
| `src/engine.js` | Terms, the seven rules, capture-avoiding substitution, strategies, parser, printer, graph exploration. No DOM. |
| `src/examples.js` | Example programs. Each carries its own `def`s; `--` lines become the caption. |
| `src/session.js` | Shared state (program, history). Widgets talk to each other only through it. |
| `src/rules.js` `workbench.js` `graph.js` `race.js` | One widget each: `mountX(root, session)`. |
| `src/redex.js` | `mountAll()`: mounts every `[data-redex]` element on the page with one shared session. |
| `src/redex.css` | Widget styles, all scoped under `.redex`, all colors and fonts from `--rdx-*` tokens. |
| `index.html`, `site.css` | The standalone page, styled after the Lean documentation (Verso). |
| `test/engine.test.js` | Normal forms of every example, strategy step counts, capture, confluence, parser. |

## Embedding in a page

Widgets are independent `<div>`s, so prose can sit between them:

```html
<link rel="stylesheet" href="/assets/redex/redex.css">

<div data-redex="rules"></div>
<p>…</p>
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
