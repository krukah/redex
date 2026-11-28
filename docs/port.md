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

JS behaviour ported as-is at first, all since fixed in both engines (see "Bugs fixed" below):
`Object.prototype` names as definitions, CRLF sources, and astral characters in the tokenizer's
error.

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
- The graph opened a term with `document.querySelector('[data-redex="workbench"]')`, as the JS
  did, reaching outside the widget's root. Since fixed in both (see "Bugs fixed").
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
5. **A freed callback under a pending promise (Rust only).** When a tween's safety timer won the
   race against the animation's `finished` promise, the `Settle` future and its JS callback were
   dropped, and the promise later called the freed closure ("closure invoked recursively or after
   being dropped", seen during Normalize after the cutover). `Settle` now leaves an unsettled
   callback to JS when dropped. Checked by loading the built page in headless Chrome and running
   Normalize: three page errors before, none after.

## 6b. Typed markup, newtypes, house style — done

What changed

- **`markup`** (new crate, pure): `Node` (element, text, list) built by composition; `Tag`; the
  closed `Class` enum (every class the widgets write, `Blank` for the empty slot the JS templates
  left in `class="rdx-tile "` and `class=" fut"`); typed `Attr` (`Path`, `Uid`, `Anchor`, `Kind`,
  `Id`, `Step`, `Example`, `Race`, `Option<Strategy>`, `Name`, `Act`, `Style`, `Num`, `Trace`,
  `Plate`, …); `Data` for `data-*` names; `Num` (a number with its JS formatting); `Seg`/`Trace`
  (SVG path data); `Query` (CSS selectors from the same pieces); `Hue`, `Anchor`, `Plate`, `Act`,
  `Opt`. `Node`'s `Display` is the only string output and the only escaping.
- **`units`** (new crate, the bottom): `Px`, `Ms`, and `Number` (moved out of `term`).
- **Layout returns geometry**: `Plot` gives `Spot`s in `Px` and a `Trace` per edge; `Thumb` gives
  `Node`s.
- **Widgets build real DOM** from `Node`s (`Doc::fill`: `createElement` / `createElementNS` in
  the SVG namespace, attributes in writing order, text nodes). No `inner_html` remains anywhere.
  Building elements does not disturb the motion approach: a swap is still synchronous, ghosts are
  still `cloneNode` copies of live elements, and measuring after a swap reads the new layout
  exactly as before. The deck and the race, which need markup in place before the first `load`
  event, fill on that event (and on mount as a fallback) instead of through `inner_html=`.
- **Reads back are typed**: `Doc::read::<T>(el, Data::X)` parses into `Path`, `Uid`, `Kind`,
  `Anchor`, `Id`, `Step`, `Act`, `Example`, `Race`, `Strategy`; every query is a `Query` built
  from `Class`, `Tag` and `Data`; inline styles go through `motion::Prop`.
- **Newtypes** (engine): `Name`, `Prim`, `Field`, `Key`, `Size`, `Nat` (term); `Source`, `Note`,
  `Failure` (syntax); `Glyph`, `Var`, `Slot`, `Part` (rules); `Cap`, `Step` (strategy); `Id`,
  `Depth` (explore); `Px`, `Ms` (units); `Prop`, `Ease`, `Clock` as speed (motion); `Level`,
  `Widget`, `Letter` (widgets). Engine `Node` is now private to `term`.
- **House style**: House Lint `--all` reports nothing for every rule, including the four you ask
  for by name (`forwarding-closure`, `type-hint`, `inline-bound`, `field-access`); the vendored
  linter passes its own rules too. Every struct reads and writes its fields through one-line
  accessors. Every import is a glob (`markup::Selector` became `Query` to stop colliding with the
  Leptos prelude; the widgets name `term::Env` in full for the same reason). Single-use bindings
  are inlined or split into small methods; the ones left are reused, ordered on purpose (uid
  allocation, a token read before the cursor moves, a borrow released before listeners run) or
  cloned into a `move` closure. No blank lines inside bodies, no inline bounds, no pub fields.

Standing counts (`--all`, every rule, outside the vendored linter)

| Rule | Before 6b | After |
|---|---|---|
| gating rules (all 16) | 0 | 0 |
| `forwarding-closure` | 14 (13 + 1 in the linter) | 0 |
| `type-hint` | 0 | 0 |
| `inline-bound` | 0 | 0 |
| `field-access` | 348 | 0 |

The one `lint:allow` left is on `redex::mount_all`: a module-level wasm-bindgen export must be a
bare `fn`. Designing it away means `Redex.mountAll()` on the host page instead of `mountAll()`.

Strings that remain, and why

- Prose: stats, verdict and route note (`set_text_content`), status, trace notes, error copy,
  numbers in sentences (`format!`). These are user copy, not markup.
- `Node::text` content and the static skeletons in `view!` (`"Step"`, `"Trace"`, …).
- Program text (`Source`) and the parser's slices of it; `Line` works on raw source lines.
- Identifiers on the wire: `Example::id`, `Race::id`, `Strategy::id`, `Kind`'s name; parsed back
  through `FromStr`.
- Ghost `cssText` (one `format!` of `Px` values and the inherited computed styles), and the
  `px` suffix on inline lengths.
- The fixture dump format (`Term::dump`) and the JSON fixtures in tests.

Parity

- `fixtures/engine` blocks HTML, thumbnails and every graph coordinate still match byte for byte
  (the typed renderer reproduces attribute order, `/>` on empty SVG elements, and `String(x)` /
  `toFixed(1)`).
- `scripts/parity.mjs`, all 28 states × 2, final run: identical canonical DOM in 55 of 56. The one
  difference is a timing race in "mid" with motion on (800 ms into a firing): one engine still
  shows the fired redex's ghost while the other has removed it. Six isolated reruns of that state
  all matched; it shows only when the whole suite runs. Screenshots: 34 of 56 pixel-identical; the
  rest differ only where a CSS animation is mid-flight (motion on), or by 125–186 scattered
  pixels around the fixed header's shadow (reduced motion), as before 6b.
- `scripts/reveal.mjs` passes for both engines; `scripts/site.sh` builds and the host page
  mounts.
- `scripts/golden.mjs` and `scripts/golden-dom.mjs` reproduce the committed fixtures.

Size: 170.3 KB gzipped (page), up from 155.2 KB; 192.8 KB for the site build (was 176.8 KB).
Building DOM node by node, the typed tree and its renderer, and the selector type account for
the growth.

## 7. Cutover — done

- Deleted: `src/` (the JS app and its stylesheet), `test/`, the root `index.html` and `site.css`
  (the copies in `bin/page` and `crates/widgets/redex.css` are now the only ones), the JS
  bundler, and the scripts whose job was comparing against the JS: `golden.mjs`,
  `golden-dom.mjs`, `parity.mjs` and `parity-states.json`.
- Kept: `fixtures/engine` and `fixtures/dom`, as frozen regression data; the Rust tests check
  against them, and they change only deliberately, from the Rust side (noted in the explore,
  layout and widgets `AGENTS.md`). `scripts/site.sh`. The single-file bundler, renamed from
  `bundle-wasm.mjs` to `bundle.mjs` (needs no packages). `scripts/reveal.mjs`, reduced to a
  Rust-only browser check of two widget groups on one page.
- `package.json` now only pins `playwright-core` for `reveal.mjs` (with a lockfile).
- CI: the golden job is gone; House Lint also runs `--all` and each opt-in rule; a new job builds
  the site bundle (wasm-bindgen CLI pinned to `Cargo.lock`, binaryen from apt) and runs
  `reveal.mjs` in headless Chrome. Not run: nothing is pushed.
- README and root `AGENTS.md` describe the Rust-only repo: layout by crate, commands, the host-page
  snippet with `await init(); mountAll();`. Comments that pointed at the JS now point at the
  fixtures.

The milestone sections above are a record of the port as it happened, so they still name the
deleted scripts.

## 8. A workbench that holds still — done

Two asks from reading the post at blog width: the workbench must never change height, and every
firing must take the same time with legible motion. Recorded in commits `ef6db14` … on master.

### Timing, measured

`motion::Cue` marks each phase on the performance timeline (`redex:fire`, `alpha`, `hold`, `lift`,
`move`, `moved`), and `scripts/tempo.mjs` steps and normalizes examples in Chrome and prints one
row per firing. Columns: α-hold (fire → hold, only for a renaming step), hold (hold → lift),
push (drawing the contracted term), move (move → moved), gap (moved → next fire), total
(fire → moved).

Before (`mix`, the widest spread; every example showed the same pattern):

```
mix: Step by Step                                mix: Normalize
step rule  α-hold  hold  push  move  gap  total  step rule  hold  push  move  gap  total
   1 ζ          0   562     6   745   36  1314     1 ζ     431     3   562    0    997
   2 π          0   562    10   626   37  1197     2 π     433     4   498    0    934
   3 δ          0   562     6   695   36  1264     3 δ     432     5   529    0    966
   4 β          0   560     7   746   36  1314     4 β     256     4   340    0    600
   5 ι          0   561     6   763   36  1331     5 ι     256     3   341    0    600
   6 β          0   562     6   696   36  1264     6 β     255     4   324    0    583
   7 β          0   562     6   746   36  1315     7 β     255     5   338    0    600
   8 ι          0   561     5   347    -   913     8 ι     255     4   174    -    433
spread 418 ms                                    spread 564 ms
capture (α then β): α-hold 1002, total 2330 by Step; 771 and 1781 under Normalize
```

Sources of the variance:

- The move was `BEAT` plus a stagger of 50 ms per piece and 40 ms per copy, so it grew with the
  term (626–763 ms).
- Each phase was a `setTimeout` racing the animation plus 60 ms of slack, and the drawing of the
  new term (2–10 ms) and the slack added to the step instead of fitting inside it.
- Normalize sped up as it went (speed 1.3, then 2.2 after three steps, then 4.5): the first steps
  ~1000 ms, later ones ~600.
- The α step had its own constant (`RENAME`, 1000 ms) unrelated to the rest.
- A firing whose result has no element of its own (ι folding into a numeral) found nothing to
  land on: the move ended at 347 ms and the result snapped in.
- Inside the move, exits ran over .55 of the beat and the crossfade over .45, but the glide of the
  surrounding context measured after the resize had begun, so the context jumped once before
  sliding and again at the end.

After (`scripts/tempo.mjs page`, every example):

```
mix: Step by Step                                mix: Normalize
step rule  α-hold  hold  push  move  gap  total  step rule  hold  push  move  gap  total
   1 ζ          0   560     8   612  287  1181     1 ζ     280     3   308  120    592
   2 π          0   560     4   616  286  1181     2 π     280     3   307  120    590
   3 δ          0   561     6   615  287  1182     3 δ     280     4   308  118    593
   4 β          0   559     7   614  286  1180     4 β     281     5   305  121    591
   5 ι          0   560     5   614  285  1180     5 ι     281     6   302  122    590
   6 β          0   561     4   618  286  1183     6 β     280     5   306  120    591
   7 β          0   561     6   615  284  1181     7 β     279     4   306  120    591
   8 ι          0   561     3   618    -  1182     8 ι     280     4   306    -    591
spread 3 ms                                      spread 3 ms
beta, church, delta: spread 1–3 ms, the same numbers
capture (α then β): α-hold 1422 (one full step), then 1181 by Step; 712 and 592 under Normalize
```

By Step the gap is Settle (240 ms) plus the check's own click; under Normalize it is Settle at
speed 2 (120 ms), so steps start exactly 710 ms apart.

### Timing, fixed

- One timeline (`motion::Schedule`, diagram in `crates/motion/AGENTS.md`): Hold 560, Move 620,
  Settle 240 ms; phases and roles are types (`Phase`, `Role`), and every part of the move gets
  its window from `Schedule::part`. No stagger: all pieces and copies of a firing leave and land
  together.
- Phase ends are deadlines from the step's start (`Schedule::until`, `performance.now`), so
  drawing the new term happens inside Hold's budget, not after it.
- Normalize and To WHNF run at one steady cadence, `Schedule::RUN` (speed 2), from the first step
  to the last; Step and clicks use `Schedule::STEP` (speed 1).
- A renaming before β or ζ is its own step on the same timeline (rule card shows α, Hold, rename,
  Settle), not a separate constant.
- A result with no element of its own lands on its nearest drawn ancestor, so a numeral absorbing
  a successor fades in with the rest of the move instead of snapping in early.
- The glide sizes the boxes around the result back to their old sizes before measuring where the
  siblings were, so the context slides once, from where it was seen, with no jump before or after.
- Links (arcs) are hidden during the move and return at its end, inside Settle, with nothing else
  moving.
- `crates/motion/tests/schedule.rs` pins it: every step lasts the same (1420 ms / 710 ms), every
  part ends with the move, the move has no stagger, phases follow one another.

Kept: straight lines, fades in place, no scaling, all motion through `motion`; reduced motion
applies steps at once and skips the timeline.

Still varies, and why: about ±3 ms per phase, from timer granularity and the main thread (a phase
deadline is a `setTimeout` that can fire a frame late). An animation can finish a frame after its
phase deadline; its cleanup lands inside Settle, which absorbs it. The α step is one full step
longer than a plain one by design.

### Legibility, verified frame by frame

A probe (not kept as a gate; it ran from a scratch script over `browser.mjs`) records, every
animation frame of every firing, the on-screen box of each drawn node (by uid, with its effective
opacity) and of each ghost, and splits the frames by the `Cue` marks. For nodes visible throughout
it checks: nothing moves during Hold or Settle; on the first and last frames of the move no node
covers more than a fifth of its total travel (a jump); the path a node travels is no longer than
the straight line between where it was and where it lands (a detour or a second movement). For
ghosts it reports when each starts and stops moving.

- Found: ι reused `s` and `n` by reference (`clone`), so after `Nat.rec z s (succ n)` fired the
  term held two copies of `s` with the same uids. The workbench keys the old layout by uid, so the
  glide sent a copy from where the other copy was: in `Nat.rec z f 3`, the `f` outside the next
  redex would slide in from the redex's `f`. The JS original did the same, so the frozen fixtures
  had it too. Fixed: ι copies `z`, `s` and `n` (`Term::copy`, which records `from`) like every
  other rule. `crates/explore/tests/engine.rs` now checks that no step of any example's run leaves
  one uid in two places. The fixtures changed deliberately: 20 random fixtures, 75 engine values
  and 58 blocks views, every difference only in fresh ids and the new `data-from` (checked by
  stripping ids and comparing).
- Found: a renaming substituted a fresh variable and then prepared the result, so renamed nodes
  recorded a never-drawn intermediate as `from`, and nothing could fly to them. Fixed: `derived`
  records what a just-built node was itself built from, and a renaming derives each rewritten
  variable from the occurrence it replaces. A second engine test checks that every `from` after
  any step (a capturing β or ζ as α, then the step) names a node of the drawn term or a
  definition. Five fixtures changed, in renamed nodes' `from` only.
- Found: the automatic α was a hold followed by an instant swap. The renamed term replaced the
  whole redex with fresh uids, so nothing could glide, and a wider name (`y` → `y₁`) shoved
  everything after it sideways in one frame. Fixed: α is a firing like the others. Its rule now
  has bindings on a λ or let (`x` the binder, `t` the body). The workbench replaces only the
  smallest subtree whose text changes, so the redex and λ keep their uids. The α step then runs
  the same hold, lift, move and settle as β: the binder and body are painted during Hold, the body
  travels to where it lands, the old binder fades as the new one appears, and the context glides
  around the wider name. `tempo.mjs` shows α as a row of its own, with the same timings as any
  step.
- Measured after these fixes, on `mix`, `beta`, `church`, `delta`, `alpha`, `capture`, a free-`f`
  `Nat.rec`, a let-capture and `double (double 2)`, every step by Step: no node visible
  throughout moves during Hold or Settle, no first or last frame of a move carries a node more
  than a fifth of its way, and no node's path is longer than its straight line. Every ghost of a
  firing starts on the same frame (within one frame, 16 ms). The ghosts stop moving visibly at
  different frames because the ease-out makes a short trip look still sooner than a long one;
  all share one window, `Role::Fly`.

### Height, fixed

- Choice: reserved space everywhere, plus one tabbed lower pane. Nothing is overlaid on the term.
  Every region has a fixed height and scrolls inside it: the program band (5.6rem: notes, then
  each `def` as one line of source, horizontally scrollable, instead of a wrapping card per def),
  the rule card (4.4rem; 5.6rem on narrow screens), the stage, the status line (3.3em), and the
  pane. The pane's tabs — Trace, Why (shown only while a capture explanation exists; selected
  when one first appears), Source — share one 15rem panel, so the α panel, the source editor and a
  parse error never add rows. The trace scrolls inside the panel and keeps the current step in
  view.
