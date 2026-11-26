#!/usr/bin/env bash
# Builds the widgets for a host page (the Jekyll site) into dist/site:
#   redex.js  redex_bg.wasm  redex.css
# A host page loads them with
#   <link rel="stylesheet" href="/assets/redex/redex.css">
#   <script type="module">
#     import init, { mountAll } from '/assets/redex/redex.js';
#     await init();
#     mountAll();
#   </script>
# Needs the wasm32-unknown-unknown target, wasm-opt, and a wasm-bindgen CLI of the version in
# Cargo.lock (WASM_BINDGEN=path overrides; Trunk's cached copy is found automatically).
set -euo pipefail
cd "$(dirname "$0")/.."
OUT=dist/site
want=$(awk '/^name = "wasm-bindgen"$/ { getline; gsub(/version = |"/, ""); print }' Cargo.lock)
bindgen=${WASM_BINDGEN:-}
for candidate in "$bindgen" "$(command -v wasm-bindgen || true)" \
  "$HOME/Library/Caches/dev.trunkrs.trunk/wasm-bindgen-$want/wasm-bindgen" \
  "$HOME/.cache/trunk/wasm-bindgen-$want/wasm-bindgen"; do
  if [ -n "$candidate" ] && [ -x "$candidate" ] && "$candidate" --version | grep -q "$want"; then bindgen=$candidate; break; fi
  bindgen=
done
[ -n "$bindgen" ] || { echo "need wasm-bindgen $want (cargo install wasm-bindgen-cli --version $want, or set WASM_BINDGEN)" >&2; exit 1; }
cargo build -p redex --release --target wasm32-unknown-unknown
rm -rf "$OUT" && mkdir -p "$OUT"
"$bindgen" --target web --no-typescript --out-dir "$OUT" --out-name redex target/wasm32-unknown-unknown/release/redex.wasm
wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals \
  "$OUT/redex_bg.wasm" -o "$OUT/redex_bg.wasm"
cp crates/widgets/redex.css "$OUT/redex.css"
echo "wrote $OUT: $(gzip -9c "$OUT/redex_bg.wasm" | wc -c | tr -d ' ') bytes of wasm, gzipped"
