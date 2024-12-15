cargo build --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/debug/oscillator_web.wasm --target web --out-dir web --out-name oscillator_loader --keep-debug