- Stop is drawn over Step's box (`.rdx-stop`: transparent label, `::after` overlay), so the
  button keeps its width and the controls never rewrap at phone width.
- The stage is 20rem (320 px at the blog's 16 px root). The tallest built-in term at blog width,
  Church numerals mid-run, is 205 px of term plus 66 px of padding: 271 of 320, about 50 px to
  spare. A larger term (any user source) scrolls inside the stage, both ways; it is never scaled.
  The widest built-in term at its widest state is 1.73× the stage and scrolls horizontally.
- Workbench height: 884.17 px on the standalone page (1100 px window) and in the blog post
  (752 px column); 943.78 px at 390 px (the controls wrap to two rows, fixed from the start).

### What `scripts/steady.mjs` proves

It opens the workbench on the standalone page (1100 px), in the blog post (the local Jekyll at
:4011, `/assets/redex/` answered from `dist/site`), and on the standalone page at 390 px, with
motion on. A `requestAnimationFrame` loop compares the workbench's border-box height every frame
with the first frame's and records any difference (an exact comparison, so any change of 1 px or
more fails). It then drives, for every example: load, one Step, hover on a badge, Normalize, Back
twice, de Bruijn on and off, a rename, links off and on, To WHNF, Reset, the trace tab; then the
automatic α with its Why panel and tab switches, a long trace (`double (double 2)`, more rows than
the panel shows) normalized and revisited by clicking an early row, a parse error, and a fixed
source. Each action waits for `aria-busy` to clear, so every frame of every animation is sampled
(about 5,600 frames per target). It fails on any height change, any page error, or any action
that throws, and names the action during which the height changed.

On macOS, Chrome's renderer is suspended while the display sleeps: in 3 s of wall time a headless
page saw 0.84 s of `performance.now` and 38 frames, so a firing's deadlines never arrived and
long runs stalled at random actions with `aria-busy` still set. Not a workbench bug; `browser.mjs`
now holds `caffeinate -dimu` for the life of the check, after which the same page sees 3.0 s and
180 frames.

The post has since moved to `/program-as-proof/`; `scripts/browser.mjs` points there (override
with `BLOG=`).

Final run: page 884.17 px, blog 884.17 px, narrow 943.78 px, no change on any frame.

Shown to catch a regression: the same run with `.redex .rdx-status { height: auto }` injected
fails: the status line starts at its content's height (856.05 px workbench) and grows a line during
`omega`'s Normalize (882.88 px), and the check names that action and the ones after it.

It does not cover the graph and race widgets, user sources other than the ones above, window
resizes, or font loading (it starts after `document.fonts.ready`).
