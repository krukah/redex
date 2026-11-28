// The workbench never changes height. Drives every kind of interaction (load every example, fire
// every step with motion on, the automatic α with its panel, rename, de Bruijn, back, reset,
// normalize, a long trace, the source editor and a parse error) on the standalone page and in the
// blog post, samples the workbench's height on every animation frame, and fails on any change.
// With reduced motion it also fails on any animation running on any frame.
// Usage: npm install && trunk build --release && scripts/site.sh && node scripts/steady.mjs [targets…]
//   targets: page, blog, narrow (the standalone page at phone width), still (the standalone page
//   with reduced motion); default all four
import { serve, launch, open } from './browser.mjs';

const TARGETS = { page: ['page', 1100, true], blog: ['blog', 1280, true], narrow: ['page', 390, true], still: ['page', 1100, false] };
const LONG = 'def double := λn. Nat.rec 0 (λk r. succ (succ r)) n\ndouble (double 2)';
const chosen = process.argv.slice(2).length ? process.argv.slice(2) : Object.keys(TARGETS);
const server = await serve();
const browser = await launch();
let failed = false;

for (const name of chosen) {
  const [target, width, motion] = TARGETS[name];
  const { page, errors } = await open(browser, server, target, { motion, width });
  const W = '[data-redex="workbench"]';
  await page.evaluate(() => document.fonts.ready);
  // Every frame, from here on, compare the workbench's height with the first one seen; with
  // reduced motion, nothing may animate at all.
  await page.evaluate(([w, still]) => {
    const el = document.querySelector(w);
    const base = el.getBoundingClientRect().height;
    window.steady = { base, frames: 0, bad: [], label: 'start' };
    const tick = () => {
      const h = el.getBoundingClientRect().height;
      const moving = still ? document.getAnimations().length : 0;
      window.steady.frames++;
      if (h !== base && window.steady.bad.length < 20) window.steady.bad.push([window.steady.label, `${h}px`]);
      if (moving && window.steady.bad.length < 20) window.steady.bad.push([window.steady.label, `${moving} animations`]);
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  }, [W, !motion]);
  const label = l => page.evaluate(x => { window.steady.label = x; }, l);
  const idle = () => page.waitForFunction(w => document.querySelector(w).getAttribute('aria-busy') !== 'true', W, { timeout: 120000 });
  const act = async (l, sel) => {
    await label(l);
    if (await page.$(`${W} ${sel}:not([disabled]):not([hidden])`)) {
      await page.click(`${W} ${sel}`);
      await page.waitForTimeout(50);
      await idle();
    }
  };
  const load = async (l, src) => {
    await act(`${l}: open the source`, '[data-act="source"]');
    await label(`${l}: load`);
    await page.fill(`${W} textarea`, src);
    await page.click(`${W} [data-act="load"]`);
    await idle();
  };
  // A step that throws still reports the heights recorded before it.
  let crash = null;
  try {
    const ids = await page.evaluate(() => [...document.querySelectorAll('[data-redex="rules"] [data-ex]')].map(e => e.dataset.ex));
    for (const id of ids) {
      await label(`${id}: load`);
      await page.click(`[data-redex="rules"] [data-ex="${id}"]`);
      await idle();
      await act(`${id}: step`, '[data-act="step"]');
      await label(`${id}: hover a badge`);
      await page.hover(`${W} .rdx-line .rm`, { timeout: 1000 }).catch(() => {});
      await act(`${id}: normalize`, '[data-act="nf"]');
      await act(`${id}: back`, '[data-act="back"]');
      await act(`${id}: back`, '[data-act="back"]');
      await act(`${id}: de Bruijn on`, '[data-opt="db"]');
      await act(`${id}: de Bruijn off`, '[data-opt="db"]');
      await act(`${id}: rename`, '.rdx-line button.rb');
      await act(`${id}: links off`, '[data-opt="arcs"]');
      await act(`${id}: links on`, '[data-opt="arcs"]');
      await act(`${id}: to WHNF`, '[data-act="whnf"]');
      await act(`${id}: reset`, '[data-act="reset"]');
      await act(`${id}: trace tab`, '[data-act="trace"]');
    }
    await label('α with its panel: load');
    await page.click('[data-redex="rules"] [data-ex="capture"]');
    await idle();
    await act('α with its panel: step', '[data-act="step"]');
    await act('α with its panel: trace tab', '[data-act="trace"]');
    await act('α with its panel: why tab', '[data-act="why"]');
    await load('a long trace', LONG);
    await act('a long trace: trace tab', '[data-act="trace"]');
    await act('a long trace: normalize', '[data-act="nf"]');
    await act('a long trace: click an early row', '.rdx-trace [data-i="2"]');
    await load('a parse error', 'λ. x');
    await load('a fixed source', 'λx. x x');
    await act('a fixed source: trace tab', '[data-act="trace"]');
    await page.waitForTimeout(300);
  } catch (e) { crash = e; }
  const result = await page.evaluate(() => window.steady);
  const ok = !crash && result.bad.length === 0 && errors.length === 0;
  failed ||= !ok;
  console.log(`${name}: workbench ${result.base}px over ${result.frames} frames ${ok ? 'ok' : 'FAIL'}`);
  result.bad.forEach(([l, what]) => console.log(`  ${what} during ${l}`));
  if (crash) console.log(`  stopped during ${result.label}: ${String(crash).split('\n')[0]}`);
  errors.forEach(e => console.log(`  page error: ${e}`));
  await page.close();
}
await browser.close();
server.close();
process.exit(failed ? 1 : 0);
