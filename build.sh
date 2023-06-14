cd ./wasm_modules/game_of_life && \
cargo build --release --no-default-features --target wasm32-unknown-unknown && \
wasm-bindgen --target web --out-name game_of_life --out-dir ../../wasm --no-typescript ./target/wasm32-unknown-unknown/release/game_of_life.wasm
