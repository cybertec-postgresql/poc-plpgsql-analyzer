# Development

## Getting started

First of, you need a working Rust toolchain using [rustup](https://rustup.rs/).

> **Note**
> This is due to the `wasm32-unknown-unknown` target being not a default one [yet]
> and needs to be downloaded separatly. Rustup will take care of that automatically.

To generate the WASM code and it's corresponding JavaScript/TypeScript bindings,
the [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/) tool it needed.

> **Note**
> If you already have a working Rust toolchain installed, you can also just type
> `cargo install wasm-pack` to install it from source directly.)

Then, simply type 
```sh
wasm-pack build --target nodejs
```
in the project directory to build.

This will create a full npm package in the `pkg/` directory.

## Unit tests

To run all the unit tests, simply run
```sh
cargo test
```

To run the unit tests marked with `#[ignore]`
```sh
cargo test -- --ignored
```

To run the coverage test
```sh
cargo test -F coverage-tests
```

To run the TypeScript test suite
```sh
npm --prefix tests/typescript clean-install
npm --prefix tests/typescript test
```

## Rustdoc

To generate the (public) rustdoc for this library:
```sh
cargo doc
```

And to generate the full rustdoc for all items, including private ones:
```sh
cargo doc --document-private-items
```

#### Optional tools

It is desired to also install the `wasm-opt` tool from the
[`binaryen`](https://github.com/WebAssembly/binaryen) project,
which `wasm-pack` will then automatically use to optimize the produced WASM code.
