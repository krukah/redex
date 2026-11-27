# widgets

```mermaid
flowchart LR
  Session -->|context| Rules & Workbench & Graph & Race
  Workbench --> Blocks -->|markup::Node| Doc -->|fill| DOM
  Workbench & Graph & Race --> motion
  Doc -->|read Data| typed[Path, Uid, Kind, Step, Id …]
```

Leptos components: `Deck` (rules), `Workbench`, `Paths` (graph), `Lanes` (race). Every widget reads and writes the shared `Session` (program, history, cursor,
capture, busy) from context and never touches another widget's DOM. Term-shaped regions are `markup::Node`s that `Doc::fill` builds into real elements; data
attributes are read back with `Doc::read` into their types and queried with `markup::Query`.
`redex.css` here is the one stylesheet; every selector starts with `.redex`. Exempt from `one-type-per-file` and `pub-field`
(see House Lint), not from `free-function`.

The blocks markup in `fixtures/engine` is frozen regression data; `tests/blocks.rs` checks the
renderer against it byte for byte, and it changes only deliberately, from the Rust side.
