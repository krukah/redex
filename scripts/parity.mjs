// Side-by-side parity: drives the JS page (repo root) and the Rust page (dist/, from `trunk build`)
// through the same states in headless Chrome, screenshots each, and diffs the widgets' DOM.
// Usage: npm i --no-save playwright-core && node scripts/parity.mjs <out-dir> [chrome-path]
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

// Ids differ between the pages (each allocates its own); compare them relative to the first one.
const rebase = html => {
  const ids = [...html.matchAll(/data-(?:uid|from)="(\d+)"/g)].map(m => +m[1]);
  const base = Math.min(...ids, Infinity);
  return html.replace(/data-(uid|from)="(\d+)"/g, (_, a, n) => `data-${a}="${n - base}"`);
};
const strip = html => rebase(html).replace(/>\s+</g, '><').trim();

const browser = await chromium.launch({ executablePath: CHROME });
const STATES = JSON.parse(process.env.PARITY_STATES || 'null') || [
  { name: 'load', steps: [] },
];
const report = [];
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
        if (s.wait) await page.waitForTimeout(s.wait);
      }
      await page.screenshot({ path: `${OUT}/${st.name}${reduced ? '' : '-motion'}-${side}.png`, fullPage: true });
      shots[side] = {
        errors,
        widgets: await page.evaluate(() => Object.fromEntries([...document.querySelectorAll('[data-redex]')].map(el => [el.dataset.redex, el.innerHTML]))),
      };
      await page.close();
    }
    const diffs = Object.keys(shots.js.widgets).filter(k => strip(shots.js.widgets[k]) !== strip(shots.rs.widgets[k] ?? ''));
    report.push({ state: st.name, reduced, diffs, errors: shots.rs.errors });
    writeFileSync(`${OUT}/${st.name}${reduced ? '' : '-motion'}.json`, JSON.stringify(Object.fromEntries(diffs.map(k => [k, { js: strip(shots.js.widgets[k]), rs: strip(shots.rs.widgets[k] ?? '') }])), null, 1));
  }
}
await browser.close();
js.close(); rs.close();
console.log(JSON.stringify(report, null, 1));
