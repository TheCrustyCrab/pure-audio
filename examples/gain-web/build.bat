cargo build --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/debug/gain_web.wasm --target web --out-dir web --out-name gain_loader --keep-debug