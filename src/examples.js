// Example programs. Each one is self-contained: its definitions travel with it.
// Comment lines (`-- ...`) become the caption above the workbench.

export const RULES = [
  {
    id: 'alpha', k: 'alpha', name: 'alpha', sub: 'rename a binder', schema: 'λx. t ≡ λy. t[x:=y]',
    src: `-- The argument y is free. Dropping it under λy would capture it.
-- Fire β: the engine renames the inner binder first.
-- Click any binder to rename it yourself. The links never move.
(λx. λy. x) y`,
  },
  {
    id: 'beta', k: 'beta', name: 'beta', sub: 'apply a function', schema: '(λx. t) s ⟶ t[x:=s]',
    src: `-- f appears twice, so the argument is copied twice.
(λf. f (f a)) (λy. (y, y))`,
  },
  {
    id: 'eta', k: 'eta', name: 'eta', sub: 'drop a wrapper', schema: 'λx. f x ⟶ f', eta: true,
    src: `-- Two η-redexes around a β-redex. Every route ends at g.
-- Step skips η, as Lean does: η is for comparing terms, not computing.
-- Click the η marks yourself.
λz. (λy. g y) z`,
  },
  {
    id: 'delta', k: 'delta', name: 'delta', sub: 'unfold a definition', schema: 'c ⟶ body of c',
    src: `-- Constants unfold to their definitions, one at a time.
def id := λx. x
def twice := λf. λx. f (f x)
twice id a`,
  },
  {
    id: 'zeta', k: 'zeta', name: 'zeta', sub: 'inline a let', schema: 'let x := v; t ⟶ t[x:=v]',
    src: `-- let x := v; t acts like (λx. t) v, but it is a separate rule.
let p := (a, b); (p.2, p.1)`,
  },
  {
    id: 'iota', k: 'iota', name: 'iota', sub: 'eliminate a constructor', schema: 'Nat.rec z s (succ n) ⟶ s n (Nat.rec z s n)',
    src: `-- The recursor fires only when its target is a constructor.
def double := λn. Nat.rec 0 (λk r. succ (succ r)) n
double 2`,
  },
  {
    id: 'proj', k: 'proj', name: 'proj', sub: 'project from a pair', schema: '(a, b).1 ⟶ a',
    src: `-- Projections fire when they meet a pair.
(λp. (p.2, p.1)) (a, b)`,
  },
];

export const MORE = [
  {
    id: 'etas', label: 'Structure η', eta: true,
    src: `-- (p.1, p.2) is p. Here η and two π steps reach the same pair.
((a, b).1, (a, b).2)`,
  },
  {
    id: 'diamond', label: 'Diamond',
    src: `-- Two redexes, two orders, one answer.
(λx. x x) ((λy. y) a)`,
  },
  {
    id: 'omega', label: 'Ω loops',
    src: `-- Ω reduces to itself, so it has no normal form.
(λx. x x) (λx. x x)`,
  },
  {
    id: 'lazy', label: 'Lazy escape',
    src: `-- The inner redex loops forever. The outer one throws it away.
(λx. z) ((λw. w w) (λw. w w))`,
  },
  {
    id: 'capture', label: 'Deep capture',
    src: `-- Two binders in the way. Both get renamed before β.
(λx. λy. λz. x y z) (y z)`,
  },
  {
    id: 'church', label: 'Church 2 + 2',
    src: `-- Numbers as iterators: n f x applies f n times.
def two := λf. λx. f (f x)
def plus := λm. λn. λf. λx. m f (n f x)
plus two two`,
  },
  {
    id: 'mix', label: 'All rules',
    src: `-- δ, β, ζ, π and ι in one term.
def double := λn. Nat.rec 0 (λk r. succ (succ r)) n
let p := (double 1, b); p.1`,
  },
];

export const EXAMPLES = [...RULES, ...MORE];

// Terms for the strategy race. `bench` means "whatever the workbench started from".
export const RACES = [
  { id: 'lazy', label: 'Discarded loop', src: '(λx. z) ((λw. w w) (λw. w w))' },
  { id: 'dup', label: 'Duplicated work', src: '(λx. x x x) ((λy. y) a)' },
  { id: 'under', label: 'Under a binder', src: 'λz. (λx. x) ((λy. y) z)' },
  { id: 'bench', label: 'Workbench term' },
];
