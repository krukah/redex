// Bundles the Rust page into one HTML file without a document skeleton, for publishing as a
// single-file claude.ai artifact preview: the page, both stylesheets, the wasm-bindgen glue, and
// the wasm itself as base64. Run scripts/site.sh first.
// Usage: node scripts/bundle-wasm.mjs <out.html>
import { readFileSync, writeFileSync } from 'node:fs';

const read = (p, enc = 'utf8') => readFileSync(new URL(`../${p}`, import.meta.url), enc);
const out = process.argv[2];
if (!out) throw new Error('usage: node scripts/bundle-wasm.mjs <out.html>');

const wasm = read('dist/site/redex_bg.wasm', null).toString('base64');
const glue = read('dist/site/redex.js');
const script = `<script type="module">
${glue}
const bytes = Uint8Array.from(atob(${JSON.stringify(wasm)}), c => c.charCodeAt(0));
await __wbg_init({ module_or_path: bytes });
mountAll();
</script>`;

const page = read('bin/page/index.html')
  .replace(/<!doctype html>\s*|<\/?html[^>]*>\s*|<\/?head>\s*|<\/?body>\s*/gi, '')
  .replace(/\s*<meta charset[^>]*>|\s*<meta name="viewport"[^>]*>/g, '')
  .replace('<link data-trunk rel="css" href="site.css">', () => `<style>\n${read('bin/page/site.css')}</style>`)
  .replace('<link data-trunk rel="css" href="../../crates/widgets/redex.css">', () => `<style>\n${read('crates/widgets/redex.css')}</style>`)
  .replace(/\s*<link data-trunk rel="rust"[^>]*>/, '')
  .trimEnd() + `\n${script}\n`;

writeFileSync(out, page.trimStart());
console.log(`wrote ${out} (${page.length} bytes)`);
