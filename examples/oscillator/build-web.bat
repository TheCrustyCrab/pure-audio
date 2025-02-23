cargo build --target wasm32-unknown-unknown
wasm-bindgen ../../target/wasm32-unknown-unknown/debug/oscillator.wasm --target web --out-dir web --keep-debug