cargo build --manifest-path ../freeverb/Cargo.toml --target wasm32-unknown-unknown
wasm-bindgen ../../target/wasm32-unknown-unknown/debug/freeverb.wasm --target web --out-dir src/assets/freeverb --keep-debug
cargo build --manifest-path ../gain/Cargo.toml --target wasm32-unknown-unknown
wasm-bindgen ../../target/wasm32-unknown-unknown/debug/gain.wasm --target web --out-dir src/assets/gain --keep-debug
cargo build --manifest-path ../oscillator/Cargo.toml --target wasm32-unknown-unknown
wasm-bindgen ../../target/wasm32-unknown-unknown/debug/oscillator.wasm --target web --out-dir src/assets/oscillator --keep-debug
cargo build --manifest-path ../pan/Cargo.toml --target wasm32-unknown-unknown
wasm-bindgen ../../target/wasm32-unknown-unknown/debug/pan.wasm --target web --out-dir src/assets/pan --keep-debug
cargo build --manifest-path ../surge-synth-saw-demo/Cargo.toml --target wasm32-unknown-unknown
wasm-bindgen ../../target/wasm32-unknown-unknown/debug/surge_synth_saw_demo.wasm --target web --out-dir src/assets/surge-synth-saw-demo --keep-debug
cargo build --manifest-path ../tremolo/Cargo.toml --target wasm32-unknown-unknown
wasm-bindgen ../../target/wasm32-unknown-unknown/debug/tremolo.wasm --target web --out-dir src/assets/tremolo --keep-debug
cargo build --manifest-path midi-file-parser/Cargo.toml --target wasm32-unknown-unknown
wasm-bindgen ../../target/wasm32-unknown-unknown/debug/midi_file_parser.wasm --target web --out-dir src/assets/midi-file-parser --keep-debug