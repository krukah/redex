// Shared state for every widget on a page: the loaded program and the reduction history.
// Widgets render from it and call its methods; it notifies them with 'load', 'step' or 'reveal'.
import { parseProgram, CONSTS, EMPTY } from './engine.js';
import { EXAMPLES } from './examples.js';

export function createSession() {
  const listeners = new Set();
  return {
    env: EMPTY,       // Map of δ-unfoldable definitions
    notes: [],        // comment lines from the source
    src: '',
    ex: null,         // id of the loaded example, if any
    eta: false,       // whether the example wants η edges in the graph
    hist: [],         // [{ term, kind, note }]; hist[0] is the start term
    idx: 0,           // position in hist
    capture: null,    // the wrong, capturing result of the last renamed β
    busy: false,      // a widget is animating or running a strategy

    get term() { return this.hist[this.idx].term; },
    get start() { return this.hist[0].term; },

    on(fn) { listeners.add(fn); return () => listeners.delete(fn); },
    emit(type) { listeners.forEach(fn => fn(type)); },

    // Throws with a readable message when the source does not parse.
    load(src, { id = null, eta = false } = {}) {
      const prog = parseProgram(src);
      prog.env.forEach((_, name) => CONSTS.add(name));
      Object.assign(this, {
        env: prog.env, notes: prog.notes, src, ex: id, eta,
        hist: [{ term: prog.term, kind: null }], idx: 0, capture: null,
      });
      this.emit('load');
    },
    loadExample(id) {
      const ex = EXAMPLES.find(e => e.id === id);
      if (ex) this.load(ex.src, { id, eta: !!ex.eta });
    },
    push(entry) {
      this.hist = this.hist.slice(0, this.idx + 1);
      this.hist.push(entry);
      this.idx = this.hist.length - 1;
      this.emit('step');
    },
    seek(i) { this.idx = i; this.emit('step'); },
    reset() { this.hist = [this.hist[0]]; this.idx = 0; this.capture = null; this.emit('step'); },
    replaceHistory(hist) { this.hist = hist; this.idx = hist.length - 1; this.capture = null; this.emit('step'); },
    // Asks the workbench to bring itself into view, e.g. after the graph opened a term in it.
    reveal() { this.emit('reveal'); },
  };
}
