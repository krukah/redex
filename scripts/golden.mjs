// Golden fixtures for the Rust port: what the JS engine says about every example, every race and a
// seeded set of random terms. The Rust engine crates must reproduce these files exactly.
// Usage: node scripts/golden.mjs   (writes fixtures/engine/*.json)
//
// Node ids are global counters, so every id in a case is written relative to a probe node made at
// the start of the case: equal files mean equal allocation order, and so equal provenance.
import { mkdirSync, rmSync, writeFileSync } from 'node:fs';
import {
  A, C, L, LET, P, PR, V, applicativeNext, at, bindings, exploreGraph, fv, hasCycle, key, normalNext,
  parseProgram, prepare, redexes, runStrategy, schemaFor, schemaVars, size, step, substNaive, text, whnfNext,
} from '../src/engine.js';
import { renderBlocks } from '../src/blocks.js';
import { EXAMPLES, RACES } from '../src/examples.js';

const OUT = new URL('../fixtures/engine/', import.meta.url);
const KIDS = { var: [], const: [], lam: ['b'], app: ['f', 'a'], let: ['v', 'b'], pair: ['a', 'b'], proj: ['t'] };
const STRATEGIES = { normal: normalNext, applicative: applicativeNext, whnf: whnfNext };

// One line per term: kind, label, relative id, provenance, mark and capture flag, then children.
const dump = (t, base) => {
  const label = t.k === 'var' || t.k === 'const' ? t.n : t.k === 'lam' || t.k === 'let' ? t.x : t.k === 'proj' ? String(t.i) : '';
  const meta = `#${t.uid - base}${t.from ? `<${t.from - base}` : ''}${t.mark ? `!${t.mark}` : ''}${t.cap ? '^' : ''}`;
  const kids = KIDS[t.k].map(c => dump(t[c], base));
  return `${t.k}:${label}${meta}${kids.length ? `(${kids.join(' ')})` : ''}`;
};
const rebase = (html, base) => html.replace(/data-(uid|from)="(\d+)"/g, (_, a, n) => `data-${a}="${n - base}"`);

function run(t, env, next, cap) {
  const r = runStrategy(t, env, next, cap);
  return { status: r.status, rows: r.rows.map(x => [x.kind, text(x.term)]) };
}

function graph(t, env, eta, cap) {
  const G = exploreGraph(t, env, eta, cap), keys = [...G.nodes.keys()], ix = k => keys.indexOf(k);
  return {
    truncated: G.truncated,
    cycle: hasCycle(G),
    nodes: [...G.nodes.values()].map(n => [n.key, text(n.term), n.depth, n.parent === null ? null : ix(n.parent), n.pkind ?? null, !!n.nf, !!n.open]),
    edges: G.edges.map(e => [ix(e.from), ix(e.to), e.kind]),
  };
}

function study(id, src, eta) {
  const base = V('_').uid;
  let prog;
  try { prog = parseProgram(src); } catch (e) { return { id, src, error: e.message }; }
  const { term, env, notes } = prog;
  const out = {
    id, src, eta, notes,
    env: [...env].map(([n, b]) => ({ name: n, text: text(b), tree: dump(b, base), blocks: rebase(renderBlocks(b, { env }), base) })),
    text: text(term), key: key(term), size: size(term), tree: dump(term, base),
    fv: [...fv(term)].sort(),
    blocks: rebase(renderBlocks(term, { live: true, env }), base),
    debruijn: rebase(renderBlocks(term, { live: true, env, db: true }), base),
    redexes: redexes(term, env).map(r => {
      const node = at(term, r.path), schema = schemaFor(node, r.kind);
      const entry = {
        path: r.path, kind: r.kind, schema, vars: schemaVars(schema),
        bindings: bindings(node, r.kind).map(b => [b.name, b.rel, !!b.binder]),
      };
      if (r.kind === 'beta' || r.kind === 'zeta') {
        const lam = r.kind === 'beta' ? node.f : node, arg = r.kind === 'beta' ? node.a : node.v;
        const log = [], body = prepare(lam.b, lam.x, fv(arg), log);
        const naive = substNaive(lam.b, lam.x, arg);
        Object.assign(entry, { log, prepared: dump(body, base), naive: dump(naive, base), naiveText: text(naive) });
      }
      const result = step(term, r, env);
      return Object.assign(entry, { result: dump(result, base), text: text(result), blocks: rebase(renderBlocks(result, { live: true, env }), base) });
    }),
    strategies: Object.fromEntries(Object.entries(STRATEGIES).map(([n, next]) => [n, { 24: run(term, env, next, 24), 200: run(term, env, next, 200) }])),
    graphs: { plain: graph(term, env, false, 70), eta: graph(term, env, true, 70), thumb: graph(term, env, !!eta, 40) },
  };
  return out;
}

