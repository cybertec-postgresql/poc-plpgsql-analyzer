# Development

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

* A working Rust toolchain using [rustup](https://rustup.rs/) (at least 1.69.0)
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
