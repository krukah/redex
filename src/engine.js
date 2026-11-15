// Term engine: syntax, the seven rules, strategies, parser, printer, reduction graph.
// Pure: no DOM. Tested by test/engine.test.js.
// Every node gets a fresh `uid`. A node derived from an older one records it in `from`,
// so a renderer can tell where each piece of a result came from.
let UID = 0;
const mk = o => ((o.uid = ++UID), o);
export const V = n => mk({ k: 'var', n });
export const C = n => mk({ k: 'const', n });
export const L = (x, b) => mk({ k: 'lam', x, b });
export const A = (f, a) => mk({ k: 'app', f, a });
export const LET = (x, v, b) => mk({ k: 'let', x, v, b });
export const P = (a, b) => mk({ k: 'pair', a, b });
export const PR = (i, t) => mk({ k: 'proj', i, t });
const from = (n, old) => ((n.from = old.uid), n);

export const PRIMS = new Set(['zero', 'succ', 'Nat.rec']);
export const ALIAS = { 'Nat.zero': 'zero', 'Nat.succ': 'succ' };
export const CONSTS = new Set(PRIMS);
export const EMPTY = new Map();
export const KIDS = { var: [], const: [], lam: ['b'], app: ['f', 'a'], let: ['v', 'b'], pair: ['a', 'b'], proj: ['t'] };

export function clone(t) {
  switch (t.k) {
    case 'var': return from(V(t.n), t);
    case 'const': return from(C(t.n), t);
    case 'lam': return from(L(t.x, clone(t.b)), t);
    case 'app': return from(A(clone(t.f), clone(t.a)), t);
    case 'let': return from(LET(t.x, clone(t.v), clone(t.b)), t);
    case 'pair': return from(P(clone(t.a), clone(t.b)), t);
    case 'proj': return from(PR(t.i, clone(t.t)), t);
  }
}
export function size(t) { let n = 1; for (const c of KIDS[t.k]) n += size(t[c]); return n; }
export function fv(t) {
  switch (t.k) {
    case 'var': return new Set([t.n]);
    case 'const': return new Set();
    case 'lam': { const s = fv(t.b); s.delete(t.x); return s; }
    case 'let': { const s = fv(t.b); s.delete(t.x); for (const y of fv(t.v)) s.add(y); return s; }
    default: { const s = new Set(); for (const c of KIDS[t.k]) for (const y of fv(t[c])) s.add(y); return s; }
  }
}
export function names(t, s = new Set()) {
  if (t.k === 'var') s.add(t.n);
  if (t.k === 'lam' || t.k === 'let') s.add(t.x);
  for (const c of KIDS[t.k]) names(t[c], s);
  return s;
}
export const SUBS = '₀₁₂₃₄₅₆₇₈₉';
export const stripSub = s => s.replace(/[₀-₉]+$/, '');
export function fresh(base, avoid) {
  const root = stripSub(base);
  for (let i = 1; ; i++) {
    const n = root + [...String(i)].map(d => SUBS[+d]).join('');
    if (!avoid.has(n) && !CONSTS.has(n)) return n;
  }
}
export function stripMarks(t) { delete t.mark; delete t.from; for (const c of KIDS[t.k]) stripMarks(t[c]); }

// Capture-avoiding substitution t[x := s]. Copies of s get `mark`.
export function subst(t, x, s, fvs, mark) {
  switch (t.k) {
    case 'var': {
      if (t.n !== x) return t;
      const c = clone(s); if (mark) c.mark = mark; return c;
    }
    case 'const': return t;
    case 'app': return from(A(subst(t.f, x, s, fvs, mark), subst(t.a, x, s, fvs, mark)), t);
    case 'pair': return from(P(subst(t.a, x, s, fvs, mark), subst(t.b, x, s, fvs, mark)), t);
    case 'proj': return from(PR(t.i, subst(t.t, x, s, fvs, mark)), t);
    case 'lam': case 'let': {
      const v = t.k === 'let' ? subst(t.v, x, s, fvs, mark) : null;
      let y = t.x, b = t.b;
      if (y !== x) {
        if (fvs.has(y) && fv(b).has(x)) {
          const y2 = fresh(y, new Set([...fvs, ...names(b), x]));
          b = subst(b, y, V(y2), new Set([y2]));
          y = y2;
        }
        b = subst(b, x, s, fvs, mark);
      }
      return from(t.k === 'lam' ? L(y, b) : LET(y, v, b), t);
    }
  }
}
export const substitute = (t, x, s, mark) => subst(t, x, s, fv(s), mark);

