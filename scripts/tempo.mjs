// Times every firing of a few examples, Step by Step and under Normalize, from the marks the
// workbench leaves on the performance timeline (motion::Cue), and prints one row per step.
// Usage: npm install && trunk build --release && node scripts/tempo.mjs [page|blog] [ids…]
import { serve, launch, open } from './browser.mjs';

const target = process.argv[2] || 'page';
const ids = process.argv.slice(3).length ? process.argv.slice(3) : ['mix', 'beta', 'capture', 'church'];
const server = await serve();
const browser = await launch();
const { page, errors } = await open(browser, server, target, { motion: true, width: 1280 });

const marks = () => page.evaluate(() => performance.getEntriesByType('mark').filter(m => m.name.startsWith('redex:')).map(m => [m.name.slice(6), m.startTime]));
const kinds = () => page.evaluate(() => [...document.querySelectorAll('[data-redex="workbench"] .rdx-trace li')].slice(1).map(li => li.querySelector('.rm').textContent));

// Splits the marks into steps (an automatic α is a step of its own) and measures each phase.
const table = (list, kinds) => {
  const rows = [];
  let row = null;
  for (const [name, t] of list) {
    if (name === 'fire' || name === 'alpha') { row = { t0: t, at: {} }; rows.push(row); continue; }
    row.at[name] = t;
  }
  return rows.map((r, i) => {
    const a = r.at, next = rows[i + 1]?.t0;
    return {
      step: i + 1,
      rule: kinds[i],
      hold: Math.round(a.lift - a.hold),
      push: Math.round(a.move - a.lift),
      move: Math.round(a.moved - a.move),
      gap: next === undefined ? null : Math.round(next - a.moved),
      total: Math.round(a.moved - r.t0),
    };
  });
};

const print = (title, rows) => {
  console.log(`\n${title}`);
  console.log('step rule  hold  push  move   gap  total');
  rows.forEach(r => console.log(`${String(r.step).padStart(4)} ${r.rule.padEnd(4)} ${String(r.hold).padStart(5)} ${String(r.push).padStart(5)} ${String(r.move).padStart(5)} ${String(r.gap ?? '-').padStart(5)} ${String(r.total).padStart(6)}`));
  const totals = rows.map(r => r.total);
  console.log(`total: min ${Math.min(...totals)} max ${Math.max(...totals)} spread ${Math.max(...totals) - Math.min(...totals)} ms`);
};

for (const id of ids) {
  await page.click(`:is([data-redex="rules"],[data-redex="examples"]) [data-ex="${id}"]`);
  await page.evaluate(() => performance.clearMarks());
  for (;;) {
    const ready = await page.evaluate(() => !document.querySelector('[data-redex="workbench"] [data-act="step"]').disabled);
    if (!ready) break;
    const before = (await marks()).filter(([n]) => n === 'moved').length;
    await page.click('[data-redex="workbench"] [data-act="step"]');
    await page.waitForFunction(n => performance.getEntriesByType('mark').filter(m => m.name === 'redex:moved').length > n, before, { timeout: 30000 });
    await page.waitForFunction(() => document.querySelector('[data-redex="workbench"]').getAttribute('aria-busy') === 'false');
  }
  print(`${id}: Step by Step`, table(await marks(), await kinds()));
  await page.click('[data-redex="workbench"] [data-act="reset"]');
  await page.evaluate(() => performance.clearMarks());
  await page.click('[data-redex="workbench"] [data-act="nf"]');
  await page.waitForFunction(() => document.querySelector('[data-redex="workbench"]').getAttribute('aria-busy') === 'true');
  await page.waitForFunction(() => document.querySelector('[data-redex="workbench"]').getAttribute('aria-busy') === 'false', null, { timeout: 180000 });
  print(`${id}: Normalize`, table(await marks(), await kinds()));
}
if (errors.length) console.log('page errors:', errors);
await browser.close();
server.close();
