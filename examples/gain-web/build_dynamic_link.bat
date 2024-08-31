cargo build --target wasm32-unknown-unknown --features build_processor
wasm-bindgen target/wasm32-unknown-unknown/debug/gain_web.wasm --target web --out-dir web_dynamic --out-name gain --keep-debug --no-typescript
cargo build --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/debug/gain_web.wasm --target web --out-dir web_dynamic --out-name gain_loader --keep-debug