// Rename only the binders inside t that would capture a free variable of the argument.
export function prepare(t, x, fvs, log) {
  switch (t.k) {
    case 'var': case 'const': return t;
    case 'app': return from(A(prepare(t.f, x, fvs, log), prepare(t.a, x, fvs, log)), t);
    case 'pair': return from(P(prepare(t.a, x, fvs, log), prepare(t.b, x, fvs, log)), t);
    case 'proj': return from(PR(t.i, prepare(t.t, x, fvs, log)), t);
    case 'lam': case 'let': {
      const v = t.k === 'let' ? prepare(t.v, x, fvs, log) : null;
      if (t.x === x) return t.k === 'lam' ? t : from(LET(t.x, v, t.b), t);
      let y = t.x, b = t.b, renamed = false;
      if (fvs.has(y) && fv(b).has(x)) {
        const y2 = fresh(y, new Set([...fvs, ...names(b), x]));
        log.push([y, y2]);
        b = subst(b, y, V(y2), new Set([y2]), 'fl-alpha');
        y = y2; renamed = true;
      }
      b = prepare(b, x, fvs, log);
      const n = from(t.k === 'lam' ? L(y, b) : LET(y, v, b), t);
      if (renamed) n.mark = 'alphaB';
      return n;
    }
  }
}

// Substitution without renaming: shows what capture looks like.
export function tagFree(t, fr, bound = new Set()) {
  if (t.k === 'var' && !bound.has(t.n) && fr.has(t.n)) t.cap = true;
  if (t.k === 'lam' || t.k === 'let') {
    if (t.k === 'let') tagFree(t.v, fr, bound);
    const b2 = new Set(bound); b2.add(t.x); tagFree(t.b, fr, b2);
    return t;
  }
  for (const c of KIDS[t.k]) tagFree(t[c], fr, bound);
  return t;
}
export function substNaive(t, x, s) {
  switch (t.k) {
    case 'var': return t.n === x ? tagFree(clone(s), fv(s)) : clone(t);
    case 'const': return clone(t);
    case 'app': return A(substNaive(t.f, x, s), substNaive(t.a, x, s));
    case 'pair': return P(substNaive(t.a, x, s), substNaive(t.b, x, s));
    case 'proj': return PR(t.i, substNaive(t.t, x, s));
    case 'lam': return t.x === x ? clone(t) : L(t.x, substNaive(t.b, x, s));
    case 'let': return LET(t.x, substNaive(t.v, x, s), t.x === x ? clone(t.b) : substNaive(t.b, x, s));
  }
}

// α-equivalence key: de Bruijn indices.
export function key(t, ctx = []) {
  switch (t.k) {
    case 'var': { const i = ctx.lastIndexOf(t.n); return i < 0 ? '$' + t.n : '#' + (ctx.length - 1 - i); }
    case 'const': return t.n;
    case 'lam': { ctx.push(t.x); const s = 'λ(' + key(t.b, ctx) + ')'; ctx.pop(); return s; }
    case 'app': return '@(' + key(t.f, ctx) + ',' + key(t.a, ctx) + ')';
    case 'let': { const v = key(t.v, ctx); ctx.push(t.x); const s = 'L(' + v + ',' + key(t.b, ctx) + ')'; ctx.pop(); return s; }
    case 'pair': return 'P(' + key(t.a, ctx) + ',' + key(t.b, ctx) + ')';
    case 'proj': return 'π' + t.i + '(' + key(t.t, ctx) + ')';
  }
}
export const aeq = (a, b) => key(a) === key(b);

