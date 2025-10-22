# Development

## Repository Structure
This crate is split into 3 parts. The first one is the repository's root folder, which we'll  be refering to as the "outer"-crate.
The second and third are the "inner"-crates, which are in the "crates"-directory. 

---

Of these the "definitions"-crate contains all code about tokens, what operators and expressions there are etc. If you need to make 
changes to these topics please do so in crates/definitions. 

---
The "source_gen"-crate rarely needs any changes. It contains a build.rs file, which automatically builds new 
source code needed for the project from the definitions crate. 

:warning: The generated.rs files in the "source_gen"-crate needn't
be touched (they'll be overwritten by the next build with changes anyways) :warning:. 

---
All code concerning AST, parsing and such
topics can be found in the src directory of the "outer"-crate. Please make your changes there. 

## Contributing

Contributions are welcome! If you have any ideas, suggestions, or bug reports, please open an issue or submit a pull
request. \
To contribute to this project, follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bug fix:

    ```shell
    git checkout -b feature/my-feature
    ```

3. Make your changes and ensure that the code follows the project's coding conventions.
4. Run the tests to ensure they pass:

    ```shell
    cargo test --features coverage-tests
    ```

5. Commit your changes:

    ```shell
    git commit -m "Add my feature"
    ```

6. Push to your branch:

    ```shell
    git push origin feature/my-feature
    ```

7. Open a pull request on GitHub.

## Requirements

* A working Rust toolchain using [rustup](https://rustup.rs/) (at least 1.90.0)
* [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/) (for building WebAssembly)

## Getting started

To build the application and generate a full npm package in the `pkg/` directory, use

```sh
wasm-pack build --target nodejs
```

## Unit tests

To run all unit tests, use

```sh
cargo test
```

To run unit tests marked with `#[ignore]`, use

```sh
cargo test -- --ignored
```

To run the coverage tests, use

```sh
cargo test -F coverage-tests
```

To run the TypeScript test suite, use

```sh
npm --prefix tests/typescript clean-install
npm --prefix tests/typescript test
```

## Rustdoc

To generate the (public) rustdoc for this library, use:

```sh
cargo doc
```

To generate the full rustdoc for all items, including private ones, use:

```sh
cargo doc --document-private-items
```

## Optional tools

It is endorsed to install the `wasm-opt` tool from the
[`binaryen`](https://github.com/WebAssembly/binaryen) project,
which `wasm-pack` will automatically use to optimize the generated WASM code.
