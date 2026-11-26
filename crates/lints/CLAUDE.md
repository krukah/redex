# lints

Vendored from `noether-labs/crates/lints` (itself ported from the `novig`
monorepo). Only `repo.rs` knows about redex: the base branch is `master`, and
the Leptos exemptions name `crates/widgets`. Rules live in two
places, and each rule lives in exactly one of them:

| Home | Expresses | Opt out with |
|---|---|---|
| `[workspace.lints.clippy]` in the root `Cargo.toml` | a lint the compiler already knows | `#[allow(clippy::…)]` and a comment |
| this crate | a shape in the AST, or a comment | `// lint:allow(<rule>) <reason>` |

## What gates a PR

Every rule outside `OPT_IN_RULES`, on every file a diff touches. The comment
rules judge the comment blocks the diff reaches; every other rule judges the
whole file. A file the PR creates, renames, or moves answers for every line.
The standing count of each of these rules is zero, so a PR pays only for what
it writes.

## What does not

Three shape rules read the Taste section of the root `AGENTS.md` and sit in
`OPT_IN_RULES`. A rule there fires only when `--enable` or `--only` names it, so
CI never sees one. They are here to be read, not to block.

| Rule | Finds | Today |
|---|---|---|
| `forwarding-closure` | `\|d\| d.magnitude()`, which a path says better | 0 |
| `type-hint` | `let a: Vec<_> = b.collect()`, which a turbofish says better | 0 |
| `inline-bound` | `fn f<T: Clone>`, which a `where` clause says better | 0 |

To start gating one, drive its count to zero, then move it out of
`OPT_IN_RULES` in `rules.rs`. In redex every count starts at zero, and the root
`AGENTS.md` already states `inline-bound` and `type-hint` as absolute.

## The standing count

Zero, for every gating rule, across every tracked file — `--all` reports no
violations. Keep it there: there is no baseline file and there will not be one,
and no `lint:allow` stands anywhere in the repo.

## free-function

A function is free when nothing owns it — its home is a file or a module, not
an `impl` or a `trait`. Hang it on a type: a method when it has a receiver, an
associated function when it constructs or relates to one type, a trait when
several types answer the same question. `foo(x)` hides what `x.foo()` states.

Exempt: `fn main`, a `#[test]` function, a `#[proc_macro*]` entry point, and a
function whose macro requires a bare `fn` — `#[component]`, `#[server]` and
`#[island]`, in `FRAMEWORK_ATTRIBUTES`. The rule skips `benches/`, where
`criterion_group!` takes a path to a bare `fn`.

It does **not** skip `crates/widgets`. A Leptos `#[component]` is already exempt;
every other helper in that crate is a free function like any other.

## one-type-per-file

A file holds one type. The rule judges a **module body** — the file's own scope
or an inline `mod` — and counts **families**: a trait and the types beside it
that implement it. The **namesake** owns the body, so `size.rs` belongs to
`Size` whatever else it holds; failing a namesake, the largest family owns it,
and two of a size, the one declared first. Every other type is a violation, and
the report names the family it lost to.

Every declaration counts: private types, markers, test fixtures, and types
inside a function body all crowd the body that holds them. A PR that touches a
file answers for every type in it, not just the lines it changed.

Scoped out of `crates/widgets`: a Leptos file holds the component and the types
it renders.

Two ways out, in order:

1. **Design.** Move the type to its own file, or let the trait it implements
   gather it. A fix that changes the code beats one that annotates it.
2. **`// lint:allow(one-type-per-file) <reason>`** above the declaration.

## pub-field

A struct keeps its fields to itself. Every visibility modifier exposes one, so
`pub(crate)` and `pub(super)` answer to the rule exactly as `pub` does. Read a
field through an accessor, and build the struct through `From`, `new`, or a
builder.

A tuple field answers to nothing. `Foo(Inner)` is the house newtype, and it is
read by destructuring, not by `.0`.

Scoped out of `crates/widgets`: the `view!` macro fills a props struct field by
field.

## field-access