export function natVal(t) {
  let n = 0;
  while (t.k === 'app' && t.f.k === 'const' && t.f.n === 'succ') { n++; t = t.a; }
  return t.k === 'const' && t.n === 'zero' ? n : null;
}
export const isCtor = t => (t.k === 'const' && t.n === 'zero') || (t.k === 'app' && t.f.k === 'const' && t.f.n === 'succ');
export function recArgs(t) {
  if (t.k !== 'app' || t.f.k !== 'app' || t.f.f.k !== 'app') return null;
  const h = t.f.f.f;
  return h.k === 'const' && h.n === 'Nat.rec' ? [t.f.f.a, t.f.a, t.a] : null;
}

export function redexKind(t, env) {
  switch (t.k) {
    case 'app': { if (t.f.k === 'lam') return 'beta'; const r = recArgs(t); return r && isCtor(r[2]) ? 'iota' : null; }
    case 'const': return env.has(t.n) ? 'delta' : null;
    case 'let': return 'zeta';
    case 'proj': return t.t.k === 'pair' ? 'proj' : null;
    case 'lam': return t.b.k === 'app' && t.b.a.k === 'var' && t.b.a.n === t.x && !fv(t.b.f).has(t.x) ? 'eta' : null;
    case 'pair': return t.a.k === 'proj' && t.b.k === 'proj' && t.a.i === 1 && t.b.i === 2 && aeq(t.a.t, t.b.t) ? 'etaS' : null;
  }
  return null;
}
export const isEta = k => k === 'eta' || k === 'etaS';

// Each rule as a schema over pattern variables, and where each variable sits in a redex.
// `bindings` returns [{ name, rel, binder }], `rel` a path relative to the redex.
// A binder binding names the λ/let whose bound name it is.
export function schemaFor(t, kind) {
  switch (kind) {
    case 'alpha': return 'λ{x}. {t}  ≡  λ{y}. {t}[{x} := {y}]';
    case 'beta': return '(λ{x}. {t}) {s}  ⟶  {t}[{x} := {s}]';
    case 'zeta': return 'let {x} := {v}; {t}  ⟶  {t}[{x} := {v}]';
    case 'delta': return '{c}  ⟶  the definition of {c}';
    case 'iota': return recArgs(t)[2].k === 'const' ? 'Nat.rec {z} {s} 0  ⟶  {z}' : 'Nat.rec {z} {s} (succ {n})  ⟶  {s} {n} (Nat.rec {z} {s} {n})';
    case 'proj': return t.i === 1 ? '({a}, {b}).1  ⟶  {a}' : '({a}, {b}).2  ⟶  {b}';
    case 'eta': return 'λ{x}. {f} {x}  ⟶  {f}';
    case 'etaS': return '({p}.1, {p}.2)  ⟶  {p}';
  }
}
export function bindings(t, kind) {
  switch (kind) {
    case 'beta': return [{ name: 'x', rel: 'f', binder: true }, { name: 't', rel: 'fb' }, { name: 's', rel: 'a' }];
    case 'zeta': return [{ name: 'x', rel: '', binder: true }, { name: 'v', rel: 'v' }, { name: 't', rel: 'b' }];
    case 'delta': return [{ name: 'c', rel: '' }];
    case 'iota': {
      const zs = [{ name: 'z', rel: 'ffa' }, { name: 's', rel: 'fa' }];
      return recArgs(t)[2].k === 'const' ? zs : [...zs, { name: 'n', rel: 'aa' }];
    }
    case 'proj': return [{ name: 'a', rel: 'ta' }, { name: 'b', rel: 'tb' }];
    case 'eta': return [{ name: 'x', rel: '', binder: true }, { name: 'f', rel: 'bf' }];
    case 'etaS': return [{ name: 'p', rel: 'at' }];
  }
  return [];
}
// Pattern variables in order of first appearance; a variable's slot color is its index.
export const schemaVars = schema => [...new Set([...schema.matchAll(/\{(\w)\}/g)].map(m => m[1]))];

