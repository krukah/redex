# lints

The `House Lint` CI job: a diff-scoped linter for the house style a machine can
decide. It parses Rust and TypeScript with tree-sitter and reads a PR's diff,
so it judges what you wrote rather than what the repo already holds.

The crate sits **outside the workspace** (`exclude` in the root `Cargo.toml`).
tree-sitter builds C, and a workspace member would pay that cost in every
compile-matrix job. Run it by manifest path:

```bash
# what CI runs — your diff against master
cargo run --manifest-path crates/lints/Cargo.toml -- --base master

# one rule, across every tracked file
cargo run --manifest-path crates/lints/Cargo.toml -- --all --only free-function

# rank every function, method, and closure by branch complexity
cargo run --manifest-path crates/lints/Cargo.toml -- --all --census

# add an opt-in rule to your own diff
cargo run --manifest-path crates/lints/Cargo.toml -- --enable forwarding-closure
```

`--only` implies `--enable`, so naming a rule is enough to see it.

Every rule opts out with `// lint:allow(<rule>) <reason>` on the line above the
construct. The reason is not optional, and the directive covers only the
construct it sits above.

`CLAUDE.md` beside this file has what each rule decides, what gates a PR, and
how to promote a rule.
