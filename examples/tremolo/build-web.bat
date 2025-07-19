cargo build --target wasm32-unknown-unknown
wasm-bindgen ../../target/wasm32-unknown-unknown/debug/tremolo.wasm --target web --out-dir web --out-name tremolo --keep-debug