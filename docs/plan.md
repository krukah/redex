# Redex → Rust: port plan

Brief for a long-running agent. Port the whole Redex Workbench (`~/Code/redex`, ~2.3k lines of
JS/CSS) to a Rust workspace with a Leptos frontend, behaviour-identical, then delete the JS.

## Decisions

| Question | Decision | Why |
|---|---|---|
| Framework | **Leptos 0.8, CSR** (`features = ["csr"]`), built with Trunk | Fine-grained signals, no virtual DOM: an element stays the same element while `motion` animates it, and `NodeRef`s give direct handles for FLIP. Dioxus diffs a VDOM and would fight hand-driven animation. Same stack as `noether-labs/crates/client`. |
| Where | The `redex` repo, branch `rust`. JS stays in place as the reference until cutover. | The JS is the oracle for parity tests and side-by-side screenshots. |
| Style source | `noether-labs`: `rustfmt.toml`, `[workspace.lints]`, the Taste and Comments sections of `AGENTS.md`, House Lint | The user's own house style. |
| CSS | Keep `redex.css` as CSS. Port selectors and `--rdx-*` tokens unchanged. | Host-page isolation rules are already right; rewriting CSS in Rust buys nothing. |
| Lints | House Lint vendored into `crates/lints`, enforced in CI | User decision. |
| Embedding | `redex::mount_all()` mounts every `[data-redex]` element on the page with one shared session, as now | The Jekyll post contract stays the same, with `.js + .wasm` in place of ES modules. |

## Workspace

Deep, narrow crates. Each crate answers one question, exposes few types, and hides a lot. The
engine crates have no `web-sys`/`leptos` dependency and test natively.

```
crates/
  term/       Term tree, Uid, provenance (`from`), Path, size, free vars, fresh names,
              α-equivalence key. Immutable `Rc` nodes.
  subst/      Capture-avoiding substitution, renaming (`prepare`), the naive capturing variant
              that the "capture" demo shows.
  syntax/     Tokenizer, parser, Program (defs + `--` notes), text printer.
  rules/      Rule enum (α β η δ ζ ι π), schemas, bindings, redex search, contraction, Env.
  strategy/   Normal, applicative and WHNF order, and the step-capped runner. Never η.
  explore/    Reduction graph: exploration to a cap, edges by rule, cycle detection.
  layout/     Pure geometry: graph rows/columns/loop-back routes, thumbnail layout. No DOM.
  examples/   Example and race programs as data (sources stay as text in the original syntax).
  motion/     Motion algebra on WAAPI via web-sys: primitives (enter, exit, slide, resize, fly,
              crossfade, draw), compositions (all, seq, after, stagger, act), Clock, BEAT, EASE,
              ghost + delta for FLIP. The only crate that calls `Element::animate`.
  widgets/    Leptos components: blocks, rules deck, workbench, graph, race, and Session (shared
              context: program, history, cursor, capture, busy).
  redex/      Facade: `mount_all`, re-exports. cdylib for wasm.
bin/
  page/       The standalone page (index.html + site.css), served by Trunk.
```

Dependency direction: `term ← subst ← rules ← strategy ← explore ← layout`, `term ← syntax`,
`syntax ← examples`; `motion` and everything above → `widgets` → `redex` → `page`. No cycles,
and nothing below `widgets` knows about the DOM.

Each crate gets an `AGENTS.md` that opens with a small diagram, and a row in the root index.

## House style (copy, don't paraphrase)

1. Copy from `~/Code/noether-labs`: `rustfmt.toml`, `rust-toolchain.toml`, and the whole
   `[workspace.lints]` block (clippy `pedantic` + its allow list), and add `-D warnings` in CI.
2. Root `AGENTS.md` = the Redex rules from the current `AGENTS.md` (rewritten for Rust: "all
   motion through `motion`", "engine crates stay DOM-free", "three color channels", "one firing
   grammar", "class prefixes and `--rdx-*` tokens") + the noether Taste and Comments sections
   verbatim: one type per file (`widgets` exempt, as `client` is), no free functions except
   `#[component]`, no pub fields, `where` bounds, glob re-exports in `mod.rs`, one-word names,
   no single-use bindings, chains over loops, no blank lines in bodies, comments by default absent.
3. House Lint: vendor `noether-labs/crates/lints` into `crates/lints`, outside the workspace
   (`exclude`), as noether-labs does. Point its repo assumptions (base branch, `client` exemption →
   `widgets`) at redex. CI runs it diff-scoped against the base.
4. `CLAUDE.md` = one line: `@AGENTS.md`.

## Verification

- **Golden fixtures.** Before any Rust, add `scripts/golden.mjs`: for every example and race,
  and for a seeded set of random terms, dump JSON from the JS engine: parsed term (printed),
  redexes (path, kind), each one-step result, bindings per redex, strategy step counts and
  normal forms, graph nodes/edges/cycle flag. Commit to `fixtures/`. Rust engine crates must
  match it exactly (`cargo test`). Port `test/engine.test.js` cases as well.
- **Gates per commit:** `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D
  warnings`, `cargo test --workspace`, `trunk build`, House Lint against the branch base.
- **Visual parity.** For each widget, screenshot JS and Rust pages side by side with a headless
  browser at the same states (example loaded, hover on a badge, mid-firing, graph, race end).
  Record differences in `docs/port.md`. Motion parity is judged by timing constants and
  composition structure, not frames.
- **Size.** Report the gzipped `.wasm` size at each milestone. Release profile: `opt-level = "z"`,
  LTO, `codegen-units = 1`, `wasm-opt`.

## Milestones

Each ends with a green gate and a commit. The agent keeps `docs/port.md` as a ledger: milestone,
status, decisions taken, parity gaps.

0. **Scaffold.** Branch, workspace, config files, AGENTS.md, CI workflow, empty crates that
   build to wasm. Golden fixture script and fixtures.
1. **Engine.** `term`, `subst`, `syntax`, `rules`, `strategy`, `explore`. All goldens pass.
2. **Geometry and data.** `layout`, `examples`. Layout tested against JS coordinates in goldens.
3. **Motion and blocks.** `motion`; the blocks view; `page` renders a static term with the
   ported CSS. Session as Leptos context.
4. **Workbench.** Hover-to-match, bind, move, rename, history, keys. The hardest widget: one
   firing grammar for all seven rules, no rule-specific animation code.
5. **Rules deck, graph, race.** One at a time, each with a parity screenshot pass.
6. **Embedding.** `mount_all` over `[data-redex]`, several widgets sharing one session, a build
   output for the Jekyll site, and the single-file artifact bundle (`scripts/bundle`).
7. **Cutover.** Delete `src/`, `test/`, `package.json`, the JS bundler; update README and AGENTS.
   Stop before this step and ask the user.

## Agent rules of engagement

- Work only in `~/Code/redex` on branch `rust`. Commit at each green gate; never commit red.
- Never push. Commits stay local; the user pushes.
- Stop and ask when: a JS behaviour looks like a bug (port it faithfully first, flag it); a
  house rule conflicts with Leptos ergonomics; the wasm exceeds 500 KB gzipped; a milestone
  needs a new dependency outside leptos/web-sys/wasm-bindgen/js-sys/serde.
- Do not change behaviour, copy, colors or timing during the port. Improvements go in
  `docs/todo.md`.
