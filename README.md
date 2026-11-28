# Redex Workbench

Fire α, β, η, δ, ζ, ι and π redexes by hand, see every reduction path of a term as a graph, and
race three evaluation strategies. A Rust workspace with a Leptos (CSR) frontend, built to
WebAssembly by Trunk.

```bash
trunk serve                    # http://localhost:8001, the standalone page
trunk build --release          # the page, into dist/
cargo test --workspace         # engine, layout and markup tests, against the frozen fixtures
scripts/site.sh                # dist/site: redex.js, redex_bg.wasm, redex.css for a host page
node scripts/bundle.mjs out.html   # one self-contained file (after scripts/site.sh)
npm install && node scripts/reveal.mjs   # browser check of two widget groups on one page
node scripts/steady.mjs    # the workbench never changes height (every frame, every state)
node scripts/tempo.mjs     # how long each phase of each firing takes
```

Needs the `wasm32-unknown-unknown` target, Trunk, and for `scripts/site.sh` a `wasm-bindgen` CLI
of the version in `Cargo.lock` plus `wasm-opt`. `package.json` exists only to pin
`playwright-core` for the browser checks (`reveal.mjs`, `steady.mjs`, `tempo.mjs`, sharing
`browser.mjs`); `scripts/bundle.mjs` needs no packages.

## Layout

| Path | What it holds |
|---|---|
| `crates/units` | `Px`, `Ms`, and numbers printed as JavaScript prints them. |
| `crates/term` | Terms: nodes with uids and provenance (`from`), names, paths, α-keys, definitions. |
| `crates/subst` | Capture-avoiding substitution, binder renaming (`prepare`), the naive capturing variant. |
| `crates/syntax` | Tokenizer, parser, programs (`def`s and `--` notes), the text printer. |
| `crates/rules` | The seven rules as schemas with bindings; redex search and contraction. |
| `crates/strategy` | Normal, applicative and weak head order; the step-capped runner. Never η. |
| `crates/explore` | The reduction graph: every path to a cap, edges by rule, cycles. |
| `crates/examples` | Example and race programs. Each carries its own `def`s; `--` lines become the caption. |
| `crates/markup` | A typed HTML/SVG tree, its closed class set, typed attributes, selectors, one escaping renderer. |
| `crates/layout` | Pure geometry for the graph and the deck's thumbnails. |
| `crates/motion` | Motion primitives (enter, exit, slide, resize, fly, crossfade, draw) and compositions (all, seq, after, stagger, act). Every animation is built from these. |
| `crates/widgets` | The four Leptos widgets (rules deck, workbench, graph, race), the shared `Session`, and `redex.css`. |
| `crates/redex` | `mountAll()`: mounts every `[data-redex]` element on the page with one shared session. |
| `bin/page` | The standalone page (`index.html`, `site.css`), styled after the Lean documentation (Verso). |
| `crates/lints` | House Lint, outside the workspace. |
| `fixtures/` | Frozen regression data the tests check against (`engine/` per term, `dom/` per drawing). |
| `scripts/` | `site.sh` (host-page build), `bundle.mjs` (single-file artifact), `reveal.mjs`, `steady.mjs`, `tempo.mjs` (browser checks, over `browser.mjs`). |
| `docs/` | The port plan and its ledger, and the todo list. |

## Core model

- **Term**: a tree of `var`, `const`, `lam`, `app`, `let`, `pair`, `proj`. Every node has a `uid`;
  a node built from an older one records it in `from`.
- **Rule**: a schema over pattern variables, such as `(λ{x}. {t}) {s} ⟶ {t}[{x} := {s}]`, plus
  `bindings`, which say where each pattern variable sits inside a redex.
- **Firing**: match (paint each bound piece in its variable's highlighter color), bind (list the
  bindings on the rule card), move. Move is one composition over one beat: the rest of the redex
  fades out in place, each bound piece flies in a straight line to every node of the result whose
  `from` is that piece (or fades if there is none), the result fades in, and the surrounding term
  glides to its new layout. The same three steps serve all seven rules.
- **Color channels**: rule ink (badges, edges), binder hue (λ boxes and their variable tags),
  highlighter (pattern variables, only while matching). They never stand in for each other.

## Embedding in a page

Widgets are independent `<div>`s, so prose can sit between them:

```html
<link rel="stylesheet" href="/assets/redex/redex.css">

<p>…</p>
<div data-redex="rules"></div>
<div data-redex="examples"></div>
<div data-redex="workbench" data-example="beta"></div>
<div data-redex="graph"></div>
<div data-redex="race"></div>

<script type="module">
  import init, { mountAll } from '/assets/redex/redex.js';
  await init();
  mountAll();
</script>
```

For the Jekyll site, run `scripts/site.sh`, copy `dist/site/` to `assets/redex/`, and put the block
above in a post; kramdown passes raw HTML blocks through. `mountAll(scope)` mounts only the
widgets under `scope`, each group with its own session. Two things to handle there:

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
