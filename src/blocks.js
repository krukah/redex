// Nested-block view of a term. Each node is one element carrying its path and uid:
//   λ / let   a box with a binder tab, tinted by the binder's hue
//   apply     one box per application spine: head first, then the arguments
//             (inner applications of the spine are borderless groups)
//   pair      a box holding two components
//   var       a tag in its binder's hue; free variables are dashed and gray
//   const     a bold word; δ-unfoldable ones are underlined
// A redex gets a rule badge on its top-left corner.
import { EMPTY, GLYPH, KCLS, esc, natVal, redexKind } from './engine.js';

const HUES = 6;
const hueOf = name => { let h = 7; for (const c of name) h = (h * 31 + c.codePointAt(0)) >>> 0; return h % HUES; };

export function renderBlocks(term, o = {}) {
  const env = o.env || EMPTY, live = !!o.live, db = !!o.db;
  let nid = 0;

  // d: depth of the nearest enclosing box, for alternating backgrounds.
  // spine: this node is the function part of an application.
  const go = (t, p, ctx, d, spine) => {
    const kind = live ? redexKind(t, env) : null;
    const cls = ['b'];
    let attrs = ` data-path="${p}" data-uid="${t.uid}"${t.from ? ` data-from="${t.from}"` : ''}`;
    let inner, boxed = false;
    const nat = natVal(t);
    if (nat !== null) { cls.push('b-lit'); inner = String(nat); }
    else switch (t.k) {
      case 'var': {
        let j = ctx.length - 1; while (j >= 0 && ctx[j].x !== t.n) j--;
        if (j < 0) { cls.push('b-var', 'b-free'); inner = esc(t.n); break; }
        cls.push('b-var', `h${ctx[j].hue}`);
        if (db) cls.push('b-ix');
        attrs += ` data-bid="${ctx[j].id}"`;
        inner = db ? String(ctx.length - 1 - j) : esc(t.n);
        break;
      }
      case 'const':
        cls.push('b-const'); if (env.has(t.n)) cls.push('b-def');
        inner = esc(t.n);
        break;
      case 'lam': case 'let': {
        const id = ++nid, hue = hueOf(t.x);
        boxed = true; cls.push(t.k === 'lam' ? 'b-lam' : 'b-let', `h${hue}`);
        attrs += ` data-binder="${id}"`;
        const fl = t.mark === 'alphaB' ? ' fl fl-alpha' : '';
        const name = db ? '' : live
          ? `<button type="button" class="rb${fl}" data-path="${p}" data-anchor="${id}" title="Rename (α)">${esc(t.x)}</button>`
          : `<span class="rb" data-anchor="${id}">${esc(t.x)}</span>`;
        const kw = s => `<span class="rk"${db ? ` data-anchor="${id}"` : ''}>${s}</span>`;
        const inside = [...ctx, { x: t.x, id, hue }];
        inner = t.k === 'lam'
          ? `<span class="bt">${kw('λ')}${name}</span>${go(t.b, p + 'b', inside, d + 1)}`
          : `<span class="bt">${kw('let')}${name}<span class="rk">:=</span>${go(t.v, p + 'v', ctx, d + 1)}</span>${go(t.b, p + 'b', inside, d + 1)}`;
        break;
      }
      case 'app': {
        const k = spine ? d : d + 1;
        boxed = !spine; cls.push(spine ? 'b-grp' : 'b-app');
        inner = go(t.f, p + 'f', ctx, k, true) + go(t.a, p + 'a', ctx, k, false);
        break;
      }
      case 'pair':
        boxed = true; cls.push('b-pair');
        inner = go(t.a, p + 'a', ctx, d + 1) + '<span class="rp">,</span>' + go(t.b, p + 'b', ctx, d + 1);
        break;
      case 'proj':
        cls.push('b-proj');
        inner = go(t.t, p + 't', ctx, d) + `<span class="bj">.${t.i}</span>`;
        break;
    }
    if (boxed) cls.push(d % 2 ? 'odd' : 'even');
    if (kind) { cls.push('rr', `k-${KCLS(kind)}`); attrs += ` data-kind="${kind}"`; }
    if (t.mark && t.mark !== 'alphaB') cls.push('fl', t.mark);
    if (t.cap) cls.push('cap');
    const badge = kind ? `<button type="button" class="rm k-${KCLS(kind)}" aria-label="Fire ${kind}">${GLYPH[kind]}</button>` : '';
    return `<span class="${cls.join(' ')}"${attrs}>${badge}${inner}</span>`;
  };
  return go(term, '', [], 0, false);
}
