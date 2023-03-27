# nexus-compiler

## Project Locations
* Project 1 (Lexer) - project1 branch
* Project 2 (Parser) - project2 branch
* Project 3 (Semantic Analyzer) - main and project3 branches

## Setup Instructions
1. Install Rust, which can be found [here](https://www.rust-lang.org/tools/install).
2. Make sure `cargo` is added to your $PATH environment variable. Its default location is ~/.cargo/bin.
3. Install `wasm-pack` to get all of the WebAssembly utilities for compiling the Rust code by running `cargo install wasm-pack`.
4. Install Node.js, which can be found [here](https://nodejs.org/en/).
5. Make sure Node.js is added to your $PATH environment variable. Its default location is ~/.npm-global/bin.
6. *Recommended:* Install `serve` to be able to start up a simple web server by running `npm i -g serve`. If this is not done, the `make run` command will automatically call `npx serve` to collect the package from the internet.

## Makefile Commands
* `make` / `make build`: Builds Nexus into a WebAssembly module that can be run on the web through JavaScript.
* `make clean`: Removes the files created when the project is built, including the WebAssembly output.
* `make run`: Spins up a basic server to host Nexus. This is required as the current state of WebAssembly requires it to be fetched and it cannot be directly imported to the JavaScript.

* Alan: Run in Chrome.
