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