export function contract(t, kind, env, mark) {
  let r;
  switch (kind) {
    case 'beta': case 'zeta': {
      const x = kind === 'beta' ? t.f.x : t.x, body = kind === 'beta' ? t.f.b : t.b, arg = kind === 'beta' ? t.a : t.v;
      r = substitute(body, x, arg, mark);
      if (mark && !fv(body).has(x)) { r = clone(r); r.mark = mark; }
      return r;
    }
    case 'delta': r = clone(env.get(t.n)); break;
    case 'iota': {
      const [z, s, n] = recArgs(t);
      r = n.k === 'const' ? clone(z) : A(A(s, n.a), A(A(A(C('Nat.rec'), z), s), n.a));
      break;
    }
    case 'proj': r = clone(t.i === 1 ? t.t.a : t.t.b); break;
    case 'eta': r = clone(t.b.f); break;
    case 'etaS': r = clone(t.a.t); break;
  }
  if (mark) r.mark = mark;
  return r;
}

export function at(t, p) { for (const c of p) t = t[c]; return t; }
export function replace(t, p, s, i = 0) {
  if (i === p.length) return s;
  const n = { ...t }; delete n.mark;
  n[p[i]] = replace(t[p[i]], p, s, i + 1);
  return n;
}
export function redexes(t, env, p = '', out = []) {
  const k = redexKind(t, env);
  if (k) out.push({ path: p, kind: k });
  for (const c of KIDS[t.k]) redexes(t[c], env, p + c, out);
  return out;
}
export const step = (t, r, env) => replace(t, r.path, contract(at(t, r.path), r.kind, env));

// Strategies. None of them performs η, the way Lean never η-reduces while computing.
export function normalNext(t, env) { return redexes(t, env).find(r => !isEta(r.kind)) || null; }
export function applicativeNext(t, env, p = '') {
  for (const c of KIDS[t.k]) { const r = applicativeNext(t[c], env, p + c); if (r) return r; }
  const k = redexKind(t, env);
  return k && !isEta(k) ? { path: p, kind: k } : null;
}
export function whnfNext(t, env, p = '') {
  switch (t.k) {
    case 'app': {
      let h = t, n = 0;
      while (h.k === 'app') { h = h.f; n++; }
      const hp = p + 'f'.repeat(n);
      if (h.k === 'lam') return { path: hp.slice(0, -1), kind: 'beta' };
      if (h.k === 'let') return { path: hp, kind: 'zeta' };
      if (h.k === 'const' && env.has(h.n)) return { path: hp, kind: 'delta' };
      if (h.k === 'proj') return whnfNext(h, env, hp);
      if (h.k === 'const' && h.n === 'Nat.rec' && n >= 3) {
        const rel = 'f'.repeat(n - 3), node = at(t, rel);
        if (isCtor(node.a)) return { path: p + rel, kind: 'iota' };
        return whnfNext(node.a, env, p + rel + 'a');
      }
      return null;
    }
    case 'let': return { path: p, kind: 'zeta' };
    case 'const': return env.has(t.n) ? { path: p, kind: 'delta' } : null;
    case 'proj': return t.t.k === 'pair' ? { path: p, kind: 'proj' } : whnfNext(t.t, env, p + 't');
  }
  return null;
}