`self.j` is read as `self.j()`, everywhere: any receiver, test code, and the
token tree of a macro (`format!("{}", self.j)`). The one exempt site is the
accessor's own body — a function whose only parameter is `self` and whose body
is `self.f`, `&self.f`, `&mut self.f`, or one argument-free method on it
(`self.f.as_slice()`). Writes go through `f_mut()`, or better, rebuild by value
(`Self { f, ..self }`) or fold a step method (`Hamerly::fold`). A tuple index
(`self.0`) answers to nothing. A foreign type with no method for a field is
read by destructuring it in a pattern.

Opt-in: a type's own impl may touch its fields. Ask for it by name to audit a diff.

Two ways out, in order:

1. **A DTO derive.** A struct that a wire format, a database driver, a
   framework, or a CLI parser fills field by field is a boundary DTO, and the
   derive is the machine-readable mark of one: `Serialize`, `Deserialize`,
   `FromRow`, `QueryRow`, `IntoParams`, `ToSchema`, `FromRef`, `Parser`,
   `Args`, `Subcommand`. A struct that carries domain logic gets private fields
   even when it derives one of these.
2. **`// lint:allow(pub-field) <reason>`**. Over the struct it exempts every
   field; over one field it exempts that field.

## branch-complexity

Every function, method, and closure scores by its branches: one point to enter,
plus one for each `if`, loop, `&&`, `||`, match guard, and `let`-else. A `match`
arm counts only when its body branches further, so an exhaustive `match` over a
domain enum scores 1 and a dispatch table needs no exemption. A `?` scores
nothing. A nested closure scores as its own unit.

A file fails when a unit scores above 15. `--all --census` ranks every unit,
worst first.

## wiring-file and entry-file

A file that Cargo or the module tree names holds no logic of its own.

`wiring-file` judges `lib.rs` and `mod.rs`. The file may hold `mod a;`, a `use`,
a re-export, an `extern crate`, an attribute, and a comment. Every other item
fails: a `fn`, a type, a `const`, a `static`, an `impl`, a macro invocation, an
inline `mod a { … }` with a body, a `#[cfg(test)] mod tests` included. Rust
keeps one exception: a `#[proc_macro*]` function must sit in the crate root.
`entry-file` judges `main.rs` and allows the same wiring, plus `fn main`.

Move the item to its own file and re-export it.

Note the tension with the root `AGENTS.md`, which asks a `mod.rs` to carry
`pub use module::*;` — a re-export, which the rule allows. It is the `const`,
the `impl`, and the inline `mod tests` that fail.

## Scoping a rule out of a path

`SCOPED_OUT` in `repo.rs` maps a rule to the path prefixes it skips. It is the
only place a rule learns about the repo's layout, and every entry carries its
reason beside it.

Two prefixes sit there today. `crates/widgets` skips `one-type-per-file` and
`pub-field`. `benches` skips `free-function`. Nothing else is scoped out, this
crate included: it passes every rule it enforces, opt-in ones aside.

## Layout

The crate holds itself to its own rules, so `lib.rs`, `main.rs` and each
`mod.rs` only wire modules together, and every function hangs on a type.

| File | Holds |
|---|---|
| `rules.rs` | every rule name, `OPT_IN_RULES`, the comment caps |
| `lint.rs` | `Language::lint_file`, the one entry point, and the comment-rule tests |
| `block.rs`, `check.rs` | a comment `Block`: how one is read, and the comment rules it answers to |
| `directive.rs` | `lint:allow` parsing |
| `entry.rs`, `crowd_dir.rs` | `wiring-file` / `entry-file`, and `Violation::crowding` |
| `diff.rs` | `Diff`: the lines a unified diff adds |
| `complexity/`, `shape/` | `branch-complexity`, and the AST shape rules; `language.rs` in each holds the `impl Language` |
| `args.rs`, `repo.rs`, `census.rs` | the binary: CLI flags, the git checkout and its scoped-out paths, `--census` |

## Adding a rule

A shape rule is a node kind and a predicate. Add the variant to `shape::Kind`,
its node kind to `Scan::kind`, and the rule name to both `RULES` and
`OPT_IN_RULES` in `rules.rs`. Land it opt-in, measure it with `--all --only`, and
promote it once the count on the files a PR touches is one you would enforce.

Test it against the workspace before you ship it. A rule that fires on a correct
file teaches people to suppress rules.
