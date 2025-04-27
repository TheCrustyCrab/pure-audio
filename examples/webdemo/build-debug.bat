cargo build --manifest-path ../oscillator/Cargo.toml --target wasm32-unknown-unknown
wasm-bindgen ../../target/wasm32-unknown-unknown/debug/oscillator.wasm --target web --out-dir src/assets/oscillator --keep-debug
cargo build --manifest-path ../surge-synth-saw-demo/Cargo.toml --target wasm32-unknown-unknown
wasm-bindgen ../../target/wasm32-unknown-unknown/debug/surge_synth_saw_demo.wasm --target web --out-dir src/assets/surge-synth-saw-demo --keep-debug