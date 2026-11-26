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
