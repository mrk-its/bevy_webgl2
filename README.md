
# Running examples

## Prerequisites

```
cargo install wasm-bindgen-cli
cargo install basic-http-server
```

to build `sprite` example:
```
cargo build --release --example sprite --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --target web --out-name wasm --out-dir . ./target/wasm32-unknown-unknown/release/examples/sprite.wasm
basic-http-server
```

and point your browser to `http://127.0.0.1:4000`
