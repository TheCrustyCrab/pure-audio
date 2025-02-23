cargo build --target wasm32-unknown-unknown
wasm-bindgen ../../target/wasm32-unknown-unknown/debug/gain.wasm --target web --out-dir web --out-name gain --keep-debug