# Port ledger

The running record of the Rust port (`docs/plan.md`). One section per milestone: status, decisions,
parity gaps, gzipped wasm size.

## Gates

`cargo fmt --all --check` · `cargo clippy --workspace --all-targets -- -D warnings` ·
`cargo test --workspace` · `trunk build --release` ·
`cargo run --manifest-path crates/lints/Cargo.toml -- --base master`

## 0. Scaffold — done

- Workspace with the eleven crates of the plan plus `bin/page`; `crates/lints` vendored and
  `exclude`d. `rustfmt.toml`, `rust-toolchain.toml` and the `[workspace.lints]` block copied
  verbatim from `noether-labs`. Release profile: `opt-level = "z"`, LTO, one codegen unit,
  `panic = "abort"`; Trunk runs `wasm-opt -Oz` (`data-wasm-opt="z"`).
- `scripts/golden.mjs` writes `fixtures/engine/*.json` from the JS engine: 14 examples, 3 races, 80
  seeded random terms, and 26 parser error cases. Per case: notes, defs, printed term, α-key, size,
  free variables, the term tree with relative uids/provenance/marks, the blocks HTML (live and de
  Bruijn), every redex with schema, pattern variables, bindings, `prepare` log and result, naive
  substitution, the contracted result and its blocks HTML; the three strategies at caps 24 and 200;
  graphs at (no η, 70), (η, 70) and (the example's η, 40). The script is deterministic; CI reruns it
  and fails on a diff.
- Ids: the JS engine numbers nodes from one global counter. Fixtures write every id relative to a
  probe node made at the start of the case, so matching fixtures pin the allocation order, and with
  it provenance.

Decisions

- House Lint: base branch `master`, Leptos exemptions moved from `crates/client` to
  `crates/widgets`. Its `CLAUDE.md` standing counts reset to redex's (all zero).
- Root `AGENTS.md`: the Taste and Comments sections are noether's verbatim, except the line about
  robopoker's float aliases (`Utility`, `Probability`, …), which names types redex does not have,
  and `crates/client` → `crates/widgets`.
- `redex.css` and `site.css` are copied to `crates/widgets/redex.css` and `bin/page/site.css`; the
  originals stay until cutover. They must stay byte-identical until then.
- Trunk serves on port 8001 so the JS page (`npm run serve`, 8000) can run beside it.

Size: 8.5 KB gzipped (empty page).

## 1. Engine — done

`term`, `subst`, `syntax`, `rules`, `strategy`, `explore` (and `examples`, which the ported engine
tests need). `crates/explore/tests/golden.rs` checks every fixture key the engine produces (all but
the blocks HTML, which is `widgets`' and comes in milestone 3) and the parser error copy;
`crates/explore/tests/engine.rs` ports `test/engine.test.js` case for case. All pass, uids
included: allocation order matches the JS engine node for node.

Decisions

- `Term` is `Rc<Node>`; `from`, `mark` and `cap` sit in `Cell`s. The JS workbench mutates them
  (`stripMarks` after the first render, `tagFree` on fresh copies) on nodes that history entries
  share, so immutable decorations would change what a revisited history entry shows.
- Uids and the constant set (`CONSTS`, which `session.load` grows with every defined name and
  `fresh` avoids) are per-thread globals, as in the JS. Uid numbering is part of the spec.
- `Env` lives in `term`, not `rules`: `syntax` needs it to resolve names and must not depend on
  `rules`. `examples` depends on `rules` for `Kind`.
- Helpers that the JS keeps as module functions hang on traits over `Term`: `Substitute` (subst),
  `Print` (syntax), `Reduce` (rules). Their private helpers are trait methods too, since a second
  trait in the file would break one-type-per-file.
- `term::Number` prints numbers as JavaScript does (`String(x)`, `toFixed`), for error copy now and
  SVG coordinates later; tested against node's output.

JS behaviour ported as-is (looks like a bug)

- `def toString := …` (or any `Object.prototype` member name) is rejected as "built in", because
  the JS checks `ALIAS[name]` on a plain object. Covered by the error fixtures.
- A `\r` before a line's end defeats the `def` and `--` regexes (`.` does not match it), so with
  CRLF sources notes and definitions fall into the body. Ported, fixture-covered.
- The tokenizer walks UTF-16 code units, so an astral character is reported as half a surrogate
  pair in "Unexpected character". Rust reports the whole character. Not reproducible in a Rust
  `String`; the only gap, and only in an error message.

Size: unchanged (engine not linked into the page yet).

## 2. Geometry and data — done

`layout` (`Plot` for the graph widget, `Thumb` for the deck) and `examples` (`Example`, `Race`).

- `scripts/golden-dom.mjs` reads the JS page in headless Chrome (`playwright-core` driving the
  installed Chrome, reduced motion) and writes `fixtures/dom/`: the deck's HTML, the graph widget's
  SVG, stats, verdict and all four route bands for every example and random term, with η off and
  on, and the race lanes for every race. Deterministic (two runs, no diff); CI reruns it.
- `crates/layout/tests/plot.rs`: width, height, every vertex position and every edge path, as the
  strings JS wrote, for 97 terms × 2. `tests/thumb.rs`: every thumbnail SVG and fate line, byte for
  byte (the fixture's DOM serialization `></path>` is mapped back to the source's `/>`).

Decisions

- Coordinates print through `term::Number` (`String(x)` and `toFixed(1)`), and every expression
  keeps the JS operation order, since the strings must match to the last digit.
- `playwright-core` is installed with `--no-save`: `package.json` stays as it was, and the
  dependency disappears with the JS at cutover.

## 3. Motion and blocks — done

- `motion`: `Motion` (a boxed `FnOnce(Clock) -> Future`), the primitives `enter` `exit` `slide`
  `resize` `fly` `crossfade` `draw`, the compositions `all` `seq` `after` `stagger` `act`, `Clock`,
  `Ease`, `Tween` options, `Ghost` (pin and `delta`). Constants and composition structure are the
  JS's: `BEAT` 620, exit 0.55 beat, crossfade 0.45 beat, the timer race at `delay + duration + 60`.
- `widgets`: `Blocks` renders the blocks markup; `tests/blocks.rs` matches every `blocks` and
  `debruijn` string in `fixtures/engine` (parsed terms, definitions, every one-step result) byte for
  byte, ids rebased. `Session` is the JS session's state and synchronous listener list, shared by
  context. `Mount::all` mounts every `[data-redex]` element with `leptos::mount::mount_to`.
- `bin/page/index.html` is the JS `index.html` with Trunk links in place of the stylesheets and the
  module script. The page renders the workbench's blocks line; its DOM equals the JS page's after
  rebasing ids (`scripts/parity.mjs`, which screenshots both pages in headless Chrome and diffs each
  widget's DOM).

Decisions

- **Widgets render their term regions as markup strings, not per-node Leptos views.** The JS
  rebuilds a region's `innerHTML` on every change and its FLIP animation depends on that: it lifts
  ghosts from the old DOM, swaps the markup, then measures the new DOM. Leptos owns each widget's
  skeleton (controls, toggles, regions, `NodeRef`s, events); the term-shaped regions (blocks line,
  trace, status, rule card, graph SVG, lanes) are pure string renderers, tested natively against
  the JS output, and set with `set_inner_html`. Element continuity during motion comes from ghosts,
  exactly as in the JS.
- **The session notifies synchronously**, not through signals. The workbench pushes a step and
  measures the re-rendered DOM in the same tick (`moveTo` right after `session.push`); a Leptos
  effect would run a microtask later. `Session` keeps the JS listener list; the context holds a
  `StoredValue<Session, LocalStorage>` (context values must be `Send + Sync`).
- `Element::animate` is behind web-sys's unstable APIs; `motion` calls it (and `cancel`,
  `finished`) through `js_sys::Reflect` rather than build with `--cfg web_sys_unstable_apis`.
- Futures: `Settle` (a promise as a future), `Join` and `Race` are written in `motion` instead of
  adding `futures` or `wasm-bindgen-futures` as dependencies. Primitives start their animation when
  played, synchronously, as the JS does.
- `widgets` keeps pub fields on `State` and `Entry` (the crate is exempt from `pub-field`).

Size: 75.5 KB gzipped (engine, Leptos, blocks).

## 4. Workbench — done

`widgets/src/workbench.rs` ports `workbench.js` function for function (`render`, `matched`, `paint`,
`show_rule`, `lift`, `move_to`, `travel`, `glide`, `fire_at`, `run`, `rename`, …) on a `Bench`
that holds the session, an anchor `NodeRef` and the widget's local state; `Doc` holds the DOM
helpers and `drawArcs`. One firing grammar: nothing in the move is rule-specific.

Parity (`scripts/parity.mjs`, canonical DOM: attributes sorted, whitespace text dropped, ids
numbered by first appearance), reduced motion and full motion, final states: load, hover a badge,
Step, Normalize, the automatic α before a capturing β (capture panel included), rename a binder,
de Bruijn, Back, trace click, ←/→ keys, a parse error, δ hover (definitions list), Ω under
Normalize (stops on the loop). The workbench DOM, binding-link SVG included, is identical in all.
Mid-firing (300, 800, 1000 ms after a click) the ghosts are the same set with the same markup and
inherited text settings; only their page offsets differ, because the deck above the workbench is
not ported yet.

Decisions

- `Doc` (not `Dom`, which `leptos::prelude` exports).
- Ghost styles are written through `style.cssText`, as the JS does, so the browser normalizes them
  the same way.
- Static markup is in `view!`; the canonical-DOM diff ignores attribute order, which Leptos
  changes (`class` first) and which means nothing to CSS.
- `run` and `fire_once` start through `leptos::task::spawn_local`, a microtask later than the JS's
  synchronous start of an async function. Nothing observable sits between the click and that
  microtask.

Size: 125.6 KB gzipped (the workbench brings in more of Leptos and web-sys).

## 5. Rules deck, graph, race — done

`Deck` (`Tiles` renders the markup), `Paths` (`Chart`: SVG, stats, verdict, route bands, tip,
selection, chain to open), `Lanes` (`Track`: pills, lanes, endings; the tick schedule on
`gloo_timers::callback::Timeout`). Committed one at a time, each after its own parity pass.

Parity: `scripts/parity.mjs` with `scripts/parity-states.json` (28 states: the workbench set from
milestone 4, graph routes ×3, η toggle, Ω, tip, select, open in workbench, walked marks, rebuild,
races lazy/under/bench/again, mid-race at 1.3 s, mid-firing at 0.3 and 0.8 s, keys), each with
reduced motion and full motion. **The canonical DOM of all four widgets, plus every ghost pinned
to the body, is identical in all 56 runs**, tooltip position styles included.

Screenshots, pixel-compared in the same headless Chrome: 32 of 56 identical. The rest:

- Reduced motion, 3 states (back, lazy race, keys): 139–174 scattered pixels, single-pixel
  differences around the fixed header's shadow and a few glyph edges, after the page has scrolled.
  Not visible side by side; not traced further.
- Full motion: differences sit only where a CSS keyframe is mid-flight at capture time (the
  graph's "here" halo pulse, the landing and flash afterglows, the race lanes fading in), whose
  phase depends on when the screenshot lands. Motion parity is judged by constants and
  composition structure, per the plan.

Decisions

- Component names avoid the engine's types: `Deck` (rules), `Paths` (graph), `Lanes` (race).
- The graph opens a term with `document.querySelector('[data-redex="workbench"]')`, as the JS
  does, although it reaches outside the widget's root. Ported as-is.
- A widget whose markup must exist before the first `load` event (the deck, the race's initial
  lanes) renders it through `inner_html` in `view!`, so it is in the DOM when `mount_to` returns.

Size: 155.0 KB gzipped.

## 6. Embedding — done

- `mountAll(scope?)` is exported from the `redex` cdylib (`crates/redex/src/embed.rs`). Every
  `[data-redex]` element under the scope (default: the document) gets its widget, all on one
  session; `data-example` on any of them picks the first example; an unknown kind is skipped.
- `scripts/site.sh` builds `dist/site/{redex.js, redex_bg.wasm, redex.css}` for the Jekyll site
  (wasm-bindgen `--target web`, `wasm-opt -Oz`). Checked on a host page in headless Chrome with
  prose between widgets, the graph before the workbench, `data-example="church"`, and a bogus
  `data-redex="nope"`: all four widgets mount, picking a tile moves the workbench, the graph and
  the race's "Workbench term" together. 176.8 KB of wasm, gzipped.
- `scripts/bundle-wasm.mjs` writes the single-file artifact: the page, both stylesheets, the glue
  and the wasm as base64 (622 KB). It boots and mounts all four widgets in headless Chrome.

Decisions and gaps

- Host contract changes from `import { mountAll } from '…/redex.js'; mountAll();` to
  `import init, { mountAll } from '…/redex.js'; await init(); mountAll();`. README is updated at
  cutover.
- `mountAll` returns nothing; the JS returned the session object. Nothing on the page used it.
- `mount_all` carries `// lint:allow(free-function)`: a module-level wasm-bindgen export must be a
  bare `fn`. The alternative, a static method (`Redex.mountAll()`), would change the host API
  further.
- The bundle is a new script beside the JS `scripts/bundle.mjs`, which still bundles the JS page;
  at cutover it replaces it.
- Not verified: that a claude.ai artifact's sandbox allows compiling wasm (it needs
  `wasm-unsafe-eval` under a strict CSP). CI does not run `scripts/site.sh` (it needs a
  wasm-bindgen CLI pinned to `Cargo.lock`'s version and a recent `wasm-opt`).

Size: 155.2 KB gzipped (page); 176.8 KB (site build, with the `mountAll` export).

## Bugs fixed

Each fix lands in the JS and the Rust together, so `scripts/golden.mjs` still writes every fixture
from one source, and each adds a case that fails before the fix and passes after.

1. **Definition names that are `Object.prototype` members.** `def toString := a` (or
   `constructor`, `valueOf`, `__proto__`, …) was rejected as "built in", because the JS looked
   the name up with `ALIAS[name]` on a plain object. Now `Object.hasOwn(ALIAS, name)`, in the
   check and in name resolution; the Rust drops its copy of the prototype list. Cases in
   `errors.json`: `toString`, `constructor`, `__proto__` (used as a constant), `valueOf`.
2. **CRLF sources.** With `\r\n` line ends, the `def` and `--` regexes failed (`.` does not
   match `\r`), so notes and definitions fell into the term body. Both parsers now drop one
   trailing `\r` per line. Cases: `-- note\r\nx` (the note is kept now) and
   `def k := a\r\n-- uses k\r\nk b\r\n`.
3. **Characters outside the Basic Multilingual Plane** in "Unexpected character". The JS walked
   UTF-16 code units and quoted half a surrogate pair; it now quotes the whole code point
   (`String.fromCodePoint(src.codePointAt(i))`), as the Rust always did. Cases: `x 😀 y`,
   `λ𝑥. 𝑥`; the Rust test failed against the old fixture and passes now.
4. **The graph reached outside its own root** (`document.querySelector('[data-redex="workbench"]')`)
   to scroll the workbench into view after "Open in workbench", so with two groups of widgets on
   a page it scrolled the first group's workbench. The session now carries a `reveal` event; the
   graph emits it and each workbench scrolls its own root. `scripts/reveal.mjs` mounts two groups
   on one page and opens a term from the second group's graph: before the fix both engines left
   the second workbench 3867 px down; after, it sits at the top, in the JS and the Rust.
