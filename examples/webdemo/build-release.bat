cargo build --manifest-path ../freeverb/Cargo.toml --target wasm32-unknown-unknown --release
wasm-bindgen ../../target/wasm32-unknown-unknown/release/freeverb.wasm --target web --out-dir src/assets/freeverb
cargo build --manifest-path ../gain/Cargo.toml --target wasm32-unknown-unknown --release
wasm-bindgen ../../target/wasm32-unknown-unknown/release/gain.wasm --target web --out-dir src/assets/gain
cargo build --manifest-path ../oscillator/Cargo.toml --target wasm32-unknown-unknown --release
wasm-bindgen ../../target/wasm32-unknown-unknown/release/oscillator.wasm --target web --out-dir src/assets/oscillator
cargo build --manifest-path ../pan/Cargo.toml --target wasm32-unknown-unknown --release
wasm-bindgen ../../target/wasm32-unknown-unknown/release/pan.wasm --target web --out-dir src/assets/pan
cargo build --manifest-path ../surge-synth-saw-demo/Cargo.toml --target wasm32-unknown-unknown --release
wasm-bindgen ../../target/wasm32-unknown-unknown/release/surge_synth_saw_demo.wasm --target web --out-dir src/assets/surge-synth-saw-demo
cargo build --manifest-path ../tremolo/Cargo.toml --target wasm32-unknown-unknown --release
wasm-bindgen ../../target/wasm32-unknown-unknown/release/tremolo.wasm --target web --out-dir src/assets/tremolo
cargo build --manifest-path midi-file-parser/Cargo.toml --target wasm32-unknown-unknown --release
wasm-bindgen ../../target/wasm32-unknown-unknown/release/midi_file_parser.wasm --target web --out-dir src/assets/midi-file-parser