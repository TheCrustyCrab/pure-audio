cargo build --target wasm32-unknown-unknown
wasm-bindgen ../../target/wasm32-unknown-unknown/debug/pan.wasm --target web --out-dir web --keep-debug