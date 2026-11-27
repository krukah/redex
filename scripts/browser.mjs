// Shared plumbing for the browser checks: headless Chrome, and the two pages the widgets live on.
//   page  the standalone page from `trunk build --release` (dist/), served here
//   blog  the post on the local Jekyll server, with /assets/redex/ answered from dist/site
//         (`scripts/site.sh`), so the check runs this build inside the blog's own CSS
import { createServer } from 'node:http';
import { readFileSync } from 'node:fs';
import { extname } from 'node:path';
import { chromium } from 'playwright-core';

const ROOT = new URL('../', import.meta.url);
const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm' };
export const CHROME = process.env.CHROME || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
export const BLOG = process.env.BLOG || 'http://localhost:4011/firing-redexes-by-hand/';

// A static server over the repository, falling back to dist/ for the page's own assets; `pages`
// answers extra paths with fixed HTML.
export const serve = (pages = {}) => new Promise(ok => {
  const server = createServer((req, res) => {
    const url = req.url.split('?')[0];
    try {
      const path = url.endsWith('/') ? `${url}index.html` : url;
      const body = pages[url] ?? (() => {
        try { return readFileSync(new URL(`.${path}`, ROOT)); } catch { return readFileSync(new URL(`dist${path}`, ROOT)); }
      })();
      res.writeHead(200, { 'content-type': pages[url] ? 'text/html' : TYPES[extname(url)] || 'text/html' });
      res.end(body);
    } catch { res.writeHead(404); res.end(); }
  }).listen(0, () => ok(server));
});

export const launch = () => chromium.launch({ executablePath: CHROME });

// Opens one of the two pages in a fresh tab and waits for the workbench to draw.
export async function open(browser, server, target, { motion = false, width = 1100 } = {}) {
  const page = await browser.newPage({ viewport: { width, height: 900 } });
  await page.emulateMedia({ reducedMotion: motion ? 'no-preference' : 'reduce' });
  const errors = [];
  page.on('pageerror', e => errors.push(String(e)));
  if (target === 'blog') {
    await page.route('**/assets/redex/**', route => {
      const file = new URL(`dist/site/${route.request().url().split('/assets/redex/')[1].split('?')[0]}`, ROOT);
      route.fulfill({ body: readFileSync(file), contentType: TYPES[extname(file.pathname)] });
    });
    await page.goto(BLOG);
  } else {
    await page.goto(`http://localhost:${server.address().port}/dist/`);
  }
  await page.waitForSelector('[data-redex="workbench"] .rdx-line .b');
  return { page, errors };
}
