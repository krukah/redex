// A browser check of the embedding: two independent groups of widgets on one page, each mounted with
// its own session. Opening a term from the second group's graph must bring the second group's
// workbench into view, not the first's.
// Usage: npm install && scripts/site.sh && node scripts/reveal.mjs [chrome-path]
import { createServer } from 'node:http';
import { readFileSync } from 'node:fs';
import { extname } from 'node:path';
import { chromium } from 'playwright-core';

const ROOT = new URL('../', import.meta.url);
const CHROME = process.argv[2] || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const TYPES = { '.js': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm' };
const group = n => `<section id="g${n}"><div data-redex="workbench" data-example="diamond"></div><div data-redex="graph"></div></section><div style="height:3000px"></div>`;
const PAGE = `<!doctype html><meta charset="utf-8"><link rel="stylesheet" href="/dist/site/redex.css">
<body>${group(1)}${group(2)}<script type="module">import init, { mountAll } from '/dist/site/redex.js';
await init();
for (const g of document.querySelectorAll('section')) mountAll(g);
window.done = true;</script></body>`;
const server = createServer((req, res) => {
  const url = req.url.split('?')[0];
  try {
    const body = url === '/' ? PAGE : readFileSync(new URL(`.${url}`, ROOT));
    res.writeHead(200, { 'content-type': url === '/' ? 'text/html' : TYPES[extname(url)] || 'text/plain' });
    res.end(body);
  } catch { res.writeHead(404); res.end(); }
}).listen(0);

const browser = await chromium.launch({ executablePath: CHROME });
const page = await browser.newPage({ viewport: { width: 1100, height: 800 } });
await page.emulateMedia({ reducedMotion: 'reduce' });
await page.goto(`http://localhost:${server.address().port}/`);
await page.waitForFunction(() => window.done);
await page.click('#g2 .rdx-gn[data-id="3"] .hit');
await page.click('#g2 [data-act="open"]');
const top = await page.evaluate(() => Math.round(document.querySelector('#g2 [data-redex="workbench"]').getBoundingClientRect().top));
const ok = Math.abs(top) <= 1;
console.log(`second workbench top ${top}px ${ok ? 'ok' : 'FAIL'}`);
await browser.close();
server.close();
process.exit(ok ? 0 : 1);
