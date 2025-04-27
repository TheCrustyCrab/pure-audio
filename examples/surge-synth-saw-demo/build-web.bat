cargo build --target wasm32-unknown-unknown
wasm-bindgen ../../target/wasm32-unknown-unknown/debug/surge_synth_saw_demo.wasm --target web --out-dir web --keep-debug