// Random terms from a seeded generator: small, closed or not, rich in redexes of every rule.
function mulberry(seed) {
  return () => {
    seed = (seed + 0x6d2b79f5) | 0;
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
function randomSource(seed) {
  const rnd = mulberry(seed), pick = xs => xs[Math.floor(rnd() * xs.length)];
  const NAMES = ['x', 'y', 'z', 'f', 'g', 'a', 'b'];
  const defs = rnd() < 0.3;
  const gen = (d, scope) => {
    const r = rnd();
    if (d <= 0 || (d < 4 && r < 0.18)) {
      const s = rnd();
      if (s < 0.6 && scope.length) return V(pick(scope));
      if (s < 0.7) return defs ? V('k') : V(pick(NAMES));
      if (s < 0.8) return C(pick(['zero', 'zero', 'succ']));
      return V(pick(NAMES));
    }
    if (r < 0.36) { const x = pick(NAMES); return L(x, gen(d - 1, [...scope, x])); }
    if (r < 0.58) return (x => A(L(x, gen(d - 1, [...scope, x])), gen(d - 1, scope)))(pick(NAMES));
    if (r < 0.70) return A(gen(d - 1, scope), gen(d - 1, scope));
    if (r < 0.78) { const x = pick(NAMES); return LET(x, gen(d - 1, scope), gen(d - 1, [...scope, x])); }
    if (r < 0.86) return P(gen(d - 1, scope), gen(d - 1, scope));
    if (r < 0.93) return PR(rnd() < 0.5 ? 1 : 2, rnd() < 0.6 ? P(gen(d - 1, scope), gen(d - 1, scope)) : gen(d - 1, scope));
    const n = Math.floor(rnd() * 3);
    let k = C('zero'); for (let i = 0; i < n; i++) k = A(C('succ'), k);
    return A(A(A(C('Nat.rec'), gen(d - 1, scope)), L('m', L('r', A(C('succ'), V('r'))))), k);
  };
  const body = text(gen(4, []));
  return `-- random ${seed}\n${defs ? 'def k := λx. λy. x\n' : ''}${body}`;
}

const ERRORS = [
  'λ. x', '(a, b', 'λx y', 'let x := a b', 'let := a; b', 'a )', '41', 'x # y', '', 'def zero := a\nzero',
  'def k := λ. x\nk', '-- only a note', 'fun x => ', '(a, b).3', 'p.12', 'a.1.2', 'λx => x', 'let x := a in x',
  'Nat.succ Nat.zero', 'f.g x', 'Foo.bar.baz y', 'x₁ y₂', 'λx. λx. x', '\\x. x', 'x ↦ y', '(λx ↦ x) a',
  'def toString := a\nb', 'def constructor := a\nb', '-- note\r\nx', 'def a\r:= b\na', 'def k:=λx. x\nk y',
  '  def   k :=  z\nk', 'def k := a\ndef k := b\nk', 'let 5 := a; b', '99999999999999999999999 x', 'a.1.12', 'x.2.1',
];

rmSync(OUT, { recursive: true, force: true });
mkdirSync(OUT, { recursive: true });
const write = (name, data) => writeFileSync(new URL(`${name}.json`, OUT), JSON.stringify(data) + '\n');
const cases = [
  ...EXAMPLES.map(e => ['example', e.id, e.src, !!e.eta]),
  ...RACES.filter(r => r.src).map(r => ['race', r.id, r.src, false]),
  ...Array.from({ length: 80 }, (_, i) => ['random', String(i + 1), randomSource(i + 1), (i % 3) === 0]),
];
for (const [group, id, src, eta] of cases) write(`${group}-${id}`, study(`${group}-${id}`, src, eta));
write('errors', ERRORS.map(src => study('error', src, false)).map(({ src, error, text, notes, env }) => ({ src, error: error ?? null, text: text ?? null, notes: notes ?? null, defs: env ? env.map(d => [d.name, d.text]) : null })));
console.log(`wrote ${cases.length + 1} fixtures to ${OUT.pathname}`);
