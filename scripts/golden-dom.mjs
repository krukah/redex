// DOM fixtures for the Rust port: what the JS widgets draw, read from the real page in headless
// Chrome with reduced motion. The layout and widget crates must reproduce these exactly.
// Usage: npm i --no-save playwright-core && node scripts/golden-dom.mjs [chrome-path]
//   (writes fixtures/dom/*.json)
import { createServer } from 'node:http';
import { mkdirSync, readFileSync, rmSync, writeFileSync, readdirSync } from 'node:fs';
import { extname } from 'node:path';
import { chromium } from 'playwright-core';
import { EXAMPLES, RACES } from '../src/examples.js';

const ROOT = new URL('../', import.meta.url);
const OUT = new URL('fixtures/dom/', ROOT);
const CHROME = process.argv[2] || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css' };

const server = createServer((req, res) => {
  try {
    const path = new URL(`.${decodeURIComponent(req.url.split('?')[0])}`, ROOT);
    const body = readFileSync(path);
    res.writeHead(200, { 'content-type': TYPES[extname(path.pathname)] || 'application/octet-stream' });
    res.end(body);
  } catch { res.writeHead(404); res.end(); }
}).listen(0);
const port = server.address().port;

const browser = await chromium.launch({ executablePath: CHROME });
const page = await browser.newPage({ viewport: { width: 1200, height: 900 } });
await page.emulateMedia({ reducedMotion: 'reduce' });
await page.goto(`http://localhost:${port}/index.html`);
await page.waitForSelector('.rdx-gsvg .rdx-gn');

// Everything the graph widget drew, by query: geometry as the attribute strings JS wrote.
const graph = () => page.evaluate(() => {
  const root = document.querySelector('[data-redex="graph"]'), svg = root.querySelector('.rdx-gsvg');
  return {
    eta: root.querySelector('[data-opt="eta"]').checked,
    width: svg.getAttribute('width'), height: svg.getAttribute('height'), viewBox: svg.getAttribute('viewBox'),
    svg: svg.innerHTML,
    stats: root.querySelector('.rdx-gstats').textContent,
    verdict: root.querySelector('.rdx-verdict').textContent,
    routes: [...root.querySelectorAll('[data-route]')].map(b => b.dataset.route),
  };
});
const routes = async () => {
  const out = {};
  for (const id of ['normal', 'applicative', 'whnf', 'none']) {
    await page.click(`[data-redex="graph"] [data-route="${id}"]`);
    out[id] = await page.evaluate(() => {
      const root = document.querySelector('[data-redex="graph"]');
      return { band: root.querySelector('.routes').innerHTML, note: root.querySelector('.rdx-route-note').textContent, pills: root.querySelector('[data-routes]').innerHTML };
    });
  }
  return out;
};
const both = async () => {
  const first = { ...(await graph()), routes: await routes() };
  await page.click('[data-redex="graph"] [data-opt="eta"]');
  const second = { ...(await graph()), routes: await routes() };
  await page.click('[data-redex="graph"] [data-opt="eta"]');
  return [first, second];
};

rmSync(OUT, { recursive: true, force: true });
mkdirSync(OUT, { recursive: true });
const write = (name, data) => writeFileSync(new URL(`${name}.json`, OUT), JSON.stringify(data) + '\n');

write('deck', { html: await page.evaluate(() => document.querySelector('[data-redex="rules"]').innerHTML) });

for (const e of EXAMPLES) {
  await page.click(`[data-redex="rules"] [data-ex="${e.id}"]`);
  write(`graph-example-${e.id}`, { id: e.id, src: e.src, graphs: await both() });
}

// Random terms go in through the workbench's source box, with η off.
const sources = readdirSync(new URL('fixtures/engine/', ROOT)).filter(f => f.startsWith('random-'))
  .map(f => JSON.parse(readFileSync(new URL(`fixtures/engine/${f}`, ROOT))))
  .filter(c => !c.error);
await page.click('[data-redex="rules"] [data-ex="beta"]');
for (const c of sources) {
  await page.evaluate(src => {
    const root = document.querySelector('[data-redex="workbench"]');
    root.querySelector('.rdx-src textarea').value = src;
    root.querySelector('[data-act="load"]').click();
  }, c.src);
  write(`graph-${c.id}`, { id: c.id, src: c.src, graphs: await both() });
}

const races = {};
await page.click('[data-redex="rules"] [data-ex="beta"]');
for (const r of RACES) {
  await page.click(`[data-redex="race"] [data-race="${r.id}"]`);
  races[r.id] = await page.evaluate(() => document.querySelector('[data-redex="race"]').innerHTML);
}
write('race', { bench: 'beta', races });

await browser.close();
server.close();
console.log(`wrote DOM fixtures to ${OUT.pathname}`);