// ===================== Parser =====================
export function tokenize(src) {
  const toks = []; let i = 0;
  const idStart = c => /[A-Za-z_Ͱ-Ͽ]/.test(c) && c !== 'λ';
  const idChar = c => /[A-Za-z0-9_'!?Ͱ-Ͽ₀-₉]/.test(c) && c !== 'λ';
  while (i < src.length) {
    const c = src[i];
    if (/\s/.test(c)) { i++; continue; }
    if (src.startsWith('--', i)) { while (i < src.length && src[i] !== '\n') i++; continue; }
    if (src.startsWith(':=', i)) { toks.push({ t: ':=' }); i += 2; continue; }
    if (src.startsWith('=>', i)) { toks.push({ t: '=>' }); i += 2; continue; }
    if (c === '↦') { toks.push({ t: '=>' }); i++; continue; }
    if (c === 'λ' || c === '\\') { toks.push({ t: 'λ' }); i++; continue; }
    if (c === '.' && /[12]/.test(src[i + 1] || '') && !/[0-9]/.test(src[i + 2] || '')) { toks.push({ t: 'proj', v: +src[i + 1] }); i += 2; continue; }
    if ('().,;'.includes(c)) { toks.push({ t: c }); i++; continue; }
    if (/[0-9]/.test(c)) { let j = i; while (j < src.length && /[0-9]/.test(src[j])) j++; toks.push({ t: 'num', v: +src.slice(i, j) }); i = j; continue; }
    if (idStart(c)) {
      let j = i + 1;
      const dotted = /[A-Z]/.test(c);
      while (j < src.length) {
        if (idChar(src[j])) j++;
        else if (dotted && src[j] === '.' && idStart(src[j + 1] || '')) j++;
        else break;
      }
      const w = src.slice(i, j);
      if (w === 'fun') toks.push({ t: 'λ' });
      else if (w === 'let' || w === 'in') toks.push({ t: w });
      else toks.push({ t: 'id', v: w });
      i = j; continue;
    }
    throw new Error(`Unexpected character “${c}”.`);
  }
  return toks;
}
export const tokText = tk => tk.t === 'id' || tk.t === 'num' ? String(tk.v) : tk.t === 'proj' ? '.' + tk.v : tk.t;

export function parseTerm(src) {
  const toks = tokenize(src); let k = 0;
  const peek = () => toks[k], next = () => toks[k++];
  const expect = t => {
    const x = next();
    if (!x || x.t !== t) throw new Error(`Expected “${t}”${x ? ` before “${tokText(x)}”` : ' at the end'}.`);
    return x;
  };
  const startsAtom = tk => tk && (tk.t === 'id' || tk.t === 'num' || tk.t === '(');
  function term() {
    const tk = peek();
    if (!tk) throw new Error('The term ends too early.');
    if (tk.t === 'λ') {
      next(); const xs = [];
      while (peek() && peek().t === 'id') xs.push(next().v);
      if (!xs.length) throw new Error('Write a variable after λ.');
      const sep = next();
      if (!sep || (sep.t !== '.' && sep.t !== '=>')) throw new Error('Write “.” or “=>” after the binders.');
      let b = term();
      for (let i = xs.length - 1; i >= 0; i--) b = L(xs[i], b);
      return b;
    }
    if (tk.t === 'let') {
      next(); const x = expect('id').v; expect(':='); const v = term();
      const s = next();
      if (!s || (s.t !== ';' && s.t !== 'in')) throw new Error('Write “;” after the let value.');
      return LET(x, v, term());
    }
    let f = postfix();
    while (startsAtom(peek())) f = A(f, postfix());
    if (peek() && (peek().t === 'λ' || peek().t === 'let')) f = A(f, term());
    return f;
  }
  function postfix() { let a = atom(); while (peek() && peek().t === 'proj') a = PR(next().v, a); return a; }
  function atom() {
    const tk = next();
    if (!tk) throw new Error('The term ends too early.');
    if (tk.t === 'id') return V(tk.v);
    if (tk.t === 'num') {
      if (tk.v > 40) throw new Error('Keep numerals at 40 or below; each one is a chain of succ.');
      let t = C('zero'); for (let i = 0; i < tk.v; i++) t = A(C('succ'), t); return t;
    }
    if (tk.t === '(') {
      const items = [term()];
      while (peek() && peek().t === ',') { next(); items.push(term()); }
      expect(')');
      let t = items.pop();
      while (items.length) t = P(items.pop(), t);
      return t;
    }
    throw new Error(`Unexpected “${tokText(tk)}”.`);
  }
  const t = term();
  if (k < toks.length) throw new Error(`Unexpected “${tokText(toks[k])}” after the term.`);
  return t;
}

export function resolve(t, env, bound = new Set()) {
  switch (t.k) {
    case 'var': {
      if (bound.has(t.n)) return t;
      const n = ALIAS[t.n] || t.n;
      return PRIMS.has(n) || env.has(n) ? C(n) : t;
    }
    case 'const': return t;
    case 'lam': { const b = new Set(bound); b.add(t.x); return L(t.x, resolve(t.b, env, b)); }
    case 'let': { const b = new Set(bound); b.add(t.x); return LET(t.x, resolve(t.v, env, bound), resolve(t.b, env, b)); }
    case 'app': return A(resolve(t.f, env, bound), resolve(t.a, env, bound));
    case 'pair': return P(resolve(t.a, env, bound), resolve(t.b, env, bound));
    case 'proj': return PR(t.i, resolve(t.t, env, bound));
  }
}

export function parseProgram(src) {
  const env = new Map(), notes = [], body = [];
  src.split('\n').forEach((line, i) => {
    const d = line.match(/^\s*def\s+(\S+?)\s*:=(.*)$/);
    const c = line.match(/^\s*--\s?(.*)$/);
    if (c) { notes.push(c[1]); return; }
    if (d) {
      const name = d[1];
      if (PRIMS.has(name) || ALIAS[name]) throw new Error(`Line ${i + 1}: ${name} is built in and cannot be redefined.`);
      let t;
      try { t = parseTerm(d[2]); } catch (e) { throw new Error(`Line ${i + 1}: ${e.message}`); }
      env.set(name, resolve(t, env));
      return;
    }
    body.push(line);
  });
  const text = body.join('\n').trim();
  if (!text) throw new Error('Write a term after the definitions.');
  return { env, notes, term: resolve(parseTerm(text), env) };
}

// ===================== Printer =====================
export const esc = s => String(s).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
export const KCLS = k => (k === 'etaS' ? 'eta' : k);
export const GLYPH = { alpha: 'α', beta: 'β', eta: 'η', etaS: 'η', delta: 'δ', zeta: 'ζ', iota: 'ι', proj: 'π' };

export function show(t, o = {}) {
  const H = !!o.html, env = o.env || EMPTY, chips = !!o.chips, db = !!o.db && H;
  let nid = 0;
  const go = (t, p, ctx, prec) => {
    let body, paren = false, attrs = '';
    const kind = chips ? redexKind(t, env) : null;
    const nat = natVal(t);
    if (nat !== null) body = H ? `<span class="rnum">${nat}</span>` : String(nat);
    else switch (t.k) {
      case 'var': {
        let j = ctx.length - 1; while (j >= 0 && ctx[j].x !== t.n) j--;
        if (j < 0) body = H ? `<span class="rv rfree">${esc(t.n)}</span>` : t.n;
        else {
          const lab = db ? String(ctx.length - 1 - j) : t.n;
          body = H ? `<span class="rv${db ? ' rix' : ''}">${esc(lab)}</span>` : lab;
          attrs += ` data-bid="${ctx[j].id}"`;
        }
        break;
      }
      case 'const': body = H ? `<span class="rc${env.has(t.n) ? ' rdef' : ''}">${esc(t.n)}</span>` : t.n; break;
      case 'lam': case 'let': {
        const id = ++nid; attrs += ` data-binder="${id}"`;
        const tag = chips ? 'button' : 'span';
        const fl = t.mark === 'alphaB' ? ' fl fl-alpha' : '';
        const bn = H ? `<${tag} class="rb${fl}"${chips ? ` type="button" title="Rename (α)"` : ''} data-path="${p}" data-anchor="${id}">${esc(t.x)}</${tag}>` : t.x;
        const kw = (s, anchor) => (H ? `<span class="rk"${anchor ? ` data-anchor="${id}"` : ''}>${s}</span>` : s);
        const inner = [...ctx, { x: t.x, id }];
        if (t.k === 'lam') body = kw('λ', db) + (db ? '' : bn) + kw('.') + ' ' + go(t.b, p + 'b', inner, 0);
        else body = kw('let', db) + ' ' + (db ? '' : bn + ' ') + kw(':=') + ' ' + go(t.v, p + 'v', ctx, 0) + kw(';') + ' ' + go(t.b, p + 'b', inner, 0);
        paren = prec >= 1;
        break;
      }
      case 'app': body = go(t.f, p + 'f', ctx, 1) + ' ' + go(t.a, p + 'a', ctx, 2); paren = prec >= 2; break;
      case 'pair': {
        const pu = s => (H ? `<span class="rp">${s}</span>` : s);
        body = pu('(') + go(t.a, p + 'a', ctx, 0) + pu(',') + ' ' + go(t.b, p + 'b', ctx, 0) + pu(')');
        break;
      }
      case 'proj': body = go(t.t, p + 't', ctx, 3) + (H ? `<span class="rj">.${t.i}</span>` : '.' + t.i); break;
    }
    if (!H) return paren ? `(${body})` : body;
    let cls = 'rn';
    if (kind) cls += ` rr k-${KCLS(kind)}`;
    if (t.mark && t.mark !== 'alphaB') cls += ` fl ${t.mark}`;
    if (t.cap) cls += ' cap';
    const chip = kind ? `<button type="button" class="rm k-${KCLS(kind)}" aria-label="Fire ${kind}">${GLYPH[kind]}</button>` : '';
    const span = `<span class="${cls}" data-path="${p}"${kind ? ` data-kind="${kind}"` : ''}${attrs}>${chip}${body}</span>`;
    return paren ? `<span class="rp">(</span>${span}<span class="rp">)</span>` : span;
  };
  return go(t, '', [], 0);
}
export const text = t => show(t);

// ===================== Reduction graph =====================
export function exploreGraph(start, env, withEta, cap) {
  const nodes = new Map(), edges = [];
  const k0 = key(start);
  nodes.set(k0, { key: k0, term: start, depth: 0, parent: null });
  const q = [k0]; let truncated = false;
  while (q.length) {
    const nd = nodes.get(q.shift());
    const rs = redexes(nd.term, env).filter(r => withEta || !isEta(r.kind));
    nd.nf = rs.length === 0;
    if (!rs.length) continue;
    if (nodes.size >= cap) { nd.open = true; truncated = true; continue; }
    for (const r of rs) {
      const t2 = step(nd.term, r, env);
      if (size(t2) > 600) { nd.open = true; truncated = true; continue; }
      const k2 = key(t2);
      if (!nodes.has(k2)) { nodes.set(k2, { key: k2, term: t2, depth: nd.depth + 1, parent: nd.key, pkind: r.kind }); q.push(k2); }
      if (!edges.some(e => e.from === nd.key && e.to === k2 && e.kind === r.kind)) edges.push({ from: nd.key, to: k2, kind: r.kind });
    }
  }
  return { root: k0, nodes, edges, truncated };
}

export function runStrategy(start, env, next, cap) {
  const rows = [{ term: start, kind: null }]; let t = start;
  for (let i = 0; i < cap; i++) {
    const r = next(t, env);
    if (!r) return { rows, status: 'done' };
    t = step(t, r, env); rows.push({ term: t, kind: r.kind });
    if (size(t) > 1500) return { rows, status: 'big' };
  }
  return { rows, status: next(t, env) ? 'cap' : 'done' };
}
