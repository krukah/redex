// Side-by-side parity: drives the JS page (repo root) and the Rust page (dist/, from `trunk build`)
// through the same states in headless Chrome, screenshots each, and diffs the widgets' DOM.
// Usage: npm i --no-save playwright-core && trunk build --release && node scripts/parity.mjs <out-dir> [chrome-path]
//   (states from scripts/parity-states.json, or PARITY_STATES as JSON)
import { createServer } from 'node:http';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { extname } from 'node:path';
import { chromium } from 'playwright-core';

const OUT = process.argv[2] || 'parity';
const CHROME = process.argv[3] || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm' };
const serve = root => new Promise(ok => {
  const s = createServer((req, res) => {
    try {
      const path = new URL(`.${decodeURIComponent(req.url.split('?')[0])}`, root);
      const body = readFileSync(path.pathname.endsWith('/') ? new URL('index.html', path) : path);
      res.writeHead(200, { 'content-type': TYPES[extname(path.pathname)] || 'text/html' });
      res.end(body);
    } catch { res.writeHead(404); res.end(); }
  }).listen(0, () => ok(s));
});
const js = await serve(new URL('../', import.meta.url));
const rs = await serve(new URL('../dist/', import.meta.url));
mkdirSync(OUT, { recursive: true });

// Ids differ between the pages (each allocates its own); number them by first appearance.
const rebase = html => {
  const seen = new Map();
  return html.replace(/data-(uid|from)="(\d+)"/g, (_, a, n) => {
    if (!seen.has(n)) seen.set(n, seen.size);
    return `data-${a}="${seen.get(n)}"`;
  });
};
const strip = html => rebase(html).replace(/>\s+</g, '><').trim();

const browser = await chromium.launch({ executablePath: CHROME });
// Each state starts from a fresh page: clicks, hovers, keys, script and waits, in order.
const STATES = JSON.parse(process.env.PARITY_STATES || readFileSync(new URL('parity-states.json', import.meta.url), 'utf8'));
const report = [];
// The rows of the two screenshots that differ, as [first, last], or null when they are identical.
const pixelDiff = async stem => {
  const page = await browser.newPage();
  const url = side => `data:image/png;base64,${readFileSync(`${stem}-${side}.png`).toString('base64')}`;
  const out = await page.evaluate(async ([a, b]) => {
    const load = src => new Promise(ok => { const i = new Image(); i.onload = () => ok(i); i.src = src; });
    const [x, y] = await Promise.all([load(a), load(b)]);
    if (x.width !== y.width || x.height !== y.height) return `size ${x.width}x${x.height} vs ${y.width}x${y.height}`;
    const data = img => { const c = document.createElement('canvas'); c.width = img.width; c.height = img.height; const g = c.getContext('2d'); g.drawImage(img, 0, 0); return g.getImageData(0, 0, c.width, c.height).data; };
    const [p, q] = [data(x), data(y)];
    let first = -1, last = -1;
    for (let i = 0; i < p.length; i += 4) if (p[i] !== q[i] || p[i + 1] !== q[i + 1] || p[i + 2] !== q[i + 2]) { const row = Math.floor(i / 4 / x.width); if (first < 0) first = row; last = row; }
    return first < 0 ? null : [first, last];
  }, [url('js'), url('rs')]);
  await page.close();
  return out;
};
for (const reduced of [true, false]) {
  for (const st of STATES) {
    const shots = {};
    for (const [side, server] of [['js', js], ['rs', rs]]) {
      const page = await browser.newPage({ viewport: { width: 1100, height: 900 } });
      await page.emulateMedia({ reducedMotion: reduced ? 'reduce' : 'no-preference' });
      const errors = [];
      page.on('pageerror', e => errors.push(String(e)));
      page.on('console', m => m.type() === 'error' && errors.push(m.text()));
      await page.goto(`http://localhost:${server.address().port}/index.html`);
      await page.waitForSelector('[data-redex="workbench"] .rdx-line .b');
      for (const s of st.steps) {
        if (s.click) await page.click(s.click);
        if (s.hover) await page.hover(s.hover);
        if (s.eval) await page.evaluate(s.eval);
        if (s.key) await page.keyboard.press(s.key);
        if (s.wait) await page.waitForTimeout(s.wait);
      }
      await page.screenshot({ path: `${OUT}/${st.name}${reduced ? '' : '-motion'}-${side}.png`, fullPage: true });
      shots[side] = {
        errors,
        widgets: await page.evaluate(() => {
          // A canonical serialization: attributes sorted, whitespace-only text dropped.
          const ser = n => n.nodeType === 3 ? (n.data.trim() ? n.data.replace(/&/g, '&amp;').replace(/</g, '&lt;') : '')
            : n.nodeType !== 1 ? ''
            : `<${n.localName}${[...n.attributes].map(a => [a.name, a.value]).sort().map(([k, v]) => ` ${k}="${v}"`).join('')}>${[...n.childNodes].map(ser).join('')}</${n.localName}>`;
          return { ...Object.fromEntries([...document.querySelectorAll('[data-redex]')].map(el => [el.dataset.redex, [...el.childNodes].map(ser).join('')])),
            ghosts: [...document.querySelectorAll('.rdx-ghost')].map(g => { const c = g.cloneNode(true); c.removeAttribute('style'); return `${g.style.cssText.replace(/opacity: [^;]*;?/, '')}|${ser(c)}`; }).sort().join('\n') };
        }),
      };
      await page.close();
    }
    const pixels = await pixelDiff(`${OUT}/${st.name}${reduced ? '' : '-motion'}`);
    const diffs = Object.keys(shots.js.widgets).filter(k => strip(shots.js.widgets[k]) !== strip(shots.rs.widgets[k] ?? ''));
    report.push({ state: st.name, reduced, diffs, pixels, errors: shots.rs.errors });
    writeFileSync(`${OUT}/${st.name}${reduced ? '' : '-motion'}.json`, JSON.stringify(Object.fromEntries(diffs.map(k => [k, { js: strip(shots.js.widgets[k]), rs: strip(shots.rs.widgets[k] ?? '') }])), null, 1));
  }
}
await browser.close();
js.close(); rs.close();
console.log(JSON.stringify(report, null, 1));
