cargo build --target wasm32-unknown-unknown
wasm-bindgen ../../target/wasm32-unknown-unknown/debug/freeverb.wasm --target web --out-dir web --out-name freeverb --keep-debug