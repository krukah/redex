// Bundle index.html, its CSS and the ES modules into one HTML file without a document
// skeleton, for publishing as a single-file claude.ai artifact preview.
// Usage: node scripts/bundle.mjs <out.html>
import { readFileSync, writeFileSync } from 'node:fs';

const read = p => readFileSync(new URL(`../${p}`, import.meta.url), 'utf8');
const ORDER = ['engine', 'examples', 'session', 'dom', 'blocks', 'rules', 'workbench', 'graph', 'race', 'redex'];

// Each module becomes a function scope; imports read from, and exports write to, __m.
let js = 'const __m = {};\n';
for (const name of ORDER) {
  const names = [];
  const body = read(`src/${name}.js`)
    .replace(/import\s*\{([^}]*)\}\s*from\s*'\.\/([\w-]+)\.js';/g, (_, ids, mod) => `const {${ids}} = __m['${mod}'];`)
    .replace(/^export (const|let|function|async function) ([\w$]+)/gm, (_, kw, id) => { names.push(id); return `${kw} ${id}`; });
  js += `__m['${name}'] = (() => {\n${body}\nreturn { ${names.join(', ')} };\n})();\n`;
}

const page = read('index.html')
  .replace(/<!doctype html>\s*|<\/?html[^>]*>\s*|<\/?head>\s*|<\/?body>\s*/gi, '')
  .replace(/\s*<meta charset[^>]*>|\s*<meta name="viewport"[^>]*>/g, '')
  .replace('<link rel="stylesheet" href="site.css">', () => `<style>\n${read('site.css')}</style>`)
  .replace('<link rel="stylesheet" href="src/redex.css">', () => `<style>\n${read('src/redex.css')}</style>`)
  .replace(/<script type="module">\s*import \{ mountAll \} from '\.\/src\/redex\.js';/, () => `<script>\n${js}\nconst { mountAll } = __m['redex'];`);

const out = process.argv[2];
if (!out) throw new Error('usage: node scripts/bundle.mjs <out.html>');
writeFileSync(out, page.trimStart());
console.log(`wrote ${out} (${page.length} bytes)`);
