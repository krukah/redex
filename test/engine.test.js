import test from 'node:test';
import assert from 'node:assert/strict';
import {
  applicativeNext, exploreGraph, fv, key, normalNext, parseProgram, parseTerm, prepare, runStrategy,
  substNaive, text, whnfNext,
} from '../src/engine.js';
import { EXAMPLES, RACES } from '../src/examples.js';

const run = (src, next = normalNext, cap = 200) => {
  const p = parseProgram(src);
  const r = runStrategy(p.term, p.env, next, cap);
  return { status: r.status, steps: r.rows.length - 1, last: text(r.rows.at(-1).term), kinds: r.rows.slice(1).map(x => x.kind) };
};
const src = id => EXAMPLES.find(e => e.id === id).src;

test('every example and race term parses', () => {
  for (const e of EXAMPLES) assert.doesNotThrow(() => parseProgram(e.src), e.id);
  for (const r of RACES.filter(r => r.src)) assert.doesNotThrow(() => parseProgram(r.src), r.id);
});

test('normal forms of the examples', () => {
  const expected = {
    alpha: 'λy₁. y',
    beta: '((a, a), (a, a))',
    eta: 'λz. g z',              // strategies never perform η
    delta: 'a',
    zeta: '(b, a)',
    iota: '4',
    proj: '(b, a)',
    etas: '(a, b)',
    diamond: 'a a',
    lazy: 'z',
    capture: 'λy₁. λz₁. y z y₁ z₁',
    church: 'λf. λx. f (f (f (f x)))',
    mix: '2',
  };
  for (const [id, nf] of Object.entries(expected)) {
    const r = run(src(id));
    assert.equal(r.status, 'done', id);
    assert.equal(r.last, nf, id);
  }
});

test('Ω has no normal form', () => {
  assert.equal(run(src('omega'), normalNext, 30).status, 'cap');
});

test('each rule fires in its own example', () => {
  assert.deepEqual(run(src('delta')).kinds, ['delta', 'beta', 'beta', 'delta', 'beta', 'delta', 'beta']);
  assert.deepEqual(run(src('zeta')).kinds, ['zeta', 'proj', 'proj']);
  assert.ok(run(src('iota')).kinds.includes('iota'));
});

test('strategies: normal order duplicates work, applicative order loops on unused arguments, whnf stops at the head', () => {
  const dup = '(λx. x x x) ((λy. y) a)';
  assert.equal(run(dup, normalNext).steps, 4);
  assert.equal(run(dup, applicativeNext).steps, 2);
  assert.equal(run(dup, whnfNext).last, 'a ((λy. y) a) ((λy. y) a)');
  const lazy = '(λx. z) ((λw. w w) (λw. w w))';
  assert.equal(run(lazy, normalNext).last, 'z');
  assert.equal(run(lazy, applicativeNext, 24).status, 'cap');
  assert.equal(run('λz. (λx. x) ((λy. y) z)', whnfNext).steps, 0);
});

test('β renames binders that would capture, and naive substitution captures', () => {
  const t = parseTerm('(λx. λy. x) y');
  const log = [];
  const body = prepare(t.f.b, t.f.x, fv(t.a), log);
  assert.deepEqual(log, [['y', 'y₁']]);
  assert.equal(text(body), 'λy₁. x');
  assert.equal(text(substNaive(t.f.b, t.f.x, t.a)), 'λy. y');
});

test('α-equivalent terms share a key', () => {
  assert.equal(key(parseTerm('λx. λy. x y')), key(parseTerm('λa. λb. a b')));
  assert.notEqual(key(parseTerm('λx. λy. x')), key(parseTerm('λx. λy. y')));
});

test('confluence: no example graph has two normal forms', () => {
  for (const e of EXAMPLES) {
    const p = parseProgram(e.src);
    const g = exploreGraph(p.term, p.env, !!e.eta, 70);
    const nfs = [...g.nodes.values()].filter(n => n.nf);
    assert.ok(nfs.length <= 1, `${e.id}: ${nfs.map(n => text(n.term)).join(' | ')}`);
  }
});

test('parser accepts Lean syntax', () => {
  assert.equal(text(parseTerm('fun x y => x')), 'λx. λy. x');
  assert.equal(text(parseTerm('let p := (a, b, c); p.2.1')), 'let p := (a, (b, c)); p.2.1');
  assert.equal(text(parseProgram('Nat.rec 0 (fun k r => Nat.succ r) 3').term), 'Nat.rec 0 (λk. λr. succ r) 3');
  assert.throws(() => parseTerm('λ. x'), /variable after λ/);
});

test('provenance: β copies of the argument point back to it, and the body to the old body', async () => {
  const { contract, bindings, schemaFor, schemaVars } = await import('../src/engine.js');
  const t = parseTerm('(λf. f (f a)) (λy. y)');
  const r = contract(t, 'beta', new Map());
  assert.equal(r.from, t.f.b.uid);                 // the result root came from the body
  assert.equal(r.f.from, t.a.uid);                 // first f became a copy of the argument
  assert.equal(r.a.f.from, t.a.uid);               // so did the second
  assert.deepEqual(bindings(t, 'beta').map(b => b.name), schemaVars(schemaFor(t, 'beta')));
});

test('every rule binds exactly the pattern variables its schema mentions', async () => {
  const { bindings, schemaFor, schemaVars, redexes } = await import('../src/engine.js');
  for (const e of EXAMPLES) {
    const p = parseProgram(e.src);
    for (const r of redexes(p.term, p.env)) {
      let node = p.term; for (const c of r.path) node = node[c];
      const vars = schemaVars(schemaFor(node, r.kind));
      for (const b of bindings(node, r.kind)) assert.ok(vars.includes(b.name), `${e.id} ${r.kind} ${b.name}`);
    }
  }
});
